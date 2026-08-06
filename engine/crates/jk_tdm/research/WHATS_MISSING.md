# What is missing / not built yet — 2026-08-06 (rev 3)

Compiled from `BACKLOG.md`, `THOR_LOG.md`'s ranked findings, and session
knowledge. Ordering inside each tier is Thor's ranking rule: an item
moves up when its blocker clears, not when it becomes interesting.

## 0. Needs the USER (nothing code-side can proceed)

Nothing. The three branding PNGs landed 2026-08-04 (key art, wordmark,
emblem) and are wired through the splash, menus, and seal footers.

## 1. Small, one-session buildable (ranked, next up)

1. **Derive the screen-intrusion budgets from the geometry.** The three
   profiles now exist and every weapon is swept under its own across
   every sustained pose — but the bounded-part extents are still
   *audited* off the model tables, not *measured* from them, so widening
   a weapon model fails nothing. Unblocker: lift the `match kind` part
   tables out of `spawn_weapon_model` into a pure
   `weapon_parts(kind) -> Vec<WPart>` the tests can call. The Minigun's
   barrel spinner is the one arm that spawns an entity mid-match and
   will need splitting out.
2. **HUD award toasts for a resource economy** — the toast system
   itself shipped (kills/assists/parries/streaks). §4.3's *resource*
   awards still have no economy in TDM/KOTH to award from, so this
   waits on a mode that has one rather than being faked.
3. **A HUD read for suppression.** The mechanic landed; the player's
   feedback is a viewmodel shake and nothing else. Whether that wants a
   directional tell (which way the fire is coming from) is a design
   question, not a gap — recorded so the decision is made rather than
   defaulted into.

### Closed 2026-08-06

| Item | How it closed |
|---|---|
| **Helmets were a colour, not a shape** | Five helmet SHAPES (FIELD CAP, VISOR, CREST, HOOD, HORNS) as a data table of primitive pieces, orthogonal to the four tints — 20 heads. Bots derive theirs from slot INDEX so a firefight has five silhouettes and replays stay bit-identical. Index 0 frozen byte-for-byte to the old cap; saved profiles in the pre-helmet four-field format still load. Three invariants tested, each a real failure the geometry can have. |
| **Third-person bow anchor** | The restage, and it turned out to be three bugs rather than one: the draw hand rode 14 cm ABOVE a now-horizontal string, the string was ONE STATIC BOX that never moved at all, and the nocked arrow hung off the BOW hand so it tracked a hand instead of a bow. The nock is a function now and the halves, the arrow and the hand all derive from it. The viewmodel gained a nocked arrow it never had. Verified in `bow_draw_fp`. |
| **AI suppression** | Produced by GEOMETRY — every round banks suppression on every enemy it passes within 1.15 m of, measured against the segment the tracer actually drew. Bots pay in accuracy (a multiplier on their own sigma, never a floor, so a pinned Hard bot still out-shoots a free Easy one) and stop walking into the fire. The player pays in a viewmodel shake and nothing else: the round still leaves the camera ray where the crosshair is. |
| **AI bounding overwatch** | A second axis, not folded into `SquadRole` — anchor/flank is WHERE, this is WHEN. One half moves and holds fire, the other plants and shoots; they swap on a function of the sim clock and index rank, so no phase is stored and every replay test survives. A lone bot is `Free`, and so is a room fight. Three of its six tests cover what a determinism test cannot see: a deadlock reproduces perfectly. |
| **Per-weapon screen-intrusion profiles** | `spear_raised` and `bow_drawn` built, and the sweep taught to see them: it had been running ONE hard-coded root and `carry_offset` alone, so no weapon's own placement and no pose shift had ever been checked. `bow_drawn` deliberately supersedes the brief's vertical-bow wording — restated, not dropped. |

### Closed 2026-08-05

| Item | How it closed |
|---|---|
| Viewmodel drew over every menu | The vm camera renders after MainCam (order 1, no clear). New `vm_rendered` predicate gates on the same `hud_visible` the HUD uses. Proven by a capture that pauses from live first-person play — the old `menus` script could not reach the case. |
| Pilot held his stowed rifle in a mech | Two hull-mount viewmodels: a spinning gatling cluster and a launch tube. Punch rides the mount's own cycle via `shot_clock`; rifle reload/sight/scope poses gated out in a chassis. |
| RMB zoomed while pre-aiming rockets | `pod_aim_owns_rmb` gate in `input_and_step`. Also dropped an accidental `ADS_SPEED_MULT` slow that Y-targeting never paid. |
| Guard plate blinded its own holder | FP-only translucent material set; third-person and enemy shields stay opaque. |
| Shield bound to E | Now inventory slot 4, an essential beside the three guns. E is dead. |
| Mech HUD garbled / overlapping | Bottom-left is chassis vitals only; the mounts own the bottom-right. Fixed a real argument swap — POWER printed the turret belt and AMMO printed the energy core. |
| Turntable showed a placeholder mannequin | `spawn_soldier_body` extracted from `spawn_fighter_rigs`, so the card and the field share one geometry source. Verified a pure move: all 183 geometry constants byte-identical. |
| Mech read as a plain slab | Detail pass — visor brow/cheeks, spine fins, exhaust stacks, pauldron trim and bolts, waist pistons, gatling clamp rings and feed chute, drum ribs, recessed pod face, two-tone paint, stencils, knee hazard, toe teeth. Detach parts became group nodes so a damage stage sheds a whole cluster. |
| Mech hull-climbing | Full 7-item checklist: zone grabs on stripped plates, position parenting, asymmetric grip drain, involuntary detach, 1.6x climbing strike on both melee paths, 4 tests. |
| Mech inventory / rocket controls | Strip shows TURRET/ROCKETS only while piloting; RMB pre-aims, LMB fires through `try_fire_rocket`. |
| Killfeed WALLBANG modifier | Bullet penetration shipped (`PEN_WINDOW_M`, 0.5x damage), killfeed marks it `#`. |
| Melee parry + stagger | Parry window on both the axe arc-sweep and knife/thrust line paths; attacker takes `PARRY_STAGGER_S`. |

### Closed 2026-08-05 (rev 2)

| Item | How it closed |
|---|---|
| Grip HUD + climb prompt | Ten-segment GRIP bar in the vitals block while hanging; "U - GRAB THE HULL" / "U - LET GO", and a distinct line when the pool is too spent. The attach search moved out of the step into `climb_target` so the prompt asks the same question the verb answers. |
| Mech mount audio | `own_shot` read `fire_cd`, which never moves in a chassis - a piloted mount fired silent for its own pilot. Now on `shot_clock`, one sound per mount, and the "same weapon" guard understands a mount swap. |
| Iron sights on every gun | Two guns had NO rear sight at all (shotgun, M249); five declared the front-post centre as their sight line instead of the aperture centre. Then all of it was superseded: every firearm now carries a **1x red-dot optic with an illuminated red cross**, the AWM's cross living in its scope overlay. |
| M249 "grey wall" | Its sight line sat 2 mm above a flat feed cover, so aiming laid a 30 cm plate across the eye; it also shared the rifle carry offset despite a receiver 2.1x the AK's cross-section. Own carry, raised sights, arched carry handle. |
| Recoil correctness | The permanent camera-pitch channel ignored two rules the sim obeys - projectile weapons produce no punch (a drawn bow was walking the aim up the screen) and a scoped rifle's punch is scaled 25/78. The M249's kick equalled the AK's while cycling 40% faster. |
| Arrow + spear models | Both were one featureless box. Now a bodkin head, tapered shaft, nock and three spinning fletching vanes; a leaf blade, collar, haft and butt spike. |
| Mech fire came from the screen centre | The ray still leaves the eye (that is the hit test), but the STREAK is drawn from the muzzle actually on screen. Rockets ease off the tube onto their true position over 90 ms without touching physics. |
| The HUD was drawn OVER by the weapon | The interface rendered on MainCam (order 0) while the viewmodel camera is order 1 with no clear, so the gun composited on top - it ate the first characters of the mech ammo readout. The UI has its own Camera2d at order 2 now; nothing 3D can draw over it again. |
| **AI squad coordination** | Bots read teammates for the first time. Focus fire (a target a squadmate is on scores as closer, bounded so it cannot out-weigh a man at point blank), anchor/flank roles by index order, and spacing so a squad stops arriving as one clump. All rng-free so replays stay bit-identical - there is a test asserting a full 4v4 runs identically twice. *(Suppression and bounding overwatch, open at the time, both closed 2026-08-06 — see below.)* |
| **Melee depth v2** | Directional attacks: strafe picks the swing's LINE (left / right / overhead) at the wind, latched for the whole strike. A parry only meets a blade on the same line, so a knife fight is a read rather than a timing check. Both viewmodel and third-person silhouettes cock to the chosen line, because the defender has to be able to SEE what to answer. |
| **4-class system** | LINE / SKIRMISHER / WARDEN / MARKSMAN, each hooked to health, movement, spread and swap speed, with a per-class silhouette and a Forge picker. LINE is 1.0 across the board; a test proves the other three each trade. |

## 2. Large, multi-session architectural (do not rush — BACKLOG.md)

Ranked by value, blockers first:

- **26-piece armour**: the 4-CLASS half is built (see below); the
  per-piece armour half is not. Today armour is 5 whole-body presets
  found as loot, and a class is a separate standing pick.
- **20-segment mass-bearing rig** (Steps 0, 2–7 remain): real
  pelvis→lumbar→thorax trunk, clavicles, toe segments, mass-fraction /
  CoM / radius-of-gyration data driving spring stiffness. Touches every
  posing system in main.rs.
- **Castle map**: content work (geometry), not code. The intro's CASTLE
  BAILEY / CASTLE GARDENS entries select layouts of the existing arena
  blockout, not a real castle.
- **Forge editor UI**: the turntable and SAVE/LOAD/RANDOMIZE rows are
  built; the specced per-piece category grid is not (and is really the
  front end of the 26-piece armour item above).
- **Mech weapon-kit D.7**: 20 named swappable parts with part-by-part
  damage states. The silhouette, the core kit, and three damage stages
  are built; per-part swapping is not.
- **Traversal** (#7): climb/vault/mantle — blocked on map metrics.
  Hull climbing proved the attach/parent/stamina mechanic.
- **Full character customization** (§8.1): sliders and per-piece
  cosmetic variants. The HELMET half landed 2026-08-06 (5 shapes × 4
  tints, a real part library rather than a colour swap); body/face
  sliders and weapon cosmetics are still 0 of 0.

## 3. Blocked, with the named unblocker

| System | Blocker |
|---|---|
| Weapon material stack, wear maps, decals, image import | Zero texture pipeline beyond branding UI images — every world surface is flat colour. Unblocker: mesh/material texture loading. |
| Advanced rendering | Depends on the above. |
| **Networking** (rollback/prediction/lag comp) | Zero networking deps; local-only. The deterministic sim + bit-identical replay is the right foundation, but no netcode exists. Scoreboard deliberately has no Ping column for this reason. |
| Swimming / ropes / ladders / fluids | No water volumes, no ropes, no muscle layer. |
| Grenade extra materials (mud/sand/ice…) | No such surface exists in any map. |

## What is NOT missing (commonly assumed otherwise)

- Mech presentation §A–§C: **done** (brace, entry/exit, idle-life, visor
  view, gatling + autocannon, and now first-person mounts).
- Mech silhouette D.1–D.6: **done** (43-plate hull, leg armour, three
  detach stages, detail pass).
- Hull climbing: **done and tested**.
- Assists, K/A/D/DMG scoreboard, death→killer-cam→spectate: **done**.
- Paged intro flow with branding + real-rig turntable: **done**.
- Determinism/replay guarantee incl. bot mechs: **done and tested**.
- Capture coverage for first-person mounts, the guard plate, and the
  pause menu: **done** (`mech_fp`, `shield_fp`).
