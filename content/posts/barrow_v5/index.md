+++
title = "Barrow, v5 (final?)"
date = "2023-01-16"
description = "It's done?"
+++

I think it's done, or at least ready to play.

What is Barrow?  A minimalist roguelike with traditionalist impulses and a modern approach to 
accessibility and quality-of-life.

Explore the dungeon, kill monsters, find gold, go to town, buy stuff, go deeper, get more gold, buy more stuff, beat the boss, get his treasure (hint: it's an amulet) and escape back to town alive.

Barrow allows bump-to-attack combat, but the monsters will punish you for carelessness. Manage your stamina, and use powerful special moves and defensive maneuvers to overcome the obstacles in your path.

Don't forget to heal - healing consumes food.  If you need more food, go back to town buy heading upstairs from the starting point, or hitting T at any upwards staircase from deeper levels.

Finally - there are tooltips - just hover over any monster or action on the right-hand menu.  There is also a list of hints and tips, some spoiler/cheat level, [here](../barrow-hints)

<canvas id="canvas" width="640" height="480"></canvas>
<script src="./wasm/barrow_v5.js"></script>
<script>
    window.addEventListener("load", async () => {
    await wasm_bindgen("./wasm/barrow_v5_bg.wasm");
    });
</script>