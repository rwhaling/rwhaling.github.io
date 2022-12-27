use rltk::{ RGB, RandomNumberGenerator };
use specs::prelude::*;
use super::{CombatStats, AttackMove, CombatStance, Player, Renderable, Map, Name, Position, Item, Items, Viewshed, Monster, BlocksTile, SmartMonster, SmartMonsterState };
use super::Items::*;
use super::Command::*;
use super::AttackMove::*;
use super::WaitMove::*;
use super::CombatStance::*;

/// Spawns the player and returns his/her entity object.
pub fn player(ecs : &mut World, player_x : i32, player_y : i32, player_state: Option<&Player>) -> Entity {
    // TODO uncheat haha
    let player = player_state.unwrap_or(&Player { food: 15, max_food: 15, coin: 300, potions: 0, atk_bonus: 0, def_bonus: 0, has_amulet: false } );
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
        .with(Name{name: "Player".to_string() })
        .with(player_stats)
        .build();
    }

pub fn coins(ecs: &mut World, x: i32, y: i32, amount: i32) {
    ecs.create_entity()
        .with(Position{ x, y })
        .with(Renderable{
            glyph: rltk::to_cp437('$'),
            fg: RGB::from_u8(182_u8,182_u8,182_u8),
            bg: RGB::named(rltk::BLACK),
        })
        .with(Name{ name : "Coins".to_string() })
        .with(Item{ item: Coin(amount) })
        .build();
}

/// Spawns a random monster at a given location
// pub fn random_monster(ecs: &mut World, x: i32, y: i32) {
//     let roll :i32;
//     {
//         let mut rng = ecs.write_resource::<RandomNumberGenerator>();
//         roll = rng.roll_dice(1, 6);
//     }
//     match roll {
//         3 => { orc(ecs, x, y) }
//         2 => { orc(ecs, x, y) }
//         1 => { orc(ecs, x, y) }
//         _ => { goblin(ecs, x, y) }
//     }
// }

pub fn orc(ecs: &mut World, x: i32, y: i32) { monster(ecs, x, y, rltk::to_cp437('o'), 15, 45, 15, 6, 1, "Orc", Power, Smash, 0.2, 0, 1.0); }
pub fn goblin(ecs: &mut World, x: i32, y: i32) { monster(ecs, x, y, rltk::to_cp437('g'), 18, 20, 5, 3, 1, "Goblin", Ready, Melee, 0.4, 0, 1.0); }
pub fn hobgoblin(ecs: &mut World, x: i32, y: i32) { monster(ecs, x, y, rltk::to_cp437('h'), 25, 45, 15, 4, 1, "Hobgoblin", Guard, Bash, 0.5, 20, 0.3); }
pub fn ogre(ecs: &mut World, x: i32, y: i32) { monster(ecs, x, y, rltk::to_cp437('O'), 30, 45, 5, 4, 1, "Ogre", Ready, Melee, 0.4, 30, 0.7); }
pub fn troll(ecs: &mut World, x: i32, y: i32) { monster(ecs, x, y, rltk::to_cp437('T'), 50, 30, 15, 5, 2, "Troll", Power, Smash, 0.3, 0, 1.0); }
pub fn kobold(ecs: &mut World, x: i32, y: i32) { monster(ecs, x, y, rltk::to_cp437('k'), 25, 30, 5, 4, 1, "Kobold", Ready, Melee, 0.2, 10, 0.6); }

fn monster<S : ToString>(ecs: &mut World, x: i32, y: i32, glyph : rltk::FontCharType, hp:i32, ep:i32, cost:i32, pow:i32, def:i32, name : S, stance: CombatStance, attack: AttackMove, chase_chance: f32, ep_threshold: i32, recover_ep_chance: f32) {
    ecs.create_entity()
        .with(Position{ x, y })
        .with(Renderable{
            glyph,
            fg: RGB::named(rltk::RED),
            bg: RGB::named(rltk::BLACK),
        })
        .with(Viewshed{ visible_tiles : Vec::new(), range: 8, dirty: true })
        .with(Monster{})
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

pub fn populate_level_1(ecs: &mut World, rng: &mut RandomNumberGenerator, map: &Map) {
    let last_room = map.rooms.len() - 1;
    for (i,room) in map.rooms.iter().enumerate().skip(1) {
        let (x,y) = room.center();
        let item_x = (room.x1 + rng.roll_dice(1, i32::abs(room.x2 - room.x1)));
        let item_y = (room.y1 + rng.roll_dice(1, i32::abs(room.y2 - room.y1)));

        if i == 1 {
            // console::log(format!("{} spawning goblin at {},{}", i, x, y));
            goblin( ecs, x, y);
            let coin_amount = rng.roll_dice(1,5);
            coins( ecs, item_x, item_y, coin_amount);
        } else if i == last_room {
            // console::log(format!("{} spawning goblin knight at {},{}", i, x, y));
            hobgoblin( ecs, x, y);
            coins( ecs, item_x, item_y, 10);
        } else {
            // console::log(format!("{} spawning monster at {},{}", i, x, y));
            let roll = rng.roll_dice(1, 6);

            match roll {
                3 => { orc(ecs, x, y) }
                2 => { orc(ecs, x, y) }
                1 => { orc(ecs, x, y) }
                _ => { goblin(ecs, x, y) }
            }        

            let coin_amount = rng.roll_dice(1,5);
            coins( ecs, item_x, item_y, coin_amount);
        }
    }
}

pub fn populate_level_2(ecs: &mut World, rng: &mut RandomNumberGenerator, map: &Map) {
    let last_room = map.rooms.len() - 1;
    for (i,room) in map.rooms.iter().enumerate().skip(1) {
        let (x,y) = room.center();
        let item_x = (room.x1 + rng.roll_dice(1, i32::abs(room.x2 - room.x1)));
        let item_y = (room.y1 + rng.roll_dice(1, i32::abs(room.y2 - room.y1)));

        if i == 1 {
            // console::log(format!("{} spawning goblin at {},{}", i, x, y));
            kobold( ecs, x, y);
            let coin_amount = rng.roll_dice(3,7);
            coins( ecs, item_x, item_y, coin_amount);

        } else if i == last_room {
            // console::log(format!("{} spawning goblin knight at {},{}", i, x, y));
            coins( ecs, item_x, item_y, 20);

            ogre( ecs, x, y);
        } else {
            // console::log(format!("{} spawning monster at {},{}", i, x, y));
            let roll = rng.roll_dice(1, 6);

            match roll {
                4 => { hobgoblin(ecs, x, y) }
                3 => { kobold(ecs, x, y) }
                2 => { kobold(ecs, x, y) }
                1 => { goblin(ecs, x, y) }
                _ => { orc(ecs, x, y) }
            }        

            // random_monster( ecs, x, y);
            let coin_x = (room.x1 + rng.roll_dice(1, i32::abs(room.x2 - room.x1)));
            let coin_y = (room.y1 + rng.roll_dice(1, i32::abs(room.y2 - room.y1)));
            // let idx = (y * MAPWIDTH) + x;
            let coin_amount = rng.roll_dice(3,7);
            coins( ecs, coin_x, coin_y, coin_amount);
        }
    }
}