# Brief VII v2 — Handback

82/82 tests green (up from 49 at session start). Build stable at launch
(verified via stderr-capture + 12s liveness check). Committed as `a1dadc6`.

## 1. Section 0 audit — "after" state

| Feature | Coded | Tested | Visible in launched build | Root cause (before) | Fix |
|---|---|---|---|---|---|
| Third-person-by-default | n/a (design) | n/a | **Confirmed via screenshot** — this is correct, expected behavior, not a bug | player likely never pressed V | Documented; V still toggles |
| First-person viewmodel (Brief VI) | yes | yes | **Yes — screenshot evidence** (big M4 lower-right, correct FOV) | none — was already working | n/a |
| 353 invisible UI glyphs (em-dash, middle-dot, health-cross, warning, killfeed icons, weapon-strip arrow, missile-tube icons) | — | — | **Was invisible tofu boxes** in every capture | bundled font has no glyphs outside plain ASCII | Replaced all with ASCII equivalents, file-wide |
| Idle "statue" complaint | partial | now full | **Fixed** — living-motion layer added | old layer existed but was too subtle/undifferentiated | New breathing-heat, weight-shift, head-glance, grip-fidget, reactions, posture layers |
| Spear thrust (Brief V) | yes | yes | yes | none | untouched |
| Spear throw stick/bounce | **no** | **no** | **no** | never built | Built: angle-based embed/bounce, zone damage, carry cap 2 |
| Bow draw/piercing | **no** (flat instant-fire) | **no** | **no** | never built | Built: draw/letdown/power-scaling/pierce-cascade |
| Third-person camera states | partial | partial | yes but untuned | old constants didn't match spec | Retuned hip/sprint/aim boom+offsets; added torso-limit clamp |
| Mech entry/exit | **instant stat-swap** | no | **no animation state at all** | never built | Built: 1.6s/1.2s committal timer, blocks firing while sealing |
| Mech damage-state plates | **no** | **no** | **no** | never built | Built: 70/40/15% HP thresholds, ×1.25 exposed-frame bonus |
| The Forge | **no** | **no** | **no** | never built | Built: save/load over existing hat/tunic/melee/grenade cosmetics |

## 2. Test commands and current output

```
cargo test --release -p jk_tdm   →  ok. 82 passed; 0 failed; 2 ignored
```
Key new suites: `living_motion_tests` (9), `hand_craft_tests` (6),
`spear_throw_v2` tests (7, inside `sim::tests`), bow draw/pierce tests (5),
`camera_v2_tests` (2), `mech_v2_tests` (4), `forge_tests` (4).

## 3. Captures
`handback/brief-vii/baseline/` (5 shots: 3rd-person default, 1st-person
rest/fire/mid-spray), `handback/brief-vii/idle_life/` (3 shots showing
breathing/weight-shift motion between frames), `handback/brief-vii/bow_draw/`
(5 shots confirming bow equips and holds correctly).

## 4. Feel questions
- **Idle reads as alive or twitchy?** Not independently re-verified after
  the final tune — the statue test guarantees motion exists and stays
  inside Brief IV's translation budget, but "alive vs twitchy" is a human
  judgment call the next playtest should make.
- **Hands read as hands?** Trigger-finger timing retuned to spec
  (0.06s out / 0.10s back) and unit-tested; not re-screenshotted at
  full digit resolution — this engine's hands are low-poly primitives,
  not modeled phalanges, so "do they read as hands" has a hard visual
  ceiling regardless of timing accuracy.
- **Spear/bow mechanically real?** Yes — both are sim-side, deterministic,
  and covered by golden-value tests (angle threshold, draw-power curve,
  pierce cascade).
- **Mech reads as heavier now?** The entry/exit committal window and
  damage-state stripping are real; the material/palette pass explicitly
  stayed at Brief VI's khaki (this document's later "MISSION" brief
  supersedes that with a different scale/palette — see the follow-up
  work log).

## 5. Every tunable introduced
| Constant | Value | File |
|---|---|---|
| Living motion: breath rate 0.2-0.5Hz, weight-shift 6-12s, grip-fidget 8-15s, head-glance 4s/1.1s window | see `breath_hz`/`id_period`/`grip_fidget`/`head_glance` | main.rs |
| `ELBOW_FLEX_MIN/MAX_DEG` | -5 / 145 | main.rs |
| `DIP_PIP_COUPLING` | 0.7 | main.rs |
| `SPRING_K_*` (hand/elbow/finger/shoulder/camera) | 120/60/220/45/90 | main.rs |
| `TRIGGER_OUT_S` / `TRIGGER_BACK_S` | 0.06 / 0.10 | main.rs |
| `SPEAR_WINDUP_S` / spear v0 / damage / `SPEAR_HEAD_MULT` | 0.40 / 22 / 85 / 2.0 | sim.rs |
| `SPEAR_STICK_ANGLE_DEG` / `AMMO_CAP_SPEAR` | 30 / 2 | sim.rs |
| `BOW_DRAW_MIN/FULL/FORCE_S`, `BOW_V0_FULL`, `BOW_PIERCE_DMG` | 0.15/0.7/10 · 55 · [90,67.5,50.625] | sim.rs |
| `TP_BOOM`/`TP_UP`/`TP_RIGHT` (hip), `_SPRINT`, `_AIM` variants | 2.2/0.12/0.45, 2.5, 1.35/0.55 | main.rs |
| `TORSO_AIM_LIMIT_DEG` | 60 | main.rs |
| `MECH_ENTER_S`/`MECH_EXIT_S` | 1.6 / 1.2 | sim.rs |
| `MECH_PLATE_70/40/15_PCT`, `MECH_EXPOSED_DMG_MULT` | 0.70/0.40/0.15, 1.25 | sim.rs |
| Forge slots | 3 (Ctrl+1/2/3 save, 1/2/3 load) | main.rs |

## 6. Named deferrals (honest, per C7)
- **RON/hot-reload config files** — every tunable above is a Rust `const`,
  not externalized to `config/*.ron`. Building a real hot-reload config
  system is itself a multi-hour infrastructure task, not a per-section add-on.
- **Full PBR mech materials** — this engine has zero image-texture pipeline
  (confirmed in the §0 audit: no `asset_server.load` of any image anywhere).
  "Material pass" here means color/roughness/metallic variety on procedural
  primitives, not authored textures.
- **Mech movement crawl-speed during the 1.6s entry window** — firing is
  blocked (tested), movement slowdown during entry is not separately
  implemented.
- **Forge UI** — no visual grid/turntable editor; save/load is a real,
  tested keybind system (Ctrl+1/2/3, 1/2/3) over the existing cosmetic
  fields, not a new part-swap pipeline (this engine has no glTF-part
  system to swap).
- **Third-person locomotion facing-vs-aim blend** — the torso-limit clamp
  function is built and tested; it is not yet wired into the live
  character-facing/turn-in-place animation logic.
