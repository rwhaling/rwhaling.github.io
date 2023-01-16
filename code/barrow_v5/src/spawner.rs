use rltk::{ RGB, RandomNumberGenerator };
use rltk::console;
use specs::prelude::*;
use super::{CombatStats, AttackMove, CombatStance, Player, Renderable, Rect, Map, TileType, Name, Position, Container, Containers, Item, Items, Viewshed, Monster, BlocksTile, SmartMonster, SmartMonsterState };
use super::Containers::*;
use super::Items::*;
use super::Command::*;
use super::AttackMove::*;
use super::WaitMove::*;
use super::CombatStance::*;

/// Spawns the player and returns his/her entity object.
pub fn player(ecs : &mut World, player_x : i32, player_y : i32, player_state: Option<&Player>) -> Entity {
    // TODO uncheat haha
    let player = player_state.unwrap_or(&Player { food: 10, max_food: 10, coin: 0, potions: 0, atk_bonus: 0, def_bonus: 0, deepest_level: 0, has_amulet: false } );
    let player_stats = CombatStats{ max_hp: 30, hp: 30, hp_regen: -10, max_ep: 40, ep: 40, ep_regen: -5, defense: 0 + &player.def_bonus, power: 4 + &player.atk_bonus, attack_cost: 5, stance: Ready, current_target: None, visible_targets: vec![], last_command: None };
    return ecs
        .create_entity()
        .with(Position { x: player_x, y: player_y })
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
        })
        .with(*player)
        .with(Viewshed{ visible_tiles : Vec::new(), range: 8, dirty: true })
        .with(Name{name: "You".to_string() })
        .with(player_stats)
        .build();
    }

pub fn coins(ecs: &mut World, loc: (i32, i32), tag:u64, amount: i32) {
    ecs.create_entity()
        .with(Position{ x: loc.0, y: loc.1 })
        .with(Renderable{
            glyph: rltk::to_cp437('$'),
            fg: RGB::from_u8(182_u8,182_u8,182_u8),
            bg: RGB::named(rltk::BLACK),
        })
        .with(Name{ name : "Coins".to_string() })
        .with(Item{ item: Coin(amount), tag: tag })
        .build();
}

pub fn amulet(ecs: &mut World, loc: (i32, i32), tag:u64) {
    ecs.create_entity()
        .with(Position{ x: loc.0, y: loc.1 })
        .with(Renderable{
            glyph: rltk::to_cp437('!'),
            fg: RGB::from_u8(212_u8,175_u8,55_u8),
            bg: RGB::named(rltk::BLACK),
        })
        .with(Name{ name : "Amulet".to_string() })
        .with(Item{ item: Amulet, tag: tag })
        .build();
}

pub fn barrel(ecs: &mut World, loc: (i32, i32), tag: u64) {
    ecs.create_entity()
    .with(Position{ x: loc.0, y: loc.1 })
    .with(Renderable{
        glyph: rltk::to_cp437('?'),
        fg: RGB::from_u8(150_u8,120_u8,100_u8),
        bg: RGB::named(rltk::BLACK),
    })
    .with(Name{ name : "Barrel".to_string() })
    .with(Container{ container : Barrel, tag: tag })
    .build();
}

pub fn treasure(ecs: &mut World, loc: (i32, i32), tag: u64) {
    ecs.create_entity()
    .with(Position{ x: loc.0, y: loc.1 })
    .with(Renderable{
        glyph: rltk::to_cp437('?'),
        fg: RGB::from_u8(212_u8,175_u8,55_u8),
        bg: RGB::named(rltk::BLACK),
    })
    .with(Name{ name : "Treasure Chest".to_string() })
    .with(Container{ container : Treasure, tag: tag })
    .build();
}

pub fn orc(ecs: &mut World, loc: (i32, i32), tag: u64) { monster(ecs, loc.0, loc.1, tag, rltk::to_cp437('o'), 15, 30, 15, 4, 1, "Orc", Power, Smash, 0.2, 0, 1.0); }
pub fn goblin(ecs: &mut World, loc: (i32, i32), tag: u64) { monster(ecs, loc.0, loc.1, tag, rltk::to_cp437('g'), 18, 20, 5, 3, 1, "Goblin", Ready, Melee, 0.4, 0, 1.0); }
pub fn hobgoblin(ecs: &mut World, loc: (i32, i32), tag: u64) { monster(ecs, loc.0, loc.1, tag, rltk::to_cp437('h'), 25, 45, 15, 5, 1, "Hobgoblin", Guard, Bash, 0.5, 20, 0.3); }
pub fn ogre(ecs: &mut World, loc: (i32, i32), tag: u64) { monster(ecs, loc.0, loc.1, tag, rltk::to_cp437('O'), 40, 45, 5, 6, 3, "Ogre", Ready, Melee, 0.4, 30, 0.7); }
pub fn troll(ecs: &mut World, loc: (i32, i32), tag: u64) { monster(ecs, loc.0, loc.1, tag, rltk::to_cp437('T'), 40, 30, 15, 5, 2, "Troll", Power, Smash, 0.3, 0, 1.0); }
pub fn kobold(ecs: &mut World, loc: (i32, i32), tag: u64) { monster(ecs, loc.0, loc.1, tag, rltk::to_cp437('k'), 25, 30, 5, 4, 1, "Kobold", Ready, Melee, 0.2, 10, 0.6); }
pub fn goblin_knight(ecs: &mut World, loc: (i32, i32), tag: u64) { monster(ecs, loc.0, loc.1, tag, rltk::to_cp437('G'), 35, 45, 15, 6, 2, "Goblin Knight", Guard, Bash, 0.5, 20, 0.3); }
pub fn barrow_lord(ecs: &mut World, loc: (i32, i32), tag: u64) { monster(ecs, loc.0, loc.1, tag, rltk::to_cp437('B'), 40, 45, 15, 6, 3, "Barrow-Lord", Power, Smash, 0.4, 30, 0.7); }


fn monster<S : ToString>(ecs: &mut World, x: i32, y: i32, tag:u64, glyph : rltk::FontCharType, hp:i32, ep:i32, cost:i32, pow:i32, def:i32, name : S, stance: CombatStance, attack: AttackMove, chase_chance: f32, ep_threshold: i32, recover_ep_chance: f32) {
    ecs.create_entity()
        .with(Position{ x, y })
        .with(Renderable{
            glyph,
            fg: RGB::named(rltk::RED),
            bg: RGB::named(rltk::BLACK),
        })
        .with(Viewshed{ visible_tiles : Vec::new(), range: 8, dirty: true })
        .with(Monster{ tag: tag })
        .with(Name{ name : name.to_string() })
        .with(BlocksTile{})
        .with(CombatStats{ max_hp: hp, hp: hp, hp_regen:-5, max_ep: ep, ep: ep, ep_regen:-10, defense: def, power: pow, attack_cost: 5, stance: Ready, current_target: None, visible_targets: vec![], last_command: None })
        .with(SmartMonster{ 
            state: SmartMonsterState::Asleep,
            time_in_current_state: 0,
            target_location: None,
            primary_stance: stance,
            primary_attack: attack,
            primary_attack_cost: cost,
            recover_ep_threshold: ep_threshold,
            recover_ep_chance: recover_ep_chance,
            visible_chase_chance: 0.9,
            invisible_chase_chance: chase_chance
        })
        .build();
}

pub fn gen_spawn_points(room: &Rect, count: i32, rng: &mut RandomNumberGenerator) -> Vec<(i32, i32)> {
    let mut res : Vec<(i32, i32)> = vec![];
    res.push(room.center());
    while (res.len() as i32) < count {
        // todo - check if duplicate
        let x = rng.range(room.x1 + 1, room.x2 + 1);
        let y = rng.range(room.y1 + 1, room.y2 + 1);
        res.push((x,y));
    }
    console::log(format!("spawn points for room {:?}: {:?}", room, res));

    return res
}

pub fn gen_tags(count: i32, rng: &mut RandomNumberGenerator) -> Vec<u64> {
    let mut res : Vec<u64> = vec![];
    for i in 0..count {
        res.push(rng.next_u64());
    }

    return res
}

pub fn populate_level_1(ecs: &mut World, rng: &mut RandomNumberGenerator, map: &Map) {
    let last_room = map.rooms.len() - 1;
    for (i,room) in map.rooms.iter().enumerate().skip(1) {
        let spawn_points = gen_spawn_points(&room, 5, rng);
        let tags = gen_tags(10, rng);
        // let (x,y) = spawn_points[0];
        // let (item_x, item_y) = spawn_points[1];
        // let (barrel_x, barrel_y) = spawn_points[2];

        if i == 1 {
            // console::log(format!("{} spawning goblin at {},{}", i, x, y));
            // let gob_tag = rng.next_u64();
            goblin( ecs, spawn_points[0], tags[0]);            
            let coin_amount = rng.range(2,6);
            coins( ecs, spawn_points[1], tags[1], coin_amount);
            barrel( ecs, spawn_points[1], tags[2]);
        } else if i == last_room {
            // console::log(format!("{} spawning goblin knight at {},{}", i, x, y));
            // let hob_tag = rng.next_u64();
            hobgoblin( ecs, spawn_points[1], tags[0]);

            coins( ecs, spawn_points[2], tags[1],15);
            treasure( ecs, spawn_points[2], tags[2]);
        } else {
            // console::log(format!("{} spawning monster at {},{}", i, x, y));
            let roll = rng.roll_dice(1, 6);
            let monster_tag = rng.next_u64();

            match roll {
                3 => { orc(ecs, spawn_points[0], tags[0]) }
                2 => { orc(ecs, spawn_points[0], tags[0]) }
                1 => { orc(ecs, spawn_points[0], tags[0]) }
                _ => { goblin(ecs, spawn_points[0], tags[0]) }
            }        

            let coin_amount = rng.range(2,6);

            coins( ecs, spawn_points[1], tags[1], coin_amount);
            barrel( ecs, spawn_points[1], tags[2]);
            barrel( ecs, spawn_points[2], tags[3]);
        }
    }
}

pub fn populate_level_2(ecs: &mut World, rng: &mut RandomNumberGenerator, map: &Map) {
    let last_room = map.rooms.len() - 1;
    for (i,room) in map.rooms.iter().enumerate().skip(1) {
        let spawn_points = gen_spawn_points(&room, 5, rng);
        let tags = gen_tags(10, rng);
        if i == last_room {
            kobold( ecs, spawn_points[1], tags[0]);
            kobold( ecs, spawn_points[2], tags[1]);
            coins(ecs, spawn_points[3], tags[2], 20);
            treasure(ecs, spawn_points[3], tags[3]);
        } else {
            let roll = rng.roll_dice(1, 6);
            match roll {
                1 => { 
                    kobold(ecs, spawn_points[0], tags[0]) 
                },
                2 => { 
                    goblin(ecs, spawn_points[0], tags[0]);
                    goblin(ecs, spawn_points[1], tags[1]);
                },
                3 => {
                    kobold(ecs, spawn_points[0], tags[0]);
                    goblin(ecs, spawn_points[1], tags[1]);
                },
                _ => {
                    orc(ecs, spawn_points[0], tags[0]);
                }
            }

            let coin_amount = rng.range(4,10);
            console::log(format!("spawning {} coins", coin_amount));
            coins( ecs, spawn_points[2], tags[2], coin_amount);
            barrel( ecs, spawn_points[2], tags[3]);
            barrel( ecs, spawn_points[3], tags[4]);

        }
    }
}


pub fn populate_level_3(ecs: &mut World, rng: &mut RandomNumberGenerator, map: &Map) {
    let last_room = map.rooms.len() - 1;
    for (i,room) in map.rooms.iter().enumerate().skip(1) {
        let spawn_points = gen_spawn_points(&room, 5, rng);
        let tags = gen_tags(10, rng);
        if i == last_room {
            orc( ecs, spawn_points[1], tags[0]);
            orc( ecs, spawn_points[2], tags[1]);
            coins(ecs, spawn_points[3], tags[2], 25);
            treasure(ecs, spawn_points[3], tags[3]);
        } else {
            let roll = rng.roll_dice(1, 6);
            match roll {
                1 => { 
                    kobold(ecs, spawn_points[0], tags[0]) 
                },
                2 => { 
                    goblin(ecs, spawn_points[0], tags[0]);
                    goblin(ecs, spawn_points[1], tags[1]);
                },
                3 => {
                    orc(ecs, spawn_points[0], tags[0]);
                    goblin(ecs, spawn_points[1], tags[1]);
                },
                _ => {
                    orc(ecs, spawn_points[0], tags[0]);
                }
            }

            let coin_amount = rng.range(5,11);
            console::log(format!("spawning {} coins", coin_amount));
            coins( ecs, spawn_points[2], tags[2], coin_amount);
            barrel( ecs, spawn_points[2], tags[3]);
            barrel( ecs, spawn_points[3], tags[4]);

        }
    }
}

pub fn populate_level_4(ecs: &mut World, rng: &mut RandomNumberGenerator, map: &Map) {
    let last_room = map.rooms.len() - 1;
    for (i,room) in map.rooms.iter().enumerate().skip(1) {
        let spawn_points = gen_spawn_points(&room, 5, rng);
        let tags = gen_tags(10, rng);
        if i == last_room {
            troll( ecs, spawn_points[1], tags[0]);
            coins(ecs, spawn_points[2], tags[2], 35);
            treasure(ecs, spawn_points[2], tags[3]);
        } else {
            let roll = rng.roll_dice(1, 6);
            match roll {
                1 => { 
                    kobold(ecs, spawn_points[0], tags[0]) 
                },
                2 => { 
                    hobgoblin(ecs, spawn_points[0], tags[0]);
                    goblin(ecs, spawn_points[1], tags[1]);
                },
                3 => {
                    orc(ecs, spawn_points[0], tags[0]);
                    orc(ecs, spawn_points[1], tags[1]);
                },
                4 => {
                    hobgoblin(ecs, spawn_points[0], tags[0]);
                }
                _ => {
                    goblin(ecs, spawn_points[0], tags[0]);
                    kobold(ecs, spawn_points[1], tags[1]);
                }
            }

            let coin_amount = rng.range(7,15);
            coins( ecs, spawn_points[2], tags[2], coin_amount);
            barrel( ecs, spawn_points[2], tags[3]);
            barrel( ecs, spawn_points[3], tags[4]);

        }
    }
}

pub fn populate_level_5(ecs: &mut World, rng: &mut RandomNumberGenerator, map: &Map) {
    let last_room = map.rooms.len() - 1;
    for (i,room) in map.rooms.iter().enumerate().skip(1) {
        let spawn_points = gen_spawn_points(&room, 5, rng);
        let tags = gen_tags(5, rng);

        // let (x,y) = spawn_points[0];
        // let (item_x, item_y) = spawn_points[1];

        if i == 1 {
            // console::log(format!("{} spawning goblin at {},{}", i, x, y));
            kobold( ecs, spawn_points[0], tags[0]);
            let coin_amount = rng.range(8,17);
            coins( ecs, spawn_points[1], tags[1], coin_amount);
            barrel( ecs, spawn_points[1], tags[2]);

        } else if i == last_room {


            goblin_knight( ecs, spawn_points[1], tags[0]);
            coins( ecs, spawn_points[2], tags[1], 45);
            treasure( ecs, spawn_points[2], tags[2]);

        } else {

            let roll = rng.roll_dice(1, 6);

            match roll {
                4 => { hobgoblin(ecs, spawn_points[0], tags[0]) }
                3 => { kobold(ecs, spawn_points[0], tags[0]) }
                2 => { kobold(ecs, spawn_points[0], tags[0]) }
                1 => { troll(ecs, spawn_points[0], tags[0]) }
                _ => { orc(ecs, spawn_points[0], tags[0]) }
            }

            let coin_amount = rng.range(8,16);
            coins( ecs, spawn_points[1], tags[1], coin_amount);
            barrel( ecs, spawn_points[1], tags[2]);
            barrel( ecs, spawn_points[2], tags[3]);
        }
    }
}

pub fn populate_level_6(ecs: &mut World, rng: &mut RandomNumberGenerator, map: &Map) {
    let last_room = map.rooms.len() - 1;
    for (i,room) in map.rooms.iter().enumerate().skip(1) {
        let spawn_points = gen_spawn_points(&room, 5, rng);
        let tags = gen_tags(5, rng);

        // let (x,y) = spawn_points[0];
        // let (item_x, item_y) = spawn_points[1];

        if i == 1 {
            // console::log(format!("{} spawning goblin at {},{}", i, x, y));
            kobold( ecs, spawn_points[0], tags[0]);
            let coin_amount = rng.range(12,21);
            coins( ecs, spawn_points[1], tags[1], coin_amount);
            barrel( ecs, spawn_points[1], tags[2]);
            kobold( ecs, spawn_points[2], tags[3]);

        } else if i == last_room {
            barrow_lord( ecs, spawn_points[0], tags[0]);
            amulet( ecs, spawn_points[1], tags[1]);
            treasure( ecs, spawn_points[1], tags[2]);

        } else {

            let roll = rng.roll_dice(1, 6);

            match roll {
                4 => { hobgoblin(ecs, spawn_points[0], tags[0]) }
                3 => { goblin_knight(ecs, spawn_points[0], tags[0]) }
                2 => { 
                    hobgoblin(ecs, spawn_points[0], tags[0]);
                    orc(ecs, spawn_points[1], tags[1]);
                }
                1 => { troll(ecs, spawn_points[0], tags[0]) }
                _ => { 
                    kobold(ecs, spawn_points[0], tags[0]);
                    kobold(ecs, spawn_points[1], tags[1]);
                }
            }

            let coin_amount = rng.range(12,21);
            coins( ecs, spawn_points[2], tags[2], coin_amount);
            barrel( ecs, spawn_points[2], tags[3]);
            barrel( ecs, spawn_points[3], tags[4]);
        }
    }
}