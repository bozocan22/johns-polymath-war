# JOHN KINGDOM GAME

Working title for the game built from the **SHIELDWALL: REFORGED** research
program (see `../shieldwall_reforged/` — brief, critique, feasibility report,
calibrated constants, and validated math prototypes).

## What this is

The actual game codebase. Development follows the vertical-slice plan in
`../shieldwall_reforged/06_VERTICAL_SLICE.md`:

> **Era 1 only** — bloomery → one blade → one 40v40 shield wall,
> end to end, all systems real, nothing faked.

## Status — Milestone 1: the 40v40 physics spike — PASSED

The month-3 kill criterion from the feasibility report applied: if ~80
physics-driven bodies cannot *read as a wall* (hold formation, transmit push,
collapse when cohesion fails), the design falls back to the validated
push-chain hybrid. Verdict from the first spike (see `output/BATTLE_REPORT.md`):

- **Reads as a wall.** With shield colliders (enemy-only collision groups —
  own-side overlap is free, that overlap IS the wall), lines hold Ω ≈ 0.33
  against a 0.333 formation target through a 90 s engagement.
- **Push transmits with diminishing returns.** Depth 1→4 grows front force
  5.1 → 10.1 kN, then plateaus; fitted α ≈ 0.52 with NO authored attenuation.
  The plateau, not α, is the physical anchor (instrumented crowd/scrum band).
  Ranks 4+ buy reserve and replacement, not push — matching the othismos
  scholarship.
- **Collapse grammar is strategic.** Striking down 3 front-rankers doesn't
  unzip the line (ranks step up and dress it); it creates a pressure
  imbalance: the holed side is driven back and its attrition differential
  swings negative. Validated in `cascade_after_front_rank_losses`.
- **Deterministic** (bit-identical same-seed runs) and **fast**: 80 bodies at
  120 Hz run ~30× realtime on 4 container cores — ample headroom toward the
  research ceiling (~400–700 ragdolls on 8 cores with Jolt).

## Milestone 2: third-person direction — SHIPPED

The game is 3D third-person, per the original *Shieldwall*'s core fantasy:
you stand IN the wall (see ADR-005 and `research/05_third_person_design.md`).

- **Squad commands** (`jk_wall::command`): Advance / Hold / Brace / Charge,
  each a reweighting of real physics levers. Validated: charge spikes impact
  force 1.2×+, brace yields less ground than hold under a charge, hold
  recovers stamina, and the same 3-man front loss that a 5-deep wall absorbs
  cascades against a 2-deep wall (`tests/commands.rs`, `tests/validation.rs`).
- **The player is a body in the crush** (`take_player`/`set_player_input`):
  velocity-servo control, force-capped below peak crowd pressure — you can be
  shoved, pinned, and crushed. Deterministic under scripted input.
- **Playable client** (`jk_client`, macroquad): WASD + mouse-look, SHIFT to
  shoulder in, 1–4 orders, SPACE to take over the next man when you go down.
  `cargo run -p jk_client --release` on a machine with a display.
- **Third-person demo** (`output/third_person.gif`): scripted battle rendered
  over-the-shoulder — enemy charge, braced impact, counter-charge.

## Milestone 3: melee combat + rank rotation — SHIPPED

- **Strikes are energy, not damage numbers** (`jk_wall::combat`, brief §2.5):
  a spear thrust carries ~15–25 J (sourced); armor demands energy to defeat
  (cloth ~5 J, gambeson ~30 J, riveted mail ~100 J — research/02 Williams
  data); what penetrates wounds. An all-mail wall bleeds ~3×+ less than an
  all-cloth wall under identical spears (`tests/combat_battle.rs`) — **the
  metallurgy's voice in combat**. Blades from the forge plug into `Weapon`
  in M4.
- **Blocking is state, not RNG flavor**: crushed, exhausted men can't work a
  shield; braced ones block better. Kit follows wealth: mail-rich front rank.
- **Rank rotation** (order key 5): tired file leaders slide back through the
  seam while the second man presses up; the drill auto-reverts. Validated:
  after a 60 s grind the rotated front is meaningfully fresher, and the line
  holds through the swap. Enabled by two physiology fixes: sustained chest
  compression costs energy (why walls rotated at all), and followers LEAN
  (~250 N sustainable) rather than max-shove — only a CHARGE order opens the
  whole column's push. Depth is now truly a fresh reserve.
- **Client**: left-click thrusts (spear cooldown + kills on the HUD), key 5
  rotates. The scripted demo now survives its player: brace the charge,
  rotate the line, counter-charge.

## Milestone 4: morale, the weapon triangle, and scale — SHIPPED

Built to the owner's battlefield-simulator architecture spec (see
`DESIGN_MAP.md` for the full spec-to-system mapping).

- **Morale — "the most important system"** (`agents.fear`): fear flows from
  witnessed falls, chest compression, and local outnumbering; nerve returns
  with distance and the commander's aura (the player IS the leadership
  source — his fall broadcasts fear for 12 m). Breaking men rout (flee,
  can't block, leave gaps), and can rally. Validated: an even bloodless
  grind holds its nerve; a mauled flank fears and flees; the commander's
  fall weighs more than an ordinary man's; routs flee and rally
  (`tests/morale.rs`).
- **Weapon triangle**: spear (long, deliberate, formation), sword (fast,
  short), axe (100 J — opens mail, slow, exhausting), all in the research
  energy bands, mixed through the ranks.
- **Withdraw order (key 6)**: fighting retreat, faces to the enemy — breaks
  contact for the full historical relief drill: *withdraw → rotate → close
  ranks → meet the next pulse with a fresh front*. The sim taught us this
  order was mandatory: rotating in contact lets the enemy pour into the
  opened seams (tested), and a man is wider than a closed-order seam.
- **Scale benchmark** (`bench`): full-fidelity, single-threaded, this
  4-core container: 80 bodies 29×, 256 at 8.7×, 500 at 4.5×, **1024 at
  1.8× realtime** — the spec's 1000+ ladder is reachable before LOD or
  multithreading even enter.
- Emergent find worth keeping: over-hot compression fear produced
  **battle pulses** (mutual disengage-rest-clash cycles — Sabin's pulse
  model of ancient combat, uninvited). Tuned to sensitize rather than
  break; the pulse dynamic remains reachable via casualties.

## Milestone 5a: Bevy 3D client + Tripo3D character pipeline — SHIPPED

`cargo run -p jk_bevy --release` — the real 3D world (per ADR-006):

- **Your AI-generated characters drop straight in**: export from Tripo3D
  as GLB into `assets/characters/` (`commander.glb`, `soldier_a.glb`,
  `soldier_b.glb` — see `assets/README.md`). Missing files fall back to
  low-poly capsule men, so add characters one at a time. Same sim, same
  determinism — the renderer is a skin.
- **Camera feel** (the "swinging-game" lesson: the LENS carries the
  momentum): critically-damped spring follow, mouse orbit with pitch,
  FOV that widens with real body speed (65°→78° at a full charge),
  PBR sun + shadows.
- Full controls (WASD/mouse/SHIFT/LMB/1–6/SPACE), live HUD (order,
  stamina, chest load, cohesion, press, morale/fleeing).
- The macroquad client (`jk_client`) remains as the zero-dependency
  fallback; the sim crates are untouched.

## Milestone 6: Team Deathmatch mode — SHIPPED (v3)

`cargo run -p jk_tdm --release` — a third-person-shooter arena on the same
deterministic 120 Hz foundations (the battle sim is untouched; army dynamics
resume where we left them):

- **Weapons are pickups** — you spawn UNARMED and grab guns from glowing
  pads: handgun, assault rifle, machine gun, sniper, war bow (arrows with
  drop), war spear (heavy ballistic throw). Pads respawn on timers.
- **Global TTK rule: 2 headshots / 5 body shots** for every bullet;
  arrows/spears carry their own mass-based damage.
- **High ground matters**: side plateaus with stairs, a 3 m center tower
  (gravity + step-up on real elevation), trench walls, mirrored crates.
- **Hit feedback both ways**: shooters see WHERE they hit (legs / body /
  shoulder / HEAD + damage), victims get red screen-edge flashes pointing
  at the shooter; persistent bullet marks pock the map (surface-normal
  decals); crosshair flashes on confirm, gold on headshots.
- **Recoil**: camera kicks per shot, bloom widens sustained fire, elbows
  and the gun itself jerk with every round — deliberately not stable.
- **Two modes at the intro screen**: Team Deathmatch (first to 30) and
  King of the Hill (hold the tower 90 s). 5-minute clock with an 80 s
  sudden-death overtime, auto-rematch.
- **Pickups beyond guns**: hidden health/ammo caches behind cover, and the
  ROBOT ARMOR crowning the tower (100 armor that soaks 70 % of damage,
  +12 % move speed, visible bulky shell).
- **Cute Roblox-Spartan fighters**: round heads, metal helmets with
  per-man crest colors, chunky capsule bodies, named roster (Brasidas,
  Xerxes, …) with per-man tunic shades.
- Health bars over every head (+ armor bar), big own HP bar, TAB
  scoreboard (K / D / hits / weapon per man), kill feed by name,
  ESC menu, LMB = aim (per-gun zoom, sniper scope) / RMB = fire.
- fully deterministic: same seed = identical match (sim tests incl.
  unarmed-can't-shoot, pickup-arms-you, TTK rule, elevation gravity,
  arrow ballistics, KOTH scoring, jump, bit-identical replay)

### v4 — robot cowboys, real weapon models, jump, and SOUND

- **Every weapon is a modeled object**: barrels, stocks, magazines, a
  glowing sniper scope, bow limbs + string + nocked arrow, bladed war
  spear — the same detailed model floats over its pickup pad and sits
  in the carrier's hands. Medkits (white box, green cross), ammo crates
  (brass tips), and the robot armor (chest plate, power pack with a
  burning core, pauldrons) are modeled too, worn visibly when taken.
- **You hold the gun**: two-handed grip (left hand crossed to the
  foregrip), elbows kick with every shot; the bow visibly draws when
  you aim; the spear cocks overhead javelin-style and whips forward on
  release.
- **Robot cowboys**: everyone — metal heads with per-man glowing
  visors, proper hats (brim, crown, band; the player rides in white),
  team bandanas, belts with shining gold buckles, boots, and a little
  antenna. Per-man hat and visor colors are the personality.
- **JUMP** (SPACE): real gravity, crate-clearing hops, legs tuck in
  the air. Fire is RIGHT CLICK **or T**; aim/zoom stays LEFT CLICK.
- **Status panel** (bottom right): weapon name, ammo count + ammo kind
  (5.56 / .338 / arrows…), HP + armor numbers and bars.
- **Sound**: procedurally generated WAVs (`assets/audio/gen_sfx.py`) —
  per-gun shots, bow twang, spear whoosh, hit/headshot confirms, hurt,
  pickup chime, reload, jump, kill sting, round-win fanfare, and
  distance-faded enemy gunfire.

### v5 — Phantom-Forces feel: first person, dodge rolls, real legs, castles

- **First OR third person** (V toggles): first person puts real HANDS on
  the screen — gloved robot forearms gripping a full weapon viewmodel
  that bobs with the run, kicks with each shot, and dips through reloads.
  Your own body and health bar leave the frame; the crosshair stays true.
- **DODGE ROLL** (Q, or tap crouch at a sprint): a duck-spin somersault —
  faster than sprinting, balled up small (harder to headshot), gun locked
  while tumbling, short cooldown. **Hard landings breakfall
  automatically**: drop off the tower and the fall rolls out along the
  ground, parkour-style. Ordinary hops still land on their feet.
- **Legs with KNEES and ANKLES**: thigh → shin → foot chains with a real
  gait (hips swing, knees bend hardest on the forward recovery, ankles
  keep the sole level with a toe-off flick), a genuinely deep FULL
  crouch, an air tuck, and the roll's tucked ball.
- **Three battlefields** at the intro screen, all deterministic:
  - *Dust Arena* — the original plateaus / tower / crates range;
  - *Castle Bailey* — a keep with drum corner towers, bailey cross-walls,
    crenellated battlements, a grass courtyard;
  - *Castle Gardens* — green: hedge lanes, ruined garden walls, old trees
    (trunks collide, crowns drawn), a stone gazebo. Castle maps tint the
    sky/fog green and dress the border in stone.
- **Recoil HALVED** across the arsenal (owner request) — camera kick and
  bloom growth both, since both feed off the same per-gun kick.
- **Predicted flight arc**: aim a bow or spear and a red laser of dots
  traces EXACTLY the trajectory the sim will fly (same integrator —
  `predict_arc` is tested to land within 0.6 m of the real projectile),
  ending in a landing ring. Loose the shot; it flies the dots.
- Fixed en route: the v3 plateau-top MG/sniper pads were buried INSIDE
  the plateau and uncollectable — pickups now snap onto the terrain
  under them (regression-tested for every map).
- New roll whoosh SFX; map-validity, roll, breakfall, and arc-accuracy
  tests join the suite.

### v6 — the big one: real skeletons, cute bodies, the CS roster, the SHIELD

Built to the owner's full v6 spec (loadouts, shield, checkpoints, minimap,
settings — every section), with four owner-decided rules locked in:
baseline 2 headshots / 8 body shots; AWM = only the head is instant;
"check back" = capturable respawn checkpoints; battles cap at 8v8.

- **Full joint articulation**: legs hip/knee/ankle (v5) joined by arms
  with shoulder → elbow (hinged, never inverts) → wrist, and HANDS with
  four two-jointed fingers + thumb curled per grip (rifle, pistol,
  forend cradle, shield fist). The arm pose solver keeps the weapon's
  pitch on the crosshair while the elbow carries a visible bend.
- **Rounded, cute restyle**: Baymax construction (spheres + capsules,
  no hard edges), Smurf proportions (big head, short limbs) — hats,
  visors, bandanas and all, still on the same 1.78 m hitbox.
- **Damage model**: 100 HP, zones head ×4 / torso ×1 / arms & legs
  ×0.75. Baseline M4A1: exactly 2 headshots / 8 body shots (tested).
- **CS-style roster under real names, original stats & art**: Glock 17,
  Desert Eagle (the one-tap head stays), MP5, Remington 870 (8 real
  pellets per shell), AK-47, M4A1, AWM, M249 — plus the war bow and
  spear. Per-gun voices (7 new WAVs), dry-fire click, ADS-only detail
  greebles on every model.
- **AWM**: full-screen scope glass (curtains + lens ring + fine cross),
  scoped crawl (×0.35 move), 1.6 s bolt. Only the head is instant.
- **Loadouts (§6)**: primary / secondary / special picked on the intro
  screen, each slot keeps its own magazine, 1/2/3 to switch. The
  SHIELD always rides in its own slot.
- **The shield (§7)**: E raises a tower shield — front ±60° arc only,
  65% cut standing, 95% crouched, sides/rear ignore it ENTIRELY
  (flanking is the counter, by design — tested both ways). No shooting
  while raised, slow walk, bots turtle it when caught reloading.
- **Checkpoints ("check back")**: two rings per map; stand uncontested
  4 s to flip one and your team respawns AT it. Rings tint by owner,
  bots contest them.
- **Difficulty**: Easy / Normal / Hard reshape bot aim σ, reaction
  time, engagement range, and aggression (tested: Hard bleeds you
  harder than Easy on the same seed).
- **Battle size**: 5v5 or 8v8; castle maps grew (Bailey half 34→40 with
  climbable RAMPARTS + parapets, Gardens 34→38 with terraces, a second
  hedge ring, more trees and ruins).
- **Lean** (Z/X): Phantom-Forces peek — eye, muzzle and body tilt
  sideways, recoil ×0.8 while leaning.
- **Minimap (§12)**: teammates, checkpoint rings by owner, the hill,
  your facing needle; M toggles it, so does settings.
- **Settings & manual (§14)**: mouse-button swap (LMB-aim/RMB-fire
  stays the default per the owner), minimap toggle, restart, change
  mode/loadout mid-session; RULES & MANUAL page generated from the
  live weapon table.

Milestone 5b/7 (next): glTF skeletal animation playback from rigged Tripo
exports, terrain, arrows/the forge for the battle game — and cross-mode
polish.

Current contents:

```
engine/                Rust workspace
  crates/jk_core       fixed 120 Hz timestep, deterministic RNG, calibrated constants
  crates/jk_wall       wall sim: capsule agents, push chain, stamina, cohesion, breach
  crates/jk_spike      headless 40v40 battle: metrics + top-down PNG/GIF output
output/                spike artifacts (frames, GIF, battle report)
```

## Running

```sh
cd engine
cargo test              # determinism, alpha-band, cascade-collapse tests
cargo run -p jk_spike --release   # runs the 40v40, writes ../output/
```

## Ground rules (inherited from the research)

- Every constant traces to `../shieldwall_reforged/07_CONSTANTS.md`
  (SOURCED) or is tagged `PROVISIONAL` in `jk_core::constants`.
- Fixed timestep 120 Hz; no variable dt in the solver, ever (Pillar P5).
- Deterministic: same seed → bit-identical replay (tested in CI).
- Physics backend is abstracted (`jk_wall::backend`) — currently Rapier
  (pure-Rust, deterministic, installs anywhere); the feasibility report's
  Jolt recommendation swaps in behind the same trait when the spike
  graduates to a rendered client (ADR-002).
