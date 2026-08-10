# PROMPT — Motion & Maneuvering System: Research and Architecture Decision

Optimized from a draft that targeted Unreal Engine 5. **This project is Rust + Bevy 0.15 +
rapier3d.** That single correction changes most of the answer, so it is applied throughout.

Paste everything between the BEGIN and END markers into Claude Code.

---

## Why the original draft would have failed

| # | Draft said | Problem | Fix applied |
|---|---|---|---|
| 1 | "Identify reusable code suitable for integration into **Unreal Engine 5**" | The repo is Rust/Bevy 0.15/rapier3d 0.22. UE5 code is not portable here, and Epic's sample code is under the **Unreal EULA — not open source** and cannot be copied into a non-Unreal project at all. A session pointed at UE5 returns a shopping list you legally and practically cannot use. | Retargeted to Rust/Bevy. UE5, Ubisoft and Naughty Dog work is **reference-only: read the technique, never the code.** |
| 2 | "Prioritize permissively licensed" | Stated as a preference with no verification step, which is how NC-licensed assets end up in commercial builds. **LAFAN1 — the dataset behind Learned Motion Matching, DReCon and Robust Motion In-betweening — is CC BY-NC-ND 4.0: NonCommercial *and* NoDerivatives.** | License verification is a **hard gate**. Every artifact gets its licence recorded and classified before it can be recommended. |
| 3 | Nothing about determinism | `rapier3d` is already configured with `enhanced-determinism`, and the sim has a passing replay-determinism suite. Motion matching does a nearest-neighbour search; neural animation does float-heavy inference. Both can break bit-exact replay. This constraint **eliminates some architectures outright** — the draft would never surface it. | Determinism is a first-class evaluation axis. Every candidate architecture is scored on it, and the SIM/COSMETIC split decides what is even allowed to be non-deterministic. |
| 4 | "Rank the top research papers by practical usefulness" | Ranking is busywork. A ranked list is not a decision, and nothing is buildable at the end. | Deliverable is **one chosen architecture with a defended rejection of the alternatives**, plus a migration path from the code that exists today. |
| 5 | No asset-pipeline reality | Motion matching needs hours of mocap you are licensed to ship. Neural animation needs training data and infrastructure. Without an answer to "where does the motion data come from," an architecture recommendation is fiction. | Added a **feasibility gate**: any architecture requiring a motion database must name a specific, licence-cleared source, or be marked unavailable. |
| 6 | No stopping condition | "Comprehensive research" with no quota ends when the model gets bored. | Quotas per axis, and a verification contract that makes unread sources uncountable. |
| 7 | Ignored existing work | The repo already has a 20-segment body, an elastic load model, a kinetic-chain utility, and a working traversal set. The draft would have researched them from scratch and proposed replacing them. | Prompt reads the briefs first and must state, per system, **extend / replace / leave alone** with a reason. |
| 8 | Scope sprawl — vehicles, drones, "if applicable" | Unbounded scope with an escape hatch produces thin coverage everywhere. | Scope tiered: **core** (must), **adjacent** (should), **deferred** (name and stop). Vehicles and drones are deferred. |
| 9 | "Deep learning research" as the framing | Conflates *deep research* with *deep learning*. Motion matching is not deep learning; Learned Motion Matching, PFNN and RL controllers are. Merging them muddies the comparison. | Separated into **classical**, **data-driven**, and **learned** families, compared on the same axes. |

---

=== BEGIN PROMPT ===

# Motion & Maneuvering System — Research and Architecture Decision

## Read first

1. `CLAUDE.md`
2. `projects/john_kingdom_game/briefs/README.md`, then `BRIEF_VIII_master.md` and
   `BRIEF_VIII_B_addendum.md` — these define the **20-segment body**, the **elastic load model**
   (load then release, release 2–3× faster than load, stored energy scales output), and the
   **kinetic chain utility** (proximal-to-distal sequencing). These already exist and are
   specified. You are not redesigning them.
3. `engine/crates/jk_tdm/research/TREVOR_TASKS.md` + `TREVOR_LEDGER.md` — what is
   actually built. **Corrected 2026-08-10:** was `DESIGN_MAP.md`, which maps
   `jk_wall`, not this game.
4. The engine crates: `jk_core` (fixed timestep, seeded RNG), `jk_wall`, `jk_bevy`, `jk_tdm`.

Branch: `claude/motion-architecture`.

## The actual target

**Rust. Bevy 0.15. rapier3d 0.22 with `enhanced-determinism`. macroquad for the lighter client.**

Unreal Engine 5, Unity, and studio-proprietary systems are **reference-only**. Read them for
technique and numbers. Never propose importing their code:
- Unreal sample content (Lyra, Game Animation Sample, Motion Matching plugin) is under the
  **Unreal Engine EULA**, not an open-source licence. It cannot be copied into a Rust project.
- Unity assets and Naughty Dog / Rockstar / EA systems are proprietary and unavailable.

A recommendation that requires porting UE5 code is a failed deliverable. A recommendation that
says "UE5's Pose Search does X, here is how to achieve X in Bevy with these crates" is what is
wanted.

## Objective

Decide the movement and animation architecture for this game and prove the decision. Not survey
it — **decide it**, with the losing options rejected on stated grounds, and a migration path
from the code that exists today.

---

## SECTION 0 — CONTRACT

**R1 — No claim without a source row.** Every number and every capability claim traces to a
numbered row in `research/motion-architecture/SOURCES.md`. Tag inline: `[S-12]`.

**R2 — Never invent a source.** Unreachable, paywalled, or transcript-less sources are recorded
with that status and do not count. If you only saw a search snippet the status is
`SNIPPET-ONLY`, never `READ`. **A fabricated benchmark or quote fails the session.**

**R3 — Licence before recommendation.** No repository, dataset, or model may be *recommended*
until its licence is recorded verbatim from the LICENSE file or the repository metadata — not
inferred from the README, not assumed from the platform. Classification:

| Class | Meaning | May we ship it? |
|---|---|---|
| `PERMISSIVE` | MIT, Apache-2.0, BSD, Zlib, CC0, dual MIT/Apache | Yes |
| `WEAK-COPYLEFT` | MPL-2.0, LGPL | Yes with conditions — state them |
| `STRONG-COPYLEFT` | GPL, AGPL | No — would infect the game |
| `NON-COMMERCIAL` | CC BY-NC, CC BY-NC-ND, research-only | **No.** Research reference only |
| `PROPRIETARY` | Unreal EULA, Unity asset EULA, closed | **No** |
| `UNCLEAR` | No LICENSE file, contradictory terms | Treat as PROPRIETARY until resolved |

**Known trap, already verified:** the Ubisoft LaFAN1 dataset — 5 subjects, 77 sequences,
496,672 frames at 30 fps, ~4.6 hours, BVH — is **CC BY-NC-ND 4.0**. NonCommercial *and*
NoDerivatives. It is the dataset behind Learned Motion Matching, DReCon, Robust Motion
In-betweening and Recurrent Transition Networks. **You may read every one of those papers. You
may not ship anything trained on or derived from that data.** Check whether any repository you
evaluate embeds it, and flag it if so. Assume other academic motion datasets carry similar
terms until you verify otherwise.

**R4 — Determinism is a hard constraint, not a preference.** `rapier3d` runs with
`enhanced-determinism` and the replay suite passes. Classify every candidate technique:

- **SIM-safe** — deterministic under fixed timestep, reproducible from a seed, no float drift
  across platforms. May drive gameplay state.
- **COSMETIC-only** — may vary between clients without affecting outcomes. May drive visible
  pose but never hit registration, damage, or movement collision.

An architecture that puts a non-deterministic component in the SIM layer is rejected on those
grounds regardless of how good it looks. **State the classification for every technique before
recommending it.** Note explicitly: a neural network's *inference* can be deterministic on one
platform and drift across platforms — check, do not assume.

**R5 — Feasibility gate.** Any architecture requiring a motion database must answer: where does
the data come from, what does it cost, what is its licence, how many hours are needed for
acceptable coverage, and who authors it. "Use mocap" without a licence-cleared source means the
architecture is **unavailable to this project** — say so plainly rather than recommending it.

**R6 — Extend, don't replace by default.** For each existing system — 20-segment body, elastic
load model, kinetic chain, current traversal set, `jk_wall` locomotion — state **extend /
replace / leave alone** with a reason. Replacement needs a stronger argument than extension.

**R7 — Research is committed.** Everything lands in `research/motion-architecture/` and is
committed.

---

## SECTION 1 — SCOPE, TIERED

**Core — must cover, full quota:**
- Locomotion: walk, run, sprint, crouch, and the transitions between them
- Acceleration, deceleration, turning, and momentum preservation
- Dodging and evasive movement
- Parkour and obstacle traversal (vault, mantle, climb)
- Foot placement and IK on uneven ground and stairs
- Animation blending, transitions, and the state-machine-versus-alternatives question
- Root motion versus procedural, and where each belongs

**Adjacent — cover at half quota:**
- Ragdoll transitions: active ragdoll, get-up, and the blend back to animation
- Physics-based movement and how it interacts with a deterministic solver
- Tactical AI movement: cover seeking, peeking, suppression response
- Predictive path planning and dynamic obstacle avoidance

**Deferred — name the best resource, then stop:**
- Vehicle and drone dynamics. Not in this game. One paragraph, no quota.

---

## SECTION 2 — THE THREE FAMILIES

Compare on the same axes. Do not blur them — they have different costs and different failure
modes.

**Family A — Classical.** Hand-authored state machines, blend trees, blend spaces, additive
layers, procedural IK. What every shipped game did before ~2016 and most still do underneath.

**Family B — Data-driven.** Motion matching: a database of poses searched each frame for the
best continuation given current pose and desired trajectory. No state machine. Memory and search
cost scale with database size.

**Family C — Learned.** Neural approaches: Learned Motion Matching (compresses the database into
networks, solving the memory scaling), phase-functioned and mode-adaptive networks,
reinforcement-learning physical controllers (DeepMimic-lineage, adversarial motion priors),
motion in-betweening and diffusion models.

### Evaluation axes — score every family and every specific technique on all nine

| Axis | What to establish |
|---|---|
| 1. Responsiveness | Latency from input to visible response, in ms and frames. This is the axis players feel. |
| 2. Realism | Quality of the resulting motion, and how it degrades under bad input |
| 3. Determinism | SIM-safe or COSMETIC-only per R4, with the reason |
| 4. Memory | Runtime footprint, and how it scales with content |
| 5. CPU | Per-character per-frame cost, and cost at our crowd counts — `jk_wall` targets hundreds of bodies |
| 6. Asset cost | Hours of motion data required, and its licence per R5 |
| 7. Authoring cost | What a human must produce and maintain per new move |
| 8. Rust/Bevy availability | What exists in this ecosystem today, and its maturity |
| 9. Integration cost | Distance from the current codebase |

**Axis 5 is decisive and the original draft missed it entirely.** This project runs large
battles — `jk_wall` is benchmarked at 250v250 and the research ceiling is 400–700 full-physics
bodies. A per-character cost that is fine for one hero character may be impossible for a crowd.
The answer may legitimately be **two architectures**: one for the player and nearby characters,
a cheaper one for the crowd, with a defined LOD handoff. Evaluate that explicitly.

---

## SECTION 3 — RESEARCH TASKS

Quota: **14 counted sources for core axes, 7 for adjacent**, with ≥4 tier-P (peer-reviewed
paper, SIGGRAPH/GDC talk, engine source, official docs) and ≥3 tier-V (talk with timestamped
quotes) overall.

### Task 1 — Papers

Venues: SIGGRAPH and SIGGRAPH Asia, ACM TOG, Eurographics/SCA, arXiv cs.GR and cs.RO, plus
CVPR/NeurIPS for motion synthesis and ICRA/IROS for legged control.

Anchor works to find and read — verify each yourself, these are pointers not citations:
- **Learned Motion Matching**, Holden, Kanoun, Perepichka, Popa — SIGGRAPH 2020, Ubisoft La
  Forge. Solves motion matching's linear memory scaling with neural networks. PDF is hosted at
  `theorangeduck.com`. **Start here** — it is the bridge between families B and C.
- **Phase-Functioned Neural Networks for Character Control**, Holden, Komura, Saito —
  SIGGRAPH 2017. The origin of the learned-controller line.
- **DReCon: Data-Driven Responsive Control of Physics-Based Characters**, Bergamin et al. —
  physics plus motion matching, directly relevant to a rapier-based sim.
- **Robust Motion In-betweening**, Harvey et al. — transition generation.
- The **DeepMimic** lineage and adversarial motion priors — RL physical controllers.
- Daniel Holden's site `theorangeduck.com` — he publishes long technical articles **with
  working code**, including a motion-matching implementation. Check licences on each.

For each paper record: the problem, the method in three sentences, the reported numbers
(memory, latency, training time, quality metric), what data it used **and that data's licence**,
whether an implementation exists, and its determinism story.

### Task 2 — Code

Search crates.io, lib.rs, and GitHub. Rust and Bevy first; C++/C# only for technique.

Rust/Bevy candidates confirmed to exist — evaluate each properly:
- `bevy_animation_graph` (mbrea-c) — animation graphs with **state machines embedded as graph
  nodes**, two-bone IK, and **visual ragdoll editing with partial ragdolls** where some bones
  simulate while others stay kinematically driven. Closest thing to a ready foundation.
- `bevy_animation_graph_editor` — visual editor for the above.
- `bevy_motion_matching` (voxell-tech) — motion matching for Bevy, being split into library and
  example crates for crates.io.
- `bevy_mod_inverse_kinematics` — positional and pole targets.

For **every** repository record: licence verbatim per R3, last commit date, open issues, Bevy
version compatibility (**0.15 — Bevy breaks API between minor versions and this matters more
than anything else in the table**), test coverage, dependency weight, whether it is a library or
a demo, and an honest maturity read. A promising crate pinned to Bevy 0.12 is a rewrite, not an
integration — say so.

### Task 3 — Technique from unusable sources

Read UE5's Motion Matching / Pose Search documentation, the Game Animation Sample writeups,
Ubisoft's For Honor motion-matching talk, and studio postmortems **for numbers and technique
only**. Extract: database sizes, per-frame search costs, feature-vector composition (what
actually goes into the matching features), trajectory prediction windows, and the tuning
failures they hit. Record clearly that the code is unusable and only the technique transfers.

### Task 4 — Adjacent (half quota)

Active ragdoll and get-up; physics-based movement under a deterministic solver; tactical AI
movement — cover evaluation, peek mechanics, suppression response; path planning and dynamic
avoidance. For tactical AI, prefer GDC AI Summit material and the *Game AI Pro* series; for
avoidance, look at ORCA/RVO and its known failure modes at high density, which matters for our
crowd counts.

---

## SECTION 4 — THE DECISION

`research/motion-architecture/DECISION.md`:

1. **The recommended architecture**, concretely: what runs for the player, what runs for nearby
   characters, what runs for the crowd, and the LOD handoff between them. Which crates, which
   custom code, which existing systems retained.
2. **Every rejected alternative**, with the axis that killed it. "Motion matching rejected —
   requires ≥N hours of licence-cleared mocap we do not have [S-nn], and per-character search
   cost of X ms is untenable at 250 bodies [S-nn]" is a real rejection. "Not the best fit" is not.
3. **The nine-axis scoring table**, filled, with source ids.
4. **Migration path** — ordered, from the code that exists now, each step independently shippable
   and independently revertible. No big-bang rewrite.
5. **What this costs** — engineering time, asset production, runtime budget.
6. **What it does to determinism** — per R4, the SIM/COSMETIC line drawn explicitly, and what
   the replay suite must still pass.
7. **Extend / replace / leave alone** for each existing system, per R6.
8. **The honest risk** — what could make this the wrong call in a year, and the early warning
   sign to watch for.

**Then stop. Do not implement.** The decision, reviewed, comes before the build. If the decision
is obvious and cheap in one area — foot IK, say — note it as a quick win, but do not start.

---

## SECTION 5 — TESTS

| Test | Asserts |
|---|---|
| `source_quota` | 14 counted core, 7 adjacent, ≥4 P, ≥3 V |
| `no_snippet_only_counted` | no `SNIPPET-ONLY` row counted |
| `every_repo_has_licence` | every recommended artifact has a verbatim licence and a class |
| `no_noncommercial_recommended` | nothing classed NON-COMMERCIAL or PROPRIETARY appears in the recommendation |
| `no_orphan_numbers` | every number in DECISION.md carries a source id |
| `bevy_version_recorded` | every Rust crate has its Bevy compatibility recorded |
| `determinism_classified` | every technique is marked SIM-safe or COSMETIC-only with a reason |
| `rejections_have_reasons` | every rejected alternative names the axis that killed it |

## SECTION 6 — REPORT

1. Ledger summary — counted, tier P, tier V, unreachable, licences by class
2. The nine-axis table
3. The recommendation in one paragraph a person could act on
4. The rejections, one line each
5. The migration path as an ordered list
6. The determinism verdict
7. **Anything you could not verify**, plainly — unreachable papers, repos with no licence file,
   benchmarks you could not reproduce. Per R2 this is expected and reporting it is correct
   behaviour, not failure.

Commit and push to `claude/motion-architecture`. No pull request unless asked.

## SECTION 7 — FAILURE CONDITIONS

- A source marked `READ` was not read (R2)
- A recommended artifact has an unverified or misreported licence (R3)
- Anything NON-COMMERCIAL or PROPRIETARY appears in the recommendation
- A UE5-code-port is proposed as the integration path
- Determinism impact is unstated for any recommended technique (R4)
- An architecture requiring motion data is recommended without naming a licence-cleared source (R5)
- The deliverable is a ranked survey rather than a decision
- Implementation was started (Section 4 says stop)

=== END PROMPT ===

---

## Notes for the owner

**The engine correction is the whole point.** Your draft's deliverable 5 asked for code to
integrate into Unreal Engine 5. This repo is Rust, Bevy 0.15, rapier3d 0.22. A session run
against the original would have spent its budget assembling a UE5 shopping list — and Epic's
sample code is EULA-licensed, so it could not be used even in an Unreal project of your own.

**The licence trap is real and specific.** LaFAN1 is CC BY-NC-ND 4.0 — NonCommercial *and*
NoDerivatives. It is the data behind most of the papers your draft would have surfaced as top
recommendations. The papers are free to read and the techniques are free to implement; the data
is not free to ship. Your draft said "prioritize permissively licensed" but had no step that
would ever have caught this.

**Determinism may decide the answer for you.** You already run `enhanced-determinism` and have a
passing replay suite — that is an unusual and valuable position, and it constrains the choice in
a way no generic prompt would know to ask about.

**Crowd cost is the axis nobody writes down.** Motion matching and neural controllers are
evaluated in papers on one character. You run battles. The honest answer may be two
architectures with an LOD handoff, and the prompt now forces that to be considered rather than
discovered late.

**There is a real Rust path.** `bevy_animation_graph` gives state machines as graph nodes, 
two-bone IK, and partial ragdolls where some bones simulate and others stay animation-driven — 
which is exactly the shape of an active-ragdoll hit reaction. `bevy_motion_matching` exists too. 
Both need their Bevy-version compatibility checked before anything else; Bevy breaks API across 
minor versions, and a crate pinned to 0.12 is a rewrite, not an integration.
