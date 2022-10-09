+++
title = "WebAssembly embed test"
date = "2022-10-08"
description = "First attempt to embed a rust WebAssembly program into the blog via an iframe"
+++

No idea if this will work, but, if does, you'll see a black terminal screen below with the text `Hello Rust World`:

<canvas id="canvas" width="640" height="480"></canvas>
<script src="./wasm/wasm_test.js"></script>
<script>
    window.addEventListener("load", async () => {
    await wasm_bindgen("./wasm/wasm_test_bg.wasm");
    });
</script>

I think this is considerably more likely to work on desktop browsers than mobile, but we'll see.

What's happening: I discovered Herbert Wolverson's lovely [Rust Roguelike Tutorial](https://bfnightly.bracketproductions.com/) and it's accompanying [book](https://pragprog.com/titles/hwrust/hands-on-rust/) and [library](https://github.com/amethyst/bracket-lib), and in particular, saw that it's WebAssembly support appeared to be more stable and better documented than anything else I've seen.

Here's the program running above:

```rust
use rltk::{Rltk, GameState};

struct State {}
impl GameState for State {
    fn tick(&mut self, ctx : &mut Rltk) {
        ctx.cls();
        ctx.print(1, 1, "Hello Rust World");
    }
}

fn main() -> rltk::BError {
    use rltk::RltkBuilder;
    let context = RltkBuilder::vga(80,30)
        .with_title("Roguelike Tutorial")
        .build()?;
    let gs = State{ };
    rltk::main_loop(context, gs)
}
```

And when I say "running above" I mean it - the Rust code above is compiled into webassembly, deployed to this Github Pages static site, and runs in the browser via a small js shim which is embedded in the markdown for this post; literally it's just embedded in the index.md file:

```html
<canvas id="canvas" width="640" height="480"></canvas>
<script src="./wasm/wasm_test.js"></script>
<script>
    window.addEventListener("load", async () => {
    await wasm_bindgen("./wasm/wasm_test_bg.wasm");
    });
</script>
```

Why? I, uh, really enjoy many different kinds of role-playing and similar games, and I find myself spending a fair amount of my free mental bandwidth thinking about them. I think game design in general is fascinating, especially where it works with the same kinds of random and pseudorandom processes I work with in music.  rltk/bracket is especially suited for this, since it has a [very nice implementation](https://github.com/amethyst/bracket-lib/tree/master/bracket-noise) of [perlin noise](https://en.wikipedia.org/wiki/Perlin_noise) and many other aesthetically interesting noise algorithms.

But - I don't really "get" gamedev, I don't like IDE's or platforms with gatekeepers, and I don't particularly want to spend years of my life crafting a major side project. Instead, I'm hoping that this zola/rust/webassembly toolchain gives me a platform for building small, fun programs, that either exhibit some simple game design ideas, or just pure generative audiovisual work, that I can embed in a essay-like format - indeed, if the above program works, this post itself arguably accomplishes that.

I won't speculate too much about where it all goes from here - let's see if this works at all, and how reliably - but if it does, I'm excited to start hacking on things like this a lot more often.