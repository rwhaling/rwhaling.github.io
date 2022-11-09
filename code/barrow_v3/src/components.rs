use specs::prelude::*;
use specs_derive::*;
use rltk::{RGB};

#[derive(PartialEq, Component, Debug, Clone, Copy)]
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
pub enum ActionType { Move, Wait, Attack}

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum Attack { Melee, StrongMelee }

// #[derive(PartialEq, Copy, Clone, Debug)]
// pub enum CombatIntents { Move, Wait, Melee, StrongMelee }

#[derive(Component, Debug, Clone)]
pub struct Action {
    pub action_type: ActionType,
    pub attack: Option<Attack>,
    pub target : Option<Entity>,
    pub position: Option<Position>
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum SmartMonsterState { 
    Asleep, 
    Attacking, 
    Recovering, 
    Idle
}

#[derive(PartialEq, Component, Debug, Clone)]
pub struct SmartMonster {
    pub state: SmartMonsterState,
    pub time_in_current_state: i32,
    pub target_location: Option<Position>,
    pub primary_stance: CombatStance,
    pub primary_attack: Attack,
    pub recover_ep_threshold: i32,
    pub chase_chance: f32
}

