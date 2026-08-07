# R&D Backlog

Seeded from `briefs/PROMPT_RND_CYCLE.md` Appendix A on 2026-08-01. Ranking
rule: an item moves up when its blocker clears, not when it becomes
interesting. Blocked items are never researched "in preparation."

## Critical — ready now, high value, direct extension of working systems

| # | System | Why now | Attaches to | Status |
|---|---|---|---|---|
| 1 | Mech entry sequence — approach, ID, cockpit open, climb-in, harness, power-up, servo sync, gyro calibration, weapon diagnostics, HUD boot, camera transition | Turns entering the mech from a state toggle into an event | Mech walker, committal enter/exit (1.6s/1.2s) | **Cycle 1 done (sim-side)** — `mech_enter_stage`/`mech_enter_stage_for` in sim.rs, 8 named stages, tested. Presentation wiring split out as #16. See `engine/crates/jk_tdm/research/mech-entry/CYCLE_1_REPORT.md` |
| 2 | Infantry vs. giant mech — climbing the hull, joint strikes, hydraulic failure, cable cutting, sensor destruction, weak-point exposure | Makes the mech an encounter, not a health bar | Mech + spear + segment-mapped armour spec | **Cycle 3 done (design only)** — confirmed the damage-differentiation core (angle armor/visor/plate-strip) already applies uniformly to melee; designed a stamina-gated hull-climb mechanic reusing the existing plate-detach zones as attach points, grounded in real grip-fatigue research. Build queued next cycle - see `engine/crates/jk_tdm/research/mech-climb/DESIGN.md` for the 6-item build-readiness checklist |
| 3 | Grenade surface interaction — per-material restitution/friction, rolling, spin, angular momentum | Cheapest high-value item; `grenade_tick` already has per-surface bounce + determinism test to extend | `grenade_tick`, existing R11 tests | **Cycle 2 done** — `surface_friction` added alongside `surface_restitution` (Stone 0.30, Crate 0.45), behavioral test proves stone skids further under identical impact. Rotation/spin and new materials split out as #17/#18. See `engine/crates/jk_tdm/research/grenade/CYCLE_2_REPORT.md` |

## High — ready, medium cost

| # | System | Why | Attaches to | Status |
|---|---|---|---|---|
| 4 | Melee depth: parry, deflection, directional attack, stagger, armour penetration, weak points | Melee exists but shallow | axe/spear/shield | Not started |
| 5 | AI squad coordination: flanking, suppression, bounding overwatch, retreat | Bots exist; `jk_wall` already models morale/fear/rout | bot AI, jk_wall morale | **Flanking, suppression and bounding overwatch DONE** (2026-08-05/06). RETREAT is the remainder and is the one that genuinely wants `jk_wall`'s morale model rather than another derived-from-index rule — a squad that breaks needs memory of what broke it, which nothing in the TDM sim currently keeps. |
| 6 | Mech operation feel: weight, mechanical inertia, cockpit vibration, heat, internal damage, emergency shutdown/eject | Extends #1; heat-and-cool creates rhythm a fuel bar can't | mech sim | Partially started — power stride (this session, Section H) added a heat-budget mechanic to mech mobility, not yet extended to internal damage/eject |
| 7 | Traversal: climb, vault, mantle, ledge bands | Already scoped in the master prompt; must match map metrics | none yet — needs map metrics first | Blocked on map metrics quota |

## Medium

| # | System | Note |
|---|---|---|
| 8 | Motion architecture decision | Already specced in `PROMPT_motion_system_research.md` — session 1 done (5/14 core sources). **Continue that prompt, don't duplicate.** |
| 9 | Character creation layers (L0-L4) | Blocked in practice: no class system, only 5 whole-body armour presets |
| 10 | Destruction and environmental interaction | rapier supports some; needs a design reason before a technique |
| 11 | Injury, fatigue, equipment weight, dynamic centre of gravity | Armour-weight formula exists but is unwired |

## Blocked — named, with the specific unblocker

| # | System | Blocker |
|---|---|---|
| 12 | Weapon material stack, wear maps, decals, player image import, in-game console | Zero texture pipeline. All 21 `asset_server.load` calls are `.wav`. Unblocker: any image loading at all. |
| 13 | Advanced rendering (Nanite/Lumen-equivalents, GPU-driven pipelines, path tracing, upscalers) | Wrong engine and nothing to render into. Unblocker: #12, then a Bevy-native rendering decision. |
| 14 | Networking — rollback, prediction, reconciliation, lag compensation | Zero networking dependencies; build is local-only. Reitich's prediction mechanisms already recorded in `research/grenade/SOURCES.md` for when this unblocks. |
| 15 | Swimming, diving, rope traversal, ladders, muscle simulation, soft bodies, fluids | No water volumes, no ropes, no muscle layer. |

## Discovered this session (not in the original Appendix A)

| # | System | Tier | Why | Attaches to |
|---|---|---|---|---|
| 16 | Mech entry stage presentation — wire `mech_enter_stage_for` to visor flicker, per-stage servo audio, and the camera transition on HudBoot; capture it | High | Cycle 1's sim-side staging (#1) has nothing to render yet; this is the concrete remainder of #1, split out rather than left implicit | `mech_enter_stage_for` (sim.rs, Cycle 1), viewmodel/third-person rig |
| 17 | Grenade rotational dynamics — real spin/angular momentum affecting bounce direction, not just cosmetic tumble | Low | Deliberately deferred in Cycle 2: real rotational dynamics in a fixed-timestep, replay-critical integrator is a much larger determinism-risk change than the tangential-friction win already banked, for a bounce-feel difference players are unlikely to distinguish | `grenade_tick`, would need a new angular-velocity field on `Grenade` |
| 18 | Additional grenade surface materials — mud, sand, snow, ice, water | Blocked | None of these exist as a `CoverKind` or appear in any map today; researching coefficients now would go stale before any map could use them | Unblocker: any map content adding one of these surface types as real cover geometry |
| 19 | §5.4 spear running-throw bonus | Done | Built 2026-08-01 (`baf50ca`) | — |
| 20 | §4.3 minimap enemy spotting | Done | Built 2026-08-01 (`5fbb3c7`) | — |
| 21 | §4.1 bow full-draw hold sway | Done | Built 2026-08-01 (`054a283`) | — |
| 22 | Derive the screen-intrusion budgets from the weapon geometry | **Done 2026-08-07** | `weapon_parts(kind)` lifted out of `spawn_weapon_model`; `weapon_bounded_extent` measures. The audit it replaced was wrong by 2.3x on the tallest weapon | — |
| 23 | Armour damage STATES (Brief IX §C) | High | The 24 plates equip and expose, but a worn plate has no condition. The brief's Fresh/Scuffed/Cracked/Severed table needs per-piece HP, which needs the hit path to resolve which PIECE was struck rather than which zone - a real extension of `apply_hit_dmg`, not a polish pass | `ArmorLoadout`, `apply_hit_dmg`'s zone resolution |
| 24 | Per-piece armour GEOMETRY | Medium | The plates are a stat model with a Forge grid; the soldier's mesh is unchanged when you strip a gauntlet. Cheap in principle now that §B.1's rig has clavicles, toes and a three-part trunk to hang plates from - but it is 24 models, so it is content work wearing a code shape | `spawn_soldier_body`, the 20-segment rig |
| 25 | Ragdoll + hit-reaction impulse from the mass model | Medium | §B.5 names both as the payoff for the inertia column, and the column now exists and is tested. Nothing reads it yet beyond `derived_spring_k` | `segment_data`, `segment_inertia` |

## Thor's audit (2026-08-01): 143 findings across all 9 briefs, full detail in `THOR_LOG.md`

Six discover agents + independent re-verification found 97 double-confirmed
gaps and 46 provisional ones (session-limit hit mid-verify; carrying the
discover agent's own evidence rather than a fabricated disposition — see
THOR_LOG.md for the exact mechanism). Two are done as of #19/#20 above.
The rest split into two very different sizes:

### Small, well-scoped, one-session-buildable (the majority of the 97)
Individually real and individually small: missing HUD elements (crosshair
settings, killfeed modifiers/border, scoreboard columns, death→spectate
flow, resource/toast display), missing numeric tunings (bow sway ramp,
various screen-intrusion profiles, spear FOV/speed changes), dead fields
that were assigned but never read (`ElasticMove.return_efficiency`,
counter-movement bonus not reaching jump), test-coverage gaps for
mechanics that DO exist. These get implemented the same way #19/#20 did —
picked off one at a time, built, tested, committed, pushed — as work
continues.

### Large, genuinely multi-session architectural rework (do not rush)
- **20-segment mass-bearing body rig** (BRIEF_VIII_B_addendum B.1-B.6,
  PROMPT_mech_rebuild Task2): the existing rig has ~14 transforms (3-seg
  limbs, 1 torso). The brief wants a real pelvis→lumbar→thorax trunk,
  clavicles, toe/forefoot segments, WITH mass fraction / CoM / radius-of-
  gyration data driving spring stiffness. This is a rig rebuild, not a
  bug fix — touches every posing system in main.rs.
- **Mech visual + weapon-kit rebuild** (BRIEF_VIII_B_addendum D.1-D.7,
  PROMPT_mech_rebuild Task5): "walking weapons platform" silhouette
  (currently a scaled humanoid), gatling+autocannon as the CORE kit
  (missile pod becomes optional, not vice versa), 20 named swappable
  mesh parts (currently 33 generic plates), named part-by-part damage
  states.
- **Forge editor UI** (BRIEF_VII §7.2, BRIEF_VIII §8.2): a real visual
  editor (category grid, turntable, randomize) — today Forge is 3 hotkey
  save/load slots to a flat text file, no UI at all.
- **26-piece armour + 4-class system + castle map** (BRIEF_IX §C, §A):
  confirmed absent; each is its own subsystem with no existing code to
  extend, and the castle map is content work (geometry), not code.
- **Full character customization** (BRIEF_VIII §8.1): body/face sliders,
  weapon cosmetic variants, mech cosmetics — currently 2 flat color
  fields (hat, tunic) exist total.
- **config/*.ron migration**: many findings cite "not in config/*.ron."
  This repo's established, deliberate convention is hand-rolled
  `key=value` text files (camera_tuning.txt, settings.txt) specifically
  to avoid a serde/RON dependency for a handful of values — the SPIRIT
  of "tunables aren't hardcoded" is already met. Treating every one of
  these as a literal "add a .ron file" gap would mean introducing a new
  dependency and file format across the codebase for its own sake; more
  likely each should be evaluated case-by-case (does this specific
  value actually need runtime-editability, or does the existing
  external-text-file convention already satisfy the underlying need).

*(further items appended as cycles complete, per Section 4's "what
should exist that we have not discussed" requirement)*
