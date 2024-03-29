use rltk::{ RGB, Rltk, Point, VirtualKeyCode };
use specs::prelude::*;
use super::{CombatStats, Player, Monster, gamelog::GameLog, Map, Name, Position, RunState, State};

#[derive(PartialEq, Copy, Clone)]
pub enum MainMenuSelection { NewGame, Quit }

#[derive(PartialEq, Copy, Clone)]
pub enum MainMenuResult { NoSelection{ selected : MainMenuSelection }, Selected{ selected: MainMenuSelection } }

#[derive(PartialEq, Copy, Clone)]
pub enum GameOverResult { NoSelection, QuitToMenu }

pub fn draw_ui(ecs: &World, ctx : &mut Rltk) {
    ctx.draw_box(0, 43, 49, 16, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK));
    ctx.draw_box(50, 0, 29, 59, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK));

    let combat_stats = ecs.read_storage::<CombatStats>();
    let players = ecs.read_storage::<Player>();

    let monsters = ecs.read_storage::<Monster>();
    let names = ecs.read_storage::<Name>();
    let positions = ecs.read_storage::<Position>();
    let entities = ecs.entities();
    let map = ecs.fetch::<Map>();

    let mut target_offset = 1;
    let mut gui_offset = 1;

    for (_player, stats) in (&players, &combat_stats).join() {
        let health = format!("HP:{}/{} ", stats.hp, stats.max_hp);
        ctx.print_color(51, 1, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), &health);
        ctx.draw_bar_horizontal(63, 1, 16, stats.hp, stats.max_hp, RGB::named(rltk::RED), RGB::named(rltk::BLACK));
        let energy = format!("EP:{}/{} ", stats.ep, stats.max_ep);
        ctx.print_color(51, 2, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), &energy);
        ctx.draw_bar_horizontal(63, 2, 16, stats.ep, stats.max_ep, RGB::named(rltk::BLUE), RGB::named(rltk::BLACK));
        let stance = format!("Stance: {:?}", stats.stance);
        ctx.print_color(51, 3, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), &stance);

        for (entity, _monster, monster_stats, name, position) in (&entities, &monsters, &combat_stats, &names, &positions).join() {
            let idx = map.xy_idx(position.x, position.y);
            if map.visible_tiles[idx] == true {
                if stats.visible_targets.contains(&entity) {
                    if stats.current_target == Some(entity) {
                        let target_string = format!("({}) {}", target_offset, name.name);
                        ctx.print_color(51, 3 + gui_offset, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), target_string);
                        ctx.draw_bar_horizontal(63, 3 + gui_offset, 16, monster_stats.hp, monster_stats.max_hp, RGB::named(rltk::RED), RGB::named(rltk::BLACK));
                        ctx.draw_bar_horizontal(63, 4 + gui_offset, 16, monster_stats.ep, monster_stats.max_ep, RGB::named(rltk::BLUE), RGB::named(rltk::BLACK));
                        let monster_stance = format!("    Stance: {:?}", monster_stats.stance);
                        ctx.print_color(51, 5 + gui_offset, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), monster_stance);
                        target_offset += 1;
                        gui_offset += 2;
                    } else {
                        let target_string = format!("({}) {}", target_offset, name.name);
                        ctx.print(51, 3 + gui_offset, target_string);    
                        target_offset += 1;
                        gui_offset += 1;
                    }
                }
            }
            ctx.print(51, 4 + gui_offset, format!("                         "));
            ctx.print(51, 5 + gui_offset, format!("Commands                 "));
            ctx.print(51, 6 + gui_offset, format!("--------                 "));
            ctx.print(51, 7 + gui_offset, format!("(ASDW) Move              "));
            ctx.print(51, 8 + gui_offset, format!("(QEZC) Diag. Move        "));
            ctx.print(51, 9 + gui_offset, format!("(X/Sp) Wait              "));
            ctx.print(51, 10 + gui_offset, format!("(J) Attack Target       "));
            ctx.print(51, 11 + gui_offset, format!("(K) Strong Attack Target"));
            ctx.print(51, 12 + gui_offset, format!("(L) Change Stance       "));

        }
    }

    let log = ecs.fetch::<GameLog>();
    let mut y = 58;
    for s in log.entries.iter().rev() {
        if y >= 44 { ctx.print(2, y, s); }
        y -= 1;
    }

    // Draw mouse cursor
    let mouse_pos = ctx.mouse_pos();
    ctx.set_bg(mouse_pos.0, mouse_pos.1, RGB::named(rltk::MAGENTA));
    draw_tooltips(ecs, ctx);
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

    ctx.print_color_centered(15, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), "richard's untitled roguelike");

    if let RunState::MainMenu{ menu_selection : selection } = *runstate {
        if selection == MainMenuSelection::NewGame {
            ctx.print_color_centered(24, RGB::named(rltk::MAGENTA), RGB::named(rltk::BLACK), "Begin New Game (press Enter)");
        } else {
            ctx.print_color_centered(24, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), "Begin New Game");
        }

        if selection == MainMenuSelection::Quit {
            ctx.print_color_centered(26, RGB::named(rltk::MAGENTA), RGB::named(rltk::BLACK), "Quit (press Enter)");
        } else {
            ctx.print_color_centered(26, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), "Quit");
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
    ctx.print_color_centered(15, RGB::named(rltk::RED), RGB::named(rltk::BLACK), "You died!");
    ctx.print_color_centered(17, RGB::named(rltk::MAGENTA), RGB::named(rltk::BLACK), "Press ESCAPE to return to the menu.");

    match ctx.key {
        None => GameOverResult::NoSelection,
        Some(VirtualKeyCode::Escape) => { return GameOverResult::QuitToMenu }
        Some(_) => GameOverResult::NoSelection
    }
}