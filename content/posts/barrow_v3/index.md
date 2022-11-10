+++
title = "Barrow, v3"
date = "2022-11-09"
description = "Beginning to approximate an actual game"
+++

No longer untitled - I'm calling the WIP game "Barrow", which I suppose is Tolkien-inspired, but also generally evocative of the experience of exploring a dark and ancient stone tomb.  

Try it:

<canvas id="canvas" width="800" height="600"></canvas>
<script src="./wasm/barrow_v3.js"></script>
<script>
    window.addEventListener("load", async () => {
    await wasm_bindgen("./wasm/barrow_v3_bg.wasm");
    });
</script>

What is it about?  Barrow is a roguelike with a traditional lineage, focused on an old-school dungeon crawl, and an emphasis on tactical combat.  Barrow allows bump-to-attack gameplay, but it will punish you for it - you can win a fight against the weakest enemies, goblins, by mindlessly attacking, but the stronger enemies demand use of five more advanced combat strategies:

- power attacks to deplete the opponents health and stamina faster
- swapping stances to keep your guard up and deflect more damage
- waiting for the opponent to deplete their stamina, and conserving your own
- breaking the opponent's guard by depleting their stamina to 0
- taking advantage of the opponents broken guard to rapidly finish them off

As it stands right now, all of these mechanisms are implemented and working, and the game is playable, but it needs more balancing; it also needs a lot of UI love to make these mechanisms visible and intuitive, which is the biggest gap right now.  I did a first pass to make the monster AI a little more dynamic, but there's a lot more to be done on that front as well.

What I'm proudest of right now is the graphics, and especially the lighting/visiblity rendering - I made a flickering light effect using RLTK's lovely noise module - I think this is simplex noise, not perlin noise, but it's pretty either way:

```rust
            let distance = rltk::DistanceAlg::Pythagoras.distance2d(Point::new(x, y), player_pos);
            let dist_factor = 1.0 - ((distance - 3.0).max(0.0)/ 9.0) - (noise.get_noise3d(0.08 * x as f32, 0.08 * y as f32, 0.14 * map.frame_count as f32) * 0.1);

            match tile {
                TileType::Floor => {
                    glyph = rltk::to_cp437('.');
                    fg = RGB::from_f32(
                        0.25 * dist_factor, 
                        0.2 * dist_factor, 
                        0.15 * dist_factor
                    );
                    bg = RGB::from_f32(
                        dist_factor * 0.1,
                        dist_factor * 0.07,
                        dist_factor * 0.05);
                }
                TileType::Wall => {
                    glyph = rltk::to_cp437('#');
                    fg = RGB::from_f32(
                        dist_factor * (0.5 + 0.08 * (noise.get_noise(500.0 + x as f32, 800.0 + y as f32)) ), 
                        dist_factor * (0.5 + 0.08 * (noise.get_noise(500.0 + x as f32, 800.0 + y as f32)) ),
                        dist_factor * (0.5 + 0.08 * (noise.get_noise(500.0 + x as f32, 800.0 + y as f32)) )
                    );
                    bg = RGB::from_f32(0.15,0.1,0.0);
                }
            }
```

This was also the first time I've really hit a roadblock because of Rust's borrow checker.  As part of implementing the new game logic, I merged Wolverson's `MeleeCombatSystem` and `DamageSystem` into a single `ActionSystem` that would apply all the combat rules in a single pass; but what I found is that Rust's borrow checker wouldn't permit me to borrow mutable references to two items in the same vector; in particular, if I need to borrow both the players stats struct as well as the target monsters stats from the same `CombatStats` Storage (which is an abstraction over a sparse vector, hashmap, etc.) then I end up having two mutable borrows of the storage object itself.

The workaround wasn't that bad, just unintuitive - I had to do the borrows separately in bare blocks so they aren't in the same scope:

```rust
                        log.entries.push(format!("{} hits #[orange]{}#[] fiercely for #[orange]{} hp#[].", &name.name, &target_name.name, raw_damage));
                        {
                            let subject_stats = combat_stats.get_mut(entity).unwrap();
                            apply_ep_damage(subject_stats,ep_cost);
                        }
                        {
                            let target_stats = combat_stats.get_mut(*target).unwrap();
                            apply_hp_damage(target_stats, raw_damage);
                            apply_ep_damage(target_stats, ep_damage);
                        }
```

I'm still on the fence about Rust conceptually - it's just not 100% clear to me how much Rust's borrow checker is actually intrusive, and how many legitimate design patterns it gets in the way of.  [Doubly linked lists](https://rust-unofficial.github.io/too-many-lists/index.html) are the classic example, and maybe I am starting to understand Rust well enough to read through that book.

Anyhow - the other big thing missing from Barrow right now is audio, which I don't think RLTK supports at all - that might entail porting it to Bevy or another framework, we'll see, but in the meantime I might want to change it up and work on a few audio projects.