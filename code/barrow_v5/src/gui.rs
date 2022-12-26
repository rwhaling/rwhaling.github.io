use rltk::{ RGB, RGBA, Rltk, Point, VirtualKeyCode };
use rltk::console;
use specs::prelude::*;
use super::player::get_available_moves;
use super::{CombatStats, Player, Monster, gamelog::GameLog, Map, Name, Position, RunState, State, Command, MenuCommand};
use super::Command::*;
use super::AttackMove::*;
use super::WaitMove::*;
use super::CombatStance::*;
use bracket_terminal::prelude::TextAlign;

#[derive(PartialEq, Copy, Clone)]
pub enum MainMenuSelection { NewGame, Quit }

#[derive(PartialEq, Copy, Clone)]
pub enum MainMenuResult { NoSelection{ selected : MainMenuSelection }, Selected{ selected: MainMenuSelection } }

#[derive(PartialEq, Copy, Clone)]
pub enum GameOverResult { NoSelection, QuitToMenu }

// pub fn draw_stat_bar(text, current_stat, max_stat, x, y, w, text_color,ctx: &mut Rltk)

pub fn draw_ui(ecs: &World, ctx : &mut Rltk) {
    // ctx.draw_box(0, 43, 49, 16, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK));
    // ctx.draw_box(50, 0, 29, 42, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK));

    let combat_stats = ecs.read_storage::<CombatStats>();
    let players = ecs.read_storage::<Player>();

    let monsters = ecs.read_storage::<Monster>();
    let names = ecs.read_storage::<Name>();
    let positions = ecs.read_storage::<Position>();
    let entities = ecs.entities();
    let map = ecs.fetch::<Map>();

    let mut target_offset = 1;
    let mut gui_offset = 2;

    let mouse_pos = ctx.mouse_pos();

    ctx.set_active_console(1);
    ctx.cls();
    let menu_mouse_pos = ctx.mouse_pos();
    let menu_y = menu_mouse_pos.1;
    let mut info_popup : Option<String> = None;

    for (player, stats) in (&players, &combat_stats).join() {
        let health = format!("HP:{}/{} ", stats.hp, stats.max_hp);
        let name = format!("Player");
        ctx.print_color(51, 1, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), &name);
        ctx.draw_bar_horizontal(65, 1, 16, stats.hp, stats.max_hp, RGB::named(rltk::RED), RGB::named(rltk::BLACK));
        ctx.print_color(68, 1, RGB::named(rltk::WHITE), RGBA::from_f32(0.0,0.0,0.0,0.0), &health);
        let stance = format!("Stance: {:?}", stats.stance);
        // console::log(format!("current stance: {:?}", stats.stance));
        match stats.stance {
            Ready => ctx.print_color(51, 2, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), &stance),
            Power => ctx.print_color(51, 2, RGB::named(rltk::CYAN), RGB::named(rltk::BLACK), &stance),
            Guard => ctx.print_color(51, 2, RGB::named(rltk::GREEN), RGB::named(rltk::BLACK), &stance),
            Stun => ctx.print_color(51, 2, RGB::named(rltk::RED), RGB::named(rltk::BLACK), &stance),
            // _ => ctx.print_color(51, 2, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), &stance)
        };

        let energy = format!("EP:{}/{} ", stats.ep, stats.max_ep);
        ctx.draw_bar_horizontal(65, 2, 16, stats.ep, stats.max_ep, RGB::named(rltk::BLUE), RGB::named(rltk::BLACK));
        ctx.print_color(68, 2, RGB::named(rltk::WHITE), RGBA::from_f32(0.0,0.0,0.0,0.0), &energy);

        let items = format!("Food:   {}     Coin: {}", &player.food, &player.coin); 
        ctx.print_color(51, 3, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), &items );

        if menu_y == 1 || menu_y == 2 || menu_y == 3 {
            info_popup = Some(monster_tooltip(&name));
        }

        for (entity, _monster, monster_stats, name, position) in (&entities, &monsters, &combat_stats, &names, &positions).join() {
            let idx = map.xy_idx(position.x, position.y);
            if map.visible_tiles[idx] == true {
                if stats.visible_targets.contains(&entity) {
                    if stats.current_target == Some(entity) {
                        let target_string = format!("{}){}", target_offset, name.name);
                        ctx.print_color(51, 3 + gui_offset, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), target_string);
                        ctx.draw_bar_horizontal(65, 3 + gui_offset, 16, monster_stats.hp, monster_stats.max_hp, RGB::named(rltk::RED), RGB::named(rltk::BLACK));
                        let health = format!("HP:{}/{}", monster_stats.hp, monster_stats.max_hp);
                        ctx.print_color(68, 3 + gui_offset, RGB::named(rltk::WHITE), RGBA::from_f32(0.0,0.0,0.0,0.0), &health);
                        let monster_stance = format!("Stance: {:?}", monster_stats.stance);
                        match monster_stats.stance {
                            Ready => ctx.print_color(51, 4 + gui_offset, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), monster_stance),
                            Power => ctx.print_color(51, 4 + gui_offset, RGB::named(rltk::CYAN), RGB::named(rltk::BLACK), monster_stance),
                            Guard => ctx.print_color(51, 4 + gui_offset, RGB::named(rltk::GREEN), RGB::named(rltk::BLACK), monster_stance),
                            Stun => ctx.print_color(51, 4 + gui_offset, RGB::named(rltk::RED), RGB::named(rltk::BLACK), monster_stance),
                            // _ => ctx.print_color(51, 4 + gui_offset, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), monster_stance)
                
                        }
                        ctx.draw_bar_horizontal(65, 4 + gui_offset, 16, monster_stats.ep, monster_stats.max_ep, RGB::named(rltk::BLUE), RGB::named(rltk::BLACK));
                        let energy = format!("EP:{}/{} ", monster_stats.ep, monster_stats.max_ep);
                        ctx.print_color(68, 4 + gui_offset, RGB::named(rltk::WHITE), RGBA::from_f32(0.0,0.0,0.0,0.0), &energy);

                        if menu_y == 3 + gui_offset || menu_y == 4 + gui_offset {
                            info_popup = Some(monster_tooltip(&name.name));
                        }
                        target_offset += 1;
                        gui_offset += 2;
                    } else {
                        let target_string = format!("{} {}", target_offset, name.name);
                        ctx.print(51, 3 + gui_offset, target_string);
                        if menu_y == 3 + gui_offset {
                            info_popup = Some(monster_tooltip(&name.name));
                        }
                        target_offset += 1;
                        gui_offset += 1;
                    }
                }
            }
        }
        gui_offset += 4;
        ctx.print(51, 0 + gui_offset, format!("                            "));
        ctx.print(51, 1 + gui_offset, format!("Commands             EP Cost"));
        ctx.print(51, 2 + gui_offset, format!("--------             -------"));
        ctx.print(51, 3 + gui_offset, format!("(ASDW) Move                 "));
        ctx.print(51, 4 + gui_offset, format!("(QEZC) Diag. Move           "));
        ctx.print(51, 5 + gui_offset, format!("(.)    Descend              "));

        gui_offset += 6;

        let moves : Vec<MenuCommand> = get_available_moves(&stats);
        let mut move_offset = 0;
        let move_keys : Vec<&str> = vec!["(X/Sp)","(J)","(K)","(L)","(N)","(M)"];
        for (i,m) in moves.iter().enumerate() {
            if m.enabled == true {
                ctx.printer(51, gui_offset + move_offset, format!("#[white]{:6} {}       ", move_keys[i], print_command(&m)), TextAlign::Left,Some(RGBA::named(rltk::BLACK)));
            } else {
                // console::log(format!("{:?}",m));
                ctx.printer(51, gui_offset + move_offset, format!("#[grey]{:6} {}       ", move_keys[i], print_command(&m)), TextAlign::Left,Some(RGBA::named(rltk::BLACK)));

            }
            if menu_y == gui_offset + move_offset {
                info_popup = Some(command_tooltip(&m));
            }
            move_offset += 1;           
        }
    }
    // ctx.target(0);

    let log = ecs.fetch::<GameLog>();
    // ctx.set_active_console(1);
    // ctx.cls();
    let mut y = 29;
    for s in log.entries.iter().rev() {
        if y >= 21 { 
            ctx.printer(1, y, format!("#[white]{}",s), TextAlign::Left,Some(RGBA::named(rltk::BLACK))); 
        }
        y -= 1;
    }
    ctx.set_active_console(0);

    // Draw mouse cursor
    ctx.set_bg(mouse_pos.0, mouse_pos.1, RGB::named(rltk::MAGENTA));
    draw_tooltips(ecs, ctx);

    ctx.set_active_console(1);

    // Draw mouse cursor
    if menu_mouse_pos.0 > 50 && info_popup != None {
        ctx.set_bg(menu_mouse_pos.0, menu_mouse_pos.1, RGB::named(rltk::MAGENTA));
        ctx.draw_box(8,1,34,18,rltk::WHITE,rltk::BLACK);
        let popup_text = info_popup.unwrap();
        let popup_lines: Vec<&str> = popup_text.split('\n').collect();
        for (i,line) in popup_lines.iter().enumerate() {
            ctx.print_color(9,2 + i, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), line);
        }
    }
}

fn monster_tooltip(name: &String) -> String {
    return match name.as_str() {
        "Player" => String::from("Player\nThis is you."),
        "Goblin" => String::from("Goblin\nWeak and cowardly\nBlocking is very effective"),
        "Orc" => String::from("Orc\nAttacks fiercely, easily tired.\nFend is very effective."),
        "Goblin Knight" => String::from("Goblin Knight\nFormidable attack and defense.\nVulnerable when stamina is low\nGuard stance is vulnerable to \nsmash attacks"),
        _ => name.clone()
    }
}

fn command_tooltip(command: &MenuCommand) -> String {
    let command_str = match command.command {
        AttackCommand(Melee) => { format!("Melee attack\nZero cost, low damage.\nEasily blocked or fended.")},
        AttackCommand(Slash) => { format!("Slash\nPowerful attack, no stamina damage")},
        AttackCommand(Smash) => { format!("Smash\nHigh cost\nVery Powerful attack\nDamages stamina\nResisted by Fend")},
        AttackCommand(Bash) => { format!("Bash\nHigh cost\nDamages stamina\nResisted by Block\n")},
        AttackCommand(Poke) => { format!("Poke\nModerate cost\nMaintains guard.")},

        WaitCommand(Wait) => { format!("Wait\nRecover 10 EP\nRecover HP if not in combat") },
        WaitCommand(Fend) => { format!("Fend\nZero cost\nModerate defense bonus\nHighly effective against Smash") },
        WaitCommand(Block) => { format!("Block\nHighly resilient.\nWeak against Smash\nStrong against Bash") },
        WaitCommand(Brace) => { format!("Brace\nTake the hit.\nRecover 5 EP\nRemain in Power Stance") },

        MoveCommand => { format!("") }
    };
    return command_str
}

fn print_command(command: &MenuCommand) -> String {
    let command_str = match command.command {
        
        Command::AttackCommand(a) => { format!("{:?}",a) },
        Command::WaitCommand(w) => { format!("{:?}",w) },
        Command::MoveCommand => { format!("") }
    };
    let cost_str = format!("{:3}",command.cost).replace("-","+");
    return format!("{:5} > {:?}  ({})", command_str, command.stance_after, cost_str)
}

fn draw_tooltips(ecs: &World, ctx : &mut Rltk) {
    let map = ecs.fetch::<Map>();
    let names = ecs.read_storage::<Name>();
    let positions = ecs.read_storage::<Position>();

    let mouse_pos = ctx.mouse_pos();
    if mouse_pos.0 >= map.width || mouse_pos.1 >= map.height { return; }
    let mut tooltip : Vec<String> = Vec::new();
    for (name, position) in (&names, &positions).join() {
        let idx = map.xy_idx(position.x, position.y);
        if position.x == mouse_pos.0 && position.y == mouse_pos.1 && map.visible_tiles[idx] {
            tooltip.push(name.name.to_string());
        }
    }

    if !tooltip.is_empty() {
        let mut width :i32 = 0;
        for s in tooltip.iter() {
            if width < s.len() as i32 { width = s.len() as i32; }
        }
        width += 3;

        if mouse_pos.0 > 40 {
            let arrow_pos = Point::new(mouse_pos.0 - 2, mouse_pos.1);
            let left_x = mouse_pos.0 - width;
            let mut y = mouse_pos.1;
            for s in tooltip.iter() {
                ctx.print_color(left_x, y, RGB::named(rltk::WHITE), RGB::named(rltk::GREY), s);
                let padding = (width - s.len() as i32)-1;
                for i in 0..padding {
                    ctx.print_color(arrow_pos.x - i, y, RGB::named(rltk::WHITE), RGB::named(rltk::GREY), &" ".to_string());
                }
                y += 1;
            }
            ctx.print_color(arrow_pos.x, arrow_pos.y, RGB::named(rltk::WHITE), RGB::named(rltk::GREY), &"->".to_string());
        } else {
            let arrow_pos = Point::new(mouse_pos.0 + 1, mouse_pos.1);
            let left_x = mouse_pos.0 +3;
            let mut y = mouse_pos.1;
            for s in tooltip.iter() {
                ctx.print_color(left_x + 1, y, RGB::named(rltk::WHITE), RGB::named(rltk::GREY), s);
                let padding = (width - s.len() as i32)-1;
                for i in 0..padding {
                    ctx.print_color(arrow_pos.x + 1 + i, y, RGB::named(rltk::WHITE), RGB::named(rltk::GREY), &" ".to_string());
                }
                y += 1;
            }
            ctx.print_color(arrow_pos.x, arrow_pos.y, RGB::named(rltk::WHITE), RGB::named(rltk::GREY), &"<-".to_string());
        }
    }
}

pub fn main_menu(gs : &mut State, ctx : &mut Rltk) -> MainMenuResult {
    let runstate = gs.ecs.fetch::<RunState>();

    ctx.set_active_console(1);
    ctx.print_color_centered(8, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), "Barrow");

    if let RunState::MainMenu{ menu_selection : selection } = *runstate {
        if selection == MainMenuSelection::NewGame {
            ctx.print_color_centered(12, RGB::named(rltk::MAGENTA), RGB::named(rltk::BLACK), "Begin New Game (press Enter)");
        } else {
            ctx.print_color_centered(12, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), "Begin New Game");
        }

        if selection == MainMenuSelection::Quit {
            ctx.print_color_centered(13, RGB::named(rltk::MAGENTA), RGB::named(rltk::BLACK), "Quit (press Enter)");
        } else {
            ctx.print_color_centered(13, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), "Quit");
        }

        match ctx.key {
            None => return MainMenuResult::NoSelection{ selected: selection },
            Some(key) => {
                match key {
                    VirtualKeyCode::Space => { return MainMenuResult::Selected{ selected: MainMenuSelection::NewGame } }
                    VirtualKeyCode::Up => {
                        let newselection;
                        match selection {
                            MainMenuSelection::NewGame => newselection = MainMenuSelection::Quit,
                            MainMenuSelection::Quit => newselection = MainMenuSelection::NewGame
                        }
                        return MainMenuResult::NoSelection{ selected: newselection }
                    }
                    VirtualKeyCode::Down => {
                        let newselection;
                        match selection {
                            MainMenuSelection::NewGame => newselection = MainMenuSelection::Quit,
                            MainMenuSelection::Quit => newselection = MainMenuSelection::NewGame
                        }
                        return MainMenuResult::NoSelection{ selected: newselection }
                    }
                    VirtualKeyCode::Return => return MainMenuResult::Selected{ selected : selection },
                    _ => return MainMenuResult::NoSelection{ selected: selection }
                }
            }
        }
    }

    MainMenuResult::NoSelection { selected: MainMenuSelection::NewGame }
}

pub fn game_over(ctx : &mut Rltk) -> GameOverResult {
    match ctx.key {
        None => GameOverResult::NoSelection,
        Some(VirtualKeyCode::Escape) => { return GameOverResult::QuitToMenu }
        Some(_) => GameOverResult::NoSelection
    }
}