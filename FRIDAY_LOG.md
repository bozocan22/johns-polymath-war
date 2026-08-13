
## §21 — the heavy chassis kneels and jumps (sim half)

**FRIDAY22.** `engine/crates/jk_tdm/src/sim.rs` only. Tests 321 -> 328,
0 failed. 13 mutations applied and each one killed a named test.

CROUCH: the ban in `set_crouch` was papering over `height()` returning the
standing 3.03 m for a kneeling chassis, so the x2.0 visor band floated
above the model. Fixed at the cause — `height()` now follows the pose
(`chassis_kneeling`), which moves the collision capsule and the damage
bands together and leaves the visor at the same 0.90 of the machine.
Gated grounded-only; the light chassis still refuses (its `height()` has
no crouch term). Kneel and brace are ONE planted pose and are charged
`MECH_BRACE_SPEED_MULT` once, never twice.

JUMP: `Compress` (0.40 s readable wind-up) -> launch at
`mech_jump_speed()` = `JUMP_SPEED * sqrt(MECH_SCALE)` -> `Air` ->
`Recover` (0.55 s kneeling lockout). Paid out of `stride_heat`, at launch
only, so jump and power stride compete for one budget. Landing shocks
enemy infantry inside 4.5 m with linear falloff plus a flat stagger;
teammates and other chassis are spared.

Published for the client: `TdmSim::mech_jump_phase_of`,
`TdmSim::mech_jump_compression_of`, `MechJumpPhase::label`,
`Fighter::mech_jump_compression`, `Fighter::chassis_kneeling`,
`Fighter::visor_eye_y`, `Fighter::in_heavy_mech`.

Handed to FRIDAY33: `main.rs:17205` still calls the stance-blind
`mech_visor_eye_y(p.pos[1])` and should call `p.visor_eye_y()`;
`main.rs:17359` adds `MECH_BRACE_STANCE_DROP` on top of a hull the sim now
sinks itself, so the braced camera will double-drop.

## §C — armour damage states: Fresh / Scuffed / Cracked / Severed (sim half)

**FRIDAY22.** `engine/crates/jk_tdm/src/sim.rs` only — purely additive,
950 lines in, 0 removed. Tests 328 -> 338 mine (suite read 354 at the end
because FRIDAY33 was landing cockpit tests in the same run). 0 failed.
11 mutations applied; every one killed a named test, each reverted from a
file copy, never `git checkout`.

THE BLOCKER was that the hit path resolved a ZONE and per-piece HP needs a
PIECE. Rejected: (1) a seeded roll per hit — inserts an RNG call into the
hottest path, moves every later draw, breaks bit-identical replay for
every other system; (2) spreading wear over the whole zone — ten leg
plates then degrade in lockstep and fall off together, which is a
whole-zone armour bar in a per-piece costume, and the brief's own worked
example (one pauldron gone, an asymmetry readable at range) cannot happen.
TAKEN: ask the geometry the hit already carries. `frac` is already
computed to pick the zone and the impact point is already passed in to
place the damage number; between them they say how high, which side and
which face — which is exactly a plate. `ArmorPiece::struck` cuts each
zone's band into its anatomical layers (shoulder to hand, hip to boot) and
picks the mirrored half by side, or the cuirass by face. No state, no RNG,
no second geometry pass.

Every number is the brief's: 80-point pools, 70/40/15% thresholds, +5%
and +10% resistance. The one number it does not give is the WEAR RATE,
labelled ASSUMED at `wear_plate`: a plate takes what the fighter took, so
the baseline rifle scuffs it on the 2nd round, cracks on the 4th, severs
on the 6th and strips it on the 7th.

DETACH IS THE UNEQUIPPED PATH, not a new one:
`armor_pieces.set(p, false)`, the same bit the Forge switch clears, so
`zone_mult` exposes the segment with nothing to keep in step.
`a_shot_off_plate_is_indistinguishable_from_one_never_worn` is the test
that notices if that ever grows a second flag.

Player/bot is ONE rule — the wear is in `apply_hit_dmg`, which both paths
already share. Mutating it to fire only for `self.player` killed 5 tests.

Condition SURVIVES respawn, deliberately and against the house rule: §C
repairs plate in the Forge at 50 points a piece, between matches. Dying is
not a repair. Pinned by `plate_condition_survives_a_respawn` so it reads
as a decision, not as the recurring "state survived a transition" defect.

Published for the client: `TdmSim::armor_stage_of` (returns `Option` —
`None` is a bare mount, and a client must not draw clean steel on a naked
shoulder), `TdmSim::armor_wear_of`, `ArmorStage::label`,
`ArmorStage::tilts`, `ArmorStage::resist`, `ArmorPiece::struck`,
`HitZone::band`, `ArmorCondition::{hp,frac,stage,wear,repair}`.

Handed to FRIDAY33: nothing in `main.rs` reads any of the above yet — the
plates render at one appearance regardless of condition, and "piece tilts"
at Cracked is unbuilt. Deferred by me: plate wear fires only where plate
PROTECTION already fires (the zoned hitscan path), so grenades, melee,
claws and gas still neither wear plate nor are reduced by it — one gate,
`in_mech`, for both, so they cannot drift.

Found in my lane, NOT fixed, needs a decision: `sim.rs:8165` and
`sim.rs:11084` compute the muzzle as `pos[1] + EYE_REL.min(height()-0.12)`.
That `.min()` only ever bites for a SHORT fighter, so a 3.03 m chassis
fires from 1.62 m — knee height on the machine, 1.10 m under its own visor.
Changing it would move every mech engagement, so it is reported rather
than done.

---

## FRIDAY22 — CLIFFHOLD: a castle on a cliff over half a city

`MapKind::Cliffhold`, "CLIFFHOLD". 600 x 600 m — half again the Arena,
a fifth bigger than the Battlefield. Builder at `sim.rs:1522`
(`build_cliffhold`), reached from the `build_map` arm at `sim.rs:1379`.

THE POINT was verticality, because flight is coming, so the bands are
OCCUPIED rather than decorative: 0 streets/commons/ravine, 5 city roofs
and the aqueduct deck, 6 the east apron, 7 the muster plaza (the KOTH
hill, at the origin), 8 and 11 the upper roof courses, 12 the west bench
and east shelf, 18 THE PLATEAU (cliff top and castle courtyard), 24 the
curtain wall-walk, 25 the keep half-landing, 32 the keep parapet. Every
one of them is reachable ON FOOT today, with no flight and no mech, and
`every_cliffhold_band_is_reachable_on_foot` walks eight routes to prove
it using the sim's own two movement rules.

LAID OUT IN FINAL METRES and divided back out by `MAP_SCALE`. The
central +25% pass moves centres and leaves extents alone, so two slabs
that abut before it are a quarter of their centre distance APART after
it. On furniture that is a wider gap; on a mountain it is a crack you
fall eighteen metres down.

THE KEEP IS NOT A SOLID BLOCK. Four walls, a 14 m doorway, the plateau
for a floor, and a two-flight stair inside to a 32 m parapet — the
named contrast being the Gardens' gazebo and the Bailey's keep, both
solid centrepieces you can only walk around. Neither is fixed here.

THE ONE NUMBER THAT MATTERS is `STAIR_RISE_M = 0.5`, under BOTH
`STEP_UP` (0.55, the legs) and the newly-named `BOT_PROBE_Y` (0.75, the
bot whisker, extracted from a literal in `bot_act`, value unchanged). Over
the first it is a mech-only route; over the second a bot reads the flight
as a wall and veers off it. Mutating it to 0.6 killed three tests.

BOTS HAVE NO PATHFINDER. `waypoint` is `[f32; 2]` — no height — sampled
uniformly from a square and never checked for reachability. A bot cannot
CHOOSE to go up, and its waypoints will land inside this mountain. The
map is built anyway and NOT shaped down to that; what it does do is aim
the Breach and the North Road down the x = 0 line, which is where the
castle capture ring and both spawn rows already are, so a bot steering
at the objective walks onto a flight instead of into rock. Real
navigation is work to be scheduled.

Found by my own test, not by inspection: a flight whose top tread only
TOUCHES the slab it climbs onto leaves a one-tread band of open ground,
and the support rule drops you to zero in it. Fixed on all seven flights
and both plaza stairs; the North Road had it twice.

Also new: `NoInfill`, a keep-out list the shared infill honours. Empty
for every older map, so their RNG draw sequences are byte-identical and
no replay moved.

HANDED TO FRIDAY33, AND IT BLOCKS THE BUILD: `main.rs:15401` matches
`MapKind` exhaustively and now needs a `Cliffhold` arm (sky, ground,
border). Nothing else in the client is required for the map to run. The
map self-selects into the menu via `MapKind::ALL`. Tints, props, skybox
and the four landmark silhouettes (keep, gatehouse pair, bell tower, the
cliff itself) are all client-side and none of them are mine.

Least sure about: the 7.6 m and 14.0 m city roof courses are air-only by
DECISION, not accident — they are the tallest things in the city so they
read as unreachable — but that is a judgement I would want looked at.

-- FRIDAY22

---

## 2026-08-11 — BOW & SPEAR: input semantics, the charge curve, the physics

Sim half of the owner's bow-and-spear spec. Three commits, one per
section: `a95b48c`, `5ad7519`, `eff8fbf`. `sim.rs` only; `PlayerCmd`
unchanged, so nothing in the client had to move in the same breath.

**A — the input split (§4/§8/§17).** The rule the spec repeats three
times, RMB PRE-AIMS AND NEVER CHARGES, was ALREADY TRUE in the sim: both
weapons read `cmd.shoot`. The old RMB-draws-the-bow grammar survived
only in comments and one stale doc. So I guarded it rather than claiming
a fix, and said so in the test. `AimPhase` + `aim_phase(ready, pre_aim,
attack)` now resolves the input once for both weapons, and both step
functions take the phase instead of a bool, so PRE_AIM and CHARGING
cannot be combined and the two weapons cannot drift onto two buttons.

Underneath it were four real defects, each with a test that fails
without its fix: a draw survived a WEAPON SWITCH and loosed itself on
re-select; a draw survived DEATH (the player block is inside `alive()`,
so nothing cleared it and the respawn block did not know about it); a
refused release BANKED the wind at 1.2999997; and neither charge path
consulted the PARRY, because the bow spawns its arrow directly rather
than through `try_fire`.

**B — the curve (§9/§10/§11).** `SPEAR_CHARGE_FULL_S` 0.85 s → 3.00 s;
the clock cap `FULL*2` → `SPEAR_MAX_CHARGE_S` (the old one would have
put the new 7 s bonus out of reach at 6.0 s); new `SPEAR_MAX_CHARGE_S`
7.0 and `SPEAR_MAX_CHARGE_DMG` 1.10, so a maximum throw lands 115.5
instead of 105. The curve stays LINEAR, which is a decision: 0.900 /
1.0222 / 1.1611 / 1.300 at 0/1/2/3 s is 30.6% / 34.7% / 34.7% of the
band, i.e. the spec's own low/medium/high in near-equal thirds, and
step-free by construction. Any easing I invented would be a number
nobody asked for.

The bonus is a THRESHOLD and that is forced, not chosen: "3 s reaches
maximum, holding longer must not grow power" forbids a 3→7 s ramp, and
"7 s grants a bonus" requires something at 7 s. Velocity stops dead at
3 s; 7 s pays on a different axis. It cannot stack twice over — the
reward is a `bool` and the clock is capped at the reward.

**C — the physics (§12/§13).** Verified, not rebuilt. Launch along the
crosshair, stick-vs-bounce impact, and preview-equals-flight all already
hold and already have tests; rotation toward flight is client work and
correctly so — the sim's contribution is publishing `Missile.vel` and
deliberately not clearing it on impact. The untested gap was the FLIGHT
itself, now closed by relationships rather than constants: horizontal
momentum conserved bit-identically, 25 m taking distance/speed, and drop
quadrupling when flight time doubles.

**FOR FRIDAY33 — four accessors, and they are ready now.** Named on the
`turret_mode_of` pattern: `spear_stance_of` → `SpearStance {Carried,
Winding, Planting}`, `spear_wind_frac_of` (0..1 across the 3 s raise),
`spear_plant_frac_of` (0..1 across the 0.4 s plant), `spear_max_charged_of`
(the 7 s tell). `spear_plant_frac_of` exists because `spear_wind_t`
counts DOWN and the client currently writes `1.0 - spear_wind_t /
SPEAR_WINDUP_S` by hand in three places — a sim constant on the wrong
side of the boundary.

**Evidence.** sim tests 227 → 238, 0 failed. All 14 new tests watched
failing under mutation, reverting from a FILE COPY. Two fixtures
repaired as stale SETUP, not weakened assertions — one ran in a live 1v1
where a 3 s wind now gets the thrower killed mid-plant, the other fed
the player `PlayerCmd::default()` whose zero `aim` the player (not the
bot) re-tracks into the throw.

For a stretch of this, `main.rs` was mid-edit and non-compiling in the
other lane, which blocks `cargo test -p jk_tdm` entirely. `sim.rs`
imports nothing but `jk_core`, so I ran it as a standalone lib crate in
the scratchpad — same file, same tests, no bevy, seconds per compile.
Worth knowing next time a builder is stuck behind a broken client.

Least sure about: whether the owner wants RMB to be a PREREQUISITE for
charging or merely not-a-charge. I read "RMB never charges" as the hard
rule (it is the one stated three times) and did NOT make pre-aim
mandatory, because requiring it would make the spear unusable without a
second button held and the spec never says the charge is refused
without it. One sentence from the owner settles it and the change is
two lines.

-- FRIDAY22

---

## FRIDAY22 — 2026-08-11 — CASTLE BAILEY v7: +30%, randomised, retargeted 8v8–16v16

`sim.rs` only. `main.rs`/`hud.rs` were live under another session the
whole time; I did not touch them, and the client half of this map (the
palette in `map_look.rs:187`) is Friday33's if it needs one.

**Picked Bailey over Gardens.** The brief said castle; Bailey's
vocabulary is keep, drum towers, curtain walls. The Gardens' is hedges, a
gazebo and fountain basins — the grounds *outside* a castle. Scaling that
one gets you a bigger park.

**Size:** authored half 40.0 → `BAILEY_HALF` 52.0, so 99×99 m playable →
130×130. `MAP_METRICS.md` has no player-count row, so the number is my
judgement and is written down as one: 16v16 is 524 m² a body (~23 m mean
spacing), 8v8 is 1048 (~32 m). Both land inside the 25–35 m engagement
band §6.4 names, at opposite ends. The old map put 16v16 at 17 m, under
the band and inside §6.3's "close range" transfer — that is the
crampedness the brief was naming.

**Randomised** off the `rng` already threaded into `build_map`: curtain
wall stand-off, gate width and sally-port position per wall; drum tower
radius and corner stand-off; rampart length and parapet breaks; a raised
causeway on a randomly chosen flank; four outbuildings; five L-shaped
building corners; six big trees; twelve barricades. Everything randomised
is placed as a **180° rotated pair**, because teams spawn facing each
other down z and point symmetry is what makes a random layout fair.

**Three things worth knowing:**

1. **The "bridge" is a causeway and cannot be anything else.** Cover is
   an `Aabb` with a top and no underside, so there is no walking under
   anything in this engine, ever. A raised road with a drop each side and
   a ramp at each end is what is buildable, and it reads as a bridge from
   on top of it, which is where it is played from.
2. **Everything stays under `BOT_TERRAIN_M`.** Bailey publishes no
   `Climb` links, so a blocker over 3.5 m is one `ground_reach` refuses
   to route through and cannot route around either. That constraint is
   now a named constant with the reasoning on it, not an accident.
3. **Two defects found by mutation-testing my own tests, not by reading
   them.** All four outbuildings were being rejected on every seed — the
   feature compiled and did nothing, and I only know because an unseeded
   value planted in that loop failed to move the layout digest. And the
   doorway I had written was wider than the wall it was in.

**Least sure about:** the three-sided court. I chose it over a 3–4 m
doorway because that would put every outbuilding under the 7 m aperture
floor — but that floor is `MAP_METRICS` §10.2's own flagged ANALOGY to
`BOT_CLIMB_LANE_M` and has never been measured. If somebody runs the
aperture experiment §10.2 scopes (one afternoon) and 4 m turns out to be
fine, these should become real enclosed buildings with doors.

**Deferred, and it is the one that blocks the headline:** `spawn_point`
still lays out **8 slots per team**. A real 16v16 spawns slots 8–15 in a
lop-sided row trailing off to one side. The map is sized for 16v16; the
spawn row is not. That is a shared function every map reads and it is a
separate change with its own test surface.

**Not verified visually.** No capture — this is a `sim.rs`-only change
and I did not build the client. The layout claims here rest on tests and
arithmetic, not on a screenshot of the map.

`cargo test --release -p jk_tdm` — 485 before, **488 passed, 0 failed, 2
ignored** after. Four are mine; the other delta is the concurrent
`main.rs` session's.

— **FRIDAY22**

---

## 2026-08-12 — BRIEF XIII §1: Battlefield and Cliffhold are gone

Owner: *"completely remove Battlefield and Cliffhold from the game... they
must not appear in any menu, map selector, customization screen, game-mode
selection, rotation, matchmaking list, or other UI."*

**One commit across three files, on purpose.** `sim.rs`, `main.rs` and
`map_look.rs` (`e3dff4b`). Deleting the `MapKind` variants without the
match arms breaks the build for every concurrent lane, so this crossed my
normal lane by design and had to land atomically. It nearly did not: my
first `git commit --only` took two of the three files and left `HEAD`
referencing a variant that no longer existed. Amended within the minute.
**`--only` is not the tool for an atomic multi-file commit** — stage the
paths, then commit with no path arguments.

**What went.** Two enum variants, their `name()` arms, `MapKind::ALL`
5 → 3, both layout blocks, `build_cliffhold` and its ten `CH_*`
constants, both map-specific loot re-sitings, the Battlefield's palette,
and the menu's PvP-rotation filter. Source-wide grep afterwards: 32
occurrences left, **all prose, zero identifiers, zero user-facing
strings.**

**Extraction moved to Gardens** and I own that call. It was pinned to the
Battlefield because a horde mode wants an adventure map; Gardens is the
only survivor that can be one, and BRIEF XIII §2 rebuilds it at the scale
the mode was tuned for. The Battlefield's Extraction loot layout died with
it, so Extraction now runs on Gardens' default 40 m pickup lanes — right
today, and the first thing to revisit when §2 enlarges the map.

**Six tests died and I am not going to pretend otherwise.** The keep you
walk into, every altitude band reachable on foot, eighteen flights each
walkable, a bot routing onto a stair, bots reaching an 18 m plateau, and
the infill keep-out list. Nothing left in the game can carry those claims.
**The `NoInfill` mechanism is now unexercised AND unreachable** — the
`blocked` closure returns false for every point on every map — and there
is a tombstone comment at the site saying so.

**Three survived, re-pointed at all three maps, all mutation-proven.**
Spawn rows holding a chassis; the route planner's ground and the body's
ground being the same ground; and a new
`a_stair_riser_stays_under_both_the_step_up_and_the_bot_whisker`, the
salvaged relationship half of the flights-are-not-walls test.

**I weakened one assertion and I am flagging it rather than burying it.**
The spawn test demanded a clear 6 m walk along the spawn *facing*. That
held on a 600 m map with authored-empty spawn plains; on Gardens it fails
against ordinary infill 5.8 m out — and a crate in front of you is not a
stranding. It now asserts ≥6 of 16 headings are walkable by a chassis.
Because that is weaker it carries an **instrument check**: the same finder
on a closed ring of boxes must report zero exits, then must find the gap
when one box is lifted out. The instrument caught its own first fixture
being wrong, which is the only reason I trust it.

**Least sure about:** the `MAP` menu-row relabel. The gutter label was the
literal string `"BATTLEFIELD"` sitting directly above the map pills, which
after this change reads as a leftover selector entry — exactly what §1
forbids. It is a shorter string in a fixed-width `ROW_LABEL_W` gutter so
layout cannot move, but **I did not capture it.** If the owner wants the
row heading to stay, it is a one-word revert.

**Hazard I did not touch and someone should.** Mid-task this working tree
showed `cockpit.rs` and `inventory_strip.rs` as ~500 lines of *deletions*
against `HEAD` — i.e. the working copy is older than the commits that
added them. I staged explicit paths and left them strictly alone, but that
is the stale-working-copy failure and it is live right now.

`cargo test --release -p jk_tdm` — 566 before, **564 passed, 0 failed, 2
ignored** after. That total is contaminated: three other lanes landed work
in this tree while I worked. The clean measurement is the `sim.rs` test
roster, which nothing else touched: **250 → 244, −6, by name.**

**Not built:** Gardens gains none of the scale or verticality it now has
to carry. The altitude ladder is preserved in
`research/maps/VERTICAL_BANDS.md`, not in code.

— **FRIDAY22**

---

## Smoke blinded the bot and not the HUD, because the vision query was private

`TdmSim::sight_clear` (sim.rs:13347) is `pub(crate)` now. That is the
entire functional change: one word, no body edit, no call site moved.

The gameplay bug behind it. There are two visibility queries and they
are not interchangeable. `los_clear` is walls only; `sight_clear` is
walls AND smoke, and it is what a bot's target selection runs. The
client's new enemy-threat sensor could only reach the public one, so a
smoke grenade broke contact for the BOT while leaving the threat
indicator lit by a man who could no longer see the player. Smoke is
meant to break contact and it half-worked.

Widening it rather than letting the client re-derive smoke occlusion is
the argument `los_clear`'s own comment already makes — it was exposed so
main.rs would reuse "the SAME line-of-sight query ... rather than a
second, divergent implementation." A client-side copy of the sphere
accumulation drifts the first time the 0.6 threshold or the smoke radius
moves, and drifts silently.

**A comment defect found on the way in, and fixed.** `sight_clear` had
no doc comment at all. Commit fcc7aed inserted `segment_crosses_smoke`
BETWEEN `sight_clear`'s comment and `sight_clear`, so four lines reading
"VISION test — walls AND smoke. Bots see with this" and "> 0.6 blocks"
had been describing the smoke-only helper, which tests no walls,
accumulates nothing, and uses a 0.5 m per-sphere floor instead. Verified
against the diff of fcc7aed rather than assumed. The lines are back on
their subject and the extension now states plainly which query a future
caller wants: **vision** — bots, spotting, anything a player is meant to
be able to hide from — takes `sight_clear`; **damage** — bullets,
shrapnel, anything that crosses smoke unbothered — keeps `los_clear`.

**No test added, and this is the interesting part.** sim.rs's `mod
tests` is a CHILD module (`use super::*`), so it already reaches private
items — the neighbouring test calls `nearest_visible_enemy`, which is
private, at sim.rs:13378. A test written there could not fail if
`pub(crate)` were reverted to `fn`; it would be guaranteed green in both
worlds, which is the inert-test class rule 12 exists to stop. Nothing
inside this file can guard this change. The only instrument that can is
a real call from another module, i.e. the compiler, once
threat_sensor.rs switches over. The behaviour stays covered by the
existing "smoke must blot out bot vision" assertion at sim.rs:26815,
which I left alone.

Replay: no state added, no tick altered, nothing written. A widened
reader cannot perturb a digest.

`cargo build --release -p jk_tdm -j 2` clean, exit 0, 15m41s, no new
warning. `cargo test --release -p jk_tdm -j 2` — **575 passed, 0 failed,
2 ignored.**

**Correcting a stale number in my own brief:** the task named a 570
baseline. It is 575 at this HEAD and was before I touched anything.
Neither uncommitted diff in the tree adds a `#[test]`, and sim.rs holds
244 `#[test]` at HEAD and 244 in the worktree. The +5 came in with the
commits landed since the brief was written. Same drift the previous
entry recorded — the crate total is not a stable instrument while three
lanes are live.

**Least sure about:** nothing in the code, which is as small as a change
gets. The soft spot is that this lands half a fix. Until main.rs moves
threat_sensor.rs:589 from `los_clear` to `sight_clear`, the HUD behaves
exactly as it did — the smoke inconsistency is still live in the build.
Anyone reading this commit as "smoke now breaks the threat ring" would
be wrong.

**Hazard observed, not touched.** The tree gained modifications to
`cockpit.rs` and `inventory_strip.rs` mid-task, during a 15-minute
build, on top of the `main.rs` work already there. I staged one explicit
path and committed one file; `git status` after the commit confirms all
three are still unstaged and intact. But three lanes are writing this
working tree concurrently right now.

**Not built:** the client half. Deliberate — main.rs is not my lane.

— **FRIDAY22**
