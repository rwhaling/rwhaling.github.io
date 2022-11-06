use specs::prelude::*;
use super::{Viewshed, Monster, Map, Position, Action, ActionType, Attack, CombatStats, RunState};
use rltk::{Point};
// use rltk::console;


pub struct MonsterAI {}

impl<'a> System<'a> for MonsterAI {
    #[allow(clippy::type_complexity)]
    type SystemData = ( WriteExpect<'a, Map>,
                        ReadExpect<'a, Entity>,
                        ReadExpect<'a, RunState>,
                        Entities<'a>,
                        ReadStorage<'a, Viewshed>,
                        ReadStorage<'a, Monster>,
                        ReadStorage<'a, Position>,
                        WriteStorage<'a, Action>,
                        WriteStorage<'a, CombatStats>);

    fn run(&mut self, data : Self::SystemData) {
        let (mut map, player_entity, runstate, entities, viewsheds, monster, position, mut combat_intent, mut combat_stats) = data;

        if *runstate != RunState::MonsterTurn { return; }

        let player_ent_pos = position.get(*player_entity).unwrap();
        let player_pos = Point::new(player_ent_pos.x, player_ent_pos.y);
    
        for (entity, viewshed, _monster, pos, mut stats) in (&entities, &viewsheds, &monster, &position, &mut combat_stats).join() {
            stats.visible_targets.clear();

            if viewshed.visible_tiles.contains(&player_pos) {
                stats.current_target = Some(*player_entity);
                stats.visible_targets.push(*player_entity);
            } else {
                stats.current_target = None;
            }
    
            let distance = rltk::DistanceAlg::Pythagoras.distance2d(Point::new(pos.x, pos.y), player_pos);
            if distance < 1.5 {
                combat_intent.insert(entity, Action{ action_type:ActionType::Attack, attack:Some(Attack::Melee), target: Some(*player_entity), position: None }).expect("Unable to insert attack");
            }
            else if viewshed.visible_tiles.contains(&player_pos) {
                // Path to the player
                // console::log(format!("{:?} at {},{} can see player at {},{}", entity, player_pos.x, player_pos.y));
                let path = rltk::a_star_search(
                    map.xy_idx(pos.x, pos.y),
                    map.xy_idx(player_pos.x, player_pos.y),
                    &mut *map
                );
                if path.success && path.steps.len()>1 {

                    let idx = map.xy_idx(pos.x, pos.y);
                    map.blocked[idx] = false;
                    let new_x = path.steps[1] as i32 % map.width;
                    let new_y = path.steps[1] as i32 / map.width;
                    // console::log(format!("{:?} moving to {},{}", entity, new_x, new_y));
                    combat_intent.insert(entity, Action{ action_type:ActionType::Move, attack:None, target:None, position: Some(Position { x: new_x, y: new_y }) }).expect("Unable to insert move");
                    // pos.x = path.steps[1] as i32 % map.width;
                    // pos.y = path.steps[1] as i32 / map.width;
                    // idx = map.xy_idx(pos.x, pos.y);
                    // map.blocked[idx] = true;
                    // viewshed.dirty = true;
                } else {
                    // console::log(format!("{:?} cannot reach player: {:?} | {:?}", entity, path.success, path.steps.len()));

                    combat_intent.insert(entity, Action{ action_type:ActionType::Wait, attack:None, target:None, position: None }).expect("Unable to insert move");
                }
            } else {
                combat_intent.insert(entity, Action{ action_type:ActionType::Wait, attack:None, target:None, position: None }).expect("Unable to insert move");
            }
        }
    }
}
