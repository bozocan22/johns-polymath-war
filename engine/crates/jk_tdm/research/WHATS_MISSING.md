# What is missing / not built yet — 2026-08-05 (rev 2)

Compiled from `BACKLOG.md`, `THOR_LOG.md`'s ranked findings, and session
knowledge. Ordering inside each tier is Thor's ranking rule: an item
moves up when its blocker clears, not when it becomes interesting.

## 0. Needs the USER (nothing code-side can proceed)

Nothing. The three branding PNGs landed 2026-08-04 (key art, wordmark,
emblem) and are wired through the splash, menus, and seal footers.

## 1. Small, one-session buildable (ranked, next up)

1. **Numeric tunings** — the remaining screen-intrusion profiles called
   out in THOR_LOG. Picked off one at a time as work continues.
4. **HUD award toasts for a resource economy** — the toast system
   itself shipped (kills/assists/parries/streaks). §4.3's *resource*
   awards still have no economy in TDM/KOTH to award from, so this
   waits on a mode that has one rather than being faked.

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
| **4-class system** | LINE / SKIRMISHER / WARDEN / MARKSMAN, each hooked to health, movement, spread and swap speed, with a per-class silhouette and a Forge picker. LINE is 1.0 across the board; a test proves the other three each trade. |

## 2. Large, multi-session architectural (do not rush — BACKLOG.md)

Ranked by value, blockers first:

- **26-piece armour**: the 4-CLASS half is built (see below); the
  per-piece armour half is not. Today armour is 5 whole-body presets
  found as loot, and a class is a separate standing pick.
- **Melee depth v2**: directional attacks and deflection. Parry and
  stagger are built; direction and deflection are not.
- **AI squad coordination** (#5): flanking, suppression, bounding
  overwatch; `jk_wall` morale exists to hook into.
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
- **Full character customization** (§8.1): sliders, cosmetic variants —
  currently 2 flat colour fields.

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
