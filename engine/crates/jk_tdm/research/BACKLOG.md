# R&D Backlog

Seeded from `briefs/PROMPT_RND_CYCLE.md` Appendix A on 2026-08-01. Ranking
rule: an item moves up when its blocker clears, not when it becomes
interesting. Blocked items are never researched "in preparation."

## Critical — ready now, high value, direct extension of working systems

| # | System | Why now | Attaches to | Status |
|---|---|---|---|---|
| 1 | Mech entry sequence — approach, ID, cockpit open, climb-in, harness, power-up, servo sync, gyro calibration, weapon diagnostics, HUD boot, camera transition | Turns entering the mech from a state toggle into an event | Mech walker, committal enter/exit (1.6s/1.2s) | **Cycle 1 done (sim-side)** — `mech_enter_stage`/`mech_enter_stage_for` in sim.rs, 8 named stages, tested. Presentation wiring split out as #16. See `engine/crates/jk_tdm/research/mech-entry/CYCLE_1_REPORT.md` |
| 2 | Infantry vs. giant mech — climbing the hull, joint strikes, hydraulic failure, cable cutting, sensor destruction, weak-point exposure | Makes the mech an encounter, not a health bar | Mech + spear + segment-mapped armour spec | Not started |
| 3 | Grenade surface interaction — per-material restitution/friction, rolling, spin, angular momentum | Cheapest high-value item; `grenade_tick` already has per-surface bounce + determinism test to extend | `grenade_tick`, existing R11 tests | **Cycle 2 done** — `surface_friction` added alongside `surface_restitution` (Stone 0.30, Crate 0.45), behavioral test proves stone skids further under identical impact. Rotation/spin and new materials split out as #17/#18. See `engine/crates/jk_tdm/research/grenade/CYCLE_2_REPORT.md` |

## High — ready, medium cost

| # | System | Why | Attaches to | Status |
|---|---|---|---|---|
| 4 | Melee depth: parry, deflection, directional attack, stagger, armour penetration, weak points | Melee exists but shallow | axe/spear/shield | Not started |
| 5 | AI squad coordination: flanking, suppression, bounding overwatch, retreat | Bots exist; `jk_wall` already models morale/fear/rout | bot AI, jk_wall morale | Not started |
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

*(further items appended as cycles complete, per Section 4's "what
should exist that we have not discussed" requirement)*
