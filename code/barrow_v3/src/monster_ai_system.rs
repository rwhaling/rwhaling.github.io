use specs::prelude::*;
use super::{Viewshed, Monster, Map, Position, Action, ActionType, Attack, GameLog, CombatStats, RunState, SmartMonster};
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
                        WriteStorage<'a, CombatStats>,
                        WriteStorage<'a, SmartMonster>,
                        WriteExpect<'a, GameLog>,
                        WriteExpect<'a, rltk::RandomNumberGenerator>
                    );

    fn run(&mut self, data : Self::SystemData) {
        let (mut map, player_entity, runstate, entities, viewsheds, monster, position, mut actions, mut combat_stats, mut smart_monsters, mut log, mut rng) = data;

        if *runstate != RunState::MonsterTurn { return; }

        let player_ent_pos = position.get(*player_entity).unwrap();
        let player_pos = Point::new(player_ent_pos.x, player_ent_pos.y);
    
        for (entity, viewshed, _monster, pos, mut stats, mut smart_monster) in (&entities, &viewsheds, &monster, &position, &mut combat_stats, &mut smart_monsters).join() {
            match smart_monster.is_smart {
                true => {
                    stats.visible_targets.clear();                    

                    if smart_monster.target_location == Some(*pos) {
                        log.entries.push(format!("{:?} reached destination, giving up", entity));
                        smart_monster.target_location = None;
                    }
                    if viewshed.visible_tiles.contains(&player_pos) {
                        log.entries.push(format!("{:?} can see you", entity));
                        smart_monster.target_location = Some(Position { x: player_pos.x, y: player_pos.y});
                        stats.current_target = Some(*player_entity);
                        stats.visible_targets.push(*player_entity);
                    } else {
                        stats.current_target = None;
                    }
            
                    let distance = rltk::DistanceAlg::Pythagoras.distance2d(Point::new(pos.x, pos.y), player_pos);
                    if distance < 1.5 {
                        if stats.ep >= smart_monster.recover_ep_threshold {
                            actions.insert(entity, Action{ action_type:ActionType::Attack, attack:Some(smart_monster.primary_attack), target: Some(*player_entity), position: None }).expect("Unable to insert attack");
                        } else {
                            actions.insert(entity, Action{ action_type:ActionType::Wait, attack:None, target:None, position: None }).expect("Unable to insert move");
                        }
                    }
                    else if viewshed.visible_tiles.contains(&player_pos) {
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
                            actions.insert(entity, Action{ action_type:ActionType::Move, attack:None, target:None, position: Some(Position { x: new_x, y: new_y }) }).expect("Unable to insert move");
                        } else {
                            actions.insert(entity, Action{ action_type:ActionType::Wait, attack:None, target:None, position: None }).expect("Unable to insert move");
                        }
                    } else if smart_monster.target_location != None {
                        let targ_loc = smart_monster.target_location.unwrap();
                        let path = rltk::a_star_search(
                            map.xy_idx(pos.x, pos.y),
                            map.xy_idx(targ_loc.x, targ_loc.y),
                            &mut *map
                        );
                        // todo: dice roll to follow so you can actually run away - maybe chase_chance?
                        let dice_roll = rng.range(0,smart_monster.chase_chance);
                        if path.success && path.steps.len()>1 && dice_roll == 0 {
                            let idx = map.xy_idx(pos.x, pos.y);
                            map.blocked[idx] = false;
                            let new_x = path.steps[1] as i32 % map.width;
                            let new_y = path.steps[1] as i32 / map.width;
                            actions.insert(entity, Action{ action_type:ActionType::Move, attack:None, target:None, position: Some(Position { x: new_x, y: new_y }) }).expect("Unable to insert move");
                        } else {
                            actions.insert(entity, Action{ action_type:ActionType::Wait, attack:None, target:None, position: None }).expect("Unable to insert move");
                        }

                    } else {
                        actions.insert(entity, Action{ action_type:ActionType::Wait, attack:None, target:None, position: None }).expect("Unable to insert move");
                    }
        
                },
                false => {
                    stats.visible_targets.clear();

                    if viewshed.visible_tiles.contains(&player_pos) {
                        stats.current_target = Some(*player_entity);
                        stats.visible_targets.push(*player_entity);
                    } else {
                        stats.current_target = None;
                    }
            
                    let distance = rltk::DistanceAlg::Pythagoras.distance2d(Point::new(pos.x, pos.y), player_pos);
                    if distance < 1.5 {
                        actions.insert(entity, Action{ action_type:ActionType::Attack, attack:Some(Attack::Melee), target: Some(*player_entity), position: None }).expect("Unable to insert attack");
                    }
                    else if viewshed.visible_tiles.contains(&player_pos) {
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
                            actions.insert(entity, Action{ action_type:ActionType::Move, attack:None, target:None, position: Some(Position { x: new_x, y: new_y }) }).expect("Unable to insert move");
                        } else {
                            actions.insert(entity, Action{ action_type:ActionType::Wait, attack:None, target:None, position: None }).expect("Unable to insert move");
                        }
                    } else {
                        actions.insert(entity, Action{ action_type:ActionType::Wait, attack:None, target:None, position: None }).expect("Unable to insert move");
                    }        
                }
            }
            // stats.visible_targets.clear();

            // if viewshed.visible_tiles.contains(&player_pos) {
            //     stats.current_target = Some(*player_entity);
            //     stats.visible_targets.push(*player_entity);
            // } else {
            //     stats.current_target = None;
            // }
    
            // let distance = rltk::DistanceAlg::Pythagoras.distance2d(Point::new(pos.x, pos.y), player_pos);
            // if distance < 1.5 {
            //     actions.insert(entity, Action{ action_type:ActionType::Attack, attack:Some(Attack::Melee), target: Some(*player_entity), position: None }).expect("Unable to insert attack");
            // }
            // else if viewshed.visible_tiles.contains(&player_pos) {
            //     let path = rltk::a_star_search(
            //         map.xy_idx(pos.x, pos.y),
            //         map.xy_idx(player_pos.x, player_pos.y),
            //         &mut *map
            //     );
            //     if path.success && path.steps.len()>1 {
            //         let idx = map.xy_idx(pos.x, pos.y);
            //         map.blocked[idx] = false;
            //         let new_x = path.steps[1] as i32 % map.width;
            //         let new_y = path.steps[1] as i32 / map.width;
            //         actions.insert(entity, Action{ action_type:ActionType::Move, attack:None, target:None, position: Some(Position { x: new_x, y: new_y }) }).expect("Unable to insert move");
            //     } else {
            //         actions.insert(entity, Action{ action_type:ActionType::Wait, attack:None, target:None, position: None }).expect("Unable to insert move");
            //     }
            // } else {
            //     actions.insert(entity, Action{ action_type:ActionType::Wait, attack:None, target:None, position: None }).expect("Unable to insert move");
            // }
        }
    }
}
