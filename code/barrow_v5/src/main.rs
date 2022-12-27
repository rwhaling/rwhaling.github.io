use std::cmp::Ordering;
use rltk::{GameState, Rltk, RGB, Point, register_palette_color, RandomNumberGenerator};
// use rltk::console;
use specs::prelude::*;
mod components;
pub use components::*;
mod map;
pub use map::*;
mod player;
use player::*;
mod rect;
pub use rect::Rect;
mod visibility_system;
use visibility_system::VisibilitySystem;
mod monster_ai_system;
use monster_ai_system::MonsterAI;
mod map_indexing_system;
use map_indexing_system::MapIndexingSystem;
mod action_system;
use action_system::ActionSystem;
mod gui;
mod gamelog;
use gamelog::GameLog;
mod spawner;

#[derive(PartialEq, Copy, Clone)]
pub enum RunState { 
    AwaitingInput, 
    PreRun, 
    Descend { depth: i32 },
    PlayerTurn, 
    MonsterTurn,
    Shopping { menu_selection : i32},
    MainMenu { menu_selection : gui::MainMenuSelection },
    GameOver
}

pub struct State {
    pub ecs: World,
}

impl State {
    fn run_systems(&mut self) {
        let mut vis = VisibilitySystem{};
        vis.run_now(&self.ecs);
        let mut mob = MonsterAI{};
        mob.run_now(&self.ecs);
        let mut melee = ActionSystem{};
        melee.run_now(&self.ecs);
        let mut mapindex = MapIndexingSystem{};
        mapindex.run_now(&self.ecs);
        self.ecs.maintain();
    }

    fn load_level(&mut self, depth : i32, player_inv: Option<&Player>) {
        let mut rng = RandomNumberGenerator::new();

        {
            let mut logs = self.ecs.write_resource::<GameLog>();
            if depth == 0 {
                // TODO: fix, not appropriate after town
                logs.entries.clear();
                logs.entries.push(String::from("Welcome to Barrow!"));
            }    
        }
        // Not sure about removing player, seems ok?
        let _old_player = self.ecs.remove::<Player>();

        let _old_map = self.ecs.remove::<Map>();

        let mut to_delete = Vec::new();
        for e in self.ecs.entities().join() {
            to_delete.push(e);
        }
        for del in to_delete.iter() {
            self.ecs.delete_entity(*del).expect("Deletion failed");
        }

        let map : Map = Map::new_map_rooms_and_corridors(depth);

        let (player_x, player_y) = map.rooms[0].center();
    
        // self.ecs.insert::<Player>(player_inv);
        let player_entity = spawner::player(&mut self.ecs, player_x, player_y, player_inv);
        self.ecs.insert(player_entity);    

        if depth == 1 {
            spawner::populate_level_1(&mut self.ecs, &mut rng, &map);
        } else if depth == 2 {
            spawner::populate_level_2(&mut self.ecs, &mut rng, &map);
        } else {
            spawner::populate_level_2(&mut self.ecs, &mut rng, &map);
        }
        
        self.ecs.insert(map);


    }
}

impl GameState for State {
    fn tick(&mut self, ctx : &mut Rltk) {
        let mut newrunstate;
        {
            let runstate = self.ecs.fetch::<RunState>();
            ctx.set_active_console(1);
            ctx.cls();
            ctx.set_active_console(0);
            newrunstate = *runstate;
        }

        ctx.cls();

        match newrunstate {
            RunState::MainMenu{..} => {}
            _ => {
                draw_map(&self.ecs, ctx);
                {
                    let positions = self.ecs.read_storage::<Position>();
                    let renderables = self.ecs.read_storage::<Renderable>();
                    let items = self.ecs.read_storage::<Item>();
                    let map = self.ecs.fetch::<Map>();

                    let data = (&positions, &renderables, &items).join().collect::<Vec<_>>();
                    for (pos, render, _item) in data.iter() {
                        let idx = map.xy_idx(pos.x, pos.y);
                        if map.visible_tiles[idx] { ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph) }
                    }
                }
                {
                    let positions = self.ecs.read_storage::<Position>();
                    let renderables = self.ecs.read_storage::<Renderable>();
                    let stats = self.ecs.read_storage::<CombatStats>();
                    let map = self.ecs.fetch::<Map>();

                    let data = (&positions, &renderables, &stats).join().collect::<Vec<_>>();
                    for (pos, render, _stats) in data.iter() {
                        let idx = map.xy_idx(pos.x, pos.y);
                        if map.visible_tiles[idx] { ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph) }
                    }
                }
                player::update_targeting(&self.ecs, ctx);
                gui::draw_ui(&self.ecs, ctx);
            }
        }

        match newrunstate {
            RunState::PreRun => {
                self.run_systems();
                self.ecs.maintain();
                newrunstate = RunState::AwaitingInput;
            }
            RunState::AwaitingInput => {
                newrunstate = player_input(self, ctx);
            }
            RunState::PlayerTurn => {
                self.run_systems();
                self.ecs.maintain();
                newrunstate = RunState::MonsterTurn;
            }
            RunState::MonsterTurn => {
                self.run_systems();
                self.ecs.maintain();
                newrunstate = RunState::AwaitingInput;
            }
            RunState::Shopping { .. } => {
                // self.run_systems();
                let result = gui::shopping(self, ctx);
                match result {
                    RunState::PreRun => {
                        let player_inv:Player;
                        {
                            let player_entity = self.ecs.fetch::<Entity>();
                            let players = self.ecs.write_storage::<Player>();    
                            player_inv = *players.get(*player_entity).unwrap();
                        }
                    
                        self.load_level(1, Some(&player_inv));
                        newrunstate = result;
                    }
                    _ => {
                        newrunstate = result;
                    }
                }
            }
            RunState::Descend{ depth: d } => {
                let player_inv:Player;
                {
                    let player_entity = self.ecs.fetch::<Entity>();
                    let players = self.ecs.write_storage::<Player>();    
                    player_inv = *players.get(*player_entity).unwrap();
                }

                self.load_level(d, Some(&player_inv));
                newrunstate = RunState::PreRun;
            }
            RunState::MainMenu{ .. } => {
                let result = gui::main_menu(self, ctx);
                match result {
                    gui::MainMenuResult::NoSelection{ selected } => newrunstate = RunState::MainMenu{ menu_selection: selected },
                    gui::MainMenuResult::Selected{ selected } => {
                        match selected {
                            gui::MainMenuSelection::NewGame => newrunstate = RunState::PreRun,
                            gui::MainMenuSelection::Quit => { ::std::process::exit(0); }
                        }
                    }
                }
            }
            RunState::GameOver => {
                let result = gui::game_over(ctx);
                match result {
                    gui::GameOverResult::NoSelection => {}
                    gui::GameOverResult::QuitToMenu => {
                        {
                            let mut log = self.ecs.write_resource::<GameLog>();
                            log.entries.clear();
                        }
                        self.load_level(1, None);
                        // self.game_over_cleanup();
                        newrunstate = RunState::MainMenu{ menu_selection: gui::MainMenuSelection::NewGame };
                    }
                }
            }
        }

        {
            let mut runwriter = self.ecs.write_resource::<RunState>();
            *runwriter = newrunstate;
        }
        action_system::delete_the_dead(&mut self.ecs);
    }
}


fn main() -> rltk::BError {
    use rltk::RltkBuilder;
    let context = RltkBuilder::simple(80,60)
        .unwrap()
        .with_font("vga8x16.png", 8u32, 16u32)
        .with_sparse_console(80u32, 30u32, "vga8x16.png")
        .with_title("Barrow")
        .build()?;

    register_palette_color("grey", RGB::named(rltk::GREY));
    register_palette_color("pink", RGB::named(rltk::MAGENTA));
    register_palette_color("red", RGB::named(rltk::RED));
    register_palette_color("orange", RGB::named(rltk::ORANGE));
    register_palette_color("yellow", RGB::named(rltk::YELLOW));
    register_palette_color("green", RGB::named(rltk::GREEN));
    register_palette_color("cyan", RGB::named(rltk::CYAN));
    register_palette_color("blue", RGB::named(rltk::BLUE));

    let mut gs = State {
        ecs: World::new(),
    };
    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<Player>();
    gs.ecs.register::<Item>();
    gs.ecs.register::<Viewshed>();
    gs.ecs.register::<Monster>();
    gs.ecs.register::<Name>();
    gs.ecs.register::<BlocksTile>();
    gs.ecs.register::<CombatStats>();
    gs.ecs.register::<Action>();
    gs.ecs.register::<SmartMonster>();
    gs.ecs.insert(rltk::RandomNumberGenerator::new());
    gs.ecs.insert(gamelog::GameLog{ entries : vec![] });

    gs.ecs.insert(RunState::MainMenu{ menu_selection: gui::MainMenuSelection::NewGame });
    gs.load_level(1,None);

    rltk::main_loop(context, gs)
}
