+++
title = "WebAssembly embed test"
date = "2022-10-08"
description = "First attempt to embed a rust WebAssembly program into the blog via an iframe"
+++

No idea if this will work, but:

<iframe width="656" height="496" frameBorder=0 src="./wrapper.html"></iframe>

If the above shows a black screen with the text `Hello Rust World` - then we're in business.  I think it is considerably more likely to work on desktop browsers than mobile, but we'll see.

What's happening: I discovered Herbert Wolverson's lovely [Rust Roguelike Tutorial](https://bfnightly.bracketproductions.com/) and it's accompanying [book](https://pragprog.com/titles/hwrust/hands-on-rust/) and [library](https://github.com/amethyst/bracket-lib), and in particular, saw that it's WebAssembly support appeared to be more stable and better documented than anything else I've seen.

Of course, nothing about WebAssembly is particularly stable and documented, but I was able to get the "hello, world" running locally, both native and in wasm, after an hour or two of poking around, despite not understanding how cargo works AT ALL.

Why? I, uh, really like roguelike and other kinds of dorky rpg games, and I find myself spending a fair amount of my free mental bandwidth thinking about them. I think game design in general is fascinating, especially where it works with the same kinds of random and pseudorandom processes I work with in music.  rltk/bracket is especially suited for this, since it has a [very nice implementation](https://github.com/amethyst/bracket-lib/tree/master/bracket-noise) of [perlin noise](https://en.wikipedia.org/wiki/Perlin_noise) and many other aesthetically interesting noise algorithms.

But - I don't really "get" gamedev, with its elaborate IDE's, gnarly packaging and releasing, and corporate platform gatekeepers, and I'm not particularly interested in learning that world.  And I'm especially uninterested in spending years of my life meticulously crafting a commercial, releasable game.

Instead, I'm hoping that this zola/rust/webassembly toolchain gives me a platform for building small, fun programs, that either exhibit some simple game design ideas, or just pure generative audiovisual work, that I can embed in a essay-like format - indeed, if the above link works, this post itself arguably accomplishes that;

e.g., the code for the above looks like this:

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
    let context = RltkBuilder::simple80x50()
        .with_title("Roguelike Tutorial")
        .build()?;
    let gs = State{ };
    rltk::main_loop(context, gs)
}
```

I'm not willing to go as far as Knuth's [literate programming](https://www-cs-faculty.stanford.edu/~knuth/lp.html) approach; instead, I'm hoping each post in this format works more or less like the kind of blog posts or book chapters I used to write - an intersting program of medium length, and a medium-length essay describing how it works in whatever level of detail is interesting.

I won't speculate too much about where it goes from here - let's see if this works at all, and how reliably - but if it does, I'm excited to start hacking on things like this a lot more often.