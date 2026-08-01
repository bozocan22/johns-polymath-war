# PROMPT — Continuous R&D Cycle

Optimized from the "Ultimate 4D Research Framework." The ambition is right and mostly
preserved. Three things in the original would have actively damaged the project, and one of
them is proven — not predicted — by evidence from this repository's own research log.

Paste everything between the BEGIN and END markers.

---

## Section-by-section review of the original

### §Mission — "act as a permanent AAA R&D department… continue expanding forever"

**Kept:** the discipline of ending every piece of work by asking what is missing. That is
genuinely valuable and most teams don't do it.

**Cut:** "never stop at the requested feature," "treat the codebase as permanently unfinished,"
"continue expanding forever." These are anti-completion instructions. Combined with a mandate
that *every* report enumerate missing systems, they guarantee that scope grows faster than work
finishes and nothing ever ships. This project already has a working game with 131 passing tests;
the framework as written would bury it under a permanently expanding survey.

**Replaced with:** the discovery instinct is preserved but *redirected into a persistent file*.
Findings append to `research/BACKLOG.md`, ranked. Each cycle pulls exactly one item, finishes
it, and adds what it discovered. Expansion is unbounded across cycles; each individual cycle
terminates.

**Cut:** the 13-role interdisciplinary-team roleplay. It costs tokens and changes nothing about
output quality. Rigour comes from verification gates, not from claiming to be a Principal
Engine Architect.

### §Minimum Deliverables — "25+ peer-reviewed papers, 25+ repositories, 50+ systems, per report"

**This is the flaw that matters most, and it is not a matter of opinion.**

On 2026-08-01 this repository's aiming ledger recorded the following, and it is committed
history you can read at `research/aiming/SOURCES.md`:

> A `WebFetch` extraction of the Vicencio-Moriera CHI 2014 aim-assist paper returned "~87% hit
> rate," "2.5 degrees visual angle," "1.8× standard reticle diameter," "0.7 normalized units,"
> and "20 participants." **None of those numbers appear anywhere in that paper.** It also
> invented a fifth technique name ("Reticle Assistance") that does not exist in the study. The
> fabrication was caught only because the raw PDF was read directly and checked against the
> summary. It looked entirely plausible.

That was **one paper**, under no quota pressure at all. Getting it right required a search, a
failed 403, a hunt for the author's own copy, a full-text read, and a correction pass.

A mandate of 25 verified papers *per report* cannot be satisfied honestly in a session. What it
actually produces is 25 confident citations, most unread, some invented — and the invented ones
are indistinguishable from the real ones without exactly the verification work the quota leaves
no room for. **A breadth quota is a fabrication generator.**

**Replaced with a depth floor:** at least one tier-P source **read end to end** per cycle, with
its licence and its numbers verified. Three sources genuinely read beat twenty-five cited.
Uncounted sources are logged with an honest status and do not inflate the total.

### §Gameplay/Graphics/Physics/Networking research lists — ~200 topics

**Kept, in full, as a backlog** — the lists are a genuinely good brainstorm and several entries
are excellent. They become ranked backlog rows, not per-report obligations.

**Gated by a feasibility check,** because a large fraction of them currently have nothing to
attach to. Verified against the codebase today:

| Requested | Reality in this repo | Verdict |
|---|---|---|
| Nanite, Lumen, virtual shadow maps, Niagara, path tracing, DLSS/FSR/XeSS, hair rendering | UE5/vendor-proprietary. This is Bevy 0.15. Also: **all 21 `asset_server.load` calls are `.wav`. There are zero image loads. Every material is a procedural colour.** | **Blocked.** Researching virtualized geometry for a game with no textures is not a prioritization error, it is a category error. |
| Rollback netcode, replication graph, lag compensation, dedicated servers | Zero networking dependencies in `Cargo.toml`. Build is local-only. | **Deferred.** Named in backlog with the blocker; the Reitich prediction mechanisms are already recorded for when it exists. |
| Swimming, diving, rope traversal, ladders, muscle simulation, fluids, soft bodies | No water volumes, no ropes, no muscle layer. | **Deferred.** |
| Mech entry sequence (approach → auth → cockpit → harness → startup → HUD boot) | **Mech walker exists** with committal enter/exit and plate damage. | **Ready now.** Best item in the original document. |
| Infantry vs. giant mech (climbing, joint attacks, cable cutting, weak points) | Mech exists; spear exists; segment-mapped armour is specified. | **Ready now.** |
| Grenade surface interaction (restitution per material, rolling, spin, angular momentum) | `grenade_tick` exists with per-surface bounce and a bit-identical determinism test. | **Ready now, cheap.** Direct extension. |
| Combat depth (parry, directional attack, stagger, armour penetration, weak points) | Melee exists (axe, spear, shield). | **Ready, medium cost.** |
| AI squad coordination, suppression, bounding overwatch, morale | Bots exist; `jk_wall` already has morale/fear/rout. | **Ready, medium cost.** |
| Motion matching, pose search, full-body IK, motion warping | Covered by `PROMPT_motion_system_research.md`, already retargeted to Rust/Bevy with the LaFAN1 licence trap documented. | **Already queued.** Don't duplicate. |

### §4D Framework (Discover → Design → Develop → Deliver)

**Kept largely intact.** It is a sound structure. Each phase now gets an explicit exit
condition so a cycle can actually end.

### §Deliver — 17 mandatory report sections

**Cut to what a person will read.** A 17-section report per feature is written and never read.
Kept: executive summary, what changed, evidence, trade-offs rejected, roadmap delta, what's
missing. Dropped as per-report obligations: component diagram, data flow, code review, security
assessment — these are real artefacts, but they belong to the systems that need them, produced
on demand, not stamped out every cycle.

### §Continuous Codebase Review — 21 categories every report

**Kept, converted to a rotation.** Four categories per cycle, rotating, so each gets real
attention roughly every five cycles. Twenty-one shallow checks find less than four deep ones.
This repo's own history supports that: the four adversarial audit waves that found real bugs —
`the loyal ghost`, `the split brain`, `the shrinking-list index` — were deep passes, not
checklists.

---

=== BEGIN PROMPT ===

# R&D Cycle

## Read first

1. `CLAUDE.md`
2. `projects/john_kingdom_game/briefs/README.md` and the briefs it indexes
3. `projects/john_kingdom_game/engine/crates/jk_tdm/research/` — the existing ledgers,
   `ANTI_PATTERNS.md`, and `TASK0_AUDIT.md`
4. `research/BACKLOG.md` if it exists; create it from Appendix A on the first cycle

**Target stack: Rust, Bevy 0.15, rapier3d 0.22 with `enhanced-determinism`.** Unreal, Unity,
and vendor-proprietary technology (Nanite, Lumen, Niagara, DLSS) are **reference-only** — read
the technique, never propose porting the code. Unreal sample content is EULA-licensed and
cannot enter a Rust project.

## What a cycle is

**One cycle = one system, researched properly, built, proven, and reported.** Not fifty
systems surveyed. The backlog is unbounded and grows forever; each cycle is finite and ends.

Cycles run until the backlog's Critical and High tiers are empty, or until told to stop.

---

## SECTION 0 — CONTRACT

**R1 — Visible or it didn't happen.** Done means it appears in a capture from the build you
actually launched.

**R2 — No claim without a source row.** Every number traces to a row in the topic's
`SOURCES.md`. Tag inline `[S-nn]`.

**R3 — Never invent a source. Verify extractions against primary text.**

This project has already caught one fabrication. A `WebFetch` summary of a real, correctly
identified CHI 2014 paper returned five numbers and a technique name that appear nowhere in it.
It was caught only by reading the actual PDF. The full incident is recorded in
`research/aiming/SOURCES.md` — read it before your first extraction.

Therefore:
- **A tool's summary of a source is not the source.** When an extraction will carry a number
  into a spec, a test, or a config value, verify it against the primary text. If the tool saved
  the file locally, read the file.
- Statuses: `READ` (full text, verified) · `SKIMMED` · `SNIPPET-ONLY` · `UNREACHABLE` ·
  `PAYWALLED` · `NO-TRANSCRIPT` · `EXCLUDED`. **Only `READ` counts.**
- A number you cannot trace to primary text does not get written down. An honest gap beats a
  plausible invention, always.

**R4 — Depth floor, no breadth quota.** Per cycle: **≥1 tier-P source read end to end**, plus
however many others genuinely inform the work. There is no minimum citation count and padding
the ledger is a failure, not a success. If one paper answers the question, one paper is the
right number.

**R5 — Licence before recommendation.** No repo, dataset, or model is recommended until its
licence is read verbatim from its LICENSE file and classified: `PERMISSIVE` / `WEAK-COPYLEFT` /
`STRONG-COPYLEFT` / `NON-COMMERCIAL` / `PROPRIETARY` / `UNCLEAR`. Nothing non-commercial or
proprietary may be shipped. (Known trap: Ubisoft LaFAN1 — the data behind most motion-matching
papers — is CC BY-NC-ND 4.0. Papers readable; data unshippable.)

**R6 — Determinism is a hard constraint.** `rapier3d` runs `enhanced-determinism` and the
replay suite passes. Classify every technique **SIM-safe** or **COSMETIC-only** before
recommending it. Breaking replay determinism fails the cycle.

**R7 — SIM vs COSMETIC declared per system**, at the top of the file.

**R8 — Tunables are data.** No magic numbers in source; config only.

**R9 — Player intent wins.** Procedural motion yields to direct input within one frame. Note
that CHI 2014 (`research/aiming/SOURCES.md` S-02) is empirical support for this: assist
techniques that move the crosshair *during* the player's aiming motion measurably underperform
ones that correct only *after* the trigger. Correct the outcome, never the player's hand.

**R10 — Feasibility gate.** Before researching anything, ask: **does the codebase have
something to attach this to?** If not, it goes to the backlog with its blocker named, and the
cycle picks the next item. Do not research virtualized geometry for a renderer with no textures.

**R11 — Report what you skipped**, plainly.

---

## SECTION 1 — DISCOVER

**Exit condition:** the cycle's one system is chosen, and ≥1 tier-P source is `READ`.

1. **Pick the system.** Highest-ranked backlog item that passes R10. State why it, and not the
   items above it, if you skipped any.
2. **Research it properly.** Venues: SIGGRAPH / TOG / Eurographics / SCA for graphics and
   animation; CHI / CHI PLAY for anything a player *feels*; arXiv cs.GR and cs.RO; ICRA/IROS for
   legged and robotic motion; GDC and studio engineering blogs for production technique.
3. **Extract per R3.** Every `READ` source must yield a number with units, a named mechanism, or
   a documented failure mode. If it yields none of the three, it is `SKIMMED`.
4. **Harvest anti-patterns.** Named failures go to `ANTI_PATTERNS.md` with a source and become
   grep targets.

---

## SECTION 2 — DESIGN

**Exit condition:** one architecture chosen, alternatives rejected on stated grounds.

For the chosen system, cover only what applies — an honest "not applicable here" is better than
a padded section:

- Purpose, and what the player actually feels
- Data structures and algorithm
- SIM/COSMETIC classification (R7) and determinism impact (R6)
- Cost: per-frame CPU, memory, and **cost at crowd scale** — `jk_wall` runs hundreds of bodies;
  a per-character cost fine for one hero may be impossible for a battle
- Interaction with what already exists: the 20-segment body, the elastic load model, the kinetic
  chain, the existing FSMs
- Implementation difficulty and long-term maintenance

Where multiple approaches exist, compare and **choose one**, naming the axis that killed each
alternative. "Not the best fit" is not a rejection.

---

## SECTION 3 — DEVELOP

**Exit condition:** it builds, tests pass, determinism suite still green.

- Config first (R8), then behaviour.
- Match the repo's existing idiom — read neighbouring files before writing new ones.
- Prefer extending existing systems to replacing them; replacement needs the stronger argument.
- Every new test must **fail on the pre-change code**. A test that passes before your change is
  testing nothing — rewrite it and say so.
- Run the full suite including the replay-determinism tests.

---

## SECTION 4 — DELIVER

**Exit condition:** report posted, backlog updated, work committed.

Report contains, in this order — and nothing else:

1. **What was built**, in one paragraph a person can act on
2. **Evidence** — the source(s) `READ`, what each gave, and any fabrication or unreachable
   source encountered
3. **Tests** — command, before result, after result
4. **Capture** from the launched build (R1)
5. **Rejected alternatives** — one line each, with the axis that killed it
6. **Backlog delta** — items added, items reranked, items unblocked by this work
7. **What you did not do, and why** (R11)

Then append to `research/BACKLOG.md` and answer, in writing:

> **What should exist that we have not discussed?**

New items get a tier (Critical / High / Medium / Low / Blocked), a one-line reason, and — if
Blocked — the specific thing that unblocks them.

---

## SECTION 5 — ROTATING CODEBASE REVIEW

Each cycle, review **four** categories, rotating through this list so each is examined roughly
every five cycles. Four deep beats twenty-one shallow — this repo's real bugs (`the loyal
ghost`, `the split brain`, `the shrinking-list index`) came from deep adversarial passes.

```
1  duplicated logic / missing abstractions      6  automated-test gaps
2  performance: CPU hot paths                   7  documentation claiming things that don't exist
3  performance: allocation and cache            8  error handling and panic surfaces
4  scalability at crowd counts                  9  input validation and file-boundary safety
5  determinism risks                           10  dead code and orphaned systems
```

Record findings as backlog items. Fix anything trivial in-cycle; queue the rest.

---

## SECTION 6 — FAILURE CONDITIONS

- A source marked `READ` was not read in full, or a number was carried from a tool summary
  without checking primary text (R3)
- A ledger was padded to hit a count
- A recommended artefact has an unverified licence (R5)
- Determinism broke (R6)
- A UE5/Unity code port was proposed as the integration path
- A test passed before the change and was reported as proof
- The cycle researched something with nothing to attach to, instead of backlogging it (R10)
- The report claims completion for something not done (R11)

---

## APPENDIX A — Initial backlog

Seed `research/BACKLOG.md` with this on the first cycle. Ranking is by *what the codebase can
accept today*, verified against the code, not by how exciting the idea is.

### Critical — ready now, high value, direct extension of working systems

| # | System | Why now | Attaches to |
|---|---|---|---|
| 1 | **Mech entry sequence** — approach, identification, cockpit open, climb-in, harness, power-up, servo sync, gyro calibration, weapon diagnostics, HUD boot, camera transition | The single best idea in the source document. Turns entering the mech from a state toggle into an *event*. Pacing and audiovisual escalation are the whole deliverable. | Mech walker exists with committal enter/exit |
| 2 | **Infantry vs. giant mech** — climbing the hull, joint strikes, hydraulic failure, cable cutting, sensor destruction, weak-point exposure, coordinated squad attacks | Makes the mech an *encounter* rather than a health bar. This is the game's most distinctive unclaimed design space. | Mech + spear + segment-mapped armour spec |
| 3 | **Grenade surface interaction** — per-material restitution and friction, rolling, spin, angular momentum; concrete / metal / wood / mud / sand / snow / ice / grass / water | Cheapest high-value item here. `grenade_tick` already has per-surface bounce and a bit-identical determinism test to extend. Brief IX-B already fixes the coefficients. | `grenade_tick`, existing R11 tests |

### High — ready, medium cost

| # | System | Why | Attaches to |
|---|---|---|---|
| 4 | Melee depth: parry, deflection, directional attack, stagger, armour penetration, weak points | Melee exists but is shallow; historical-combat and biomechanics literature is rich and real | axe/spear/shield |
| 5 | AI squad coordination: flanking, suppression, bounding overwatch, retreat | Bots exist; `jk_wall` already models morale, fear and rout — reuse rather than reinvent | bot AI, `jk_wall` morale |
| 6 | Mech operation feel: weight, mechanical inertia, cockpit vibration, heat, internal damage, emergency shutdown/eject | Extends item 1; heat-and-cool creates a rhythm a fuel bar cannot | mech sim |
| 7 | Traversal: climb, vault, mantle, ledge bands | Already scoped in the master prompt; ledge bands must match map metrics or maps become unplayable | none yet — needs map metrics first |

### Medium

| # | System | Note |
|---|---|---|
| 8 | Motion architecture decision | Already fully specced in `PROMPT_motion_system_research.md`. **Do not duplicate — run that prompt.** |
| 9 | Character creation layers (L0–L4) | Blocked in practice: no class system and only 5 whole-body armour presets, so a physique/hitbox commit rule has nothing to attach to |
| 10 | Destruction and environmental interaction | rapier supports some; needs a design reason before a technique |
| 11 | Injury, fatigue, equipment weight, dynamic centre of gravity | The armour-weight formula exists but is unwired |

### Blocked — named, with the specific unblocker

| # | System | Blocker |
|---|---|---|
| 12 | Weapon material stack, wear maps, decals, player image import, in-game console | **Zero texture pipeline.** All 21 `asset_server.load` calls are `.wav`; every material is a procedural colour. Unblocker: any image loading at all. |
| 13 | All advanced rendering — Nanite/Lumen equivalents, GPU-driven pipelines, virtual shadow maps, path tracing, upscalers, hair, volumetric clouds | Wrong engine *and* nothing to render into. Unblocker: item 12, then a Bevy-native rendering decision. Vendor tech is reference-only regardless. |
| 14 | Networking — rollback, prediction, reconciliation, lag compensation, replication | Zero networking dependencies; build is local-only. Reitich's prediction mechanisms are already recorded in `research/grenade/SOURCES.md` for when this unblocks. |
| 15 | Swimming, diving, rope traversal, ladders, muscle simulation, soft bodies, fluids | No water volumes, no ropes, no muscle layer. Each needs its own content prerequisite. |

**Ranking rule for future cycles:** an item moves up when its blocker clears, not when it
becomes interesting. Blocked items are never researched "in preparation" — the research goes
stale before the blocker clears.

=== END PROMPT ===

---

## Notes for the owner

**The quota was the real problem.** Everything else in your framework was ambition, which is
fine. But "25+ peer-reviewed papers per report" is the one instruction that would have made the
output actively worse than doing nothing, because it manufactures exactly the failure this
project has already caught once. Reading the CHI 2014 aim-assist paper properly — one paper —
took a search, a 403, a hunt for the author's own copy, a full-text read, and a correction pass
after the fetch tool invented five numbers. Multiply that by 25, per report, and what comes back
is a bibliography, not research.

Depth floor instead of breadth quota: **one paper read end to end, verified against primary
text.** That one paper changed a real design position. Twenty-five cited papers would not have.

**"Continue expanding forever" was the second problem.** It is a good instinct pointed at the
wrong target. Unbounded discovery is right; unbounded *reports* are not. Moving it into a ranked
`BACKLOG.md` keeps the instinct and lets individual cycles finish. Your R&D department stays
permanent; the reports stop being infinite.

**The best content in your document survived and got promoted.** The mech entry sequence —
approach, authenticate, climb, harness, power-up, servo sync, gyro calibration, HUD boot — is
the strongest single idea in it, and the mech walker already exists to attach it to. It is
Critical #1. Infantry-versus-giant-mech is #2, because it is the most distinctive design space
this game has not claimed. Grenade surface materials is #3 because it is nearly free: the
per-surface bounce and the determinism test are already there.

**One thing to know going in:** roughly a third of your research list currently has nothing to
attach to. All 21 asset loads in the game are `.wav` files — there is not one image in the
renderer. So the graphics section isn't merely lower priority than you'd think; researching
virtualized geometry and hair rendering for it would produce notes that go stale before anything
can use them. Those items are in the backlog with the exact unblocker named, which is the honest
place for them.
