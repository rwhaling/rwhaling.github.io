use specs::prelude::*;
use super::{CombatStats, CombatIntent, CombatIntents, CombatStance, Name, SufferDamage, gamelog::GameLog};
use rltk::console;

pub struct MeleeCombatSystem {}

impl<'a> System<'a> for MeleeCombatSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = ( Entities<'a>,
                        WriteExpect<'a, GameLog>,
                        WriteStorage<'a, CombatIntent>,
                        ReadStorage<'a, Name>,
                        ReadStorage<'a, CombatStats>,
                        WriteStorage<'a, SufferDamage>,
                        WriteExpect<'a, rltk::RandomNumberGenerator>
                    );

    fn run(&mut self, data : Self::SystemData) {
        let (entities, mut log, mut combat_intents, names, combat_stats, mut inflict_damage, mut rng) = data;

        for (entity, combat_intent, name, stats) in (&entities, &combat_intents, &names, &combat_stats).join() {
            match combat_intent {
                CombatIntent{ intent: CombatIntents::Melee, target: Some(target) } => {
                    // log.entries.push(format!("{} will try to attack", name.name));
                    if stats.hp > 0 {
                        let target_stats = combat_stats.get(*target).unwrap();
                        if stats.ep >= stats.attack_cost {
                            if target_stats.hp > 0 {
                                let target_name = names.get(*target).unwrap();
                                let def_adj = if target_stats.stance == CombatStance::GuardUp { 1 } else { 0 };
                                let eff_def = target_stats.defense + def_adj;
                                let eff_pow = stats.power;
                                let eff_cost = if stats.stance == CombatStance::GuardUp { stats.attack_cost + 5 } else { stats.attack_cost };
                                let damage = damage_formula(&mut rng,eff_pow,eff_def);
                                log.entries.push(format!("{} hits #[orange]{}#[] for #[orange]{} hp#[].", &name.name, &target_name.name, damage));
                                SufferDamage::new_hp_damage(&mut inflict_damage, *target, damage);
                                SufferDamage::new_ep_damage(&mut inflict_damage, entity, eff_cost);
                            }
                        } else {
                            log.entries.push(format!("#[yellow]{}#[] cannot attack, insufficient energy, resting instead", &name.name));
                            rest_or_default(entity, stats, &mut inflict_damage);
                        }
                    }
                }
                CombatIntent{ intent: CombatIntents::StrongMelee, target: Some(target) } => {
                    // log.entries.push(format!("{} will try to STRONG attack", name.name));
                    if stats.hp > 0 {
                        let target_stats = combat_stats.get(*target).unwrap();
                        if stats.ep >= (stats.attack_cost + 10) {
                            if target_stats.hp > 0 {
                                let target_name = names.get(*target).unwrap();
                                let def_adj = if target_stats.stance == CombatStance::GuardUp { 1 } else { 0 };
                                let eff_def = target_stats.defense + def_adj;
                                let eff_pow = stats.power + 1;
                                let eff_cost = if stats.stance == CombatStance::GuardUp { stats.attack_cost + 15 } else { stats.attack_cost + 10 };
                                let damage = damage_formula(&mut rng,eff_pow,eff_def);
                                log.entries.push(format!("{} hits #[orange]{}#[] fiercely for #[orange]{} hp#[].", &name.name, &target_name.name, damage));
                                SufferDamage::new_hp_damage(&mut inflict_damage, *target, damage);
                                SufferDamage::new_ep_damage(&mut inflict_damage, *target, stats.attack_cost + 10);
                                SufferDamage::new_ep_damage(&mut inflict_damage, entity, stats.attack_cost + 10);
                            }
                        } else {
                            log.entries.push(format!("#[yellow]{}#[] cannot attack, insufficient energy, resting instead", &name.name));
                            rest_or_default(entity, stats, &mut inflict_damage);
                        }
                    }
                }
                CombatIntent{ intent: CombatIntents::Wait, target: None } => {
                    rest_or_default(entity, stats, &mut inflict_damage);
                }
                CombatIntent{ intent: CombatIntents::Move, target: None } => {
                    move_regen(entity, stats, &mut inflict_damage);
                }
                _ => {
                    log.entries.push(format!("Anomaly: {} has an incoherent intent", name.name));
                }
            }
        }
        combat_intents.clear();
    }
}

pub fn damage_formula(rng: &mut rltk::RandomNumberGenerator, attacker_pow:i32, target_def:i32) -> i32 {
    let eff_atk = i32::max(0, attacker_pow - target_def);
    let random_atk_max = 2 + ((2_f32 * eff_atk as f32)/3_f32).ceil() as i32;
    let random_atk = rng.range(0,random_atk_max);
    let damage = eff_atk + random_atk;
    return damage
}

pub fn move_regen(entity: Entity, stats: &CombatStats, inflict_damage: &mut WriteStorage<SufferDamage>) {
    // if stats.stance == CombatStance::GuardUp { return; };
    if stats.current_target == None {
        SufferDamage::new_ep_damage(inflict_damage, entity, stats.ep_regen);
    }  else {
        SufferDamage::new_ep_damage(inflict_damage, entity, stats.ep_regen / 2);
    }
}

pub fn rest_or_default(entity: Entity, stats: &CombatStats, inflict_damage: &mut WriteStorage<SufferDamage>) {
    // if stats.stance == CombatStance::GuardUp { return; };
    if stats.current_target == None {
        SufferDamage::new_hp_damage(inflict_damage, entity, stats.hp_regen);
        SufferDamage::new_ep_damage(inflict_damage, entity, stats.ep_regen);
    } 
    SufferDamage::new_ep_damage(inflict_damage, entity, stats.ep_regen);
} 
