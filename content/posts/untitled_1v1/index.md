+++
title = "untitled roguelike test 1"
date = "2022-10-08"
description = "First attempt to embed a rust WebAssembly program into the blog via an iframe"
+++

No idea if this will work, but, if does, you'll see a black terminal screen below with the title screen for `richard's untitled roguelike`:

<canvas id="canvas" width="640" height="480"></canvas>
<script src="./wasm/untitled_1v1.js"></script>
<script>
    window.addEventListener("load", async () => {
    await wasm_bindgen("./wasm/untitled_1v1_bg.wasm");
    });
</script>

I think this is considerably more likely to work on desktop browsers than mobile, but we'll see.

What's happening: I discovered Herbert Wolverson's lovely [Rust Roguelike Tutorial](https://bfnightly.bracketproductions.com/) and it's accompanying [book](https://pragprog.com/titles/hwrust/hands-on-rust/) and [library](https://github.com/amethyst/bracket-lib), and in particular, saw that it's WebAssembly support appeared to be more stable and better documented than anything else I've seen.

The program running above is adapted from chapter 8, changing some of the map rendering logic for vibes, adding the menu based on some later chapters, and my favorite contribution, a targeting system and target/command selection menu on the right hand side, which I think is the first non-trivial Rust I've ever written:

```rust
pub fn update_targeting(ecs: &World, _ctx: &mut Rltk) {
    let mut combat_stats = ecs.write_storage::<CombatStats>();
    let mut players = ecs.write_storage::<Player>();

    let monsters = ecs.read_storage::<Monster>();
    let positions = ecs.read_storage::<Position>();
    let entities = ecs.entities();
    let map = ecs.fetch::<Map>();

    for (_player, player_stats) in (&mut players, &mut combat_stats).join() {
        let mut current_target_seen = false;
        player_stats.visible_targets.clear();
    
        for (entity, _monster, position) in (&entities, &monsters, &positions).join() {
            let idx = map.xy_idx(position.x, position.y);
            if map.visible_tiles[idx] == true {
                if player_stats.current_target == None {
                    console::log("saw new target");
                    player_stats.current_target = Some(entity);
                    current_target_seen = true;
                } else if player_stats.current_target == Some(entity) {
                    current_target_seen = true;
                }
                player_stats.visible_targets.push(entity);
            }
        }    

        if current_target_seen == false && player_stats.current_target != None {
            player_stats.current_target = None;
            console::log("didn't see current target");
        }    
    }
    return;
}
```

This was fun for me - I think the big surprise is that the rust ownership/mutability semantics were a lot less intrusive than I thought they would be, and I was consistently surprised how often the compiler was able to help me - I don't think I was ever stuck on references/mutability stuff for more than 10-15 minutes.  In part that's because I'm working in an engine that solves a lot of this architecturally via the `ecs.read_storage` methods, etc., and design of a framework like this is obviously an enormous undertaking, but still, I'm surprised how not-problematic the whole thing was.

Likewise, compiling to wasm and pushing the wasm to the Github Pages static hosting was straightforward; in this case I just literally inserted a few lines of HTML into the markdown for this post's `index.md`:

```html
<canvas id="canvas" width="640" height="480"></canvas>
<script src="./wasm/untitled_1v1.js"></script>
<script>
    window.addEventListener("load", async () => {
    await wasm_bindgen("./wasm/untitled_1v1_bg.wasm");
    });
</script>
```

Why? I, uh, really enjoy many different kinds of role-playing and similar games, and I find myself spending a fair amount of my free mental bandwidth thinking about them. I think game design in general is fascinating, especially where it works with the same kinds of random and pseudorandom processes I work with in music.  rltk/bracket is especially suited for this, since it has a [very nice implementation](https://github.com/amethyst/bracket-lib/tree/master/bracket-noise) of [perlin noise](https://en.wikipedia.org/wiki/Perlin_noise) and many other aesthetically interesting noise algorithms.

So what's next? I don't consider this project complete yet - it's playable but it's not fun, and not really a game.  I'd like to add a few more systems to make it more engaging: 

1. guard and stamina meters for the player and for monsters
2. stance changes for guard up/guard down
3. guard break
4. separate monster AI for "grunt" and "tank" enemies
5. health regeneration + gradual permanent health damage

I think with those changes, some work on balance and difficulty, and 3-4 different enemies, I'd have something more like a completed prototype, rather than the tech demo I have so far.

But we'll see where we go from here, and I'll write another post with some actual game design ideas and vision once I get there.