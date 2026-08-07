# What is missing / not built yet — 2026-08-07 (rev 5)

Compiled from `BACKLOG.md`, `THOR_LOG.md`'s ranked findings, and session
knowledge. Ordering inside each tier is Thor's ranking rule: an item
moves up when its blocker clears, not when it becomes interesting.

## 0. Needs the USER (nothing code-side can proceed)

Nothing. The three branding PNGs landed 2026-08-04 (key art, wordmark,
emblem) and are wired through the splash, menus, and seal footers.

## 1. Small, one-session buildable (ranked, next up)

*(Audited 2026-08-07 against the code. Three items on this list had
already been built and were still sitting here as open — bot support
logic, per-piece armour geometry, and the second beam pool. A stale
"missing" list is worse than none: it sends the next session to rebuild
finished work. What follows is what is actually left.)*

1. **Armour damage STATES** (Brief IX §C). The 24 plates are
   equippable, visible, and a missing one exposes its segment — but a
   WORN one has no condition. The brief's four-stage table (Fresh →
   Scuffed → Cracked → Severed, with a plate detaching on the next hit)
   needs per-piece HP, which needs the hit path to resolve which *piece*
   it struck rather than which zone. A real extension of
   `apply_hit_dmg`, not a polish pass.
2. **HUD award toasts for a resource economy** — the toast system
   itself shipped (kills/assists/parries/streaks). §4.3's *resource*
   awards still have no economy in TDM/KOTH to award from, so this
   waits on a mode that has one rather than being faked.
3. **Audio for the new systems.** Plasma, the repair beam, the barrier
   deploying and the precision charge all use borrowed sounds from the
   soldier arsenal. They are placeholders and are marked as such at the
   call sites.

### Closed 2026-08-07 (the medic pass — model, HUD, and three bugs)

| Item | How it closed |
|---|---|
| **The repair beam could not be selected at all** | `cmd.slot` was hardcoded `0 => Gatling, 1 => Rockets` — the HEAVY's mounts, applied to every chassis. In a medic, whose strip reads PLASMA BOW [1] / REPAIR BEAM [2], pressing 2 set ROCKETS and the validity guard reverted it the same tick, so the support half of the chassis was unreachable while the HUD advertised it by name and number. Split brain: the strip renders from `for_set` and this handler carried its own copy of the list. One source now, and the index comes from the same slice the HUD draws, so slot N *is* the mount labelled N. Found by a capture; every existing mount test passed throughout because they all assign `mech_weapon` directly and none press a key. |
| **The capture rig could not photograph a side** | A beat named `02-medic-side-on` had produced rear views for its whole life: `look` turns the PLAYER and the third-person boom is rigidly behind the player, so the subject rotates with the camera and no yaw yields a profile. `CapBeat.orbit` swings the boom around a stationary subject (and re-aims at the anchor, which the first attempt did not — it photographed the scenery beside the machine). Capture-only and inert in play. Also pinned: beat times must not run backwards, and every script's last beat must set `end`. |
| **The medic model read as a small heavy** | Four changes, each paid for by an angle the rig could not previously take. Head: a pale ball that read as a human skull in the soldiers' own white → a raked sensor pod, longer front-to-back than wide (the one proportion that separates a bird's head from a person's at any distance). Chest: one 46 cm plate sealing the front, so the exposed core showed only in profile → two halves with a chamfered slot. Shoulders: box caps → swept pauldrons with trailing fins. And a COUNTERWEIGHT BOOM behind the hips — the heavy is a vertical column with nothing behind it, so a boom separates the two machines as pure shape with all detail resolved away. |
| **Medic HUD had no support readout** | The target bracket frames the ally the beam would take, in green, from `Sim::repair_candidate` — the function the beam itself calls, made `pub` for that reason rather than letting the HUD run a lookalike search. Precision charge also renders at the crosshair now: both prior readouts sit in corners, and a pilot winding up a 95-damage shot is looking at centre screen. Colour snaps at full charge instead of fading, because the release point is a threshold. |
| **The beam could not appear in a capture** | It only takes an allied MECH under 85% hull, which a fresh match never contains. The boarding hook parks one hurt machine in reach — by hand, because a script that waits on combat to reach a state captures a different frame every run. |

### Closed 2026-08-07 (the art + second-chassis pass)

| Item | How it closed |
|---|---|
| **Heavy mech was a slab** | Hull worn at 85% as a uniform root scale (uniform because the hull is full of rotated cylinders and a non-uniform parent scale shears every one — and every hardpoint moves inboard for free). ~120 new parts buy the size back as MASS: armour layering with shadow gaps, pistons as rod-in-barrel, louvre runs, a five-layer reactor core with conduits leaving it, joint collars. Head went from a box with a slit to segmented armour + optic pods + sensor clusters + comms blades, with the ×2 visor slit untouched. Arms went from bare hardpoints to a pauldron→elbow→cradle chain. Legs got a knee cap, calf mass and an attitude thruster. |
| **Both hull mounts were placeholders** | The turret gained spinner-mounted clamps and muzzle collars, a STATIC drive housing (the drive does not spin, the thing it drives does — most of what makes a gatling read right), a heat-sink stack with a glowing seam, a belt box with gold feed links, and a partial cowl open below. The launcher gained a ribbed casing, mouth jaws, backblast vents, a feed arm mid-travel and a boxed seeker head. |
| **Mech barrier had no visual** | An arm-mounted folding emitter (three petals, 0.18 s deploy) projecting an 8.5%-alpha fill plus a bright hex LATTICE of real geometry. That split is the answer to a contradiction in the brief — "transparent to the pilot" and "visible to enemies" cannot both come from one translucent sheet. A test pins the gap between the two alphas. |
| **Maps were small** | Every map +25%, applied centrally: POSITIONS scale, extents and heights do not. Plus an infill pass (outer ring at three height bands, mid-field stepping stones, angled flank lanes) because a bigger map is a worse map empty. |
| **AGILE SUPPORT MECH** | A second `ArmorSet`, not a flag — a third of the hull, faster than a man on foot, plasma that never runs out but overheats (hard lockout), a repair beam that mends allied CHASSIS only, and a ×2.4 vulnerability to spears and arrows that gives those two weapons something to beat. Digitigrade ~60-part frame. Its own pads on the flanks. |
| **Repair beam was invisible** | The sim publishes `repair_target`; the client draws a segmented shaft plus travelling packets. Two layers because a glowing line says a connection exists and only moving packets say something is being CARRIED — and a beam aimed at a teammate that looks like a weapon makes people dodge their own medic. |

### Closed 2026-08-07

| Item | How it closed |
|---|---|
| **Intrusion budgets were audited, not measured** | `weapon_parts(kind)` lifted out of `spawn_weapon_model` — the geometry is data now, so `weapon_bounded_extent` measures the budgets instead of transcribing them. The minigun's barrel cluster became a `spin` flag on the part, which was the one thing welding the table to `Commands`. Measurement immediately disagreed with the audit: the M249's carry handle is 0.192, not the 0.085 the comment claimed. It also caught the BOW clipping the crosshair circle when the grenade coil stacked on the draw — fixed by making the poses exclusive and dropping the bow's carry to -0.22. |
| **Suppression had no HUD read** | The sim records the bearing of the last close round; the screen edge facing it lights pale gold. It reuses the DAMAGE flash's own strips rather than a new widget — suppression is the same information one step earlier, and a second directional element would teach the same idea twice. Red at 0.55 alpha, gold at 0.25, so a hit wins the strip on alpha alone with no priority rule. |
| **Squad retreat** | The last quarter of squad AI, and the only part needing MEMORY — every other squad behaviour is re-derived each tick. `fear` flows in from witnessed falls, suppression, being outnumbered and your own blood; bleeds out over time and faster once contact breaks; crosses a threshold with hysteresis. Modelled on `jk_wall`'s morale pass, minus its rolled `rout_tolerance` — that comes from CLASS instead, making nerve a fifth thing the four classes trade. A broken man holds fire, keeps facing the enemy, and reloads. The player is never routed. |
| **26-piece armour** (built as 24 — see below) | 24 plates, each mapped to a hit zone, each with the brief's weight. A bare segment takes ×1.25; total weight past the class ceiling costs 0.15 m/s per kg, which finally WIRES `armor_weight_movement_penalty` after it sat pure and unreachable. Its own Forge page, four rows by body region, with a live weight-against-ceiling readout. **The brief's own table sums to 24, not the 26 its title claims** — built to the table, recorded rather than reconciled by inventing two plates. |
| **20-segment mass-bearing rig** | The data half (§B.3 mass, §B.4 length, §B.5 inertia) plus the three missing segment groups: a real **lumbar** between pelvis and thorax (the twist is shared 38/62 now, not landed on one hinge), **clavicle bones** the arms actually hang from (the girdle spring existed but only nudged an IK target), and **toes** — so the sprint pushes off something instead of gliding. All five of §B.6's tests are live, including the mass-closure one that catches the brief's own trap: the clavicles are carved FROM the thorax, not added beside it. Spring stiffness is `I·ω²` off the mass model rather than hand-guessed. |

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

- ~~**26-piece armour**~~ — **DONE 2026-08-07** as 24 plates (the
  brief's table, not its title). What remains of §C is the damage-STATE
  table and per-piece geometry; both are listed in tier 1 above.
- ~~**20-segment mass-bearing rig**~~ — **DONE 2026-08-07**. All 20
  segments exist as real transforms with published mass/length/inertia
  behind them. The remaining rig work is *consuming* the data further:
  ragdoll and hit-reaction impulse are the two §B.5 names as payoffs
  that nothing reads yet.
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
