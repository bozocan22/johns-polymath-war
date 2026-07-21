# Architecture Decision Records — John Kingdom Game

## ADR-006 — Art pipeline & engine strategy: no Unreal, Bevy at the art milestone, Blender for assets (2026-07-17, accepted)

**Question.** Should the project connect to Blender, Unreal Engine, or
similar (including via MCP tooling)?

**Decision.**
1. The deterministic Rust simulation (`jk_core`, `jk_wall`) is the
   project's crown jewel and stays engine-agnostic forever. Any renderer —
   macroquad today, Bevy later, even Unreal hypothetically — is a thin
   client over the same sim API (`WallSim`, `set_command`,
   `set_player_input`, telemetry).
2. **Unreal Engine: no.** It would replace our client, not help it; the
   sim would live as a native plugin inside an engine whose own systems
   (actors, PhysX/Chaos, animation authority) fight ours, with C++
   complexity, lock-in, and royalties — to buy photoreal rendering that
   isn't our aesthetic. Reconsider only under publisher/console pressure,
   as a client-only port.
3. **Bevy at the art milestone (~M7):** Rust-native 3D rendering, glTF
   loading, skeletal animation — the client swap ADR-002 planned for.
4. **Blender (or Blender MCP) when assets are needed, not before.**
   Blender authors content (models/rigs/animations → glTF); it is not a
   runtime dependency. Until the art milestone, primitive rendering is a
   feature: every hour goes into the sim. Low-poly assets may also be
   generated procedurally in code, which suits the committed aesthetic.

**Significance of MCP tooling.** Blender/Unreal MCPs change what the
AI developer can operate during development (e.g. authoring and exporting
models directly); they change nothing about the game's architecture and
ship nothing.

## ADR-001 — Language: Rust (2026-07-16, accepted)

**Decision.** The engine is a Rust cargo workspace.

**Why.** The feasibility report (`../shieldwall_reforged/05_FEASIBILITY.md`)
weighed Rust+ecosystem vs C++ + custom + Jolt. For a sim with three coupled
subsystems and a determinism requirement, ownership discipline and the cargo
test culture win the first year; the physics backend is the only place C++
interop may be needed (see ADR-002). Data-oriented layout (struct-of-arrays
in `jk_wall`) rather than a full ECS until entity counts demand it.

## ADR-002 — Physics backend: trait-abstracted; Rapier now, Jolt at client stage (2026-07-16, accepted)

**Decision.** `jk_wall` talks to physics through a narrow backend trait
(bodies, capsule colliders, force application, contact queries). The first
implementation is **Rapier3d** (crates.io, pure Rust). **Jolt** (via joltc
bindings or C++ sidecar) replaces it when the spike graduates to a rendered
client.

**Why.** The research verdict stands: Jolt is the endgame backend (only
candidate with cross-platform determinism + multithreaded islands +
intra-island splitting — the wall is one island). But the spike's kill
criterion is about *behavior* (does the wall read as a wall), not throughput;
Rapier at 80 bodies is far from any ceiling, builds from crates.io in this
environment with zero native deps, and keeps the workspace `cargo test`-able
anywhere. The trait keeps the swap cheap; the sim math (push, stamina,
cohesion) never touches backend types directly.

**Revisit when:** body count target >300, or client work starts.

## ADR-003 — Milestone 1 scope: headless spike, offline visualization (2026-07-16, accepted)

**Decision.** Milestone 1 renders no window. The 40v40 runs headless at
120 Hz and emits: a metrics CSV, top-down PNG frames, an animated GIF, and a
battle report with the emergent per-rank force attenuation (α), cohesion
timeline, and breach log.

**Why.** This session's environment is a cloud container (no display), and
the kill criterion is measurable without a renderer: formation hold, push
transmission, cascade collapse. Real-time rendering enters at milestone 2
(client window, interpolated 60 Hz presentation over the 120 Hz sim, per
Pillar P5).

## ADR-005 — Third-person direction (2026-07-17, accepted)

**Decision.** John Kingdom Game is a 3D third-person battle game in the mold
of the original *Shieldwall* (Nezon Production): the player is a soldier
standing inside the formation, fighting the press with their body and
commanding the wall with single keys. Implemented per the research in
`../shieldwall_reforged/research/05_third_person_design.md`:

- **Control authority**: the player is a velocity-servo-driven body whose
  force cap sits BELOW peak crowd pressure (being shoved is physical drama,
  not input denial — Half Sword/Gang Beasts lesson, inverted: responsive
  input, physical consequences). Shoulder-in (SHIFT) raises effort;
  compression above the brace limit degrades everyone's output equally.
- **Camera**: spring-damped chase cam, never parented to the body (crowd
  jitter must not reach the lens); high-set (~3.8 m) to read the line;
  occlusion cull of bodies between lens and player (Game AI Pro ch.47
  pattern). Hard lock-on rejected (For Honor's outnumbered failure).
- **Command vocabulary**: 4 orders (Advance/Hold/Brace/Charge) mapped to
  physics levers (speed, spacing, push authority, brace tolerance, stamina),
  not stat toggles — the original Shieldwall's 3 commands failed because they
  were "a glorified button press," not because 3 was too few. Rotate-ranks
  joins in M3 as the stamina-synergy order.
- **Melee verbs** (M3): discrete buffered inputs resolved by the physics
  (energy/penetration model), NOT physics-driven swing trajectories —
  Half Sword/Exanima's 5-hour learning wall is wrong for this game.

## ADR-004 — Constants provenance (2026-07-16, accepted)

**Decision.** `jk_core::constants` is the single source of numeric truth.
Every constant carries a doc comment: `SOURCED(research/NN)` or
`PROVISIONAL(reason)`. No literal magic numbers in sim code.

**Why.** Pillar P1 (every number derived, not authored) and the calibration
work in `../shieldwall_reforged/07_CONSTANTS.md` are only worth anything if
the game code can't silently drift from them.
