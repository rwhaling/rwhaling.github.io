use rltk::{VirtualKeyCode, Rltk, Point};
use specs::prelude::*;
use std::cmp::{max, min};
use super::{Position, Player, Viewshed, State, CombatStats, Map, Monster, RunState, WantsToMelee,};
use rltk::console;

pub fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
    let mut positions = ecs.write_storage::<Position>();
    let players = ecs.read_storage::<Player>();
    let mut viewsheds = ecs.write_storage::<Viewshed>();
    let entities = ecs.entities();
    let combat_stats = ecs.read_storage::<CombatStats>();
    let map = ecs.fetch::<Map>();
    let mut wants_to_melee = ecs.write_storage::<WantsToMelee>();

    for (entity, _player, pos, viewshed) in (&entities, &players, &mut positions, &mut viewsheds).join() {
        if pos.x + delta_x < 1 || pos.x + delta_x > map.width-1 || pos.y + delta_y < 1 || pos.y + delta_y > map.height-1 { return; }
        let destination_idx = map.xy_idx(pos.x + delta_x, pos.y + delta_y);

        for potential_target in map.tile_content[destination_idx].iter() {
            let target = combat_stats.get(*potential_target);
            if let Some(_target) = target {
                wants_to_melee.insert(entity, WantsToMelee{ target: *potential_target }).expect("Add target failed");
                return;
            }
        }

        if !map.blocked[destination_idx] {
            pos.x = min(79 , max(0, pos.x + delta_x));
            pos.y = min(49, max(0, pos.y + delta_y));

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
        // player_current_target = stats.target;
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

pub fn player_input(gs: &mut State, ctx: &mut Rltk) -> RunState {
    // Player movement
    match ctx.key {
        None => { return RunState::AwaitingInput } // Nothing happened
        Some(key) => match key {
            VirtualKeyCode::Left |
            VirtualKeyCode::Numpad4 |
            VirtualKeyCode::A |
            VirtualKeyCode::H => try_move_player(-1, 0, &mut gs.ecs),

            VirtualKeyCode::Right |
            VirtualKeyCode::Numpad6 |
            VirtualKeyCode::D |
            VirtualKeyCode::L => try_move_player(1, 0, &mut gs.ecs),

            VirtualKeyCode::Up |
            VirtualKeyCode::Numpad8 |
            VirtualKeyCode::W |
            VirtualKeyCode::K => try_move_player(0, -1, &mut gs.ecs),

            VirtualKeyCode::Down |
            VirtualKeyCode::Numpad2 |
            VirtualKeyCode::S |
            VirtualKeyCode::J => try_move_player(0, 1, &mut gs.ecs),

            // Diagonals
            VirtualKeyCode::Numpad7 |
            VirtualKeyCode::Q |
            VirtualKeyCode::Y => try_move_player(-1, -1, &mut gs.ecs),

            VirtualKeyCode::Numpad9 |
            VirtualKeyCode::E |
            VirtualKeyCode::U => try_move_player(1, -1, &mut gs.ecs),

            VirtualKeyCode::Numpad3 |
            VirtualKeyCode::C |
            VirtualKeyCode::N => try_move_player(1, 1, &mut gs.ecs),

            VirtualKeyCode::Numpad1 |
            VirtualKeyCode::Z |
            VirtualKeyCode::B => try_move_player(-1, 1, &mut gs.ecs),

            // Skip
            VirtualKeyCode::Numpad5 => return RunState::PlayerTurn,
            VirtualKeyCode::Space => return     RunState::PlayerTurn,

            _ => { return RunState::AwaitingInput }
        },
    }
    RunState::PlayerTurn
}
