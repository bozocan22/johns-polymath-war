# BRIEF X — RESEARCH & BUILD PROMPT
### Three topics: First-Person Dynamics · Layered Character Creation · Reload / Weapon-Render / Runtime Asset Console

**How to use this file:** paste everything from `=== BEGIN PROMPT ===` to `=== END PROMPT ===`
into Claude Code in VS Code. Everything above that line is for you, not for the model.

---

## What changed vs. your original prompt (section by section)

| # | Your original | Problem | Optimization applied |
|---|---|---|---|
| 1 | "go do a very detailed research about 3 different topics" | No stopping condition — the model decides when "detailed" is done | Hard **evidence quotas** per topic (16 sources, ≥4 primary, ≥4 video-with-timestamps). Research ends when the ledger is full, not when the model feels finished. |
| 2 | "find examples and learn the best code there is in the internet" | "Best code" is unverifiable and invites invented citations | Replaced with **extraction schema**: every source row must yield a *number with a unit* or a *named mechanism*. Prose-only sources are logged but don't count toward quota. |
| 3 | "do deep learning" | Not an instruction the model can execute | Replaced with **contradiction pass**: when two sources disagree on a number, both are recorded and the model must pick one and justify. That's the actual deep part. |
| 4 | "best first person dynamics" | Too broad to converge | Split into **6 named sub-systems** (camera, viewmodel, movement coupling, recoil, hands/IK, feedback) each with its own quota and its own test. |
| 5 | "character customization system setup in layers when you enter the game, physical look then game mode" | Real design, but stated as a wish | Formalized as a **5-layer pipeline with an explicit commit boundary** — which layers are locked at match start, which stay live. That boundary is the whole design. |
| 6 | "very detailed reloading dynamics and gun pixel dynamics" | Two different systems fused into one phrase | Separated: **reload as a state machine** (7 states, interrupt rules, magazine object persistence) vs. **weapon render** (materials, wear maps, decal projection, resolution budget). |
| 7 | "a console that allows me inside the game or images I put" | Feature is clear, threat model isn't | Specified as **runtime asset console** with an explicit sandbox: allowed formats, size ceilings, validation, where files land, and what is refused. Import without validation is the bug, not the feature. |
| 8 | "make system do research topics where this prompt will be optimized and also telling Claude Code to do the research itself watching YouTube videos reading research papers" | The model has no durable memory of what it read | Added **`research/` as a committed artifact tree** — SOURCES.md, NOTES.md, and quotes with timestamps land on disk and get committed. The research survives the session. |
| 9 | (absent) | No proof anything shipped | Every task ends in a **capture from the launched build** and a **failing-before / passing-after test**. Same rule as Briefs VII–IX: visible or it didn't happen. |
| 10 | (absent) | Model may silently skip unreachable sources or invent them | Added **UNREACHABLE / PAYWALLED / NO-TRANSCRIPT** status codes. A source that couldn't be read is recorded as such and does not count. Fabricating a quote is a task failure. |

---

=== BEGIN PROMPT ===

# BRIEF X — Research and Build: FP Dynamics, Layered Character Creation, Weapon Systems

You are working in this repository. Read `CLAUDE.md` first, then
`projects/john_kingdom_game/briefs/README.md` and the briefs it indexes — at minimum
`BRIEF_VIII_master.md` and `BRIEF_VIII_B_addendum.md`, which define the operating contract, the
20-segment body, the elastic load model, and the 26-piece segment-mapped armour this brief
builds on. Do not re-derive any of that; extend it.

Work on branch `claude/brief-x-research` (create it from the current default branch if it does
not exist).

This brief has two halves. **Half 1 (Tasks 1–3) is research you perform yourself** using
web search, article fetching, paper reading, and video transcripts. **Half 2 (Tasks 4–7)
is implementation** driven by what you found. Do not start Half 2 until the evidence
quotas in Half 1 are met.

---

## SECTION 0 — OPERATING CONTRACT

These rules override your defaults for this session.

**R1 — Visible or it didn't happen.**
A feature is not done when the code compiles or the test passes. It is done when it appears
in a capture taken from the build you actually launched. Every task ends with a capture.

**R2 — No claim without a source row.**
Every design number you write into a spec must trace to a numbered row in the topic's
`SOURCES.md`. Write the row number inline, e.g. `recoil recovery 0.18 s [S-07]`.
A number with no source row is a task failure.

**R3 — Never invent a source.**
If a page 403s, is paywalled, has no transcript, or the proxy blocks it, record the row with
status `UNREACHABLE`, `PAYWALLED`, or `NO-TRANSCRIPT` and move on. It does not count toward
quota. Writing a plausible-sounding quote you did not read is the single worst outcome of
this session — worse than an unmet quota.

**R4 — Numbers carry units and context.**
`0.18` is noise. `recoil vertical recovery to 90% of rest: 0.18 s, at 60 fps, ADS, assault
rifle [S-07]` is data. Record the conditions the number was measured under.

**R5 — Contradictions are recorded, then resolved.**
When two sources disagree, log both in `CONTRADICTIONS.md`, then choose one for our spec and
write one sentence of justification. Do not average them silently.

**R6 — SIM vs COSMETIC declared per system.**
Every system you build declares which layer it lives in. SIM affects hit registration,
damage, movement, or state. COSMETIC affects only what the eye sees. A cosmetic system that
touches SIM state is a bug. Say which layer, in the file, at the top.

**R7 — Tunables are data, not constants.**
No magic number in a `.rs`/`.cpp`/`.cs` file. Everything lands in `config/*.ron` (or the
project's existing config format — match what is already there). The report lists every
tunable, its value, and its file path.

**R8 — Player intent wins.**
Any procedural motion (sway, bob, recoil, drag) yields to direct player input within one
frame. If the player moves the stick and the camera argues, the camera is wrong.

**R9 — Research is committed.**
`research/` is a real directory in the repo. Notes, source ledgers, extracted numbers, and
screenshots go there and get committed. Nothing lives only in your context.

**R10 — Report what you skipped.**
If a task is partly blocked, finish every unblocked part and state plainly what you left and
why. Do not silently narrow scope.

---

## SECTION 1 — RESEARCH PROTOCOL (read once, applies to Tasks 1–3)

### 1.1 Tools and how to use them

- **Web search** for discovery. Use the exact queries listed per topic, then follow the best
  3–5 results from each.
- **Web fetch** for reading. Fetch the actual page. Do not summarize from the search snippet.
- **YouTube:** fetch the video page/transcript. If a transcript is available, quote it with a
  timestamp (`[12:41]`). If no transcript exists, mark `NO-TRANSCRIPT` and do not paraphrase
  from the title. GDC talks, studio postmortems, and engine-internals videos are the target;
  reaction videos and top-10 listicles are not.
- **Papers:** arXiv, ACM DL abstracts, SIGGRAPH course notes, IEEE VR, and university theses.
  Game-industry white papers count. Read at least the abstract, method, and results sections.
- If the proxy blocks a fetch, see `/root/.ccr/README.md` and
  `curl -sS "$HTTPS_PROXY/__agentproxy/status"`. Never disable TLS verification.

### 1.2 Source tiers

| Tier | What counts | Quota role |
|---|---|---|
| **P — Primary** | Peer-reviewed paper, GDC/SIGGRAPH talk, engine source code, official engine docs, studio engineering blog | ≥4 per topic, required |
| **V — Video with timestamps** | Conference talk, developer deep-dive, frame-analysis breakdown — transcript quoted with timestamps | ≥4 per topic, required |
| **S — Secondary** | Well-regarded technical blog, detailed teardown, open-source project README + code | fills remainder |
| **X — Excluded** | Marketing copy, wikis with no citation, forum opinion with no measurement, AI-generated listicles | logged but never counted |

**Quota per topic: 16 counted sources**, of which ≥4 are tier P and ≥4 are tier V.

### 1.3 Files each topic must produce

```
research/<topic-slug>/
  SOURCES.md          # the ledger — table format below
  NOTES.md            # extracted mechanisms, grouped by sub-system
  NUMBERS.md          # every quantitative value, with units + conditions + source id
  CONTRADICTIONS.md   # disagreements between sources, and which we chose
  clips/              # screenshots/frames you captured while researching (optional)
```

**`SOURCES.md` row format — use exactly this:**

```
| ID | Tier | Type | Title | Author / Studio | Year | URL | Accessed | Status | What it gave us |
|----|------|------|-------|-----------------|------|-----|----------|--------|-----------------|
| S-01 | P | paper | ... | ... | 2019 | https://... | 2026-07-31 | READ | 3 numbers on ... |
```

`Status` is one of: `READ`, `SKIMMED`, `UNREACHABLE`, `PAYWALLED`, `NO-TRANSCRIPT`, `EXCLUDED`.

**`NUMBERS.md` row format:**

```
| ID | Value | Unit | What it measures | Conditions (fps, platform, weapon, stance) | Source |
```

### 1.4 The extraction rule

For every source you mark `READ`, you must extract at least one of:
- a **number** (a duration, an angle, a distance, a curve exponent, a budget), or
- a **named mechanism** (a technique with a name and a described implementation), or
- a **failure mode** (a thing that was tried and did not work, and why).

If a source yields none of those three, mark it `SKIMMED` and it does not count.
This is the filter that stops the ledger from filling with vague sources.

### 1.5 Anti-pattern harvesting

While researching, collect the *named failures* — the ones practitioners have names for.
You already have some from earlier briefs: "the mannequin spin", "the wall stop", "the ice
skater", "the switch flip", "the floating gun". Add every named failure you find to
`research/ANTI_PATTERNS.md` with a source. These become grep targets in the test suite later.

---

## SECTION 2 — TASK 0: AUDIT WHAT EXISTS

Before any research, establish the baseline.

1. Log the git hash and branch at startup.
2. Search the codebase for anything already implementing the three topics. Report as a table:

```
| System | Files | Coded? | Tested? | Visible in launched build? | Root cause if not visible |
```

Cover at minimum: camera/viewmodel code, any customization or loadout code, any weapon
state machine, any reload logic, any console/command system, any runtime asset loading.

3. Launch the build. Capture 3 clips: **(a)** walking + looking around in first person,
**(b)** firing and reloading a weapon, **(c)** whatever character/loadout UI currently
exists (if none exists, capture the spawn flow and say "no customization UI").

These are the *before* clips. Every later capture is compared against them.

**Gate:** do not proceed until the three before-clips exist on disk under
`research/before/` and are committed.

---

## SECTION 3 — TASK 1: RESEARCH FIRST-PERSON DYNAMICS

**Topic slug:** `fp-dynamics`. **Quota:** 16 counted sources, ≥4 P, ≥4 V.

### 3.1 Six sub-systems — research each, do not blur them

| # | Sub-system | The question research must answer with numbers |
|---|---|---|
| 1 | **Camera** | FOV defaults and ADS transitions, vertical FOV vs horizontal, camera height relative to eye, head-bob amplitude/frequency and why most shipped games reduce or kill it, motion-sickness thresholds |
| 2 | **Viewmodel** | Weapon position offsets, viewmodel FOV as a separate value from world FOV, sway magnitude and lag time constants, why sway is rotation-led not translation-led |
| 3 | **Movement coupling** | How velocity, acceleration and turn rate feed the camera and viewmodel; landing impulse; strafe roll (and its typical magnitude, usually small); acceleration curves for start/stop |
| 4 | **Recoil** | Pattern vs random split, first-shot recoil, recovery curve shape and duration, visual recoil vs aim recoil as separate channels, deterministic spray patterns |
| 5 | **Hands / IK** | Two-handed grip constraints, left-hand IK to weapon attach point, finger poses, how the off hand is retargeted during reload, when IK is disabled |
| 6 | **Feedback** | Hit markers, damage direction, screen shake budgets, audio-visual sync windows, what the frame budget for each is |

### 3.2 Search queries — run these, then follow the good results

```
GDC first person camera design talk
viewmodel FOV separate from world FOV implementation
first person weapon sway spring damper implementation
recoil pattern deterministic spray CS GO Valorant analysis
first person hands IK two handed weapon grip Unreal
head bob motion sickness first person study
procedural weapon animation additive layers GDC
first person shooter game feel juice GDC talk
FPS camera shake trauma based Squirrel Eiserloh
site:arxiv.org first person view motion sickness field of view
weapon feel Insomniac / Respawn / id Software engineering blog
Doom Eternal animation GDC weapon
Titanfall 2 movement GDC talk
"game feel" Steve Swink weapon polish
```

### 3.3 Video targets — find and transcribe these classes of talk

- GDC Animation Bootcamp sessions on first-person weapon animation
- GDC Design/Programming track talks on camera and game feel
- Studio deep-dives on movement systems (movement-heavy shooters especially)
- Frame-by-frame analysis channels that measure in frames, not adjectives
- Engine-internals talks covering additive animation blending

For each: record channel, title, upload year, URL, and **at least three timestamped quotes**
that carry a number or a named mechanism.

### 3.4 What Task 1 delivers

- `research/fp-dynamics/{SOURCES,NOTES,NUMBERS,CONTRADICTIONS}.md` meeting quota
- `research/fp-dynamics/SYNTHESIS.md`: for each of the six sub-systems, our chosen values
  with source ids, and a one-line rationale each
- A **decision table** of every value we are adopting, formatted so it maps 1:1 to a config
  file in Task 5

**Gate:** quota met, every counted row has a `What it gave us` cell that is not empty,
`NUMBERS.md` has ≥25 rows.

---

## SECTION 4 — TASK 2: RESEARCH LAYERED CHARACTER CREATION

**Topic slug:** `character-creation`. **Quota:** 16 counted sources, ≥4 P, ≥4 V.

The design you want is a **layered entry flow**: the player enters the game, builds their
physical look, then picks a mode, then commits. Research must tell us where each layer's
commit boundary belongs and why.

### 4.1 The five layers to research

| Layer | Name | What it decides | Research question |
|---|---|---|---|
| **L0** | Identity | Name, pronouns, callsign | Where is this stored, and does it ever gate gameplay? (answer must be: no) |
| **L1** | Physique | Height, build, proportions, skin, face, hair | How do shipped games keep a body-shape slider from breaking hitboxes, animation retargeting, and armour fit? |
| **L2** | Kit | Armour pieces, weapons, weight budget | How is a stat-bearing layer made legible *before* commit — what does the preview owe the player? |
| **L3** | Cosmetic | Colours, decals, wear, paint | How is team readability preserved when players pick their own colours? |
| **L4** | Mode & Role | Game mode, class/role, spawn selection | What can still change after match start, and what is frozen? |

### 4.2 The commit-boundary question — this is the core of the task

Research and answer explicitly:

- Which layers are **frozen at match start** and which stay **live between deaths**?
- What happens to L1 physique if it affects hitbox volume — is it normalized for
  competitive integrity, or preserved for expression? Find how shipped competitive games
  resolve this. Record the trade-off with examples.
- How is the flow made **resumable** — a player who quits at L2 returns where?
- What is the **time budget** for the whole flow on first entry vs. returning entry? Find
  numbers on player drop-off in long creation flows.

### 4.3 Search queries

```
character creator UX design GDC talk
character customization system architecture layered data driven
body shape slider hitbox normalization competitive shooter
morph target blend shape character customizer runtime performance
modular character armour attachment system Unreal Unity architecture
site:arxiv.org avatar customization player identity study
avatar personalization player attachment research paper
character creator drop off funnel onboarding time to first match
team color readability enemy silhouette player customization
Destiny / Warframe / Monster Hunter armour transmog system design
material instance parameter customization runtime cost
character creator save slot preset architecture
```

### 4.4 Academic angle — do not skip this

There is real HCI/games research on avatar customization and player identification (the
"Proteus effect" literature, avatar attachment, identification and retention studies). Find
at least **two peer-reviewed papers** and extract what they measured, the sample size, and
the effect. This is the tier-P backbone of this topic.

### 4.5 What Task 2 delivers

- The four research files at quota
- `research/character-creation/FLOW_SPEC.md`: the five layers as a state machine — states,
  transitions, back-navigation rules, what each layer writes to disk, the commit boundary,
  and the resume rule
- A **data-model sketch**: what a saved character actually is as a struct/record, with every
  field's layer tag (L0–L4) and its SIM/COSMETIC tag per R6
- Integration note: how this binds to the existing **26-piece segment-mapped armour** and
  **20-segment body** from the earlier briefs — the physique layer must not break
  hip-shoulder separation or armour fit

**Gate:** quota met, ≥2 peer-reviewed papers in the ledger with extracted findings,
`FLOW_SPEC.md` names every state and every transition.

---

## SECTION 5 — TASK 3: RESEARCH RELOAD, WEAPON RENDER, AND RUNTIME ASSET CONSOLE

**Topic slug:** `weapon-systems`. **Quota:** 16 counted sources, ≥4 P, ≥4 V.
This topic has three distinct halves — keep them separate in `NOTES.md`.

### 5.1 Part A — Reload as a state machine

Research must produce a **named-state machine**, not a single animation. Target states:

```
IDLE → INITIATE → MAG_RELEASE → MAG_DROP → MAG_INSERT → SEAT → CHARGE/BOLT → RECOVER → IDLE
```

Questions research must answer with numbers:

- Where is the **interrupt boundary** — at which state can sprint/fire/melee cancel the
  reload, and what is kept vs. lost when it cancels?
- **Tactical vs empty reload:** how does a round-in-chamber change the state path and the
  duration? What is the typical time delta between them?
- **Ammo commit timing:** at which frame does the ammo count actually change — on animation
  start, on mag-seat, or on animation end? Find what shipped games do and why it matters for
  reload-cancel exploits.
- **The dropped magazine as a world object:** does it persist, for how long, does it
  collide, is it pooled? Budget for it.
- **Per-round reloads** (shotgun/bolt): the loop state, and the shot-out rule.
- **Reload speed modifiers**: how additive speed bonuses interact with animation event
  timing without desyncing the ammo commit.

Search queries:

```
weapon reload state machine architecture game programming
reload cancel exploit ammo commit timing shooter
animation notify ammo count reload Unreal
tactical reload chambered round game design implementation
shotgun per shell reload state machine loop
magazine physics object pooling shooter performance
GDC weapon systems architecture talk shooter
first person reload animation layers additive GDC
```

### 5.2 Part B — Weapon rendering ("gun pixel dynamics")

Interpretation: this covers everything that determines what the weapon looks like at the
pixel level, and how the player can change it. Research:

- **Material layering:** base metal / coating / wear / grime as stacked material layers;
  how many texture samples that costs; channel-packing conventions (which map goes in which
  channel and why).
- **Wear and edge damage:** curvature/AO-driven edge wear masks, procedural vs baked, how
  wear intensity is exposed as a single tunable.
- **Decal and pattern projection:** how a player-supplied image gets placed on a weapon —
  UV region, projection, or decal atlas. Resolution required so it reads at ADS but does not
  blow the memory budget.
- **Viewmodel-specific rendering:** separate depth range / separate FOV pass so the weapon
  never clips into world geometry; the cost of that extra pass.
- **Texture budget:** target resolution per weapon, per-map, and total VRAM for a full
  loadout. Find shipped numbers.
- **Muzzle flash, shell ejection, heat haze:** frame budgets and typical durations.

Search queries:

```
weapon skin material layering channel packing PBR game
edge wear mask curvature map procedural weapon texture
decal projection player custom image weapon skin implementation
viewmodel separate depth pass clipping first person weapon
texture memory budget per weapon shooter shipped numbers
Substance layered material weapon skin workflow
muzzle flash shell ejection VFX budget frame cost
weapon customization skin system architecture CS GO Valorant technical
```

### 5.3 Part C — In-game console and runtime image import

This is the feature you described: a console, inside the running game, that lets you inject
images (and ideally tweak values) without a rebuild.

Research must produce a design covering:

**Console core**
- Command registration pattern (how systems expose commands without central coupling)
- CVar/tunable registry: read, set, reset, list, save-to-config, load-from-config
- Autocomplete, history, scrollback, output levels
- Whether the console is dev-only or ships, and how it is gated

**Runtime asset import — the part that needs a threat model**
- **Allowed formats:** decide the whitelist (e.g. PNG, JPEG, TGA) and state it.
- **Size ceilings:** max dimensions and max file size, enforced before decode.
- **Validation before use:** decode in a guarded path; a malformed image must fail the
  import, not crash the game. Research image-decode hardening for untrusted input.
- **Where files land:** an explicit import directory, never arbitrary paths from console
  input. No path traversal — normalize and confine.
- **Hot-reload path:** file watcher vs explicit `reload` command; how the GPU resource is
  swapped without a frame hitch.
- **Scope:** does an imported image affect only the local client (COSMETIC per R6) or is it
  networked? For a first pass it must be local-only, and the console must say so.
- **What is refused:** anything outside the whitelist, oversize, or resolving outside the
  import directory. Refusals print a reason.

Search queries:

```
in game developer console architecture command registration cvar
Quake console cvar system design
hot reload textures runtime asset pipeline game engine
runtime image loading validation untrusted input game
imgui debug console game tools implementation
file watcher asset hot reload GPU resource swap
custom sprays player uploaded images game moderation architecture
path traversal validation user supplied file path game
```

### 5.4 What Task 3 delivers

- The four research files at quota, with `NOTES.md` split A/B/C
- `research/weapon-systems/RELOAD_FSM.md`: full state table — state, entry condition, exit
  condition, duration, cancellable (y/n), what happens to ammo, what happens to the mag object
- `research/weapon-systems/RENDER_SPEC.md`: material stack, map list with channel packing,
  resolution budget, decal placement method
- `research/weapon-systems/CONSOLE_SPEC.md`: command list, cvar registry design, import
  pipeline with the validation gate and the refusal list

**Gate:** quota met; the reload FSM table has every state populated including the
`ammo commit` column; the console spec has an explicit refusal list.

---

## SECTION 6 — TASK 4: SYNTHESIS AND CONFLICT RESOLUTION

Do not implement yet.

1. Write `research/SYNTHESIS.md` — one document, three sections, pulling only the *decisions*
   out of the three topics. Every decision line carries its source ids.
2. Resolve cross-topic conflicts explicitly. Known collision points:
   - **Physique sliders (L1) vs. FP viewmodel:** a taller character moves the camera; the
     viewmodel must not drift. State the rule.
   - **Physique sliders vs. the 26-piece armour:** armour must fit every physique. State
     whether armour scales, or physique range is bounded. Pick one.
   - **Reload FSM vs. sprint:** which wins, and what is preserved on cancel.
   - **Console image import vs. squint-test readability:** a player-supplied image must not
     break the 30 m silhouette rule from the character brief. State the constraint.
   - **Recoil channels vs. camera:** visual recoil must not steal aim authority (R8).
3. Produce `research/TUNABLES.md`: every value that will exist in config, its name, unit,
   default, valid range, and which config file it will live in. This is the contract Task 5
   implements against.

**Gate:** every conflict above has a written resolution. `TUNABLES.md` exists and is complete.

---

## SECTION 7 — TASK 5: IMPLEMENT

Implement in this order. Match the repo's existing language, structure, and idiom — read
neighbouring files before writing new ones. Commit after each numbered item.

**5.1 — Config first.** Create/extend the config files named in `TUNABLES.md`. No behaviour
yet. Every tunable present with its default. Per R7, nothing hardcoded later.

**5.2 — First-person dynamics.** Implement the six sub-systems using the Task 1 decision
table. Recoil must have **separate visual and aim channels**. Sway must be rotation-led.
Player input must win within one frame (R8).

**5.3 — Reload FSM.** Implement the state machine exactly as tabled in `RELOAD_FSM.md`,
including the ammo-commit frame and the cancel rules. The magazine is a pooled world object
with a lifetime from config.

**5.4 — Character creation flow.** Implement the L0–L4 state machine from `FLOW_SPEC.md`,
including back-navigation, resume, and the commit boundary. Bind to the existing 20-segment
body and 26-piece armour — do not fork them.

**5.5 — Weapon render.** Material stack and wear as specified. Decal/pattern region defined
on at least one weapon end-to-end.

**5.6 — Console + import.** Console core, cvar registry, then the import pipeline. Implement
the validation gate and the refusal list *before* the success path — the refusals are the
feature. Import is local-only and COSMETIC; the console prints that when used.

---

## SECTION 8 — TASK 6: TESTS

Each test must **fail on the pre-change code** and **pass after**. If a test passes before
your change, it is testing nothing — rewrite it. Report both results.

| Test | Asserts | Pass condition |
|---|---|---|
| `research_quota` | each topic's SOURCES.md has ≥16 counted rows, ≥4 P, ≥4 V | all three pass |
| `no_orphan_numbers` | every numeric value in the spec files has a `[S-nn]` tag | zero untagged |
| `tunables_not_hardcoded` | greps source for the literal values in TUNABLES.md | zero hits outside config |
| `input_authority` | simulated stick input during max recoil | camera follows input within 1 frame |
| `recoil_channels` | visual recoil applied with aim recoil zeroed | crosshair world-ray unchanged |
| `sway_is_rotational` | pure translation of viewmodel with no rotation | assertion fails — sway must rotate |
| `reload_states` | FSM visits every state in order for empty reload | all 8 states hit |
| `reload_ammo_commit` | cancel reload one frame before commit state | ammo unchanged |
| `reload_cancel_sprint` | sprint during MAG_INSERT | reload cancels per spec, no ammo gain |
| `mag_pooled` | fire 200 reloads | mag object allocations bounded by pool size |
| `creation_resume` | quit at L2, re-enter | returns to L2 with L0/L1 intact |
| `commit_boundary` | attempt L1 edit after match start | rejected |
| `physique_armour_fit` | every physique extreme × all 26 pieces | zero interpenetration |
| `physique_viewmodel` | min and max height characters | viewmodel offset identical |
| `squint_test` | any imported decal, silhouette at 30 m | character still reads as same character |
| `import_rejects_bad_format` | import a `.exe` renamed `.png` | refused with reason, no crash |
| `import_rejects_oversize` | image above the dimension ceiling | refused before decode |
| `import_rejects_traversal` | path containing `../` | refused, confined to import dir |
| `import_malformed` | truncated/corrupt PNG | refused, no crash, no leak |
| `console_cvar_roundtrip` | set → save → reload → get | value survives |
| `anti_patterns` | greps for named failures in ANTI_PATTERNS.md | none present in new code |

---

## SECTION 9 — TASK 7: CAPTURES

From the build you actually launched. Nothing from the editor, nothing from a test harness.

1. **Before/after FP feel** — same 15 s route: walk, sprint, stop, look sharp left/right, ADS,
   fire, recover. Two clips side by side.
2. **Reload, all three paths** — tactical (round chambered), empty (full FSM), and cancelled
   mid-reload. Slow enough that the mag drop is visible.
3. **Character creation, full flow** — L0 through L4 in one unbroken take, including one
   back-navigation and one resume-after-quit.
4. **Physique extremes** — shortest and tallest character, both in full 26-piece armour, both
   at max hip-shoulder separation, no clipping.
5. **Weapon render** — one weapon at ADS showing the material stack and wear; the same weapon
   with a player-imported image applied.
6. **Console session** — one continuous take: open console, list cvars, set one and see it
   take effect live, import a valid image and see it applied, then attempt all four refused
   imports and show each refusal message.
7. **Squint test** — the customized character as a solid black silhouette at 30 m, next to the
   default. Both must read as the same character.

---

## SECTION 10 — TASK 8: REPORT

Post one report containing, in this order:

1. **Task 0 audit table, re-run** — the same table, after. Every row that changed, changed.
2. **Research ledger summary** — per topic: counted sources, tier P count, tier V count,
   unreachable count, number of extracted values. Link the files.
3. **Contradictions found and how each was resolved.**
4. **Test results** — the exact commands, and for each test the before result and after
   result. Any test that passed before your change is called out as invalid and rewritten.
5. **Every tunable** — name, value, unit, file path. Full list, not a sample.
6. **The captures**, in the order above.
7. **Feel questions, answered in prose** — not "yes":
   - Does the weapon feel attached to the hands, or does it float?
   - Does the reload feel like a sequence of physical actions, or one blended animation?
   - Can you tell, without the HUD, which reload path just played?
   - Does the character you built at L1 feel like the character you play as?
   - Does the console feel like a tool or like a debug leftover?
8. **What you did not do, and why.** Blocked items, unmet quotas, sources you could not
   reach. State them plainly (R10).

Commit everything and push to `claude/brief-x-research` with `git push -u origin
claude/brief-x-research`. Do not open a pull request unless asked.

---

## SECTION 11 — FAILURE CONDITIONS

This session has failed, regardless of how much code exists, if any of these is true:

- A quote or source in `SOURCES.md` was not actually read (R3).
- A design number has no source id (R2).
- A test passes on the pre-change code and was reported as proof (Section 8).
- A capture came from anywhere other than the launched build (R1).
- A tunable is hardcoded in source (R7).
- The image import path accepts a traversal, an oversize file, or a malformed decode.
- The report claims completion for something not done (R10).

=== END PROMPT ===

---

## Notes for you (not part of the prompt)

**Sequencing.** This is a big prompt. If your VS Code session has a tight context budget, run
it in three passes and let the `research/` directory carry state between them:

- Pass 1: Sections 0–2 (contract + protocol + audit) and Task 1
- Pass 2: Tasks 2–3 (the other two research topics)
- Pass 3: Tasks 4–8 (synthesis, implement, test, capture, report)

Because R9 forces the research onto disk and into git, pass 2 and pass 3 can start with
"read `research/` and continue" — nothing is lost when the session resets.

**The two load-bearing rules.** If you strip this prompt down, keep R2 (no claim without a
source row) and R3 (never invent a source). Everything else is scaffolding; those two are
what turn "do deep research" into something with a pass/fail.

**On "revolutionary".** The novel part here is not any single system — it is that the reload
FSM, the creation layers, and the console import all declare their SIM/COSMETIC layer and
their commit boundary. Most games leave those implicit and then fight the resulting bugs for
years. Making both explicit at the spec level is the actual leverage.
