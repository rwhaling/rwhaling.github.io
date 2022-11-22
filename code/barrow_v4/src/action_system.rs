use specs::prelude::*;
use super::{CombatStats, Action, ActionType, AttackMove, Command, WaitMove, CombatStance, Name, Player, Position, gamelog::GameLog, RunState, Map, Viewshed};
use super::Command::*;
use super::AttackMove::*;
use super::WaitMove::*;
use super::CombatStance::*;

// use rltk::console;

pub struct ActionSystem {}

impl<'a> System<'a> for ActionSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = ( Entities<'a>,
                        ReadExpect<'a, Entity>,
                        WriteExpect<'a, GameLog>,
                        WriteStorage<'a, Action>,
                        ReadStorage<'a, Name>,
                        WriteStorage<'a, CombatStats>,
                        WriteExpect<'a, Map>,
                        WriteStorage<'a, Position>,
                        WriteStorage<'a, Viewshed>,
                        WriteExpect<'a, rltk::RandomNumberGenerator>
                    );

    fn run(&mut self, data : Self::SystemData) {
        let (entities, player_entity, mut log, mut actions, names, mut combat_stats, mut map, mut positions, mut viewsheds, mut rng) = data;

        for (entity, name, action) in (&entities, &names, &actions).join() {
            {
                let mut subject_stats = combat_stats.get_mut(entity).unwrap(); 
                if subject_stats.stance == Stun {
                    log.entries.push(format!("#[red]{} is stunned! Recovering...#[].", &name.name));
    
                    rest_or_default(&mut subject_stats);
                    actions.clear();
                    return
                }    
            }
            match action {
                Action{ command: AttackCommand(a), target: Some(target), cost: ep_cost, .. } => {
                    let subject_stats = combat_stats.get(entity).unwrap();
                    let target_stats = combat_stats.get(*target).unwrap();
                    let target_stance = target_stats.stance;
                    let pow_adj = match (a,target_stance) {
                        (Melee, Guard) => { -1 },
                        (Melee, _ ) => { 0 },
                        (Slash, Guard) => { 0 },
                        (Slash, _ ) => { 2 },
                        (Smash, Guard) => { 1 },
                        (Smash, _ ) => { 3 },
                        (Bash, _ ) => { -1 },
                        (Poke, _ ) => { -1 }
                    };                    
                    let eff_def = target_stats.defense;
                    let eff_pow = subject_stats.power + pow_adj;
                    let raw_damage = damage_formula(&mut rng, eff_pow, eff_def);
                    let current_ep = subject_stats.ep;
                    let target_name = names.get(*target).unwrap();
                    let ep_damage = match (a,target_stance) {
                        (Melee, Guard) => 5,
                        (Slash, Guard) => 5,
                        (Smash, Guard) => 10,
                        (Smash, _) => 5,
                        (Bash, Guard) => 15,
                        (Bash, _) => 10,
                        (Poke, Guard) => 5,
                        (_, _) => 0
                    };
                    if ep_cost > &current_ep {
                        let subject_stats = combat_stats.get_mut(entity).unwrap(); 
                        subject_stats.stance = action.stance_after;
                        rest_or_default( subject_stats);
                    } else {
                        log.entries.push(format!("{} hits #[orange]{}#[] for #[orange]{} hp#[].", &name.name, &target_name.name, raw_damage));
                        {
                            let subject_stats = combat_stats.get_mut(entity).unwrap();
                            apply_ep_damage(subject_stats,*ep_cost);
                            subject_stats.stance = action.stance_after;
                        }
                        {
                            let target_stats = combat_stats.get_mut(*target).unwrap();
                            apply_hp_damage(target_stats, raw_damage);
                            apply_ep_damage(target_stats, ep_damage);
                        }
                    }
                }

                Action{ command: WaitCommand(w), target: None, .. } => {
                    let mut subject_stats = combat_stats.get_mut(entity).unwrap(); 
                    // TODO: wait move ep recovery
                    rest_or_default(&mut subject_stats);
                    if subject_stats.stance != Stun {
                        subject_stats.stance = action.stance_after;
                    }

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
                    // TODO: move ep regen
                    move_regen(&mut subject_stats);
                    subject_stats.stance = action.stance_after;
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
    if stats.stance == CombatStance::Stun && amount > 0 {
    } else {
        stats.ep -= amount;
    }
    if stats.ep < 0 {
        stats.ep = 0;
        stats.stance = CombatStance::Stun;
    }
    if stats.ep >= stats.max_ep {
        stats.ep = stats.max_ep;
        if stats.stance == CombatStance::Stun {
            stats.stance = CombatStance::Ready;
        }
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

pub fn rest_or_default(stats: &mut CombatStats) {
    if stats.stance == CombatStance::Guard { return; };
    if stats.current_target == None {
        apply_hp_damage(stats, stats.hp_regen);
        apply_ep_damage(stats, stats.ep_regen);
    } else {
        apply_ep_damage(stats, stats.ep_regen);
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
        let mut log = ecs.write_resource::<GameLog>();
        for (entity, stats) in (&entities, &combat_stats).join() {
            if stats.hp < 1 {
                let player = players.get(entity);
                match player {
                    None => {
                        let victim_name = names.get(entity);
                        if let Some(victim_name) = victim_name {
                            log.entries.push(format!("#[orange]{}#[] is dead", &victim_name.name));
                        }
                        dead.push(entity)
                    }
                    Some(_) => {
                        let mut runstate = ecs.write_resource::<RunState>();
                        if *runstate != RunState::GameOver {
                            log.entries.push(format!("#[red]You died! #[pink]Press ESCAPE to return to the menu."));

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
