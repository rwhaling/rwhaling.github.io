use rltk::{VirtualKeyCode, Rltk, Point};
use specs::prelude::*;
use std::cmp::{max, min};
use super::{Position, Player, Viewshed, State, CombatStats, CombatStance, Map, Monster, RunState, CombatIntent, CombatIntents};
use rltk::console;

pub fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
    let mut positions = ecs.write_storage::<Position>();
    let players = ecs.read_storage::<Player>();
    let mut viewsheds = ecs.write_storage::<Viewshed>();
    let entities = ecs.entities();
    let combat_stats = ecs.read_storage::<CombatStats>();
    let map = ecs.fetch::<Map>();
    let mut combat_intent = ecs.write_storage::<CombatIntent>();

    for (entity, _player, pos, viewshed) in (&entities, &players, &mut positions, &mut viewsheds).join() {
        if pos.x + delta_x < 1 || pos.x + delta_x > map.width-1 || pos.y + delta_y < 1 || pos.y + delta_y > map.height-1 { return; }
        let destination_idx = map.xy_idx(pos.x + delta_x, pos.y + delta_y);

        for potential_target in map.tile_content[destination_idx].iter() {
            let target = combat_stats.get(*potential_target);
            if let Some(_target) = target {
                combat_intent.insert(entity, CombatIntent{ intent: CombatIntents::Melee, target: Some(*potential_target) }).expect("Add target failed");
                return;
            }
        }

        if !map.blocked[destination_idx] {
            pos.x = min(79 , max(0, pos.x + delta_x));
            pos.y = min(49, max(0, pos.y + delta_y));
            combat_intent.insert(entity, CombatIntent{ intent: CombatIntents::Move, target: None}).expect("Move intent failed");
            viewshed.dirty = true;
            let mut ppos = ecs.write_resource::<Point>();
            ppos.x = pos.x;
            ppos.y = pos.y;
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
                    console::log("saw new target");
                    player_stats.current_target = Some(entity);
                    current_target_seen = true;
                } else if player_stats.current_target == Some(entity) {
                    // console::log("saw current target");
                    current_target_seen = true;
                }
                player_stats.visible_targets.push(entity);
            }
        }    

        if current_target_seen == false && player_stats.current_target != None {
            player_stats.current_target = None;
            console::log("didn't see current target");
        }    
    }
    return; 
}

pub fn try_select_target(selection: usize, ecs: &World) -> RunState {
    let mut combat_stats = ecs.write_storage::<CombatStats>();
    let mut players = ecs.read_storage::<Player>();

    console::log(format!("selected target {}", selection));
    for (_player,stats) in (&players, &mut combat_stats).join() {
        if stats.visible_targets.len() >= selection {
            stats.current_target = Some(stats.visible_targets[selection - 1]);
            console::log(format!("selection ok"));

        } else {
            console::log(format!("selection failed, only {} targets visible", stats.visible_targets.len()));
        }
    }

    return RunState::AwaitingInput
}

pub fn try_attack_current_target(intent:CombatIntents, ecs: &World) -> RunState {
    let combat_stats = ecs.write_storage::<CombatStats>();
    let mut combat_intent = ecs.write_storage::<CombatIntent>();
    let player = ecs.read_storage::<Player>();
    let mut map = ecs.fetch::<Map>();
    let positions = ecs.read_storage::<Position>();
    let entities = ecs.entities();

    console::log(format!("trying to attack current target"));
    for (_player,player_entity, stats,player_pos) in (&player, &entities, &combat_stats, &positions).join() {
        match stats.current_target {
            Some(target) => {
                let target_pos = positions.get(target).unwrap();

                let distance = rltk::DistanceAlg::Pythagoras.distance2d(Point::new(target_pos.x, target_pos.y), Point::new(player_pos.x, player_pos.y));
                if distance < 1.5 {
                    combat_intent.insert(player_entity, CombatIntent{ intent: intent, target: Some(target) }).expect("Unable to insert attack");
                    return RunState::PlayerTurn
                } else {
                    console::log(format!("distance to target {:?} is {}, can't attack", target, distance));
                    return RunState::AwaitingInput
                }
            },
            _ => {
                console::log(format!("no target selected, can't attack"));
                return RunState::AwaitingInput
            }
        }
    }
    return RunState::AwaitingInput
}

pub fn try_stance_switch(ecs: &World) -> RunState {
    let players = ecs.read_storage::<Player>();
    let mut combat_stats = ecs.write_storage::<CombatStats>();

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
    let mut combat_intent = ecs.write_storage::<CombatIntent>();
    
    for (entity, _player) in (&entities, &players).join() {
        combat_intent.insert(entity, CombatIntent{ intent: CombatIntents::Wait, target: None }).expect("Rest failed");
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
            VirtualKeyCode::J => return try_attack_current_target(CombatIntents::Melee, &gs.ecs),
            VirtualKeyCode::K => return try_attack_current_target(CombatIntents::StrongMelee, &gs.ecs),
            VirtualKeyCode::L => return try_stance_switch(&gs.ecs),

            _ => { return RunState::AwaitingInput }
        },
    }
    RunState::PlayerTurn
}
