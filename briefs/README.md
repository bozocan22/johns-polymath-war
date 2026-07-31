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
| `PROMPT_mech_rebuild.md` | prompt | 6-task workflow: audit, reference gathering, 20-segment body rebuild, elastic load model, scale decision, mech rebuild, report. |
| `PROMPT_brief_X_research.md` | prompt | 8-task workflow: self-directed research into first-person dynamics, layered character creation, and reload / weapon render / runtime asset console — then implement, test, capture, report. Research lands in `research/` and is committed. |

## Reading order for someone new

1. `BRIEF_VIII_master.md` — the consolidated spec
2. `BRIEF_VIII_B_addendum.md` — the corrections that supersede parts of it
3. `BRIEF_IX_castle_grenade_customization.md` — maps, grenades, customization
4. `../DESIGN_MAP.md` — what is actually built versus what is specified

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
