use specs::prelude::*;
use super::{CombatStats, Action, WaitMove, CombatStance, Item, Items, Name, Player, Position, gamelog::GameLog, RunState, Map, TileType, Viewshed};
use super::Items::*;
use super::Command::*;
use super::AttackMove::*;
use super::WaitMove::*;
use super::CombatStance::*;

use rltk::console;

pub struct ActionSystem {}

impl<'a> System<'a> for ActionSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = ( Entities<'a>,
                        ReadExpect<'a, Entity>,
                        WriteExpect<'a, GameLog>,
                        WriteStorage<'a, Action>,
                        ReadStorage<'a, Name>,
                        WriteStorage<'a, Player>,
                        WriteStorage<'a, CombatStats>,
                        WriteExpect<'a, Map>,
                        WriteStorage<'a, Position>,
                        WriteStorage<'a, Viewshed>,
                        WriteExpect<'a, rltk::RandomNumberGenerator>,
                        WriteStorage<'a, Item>
                    );

    fn run(&mut self, data : Self::SystemData) {
        let (entities, player_entity, mut log, mut actions, names, mut player, mut combat_stats, mut map, mut positions, mut viewsheds, mut rng, mut items) = data;

        for (entity, name, action) in (&entities, &names, &actions).join() {
            let eff_action: Action;
            {
                // Check here for any conditions that would override the selected action
                let subject_stats = combat_stats.get_mut(entity).unwrap(); 
                if subject_stats.stance == Stun {
                    log.entries.push(format!("#[red]{} is stunned#[], recovering...", &name.name));
                    // TODO: use proper command/regen
                    let a = &Action { command: WaitCommand(Wait), cost: -10, stance_after: subject_stats.stance, target: None, position: None };
                    eff_action = *a;
                    // return
                }    
                else if action.cost > subject_stats.ep {
                    log.entries.push(format!("#[yellow]{}#[] has insufficient ep, recovering...", &name.name));    
                    // TODO: user proper command/regen
                    let a = &Action { command: WaitCommand(Wait), cost: -10, stance_after: subject_stats.stance, target: None, position: None };
                    eff_action = *a;
                    // return
                } else {
                    eff_action = *action;
                }
            }
            match &eff_action {
                // possibly fully refactor each of these into its own fn?
                Action{ command: AttackCommand(a), target: Some(target), cost: ep_cost, .. } => {
                    let subject_stats = combat_stats.get(entity).unwrap();
                    let target_stats = combat_stats.get(*target).unwrap();
                    let target_stance = target_stats.stance;
                    let target_last_command = target_stats.last_command;
                    let pow_adj = match (a,target_stance) {
                        (Melee, Guard) => -1,
                        (Melee, _ ) => 0,
                        (Slash, Guard) => -1,
                        (Slash, _ ) => 1,
                        (Smash, Guard) => 1,
                        (Smash, _ ) => 2,
                        (Bash, Guard ) => 1,
                        (Bash, _ ) => 0,
                        (Bash, Power) => -1,
                        (Poke, _ ) => -1,
                        (_, Stun) => 1
                    };                    
                    let eff_pow = subject_stats.power + pow_adj;
                    let def_adj = match (a, target_last_command) {
                        // TODO: fill out
                        (_, Some(WaitCommand(Block))) => 1,
                        (Smash, Some(WaitCommand(Fend))) => 3,
                        (_, Some(WaitCommand(Fend))) => 1,
                        (_, _) => 0 
                    };
                    let eff_def = target_stats.defense + def_adj;
                    let raw_damage = damage_formula(&mut rng, eff_pow, eff_def);
                    let target_name = names.get(*target).unwrap();

                    let attack_ep_damage = match (a,target_stance) {
                        (Smash, Guard) => 5,
                        (Smash, _) => 5,
                        (Bash, Guard) => 15,
                        (Bash, _) => 10,
                        (_, _) => 0
                    };

                    let attack_verb_string = match a {
                        Melee => "attacks",
                        Smash => "smashes",
                        Bash  => "shield bashes",
                        _     => "attacks"
                    };

                    let reaction_ep_damage = match (a, target_last_command) {
                        // TODO: fill out
                        (Bash, Some(WaitCommand(Block))) => -10,
                        (_, Some(WaitCommand(Block))) => 5,
                        (_, Some(WaitCommand(Fend))) => 0,
                        (_, _) => 0
                    };

                    let ep_damage = attack_ep_damage + reaction_ep_damage;

                    if ep_damage != 0 {
                        log.entries.push(format!("{} {} #[orange]{}#[] for #[orange]{} hp#[] ({} ep).", &name.name, attack_verb_string, &target_name.name, raw_damage, ep_damage));
                    } else {
                        log.entries.push(format!("{} {} #[orange]{}#[] for #[orange]{} hp#[].", &name.name, attack_verb_string, &target_name.name, raw_damage));
                    }

                    match (a, target_stance, target_last_command) {
                        (Bash, Guard, Some(WaitCommand(Block))) => log.entries.push(format!("{}'s block is super effective!", &target_name.name)),
                        (Bash, Guard, _) => log.entries.push(format!("{}'s bash attack is super effective!", &name.name)),
                        (Smash, _, Some(WaitCommand(Fend))) => log.entries.push(format!("{}'s fend is super effective!", &target_name.name)),
                        (_, Stun, _) => log.entries.push(format!("{} is stunned, {}'s attack is super effective", &target_name.name, &name.name)),
                        _ => {}
                    };

                    {
                        let subject_stats = combat_stats.get_mut(entity).unwrap();
                        apply_ep_damage(subject_stats,*ep_cost);
                        subject_stats.stance = action.stance_after;
                        subject_stats.last_command = Some(AttackCommand(*a));
                    }
                    {
                        let target_stats = combat_stats.get_mut(*target).unwrap();
                        apply_hp_damage(target_stats, raw_damage);
                        apply_ep_damage(target_stats, ep_damage);
                    }
                }

                Action{ command: WaitCommand(w), target: None, cost: ep_cost, .. } => {
                    let mut subject_stats = combat_stats.get_mut(entity).unwrap(); 
                    let mut player_inv = player.get_mut(entity);

                    // TODO: wait move ep recovery
                    if *ep_cost != 0 && subject_stats.ep != subject_stats.max_ep {
                        let ep_string = format!("{}", *ep_cost).replace("-","");
                        log.entries.push(format!("{} recovers {} ep.", &name.name, ep_string));
                    }
                    rest_or_default(&mut subject_stats, *w, *ep_cost, player_inv);
                    if subject_stats.stance != Stun {
                        subject_stats.stance = action.stance_after;
                    }
                    subject_stats.last_command = Some(WaitCommand(*w));

                }
                Action{ command: MoveCommand, target: None, position: Some(Position {x,y}), .. } => {
                    let mut pos = positions.get_mut(entity).unwrap();
                    let mut viewshed = viewsheds.get_mut(entity).unwrap();
                    pos.x = *x;
                    pos.y = *y;

                    let idx = map.xy_idx(pos.x, pos.y);
                    if entity != *player_entity {
                        map.blocked[idx] = true;
                    }
                    viewshed.dirty = true;

                    let mut subject_stats = combat_stats.get_mut(entity).unwrap(); 
                    // TODO: move ep regen?
                    move_regen(&mut subject_stats);
                    subject_stats.stance = action.stance_after;
                    subject_stats.last_command = Some(MoveCommand);

                    // check new tile contents
                    let mut p = player.get_mut(entity);
                    if p.is_some() {
                        let mut player_inv = p.unwrap();
                        let contents = &map.tile_content[map.xy_idx(*x,*y)];
                        for c in contents {
                            let i = items.get(*c);
                            console::log(format!("tile contents: {:?}", i));
                            match i { 
                                Some(Item { item: Coin(i) } ) => { 
                                    log.entries.push(format!("You pick up {} coins from the ground.", i));
                                    player_inv.coin = player_inv.coin + i;
                                    entities.delete(*c).expect("Unable to delete");
                                }
                                _ => {}
                            }
                        }
                    }
                }
                _ => {
                    log.entries.push(format!("Anomaly: {} has an incoherent intent", name.name));
                }
            }
        }
        actions.clear();
    }
}


pub fn apply_hp_damage( stats: &mut CombatStats, amount: i32) {
    stats.hp -= amount;
    if stats.hp >= stats.max_hp {
        stats.hp = stats.max_hp;
    }
}

pub fn apply_ep_damage( stats: &mut CombatStats, amount: i32) {
    if stats.stance == CombatStance::Stun {
        if stats.ep < 0 && amount < 0 { 
            stats.ep = 0;
        } else if amount < 0 {
            stats.ep -= amount;
            stats.stance = CombatStance::Ready
        }
    } else {
        stats.ep -= amount;        
        if stats.ep < 0 {
            stats.stance = CombatStance::Stun;
        }    
    } 
    if stats.ep >= stats.max_ep {
        stats.ep = stats.max_ep;
    }
}

pub fn damage_formula(rng: &mut rltk::RandomNumberGenerator, attacker_pow:i32, target_def:i32) -> i32 {
    let eff_atk = i32::max(0, attacker_pow - target_def);
    let random_atk_max = 2 + ((2_f32 * eff_atk as f32)/3_f32).ceil() as i32;
    let random_atk = rng.range(0,random_atk_max);
    let damage = eff_atk + random_atk;
    return damage
}

pub fn move_regen(stats: &mut CombatStats) {
    if stats.stance == CombatStance::Guard { return; };
    if stats.current_target == None {
        apply_ep_damage(stats, stats.ep_regen);
    }  else {
        apply_ep_damage(stats, stats.ep_regen / 2);
    }
}

pub fn rest_or_default(stats: &mut CombatStats, _wait_move: WaitMove, cost: i32, player: Option<&mut Player>) {
    if stats.current_target == None && player.is_some() && stats.hp < stats.max_hp {
        let p = player.unwrap();
        apply_hp_damage(stats, stats.hp_regen);
        apply_ep_damage(stats, cost);
        // todo: checks, etc.
        p.food = p.food - 1;
    } else {
        apply_ep_damage(stats, cost);
    }
} 

pub fn delete_the_dead(ecs : &mut World) {
    let mut dead : Vec<Entity> = Vec::new();
    // Using a scope to make the borrow checker happy
    {
        let combat_stats = ecs.read_storage::<CombatStats>();
        let players = ecs.read_storage::<Player>();
        let names = ecs.read_storage::<Name>();
        let entities = ecs.entities();
        let positions = ecs.read_storage::<Position>();
        let map = ecs.read_resource::<Map>();
        let mut log = ecs.write_resource::<GameLog>();
        for (entity, stats, position) in (&entities, &combat_stats, &positions).join() {
            let player = players.get(entity);
            if stats.hp < 1 {
                match player {
                    None => {
                        let victim_name = names.get(entity);
                        if let Some(victim_name) = victim_name {
                            log.entries.push(format!("#[orange]{}#[] is dead", &victim_name.name));
                        }
                        dead.push(entity)
                    }
                    Some(_) => {
                        let tile_type = map.tiles[map.xy_idx(position.x, position.y)];
                        let mut runstate = ecs.write_resource::<RunState>();
                        if *runstate != RunState::GameOver {
                            log.entries.push(format!("#[red]You died! "));
                            log.entries.push(format!("#[pink]Press ESCAPE to return to the menu."));
                            *runstate = RunState::GameOver;
                        }                         
                    }
                }
            }
        }
    }

    for victim in dead {
        ecs.delete_entity(victim).expect("Unable to delete");
    }
}
