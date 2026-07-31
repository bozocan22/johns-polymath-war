# What changed — audit passes, fixes, and new systems

A plain-language record of everything changed in this stretch. Written to
be read by someone who was not watching it happen.

**Tests: 49 → 120 green.** Every number below has a test behind it or an
explicit note saying it doesn't.

---

## The method: audit, then verify the audit

Instead of guessing what was broken, two audit waves ran across the whole
codebase — 45 agents total, each subsystem deep-read by its own agent,
and then **every single finding handed to a separate agent whose job was
to prove it wrong.**

| Wave | Subsystems | Claimed | Confirmed | Refuted |
|---|---|---|---|---|
| 1 | minigun, mech, spear, motion/camera | 16 | 12 | 4 |
| 2 | weapons, throwables, bots, abilities, physics | 20 | 18 | 2 |
| 3 | HUD, rendering, persistence, capture harness, modes | 20 | 14 | 6 |

**12 of 56 claims were killed by the verifiers.** That refutation rate is
the reason to trust the other 44 — a review that confirms everything it
finds isn't reviewing. In one case the verifier confirmed the bug but
corrected the finder's explanation of *why* it mattered, and only the
accurate part was fixed.

**Two of the confirmed bugs were mine**, introduced earlier in this same
session. Both are called out below rather than quietly repaired.

---

## Crashes and exploits (the ones that actually mattered)

**A sim panic.** An axe sweep through the zombie horde collected victim
*indices*, then killed them one at a time — but killing uses
`swap_remove`, which moves the last element into the dead slot and
shrinks the list. Any index collected earlier pointing at the old last
slot became invalid, and the next hit **panicked the authoritative
simulation**. One sweep covers a 2.1 m / 90° arc of a horde packed to
0.55 m and one-shots every zombie type, so multi-kill sweeps are the
*normal* case, not an edge case. Now tracks ids instead of positions.

**Team-kill point farming.** Grenades and fire were the only damage
sources without a friendly-fire filter, and the kill-credit code only
checked for *self*-kills. A player could kill their own teammates with a
grenade, score TDM points for it, and **win the match**.

**One key press did two things.** The whole input command is replayed for
each fixed simulation step, and only the shield toggle was being cleared
between steps. So tapping G to switch grenades advanced **two or more
slots** on any frame that ran more than one step — which at 60 FPS is
every frame.

**A grenade that exploded in your hand.** The cook timer wasn't reset
when you switched grenade types. Switching from a nearly-cooked smoke
(1.2 s fuse) to a frag detonated it instantly in your palm; switching the
other way silently defused a frag you'd been cooking.

---

## The bow was barely wired up

The player's bow fires through a completely separate code path from every
other weapon, and that path skipped almost everything:

- **Zero spread.** Not "a little accurate" — *perfectly* accurate, while
  sprinting, airborne, mid-turn. The bow's own accuracy stats were dead
  numbers and the whole stability system didn't apply to it.
- **No auto-reload.** A human had to press R after *every single arrow*,
  while a bot's bow reloaded itself automatically.
- **The aim arc lied.** The trajectory preview drew at a fixed 52 m/s
  while the real arrow leaves at 19–55 m/s depending on draw. So the
  preview was wrong at every draw strength *and completely frozen across
  the draw* — hiding the one thing the draw mechanic exists to teach.

All three now share one code path with every other weapon, so a future
fourth firing path can't skip it either.

---

## Fights that weren't being simulated properly

- **Melee, repulsor and flame all measured attack direction from the
  victim to themselves** — a zero-length vector. Every mech hit read as
  "side armor" regardless of where you actually hit it, and the Folk
  shield brace never matched an attack at all.
- **Zombies ignored armor entirely.** Claw damage was written straight to
  health, skipping armor sets, the brace, the raised shield, and the
  mech's 1000-point hull. **A mech was exactly as soft as a bare soldier
  against zombies.** Now the hull takes the hit first.
- **Zombies clawed people standing above them.** They have no vertical
  simulation, and the attack check only measured floor distance — so the
  horde could hit a player on top of a crate.
- **Respawned bots shot instantly.** A bot's reaction delay froze at the
  moment it died and carried through respawn, so a bot killed mid-
  firefight came back already at zero reaction time.
- **Bot-piloted mechs could crouch** even though the player's couldn't —
  the ban had been added to one path only. And a mech was being *charged*
  the crouch speed penalty for a stance it's not allowed to enter.

---

## Motion and camera (both of these were my own bugs)

**The spear follow-through did the opposite of what it claimed.** I wired
this earlier in the session and wrote a test asserting the torso goes
*still* at release — that test was encoding the bug as if it were the
spec. In reality the torso snapped to neutral, held for 0.125 s, then
swung *backwards* into the wind-up direction. It now continues through
the throw and relaxes, which is what the docs always said.

**The camera spring was governing everything.** I'd wired a heavy spring
meant for recovering from wall collisions, but it ended up filtering
*every* camera movement. Sprinting took ~0.55 s to settle instead of
~0.25 s, aiming released over 0.48 s instead of 0.12 s, and **looking up
snapped while looking down lagged 18 cm behind.** The fix needed explicit
tracking of "am I actually against a wall" — the obvious distance check
can't tell that apart from a camera that's simply catching up. My first
attempt at the fix failed its own new test, which is why the test exists.

**Other motion fixes:** the post-roll landing dip was dropping the
character's head *below the range the hit-detection test guards* — and
the test literally couldn't see it, because the dip was applied after the
function the test samples. The landing "bounce" could never actually
bounce (it ran alongside the dip, smaller and faster, so it could only
shrink it). The minigun's resting barrels looked identical to seized
ones. A hand-fidget animation was written, tested, documented as live —
and never actually reached the screen.

---

## Things the game was telling you that weren't true

- The crosshair never widened as the minigun heated up, even though the
  spread nearly **triples** across a heat cycle — and that widening is
  the minigun's entire drawback.
- Snipers saw a crosshair that ignored the scoped accuracy bonus
  entirely.
- Three separate menu strings advertised **mech jet flight** that was
  removed from the game two briefs ago.
- A "burning" state was tracked by the simulation and displayed nowhere.
  Now burning fighters visibly panic.

---

## New: real settings

Mouse sensitivity and field of view were **compile-time constants** — no
way to change them without rebuilding the game. That alone makes it
unplayable for anyone whose mouse doesn't happen to match the developer's.

- **Mouse sensitivity**, 0.50× to 3.00×
- **Field of view**, 62° to 110° (default moved to 90 — the old fixed 62
  reads as claustrophobic on a wide monitor)
- **Invert look Y**

Aim sensitivity while scoped is measured against *your chosen* FOV, so
picking a wide view doesn't secretly change your aim feel too.

Also fixed a layout bug where the longest menu labels wrapped and
overlapped the rows beneath them.

**New capture path for menus.** The loadout and settings screens never
enter gameplay, so the screenshot system literally could not photograph
them — UI was the one part of this codebase exempt from "prove it with a
picture". `JK_CAPTURE=menus` now snaps both.

---

---

## Wave 3: the instrument itself was lying

Every screenshot-backed claim in this project rests on the capture
harness. Auditing it found two bugs pointing in *opposite* directions:

- It advanced through **every** beat whose time had passed, but only ever
  photographed the last one — so snaps were **silently skipped** on any
  frame that covered more than one beat.
- It then re-photographed that survivor **every frame** until the cursor
  moved. A 4-second run wrote its 5 files **157 times**.

Both fixed with an explicit queue; verified back down to exactly 5.

Worse, it planted capture subjects at `[0,0,0]` while the comment called
that "a KNOWN clear spot (Arena center)". That is the exact centroid of
Arena's 8×8×3 m stone block — **every mech and minigun screenshot in this
repo was staged inside a rock**, relying on collision push-out to shove
the subject somewhere arbitrary. Now searches for a provably clear spot,
and all affected captures were retaken.

And a typo'd `JK_CAPTURE` name produced an empty script, so nothing fired
and the exit never came — the run just hung with a window open, looking
like a slow capture. Now exits with the list of valid names.

## Wave 3: the game was teaching the wrong controls

- The settings row that **changes** the mouse mapping described it
  backwards in *both* states, and the manual repeated the same inversion.
  Default is LEFT=fire; both texts said LEFT=aim.
- The manual also listed `T` as a fire key (it's inspect) and `C` as
  crouch — **`C` is the armour ability, so a player following the manual
  fired the flamethrower while trying to duck.**
- The vitals panel told Folk Armour players to brace with `F`. Brace is
  `C`; `F` is the knife.

All three now derive from one shared source with a test that the
displayed name matches the actual button.

## Wave 3: things that changed how the game plays

- **The minimap was mirrored left-right.** A teammate on your right was
  drawn on your left.
- **The only light in the game never reached the viewmodel.** It had no
  render-layer membership, so Bevy filtered it out of the first-person
  view entirely — the weapon in your hands was lit by ambient alone.
- **Extraction's "no respawns in a run" rule was bypassed** by grenade
  and fire deaths: cooking a grenade at your own feet was a free
  full-heal and re-arm.
- **Grenades, fire, the flamethrower and the repulsor could not damage a
  zombie at all.** Every one looped players only. In Extraction — a
  single-team mode — the flamethrower and repulsor additionally had zero
  possible targets, so the Pyro and Robot armour the map hands out were
  **completely inert in the only mode with a horde.**
- The intro screen's loadout-spec readout was never despawned, so it sat
  in the corner for the entire match (visible in the older captures).

## Determinism (R11), now enforced

Grenade physics is sim-layer, seeded and fixed-timestep, so the same
throw must repeat exactly. 1000 throws from one seed are now compared on
**raw bits** — float `==` would let a NaN or ±0.0 pair pass — plus an
assertion that the aim preview lands within 0.1 mm of the live flight.

That last one matters because of a real finding: most games step a
physics sim to draw the preview and then throw with *different* code, so
the grenade doesn't land on the shown line. This codebase shares one
integrator — and the audit caught the one weapon that had drifted from
that discipline (the bow's preview flew at a stale 52 m/s while the sim
launched at 19–55).

## Named, not done

- The 20-part detachable mech, the castle map, and the full character-
  customization system remain content builds rather than code fixes —
  tracked in `handback/brief-ix/REPORT.md` and the open-work ledger.
- **de Carpentier's closed-form ballistics solver** (a real, verified
  source: solves launch velocity for a given arc height, impact angle, or
  an over-the-wall waypoint) is **not** adopted. This sim's fixed-timestep
  determinism is load-bearing, and swapping the integrator would change
  every existing trajectory and invalidate the golden-value tests. It
  remains a genuine opportunity for throw-assist or bot grenade aim —
  recorded rather than silently skipped.

*(Fixed since first writing: bots not fighting the horde in co-op — see
above. It was a missing AI feature, and it got its own pass.)*
