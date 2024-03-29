use rltk::{ RGB, RandomNumberGenerator };
use specs::prelude::*;
use super::{CombatStats, AttackMove, CombatStance, Player, Renderable, Name, Position, Viewshed, Monster, BlocksTile, SmartMonster, SmartMonsterState };
use super::Command::*;
use super::AttackMove::*;
use super::WaitMove::*;
use super::CombatStance::*;

/// Spawns the player and returns his/her entity object.
pub fn player(ecs : &mut World, player_x : i32, player_y : i32, stats: Option<CombatStats>) -> Entity {
    let player_stats = stats.unwrap_or(CombatStats{ max_hp: 30, hp: 30, hp_regen: -10, max_ep: 40, ep: 40, ep_regen: -5, defense: 0, power: 4, attack_cost: 5, stance: Ready, current_target: None, visible_targets: vec![], last_command: None });
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
        .with(player_stats)
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
        3 => { orc(ecs, x, y) }
        2 => { orc(ecs, x, y) }
        1 => { orc(ecs, x, y) }
        _ => { goblin(ecs, x, y) }
    }
}

pub fn orc(ecs: &mut World, x: i32, y: i32) { monster(ecs, x, y, rltk::to_cp437('o'), 15, 45, 15, 6, 1, "Orc", Power, Smash, 0.2, 0, 1.0); }
pub fn goblin(ecs: &mut World, x: i32, y: i32) { monster(ecs, x, y, rltk::to_cp437('g'), 18, 20, 5, 3, 1, "Goblin", Ready, Melee, 0.4, 0, 1.0); }
pub fn hobgoblin(ecs: &mut World, x: i32, y: i32) { monster(ecs, x, y, rltk::to_cp437('h'), 25, 45, 15, 4, 1, "Hobgoblin", Guard, Bash, 0.5, 20, 0.3); }

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