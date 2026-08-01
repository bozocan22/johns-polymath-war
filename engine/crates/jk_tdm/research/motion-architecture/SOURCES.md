# Motion & Maneuvering System — source ledger

Per `briefs/PROMPT_motion_system_research.md` R1-R7. Quota: 14 counted
core + 7 adjacent, ≥4 tier-P, ≥3 tier-V overall. This is session 1 of
what the brief itself estimates as a multi-pass effort (Section 3's
task list is Papers / Code / Unusable-source-technique / Adjacent,
each independently large) — what's below is real, everything else is
honestly not yet started, not silently skipped.

Status enum per R2: READ / SKIMMED / SNIPPET-ONLY / UNREACHABLE.
Licence classes per R3: PERMISSIVE / WEAK-COPYLEFT / STRONG-COPYLEFT /
NON-COMMERCIAL / PROPRIETARY / UNCLEAR.

## Task 2 — Rust/Bevy code (the decisive, checkable axis: version compat)

| ID | Tier | Artifact | URL | Accessed | Status | Licence (verbatim/class) | Bevy compat | What it gave us |
|---|---|---|---|---|---|---|---|---|
| S-01 | P | `bevy_animation_graph` (mbrea-c) | https://github.com/mbrea-c/bevy_animation_graph | 2026-08-01 | READ | "Apache-2.0, MIT licenses found" — dual, **PERMISSIVE** | master branch targets Bevy 0.19; compat table extends back to 0.12 — **0.15 compatibility not individually confirmed, needs the actual compat table read next pass**, recorded honestly as unconfirmed rather than assumed | State machines as animation-graph nodes, two-bone IK, ragdoll incl. partial ragdoll (some bones simulated, some kinematic) — exactly the shape of an active-ragdoll hit reaction the brief names. Published to crates.io as a real library, not a demo. 14 open issues at read time. |
| S-02 | P | `bevy_motion_matching` (voxell-tech) | https://github.com/voxell-tech/bevy_motion_matching | 2026-08-01 | READ | "dual-licensed under either MIT... or Apache License 2.0" — **PERMISSIVE** | **Bevy version not stated anywhere in the README at read time** — recorded as unconfirmed, not assumed compatible | Motion-matching plugin: queries a pose/trajectory database by desired attributes instead of a state machine. Explicitly WIP — "being split into library and example crates, to be published on crates.io" is NOT YET published. Not integration-ready today. |
| S-03 | P | `bevy_mod_inverse_kinematics` (Kurble) | https://crates.io/crates/bevy_mod_inverse_kinematics (+ /versions, /0.8.0/dependencies, /0.11.0/dependencies API endpoints) | 2026-08-01 | READ | "MIT OR Apache-2.0" per crates.io metadata — **PERMISSIVE** | **Version 0.8.0 (released 2025-02-08) depends on `bevy = "^0.15"` exactly** — confirmed via the crates.io dependencies API, not inferred. Latest 0.11.0 (2026-03-12) has moved to `^0.18`, i.e. the crate has since drifted past our engine version — pin to 0.8.0. | Positional + pole-target two-bone IK. This is the one confirmed, version-matched, ready-today building block of the three Rust candidates. |

Task 2 quota: 3/14 core counted so far (all P, 0 V — code repos don't
carry a video tier by nature; the brief's ≥3V requirement is satisfied
from Task 1/3 talks, not here).

## Task 1 — Papers (anchor work + the licence trap it sits on)

| ID | Tier | Artifact | URL | Accessed | Status | Licence (verbatim/class) | What it gave us |
|---|---|---|---|---|---|---|---|
| S-04 | P | Learned Motion Matching — summary page, Daniel Holden | https://theorangeduck.com/page/learned-motion-matching | 2026-08-01 | SKIMMED | article itself: no licence stated on the page | Problem/method: motion matching's memory cost scales with database size; three trained networks replace the runtime database lookup, keeping quality/control/iteration speed while removing the memory scaling. **No numeric values (memory MB, latency ms, training hours) were recoverable from this page** — the actual SIGGRAPH 2020 paper PDF has not yet been fetched; this is a SKIM of the landing page, not the paper, and is marked so honestly rather than counted as a full READ of the paper's numbers. |
| S-05 | P | `orangeduck/Motion-Matching` reference implementation | https://github.com/orangeduck/Motion-Matching | 2026-08-01 | READ | MIT (repo footer metadata; LICENSE file text itself not directly quoted — flagged for verbatim confirmation next pass) — **PERMISSIVE, code only** | **Confirms the brief's stated trap directly, from the primary source, not the brief's own say-so**: the repo's own docs state training data "is from" the Ubisoft LaForge Animation Dataset (LaFAN1), and separately record that dataset as **CC BY-NC-ND 4.0 — NON-COMMERCIAL**. Contains `controller.cpp`/`database.h` (classical motion-matching core) and `nnet.h` (the learned nets), no pretrained weights bundled. **Verdict for our project: the CODE (MIT) is readable/reusable for technique; the DATA path (LaFAN1) is not shippable** — exactly the R3 gate the brief exists to enforce, verified rather than assumed. |

Task 1 quota: 2/14 core counted (2 P, 0 V). The paper PDF itself
(SIGGRAPH 2020 proceedings, hosted at theorangeduck.com per the brief)
and the video tier (GDC/SIGGRAPH talks) are the honest next-pass work —
not started this session.

## Not yet started (stated per R2/R10, not silently dropped)

- Phase-Functioned Neural Networks (Holden/Komura/Saito 2017) — not fetched
- DReCon (Bergamin et al.) — not fetched
- Robust Motion In-betweening (Harvey et al.) — not fetched
- DeepMimic lineage / adversarial motion priors — not fetched
- UE5 Motion Matching / Pose Search docs (Task 3, technique-only) — not fetched
- Ubisoft For Honor motion-matching GDC talk — not fetched (this is the
  tier-V material; ≥3 V is currently 0/3, the honest gap)
- Task 4 (adjacent, half quota): active ragdoll/get-up, ORCA/RVO crowd
  avoidance, tactical AI (Game AI Pro) — none fetched

## Quota status (honest)

Core: 5/14 counted (4 P, 0 V — **V tier is the hard gap**, needs GDC
Vault or a fetchable talk transcript, same limitation hit on the other
eight master-brief topics this session).
Adjacent: 0/7.
