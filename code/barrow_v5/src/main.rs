use std::collections::HashMap;
use std::cmp::Ordering;
use rltk::{GameState, Rltk, RGB, Point, register_palette_color, RandomNumberGenerator};
use rltk::console;
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
use gui::ShoppingResult::*;
mod gamelog;
use gamelog::GameLog;
mod spawner;

#[derive(PartialEq, Copy, Clone)]
pub enum RunState { 
    AwaitingInput, 
    PreRun, 
    Ascend { depth: i32},
    Descend { depth: i32 },
    PlayerTurn, 
    MonsterTurn,
    Shopping { menu_selection : i32},
    MainMenu { menu_selection : gui::MainMenuSelection },
    GameOver
}

pub struct LevelState {
    pub seed: u64,
    pub live_tags: HashMap<u64, bool>,
    pub revealed_tiles: Vec<bool>,
    pub player_pos: Option<Position>
}

pub struct LevelHistory {
    pub levels: HashMap<i32, LevelState>
}

pub struct State {
    pub ecs: World,
    pub history: LevelHistory,
    pub cheat_mode: bool
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

    fn load_level(&mut self, depth : i32, player_inv: Option<&Player>, clear: bool, ascend: bool) {

        // FIRST unload the old level

        {
            let mut logs = self.ecs.write_resource::<GameLog>();
            if depth == 0 {
                // TODO: fix, not appropriate after town
                logs.entries.clear();
                logs.entries.push(String::from("Welcome to Barrow!"));
            }    
        }

        // Not sure about removing player, seems ok?
        // finding: doesn't matter, player is never inserted as resource, always returns None
        // let old_player = self.ecs.remove::<Player>();
        // console::log(format!("removed player resource {:?}", old_player));

        let old_map = self.ecs.remove::<Map>();

        let mut old_player_pos : Option<Position> = None;
        // let mut monster_tags : Vec<u64> = vec![];
        // let mut item_tags : Vec<u64> = vec![]; 
        let mut live_tags : HashMap<u64, bool> = HashMap::new();

        let mut to_delete = Vec::new();
        {
            let players = self.ecs.read_storage::<Player>();
            let pos = self.ecs.read_storage::<Position>();
            let monsters = self.ecs.read_storage::<Monster>();
            let items = self.ecs.read_storage::<Item>();
            let containers = self.ecs.read_storage::<Container>();
            for (e,pos,player,monster,item,container) in (&self.ecs.entities(), (&pos).maybe(), (&players).maybe(), (&monsters).maybe(), (&items).maybe(), (&containers).maybe()).join() {
                if player.is_some() && pos.is_some() {
                    console::log(format!("unloading player entity {:?} {:?} at {:?}", e, player.unwrap(), pos.unwrap()));
                    old_player_pos = Some(*pos.unwrap());
                } else if monster.is_some() {
                    console::log(format!("unloading monster entity {:?} {:?}", e, monster.unwrap()));
                    live_tags.insert(monster.unwrap().tag, true);
                } else if item.is_some() {
                    console::log(format!("unloading item entity {:?} {:?}", e, item.unwrap()));
                    live_tags.insert(item.unwrap().tag, true);
                } else if container.is_some() {
                    console::log(format!("unloading container entity {:?} {:?}", e, container.unwrap()));
                    live_tags.insert(container.unwrap().tag, true);
                }
                to_delete.push(e);
            }
        }
        {
            for del in to_delete.iter() {
                self.ecs.delete_entity(*del).expect("Deletion failed");
            }
        }

        // STORE the unloaded level state in the history

        match old_map {
            Some(ref m) => {
                console::log(format!("unloading map {:?} at level {}, revealed_tiles: {:?}", m.seed, m.depth, m.revealed_tiles.len()));
                let mut old_level_state = LevelState { seed: m.seed, live_tags: live_tags, revealed_tiles: m.revealed_tiles.clone(), player_pos: old_player_pos };
                self.history.levels.insert(m.depth, old_level_state);
            }
            None => {
                console::log("no old map to unload");
            }
        }

        if clear {
            self.history.levels.clear()
        }

        // PREPARE for building or reloading a new level

        let mut rng = RandomNumberGenerator::new();

        let new_level_spawns : bool;

        let mut new_level_state = match (player_inv, self.history.levels.remove(&depth)) {
            (Some(player), Some(state)) => {
                console::log(format!("found history for {}, retaining, seed: {}", depth, state.seed));
                if player.has_amulet {
                    new_level_spawns = true;
                } else {
                    new_level_spawns = false;
                }
                state
            }
            (None, Some(state)) => {
                let new_seed = rng.next_u64();
                console::log(format!("found history but no player state, discarding, new seed: {}", new_seed));
                new_level_spawns = true;
                LevelState { seed: new_seed, live_tags: HashMap::new(), revealed_tiles: vec![], player_pos: None }
            }
            (_, None) => {
                let new_seed = rng.next_u64();
                console::log(format!("no match found in history for {}, creating new seed {}", depth, new_seed));
                new_level_spawns = true;
                LevelState { seed: new_seed, live_tags: HashMap::new(), revealed_tiles: vec![], player_pos: None }
            }
        };

        let mut map : Map = Map::new_map_rooms_and_corridors(depth, new_level_state.seed);

        // UGLY
        if new_level_state.revealed_tiles.len() > 0 {
            map.revealed_tiles = new_level_state.revealed_tiles.clone();
        };

        // let (player_x, player_y) = match (new_level_state.player_pos, ascend) {
        //     (None, false) => map.rooms[0].center(),
        //     (None, true)  => map.rooms[map.rooms.len() - 1].center(),
        //     (Some(pos),_) => (pos.x, pos.y)
        // };

        let (player_x, player_y) = match (new_level_state.player_pos, ascend) {
            (_, false) => map.rooms[0].center(),
            (_, true)  => map.rooms[map.rooms.len() - 1].center()
        };


        // ugly
        rng = RandomNumberGenerator::seeded(new_level_state.seed);

        // self.ecs.insert::<Player>(player_inv);
        let player_entity = spawner::player(&mut self.ecs, player_x, player_y, player_inv);
        self.ecs.insert(player_entity);    

        // Get vector of entity tags from spawner
        // todo: 
        if player_inv.is_some() && player_inv.unwrap().has_amulet {
            console::log(format!("loading level {:?} in amulet_mode", depth));
            spawner::populate_level_4(&mut self.ecs, &mut rng, &map);
        } else if depth == 1 {
            spawner::populate_level_1(&mut self.ecs, &mut rng, &map);
        } else if depth == 2 {
            spawner::populate_level_2(&mut self.ecs, &mut rng, &map);
        } else if depth == 3 {
            spawner::populate_level_3(&mut self.ecs, &mut rng, &map);
        } else if depth == 4 {
            spawner::populate_level_4(&mut self.ecs, &mut rng, &map);
        } else if depth == 5 {
            spawner::populate_level_5(&mut self.ecs, &mut rng, &map);
         } else if depth == 6 {
            spawner::populate_level_6(&mut self.ecs, &mut rng, &map);
            // hack but whatevs
            let stairs_down_position = map.rooms[map.rooms.len()-1].center();
            let stairs_down_idx = map.xy_idx(stairs_down_position.0, stairs_down_position.1);
            map.tiles[stairs_down_idx] = TileType::Floor;        
        } else { 
            spawner::populate_level_5(&mut self.ecs, &mut rng, &map);
        }

        self.ecs.maintain();
        // despawn any entities that shouldn't be respawned
        // don't have to check if new spawn here at the top
        if !new_level_spawns {
            let mut to_despawn : Vec<Entity> = vec![];

            {
                let monsters = self.ecs.read_storage::<Monster>();
                let items = self.ecs.read_storage::<Item>();
                let containers = self.ecs.read_storage::<Container>();
                for (e,monster,item,container) in (&self.ecs.entities(), (&monsters).maybe(), (&items).maybe(), (&containers).maybe()).join() {
                    if monster.is_some() {
                        let monster_tag = &monster.unwrap().tag;
                        if new_level_state.live_tags.contains_key(monster_tag) {
                            // could add to live_tags here
                        } else {
                            console::log(format!("despawning monster {:?}", monster));
                            to_despawn.push(e);
                        }
                    } else if item.is_some() {
                        let item_tag = &item.unwrap().tag;
                        if new_level_state.live_tags.contains_key(item_tag) {
                            // could add to live tags here
                        } else {
                            console::log(format!("despawning item {:?}", item));
                            to_despawn.push(e);
                        }
                    } else if container.is_some() {
                        let container_tag = &container.unwrap().tag;
                        if new_level_state.live_tags.contains_key(container_tag) {
                            // could add to live tags here
                        } else {
                            console::log(format!("despawning container {:?}", container));
                            to_despawn.push(e);
                        }
                    }
                }
            }
            for e in to_despawn.iter() {
                self.ecs.delete_entity(*e).expect("Deletion failed");
            }    
    
        }

        // should populate live_tags for newly generated levels here, probably skippable though

        self.history.levels.insert(depth, new_level_state );

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
                    let containers = self.ecs.read_storage::<Container>();
                    let map = self.ecs.fetch::<Map>();

                    let data = (&positions, &renderables, &containers).join().collect::<Vec<_>>();
                    for (pos, render, _container) in data.iter() {
                        let idx = map.xy_idx(pos.x, pos.y);
                        if map.visible_tiles[idx] { ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph) }
                    }
                }
                {
                    let positions = self.ecs.read_storage::<Position>();
                    let renderables = self.ecs.read_storage::<Renderable>();
                    let stats = self.ecs.read_storage::<CombatStats>();
                    let player = self.ecs.read_storage::<Player>();
                    let map = self.ecs.fetch::<Map>();

                    let data = (&positions, &renderables, &stats, (&player).maybe()).join().collect::<Vec<_>>();
                    for (pos, render, _stats, player) in data.iter() {
                        let idx = map.xy_idx(pos.x, pos.y);
                        if map.visible_tiles[idx] { ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph) }
                        if player.is_some() && player.unwrap().has_amulet == true {
                            let anim_index = map.frame_count.rem_euclid(60);
                            let anim_index_cycle = if anim_index > 30 { 30 - (anim_index - 30) } else { anim_index };
                            ctx.set(pos.x, pos.y, RGB::from_u8(255u8,8u8 * anim_index_cycle as u8, 0), render.bg, render.glyph);
                        }
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
                    Return => {
                        let player_inv:Player;
                        {
                            let player_entity = self.ecs.fetch::<Entity>();
                            let players = self.ecs.write_storage::<Player>();    
                            player_inv = *players.get(*player_entity).unwrap();
                        }
                    
                        self.load_level(1, Some(&player_inv), false, false);
                        newrunstate = RunState::PreRun;
                    }
                    LongRest => {
                        let player_inv:Player;
                        {
                            let player_entity = self.ecs.fetch::<Entity>();
                            let players = self.ecs.write_storage::<Player>();    
                            player_inv = *players.get(*player_entity).unwrap();
                        }
                        self.history.levels.clear();
                        self.load_level(1, Some(&player_inv), true, false);
                        newrunstate = RunState::Shopping { menu_selection : 0 };
                    }
                    Deepest => {
                        let player_inv: Player;
                        {
                            let player_entity = self.ecs.fetch::<Entity>();
                            let players = self.ecs.write_storage::<Player>();    
                            player_inv = *players.get(*player_entity).unwrap();
                        }
                        self.load_level(player_inv.deepest_level, Some(&player_inv), false, false);
                        newrunstate = RunState::PreRun;
                    }
                    Selected { selected: s } => {
                        newrunstate = RunState::Shopping { menu_selection : s };
                    }
                    _ => {
                        newrunstate = RunState::Shopping { menu_selection : 0 };
                    }
                }
            }
            RunState::Ascend { depth: d } => {
                if d < 1 {
                    let player_inv:Player;
                    {
                        let player_entity = self.ecs.fetch::<Entity>();
                        let players = self.ecs.write_storage::<Player>();    
                        player_inv = *players.get(*player_entity).unwrap();
                        // TODO - hack to put player at correct stairs
                    }
                    if player_inv.has_amulet {
                        console::log(format!("ascending to level {} with amulet, game ending",d));
                        let mut log = self.ecs.write_resource::<GameLog>();
                        log.entries.push(format!("You return to town safely with the Amulet..."));
                        log.entries.push(format!("But Yendor's darkness clings to your spirit."));
                        log.entries.push(format!("#[red](You have won Barrow!)"));
                        log.entries.push(format!("#[magenta](Press ESCAPE to return to the main menu!)"));

                        newrunstate = RunState::GameOver;

                    } else {
                        console::log(format!("ascending to level {}, shopping instead",d));
                        newrunstate = RunState::Shopping { menu_selection: 0 }    
                    }
    

                } else {
                    console::log("ascending to level {}, loading");
                    let player_inv:Player;
                    {
                        let player_entity = self.ecs.fetch::<Entity>();
                        let players = self.ecs.write_storage::<Player>();    
                        player_inv = *players.get(*player_entity).unwrap();
                        // TODO - hack to put player at correct stairs
                    }
    
                    self.load_level(d, Some(&player_inv), false, true);
                    newrunstate = RunState::PreRun;
                }
            }
            RunState::Descend{ depth: d } => {
                let player_inv:Player;
                {
                    let player_entity = self.ecs.fetch::<Entity>();
                    let players = self.ecs.write_storage::<Player>();    
                    player_inv = *players.get(*player_entity).unwrap();
                }

                self.load_level(d, Some(&player_inv), false, false);
                newrunstate = RunState::PreRun;
            }
            RunState::MainMenu{ .. } => {
                let result = gui::main_menu(self, ctx);
                match result {
                    gui::MainMenuResult::NoSelection{ selected } => newrunstate = RunState::MainMenu{ menu_selection: selected },
                    gui::MainMenuResult::Selected{ selected } => {
                        match selected {
                            gui::MainMenuSelection::NewGame => {
                                {
                                    let mut log = self.ecs.write_resource::<GameLog>();
                                    log.entries.push(format!("You enter the barrow of an ancient lord, having heard of its riches."));
                                    log.entries.push(format!("Many adventurers, it is said, have met their dooms within - "));
                                    log.entries.push(format!("Will you overcome the dangers, and retrieve the barrow-lord's treasure?"));
                                }

                                if self.cheat_mode {
                                    let player = Player { food: 10, max_food: 10, coin: 600, potions: 0, atk_bonus: 0, def_bonus: 0, deepest_level: 5, has_amulet: false };
                                    self.load_level(1,Some(&player),true, false);                                    
                                } else {
                                    self.load_level(1,None,true, false);
                                }
                                newrunstate = RunState::PreRun
                            },
                            gui::MainMenuSelection::Quit => { ::std::process::exit(0); },
                            gui::MainMenuSelection::CheatMode => {
                                self.cheat_mode = true;                                
                            }
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
                        self.history.levels.clear();
                        self.load_level(1, None, true, false);
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

    let mut history = LevelHistory {
        levels: HashMap::new(),
    };
    // gs.ecs.insert(history);

    let mut gs = State {
        ecs: World::new(),
        history: history,
        cheat_mode: false
    };

    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<Player>();
    gs.ecs.register::<Item>();
    gs.ecs.register::<Container>();
    gs.ecs.register::<Viewshed>();
    gs.ecs.register::<Monster>();
    gs.ecs.register::<Name>();
    gs.ecs.register::<BlocksTile>();
    gs.ecs.register::<CombatStats>();
    gs.ecs.register::<Action>();
    gs.ecs.register::<SmartMonster>();
    let mut rng = rltk::RandomNumberGenerator::new();
    gs.ecs.insert(rng);
    gs.ecs.insert(gamelog::GameLog{ entries : vec![] });

    gs.ecs.insert(RunState::MainMenu{ menu_selection: gui::MainMenuSelection::NewGame });
    gs.load_level(1,None,true, false);

    rltk::main_loop(context, gs)
}
