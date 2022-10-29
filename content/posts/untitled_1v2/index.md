+++
title = "untitled roguelike test 2"
date = "2022-10-29"
description = "Iterating on the game and learning more Rust"
+++

This one will probably be a shorter status update, since I am neck-deep in the actual game at this point.  I'll cut to the chase and share a build up here:

<canvas id="canvas" width="640" height="480"></canvas>
<script src="./wasm/untitled_1v2.js"></script>
<script>
    window.addEventListener("load", async () => {
    await wasm_bindgen("./wasm/untitled_1v2_bg.wasm");
    });
</script>

I think I am now within a stone's throw of feature-complete; I've added stamina meters, normal and strong melee attacks, waiting/resting to regen stamina/hp, and reworked the whole combat/damage system to support those features.

What I am finding as I go is that I am diverging from the architectural patterns in Wolverson's book - more and more, this game is turning out to be a turn-based RPG, and closer to Dragon Quest than Nethack mechanically, and as a result, more and more of the logic is getting consolidated into a system I'm calling `CombatIntent` for now, which gets fed either by player input or monster AI.

e.g., here is the player attack code:

```rust
pub fn try_attack_current_target(intent:CombatIntents, ecs: &World) -> RunState {
    let combat_stats = ecs.write_storage::<CombatStats>();
    let mut combat_intent = ecs.write_storage::<CombatIntent>();
    let player = ecs.read_storage::<Player>();
    let mut map = ecs.fetch::<Map>();
    let positions = ecs.read_storage::<Position>();
    let entities = ecs.entities();

    console::log(format!("trying to attack current target"));
    for (_player,player_entity, stats,player_pos) in (&player, &entities, &combat_stats, &positions).join() {
        match stats.current_target {
            Some(target) => {
                let target_pos = positions.get(target).unwrap();

                let distance = rltk::DistanceAlg::Pythagoras.distance2d(Point::new(target_pos.x, target_pos.y), Point::new(player_pos.x, player_pos.y));
                if distance < 1.5 {
                    combat_intent.insert(player_entity, CombatIntent{ intent: intent, target: Some(target) }).expect("Unable to insert attack");
                    return RunState::PlayerTurn
                } else {
                    console::log(format!("distance to target {:?} is {}, can't attack", target, distance));
                    return RunState::AwaitingInput
                }
            },
            _ => {
                console::log(format!("no target selected, can't attack"));
                return RunState::AwaitingInput
            }
        }
    }
    return RunState::AwaitingInput
}
```

and here is the handling logic for it in the CombatIntent system:

```rust
    fn run(&mut self, data : Self::SystemData) {
        let (entities, mut log, mut combat_intents, names, combat_stats, mut inflict_damage, mut rng) = data;

        for (entity, combat_intent, name, stats) in (&entities, &combat_intents, &names, &combat_stats).join() {
            match combat_intent {
                CombatIntent{ intent: CombatIntents::Melee, target: Some(target) } => {
                    log.entries.push(format!("{} will try to attack", name.name));
                    if stats.hp > 0 {
                        let target_stats = combat_stats.get(*target).unwrap();
                        if stats.ep >= stats.attack_cost {
                            if target_stats.hp > 0 {
                                let target_name = names.get(*target).unwrap();
                                let def_adj = if target_stats.stance == CombatStance::GuardUp { 1 } else { 0 };
                                let eff_def = target_stats.defense + def_adj;
                                let eff_pow = stats.power;
                                let damage = damage_formula(&mut rng,eff_pow,eff_def);
                                log.entries.push(format!("{} hits {}, for {} hp.", &name.name, &target_name.name, damage));
                                SufferDamage::new_hp_damage(&mut inflict_damage, *target, damage);
                                SufferDamage::new_ep_damage(&mut inflict_damage, entity, stats.attack_cost);
                            }
                        } else {
                            log.entries.push(format!("{} cannot attack, insufficient energy, resting instead", &name.name));
                            rest_or_default(entity, stats, &mut inflict_damage);
                        }
                    }
                }
                CombatIntent{ intent: CombatIntents::StrongMelee, target: Some(target) } => {
                    log.entries.push(format!("{} will try to STRONG attack", name.name));
                    if stats.hp > 0 {
                        let target_stats = combat_stats.get(*target).unwrap();
                        if stats.ep >= (stats.attack_cost + 10) {
                            if target_stats.hp > 0 {
                                let target_name = names.get(*target).unwrap();
                                let def_adj = if target_stats.stance == CombatStance::GuardUp { 1 } else { 0 };
                                let eff_def = target_stats.defense + def_adj;
                                let eff_pow = stats.power + 1;
                                let damage = damage_formula(&mut rng,eff_pow,eff_def);
                                log.entries.push(format!("{} hits {} fiercely for {} hp.", &name.name, &target_name.name, damage));
                                SufferDamage::new_hp_damage(&mut inflict_damage, *target, damage);
                                SufferDamage::new_ep_damage(&mut inflict_damage, entity, stats.attack_cost + 10);
                            }
                        } else {
                            log.entries.push(format!("{} cannot attack, insufficient energy, resting instead", &name.name));
                            rest_or_default(entity, stats, &mut inflict_damage);
                        }
                    }

                }
                ...
```

That's a lot of code, but I think where I see this going is that almost all game logic and rules would probably end up basically in a GameMove system or somethign like that, with some auxiliary systems upstream; and as a result, I think I'd be moving away also from having many different components for each entity, and instead consolidating almost everything into the `CombatStats` component now.

That said, even if it's not deeply exercising the ECS, I still like it - I feel a deep affinity between specs' component/join workflows and the relational database modeling and transformation that I do in my day job.

Aside from that, I'll also just note that Rust continues to impress me - I still don't have a good intuition for borrows, references, etc., but it literally doesn't matter, the compiler just tells me when I do it wrong; I can't say my code is 100% bug-free, but it's been surprisingly close for an ambititous project in a new language.  And most importantly, I'm having fun - side projects die off pretty fast when I'm not enjoying the actual coding.

Up next - I need to add stamina damage and the guard break feature, and then spend a lot of time on polish, enemy AI, balance, and UI.  But I think that gives me a good chance of getting to a good stopping point by Thanksgiving, and seeing where I want to go from there.