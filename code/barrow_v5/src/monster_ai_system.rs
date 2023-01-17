use specs::prelude::*;
use super::{Viewshed, Monster, Map, Position, Action, CombatStats, RunState, SmartMonster};
use super::Command::*;
// use super::AttackMove::*;
use super::WaitMove::*;
use super::CombatStance::*;
use rltk::{Point};

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
                        WriteExpect<'a, rltk::RandomNumberGenerator>
                    );

    fn run(&mut self, data : Self::SystemData) {
        let (mut map, player_entity, runstate, entities, viewsheds, monster, position, mut actions, mut combat_stats, mut smart_monsters, mut rng) = data;

        if *runstate != RunState::MonsterTurn { return; }

        let player_ent_pos = position.get(*player_entity).unwrap();
        let player_pos = Point::new(player_ent_pos.x, player_ent_pos.y);
    
        for (entity, viewshed, _monster, pos, mut stats, mut smart_monster) in (&entities, &viewsheds, &monster, &position, &mut combat_stats, &mut smart_monsters).join() {
            stats.visible_targets.clear();                    

            if smart_monster.target_location == Some(*pos) {
                // log.entries.push(format!("{:?} reached destination, giving up", entity));
                smart_monster.target_location = None;
            }
            if viewshed.visible_tiles.contains(&player_pos) {
                // log.entries.push(format!("{:?} can see you", entity));
                smart_monster.target_location = Some(Position { x: player_pos.x, y: player_pos.y});
                stats.current_target = Some(*player_entity);
                stats.visible_targets.push(*player_entity);
            } else {
                stats.current_target = None;
            }
    
            // TODO: handle Stun properly
            let distance = rltk::DistanceAlg::Pythagoras.distance2d(Point::new(pos.x, pos.y), player_pos);
            if distance < 1.5 {
                if stats.ep >= smart_monster.recover_ep_threshold {
                    actions.insert(entity, Action{ 
                        command: AttackCommand(smart_monster.primary_attack),
                        cost: smart_monster.primary_attack_cost,
                        stance_after: smart_monster.primary_stance,
                        target: Some(*player_entity), 
                        position: None 
                    }).expect("Unable to insert attack");
                } else if stats.ep >= smart_monster.primary_attack_cost {
                    let dice_roll = rng.range(0.0,1.0);
                    if dice_roll < smart_monster.recover_ep_chance { 
                        actions.insert(entity, Action{ 
                            command: WaitCommand(Wait),
                            cost: -10,
                            stance_after: smart_monster.primary_stance,
                            target:None, 
                            position: None 
                        }).expect("Unable to insert move");
                    } else {
                        actions.insert(entity, Action{ 
                            command: AttackCommand(smart_monster.primary_attack),
                            cost: smart_monster.primary_attack_cost,
                            stance_after: smart_monster.primary_stance,
                            target: Some(*player_entity), 
                            position: None 
                        }).expect("Unable to insert attack");        
                    }
                } else {
                    actions.insert(entity, Action{ 
                        command: WaitCommand(Wait),
                        cost: -10,
                        stance_after: smart_monster.primary_stance,
                        target:None, 
                        position: None 
                    }).expect("Unable to insert move");
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
                    actions.insert(entity, Action{ 
                        command: MoveCommand,
                        cost: 0,
                        stance_after: smart_monster.primary_stance,
                        target:None, 
                        position: Some(Position { x: new_x, y: new_y }) 
                    }).expect("Unable to insert move");
                } else {
                    actions.insert(entity, Action{ 
                        // action_type:ActionType::Wait, attack:None, 
                        command: WaitCommand(Wait),
                        cost: -10,
                        stance_after: Ready,
                        target:None, 
                        position: None 
                    }).expect("Unable to insert move");
                }
            } else if smart_monster.target_location != None {
                let targ_loc = smart_monster.target_location.unwrap();
                let path = rltk::a_star_search(
                    map.xy_idx(pos.x, pos.y),
                    map.xy_idx(targ_loc.x, targ_loc.y),
                    &mut *map
                );
                let dice_roll = rng.range(0.0,1.0);
                if path.success && path.steps.len()>1 && dice_roll < smart_monster.invisible_chase_chance {
                    let idx = map.xy_idx(pos.x, pos.y);
                    map.blocked[idx] = false;
                    let new_x = path.steps[1] as i32 % map.width;
                    let new_y = path.steps[1] as i32 / map.width;
                    actions.insert(entity, Action{ 
                        // action_type:ActionType::Move, attack:None, 
                        command: MoveCommand,
                        cost: 0,
                        stance_after: smart_monster.primary_stance,
                        target:None, 
                        position: Some(Position { x: new_x, y: new_y }) 
                    }).expect("Unable to insert move");
                } else {
                    actions.insert(entity, Action{ 
                        // action_type:ActionType::Wait, attack:None, 
                        command: WaitCommand(Wait),
                        cost: -10,
                        stance_after: smart_monster.primary_stance,
                        target:None, 
                        position: None 
                    }).expect("Unable to insert move");
                }

            } else {
                actions.insert(entity, Action{ 
                    // action_type:ActionType::Wait, attack:None, 
                    command: WaitCommand(Wait),
                    cost: -10,
                    stance_after: Ready,
                    target:None, 
                    position: None 
                }).expect("Unable to insert move");
            }        
        }
    }
}
