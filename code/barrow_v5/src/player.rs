use rltk::{VirtualKeyCode, Rltk, Point, console};
use specs::prelude::*;
use std::cmp::{max, min};
use super::{Position, Player, State, CombatStats, GameLog, Map, Monster, RunState, Action, MenuCommand, Command, TileType };
use super::Command::*;
use super::AttackMove::*;
use super::WaitMove::*;
use super::CombatStance::*;

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
                actions.insert(entity, Action{ 
                    command: AttackCommand(Melee),
                    cost: 0,
                    stance_after: Ready,
                    target: Some(*potential_target), 
                    position: None }).expect("Add target failed");
                return;
            }
        }

        if !map.blocked[destination_idx] {
            let new_x = min(79 , max(0, pos.x + delta_x));
            let new_y = min(49, max(0, pos.y + delta_y));
            actions.insert(entity, Action{ 
                command: MoveCommand,
                cost: -2,
                stance_after: Ready,
                target: None, 
                position: Some(Position { x: new_x, y: new_y })
            }).expect("Move intent failed");
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

pub fn try_descend(ecs: &World) -> RunState {
    let player_entity = ecs.read_resource::<Entity>();
    let positions = ecs.read_storage::<Position>();
    let map = ecs.read_resource::<Map>();
    let mut player_res = ecs.write_storage::<Player>();

    let mut log = ecs.write_resource::<GameLog>();

    let player_pos = positions.get(*player_entity).unwrap();
    let mut player = player_res.get_mut(*player_entity).unwrap();
    console::log(format!("{:?} attempting to descend at {:?}", *player_entity, player_pos));

    let tile_type = map.tiles[map.xy_idx(player_pos.x, player_pos.y)];

    if map.tiles[map.xy_idx(player_pos.x, player_pos.y)] == TileType::StairsDown {
        let next_level = map.depth + 1;
        log.entries.push(format!("You descend deeper into the barrow..."));
        log.entries.push(format!("(loading level {})", next_level));
        player.deepest_level = next_level;

        return RunState::Descend { depth: next_level };    
    } else {
        return RunState::AwaitingInput
    }    
}

pub fn try_ascend(ecs: &World) -> RunState {
    let player_entity = ecs.read_resource::<Entity>();
    let positions = ecs.read_storage::<Position>();
    let map = ecs.read_resource::<Map>();
    let player_res = ecs.read_storage::<Player>();

    let mut log = ecs.write_resource::<GameLog>();

    let player_pos = positions.get(*player_entity).unwrap();
    let player = player_res.get(*player_entity).unwrap();
    console::log(format!("{:?} attempting to ascend at {:?}", *player_entity, player_pos));

    let tile_type = map.tiles[map.xy_idx(player_pos.x, player_pos.y)];

    if map.tiles[map.xy_idx(player_pos.x, player_pos.y)] == TileType::StairsUp {
        if player.has_amulet {
            let next_level = map.depth - 1;
            if next_level > 0 {
                log.entries.push(format!("You ascend toward town, but the amulet's darkness pervades your mind..."));
                log.entries.push(format!("(loading level {})", next_level));
            }
            console::log(format!("ascending to {} with amulet", next_level));
            return RunState::Ascend { depth: next_level }

        } else {
            let next_level = map.depth - 1;
            if next_level > 0 {
                log.entries.push(format!("You retreat from the depths (loading level {})", next_level));
            } else {
                log.entries.push(format!("You ascend toward town, but the barrow beckons you to return..."));
            }
            console::log(format!("ascending to {}", next_level));
            return RunState::Ascend { depth: next_level }
            // return RunState::Shopping { menu_selection: 0 };        
        }
    } else {
        return RunState::AwaitingInput
    }    
}

pub fn try_quick_ascend(ecs: &World) -> RunState {
    let player_entity = ecs.read_resource::<Entity>();
    let positions = ecs.read_storage::<Position>();
    let map = ecs.read_resource::<Map>();
    let player_res = ecs.read_storage::<Player>();
    let mut log = ecs.write_resource::<GameLog>();

    let player_pos = positions.get(*player_entity).unwrap();
    let player = player_res.get(*player_entity).unwrap();
    console::log(format!("{:?} attempting to quick ascend at {:?}", *player_entity, player_pos));

    let tile_type = map.tiles[map.xy_idx(player_pos.x, player_pos.y)];

    if map.tiles[map.xy_idx(player_pos.x, player_pos.y)] == TileType::StairsUp {
        if player.has_amulet {
            log.entries.push(format!("The amulet's darkness is a heavy burden as you return to the barrow's entrance\n(cannot quick ascend, try regular ascend)"));
            console::log(format!("cannot quick ascend"));
            return RunState::AwaitingInput;
        } else {
            log.entries.push(format!("You quickly ascend to town, but the barrow beckons you to return..."));
            console::log(format!("quick ascending to town"));
            return RunState::Ascend { depth: 0 }                
        }
    } else {
        return RunState::AwaitingInput
    }    
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

pub fn get_available_moves(player_stats: &CombatStats) -> Vec<MenuCommand> {
    match player_stats.stance {
        Ready => {
            return vec![
                MenuCommand { command: WaitCommand(Wait), cost: -10, stance_after: Ready, enabled: true },
                MenuCommand { command: AttackCommand(Melee), cost: 0, stance_after: Ready, enabled: true },
                MenuCommand { command: AttackCommand(Smash), cost: 15, stance_after: Power, enabled: true },
                MenuCommand { command: AttackCommand(Bash), cost: 10, stance_after: Guard, enabled: true },
                MenuCommand { command: WaitCommand(Fend), cost: 0, stance_after: Ready, enabled: true },
                MenuCommand { command: WaitCommand(Block), cost: 0, stance_after: Guard, enabled: true }
            ]        
        },
        Power => {
            return vec![
                MenuCommand { command: WaitCommand(Wait), cost: -10, stance_after: Ready, enabled: true },
                MenuCommand { command: AttackCommand(Melee), cost: 0, stance_after: Ready, enabled: true },
                MenuCommand { command: AttackCommand(Smash), cost: 15, stance_after: Power, enabled: true },
                MenuCommand { command: AttackCommand(Bash), cost: 10, stance_after: Guard, enabled: false },
                MenuCommand { command: WaitCommand(Fend), cost: 0, stance_after: Ready, enabled: false },
                MenuCommand { command: WaitCommand(Block), cost: -5, stance_after: Guard, enabled: false }
            ]        
        },
        Guard => {
            return vec![
                MenuCommand { command: WaitCommand(Wait), cost: -10, stance_after: Ready, enabled: true },
                MenuCommand { command: AttackCommand(Melee), cost: 0, stance_after: Ready, enabled: true },
                MenuCommand { command: AttackCommand(Smash), cost: 15, stance_after: Guard, enabled: false },
                MenuCommand { command: AttackCommand(Bash), cost: 10, stance_after: Guard, enabled: true },
                MenuCommand { command: WaitCommand(Fend), cost: 0, stance_after: Ready, enabled: false },
                MenuCommand { command: WaitCommand(Block), cost: 0, stance_after: Guard, enabled: true }
            ]        
        },
        Stun => {
            return vec![
                MenuCommand { command: WaitCommand(Wait), cost: -10, stance_after: Ready, enabled: true },                
                MenuCommand { command: AttackCommand(Melee), cost: 0, stance_after: Ready, enabled: false },
                MenuCommand { command: AttackCommand(Smash), cost: 15, stance_after: Power, enabled: false },
                MenuCommand { command: AttackCommand(Bash), cost: 10, stance_after: Guard, enabled: false },
                MenuCommand { command: WaitCommand(Fend), cost: 0, stance_after: Ready, enabled: false },
                MenuCommand { command: WaitCommand(Block), cost: -5, stance_after: Guard, enabled: false }
            ]        
        }
    }
}

pub fn try_attack_menu(offset:usize, ecs: &World) -> RunState {
    let combat_stats = ecs.write_storage::<CombatStats>();
    let mut actions = ecs.write_storage::<Action>();
    let player = ecs.read_storage::<Player>();
    let positions = ecs.read_storage::<Position>();
    let entities = ecs.entities();

    for (_player,player_entity, stats,player_pos) in (&player, &entities, &combat_stats, &positions).join() {
        let commands = get_available_moves(&stats);
        let selected_command = commands[offset];

        if selected_command.enabled == false {
            return RunState::AwaitingInput;
        }

        match stats.current_target {
            Some(target) => {
                let target_pos = positions.get(target).unwrap();

                let distance = rltk::DistanceAlg::Pythagoras.distance2d(Point::new(target_pos.x, target_pos.y), Point::new(player_pos.x, player_pos.y));
                if distance < 1.5 {
                    let action = match selected_command.command {
                        AttackCommand(a) => { 
                            Action {
                                command: AttackCommand(a),
                                cost: selected_command.cost,
                                stance_after: selected_command.stance_after,
                                target: Some(target),
                                position: None
                            }
                        },
                        WaitCommand(w) => { 
                            Action {
                                command: WaitCommand(w),
                                cost: selected_command.cost,
                                stance_after: selected_command.stance_after,
                                target: None,
                                position: None
                            }                            
                        },
                        Command::MoveCommand => { return RunState::AwaitingInput }
                    };
                    actions.insert(player_entity, action).expect("Unable to insert action");
                    return RunState::PlayerTurn
                } else {
                    match selected_command.command {
                        WaitCommand(w) => { 
                            let action = Action {
                                command: WaitCommand(w),
                                cost: selected_command.cost,
                                stance_after: selected_command.stance_after,
                                target: None,
                                position: None
                            };
                            actions.insert(player_entity, action).expect("Unable to insert action");
                            return RunState::PlayerTurn              
                        },
                        _ => { return RunState::AwaitingInput }
                    }
                }
            },
            None => {
                let commands = get_available_moves(&stats);
                let selected_command = commands[offset];
                let action = match selected_command.command {
                    AttackCommand(_a) => { 
                        // console::log(format!("no target selected, can't attack"));
                        return RunState::AwaitingInput;
                    },
                    WaitCommand(w) => { 
                        Action {
                            command: WaitCommand(w),
                            cost: selected_command.cost,
                            stance_after: selected_command.stance_after,
                            target: None,
                            position: None
                        }                            
                    },
                    Command::MoveCommand => { return RunState::AwaitingInput }
                };
                actions.insert(player_entity, action).expect("Unable to insert action");
                return RunState::PlayerTurn
            }
        }
    }
    return RunState::AwaitingInput

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
            VirtualKeyCode::Numpad5 => return try_attack_menu(0, &gs.ecs),
            VirtualKeyCode::Space => return try_attack_menu(0, &gs.ecs),
            VirtualKeyCode::X => return try_attack_menu(0, &gs.ecs),

            // Attack
            VirtualKeyCode::J => return try_attack_menu(1, &gs.ecs),
            VirtualKeyCode::K => return try_attack_menu(2, &gs.ecs),
            VirtualKeyCode::L => return try_attack_menu(3, &gs.ecs),

            VirtualKeyCode::N => return try_attack_menu(4, &gs.ecs),
            VirtualKeyCode::M => return try_attack_menu(5, &gs.ecs),

            // Ascend
            VirtualKeyCode::Comma => {
                return try_ascend(&gs.ecs)
            },
            VirtualKeyCode::T => {
                return try_quick_ascend(&gs.ecs)
            }

            // Descend
            VirtualKeyCode::Period => return try_descend(&gs.ecs),

            _ => { return RunState::AwaitingInput }
        },
    }
    RunState::PlayerTurn
}
