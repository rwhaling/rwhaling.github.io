use rltk::{ RGB, RandomNumberGenerator };
use specs::prelude::*;
use super::{CombatStats, CombatStance, Player, Renderable, Name, Position, Viewshed, Monster, BlocksTile};

/// Spawns the player and returns his/her entity object.
pub fn player(ecs : &mut World, player_x : i32, player_y : i32) -> Entity {
    ecs
        .create_entity()
        .with(Position { x: player_x, y: player_y })
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
        })
        .with(Player{})
        .with(Viewshed{ visible_tiles : Vec::new(), range: 8, dirty: true })
        .with(Name{name: "Player".to_string() })
        .with(CombatStats{ max_hp: 30, hp: 30, hp_regen: -10, max_ep: 40, ep: 40, ep_regen: -5, defense: 0, power: 3, attack_cost: 5, stance: CombatStance::GuardDown, current_target: None, visible_targets: vec![] })
        .build()
}

/// Spawns a random monster at a given location
pub fn random_monster(ecs: &mut World, x: i32, y: i32) {
    let roll :i32;
    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        roll = rng.roll_dice(1, 6);
    }
    match roll {
        6 => { goblin_knight(ecs, x, y) }
        2 => { orc(ecs, x, y) }
        1 => { orc(ecs, x, y) }
        _ => { goblin(ecs, x, y) }
    }
}

fn orc(ecs: &mut World, x: i32, y: i32) { monster(ecs, x, y, rltk::to_cp437('o'), 12, 20, 4, 0, "Orc"); }
fn goblin(ecs: &mut World, x: i32, y: i32) { monster(ecs, x, y, rltk::to_cp437('g'), 22, 30, 2, 0, "Goblin"); }
fn goblin_knight(ecs: &mut World, x: i32, y: i32) { monster(ecs, x, y, rltk::to_cp437('G'), 25, 40, 3, 1, "Goblin Knight"); }

fn monster<S : ToString>(ecs: &mut World, x: i32, y: i32, glyph : rltk::FontCharType, hp:i32, ep:i32, pow:i32, def:i32, name : S) {
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
        .with(CombatStats{ max_hp: hp, hp: hp, hp_regen:-5, max_ep: ep, ep: ep, ep_regen:-5, defense: def, power: pow, attack_cost: 5, stance: CombatStance::GuardDown, current_target: None, visible_targets: vec![] })
        .build();
}