# Motion & Maneuvering Architecture — THE DECISION

Written 2026-08-10, session 2, under owner override of session 1's
deliberate refusal (see `NOTES.md`). Session 1 declined to write this
because axis 5 — per-character CPU cost at crowd scale — had no evidence
behind it. **That refusal was correct and its substance is honoured
here: axis 5 is still not answered from the literature, it is marked as
such in place, and it is converted from a research blocker into a build
task with a specification precise enough to execute (§9).**

Every other axis gets a call. Axes resting on partial evidence carry a
`THIN` marker with what would change the answer.

**Scope of writes this session:** only files under
`research/motion-architecture/`. Source was read extensively and not
touched.

---

## §0 — The decision in one paragraph

**Keep procedural, sim-driven posing. Formalise it. Add nothing from the
motion-matching or neural families, and add no animation crate.** The
player, nearby fighters and the crowd all run the *same* pose kernel —
the one that already exists in `main.rs` — differing only in update
*rate*, using the LOD rule the sim already has (`sim.rs:9221`, a pure
function of sim state, 80 m, distant actors at 15 Hz). Three additions,
in order: (1) extract the pure pose functions into a benchmarkable
module; (2) replace the bag of ~25 concurrent phase timers on `Fighter`
with an explicit action state machine, one verb at a time; (3)
generalise the existing two-bone IK solver so legs can use it, closing
the one genuine core-scope gap — foot placement on uneven ground. **The
reason this is not a compromise is a licence fact and an asset fact, not
a performance fact: this project owns zero hours of licence-cleared
motion capture and zero animation clips of any kind, and the reference
data behind the entire data-driven and learned literature is
CC BY-NC-ND 4.0.** Families B and C are not *worse* here — they are
*unavailable*, and would remain unavailable even if their CPU cost were
free.

---

## §1 — What actually exists today (read from source, not from the brief)

The brief and `DESIGN_MAP.md` describe a different codebase than the one
on disk. Everything in this section was verified by reading files this
session; file and line references are exact as of commit `e2866a9`.

### 1.1 There is no skeletal animation system at all

| Fact | Evidence | Label |
|---|---|---|
| Zero uses of Bevy's animation API | `AnimationPlayer\|AnimationGraph\|AnimationClip\|AnimationNodeIndex\|AnimationTransitions` across all `.rs` in `engine/crates`: **4 hits, none in code** — `jk_bevy/Cargo.toml`, `jk_tdm/Cargo.toml`, and two research `.md` files | MEASURED |
| The `bevy_animation` cargo feature is enabled and unused | `jk_tdm/Cargo.toml:13` lists `"bevy_animation"` in the feature set; no code consumes it | MEASURED |
| Zero animation assets | `engine/crates/jk_tdm/assets/**` — Glob returns **no files**. No glTF, no BVH, no clips | MEASURED |

**Consequence, and it is the load-bearing one:** any architecture whose
unit of work is "a clip" or "a database frame" has nothing to operate on
here. This is not a migration cost. It is a from-scratch content
pipeline that does not exist and has no budget line.

### 1.2 The body is posed by closed-form maths, per frame, from sim state

`jk_tdm/src/main.rs` contains a complete procedural rig:

| Function | Line | What it is |
|---|---|---|
| `gait_pose(crouch, theta, amp, accel_lean, settle) -> (hip_y, pitch)` | 2330 | Sinusoidal gait; hip bob at 2× stride frequency; lean capped at `ATHLETIC_LEAN_CAP = 0.52` rad |
| `solve_arm_ik(s, t, pole) -> (Quat, flex)` | 2579 | **Closed-form two-bone IK with pole vector**, `L1 = L2 = 0.21` m, elbow flex hard-clamped by `clamp_elbow_flex` (2417) to `[-5°, ELBOW_FLEX_MAX_DEG]` |
| `damped_spring` / `damped_spring3` | 2461 / 2474 | Closed-form critically-damped spring, shared by every secondary-motion channel |
| `spring_k_for_frequency(s, omega)` | ~1066 | Stiffness from `segment_inertia(s) · ω² · 1.0e4` — physically derived per segment |
| `step_leg_yaw(prev, aim_yaw, dt)` | 17441 | Legs lag the aim; torso covers up to `TORSO_AIM_LIMIT_DEG`; past that the legs must catch up — the turn-in-place |
| `torso_aim_offset` / `torso_coil_yaw` | 697 / 1268 | Torso/aim decoupling and windup coil |
| `chain_segment_scale(idx, elapsed, ramp)` | 1100 | The proximal-to-distal kinetic chain |
| `segment_data` / `segment_inertia` | 919 / 1046 | The 20-segment body's mass and inertia table |
| `reload_pose`, `trigger_finger_press` | 20150 / 2571 | Per-action hand targets; trigger travel 0.06 s out / 0.10 s back |

Per-fighter sprung state lives in `FighterRig` (2128–2188): gait phase
(**driven by distance, not time** — 2129), accel lean, sprint/carry
blends, and per-side sprung hand position, hand velocity, elbow pole,
pole velocity, plus a sprung clavicle. Spring constants are named and
tabulated: `SPRING_K_HAND_FOLLOW = 120`, `SPRING_K_ELBOW_POLE = 60`,
`SPRING_K_FINGER_SETTLE = 220`, `SPRING_K_SHOULDER = 45`,
`SPRING_K_CAMERA_BOOM = 90` (2488–2492).

**Session 1 wrote that "the 20-segment body's grip poses and reach are
hand-posed" and that the project "has no IK crate". The second is true
and the first is false.** There is no IK *crate*; there is IK. Session 1
searched crates.io for a capability that was already 2 500 lines into
`main.rs`. That correction drives §5.4 below.

### 1.3 The transform count per fighter

`FighterRig`'s posed-entity fields: `leg_l[3]`, `leg_r[3]`, `torso`,
`neck`, `arm_l[3]`, `arm_r[3]`, `toes[2]`, `lumbar`, `clavicles[2]`,
`weapon_root` = 3+3+1+1+3+3+2+1+2+1 = **20 posed transforms per
fighter** (19 body + 1 weapon root). Weapons, shield, armour rig and
chassis variants are shown/hidden, not posed.

*Caveat, stated rather than papered over: I did not verify that these 19
map one-to-one onto the 20 segments in `body-rig/SPEC_20_SEGMENT_RIG.md`.
They plainly do not (there is no separate head entity; the neck pivot
carries it). Treat 20 as the transform count, not as the spec's segment
count.*

### 1.4 The legs have no IK. This is the real gap.

`solve_arm_ik` is called at six fighter-rig sites (18388, 18417, 18465,
18496, 18580, 18592) and two viewmodel sites (20101, 20104). **Zero leg
sites.** Legs are posed open-loop from `rig.phase` sinusoids (17759–
17812, including an explicit "ankle roll" term at 17809). Grep for
`foot_ik|ground_ik` across `jk_tdm/src`: **zero hits.**

The brief's core scope names "foot placement and IK on uneven ground and
stairs". It does not exist, and it is the one core item that is missing
rather than merely informal.

### 1.5 The "state machine question" is already answered badly

`Fighter` (sim.rs:3093–3522) carries roughly two dozen concurrent phase
clocks that between them encode the character's action state:
`roll_t`/`roll_cd`/`roll_dir`/`roll_boost`, `knife_phase`/
`knife_committed`/`knife_dir`/`knife_struck`, `spear_wind_t`/
`spear_charge_t`, `bow_draw_t`, `flip_t`/`flip_kind`/`flip_used`/
`flip_recover_t`, `stride_wind_t`/`stride_t`, `mech_transition_t`/
`mech_exiting`, `sprint_gate_t`, `reload_t`, `switch_t`, `stagger_t`,
`shield_dip_t`, `turret_recover_t`, `climbing: Option<ClimbState>`.

Four of them are *already* proper enums — `MechJumpPhase` (5992),
`MechEnterStage` (6116), `BoundPhase` (6426), `TurretMode` (5758). The
codebase is drifting toward explicit states on its own.

The failure mode of the timer bag is documented in the source by the
people who hit it. `gatling_trigger_t`'s doc comment (3302–3317) records
a bug where gating heat decay on `fire_cd <= 0.0` instead of an explicit
hold state gave 88.9 % suppression instead of 100 %, and made time-to-vent
**tick-rate dependent: 11.18 s at 60 Hz, 9.08 s at 120 Hz, 8.22 s at
240 Hz.** That is precisely the class of bug an explicit state machine
prevents, discovered the expensive way, in this repo.

### 1.6 The SIM/COSMETIC line is already a hard boundary — in two places

1. **Crate boundary.** `jk_wall/Cargo.toml` depends on `rapier3d`
   (workspace, `enhanced-determinism`) + `nalgebra` + `jk_core`, and
   **not on bevy**. `jk_tdm/Cargo.toml` depends on `bevy 0.15` +
   `jk_core`, and **not on rapier**. The deterministic crowd solver and
   the renderer cannot see each other.
2. **Signature boundary.** `fn sync_fighters(time, game: Res<Game>, …)`
   (main.rs:17475) takes the sim **immutably**. The pose layer is
   structurally incapable of writing sim state. This is stronger than a
   convention; it is checked by the compiler on every build.

### 1.7 The hit-band constraint — the sharpest argument in this document

The sim classifies hits by height fraction (`HitZone`, sim.rs:2136), and
the render is *clamped to respect it*. `gait_pose`'s own doc (2300–2310)
records that a post-hoc settle dip put the head base at ~0.79 of height,
"outside the 0.82 band the test claims to guard, and classified as Arms
by the sim while looking like a head." The dip was moved inside the
function and clamped by `min_hip` so the band is enforced by
construction.

**A pose retrieved from a motion database, or emitted by a network, does
not know about your hit bands.** It will put the head where the data put
it. Every frame where that disagrees with the sim's band is a frame
where the player shoots what he sees and hits something else — an
intermittent, unreproducible desync between visuals and hit
registration. Motion matching and neural posing would each need a
constraint layer bolted on top *whose whole job is to undo the data*.
This is not a licence problem or a CPU problem, and it applies to this
game specifically because this game resolves hits off the visual band.

### 1.8 Character counts — "crowd" means two different things here

| Where | Max simultaneous characters | Evidence |
|---|---|---|
| `jk_tdm` fighters | **16** | `cfg.per_team` doc "5..=8 (owner's cap: 8v8)" (sim.rs:3043); `let per_team = cfg.per_team.clamp(1, 8)` (6860) |
| `jk_tdm` zombies | **40** | `pub const ZOMBIE_CAP: usize = 40` (6732) |
| **`jk_tdm` total posed characters** | **56** | 16 + 40, DERIVED |
| `jk_wall` bodies | 32×16×2 = **1 024** at top of the existing ladder | `jk_spike/src/bin/bench.rs:10` ladder `(8,5) (16,8) (25,10) (32,16)`; header cites a 400–700 full-physics ceiling |
| `jk_wall` posed characters | **0** | `jk_wall` has no rig and no bevy dep; `jk_bevy` draws each agent as ~9 static cuboid/capsule meshes (main.rs:337–423), no gait, no IK |

**The 250v250 crowd has no animation system, so it cannot have an
animation cost.** The animated population is 56. Axis 5's real question
for this repo is therefore *"does the existing procedural kernel pose 56
characters inside budget?"* — not *"what does motion matching cost at
250 bodies?"* That reframing is what makes §9 possible.

### 1.9 LOD already exists, and it is already determinism-safe

`sim.rs:9218–9231`: bots think every 12 ticks (**120/12 = 10 Hz**) and,
beyond 80 m from the player, act every 8 ticks (**120/8 = 15 Hz**). The
comment claims "15 Hz instead of 120" — I divided rather than trusting
the adjective, and code and comment agree. The comment states the rule
that makes it safe: *"The LOD level is a PURE function of sim state
(distance to the player) — never camera distance or frame rate, or every
replay diverges."*

Any pose-rate LOD added later must obey the same rule, and there is a
working precedent to copy.

---

## §2 — The nine-axis table

Scored per the brief. `[S-nn]` are rows in `SOURCES.md`; `[code:file:line]`
is this repo, read this session. **Every cell that is a judgement rather
than a measurement says so.**

| Axis | A — Procedural (what we have) | B — Motion matching | C — Learned | Verdict |
|---|---|---|---|---|
| **1. Responsiveness** | **Zero added latency by construction.** Pose is a pure function of *this frame's* sim state; there is no blend queue, no transition graph, no search. DERIVED from `sync_fighters(… Res<Game> …)` [code:main.rs:17475] and the pure kernels [code:main.rs:2330,2579]. **NOT MEASURED in ms.** `THIN` | Search returns a *continuation*, so response is bounded by trajectory-prediction window and inertia. **No number obtained this session** — the anchor paper was unreachable [S-07]. `THIN` | Same as B plus inference latency. **No number obtained.** `THIN` | **A wins, on a structural argument, not a measured one.** |
| **2. Realism** | Judgement only. Sinusoidal gait with real anthropometry and sprung secondaries; degrades gracefully (it cannot pick a wrong clip, only a slightly wrong angle). Its ceiling is authoring effort. **NOT MEASURED, NOT MEASURABLE without playtest.** `THIN — weakest axis in this table` | The literature's headline claim. Degrades by picking a poor neighbour when input leaves the data distribution. | Best-in-class in the literature; degrades by hallucinating plausible-but-wrong motion. | **B/C win on paper.** This is the one axis where the decision costs us something, and it is stated as a cost, not argued away. |
| **3. Determinism** | **SIM-safe where it matters and structurally COSMETIC everywhere else.** Sim owns state; pose reads immutably [code:main.rs:17475]; the crowd solver has no renderer at all [`jk_wall/Cargo.toml`]. Pure closed-form f32 maths, fixed `DT = 1/120` [`jk_core/timestep.rs:9`]. | Nearest-neighbour search is deterministic given identical DB and identical f32 inputs, but **must be COSMETIC-only**: any tie-break or index that fed gameplay would need bit-exactness across platforms. | **COSMETIC-only, and even then risky.** NN inference is deterministic on one platform and can drift across platforms/BLAS backends. | **A wins.** B/C are usable only behind a COSMETIC wall, which §1.7 says is a wall they cannot stand behind here. |
| **4. Memory** | **~0 bytes of motion content.** No clips, no DB [`assets/` empty]. Cost is code + per-fighter `FighterRig` state (~20 `Vec3`/`f32` fields). | **~24.3 MiB per hour of motion** at a 59-feature schema, 30 Hz [S-06], verified by reproducing their own arithmetic (§4 below). Scales linearly with content — the problem LMM exists to solve. | Compresses the DB into networks; the reported figures are in [S-07], **which I could not read.** `THIN` | **A wins by a wide margin, and it is the least interesting axis.** |
| **5. CPU** | **UNMEASURED. THIS IS THE NAMED GAP.** See §9. The budget arithmetic is in §4; the benchmark that closes it is specified in §9. | **UNMEASURED, and unmeasurable here without building it.** No primary source obtained: [S-07] exceeded the fetch size limit, [S-08] returned HTTP 403. | **UNMEASURED.** | **NO WINNER DECLARED.** Axis 5 does not decide this document; axis 6 does. See §3. |
| **6. Asset cost** | **Zero hours of motion data. It requires none.** | **Requires hours of licence-cleared mocap. We have zero, and cannot get the obvious source:** LaFAN1 is CC BY-NC-ND 4.0 [S-05] — NonCommercial *and* NoDerivatives. | **Requires more than B** (a training corpus, plus training infrastructure). Same data, same licence, same gate. | **A wins, and this is the axis that kills B and C outright** under brief R5. |
| **7. Authoring cost** | **High and superlinear in variety.** Every new verb is hand-tuned constants. This is the honest weakness — see §8's early-warning sign. Judgement, no source. `THIN` | Low per move once a pipeline exists; the cost moves to capture sessions. | Lowest per move; the cost moves to data and training. | **B/C win.** Second real cost of this decision, stated. |
| **8. Rust/Bevy availability** | **It is already built, running and shipped.** | `bevy_motion_matching` [S-02]: PERMISSIVE, but **not published to crates.io** and **Bevy version not stated anywhere in its README**. Explicitly WIP. | **Nothing exists** in the Bevy ecosystem. Would be a from-scratch inference stack. | **A wins.** |
| **9. Integration cost** | Zero — it is the incumbent. | Very high: needs a rigged skeleton, a clip pipeline, a DB, a feature schema, *and* a hit-band constraint layer (§1.7). | Higher still. | **A wins.** |

---

## §3 — Why axis 5 does not decide this, and what does

Session 1 stopped because the brief calls axis 5 "decisive". Having read
the code, **the brief is wrong about this project, and here is the
order of operations that shows it.**

Brief R5 is a *gate*, not an axis score: "any architecture requiring a
motion database must name a specific, licence-cleared source, **or be
marked unavailable**."

- Family B requires a motion database. Named source: none. The only
  complete, permissively-licensed reference implementation
  (`orangeduck/Motion-Matching`, MIT) trains on LaFAN1, CC BY-NC-ND 4.0
  [S-05]. → **unavailable.**
- Family C requires a *larger* motion database plus training. → **unavailable.**

Both families fail the gate before any performance number is consulted.
A per-character cost of 0 ms would not change it. **Axis 5 could only
have decided between two available options, and there is only one.**

What axis 5 *does* still decide is internal to family A: whether the
existing kernel needs pose-rate LOD at 56 characters. That is a genuine
open question, it is small, and §9 answers it with a measurement rather
than a citation.

---

## §4 — Arithmetic, shown in full

Every derived number in this document, so it can be checked.

**(a) O3DE's motion database figure [S-06] — reproduced, not trusted.**
Their claim, verbatim: *"A motion capture database holding 1 hour of
animation data together with a sample rate of 30 Hz to extract features
will generate 108,000 frames. Using the default feature schema,
comprising of 59 features, will result in a feature matrix holding ~6.4
million values and use ~24.3 MB of memory."*

```
3600 s × 30 Hz                = 108 000 frames          ✓ matches "108,000"
108 000 × 59                  = 6 372 000 values        ✓ matches "~6.4 million"
6 372 000 × 4 bytes (f32)     = 25 488 000 bytes
25 488 000 / 1 048 576        = 24.31 MiB               ✓ matches "~24.3 MB"
```
All three chain correctly and pin the element type to f32 and the unit
to MiB. **A fabricated triple would not close like this** — this is the
consistency check the aiming-ledger incident teaches.
**Precision ceiling [R8]:** the schema samples at 30 Hz, so nothing
derived from it has resolution finer than **33.3 ms**. Our sim runs at
120 Hz (8.33 ms). Do not build 120 Hz timing on a 30 Hz feature figure.

**(b) The pose budget for this game.** Render target is **ASSUMED**
(no fps target is stated in any file I read), so both plausible targets
are shown. Allocation of 10 % of the frame to all character posing is
also **ASSUMED** — it is a proposal, not a measurement.

```
60 fps:  1000/60  = 16.667 ms/frame ; 10 % = 1666.7 µs ; ÷56 chars = 29.76 µs/char/frame
120 fps: 1000/120 =  8.333 ms/frame ; 10 % =  833.3 µs ; ÷56 chars = 14.88 µs/char/frame
```

**(c) Posed transforms at full roster.**
`56 characters × 20 posed transforms = 1 120 transforms/frame` (DERIVED
from §1.3 and §1.8). This, not the trigonometry, is the number most
likely to dominate — Bevy propagates a hierarchy of that size every
frame regardless of what wrote it.

**(d) LOD rates.** `SIM_HZ = 120` [`jk_core/timestep.rs:8`];
`120/12 = 10 Hz` think, `120/8 = 15 Hz` far-act. Code and comment agree.

**(e) The vent bug's tick-rate spread**, quoted from the source comment
as evidence for §1.5, not derived by me: 11.18 s / 9.08 s / 8.22 s at
60/120/240 Hz. Ratio 60 Hz : 240 Hz = 11.18/8.22 = **1.360**.

---

## §5 — REJECTIONS

Each names the axis that killed it. Nothing is rejected as "not the best
fit".

### 5.1 Family B — Motion matching. **REJECTED on axis 6 (asset cost / R5 gate).**
We hold zero hours of licence-cleared motion capture and zero animation
clips of any kind (`assets/` is empty; `AnimationClip` appears zero times
in code). The canonical open reference implementation is MIT but its
data path is the Ubisoft LaFAN1 dataset, **CC BY-NC-ND 4.0 —
NonCommercial *and* NoDerivatives** [S-05, verified from the repo's own
docs in session 1, not from the brief's say-so]. Under R3 that is class
`NON-COMMERCIAL`: readable, unshippable.
**Secondary kills, each independently sufficient:** axis 3 — it can only
ever be COSMETIC, and §1.7 shows this game's hit resolution reads the
visual band, so a COSMETIC-only pose source would desync hit
registration from what the player sees; axis 9 — it needs a rigged
skeleton and clip pipeline that do not exist.

### 5.2 Family C — Learned (LMM / PFNN / DeepMimic lineage). **REJECTED on axis 6, more severely than B.**
It needs everything B needs *plus* a training corpus and training
infrastructure. Same dataset, same licence, same gate. **Secondary kill,
axis 3:** neural inference is deterministic on one platform and can drift
across platforms and BLAS backends; brief R4 says check rather than
assume, and we cannot even get to the check without the data.
**Secondary kill, axis 8:** nothing exists in the Bevy ecosystem.

### 5.3 `bevy_motion_matching` (voxell-tech). **REJECTED on axis 8 (Rust/Bevy availability).**
PERMISSIVE (dual MIT/Apache-2.0) [S-02], and that is the only good news:
**its Bevy version is not stated anywhere in its README**, and it is
**not published to crates.io** — its own README says it is "being split
into library and example crates, to be published", future tense. Per the
brief, an unstated or mismatched Bevy version is a rewrite, not an
integration. Moot regardless, since §5.1 rejects the family.

### 5.4 `bevy_mod_inverse_kinematics` 0.8.0. **REJECTED on axis 9 (integration cost) — and this reverses session 1's "quick win".**
The crate is real, `MIT OR Apache-2.0` (PERMISSIVE), and version-matched:
0.8.0 depends on `bevy = "^0.15"`, confirmed via the crates.io
dependencies API [S-03]. All of that stands. **What session 1 did not
know is that we already have the capability.** `solve_arm_ik`
[code:main.rs:2579] is a closed-form two-bone solver with a pole vector —
the same feature set the crate advertises — plus two things the crate
does not have: a **biomechanical elbow clamp** (`clamp_elbow_flex`, 2417,
hyperextension floor −5°) and **per-side critically-damped sprung
targets** for hand and elbow pole (`SPRING_K_HAND_FOLLOW = 120`,
`SPRING_K_ELBOW_POLE = 60`, 2488–2489) whose whole purpose is documented
at 2143–2152: without them the hand teleports between grip poses.
Adopting the crate would mean maintaining two IK paths with different
behaviour. Worse, its author has moved on — latest 0.11.0 targets Bevy
`^0.18` [S-03] — so pinning 0.8.0 means owning a frozen branch, which is
the same maintenance burden as owning our own function, with less
control.
**What would change this:** a need for chains longer than two bones (a
spine chain, a mech leg with more than thigh+shin, a tail) — that is
FABRIK/CCD territory and `solve_arm_ik` genuinely cannot do it. Note the
crate would not help there either; per [S-03] it is two-bone only. At
that point the answer is a new solver, not this crate.

### 5.5 Physics-driven / active-ragdoll locomotion (DReCon-shaped). **REJECTED on axis 9, then axis 3.**
`jk_tdm` has **no physics engine** — it does not depend on rapier at all
[`jk_tdm/Cargo.toml`]. Adding rapier to the client crate to obtain
ragdoll is a major integration, and any physics that fed back into pose
would then need to not touch the hit bands (§1.7) or the immutability of
`Res<Game>` (§1.6).
**Not rejected: cosmetic death ragdoll.** Today a corpse is rotated flat
by a single quaternion [code:main.rs:17512–17514]. A purely visual
ragdoll, written only to render transforms, sits entirely inside the
COSMETIC layer and is a legitimate, contained future item. It is out of
scope for this decision, not forbidden by it.

### 5.6 `bevy_animation_graph`. **REJECTED for now on axes 8 and 9 — with a standing watch order.**
PERMISSIVE (dual Apache-2.0/MIT) [S-01], and genuinely the closest thing
to a ready foundation: state machines as graph nodes, two-bone IK, and
**partial ragdolls where some bones simulate and some stay kinematic** —
exactly the shape of an active-ragdoll hit reaction. Two problems.
(i) Axis 8: its compat table covers 0.12 through 0.19 but **Bevy 0.15 was
not individually confirmed** [S-01], and session 1 recorded that
honestly rather than assuming it. (ii) Axis 9, and this is the real one:
its unit of work is an *animation clip*, and we have none. A graph over
an empty clip set buys nothing.
**Watch order for whoever reads this next:** if this project ever
acquires a rigged character with clips, re-open S-01 first, starting by
confirming the 0.15 row of its compatibility table. Its partial-ragdoll
feature is the single most valuable third-party capability identified in
this whole topic.

### 5.7 Anything from Unreal Engine 5. **REJECTED on licence, class `PROPRIETARY`.**
Lyra, the Game Animation Sample and the Motion Matching / Pose Search
plugin are Unreal EULA, not open source, and cannot be copied into a
Rust project. Technique may be read; code may not be ported. This is not
a close call and is recorded only because the brief's own history shows
a draft that would have recommended it.

### 5.8 Opening a new research cycle to close axis 5. **REJECTED — see §9.**

---

## §6 — Extend / replace / leave alone (brief R6)

| Existing system | Call | Reason |
|---|---|---|
| 20-segment body (`segment_data`, `segment_inertia`) | **LEAVE ALONE** | Sourced from de Leva 1996 via `body-rig/SPEC_20_SEGMENT_RIG.md`; feeds `spring_k_for_frequency`. Nothing in this decision touches it. |
| Elastic load model (load → release, release 2–3× faster) | **LEAVE ALONE** | Encoded in the wind/release timers (`spear_wind_t`, `bow_draw_t`, `stride_wind_t`). §7's state machine must *preserve* these timings exactly; it is a re-encoding, not a retune. |
| Kinetic chain (`chain_segment_scale`) | **LEAVE ALONE** | Already built once and routed through multiple verbs, per TRV-0081. |
| `solve_arm_ik` + spring solver | **EXTEND** | Generalise segment lengths so legs can call it (§7 step 3). Additive; existing call sites keep `L1 = L2 = 0.21` behaviour. |
| `gait_pose` and the leg sinusoids | **EXTEND** | Keep as the base pose; add a ground-contact correction on top. Do not replace — the hip-bob and lean caps are tied to the 0.82 head band (§1.7). |
| `Fighter`'s phase-timer bag | **REPLACE, incrementally** | The only replacement in this document, and it earns it: §1.5's tick-rate-dependent vent bug is the documented cost of the current encoding. Replace one verb at a time; four are already enum-shaped. |
| `jk_wall` locomotion | **LEAVE ALONE** | It has no pose layer and needs none. It is a physics crowd, not an animated one. |
| Bot LOD (`sim.rs:9218`) | **EXTEND if §9 says so** | Copy the rule, do not invent a second one. |
| `bevy_animation` cargo feature | **LEAVE ALONE (flagged)** | Enabled and unused. Removing it is a build-time win and a semantic clarification, but it is a `Cargo.toml` edit outside this session's write scope. Noted for Friday, not done. |

---

## §7 — MIGRATION PATH

Ordered. Each step independently shippable and independently revertible.
No big-bang rewrite. **Step 0 comes first because it sizes everything
after it.**

**Step 0 — Build the benchmark (§9). ~0.5 day.**
Deliverable: `posebench` reporting per-character pose cost in µs.
Revert: delete one file. **Blocks nothing else if it is skipped, but
every later step is guessing without it.**

**Step 1 — Extract the pose kernel. ~1 day. Zero behaviour change.**
Move the pure functions (`gait_pose`, `solve_arm_ik`, `clamp_elbow_flex`,
`damped_spring`, `damped_spring3`, `spring_k_for_frequency`,
`step_leg_yaw`, `torso_aim_offset`, `torso_coil_yaw`,
`chain_segment_scale`, `trigger_finger_press`, `reload_pose`) out of the
29 261-line `main.rs` into `jk_tdm/src/pose.rs`. Pure in, pure out; no
Bevy types beyond `Vec3`/`Quat`. `sync_fighters` becomes a driver that
calls them.
*Why first:* it makes Step 0's benchmark trivial to write, makes the
existing pose tests (`main.rs:25105`, `25473`, `25718`, `27813`, `28872`,
`29135`) addressable, and is a pure move — reviewable by diff.
*Revert:* move them back.

**Step 2 — Leg IK on ground contact. ~2 days.**
(a) Generalise: `solve_two_bone_ik(s, t, pole, l1, l2)`; keep
`solve_arm_ik` as a thin wrapper passing `0.21, 0.21`, so all eight
existing call sites are untouched and their tests still pass.
(b) Drive the ankle target from ground height under each foot, clamped
so the hip never leaves the band `gait_pose` guarantees (§1.7 — the
clamp is not optional).
(c) Extend the elbow-clamp pattern to a knee clamp (no hyperextension).
*Why second:* it is the only genuinely missing core-scope item, it needs
no new dependency, and it is visible immediately on stairs.
*Revert:* one feature flag on the ground query; the wrapper stays.
*Determinism:* COSMETIC only. It must not move the hit bands, and the
clamp in (b) is what enforces that. **Add a test asserting the head base
stays on the 0.82 line with leg IK active on a slope** — this is the
regression that §1.7 says has already happened once.

**Step 3 — The action state machine, one verb at a time. ~1 day per verb.**
Introduce `enum Action { Idle, Roll{..}, Knife{..}, SpearWind{..},
BowDraw{..}, Flip{..}, Stride{..}, MechTransition{..}, Climb{..}, … }`
plus a single clock on `Fighter`, and migrate verbs individually.
**Start with `MechJumpPhase`** — it is already an enum with a clock
(`mech_jump_t`), so the first migration is a rename and proves the
pattern with near-zero risk. Then `MechEnterStage`. Then the roll, which
has four correlated fields.
*Constraint, non-negotiable:* this is SIM state. Every migrated verb must
(i) preserve its timings to the digit, (ii) be added to the replay
digest, and (iii) keep its existing tests green unmodified. If a test
needs changing, the migration changed behaviour and is wrong.
*Revert:* per verb, independently.

**Step 4 — Pose-rate LOD. ~1 day. ONLY IF §9's benchmark demands it.**
Decimate pose updates for distant characters, reusing the *existing*
rule at `sim.rs:9227` (80 m, pure function of sim state). Interpolate
between pose updates so decimation is invisible.
*Do not build this speculatively.* If the benchmark shows 10× headroom
it is dead code with a maintenance cost.

**Step 5 — Cosmetic death ragdoll. Unscheduled, listed so it is not forgotten.**
Purely visual, COSMETIC layer only, replaces the single-quaternion flat
corpse. Out of scope for this decision.

**Explicitly NOT in the path:** adding an animation crate; adding rapier
to `jk_tdm`; building a clip pipeline; acquiring mocap; any pose source
that is not a pure function of sim state.

---

## §8 — Cost, determinism verdict, and the honest risk

### 8.1 What this costs

| Item | Engineering | Asset production | Runtime budget |
|---|---|---|---|
| Step 0 benchmark | ~0.5 day | none | none |
| Step 1 extraction | ~1 day | none | none (pure move) |
| Step 2 leg IK | ~2 days | none | +1 two-bone solve × 2 legs/char/frame |
| Step 3 state machine | ~1 day/verb, ~10 verbs, spread over time | none | neutral to negative (fewer redundant timers) |
| Step 4 LOD | ~1 day, conditional | none | reduces cost |
| **Total** | **~4 days + incremental** | **ZERO** | **Small increase, unquantified until Step 0** |

The zero in the asset column is the whole argument. Family B's
equivalent row would read "hours of studio mocap capture, cleanup,
retargeting, and a licence we do not have".

### 8.2 The determinism verdict (brief R4)

**The SIM/COSMETIC line, drawn explicitly:**

- **SIM (must stay deterministic, must be in the replay digest):**
  `jk_wall::WallSim` in full; `jk_tdm::sim::TdmSim` in full — including
  the action state machine from Step 3, which is sim state and is the
  *only* part of this decision that touches the SIM side.
- **COSMETIC (may vary freely between clients):** every pose system in
  `main.rs` — `sync_fighters`, the viewmodel rig, leg IK, springs,
  camera boom, death poses.
- **The boundary is compiler-enforced today** and must stay so: pose
  systems take `Res<Game>`, never `ResMut<Game>` [code:main.rs:17475].
  **This is the single line of review that protects the replay suite.**
- **The boundary is also crate-level for the crowd:** `jk_wall` has no
  bevy dependency and `jk_tdm` has no rapier dependency. Do not add
  either.

**What the replay suite must still pass:** everything it passes now,
unchanged, at every step. Step 3 is the only step that can break it, and
its acceptance criterion is stated in §7 — existing tests green
*unmodified*, timings preserved to the digit, new enum in the digest.

**Classification of every technique named in this document (brief R4
requires this explicitly):**

| Technique | Class | Reason |
|---|---|---|
| Closed-form gait / spring / two-bone IK (ours) | **COSMETIC-only** by placement; the maths is deterministic, but it is on the render side and nothing reads it back | It is already there and already behind an immutable borrow |
| Leg IK on ground contact (Step 2) | **COSMETIC-only** | Must not move hit bands; clamp enforces it |
| Action state machine (Step 3) | **SIM-safe, and SIM-resident** | Integer/enum state + one f32 clock at fixed `DT`; no search, no float accumulation beyond what exists |
| Pose-rate LOD (Step 4) | **COSMETIC-only** | Must key off sim state, not camera or frame rate — the rule at `sim.rs:9221` |
| Motion matching search | **COSMETIC-only at best** | Deterministic given identical DB + f32 inputs, but cannot be trusted in SIM across platforms — and §1.7 shows COSMETIC-only is not sufficient for this game's hit model |
| Neural inference (LMM/PFNN/RL) | **COSMETIC-only, and unverified even there** | Cross-platform float drift is a known hazard; R4 says check, and we could not reach the data to check |
| Physics ragdoll driving pose | **COSMETIC-only** | Would require rapier in `jk_tdm`, which does not have it |

### 8.3 The honest risk — what could make this the wrong call in a year

1. **Someone buys motion data.** A commercial mocap library with a
   permissive commercial licence, or a funded capture session, flips
   axis 6 overnight — and axis 6 is the *only* axis on which B and C
   were rejected outright. **Early warning: a purchase order, or anyone
   asking "can we licence a mocap pack?"** If that happens, this
   document should be re-opened at §5.1, not defended.
2. **The animated population grows past ~56.** If `jk_wall` ever gets a
   real animated renderer, or the roster cap lifts past 8v8, the crowd
   question becomes real for the first time. **Early warning: BM-1's
   per-character number crossing the §4(b) budget, or a brief that
   raises `per_team` past 8.**
3. **Authoring cost overtakes everything.** Procedural posing is
   superlinear in *variety*: every verb is hand-tuned constants. **Early
   warning: the count of hand-tuned pose constants in the pose module
   growing faster than the count of verbs.** That ratio is cheap to
   measure once Step 1 puts them all in one file, and Step 1 is
   therefore also the instrument for this warning.
4. **The realism ceiling is hit and matters.** Axis 2 is the axis we
   lose. If playtest feedback ever converges on "the movement looks
   synthetic" rather than on specific fixable poses, that is the signal
   that the family, not the tuning, is the limit.

---

## §9 — AXIS 5: MEASURE IT HERE. THE BENCHMARK SPECIFICATION.

**Recommendation: build the benchmark. Do not chase the talk.**

### 9.1 Why measurement beats a citation, argued rather than asserted

The plan of record was to find a conference talk reporting per-character
motion-matching CPU cost. Four reasons that is the wrong instrument for
this decision:

1. **It answers a question we no longer face.** §3 shows families B and
   C fail the R5 licence gate before any performance figure is
   consulted. A talk's number could not change the outcome.
2. **The question axis 5 actually poses here is different.** §1.8:
   the animated population is **56**, not 250. The live question is
   whether *our* kernel poses 56 characters in budget — which no talk
   about anyone else's engine can answer.
3. **A foreign number is not transferable.** Different rig, different
   bone count, different DB, different CPU, different frame budget. It
   would give an order of magnitude for a system we are not building.
4. **We have the engine and a benchmark precedent.**
   `jk_spike/src/bin/bench.rs` already walks a body-count ladder and
   prints ×realtime for `jk_wall`; `autoplay_report`
   (`sim.rs:21266`, `cargo test --release -p jk_tdm -- --ignored autoplay
   --nocapture`) already drives full headless matches. The scaffolding
   exists.

**Where a talk would still add value, stated so the gap is not hidden:**
a cross-check on the order of magnitude of DB search cost, *if* risk (1)
in §8.3 ever fires and family B comes back on the table. It is not on
the critical path today. The two transcript routes TOTO33 proved on
2026-08-08 (`youtube-transcript-api`; Internet Archive `_djvu.txt`) are
**unavailable in this session's environment** — see §10.

### 9.2 BM-1 — the pose-kernel microbenchmark (the primary instrument)

**Depends on:** Step 1 (kernel extraction). Trivial after it; awkward before.

**Where:** `engine/crates/jk_tdm/src/bin/posebench.rs`, or an
`#[ignore]`d test following the `autoplay_report` precedent. **No new
dependency** — `std::time::Instant` is sufficient and matches
`bench.rs`'s existing style. Do not add criterion.

**What to vary:**
- `N` — simulated character count ∈ `{1, 8, 16, 32, 56, 128, 256, 512}`.
  56 is the shipped ceiling (§1.8); the values past it exist to find the
  knee, not because we plan to be there.
- Pose regime per character, sampled in fixed proportion so the mix is
  reproducible: `{idle, walk, sprint, crouch, rolling, armed-two-handed
  (both arms IK), armed-one-handed, bow-draw (the most expensive arm
  path), in-mech}`. Use a fixed seed; the mix must be identical between
  runs or the comparison is noise.
- Both regimes for the ground query in Step 2: leg IK OFF and ON, so the
  cost of Step 2 is isolated and attributable.

**What to measure, per configuration:**
1. **`ns` per call** for each pure kernel individually, over ≥10^7
   iterations with a `black_box` on the result: `gait_pose`,
   `solve_two_bone_ik`, `damped_spring3`, `step_leg_yaw`,
   `chain_segment_scale`. Report mean and the ratio of the most
   expensive to the cheapest.
2. **`µs` per character per frame** for the full per-fighter pose body —
   the sum of the kernels plus the branch logic, excluding ECS
   overhead. Report mean, p50, p99 over ≥10 000 frames.
3. **`ms` per frame for `sync_fighters` as a whole**, in a real Bevy app
   at each `N`, wrapped in `Instant::now()` at system entry/exit and
   accumulated into a resource. (Bevy's `FrameTimeDiagnosticsPlugin` gives
   frame time; it does not give per-system time, so instrument directly.)
4. **`ms` per frame for Bevy's transform propagation** at each `N`
   (`propagate_transforms` / `sync_simple_transforms`), and the total
   entity count. **§4(c) predicts 1 120 posed transforms at N = 56 and
   this is the number most likely to dominate.** If it does, the answer
   is a flatter rig, not a different animation family — and that must be
   distinguishable in the output, which is why this is a separate row.
5. **Total frame time** at each `N`, so the pose share is a fraction and
   not an absolute floating free of context.

**Report format** (mirroring `bench.rs`'s existing one-line-per-row
style, so it is readable in CI logs):
```
N | pose µs/char | sync_fighters ms | propagate ms | frame ms | pose % of frame
```

**Thresholds — what result flips which decision.** These are the
acceptance criteria; they are `ASSUMED` budgets from §4(b), not measured
requirements, and whoever runs this should say so in the output.

| Measured result | Conclusion | Action |
|---|---|---|
| pose ≤ 3 µs/char at N = 56 (≈10× headroom vs the 29.76 µs 60 fps budget) | Axis 5 does not constrain the architecture at all | **Skip Step 4 entirely.** Record the number in `SOURCES.md` and close axis 5. |
| 3 µs < pose ≤ 29.76 µs/char at N = 56 | In budget at 60 fps, tight or over at 120 | Ship Steps 1–3; **hold Step 4 ready**, revisit if a 120 fps target is ever set |
| pose > 29.76 µs/char at N = 56 | The kernel itself is the bottleneck | **Build Step 4 (pose-rate LOD).** Still not a reason to change family — no other family is available (§3) |
| `propagate` > 3 × `sync_fighters` | The hierarchy, not the maths, is the cost | Flatten the rig (fewer entities per fighter, or `Transform` written in world space for leaf segments). **Not an animation-architecture change.** |
| The knee in the `N` ladder falls below 56 | Something scales worse than linearly — likely allocation or query iteration | Profile before concluding anything; do not attribute it to the pose maths without evidence |

**What would flip the whole document, and nothing less will:** a
measurement showing procedural posing cannot hit the shipped roster
*even with LOD*. That is not a plausible outcome for closed-form
trigonometry on 56 characters, and if it occurs the fault is almost
certainly ECS overhead (row 4), not the kernel — which is exactly why
rows 3 and 4 are measured separately.

### 9.3 BM-2 — extend the existing crowd bench (secondary, ~1 hour)

`jk_spike/src/bin/bench.rs` already prints `bodies | files × ranks |
sim 15 s in | ×realtime`. Add one derived column:

```
µs per body per step = wall_seconds × 1e6 / (steps × bodies)
   where steps = 15 × SIM_HZ = 15 × 120 = 1800
```

**Purpose, and it is a negative result by design:** to record on paper
that `jk_wall`'s per-body cost is *entirely* physics and behaviour, with
**zero animation component**, because that crate has no rig. It
establishes the baseline any future proposal to animate the crowd must
be measured against, and it stops the 250v250 figure from being quoted
in animation discussions where it does not belong.

### 9.4 What this benchmark cannot tell us — stated so nobody over-claims it

- It cannot measure motion matching's cost, because we would have to
  build motion matching first.
- It cannot measure axis 1 (input-to-visible-response latency in ms);
  that needs a different instrument (a frame-stepped capture), and it is
  not built here.
- It cannot measure axis 2 (realism) at all.
- It measures **this hardware**. Record the CPU model and core count in
  the output header, or the numbers are unreproducible — the same
  discipline the `bench.rs` header already shows by citing "8 cores".

---

## §10 — What I could not answer (brief §6.7)

Reported in the same detail as what I got.

1. **Learned Motion Matching (SIGGRAPH 2020), the paper itself:
   UNREACHABLE-BY-TOOL.** Located precisely — the landing page
   [S-04] gives the exact path
   `theorangeduck.com/media/uploads/other_stuff/Learned_Motion_Matching.pdf`
   — and the fetch failed with `maxContentLength size of 10485760
   exceeded`. The PDF is over 10 MiB and this environment has no
   `curl`, no `wget`, no Python, so there is no route to download and
   read it locally. **This is the same failure class as the Bournemouth
   parkour thesis in `traversal/SOURCES.md`.** It would have given
   axis 4 and axis 5 numbers for family C. *What shape of answer it
   could contain, per the standing rule about over-valuing unread
   sources:* memory-versus-quality curves and inference timings. Neither
   can reverse an R5 licence rejection, so this gap does not change §5.2
   — it only leaves the axis-4/5 cells for family C empty, and they are
   marked empty.
2. **"GPU-based Motion Matching for Crowds in the Unreal Engine"
   (SIGGRAPH Asia 2020 Posters, 10.1145/3415264.3425474):
   UNREACHABLE — HTTP 403** on `dl.acm.org/doi/fullHtml/…`. Search
   surfaced a "decreased computation times up-to 95 %" claim; that is
   **SNIPPET-ONLY, relative, absolute-free, and NOT CARRIED.** It is
   also a 2-page poster, so its expected yield was low.
3. **No tier-V source this session, and the reason is new.** TOTO33
   proved on 2026-08-08 that GDC talks are readable via
   `youtube-transcript-api` and Internet Archive `_djvu.txt`. **Both
   routes are unavailable in this session's environment.** Probed
   directly: `curl`, `wget`, `python`, `python3`, `py`, `node`,
   `cmd.exe`, `powershell.exe`, `pwsh.exe` — **every one returns exit
   127, command not found**, and `ls`/`grep`/`head`/`wc`/`git` are
   absent too. The only network tools are `WebSearch` and `WebFetch`.
   **So `NOTES.md`'s stated blocker ("GDC Vault access… not fetchable")
   is stale — it was solved two days later — and the tier is re-blocked
   today for an unrelated reason.** Both facts belong in the record.
4. **Per-character CPU cost at crowd scale from any primary source:
   NOT OBTAINED.** This is the gap session 1 refused to write around,
   and it is still open as a *literature* question. §9 converts it into a
   measurement. **The decision does not rest on it** — §3 shows why —
   but the axis-5 row of §2 declares no winner, honestly, rather than
   inventing one.
5. **Not started, and named so it is not mistaken for covered:** the
   brief's Task 4 adjacent quota (active ragdoll and get-up, ORCA/RVO
   crowd avoidance and its high-density failure modes, tactical AI cover
   and peek from *Game AI Pro*) — **0 of 7**, exactly as session 1 left
   it. PFNN, DReCon, Robust Motion In-betweening and the DeepMimic
   lineage remain unfetched. Under the R5 gate none of them can change
   §5.1 or §5.2, which is why the owner's "do not open a new research
   cycle" instruction is compatible with shipping this decision.
6. **The brief's §5 test suite (`source_quota` etc.): NOT BUILT**, and
   `source_quota` would **fail** — 14 core / ≥3 tier-V was never
   reached, and this session added no counted sources beyond one. That
   is recorded rather than worked around. Whoever owns TRV-0190 should
   know the quota test would fail today, by design, because the owner
   overrode the quota in favour of shipping the decision.

---

## §11 — Ledger summary

**Counted sources: 6** (S-01…S-06). Tier P: 6. **Tier V: 0.**
UNREACHABLE this session: 2 (S-07 size limit, S-08 HTTP 403).
Licences by class: **PERMISSIVE 4** (`bevy_animation_graph`,
`bevy_motion_matching`, `bevy_mod_inverse_kinematics`,
`orangeduck/Motion-Matching` code); **NON-COMMERCIAL 1** (LaFAN1);
**NOT VERIFIED 1** (O3DE — cited for a number, not recommended for
shipping, so R6's gate is not triggered).
**Nothing classed NON-COMMERCIAL or PROPRIETARY appears in the
recommendation.** The recommendation adds no third-party artifact at all,
which is the cleanest possible pass of that test.

The seventh and decisive source is the repository itself, read this
session: `jk_tdm/src/main.rs`, `jk_tdm/src/sim.rs`, `jk_wall/src/*`,
`jk_core/src/timestep.rs`, `jk_spike/src/bin/bench.rs`, and all seven
`Cargo.toml` files. **Every claim in §1 traces to a file and line.** As
TOTO33 recorded after the vertical-maps pass: read the source tree
before the literature when the question is "what can our thing cope
with". That is what happened here, and it is what turned a blocked
survey into a decision.
