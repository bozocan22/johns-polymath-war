# Master brief Task 0 — audit table for jk_tdm

Branch `claude/shieldwall-reforged-research-k2zm28`, merged to `main` at
`f2fe3f1`. 131/131 tests green at audit time; latest live run exit 0,
0 panics. (DESIGN_MAP.md covers the *wall-sim* project; this table is
the TDM arena shooter the master brief's Half 2 targets.)

Every row below was verified against code this week — most by the four
adversarial audit waves, several the hard way (found broken, fixed,
tested).

| System | Files | Coded? | Tested? | Visible in launched build? | Root cause if not visible |
|---|---|---|---|---|---|
| Camera (3rd person states, collision boom, landing dip/rebound) | main.rs `camera_system`, `boom_step`, `CameraTuning` | ✅ | ✅ (boom spring scope, rebound crosses neutral, config no-op fallback) | ✅ captures | — |
| Viewmodel (1st person, sway, recoil channels, cheek weld) | main.rs `fp_viewmodel`, `VmState` | ✅ | ✅ (spread/recoil goldens) | ✅ — now actually LIT (the scene light was missing the VM render layer until wave 3) | — |
| Aiming / ADS (two-stage scope, zoom-sens match, heat-widened cone) | sim `aim_spread`, main `stability_bracket` | ✅ | ✅ (shared single source, label-matches-binding) | ✅ | — |
| Dodge / roll / jump / flip | sim roll+flip blocks | ✅ (+ counter-movement launch, new) | ✅ (`a_counter_movement_dodge_launches_harder`) | ✅ | — |
| Climb / vault / mantle | — | ❌ none | — | — | never built; traversal is a master-brief Half-2 topic, blocked on its research quota |
| Turn-in-place (legs lag, torso covers) | main `step_leg_yaw` | ✅ (new this week) | ✅ | ✅ | — |
| Character / loadout screen | main intro UI, `Selected` | ✅ | ✅ (layout fixed after overlap bug) | ✅ `menus` capture | — |
| Class/armour-piece system (26-piece, weights) | — | ❌ (only `armor_weight_movement_penalty` formula exists, unwired) | formula only | — | IX-C is a full character-system build; nothing to wire the formula to yet |
| Weapon state machine (fire/reload/switch/heat/draw) | sim `try_fire`, `step_bow_draw`, minigun block | ✅ | ✅ (incl. bow spread+auto-nock fixed in wave 2) | ✅ | — |
| Reload (end-commit, cancel policy) | sim `try_reload` | ✅ | ✅ | ✅ | — (verified: no early-commit exploit surface; cancel-not-pause is a recorded design choice) |
| Grenades (per-surface bounce, falloff, cook, preview=flight) | sim `grenade_tick`, `frag_falloff_frac` | ✅ | ✅ (+R11: 1000 seeded throws bit-identical) | ✅ | — |
| Map / level data (4 maps, cover grid, sightline validator) | sim `MapLayout`, `max_unobstructed_sightline` | ✅ | ✅ (validator instrument-checked; all 4 maps measured vs the 40 m rule) | ✅ | castle map itself: ❌ content build, not started |
| Console / command system | — | ❌ none | — | — | never built; the image-import console is blocked upstream on there being no texture pipeline at all |
| Runtime asset loading | audio only (`asset_server.load` of .wav) | 🟡 | audio play-paths verified | ✅ sound | zero image/texture loading anywhere — every material is procedural colour |
| Capture harness (the evidence instrument) | main capture_* systems | ✅ | ✅ (snap-once verified 157→5; typo names exit 2; clear staging) | ✅ it IS the visibility mechanism | — |
| Settings (sens/FOV/invert/swap/minimap) | main `GameSettings`, `persist_settings`, `load_settings` | ✅ | ✅ (round-trip + clamp-on-garbage test) | ✅ capture | — (closed 2026-08-01, Section E: `config/settings.txt`, loaded at startup, rewritten on any change) |
| Powered armour / mech | sim mech blocks, main `spawn_armor_rig` | ✅ (walker: committal enter/exit, plates, parity) | ✅ | ✅ captures | flight: deliberately deleted (Brief VI §4.3); Iron-Man-style suit is research-only per the master brief's own rule |

## Before-clips (Task 0 §3)

Existing committed captures already cover (a) walk/look 1st+3rd person
(`baseline/`), (b) ADS+fire (`baseline/03-04`), (d) weapon-feel under
fire (`minigun_check/`), and the mech set. Missing vs the letter of the
brief: a dedicated traversal clip (c) and a full map lap (e) — the
capture harness drives scripted beats, and a "one lap of every elevation"
script is real work recorded as TODO, not silently skipped (R10).
