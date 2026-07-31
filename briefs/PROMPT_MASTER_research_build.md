# MASTER PROMPT — Research and Build
### Eight topics: FP dynamics · aiming · traversal · character creation · weapon systems · grenade physics · map design · powered armour

**This file replaces `PROMPT_brief_X_research.md` and `PROMPT_mech_rebuild.md` as the single
thing you paste into Claude Code.** Copy everything between the BEGIN and END markers.
Nothing else needs to be pasted — the prompt reads the specification briefs from disk.

---

=== BEGIN PROMPT ===

# MASTER BRIEF — Research and Build

You are working in this repository. Before anything else:

1. Read `CLAUDE.md`.
2. Read `projects/john_kingdom_game/briefs/README.md` and the briefs it indexes. At minimum
   read `BRIEF_VIII_master.md`, `BRIEF_VIII_B_addendum.md` and
   `BRIEF_IX_castle_grenade_customization.md`. These define the operating contract, the
   20-segment body, the elastic load model, the 26-piece segment-mapped armour, castle map
   tiers, and grenade blast tables. **Do not re-derive any of it. Extend it.**
3. Read `projects/john_kingdom_game/DESIGN_MAP.md` to see what is actually built versus specified.

Work on branch `claude/master-research` — create it from the current default branch if it does
not exist.

This brief has two halves. **Half 1 (Tasks 1–8) is research you perform yourself** with web
search, page fetching, paper reading and video transcripts. **Half 2 (Tasks 9–12) is
implementation.** Do not start Half 2 until the evidence quotas in Half 1 are met.

---

## SECTION 0 — OPERATING CONTRACT

These rules override your defaults for this session.

**R1 — Visible or it didn't happen.** A feature is not done when it compiles or the test
passes. It is done when it appears in a capture taken from the build you actually launched.

**R2 — No claim without a source row.** Every design number you write must trace to a numbered
row in that topic's `SOURCES.md`. Tag it inline: `recoil recovery 0.18 s [S-07]`. An untagged
number is a task failure.

**R3 — Never invent a source.** If a page 403s, is paywalled, has no transcript, or the proxy
blocks it, record the row with status `UNREACHABLE`, `PAYWALLED` or `NO-TRANSCRIPT` and move
on. It does not count toward quota. **Writing a quote you did not read is the worst possible
outcome of this session — worse than an unmet quota.** If you only saw a search-result snippet,
the status is `SNIPPET-ONLY`, not `READ`.

**R4 — Numbers carry units and conditions.** `0.18` is noise. `recoil vertical recovery to 90%
of rest: 0.18 s, at 60 fps, ADS, assault rifle [S-07]` is data.

**R5 — Contradictions are recorded, then resolved.** Log both values in `CONTRADICTIONS.md`,
choose one, justify in one sentence. Never average silently.

**R6 — SIM vs COSMETIC declared per system.** Every system states which layer it lives in, at
the top of the file. SIM affects hit registration, damage, movement or state. COSMETIC affects
only what the eye sees. A cosmetic system touching SIM state is a bug.

**R7 — Tunables are data.** No magic numbers in source. Everything lands in `config/*.ron`,
matching the format already used in this repo. The report lists every tunable and its path.

**R8 — Player intent wins.** Any procedural motion — sway, bob, recoil, drag, flight
stabilization — yields to direct player input within one frame.

**R9 — Research is committed.** `research/` is a real directory. Ledgers, notes, extracted
numbers and screenshots go there and get committed. Nothing lives only in your context.

**R10 — Report what you skipped.** Finish every unblocked part, then state plainly what you
left and why. Never silently narrow scope.

**R11 — Determinism is preserved.** This project's simulation is deterministic and seeded (see
`jk_core`). Anything you add to the SIM layer must not break replay determinism. Physics for
grenades and traversal must be fixed-timestep and seed-driven, never frame-rate dependent.

---

## SECTION 1 — RESEARCH PROTOCOL

### 1.1 Source tiers

| Tier | What counts | Quota role |
|---|---|---|
| **P — Primary** | Peer-reviewed paper, GDC/SIGGRAPH talk, engine source, official engine docs, studio engineering blog, university thesis | ≥3 per topic, required |
| **V — Video with timestamps** | Conference talk, dev deep-dive, frame-analysis breakdown — transcript quoted with timestamps | ≥3 per topic, required |
| **S — Secondary** | Strong technical blog, detailed teardown, open-source project README plus its code | fills remainder |
| **X — Excluded** | Marketing copy, uncited wikis, forum opinion with no measurement, SEO listicles, AI-generated content farms | logged, never counted |

**Quota: 12 counted sources per topic**, of which ≥3 tier P and ≥3 tier V.

Be ruthless about tier X. Searches on controller settings and dodge mechanics return mountains
of affiliate-SEO content that restates opinion as fact. Those are `EXCLUDED`. Chase the
underlying talk, paper, or engine documentation instead.

### 1.2 Files per topic

```
research/<topic-slug>/
  SOURCES.md          # the ledger
  NOTES.md            # extracted mechanisms, grouped by sub-system
  NUMBERS.md          # every quantitative value: units, conditions, source id
  CONTRADICTIONS.md   # disagreements, and which we chose
  clips/              # frames/screenshots captured while researching
```

**`SOURCES.md` row format — use exactly this:**

```
| ID | Tier | Type | Title | Author / Studio | Year | URL | Accessed | Status | What it gave us |
```

Status ∈ `READ` · `SKIMMED` · `SNIPPET-ONLY` · `UNREACHABLE` · `PAYWALLED` · `NO-TRANSCRIPT` · `EXCLUDED`.
Only `READ` counts toward quota.

**`NUMBERS.md` row format:**

```
| ID | Value | Unit | What it measures | Conditions (fps, platform, weapon, stance) | Source |
```

### 1.3 The extraction rule

Every source marked `READ` must yield at least one of:
- a **number** (duration, angle, distance, curve exponent, budget), or
- a **named mechanism** (a technique with a name and a described implementation), or
- a **failure mode** (something tried that did not work, and why).

Yield none of those three and it is `SKIMMED`, not counted. This is the filter that stops the
ledger filling with vague sources.

### 1.4 Anti-pattern harvesting

Collect the *named failures* practitioners have names for. You already inherit: "the mannequin
spin", "the wall stop", "the ice skater", "the switch flip", "the floating gun". Add every
named failure you find to `research/ANTI_PATTERNS.md` with a source. These become grep targets
in the test suite.

### 1.5 Seeded ledger — verified 2026-07-31, extend it

These were checked before this brief was written. Copy them into the relevant `SOURCES.md` with
the status shown, then **verify each yourself** and upgrade or downgrade the status based on
what you actually read. They are a starting point, not a substitute for your own search.

| Seed | Topic | Tier | Source | Status as checked | What it gave |
|---|---|---|---|---|---|
| A | aiming | P/V | Nick Weihs, *Techniques for Building Aim Assist in Console Shooters*, GDC 2013 — `archive.org/details/GDC2013Weihs`, also GDC Vault 1017942 | SNIPPET-ONLY | Names the three assist systems — **magnetism, centering, friction** — plus camera acceleration and deadzoning, from Resistance 3. This is the canonical talk; watch it and timestamp it. |
| B | traversal | P/V | *Vault, Slide, Mantle: Building Brink's SMART System*, GDC Vault 1015930 | SNIPPET-ONLY | Precomputes traversal opportunities by analysing world geometry offline, avoiding designer-placed hint volumes and cutting runtime cost. Directly relevant to our castle geometry. |
| C | traversal | P | *Procedural Parkour and Traversal Animation Techniques*, MSc thesis, Bournemouth NCCA — `nccastaff.bournemouth.ac.uk/jmacey/MastersProject/MSc24/02/ProceduralParkourandTraversalAnimationTechniques.pdf` | SNIPPET-ONLY | Control Rig + procedural IK + motion warping for adaptive parkour. Full PDF, readable. |
| D | grenade | P | Marcel de Carpentier, *Analytical Ballistic Trajectories with Approximately Linear Drag*, Int. J. Computer Games Technology, 2014 — `decarpentier.nl/ballistic-trajectories` | **READ** | Approximates drag as linear to get closed-form trajectories instead of stepped simulation. Solves for initial velocity given: time-to-target, an intermediate waypoint (obstacle clearance), arc height, launch or impact slope, exact launch speed, or minimum energy. Includes C++ snippets and an open-source Unity demo. Valid in constant-gravity frames at 10–1000 m game scales. Omits Coriolis and Earth curvature. **This is the strongest single find for grenade aiming — start here.** |
| E | maps | P | *Blockout metrics*, The Level Design Book — `book.leveldesignbook.com/process/blockout/metrics` | **READ** | Hard numbers, cross-engine. Player bounding box: Unity 1.0 × 1.8 m, eye 1.5–1.7 m; Unreal 60 × 176 cm, eye 152 cm; Quake/Source 32 × 72 in, eye 64 in. Min hallway: Unity 2.0 m / Unreal 150 cm / Source 64 in. Door: Unity 1.25 × 2.5 m / Unreal 110 × 220 cm. Stairs: 15 × 25 cm (Unreal), recommend 30–35° slope, landings every 12–16 steps. TF2 ranges: close ≤256 u, medium ≤1024 u, max safe drop 256 u. |
| F | maps | P/V | Andrew Yoder, *The Holy Grail of Multiplayer Level Design: Maps for Casual and Competitive Play*, GDC Level Design Workshop — GDC Vault 1025183 | SNIPPET-ONLY | Hi-Rez/Paladins process; greybox maps pushed to a public test queue for external data-driven iteration. |
| G | grenade | S | Sam Reitich, *Projectile Prediction: Part 1* — `sreitich.github.io/projectile-prediction-1/` | SNIPPET-ONLY | Modern write-up of prediction implementation; verify depth before counting. |
| H | traversal | S | Celia Wagar, *How iFrames Augment Dodge Rolls*, CritPoints | SNIPPET-ONLY | Design-analysis of i-frame placement within roll phases. Wagar is a serious analyst; likely counts once read. |

**Explicitly not seeded:** aim response curves (linear / exponential / dynamic reverse-S) and
controller deadzone values. Searches return almost entirely affiliate-SEO content quoting each
other. Treat every one of those as tier X and find the primary basis — engine documentation,
input-system source, or a talk — before recording any number.

---

## SECTION 2 — TASK 0: AUDIT

1. Log the git hash and branch at startup.
2. Report what exists, as a table:

```
| System | Files | Coded? | Tested? | Visible in launched build? | Root cause if not visible |
```

Cover at minimum: camera and viewmodel, aiming and ADS, dodge/roll/jump, any climb or vault,
character or loadout code, weapon state machine, reload logic, grenade code, map/level data,
console or command system, runtime asset loading.

3. Launch the build. Capture five before-clips into `research/before/`:
   **(a)** walk, sprint, look around, first person;
   **(b)** aim down sights and fire;
   **(c)** every traversal move that currently exists — jump, dodge, anything else;
   **(d)** throw a grenade;
   **(e)** the current map, one lap showing every elevation change.

**Gate:** all five clips on disk and committed before proceeding.

---

## SECTION 3 — RESEARCH TASKS

Eight topics. Each has a slug, a sub-system breakdown, search queries, and a deliverable.
Quota per topic: 12 counted, ≥3 P, ≥3 V.

---

### TASK 1 — First-person dynamics · slug `fp-dynamics`

| # | Sub-system | What research must answer with numbers |
|---|---|---|
| 1 | Camera | FOV defaults, ADS transition curve and duration, eye height, head-bob amplitude and frequency, why shipped games reduce or kill bob, motion-sickness thresholds |
| 2 | Viewmodel | Weapon position offsets, viewmodel FOV as a value separate from world FOV, sway magnitude and lag constants, why sway is rotation-led not translation-led |
| 3 | Movement coupling | How velocity, acceleration and turn rate feed camera and viewmodel; landing impulse; strafe roll magnitude; start/stop acceleration curves |
| 4 | Recoil | Pattern versus random split, first-shot recoil, recovery curve shape and duration, **visual recoil and aim recoil as separate channels**, deterministic spray patterns |
| 5 | Hands / IK | Two-handed grip constraints, left-hand IK to the weapon attach point, finger poses, off-hand retarget during reload, when IK is disabled |
| 6 | Feedback | Hit markers, damage direction, screen-shake budgets, audio-visual sync windows, frame budget for each |

**Queries:** `GDC first person camera design talk` · `viewmodel FOV separate from world FOV` ·
`first person weapon sway spring damper implementation` · `recoil pattern deterministic spray
analysis` · `first person hands IK two handed weapon grip` · `head bob motion sickness first
person study` · `procedural weapon animation additive layers GDC` · `FPS camera shake trauma
based Squirrel Eiserloh` · `Doom Eternal animation GDC weapon` · `Titanfall 2 movement GDC talk`

**Deliverable:** the four ledger files plus `SYNTHESIS.md` giving, per sub-system, our chosen
values with source ids and one line of rationale each. Plus a decision table mapping 1:1 to a
config file.

---

### TASK 2 — Proper aiming · slug `aiming`

This is its own topic, not a sub-section of FP dynamics. Aiming is where the game either feels
fair or feels broken.

| # | Sub-system | What research must answer |
|---|---|---|
| 1 | Input mapping | Response curves — linear, exponential, dynamic reverse-S — with the actual curve maths, not just names. Deadzone types (radial, axial, scaled-radial) and why scaled-radial is usually correct. Sensitivity scaling between hip and ADS, and what "monitor-distance-matched" means. |
| 2 | Aim assist | The three systems from seed [A]: **magnetism** (turn-rate modification toward target), **centering** (pull toward target when the stick is neutral), **friction** (turn-rate reduction across a target). Bullet magnetism as a distinct, more controversial fourth. Assist cone angles, ramp curves, and when assist must disable. |
| 3 | Fairness | How assist is tuned so controller and mouse coexist without either dominating. Find where studios drew that line and what broke. |
| 4 | Weapon accuracy | First-shot accuracy, bloom versus fixed pattern, moving versus stationary penalty, crouch bonus, the accuracy-versus-recoil split. |
| 5 | Hitscan vs projectile | When each is used, at what ranges, and how projectile speed changes the leading skill. Bullet drop and travel time, if any. |
| 6 | Flinch and aim punch | How incoming damage perturbs aim, magnitude, recovery, and why some games removed it entirely. |
| 7 | Scopes | Scope-in duration, magnification effect on sensitivity, scope glint, the hold-breath mechanic, and the viewmodel-hide rule from `BRIEF_VIII_master.md`. |

**Queries:** `GDC aim assist console shooter magnetism friction centering` · `scaled radial
deadzone implementation gamepad` · `aim assist controller vs mouse balance design` ·
`first shot accuracy bloom recoil design shooter` · `hitscan vs projectile design tradeoff
netcode` · `aim punch flinch removed design decision` · `sniper scope sensitivity scaling
monitor distance matching`

**Warning per §1.5:** most aiming search results are affiliate-SEO. Get to primaries.

**Deliverable:** ledger files plus `AIM_SPEC.md` — the full input chain from raw stick or mouse
delta to final view rotation, as an ordered pipeline with every stage named, every tunable
listed, and the assist stages marked with the exact condition that disables them.

---

### TASK 3 — Traversal: dodge, jump, flips, climbing, obstacles · slug `traversal`

| # | Sub-system | What research must answer |
|---|---|---|
| 1 | Dodge / roll | Phase structure — startup, i-frame window, recovery. Where i-frames sit inside the roll and why the middle. Duration of each phase in frames and seconds. What the roll cancels and what cancels it. Directional versus omnidirectional. Cooldown and stamina cost. |
| 2 | Jump | Jump arc as apex height and time-to-apex, not "jump force". **Coyote time** (grace period to jump after leaving a ledge) and **input buffering** (jump pressed before landing still fires) with real values in frames. Variable jump height by hold duration. Air control fraction. |
| 3 | Flips / acrobatics | When a flip is animation versus simulation. How rotation is driven without losing collision authority. Landing recovery and how a failed landing reads. |
| 4 | Mantle / vault / climb | **Ledge detection** — the trace pattern, how many traces, what shapes. The height bands that select the move (step-over, vault, mantle, climb). Seed [B]'s precomputation approach versus runtime tracing: cost, robustness, and whether it suits our castle geometry. **Motion warping** to align the animation to the actual ledge instead of snapping. |
| 5 | Momentum | What is preserved through each traversal and what is killed. This is the difference between a movement system that feels alive and one that feels like a series of stops. Ties to "the wall stop" anti-pattern. |
| 6 | Chaining | The state machine that lets sprint → vault → land → roll flow without a neutral frame between them. |

**Queries:** `Vault Slide Mantle Brink SMART system GDC` · `procedural parkour traversal
animation thesis` · `coyote time input buffer jump feel values frames` · `ledge detection trace
mantle vault implementation` · `motion warping traversal animation alignment` · `dodge roll
i-frame startup recovery frame data` · `momentum preservation movement system shooter GDC` ·
`Assassin's Creed parkour system GDC talk` · `Titanfall 2 wall run implementation`

**Deliverable:** ledger files plus `TRAVERSAL_FSM.md` — every traversal move as a state with
entry condition, phase table (startup / active / recovery in frames), i-frame window if any,
what it cancels, what cancels it, momentum in and momentum out, and the ledge-height band that
selects it. Plus `LEDGE_DETECTION.md` specifying the trace pattern concretely.

**Bind to existing work:** the 20-segment body and elastic load model from
`BRIEF_VIII_B_addendum.md` already define the load-then-release rule (release 2–3× faster than
load) and proximal-to-distal kinetic chain. Every traversal move must obey both. A dodge with
no load phase is "the switch flip" and fails review.

---

### TASK 4 — Layered character creation · slug `character-creation`

The player enters the game, builds their physical look, then picks a mode, then commits.

| Layer | Name | Decides | Research question |
|---|---|---|---|
| **L0** | Identity | Name, pronouns, callsign | Where stored, and does it ever gate gameplay? (correct answer: no) |
| **L1** | Physique | Height, build, proportions, skin, face, hair | How do shipped games stop a body-shape slider from breaking hitboxes, animation retargeting, and armour fit? |
| **L2** | Kit | Armour pieces, weapons, weight budget | How is a stat-bearing layer made legible *before* commit — what does the preview owe the player? |
| **L3** | Cosmetic | Colours, decals, wear, paint | How is team readability preserved when players choose colours? |
| **L4** | Mode & role | Game mode, class, spawn | What can still change after match start, and what freezes? |

**The commit-boundary question is the core of this task.** Answer explicitly: which layers
freeze at match start and which stay live between deaths; whether L1 physique is normalized for
competitive integrity or preserved for expression, with shipped examples of each; how the flow
resumes if a player quits at L2; and the time budget for first entry versus returning entry,
with numbers on drop-off in long creation flows.

**Queries:** `character creator UX design GDC talk` · `body shape slider hitbox normalization
competitive shooter` · `modular character armour attachment system architecture` · `morph
target blend shape runtime performance cost` · `avatar customization player identification
study` · `Proteus effect avatar research paper` · `character creator drop off onboarding time
to first match` · `team color readability enemy silhouette customization`

**Academic requirement:** find at least **two peer-reviewed papers** on avatar customization,
identification, or attachment. Extract what was measured, sample size, and effect size. This is
the tier-P backbone of this topic.

**Deliverable:** ledger files plus `FLOW_SPEC.md` (five layers as a state machine — states,
transitions, back-navigation, what each writes to disk, commit boundary, resume rule) and a
data-model sketch tagging every field L0–L4 and SIM/COSMETIC per R6. Must bind to the existing
26-piece armour and 20-segment body without forking them, and must not break hip-shoulder
separation.

---

### TASK 5 — Weapon systems: reload, render, console · slug `weapon-systems`

Three distinct halves. Keep them separate in `NOTES.md`.

**Part A — Reload as a state machine.** Target states:

```
IDLE → INITIATE → MAG_RELEASE → MAG_DROP → MAG_INSERT → SEAT → CHARGE/BOLT → RECOVER → IDLE
```

Answer with numbers: where the **interrupt boundary** sits (which states sprint/fire/melee can
cancel, and what is kept versus lost); how a chambered round changes the path and duration
(tactical versus empty, and the time delta); at which frame **ammo actually commits** — start,
mag-seat, or end — and why that matters for reload-cancel exploits; whether the dropped
magazine persists as a world object, for how long, whether it collides, whether it is pooled;
per-round shotgun/bolt reload loops and the shot-out rule; how additive reload-speed bonuses
interact with animation event timing without desyncing the ammo commit.

**Part B — Weapon rendering.** Material layering (base metal / coating / wear / grime) and its
texture-sample cost; channel-packing conventions; edge-wear masks driven by curvature or AO,
procedural versus baked; decal and pattern projection for a player-supplied image — UV region,
projection, or atlas — and the resolution needed to read at ADS without blowing the memory
budget; viewmodel-specific rendering with a separate depth range so the weapon never clips into
world geometry, and what that extra pass costs; texture budget per weapon and total VRAM for a
loadout; muzzle flash, shell ejection and heat haze frame budgets.

**Part C — In-game console and runtime image import.** Console core: command registration
without central coupling, cvar registry (read / set / reset / list / save / load), autocomplete,
history, scrollback, output levels, and whether it ships or is dev-gated.

Runtime asset import needs a threat model, and the refusals are the feature:
- **Allowed formats** — an explicit whitelist (PNG, JPEG, TGA), stated in the spec
- **Size ceilings** — max dimensions and max file size, enforced *before* decode
- **Guarded decode** — a malformed image fails the import, never crashes the game
- **Confined paths** — an explicit import directory; console input never supplies arbitrary
  paths; normalize and confine, no traversal
- **Hot reload** — file watcher versus explicit command; GPU resource swap without a frame hitch
- **Scope** — imported images are local-only and COSMETIC per R6 for the first pass, and the
  console says so when used
- **Refusals print a reason** — never a silent failure

**Queries:** `weapon reload state machine architecture` · `reload cancel ammo commit timing
exploit` · `shotgun per shell reload state machine` · `weapon skin material layering channel
packing PBR` · `edge wear curvature mask procedural weapon texture` · `viewmodel separate depth
pass clipping` · `in game developer console cvar architecture Quake` · `hot reload textures
runtime GPU resource swap` · `runtime image loading validation untrusted input`

**Deliverable:** ledger files plus `RELOAD_FSM.md` (state table with an `ammo commit` column),
`RENDER_SPEC.md` (material stack, map list with channel packing, resolution budget, decal
method) and `CONSOLE_SPEC.md` (command list, cvar design, import pipeline, and the explicit
refusal list).

---

### TASK 6 — Grenade aiming and physics · slug `grenade-physics`

Extends `BRIEF_IX` §B, which already fixes fuse timings, blast falloff, bounce coefficients and
environmental modifiers. **Do not re-derive those.** This task is about *aiming* a grenade and
simulating it correctly.

| # | Sub-system | What research must answer |
|---|---|---|
| 1 | Trajectory solve | Seed [D] is the anchor. Implement its linear-drag analytical model rather than stepping a simulation. Which of its solve modes do we expose — minimum energy for the default throw, time-to-target for a lobbed throw, waypoint clearance for throwing over a battlement? |
| 2 | Arc preview | How the predicted arc is drawn, at what update rate, and how the preview stays honest — the preview must be the *same* solver the throw uses, never a parallel approximation that drifts. Where the preview ends: first impact, or after N bounces? |
| 3 | Bounce prediction | Predicting bounces requires collision queries along the arc. Cost per frame, and how many bounces are worth predicting. Ties to the bank-shot play that `BRIEF_IX` §B's 0.40 stone coefficient enables. |
| 4 | Throw mechanics | Overhand versus underhand versus roll, and the release-angle difference. Throw strength by hold duration. **Cooking** — holding a live grenade to shorten effective fuse — and its risk model. |
| 5 | Determinism | Per R11: the throw must be reproducible from the same seed and inputs. Fixed timestep integration, no frame-rate dependence, no float drift across replays. Research how deterministic physics is achieved and what breaks it. |
| 6 | Kinetic chain | Per `BRIEF_VIII_B_addendum.md`, the throw is a proximal-to-distal chain with a load phase. The grenade release must be driven by the hand segment's velocity at release, not a constant. A running throw should carry the body's momentum. |

**Queries:** `analytical ballistic trajectory linear drag games` · `trajectory prediction line
renderer bounce collision cost` · `grenade cooking design risk reward` · `deterministic physics
fixed timestep replay float` · `projectile prediction implementation` · `throw arc preview UI
design shooter`

**Deliverable:** ledger files plus `GRENADE_SOLVER.md` — the chosen solve mode with its
equations, the preview contract (same solver, stated update rate, stated bounce depth), the
throw input model, and the determinism guarantees with the specific integration scheme named.

---

### TASK 7 — Map and level design · slug `map-design`

Extends `BRIEF_IX` §A, which already specifies three elevation tiers, the 40 m sightline rule,
the crossfire anchor, material layering, detail density by tier, and objective placement. **Do
not re-derive those.** This task grounds them in real metrics and a real process.

| # | Sub-system | What research must answer |
|---|---|---|
| 1 | Metrics | Seed [E] gives cross-engine player and architecture dimensions. Derive **our** metric set: player bounding box, eye height, step height, max safe drop, min corridor, door size, stair rise/run, cover heights for crouch and stand. Every castle asset is built to this set. |
| 2 | Blockout process | The greybox-first methodology and why art must preserve the blockout, not reshape it. Seed [F] on data-driven iteration from public greybox playtests. |
| 3 | Verticality | How multi-level maps are kept navigable — how a player knows where they are and where the level above connects. Landmark and silhouette navigation. Ties to our Tier 1/2/3 structure. |
| 4 | Flow and sightlines | Validating the 40 m rule automatically. Is there a tool-side check — raycast sweeps across the navmesh producing a sightline heatmap? Research how studios visualize map balance. |
| 5 | Traversal-aware layout | Task 3's ledge bands feed directly here. Every wall the player can mantle must be authored at a height inside a band. Research how metrics and traversal are kept in sync so a designer cannot accidentally build an unclimbable 1.9 m wall next to a climbable 1.8 m one. |
| 6 | Cover and grenade geometry | `BRIEF_IX` §A already requires merlon gaps too narrow for grenades and a well mouth wide enough. Derive the actual dimensions from Task 6's trajectory solver. |

**Queries:** `level design blockout metrics player dimensions` · `multiplayer level design
casual competitive GDC workshop` · `verticality multi level map navigation readability` ·
`sightline analysis tool level design heatmap` · `modular kit design metrics grid snapping` ·
`Valorant map blockout art pass preserve layout` · `castle architecture military design
rampart wall walk dimensions`

**Deliverable:** ledger files plus `MAP_METRICS.md` — our single authoritative metric table,
every number with a source id and a note on which engine convention it was converted from — and
`BLOCKOUT_PROCESS.md` describing how a castle map goes from layout to playable, with the
validation checks at each stage.

---

### TASK 8 — Powered armour · slug `powered-armour` · **RESEARCH ONLY, DO NOT BUILD**

This is the future Iron-Man-class suit. Research it thoroughly now so that when it is built it
is built once and built right. **Task 9 onward does not implement this.** Produce the spec,
stop there.

The bar: a powered suit that *deserves* to be in this game. That means it is not a stat buff
with a new mesh. It is a different verb — the player who suits up moves through the castle in a
way no infantry player can, and the castle's three tiers become one continuous space.

| # | Sub-system | What research must answer |
|---|---|---|
| 1 | Flight model | Thrust-to-weight, the hover equilibrium point, and what the stick actually controls — attitude, or velocity? Real jet-suit rigs vector thrust through arm position to steer, which is the mechanically interesting choice and maps to our 20-segment arms. Research both real thrust-vectoring rigs and shipped game flight controllers. |
| 2 | Hover versus flight | The two distinct modes and the transition between them. Hover is a stability problem (hold a position against gravity and drift); flight is a momentum problem. Find how shipped games separate them. |
| 3 | Resource model | Fuel, heat, or charge — pick one and justify it. Heat that builds under thrust and dissipates in free-fall creates a rhythm; a fuel bar creates a countdown. Research which produces better play and what the dive-to-cool loop does to pacing. |
| 4 | Transitions | Suit-up and suit-down sequences: duration, whether they are interruptible, whether the player is vulnerable during them. The take-off from standing and the landing — a landing must obey the elastic load model, with a real impact and recovery, not a snap to idle. |
| 5 | Weapons and HUD | Arm-mounted weapons that conflict with thrust vectoring — firing while steering with the same limb is the central design tension, and it is a feature. Suit HUD as a diegetic layer distinct from the infantry HUD in `BRIEF_VIII_master.md` §3. |
| 6 | Counterplay | A flying player must be killable. Research how games balance air units against ground: tracking difficulty, dedicated anti-air, altitude ceilings, forced descent. Without this the suit ruins the map. |
| 7 | Scale and silhouette | Where the suit sits between the 1.0× soldier and the mech (1.7× per `BRIEF_VIII_B_addendum.md` §A). It must read instantly as neither. Squint test at 30 m and 60 m. |
| 8 | Armour integration | Whether the suit reuses the 26-piece segment-mapped armour system with powered variants, or is its own rig. Reusing it is strongly preferred — state the cost either way. |

**Queries:** `jet suit thrust vectoring arm control flight stability` · `game flight controller
hover model implementation` · `jetpack flight game design heat fuel resource` · `air unit
versus ground balance design anti air` · `powered exoskeleton game design traversal verb` ·
`Anthem javelin flight design postmortem` · `mech versus power armour silhouette scale design`

**Deliverable:** ledger files plus `POWERED_ARMOUR_SPEC.md` — flight model with its control
mapping, the resource model with a defended choice, transition sequences with durations, the
counterplay answer, the scale decision with a squint-test plan, and an explicit statement of
what it reuses from existing systems versus what it needs new. End the file with a
**build-readiness checklist**: what must exist before this is implemented.

---

## SECTION 4 — TASK 9: SYNTHESIS

Do not implement yet.

1. Write `research/SYNTHESIS.md` — one document, eight sections, containing only *decisions*.
   Every line carries source ids.
2. Resolve cross-topic conflicts explicitly. Known collision points:
   - **Physique sliders (T4 L1) versus FP viewmodel (T1):** a taller character moves the
     camera; the viewmodel must not drift. State the rule.
   - **Physique sliders versus the 26-piece armour:** armour must fit every physique. Either
     armour scales or physique range is bounded. Pick one.
   - **Traversal ledge bands (T3) versus map metrics (T7):** these must be the same numbers,
     in one file, referenced by both. If they disagree, the map is unplayable.
   - **Dodge i-frames (T3) versus grenade blast (T6):** can a roll i-frame through an
     explosion? Answer explicitly; it is a balance decision, not an oversight.
   - **Aim assist (T2) versus flying targets (T8):** assist tuned for ground targets behaves
     badly against a hovering one. Note it for when the suit is built.
   - **Reload FSM (T5) versus sprint and traversal (T3):** which wins, what is preserved on
     cancel, and can you reload mid-vault?
   - **Console image import (T5C) versus squint-test readability (T4):** a player-supplied
     image must not break the 30 m silhouette rule.
   - **Recoil channels (T1) versus aim authority (T2):** visual recoil must never steal aim
     authority, per R8.
   - **Grenade determinism (T6) versus traversal physics (T3):** both are SIM, both must be
     fixed-timestep, per R11.
3. Produce `research/TUNABLES.md`: every value that will exist in config — name, unit, default,
   valid range, target file. This is the contract Task 10 implements against.

**Gate:** every conflict above has a written resolution. `TUNABLES.md` is complete.

---

## SECTION 5 — TASK 10: IMPLEMENT

In this order. Match the repo's existing language, structure and idiom — read neighbouring
files before writing new ones. Commit after each item. **Task 8 is excluded from
implementation.**

**10.1 — Config first.** Create the files named in `TUNABLES.md`. No behaviour yet. Every
tunable present with its default. Nothing hardcoded later.

**10.2 — Map metrics.** `MAP_METRICS.md` becomes real config, because everything else is built
against it. Ledge bands and player dimensions are defined once, here, and referenced by
traversal and level geometry both.

**10.3 — Aiming pipeline.** The full chain from raw input to view rotation, every stage
separable and individually disableable for testing. Assist stages are off by default and must
declare their disable conditions.

**10.4 — First-person dynamics.** The six sub-systems. Recoil has **separate visual and aim
channels**. Sway is rotation-led. Input wins within one frame.

**10.5 — Traversal.** The FSM from `TRAVERSAL_FSM.md`, including ledge detection, motion
warping to the actual ledge, i-frame windows, and momentum carry. Every move has a load phase.

**10.6 — Grenade solver.** The analytical solver from seed [D]. The preview uses the same
solver as the throw — this is a hard requirement, not an optimization. Deterministic under
fixed timestep.

**10.7 — Reload FSM.** Exactly as tabled, including the ammo-commit frame and cancel rules. The
magazine is a pooled world object with a config lifetime.

**10.8 — Character creation flow.** L0–L4 with back-navigation, resume, and the commit
boundary. Binds to the existing 20-segment body and 26-piece armour.

**10.9 — Weapon render.** Material stack and wear. One weapon taken end-to-end with a defined
decal region.

**10.10 — Console and import.** Console core, cvar registry, then import. **Implement the
validation gate and refusal list before the success path.** Import is local-only and COSMETIC,
and the console states that when used.

---

## SECTION 6 — TASK 11: TESTS

Each test must **fail on the pre-change code** and **pass after**. A test that passes before
your change is testing nothing — rewrite it. Report both results.

| Test | Asserts | Pass condition |
|---|---|---|
| `research_quota` | each topic's ledger has ≥12 counted rows, ≥3 P, ≥3 V | all eight pass |
| `no_snippet_only_counted` | no `SNIPPET-ONLY` row counted toward quota | zero |
| `no_orphan_numbers` | every numeric value in spec files has an `[S-nn]` tag | zero untagged |
| `tunables_not_hardcoded` | greps source for literals from TUNABLES.md | zero hits outside config |
| `input_authority` | stick input during max recoil | camera follows input within 1 frame |
| `recoil_channels` | visual recoil applied, aim recoil zeroed | crosshair world-ray unchanged |
| `sway_is_rotational` | pure viewmodel translation with no rotation | assertion fails — sway must rotate |
| `deadzone_shape` | stick swept around its full circle at fixed radius | output magnitude constant (scaled-radial, not axial) |
| `assist_disables` | every declared disable condition | assist contribution exactly zero in each |
| `assist_never_aims` | assist active, player input zero, target moving | crosshair does not acquire on its own beyond declared centering |
| `ledge_band_coverage` | every authored wall height in the map | falls inside a declared band |
| `traversal_momentum` | sprint → vault → land | exit speed ≥ declared fraction of entry |
| `coyote_time` | jump pressed N frames after leaving ledge | fires for N ≤ configured, not for N+1 |
| `input_buffer` | jump pressed before landing | fires on land within buffer window |
| `iframe_window` | damage applied each frame across a full roll | invulnerable only in the declared window |
| `no_instant_stop` | every traversal exit | velocity never steps to zero in one frame |
| `load_before_release` | every power move including dodge and throw | load phase present, release 2–3× faster |
| `grenade_determinism` | same seed, same inputs, 1000 throws | identical impact points bit-for-bit |
| `preview_matches_throw` | 200 random throws | preview endpoint equals actual impact within tolerance |
| `grenade_fixed_step` | simulate at 30/60/144 fps | identical trajectories |
| `reload_states` | empty reload | all 8 states visited in order |
| `reload_ammo_commit` | cancel one frame before commit | ammo unchanged |
| `reload_cancel_sprint` | sprint during MAG_INSERT | cancels per spec, no ammo gain |
| `mag_pooled` | 200 reloads | allocations bounded by pool size |
| `creation_resume` | quit at L2, re-enter | returns to L2, L0/L1 intact |
| `commit_boundary` | edit L1 after match start | rejected |
| `physique_armour_fit` | physique extremes × all 26 pieces | zero interpenetration |
| `physique_viewmodel` | min and max height | viewmodel offset identical |
| `squint_test` | any imported decal, silhouette at 30 m | still reads as the same character |
| `import_rejects_bad_format` | `.exe` renamed `.png` | refused with reason, no crash |
| `import_rejects_oversize` | above the dimension ceiling | refused before decode |
| `import_rejects_traversal` | path containing `../` | refused, confined to import dir |
| `import_malformed` | truncated PNG | refused, no crash, no leak |
| `console_cvar_roundtrip` | set → save → reload → get | value survives |
| `sim_determinism_preserved` | existing replay suite | still passes after all changes |
| `anti_patterns` | greps for named failures | none present in new code |

---

## SECTION 7 — TASK 12: CAPTURES

From the build you actually launched. Not the editor, not a test harness.

1. **Before/after FP feel** — same 15 s route, two clips side by side.
2. **Aiming** — hip to ADS transition, a tracking shot, and a flick, at 30 m and 60 m.
3. **Assist proof** — the same engagement with every assist stage off, then on. The difference
   must be visible and must not look like the game is playing itself.
4. **Traversal reel** — one unbroken take: sprint → vault a low wall → mantle a ledge → climb
   to Tier 2 → drop → roll → recover → sprint. No neutral frame between moves.
5. **Dodge i-frames** — roll through an attack, with a frame counter overlaid showing the
   window.
6. **Grenade** — arc preview drawn, thrown, and the impact landing exactly where the preview
   ended. Then a bank shot around a corner. Then the same throw twice from the same seed,
   showing identical results.
7. **Reload, three paths** — tactical, empty, and cancelled mid-reload, slow enough to see the
   mag drop.
8. **Character creation** — L0 through L4 unbroken, with one back-navigation and one
   resume-after-quit.
9. **Physique extremes** — shortest and tallest in full 26-piece armour, at max hip-shoulder
   separation, no clipping.
10. **Weapon render** — one weapon at ADS showing the material stack, then with an imported
    image applied.
11. **Console session** — open, list cvars, set one and see it take effect live, import a valid
    image, then attempt all four refused imports and show each refusal message.
12. **Map lap** — one continuous run touching all three elevation tiers, showing every
    traversal affordance used at least once.

---

## SECTION 8 — TASK 13: REPORT

One report, in this order:

1. **Task 0 audit table, re-run.** Every row that changed, changed.
2. **Research ledger summary** — per topic: counted sources, P count, V count, unreachable
   count, extracted values. Link the files.
3. **Contradictions found and how each was resolved.**
4. **Test results** — exact commands, before result and after result for each. Any test that
   passed before your change is called out as invalid and rewritten.
5. **Every tunable** — name, value, unit, path. Full list, not a sample.
6. **The captures**, in order.
7. **The powered-armour spec** — summarized in one paragraph, with its build-readiness
   checklist. State clearly that it was not implemented, by instruction.
8. **Feel questions, answered in prose** — not "yes":
   - Does the weapon feel attached to the hands, or does it float?
   - Does aiming feel like *you* aiming, or like the game helping?
   - Does the traversal chain flow, or is it a series of separate moves with stops between?
   - Can you tell, without the HUD, which reload path just played?
   - Does the grenade land where you believed it would?
   - Does the character you built at L1 feel like the character you play as?
   - Does the map read at a glance — do you know which tier you are on?
9. **What you did not do, and why.** Blocked items, unmet quotas, unreachable sources. Plainly.

Commit everything and push with `git push -u origin claude/master-research`. Do not open a pull
request unless asked.

---

## SECTION 9 — FAILURE CONDITIONS

This session has failed, regardless of how much code exists, if any of these is true:

- A quote or source in any `SOURCES.md` was not actually read (R3).
- A `SNIPPET-ONLY` row was counted toward quota.
- A design number has no source id (R2).
- A test passed on the pre-change code and was reported as proof.
- A capture came from anywhere but the launched build (R1).
- A tunable is hardcoded in source (R7).
- The grenade preview uses a different solver than the throw.
- Ledge bands in traversal disagree with ledge bands in map metrics.
- The image import path accepts a traversal, an oversize file, or a malformed decode.
- Existing replay determinism broke (R11).
- Powered armour was implemented (Task 8 is research only).
- The report claims completion for something not done (R10).

=== END PROMPT ===

---

## Notes for the owner — not part of the prompt

**Run it in three passes.** Eight topics at 12 sources each is ~96 sources; one session will not
hold that. Because R9 forces everything onto disk and into git, each pass can start with "read
`research/` and continue":

- **Pass 1** — Sections 0–1, Task 0, then Tasks 1–3 (FP dynamics, aiming, traversal)
- **Pass 2** — Tasks 4–8 (creation, weapons, grenade, maps, powered armour)
- **Pass 3** — Tasks 9–13 (synthesis, implement, test, capture, report)

**The seeded ledger is the honest part.** Section 1.5 lists eight sources checked on
2026-07-31. Two were actually fetched and read end to end — the de Carpentier ballistics paper
and the Level Design Book metrics page — and those two carry real extracted numbers. The other
six were confirmed to exist and to be about the right subject, but only from search results, so
they are marked `SNIPPET-ONLY` and do not count until the session reads them. That distinction
is the whole point of R3, and modelling it in the seed matters more than padding the list.

**The strongest single find is seed [D].** De Carpentier's analytical linear-drag model, 2014,
with C++ and an open-source Unity demo, solves exactly the grenade problem — including throwing
over an obstacle to a point behind it, which is what "throw over the battlement" needs. Most
games step a simulation every frame to draw the preview arc, then throw with different code and
wonder why the grenade misses the line. The analytical solver removes that entire class of bug,
which is why the prompt makes "preview uses the same solver as the throw" a failure condition.

**Powered armour is deliberately research-only.** You said you want it *in the future*. Building
a flight model before aiming and traversal are solid would mean rebuilding it after — a flying
player breaks aim assist tuning, map sightlines, and the whole three-tier structure at once. The
prompt researches it fully and ends with a build-readiness checklist, so when you do build it,
it is built once.

**On the missing files.** You were right that earlier messages described documents whose
contents never reached you. All six briefs are now committed under
`projects/john_kingdom_game/briefs/` and this file replaces the two older prompts — so there is
one thing to paste, and the rest is read from the repo.
