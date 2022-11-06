use specs::prelude::*;
use super::{CombatStats, SufferDamage, Player, Name, gamelog::GameLog, RunState};

pub struct DamageSystem {}

impl<'a> System<'a> for DamageSystem {
    type SystemData = ( WriteStorage<'a, CombatStats>,
                        WriteStorage<'a, SufferDamage> );

    fn run(&mut self, data : Self::SystemData) {
        let (mut stats, mut damage) = data;

        for (mut stats, damage) in (&mut stats, &damage).join() {
            stats.hp -= damage.hp_amount.iter().sum::<i32>();
            if (stats.hp >= stats.max_hp) {
                stats.hp = stats.max_hp;
            }
            stats.ep -= damage.ep_amount.iter().sum::<i32>();
            if (stats.ep < 0) {
                stats.ep = 0;
            }
            if (stats.ep >= stats.max_ep) {
                stats.ep = stats.max_ep;
            }
        }

        damage.clear();
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
