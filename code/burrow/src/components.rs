use specs::prelude::*;
use specs_derive::*;
use rltk::{RGB};

#[derive(Component)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Component)]
pub struct Renderable {
    pub glyph: rltk::FontCharType,
    pub fg: RGB,
    pub bg: RGB,
}

#[derive(Component, Debug)]
pub struct Player {}

#[derive(Component)]
pub struct Viewshed {
    pub visible_tiles : Vec<rltk::Point>,
    pub range : i32,
    pub dirty : bool
}

#[derive(Component, Debug)]
pub struct Monster {}

#[derive(Component, Debug)]
pub struct Name {
    pub name : String
}

#[derive(Component, Debug)]
pub struct BlocksTile {}

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum CombatStance { GuardUp, GuardDown, GuardBreak }

#[derive(Component, Debug)]
pub struct CombatStats {
    pub max_hp : i32,
    pub hp : i32,
    pub hp_regen: i32,
    pub max_ep: i32,
    pub ep: i32,
    pub ep_regen: i32,
    pub defense : i32,
    pub power : i32,
    pub attack_cost: i32,
    pub stance : CombatStance,
    pub visible_targets: Vec<Entity>,
    pub current_target : Option<Entity>
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum CombatIntents { Move, Wait, Melee, StrongMelee }

#[derive(Component, Debug, Clone)]
pub struct CombatIntent {
    pub intent: CombatIntents,
    pub target : Option<Entity>
}

#[derive(Component, Debug)]
pub struct SufferDamage {
    pub hp_amount : Vec<i32>,
    pub ep_amount : Vec<i32>
}

impl SufferDamage {
    pub fn new_hp_damage(store: &mut WriteStorage<SufferDamage>, victim: Entity, amount: i32) {
        if let Some(suffering) = store.get_mut(victim) {
            suffering.hp_amount.push(amount);
        } else {
            let dmg = SufferDamage { hp_amount : vec![amount], ep_amount: vec![] };
            store.insert(victim, dmg).expect("Unable to insert damage");
        }
    }

    pub fn new_ep_damage(store: &mut WriteStorage<SufferDamage>, victim: Entity, amount: i32) {
        if let Some(suffering) = store.get_mut(victim) {
            suffering.ep_amount.push(amount);
        } else {
            let dmg = SufferDamage { hp_amount : vec![], ep_amount: vec![amount] };
            store.insert(victim, dmg).expect("Unable to insert ep damage");
        }
    }
}
