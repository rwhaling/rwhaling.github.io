use specs::prelude::*;
use super::{Viewshed, Monster, Map, Position, CombatIntent, CombatIntents, CombatStats, RunState};
use rltk::{Point};

pub struct MonsterAI {}

impl<'a> System<'a> for MonsterAI {
    #[allow(clippy::type_complexity)]
    type SystemData = ( WriteExpect<'a, Map>,
                        ReadExpect<'a, Point>,
                        ReadExpect<'a, Entity>,
                        ReadExpect<'a, RunState>,
                        Entities<'a>,
                        WriteStorage<'a, Viewshed>,
                        ReadStorage<'a, Monster>,
                        WriteStorage<'a, Position>,
                        WriteStorage<'a, CombatIntent>,
                        WriteStorage<'a, CombatStats>);

    fn run(&mut self, data : Self::SystemData) {
        let (mut map, player_pos, player_entity, runstate, entities, mut viewshed, monster, mut position, mut combat_intent, mut combat_stats) = data;

        if *runstate != RunState::MonsterTurn { return; }

        for (entity, mut viewshed,_monster, mut pos, mut stats) in (&entities, &mut viewshed, &monster, &mut position, &mut combat_stats).join() {
            stats.visible_targets.clear();

            if viewshed.visible_tiles.contains(&*player_pos) {
                stats.current_target = Some(*player_entity);
                stats.visible_targets.push(*player_entity);
            } else {
                stats.current_target = None;
            }
    
            let distance = rltk::DistanceAlg::Pythagoras.distance2d(Point::new(pos.x, pos.y), *player_pos);
            if distance < 1.5 {
                combat_intent.insert(entity, CombatIntent{ intent: CombatIntents::Melee, target: Some(*player_entity) }).expect("Unable to insert attack");
            }
            else if viewshed.visible_tiles.contains(&*player_pos) {
                // Path to the player
                let path = rltk::a_star_search(
                    map.xy_idx(pos.x, pos.y),
                    map.xy_idx(player_pos.x, player_pos.y),
                    &mut *map
                );
                if path.success && path.steps.len()>1 {
                    let mut idx = map.xy_idx(pos.x, pos.y);
                    map.blocked[idx] = false;
                    pos.x = path.steps[1] as i32 % map.width;
                    pos.y = path.steps[1] as i32 / map.width;
                    combat_intent.insert(entity, CombatIntent{ intent: CombatIntents::Move, target: None }).expect("Unable to insert move");
                    idx = map.xy_idx(pos.x, pos.y);
                    map.blocked[idx] = true;
                    viewshed.dirty = true;
                } else {
                    combat_intent.insert(entity, CombatIntent{ intent: CombatIntents::Wait, target: None }).expect("Unable to insert wait");
                }
            } else {
                combat_intent.insert(entity, CombatIntent{ intent: CombatIntents::Wait, target: None }).expect("Unable to insert wait");
            }
        }
    }
}
