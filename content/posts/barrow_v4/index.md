+++
title = "Barrow, v4"
date = "2022-12-10"
description = "Another draft, getting closer"
+++

A month later, more progress - more refinement, and a lot of slow design decisions.

The biggest changes were:
- completely rewrote the combat system
  - de-emphasized the rock-paper-scissors stance system
  - modeled stance changes as part of the move system
  - added multiple reaction/recovery moves
  - de-emphasized guard break mechanic, focusing on damage bonuses
- added tooltips for monsters and commands
- reworked monster AI
  - monsters understand how to manage their own stamina
  - some are more conservative than others

What's next?
I think this is quite close to a point where I could start getting more significant feedback from folks, but first:
- tweak the balance to emphasize the intent of the combat system
- more detail and hints in the tooltips
- hone the AI until the difficulty is right

What might come after that?
- Items, especially potions
- More floors, more enemies
- Gear, experience, or other progression
- Larger floors, more variety, camera system
- Gold, scorekeeping, or other stats
- Flavor text

With all of the above, I can see Barrow as a complete game of modest ambitions, with a 6-8 floor dungeon, that takes around 45 minutes to complete.

After that, there are other ideas that I'd want to explore in subsequent games, but not in Barrow:
- Multiple classes/build variety
- Ranged combat
- Move variety:
  - Moves with charge times
  - Moves with cooldowns
  - Free moves
- Combat movement system
- Status effect systems in general
- More sophisticated AI
- Proper boss fights
- Food/Crafting
- Followers
- Overworld exploration
- Towns and NPC's
- Story/Dialogue
- Art

<canvas id="canvas" width="640" height="480"></canvas>
<script src="./wasm/barrow_v4.js"></script>
<script>
    window.addEventListener("load", async () => {
    await wasm_bindgen("./wasm/barrow_v4_bg.wasm");
    });
</script>