# DESIGN MAP — the battlefield-simulator architecture vs. what's built

> ## ⚠️ THIS FILE IS ABOUT `jk_wall`, NOT `jk_tdm`. STOP AND READ THIS FIRST.
>
> **Corrected 2026-08-10.** This document maps the **battlefield /
> shieldwall simulator** (`jk_wall`, `jk_core`) — the commander-in-the-line
> game. It maps **`jk_tdm`, the game currently under active development,
> NOWHERE.** Verified: `jk_wall` appears in this file; `jk_tdm` does not
> appear once.
>
> That matters because `briefs/PROMPT_MASTER_research_build.md` and
> `briefs/README.md` both tell every new session to read this file as
> *"what is actually built versus what is specified."* Every session that
> obeyed that instruction has been reading the design map of a different
> game. This is the highest-traffic wrong document in the repository.
>
> **For `jk_tdm`'s built-vs-specified state, read instead:**
> - `engine/crates/jk_tdm/research/TREVOR_TASKS.md` — what is open, ranked
> - `engine/crates/jk_tdm/research/TREVOR_LEDGER.md` — all 266 asks, with
>   status and evidence
> - `engine/crates/jk_tdm/research/WHATS_MISSING.md` — the live plan
>
> This file is kept, not deleted: it is still the correct map for
> `jk_wall`, and that work is real. It just is not this game.

The owner's architecture spec (2026-07-17): an **original** third-person
large-scale battlefield simulator inspired by *Shieldwall*'s gameplay —
mechanics reverse-engineered from public gameplay/reviews, zero copied code
or assets. This file maps every section of that spec onto John Kingdom
Game's systems.

Legend: ✅ built · 🟡 partial · 🔜 planned (milestone noted) · 📐 designed
(research/decision exists, no code yet)

## Vision & loop

| Spec | Status | Where |
|---|---|---|
| Player is one commander among many soldiers | ✅ | `take_player` — you hold one body in the wall |
| Influence via positioning/formations/morale/timing | ✅ | orders 1–5 + body positioning + (M4) morale |
| No scripted battles; every battle different | ✅ | everything emergent from seeded sim; no scripts |
| Loop: orders → squads react → enemy reacts → physics → morale → repeat | ✅ | `WallSim::step()` is literally this pipeline |

## Scale

| Spec | Status | Where |
|---|---|---|
| 100 / 250 / 500 / 1000+ | 🟡 | 80 validated at 23–90× realtime; 250v250 benchmarked in `bench` (M4); ceiling per research/01: 400–700 full-physics bodies, then LOD |
| ECS / instancing / LOD / animation sharing | 📐 | data-oriented structs now; LOD tiering + Jolt swap at client stage (ADR-002); renderer instancing when the GL client grows |

## Camera

| Spec | Status | Where |
|---|---|---|
| Third person, 4–7 m, zoom, shoulder switch | 🟡 | spring-damped chase cam 3.8 m (`ClientCam`, `tp::Camera`); zoom/shoulder-switch 🔜 M5 |
| Camera collision / never clip | 🟡 | occlusion cull of bodies between lens and player (Game AI Pro pattern); wall collision 🔜 with terrain |
| Dynamic FOV (sprint/combat) | 🔜 M5 | client |

## Character controller

| Spec | Status | Where |
|---|---|---|
| Walk/run/sprint/strafe/backpedal | ✅ | WASD velocity-servo, camera-relative |
| Acceleration, weight matters, armor inertia | ✅ | force-capped servo on a real rigid body; armor mass enters the same `mass_kg` the physics integrates — a mailed man IS slower to accelerate |
| Heavy 0.4 s to sprint vs light 0.15 s | ✅ (emergent) | falls out of F = ma with armor mass; not authored |

## Physics & collision

| Spec | Status | Where |
|---|---|---|
| Mass, velocity, impulse, friction per soldier | ✅ | Rapier bodies, exact body+gear mass |
| Capsule + shield colliders | ✅ | capsule body + enemy-only shield cuboid |
| Weapon colliders | 🟡 | strikes are reach+arc queries resolving energy vs armor (brief §2.5); swept weapon colliders 🔜 when animation exists |
| Formation members push each other; pressure builds like real crowds | ✅ | the core thesis: soft-contact chains, brace degradation, 4.5–8 kN emergent front saturation, crush injuries |
| Foot IK / no clipping | 🔜 | client polish, M5+ |

## Crowd movement

| Spec | Status | Where |
|---|---|---|
| Final velocity = formation + avoidance + goal + combat + terrain | 🟡 | formation slot PD + follow rule + engagement plane + (M4) rout vector; explicit avoidance & terrain cost 🔜 with terrain (M5) |
| No teleporting | ✅ | forces only, ever |

## Combat

| Spec | Status | Where |
|---|---|---|
| Attack state/recovery/stamina/reach/reaction | ✅ | strike cooldowns, per-man tempo, stamina pool cost, reach+frontal arc |
| Wind-up → swing → collision → damage → recovery cycle | 🟡 | committed-strike + cooldown today; visible wind-up/swing states 🔜 with animation (M5) |
| Weapon: length/mass/damage/momentum/speed/recovery/AP | ✅ | `Weapon { eff_mass, strike_v, reach }` + energy model; damage IS mass×v²/2 vs armor energy threshold |
| Spear long/slow/formation, sword fast/versatile, axe heavy/slow | ✅ M4 | weapon triangle with research-sourced energy bands |
| Shield: blocking angle, coverage, not button press | 🟡 | state-based blocking (compression, exhaustion, brace, facing via frontal-arc targeting); geometric blocking cone 🔜; durability 🔜 |
| Damage by location, not HP-only | 🟡 | energy-through-armor wounds; hit locations 🔜 with weapon colliders |
| Armor absorbs slash/pierce/blunt differently | 🟡 | pierce thresholds (cloth/gambeson/mail) sourced from Williams; slash/blunt split 🔜 M5 with the forge (armor properties from actual metallurgy) |

## Command hierarchy & formation AI

| Spec | Status | Where |
|---|---|---|
| Squad→company→battalion→army | 🔜 | single wall (squad) today; hierarchy at campaign layer |
| Slot assignment, nearest-free-slot fallback | ✅ | file slots + dynamic rank (front man falls → next steps up) |
| Formation types (wall/square/wedge/…) | 🟡 | wall + orders (brace=testudo-tight); more shapes 🔜 M5 |
| AI thinks 10–20×/s not every frame | 🟡 | behavior currently at tick rate (cheap); decision throttling when AI deepens |

## Morale — "the most important system"

| Spec | Status | Where |
|---|---|---|
| Fear/confidence/fatigue/leadership/outnumbering | ✅ M4 | `jk_wall::morale` — fear field per man |
| Commander dies → morale break | ✅ M4 | the player IS the leadership aura; his fall broadcasts fear |
| Friend dies nearby / flanked / losing ground | ✅ M4 | witnessed-down fear, outnumber pressure, compression fear |
| Low morale: hesitation, backing away, panic, rout | ✅ M4 | routing men flee, can't block, leave the front — gaps cascade |
| Rally | ✅ M4 | fear decays away from the enemy; men return to the line |

## Stamina

| Spec | Status | Where |
|---|---|---|
| Running/swinging/blocking consume; heavy armor more | ✅ | two-pool aerobic/anaerobic (validated 60–90 s), strike costs, compression cost, armor in mass |
| Low stamina → slower attacks, weak blocks | ✅ | strike effort ∝ stamina; block malus when exhausted |

## Terrain, siege, projectiles, sound, networking, save, progression

All 🔜/📐 — in order per the roadmap below: terrain costs + arrows
(M5, physics projectiles per brief §2.5 energy model), formations/shapes,
then siege, campaign/economy (the shieldwall_reforged economy research is
already calibrated for this), audio, multiplayer (determinism groundwork
already in: fixed tick, seeded RNG, bit-identical replays).

## Roadmap alignment (spec's 15 steps)

1. Third-person controller — ✅ M2
2. One AI soldier — ✅ M1
3. Navigation — 🟡 (formation/engagement steering; pathfinding with terrain)
4. Squad of 10 — ✅ M1 (40v40)
5. Formation system — ✅ M1–M2
6. Combat — ✅ M3
7. Morale — ✅ M4
8. Stamina/fatigue — ✅ M1/M3
9. Terrain — 🔜 M5
10. Hundreds via ECS/instancing — 🟡 M4 benchmark; LOD+Jolt planned
11. Siege — 🔜
12. Multiplayer architecture — 📐 (determinism already designed in)
13. 1000+ optimization — 📐 (research/01 ceiling + LOD strategy)
14. Campaign/strategy/logistics — 📐 (economy sim calibrated in research/03)
15. Polish/audio/UI/Steam — 🔜

## Originality statement

No code, assets, art, audio, or data from *Shieldwall* (Nezon Production)
or any other game is used. Mechanics were reverse-engineered from public
gameplay footage and reviews (research/05 cites every source); all
implementations are original Rust, and all tuning constants trace to public
scientific/historical literature (07_CONSTANTS.md).
