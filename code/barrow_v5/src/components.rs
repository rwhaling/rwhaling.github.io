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

#[derive(Component, Debug, Copy, Clone)]
pub struct Player {
    pub food: i32,
    pub max_food: i32,
    pub coin: i32,
    pub potions: i32,
    pub atk_bonus: i32,
    pub def_bonus: i32,
    pub has_amulet: bool
}

#[derive(Debug)]
pub enum Items {
    Coin(i32),
    Food(i32),
    Potion
}

#[derive(Component, Debug)]
pub struct Item {
    pub item: Items
}

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
pub enum CombatStance { Guard, Ready, Power, Stun }

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
    pub current_target : Option<Entity>,
    pub last_command : Option<Command>
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum ActionType { Move, Wait, Attack }

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum AttackMove { Melee, Slash, Smash, Bash, Poke }

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum WaitMove { Wait, Fend, Block, Brace }

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum Command {
    MoveCommand,
    WaitCommand(WaitMove),
    AttackCommand(AttackMove)
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub struct MenuCommand {
    pub command: Command,
    pub cost: i32,
    pub stance_after: CombatStance,
    pub enabled: bool
}

#[derive(Component, Debug, Clone, Copy)]
pub struct Action {
    pub command: Command,
    pub cost: i32,
    pub stance_after: CombatStance,
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
    pub primary_attack: AttackMove,
    pub primary_attack_cost: i32,
    pub recover_ep_threshold: i32,
    pub recover_ep_chance: f32,
    pub visible_chase_chance: f32,
    pub invisible_chase_chance: f32
}

