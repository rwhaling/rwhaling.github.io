use specs::prelude::*;
use super::{CombatStats, Action, ActionType, Attack, CombatStance, Name, Player, gamelog::GameLog, RunState};
use rltk::console;

pub struct MeleeCombatSystem {}

impl<'a> System<'a> for MeleeCombatSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = ( Entities<'a>,
                        WriteExpect<'a, GameLog>,
                        WriteStorage<'a, Action>,
                        ReadStorage<'a, Name>,
                        WriteStorage<'a, CombatStats>,
                        WriteExpect<'a, rltk::RandomNumberGenerator>
                    );

    fn run(&mut self, data : Self::SystemData) {
        let (entities, mut log, mut combat_intents, names, mut combat_stats, mut rng) = data;

        for (entity, name, combat_intent) in (&entities, &names, &combat_intents).join() {
            match combat_intent {
                Action{ action_type: ActionType::Attack, attack: Some(Attack::Melee), target: Some(target) } => {
                    let raw_damage: i32;
                    let ep_cost: i32;
                    let current_ep: i32;
                    {
                        let subject_stats = combat_stats.get(entity).unwrap();
                        let target_stats = combat_stats.get(*target).unwrap();
                        let def_adj = if target_stats.stance == CombatStance::GuardUp { 1 } else { 0 };
                        let eff_def = target_stats.defense + def_adj;
                        let eff_pow = subject_stats.power;
                        current_ep = subject_stats.ep;
                        ep_cost = if subject_stats.stance == CombatStance::GuardUp { subject_stats.attack_cost + 5 } else { subject_stats.attack_cost };
                        raw_damage = damage_formula(&mut rng,eff_pow,eff_def);
                    }
                    if (current_ep < ep_cost) {
                        let mut subject_stats = combat_stats.get_mut(entity).unwrap(); 

                        rest_or_default(&mut subject_stats);
                    } else {
                        let target_name = names.get(*target).unwrap();
                        log.entries.push(format!("{} hits #[orange]{}#[] for #[orange]{} hp#[].", &name.name, &target_name.name, raw_damage));
                        {
                            let mut subject_stats = combat_stats.get_mut(entity).unwrap();
                            apply_ep_damage(subject_stats,ep_cost);
                        }
                        {
                            let mut target_stats = combat_stats.get_mut(*target).unwrap();
                            apply_hp_damage(&mut target_stats, raw_damage);
                        }
                    }
                }
                Action{ action_type: ActionType::Attack, attack: Some(Attack::StrongMelee), target: Some(target) } => {
                    // log.entries.push(format!("{} will try to STRONG attack", name.name));
                    let raw_damage: i32;
                    let ep_damage: i32;
                    let ep_cost: i32;
                    let current_ep: i32;
                    {
                        let subject_stats = combat_stats.get(entity).unwrap();
                        let target_stats = combat_stats.get(*target).unwrap();

                        let def_adj = if target_stats.stance == CombatStance::GuardUp { 1 } else { 0 };
                        let eff_def = target_stats.defense + def_adj;
                        let eff_pow = subject_stats.power + 1;
                        current_ep = subject_stats.ep;
                        ep_cost = if subject_stats.stance == CombatStance::GuardUp { subject_stats.attack_cost + 15 } else { subject_stats.attack_cost + 10 };
                        raw_damage = damage_formula(&mut rng,eff_pow,eff_def);
                        ep_damage = subject_stats.attack_cost + 10;
                    }
                    if (current_ep < ep_cost) {
                        let mut subject_stats = combat_stats.get_mut(entity).unwrap(); 
                        rest_or_default(&mut subject_stats);
                    } else {
                        let target_name = names.get(*target).unwrap();
                        log.entries.push(format!("{} hits #[orange]{}#[] fiercely for #[orange]{} hp#[].", &name.name, &target_name.name, raw_damage));
                        {
                            let mut subject_stats = combat_stats.get_mut(entity).unwrap();
                            apply_ep_damage(subject_stats,ep_cost);
                        }
                        {
                            let mut target_stats = combat_stats.get_mut(*target).unwrap();
                            apply_hp_damage(&mut target_stats, raw_damage);
                            apply_ep_damage(&mut target_stats, ep_damage);
                        }
                    }
                }
                Action{ action_type: ActionType::Wait, attack: None, target: None } => {
                    let mut subject_stats = combat_stats.get_mut(entity).unwrap(); 
                    rest_or_default(&mut subject_stats);
                }
                Action{ action_type: ActionType::Move, attack: None, target: None } => {
                    let mut subject_stats = combat_stats.get_mut(entity).unwrap(); 
                    move_regen(&mut subject_stats);
                }
                _ => {
                    log.entries.push(format!("Anomaly: {} has an incoherent intent", name.name));
                }
            }
        }
        combat_intents.clear();
    }
}


pub fn apply_hp_damage( stats: &mut CombatStats, amount: i32) {
    stats.hp -= amount;
    if stats.hp >= stats.max_hp {
        stats.hp = stats.max_hp;
    }
}

pub fn apply_ep_damage( stats: &mut CombatStats, amount: i32) {
    stats.ep -= amount;
    if (stats.ep < 0) {
        stats.ep = 0;
    }
    if (stats.ep >= stats.max_ep) {
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
    if stats.stance == CombatStance::GuardUp { return; };
    if stats.current_target == None {
        apply_ep_damage(stats, stats.ep_regen);
    }  else {
        apply_ep_damage(stats, stats.ep_regen / 2);
    }
}

pub fn rest_or_default(stats: &mut CombatStats) {
    if stats.stance == CombatStance::GuardUp { return; };
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
