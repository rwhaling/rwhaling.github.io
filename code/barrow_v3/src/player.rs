use rltk::{VirtualKeyCode, Rltk, Point};
use specs::prelude::*;
use std::cmp::{max, min};
use super::{Position, Player, State, CombatStats, CombatStance, Map, Monster, RunState, Action, ActionType, Attack};

pub fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
    let mut positions = ecs.write_storage::<Position>();
    let players = ecs.read_storage::<Player>();
    let entities = ecs.entities();
    let combat_stats = ecs.read_storage::<CombatStats>();
    let map = ecs.fetch::<Map>();
    let mut actions = ecs.write_storage::<Action>();

    for (entity, _player, pos) in (&entities, &players, &mut positions).join() {
        if pos.x + delta_x < 1 || pos.x + delta_x > map.width-1 || pos.y + delta_y < 1 || pos.y + delta_y > map.height-1 { return; }
        let destination_idx = map.xy_idx(pos.x + delta_x, pos.y + delta_y);

        for potential_target in map.tile_content[destination_idx].iter() {
            let target = combat_stats.get(*potential_target);
            if let Some(_target) = target {
                actions.insert(entity, Action{ action_type: ActionType::Attack, attack: Some(Attack::Melee), target: Some(*potential_target), position: None }).expect("Add target failed");
                return;
            }
        }

        if !map.blocked[destination_idx] {
            let new_x = min(79 , max(0, pos.x + delta_x));
            let new_y = min(49, max(0, pos.y + delta_y));
            actions.insert(entity, Action{ action_type: ActionType::Move, attack:None, target: None, position: Some(Position { x: new_x, y: new_y })}).expect("Move intent failed");
        }
    }
}

pub fn update_targeting(ecs: &World, _ctx: &mut Rltk) {
    let mut combat_stats = ecs.write_storage::<CombatStats>();
    let mut players = ecs.write_storage::<Player>();

    let monsters = ecs.read_storage::<Monster>();
    let positions = ecs.read_storage::<Position>();
    let entities = ecs.entities();
    let map = ecs.fetch::<Map>();

    for (_player, player_stats) in (&mut players, &mut combat_stats).join() {
        let mut current_target_seen = false;
        player_stats.visible_targets.clear();
    
        for (entity, _monster, position) in (&entities, &monsters, &positions).join() {
            let idx = map.xy_idx(position.x, position.y);
            if map.visible_tiles[idx] == true {
                if player_stats.current_target == None {
                    player_stats.current_target = Some(entity);
                    current_target_seen = true;
                } else if player_stats.current_target == Some(entity) {
                    current_target_seen = true;
                }
                player_stats.visible_targets.push(entity);
            }
        }    

        if current_target_seen == false && player_stats.current_target != None {
            player_stats.current_target = None;
        }    
    }
    return; 
}

pub fn try_select_target(selection: usize, ecs: &World) -> RunState {
    let mut combat_stats = ecs.write_storage::<CombatStats>();
    let players = ecs.read_storage::<Player>();

    // console::log(format!("selected target {}", selection));
    for (_player,stats) in (&players, &mut combat_stats).join() {
        if stats.visible_targets.len() >= selection {
            stats.current_target = Some(stats.visible_targets[selection - 1]);
            // console::log(format!("selection ok"));

        } else {
            // console::log(format!("selection failed, only {} targets visible", stats.visible_targets.len()));
        }
    }

    return RunState::AwaitingInput
}

pub fn try_attack_current_target(attack:Attack, ecs: &World) -> RunState {
    let combat_stats = ecs.write_storage::<CombatStats>();
    let mut actions = ecs.write_storage::<Action>();
    let player = ecs.read_storage::<Player>();
    let positions = ecs.read_storage::<Position>();
    let entities = ecs.entities();

    // console::log(format!("trying to attack current target"));
    for (_player,player_entity, stats,player_pos) in (&player, &entities, &combat_stats, &positions).join() {
        match stats.current_target {
            Some(target) => {
                let target_pos = positions.get(target).unwrap();

                let distance = rltk::DistanceAlg::Pythagoras.distance2d(Point::new(target_pos.x, target_pos.y), Point::new(player_pos.x, player_pos.y));
                if distance < 1.5 {
                    actions.insert(player_entity, Action{ action_type: ActionType::Attack, attack:Some(attack), target: Some(target), position: None }).expect("Unable to insert attack");
                    return RunState::PlayerTurn
                } else {
                    // console::log(format!("distance to target {:?} is {}, can't attack", target, distance));
                    return RunState::AwaitingInput
                }
            },
            _ => {
                // console::log(format!("no target selected, can't attack"));
                return RunState::AwaitingInput
            }
        }
    }
    return RunState::AwaitingInput
}

pub fn try_stance_switch(ecs: &World) -> RunState {
    let players = ecs.read_storage::<Player>();
    let mut combat_stats = ecs.write_storage::<CombatStats>();
    // TODO: can this work as an action, while still returning RunState::AwaitingInput?

    for (_player, stats) in (&players, &mut combat_stats).join() {
        match stats.stance {
            CombatStance::GuardUp => {
                stats.stance = CombatStance::GuardDown
            },
            CombatStance::GuardDown => {
                stats.stance = CombatStance::GuardUp
            },
            CombatStance::GuardBreak => {}
        }
    }
    return RunState::AwaitingInput
}

pub fn rest(ecs: &mut World) -> RunState {
    let entities = ecs.entities();
    let players = ecs.read_storage::<Player>();
    let mut actions = ecs.write_storage::<Action>();
    
    for (entity, _player) in (&entities, &players).join() {
        actions.insert(entity, Action{ action_type: ActionType::Wait, attack: None, target:None, position: None }).expect("Rest failed");
    }

    return RunState::PlayerTurn
}

pub fn player_input(gs: &mut State, ctx: &mut Rltk) -> RunState {
    // Player movement
    match ctx.key {
        None => { return RunState::AwaitingInput } // Nothing happened
        Some(key) => match key {
            VirtualKeyCode::Left |
            VirtualKeyCode::Numpad4 |
            VirtualKeyCode::A => try_move_player(-1, 0, &mut gs.ecs),

            VirtualKeyCode::Right |
            VirtualKeyCode::Numpad6 |
            VirtualKeyCode::D => try_move_player(1, 0, &mut gs.ecs),

            VirtualKeyCode::Up |
            VirtualKeyCode::Numpad8 |
            VirtualKeyCode::W => try_move_player(0, -1, &mut gs.ecs),

            VirtualKeyCode::Down |
            VirtualKeyCode::Numpad2 |
            VirtualKeyCode::S => try_move_player(0, 1, &mut gs.ecs),

            // Diagonals
            VirtualKeyCode::Numpad7 |
            VirtualKeyCode::Q => try_move_player(-1, -1, &mut gs.ecs),

            VirtualKeyCode::Numpad9 |
            VirtualKeyCode::E => try_move_player(1, -1, &mut gs.ecs),

            VirtualKeyCode::Numpad3 |
            VirtualKeyCode::C => try_move_player(1, 1, &mut gs.ecs),

            VirtualKeyCode::Numpad1 |
            VirtualKeyCode::Z => try_move_player(-1, 1, &mut gs.ecs),

            // Num Keys
            VirtualKeyCode::Key1 => return try_select_target(1, &gs.ecs),
            VirtualKeyCode::Key2 => return try_select_target(2, &gs.ecs),
            VirtualKeyCode::Key3 => return try_select_target(3, &gs.ecs),
            VirtualKeyCode::Key4 => return try_select_target(4, &gs.ecs),
            VirtualKeyCode::Key5 => return try_select_target(5, &gs.ecs),
            VirtualKeyCode::Key6 => return try_select_target(6, &gs.ecs),
            VirtualKeyCode::Key7 => return try_select_target(7, &gs.ecs),
            VirtualKeyCode::Key8 => return try_select_target(8, &gs.ecs),
            VirtualKeyCode::Key9 => return try_select_target(9, &gs.ecs),

            // Skip
            VirtualKeyCode::Numpad5 => return rest(&mut gs.ecs),
            VirtualKeyCode::Space => return rest(&mut gs.ecs),
            VirtualKeyCode::X => return rest(&mut gs.ecs),

            // Attack
            VirtualKeyCode::J => return try_attack_current_target(Attack::Melee, &gs.ecs),
            VirtualKeyCode::K => return try_attack_current_target(Attack::StrongMelee, &gs.ecs),
            VirtualKeyCode::L => return try_stance_switch(&gs.ecs),

            _ => { return RunState::AwaitingInput }
        },
    }
    RunState::PlayerTurn
}
