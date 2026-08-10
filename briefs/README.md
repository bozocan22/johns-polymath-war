# Briefs — design specifications and executable prompts

Two kinds of file live here.

**`BRIEF_*.md`** are specifications. They describe what the game should be, with numbers,
constraints and test gates. They are reference material — read the ones a task touches.

**`PROMPT_*.md`** are executable workflows. They are written to be pasted into a Claude Code
session as the whole task. Each contains an operating contract, numbered tasks, test gates and
capture requirements. Paste the prompt; do not paste the briefs alongside it — the prompt tells
the session which briefs to read.

## Index

| File | Kind | Covers |
|---|---|---|
| `BRIEF_VII_optimized.md` | spec | Living motion, limb and hand craft, spear throw, bow, third-person camera, mech overhaul, Forge. Introduces operating contracts C1–C9 and the "visible or it didn't happen" rule. |
| `BRIEF_VIII_master.md` | spec | **Master brief.** Consolidates briefs II–VII into one standalone document. Motion doctrine, first-person, HUD anatomy, spear, mech, third-person camera, Forge, handback. Read this first if you are new to the project. |
| `BRIEF_VIII_B_addendum.md` | spec | Corrections after concept art. 20-segment body with mass fractions and joint limits, the elastic load model (stretch-shortening cycle), mech scale decision, mech visual spec with palette. |
| `BRIEF_IX_castle_grenade_customization.md` | spec | Castle maps with three elevation tiers and sightline rules; grenade opening mechanics, blast falloff and environmental interaction; layered character customization bound to the 26-piece armour set. |
| **`PROMPT_MASTER_research_build.md`** | **prompt** | **The one to paste.** 13-task workflow covering eight research topics — first-person dynamics, aiming, traversal (dodge/jump/flip/climb/vault), layered character creation, weapon systems (reload FSM / render / runtime console), grenade aiming physics, map design, and powered armour — then synthesis, implementation, tests, captures, report. Ships with a seeded source ledger. Supersedes the two prompts below. |
| `PROMPT_RND_CYCLE.md` | prompt | Repeatable R&D cycle: one system per cycle, researched properly, built, proven, reported — then the backlog grows. Depth floor (≥1 tier-P source read end to end) replaces breadth quotas, because a breadth quota manufactures fabrication. Ships with a feasibility-ranked initial backlog: mech entry sequence, infantry-vs-giant-mech, and grenade surface materials are Critical; graphics and networking are Blocked with their unblockers named. |
| `PROMPT_motion_system_research.md` | prompt | Motion and maneuvering architecture decision: locomotion, dodging, parkour, IK, ragdoll, motion matching versus state machines versus neural animation. Retargeted to Rust/Bevy/rapier — UE5 and studio systems are reference-only. Ends in a decision with rejections and a migration path, not a survey. Research-only; does not implement. |
| `PROMPT_mech_rebuild.md` | prompt | *Superseded.* 6-task mech workflow: audit, reference gathering, 20-segment body rebuild, elastic load model, scale decision, mech rebuild, report. Kept for the mech-specific detail not carried into the master prompt. |
| `PROMPT_brief_X_research.md` | prompt | *Superseded by the master prompt,* which absorbs all three of its topics. Kept for its per-section changelog. |

## If you are pasting something into Claude Code

Paste `PROMPT_MASTER_research_build.md`, between its BEGIN and END markers. Nothing else.
It reads the specification briefs from disk itself.

## Reading order for someone new

1. `BRIEF_VIII_master.md` — the consolidated spec
2. `BRIEF_VIII_B_addendum.md` — the corrections that supersede parts of it
3. `BRIEF_IX_castle_grenade_customization.md` — maps, grenades, customization
4. `../engine/crates/jk_tdm/research/TREVOR_TASKS.md` — what is actually built versus
   what is specified, ranked and ready to pick up (`TREVOR_LEDGER.md` behind it for the
   full record). **Corrected 2026-08-10:** was `../DESIGN_MAP.md`, which maps `jk_wall`,
   a different game, and never mentions `jk_tdm`.

`BRIEF_VII_optimized.md` is superseded by VIII for content, but is kept because it is where the
operating contract originates.

## Rules that carry across every brief

- **Visible or it didn't happen** — a feature is done when it appears in a capture from the
  build that was actually launched, not when the test passes.
- **SIM vs COSMETIC** — every system declares which layer it lives in. A cosmetic system that
  touches simulation state is a bug.
- **Tunables are data** — no magic numbers in source. Everything lands in config.
- **Player intent wins** — procedural motion yields to direct input within one frame.
- **The squint test** — any visual decision must survive rendering as a solid black silhouette
  at 30 m. If two weapons read the same in black, change the carry pose, not the weapon.
