# BRIEF VIII (v2, optimized) — MASTER BRIEF
## Athletic Motion · Limbs & Hands · First Person · HUD · Spear · Third Person · Mech · The Forge

**How to run this:** paste from `OPERATING CONTRACT` to the end into Claude Code.
This file is the **single standalone source of truth** for every system below —
you should not need to open Briefs II–VII to execute it. Work sections in
order. Do not stop between sections to ask permission to continue.

**What changed from Brief VIII v1:** two systems that existed in earlier briefs
were silently missing and are restored here — **Section 2 (arms/hands/fingers
craft)** and **Section 6 (third-person camera)**, the latter explicitly requested.
The bow is parked with a written decision rather than dropped (Appendix A).
Everything else is deepened with sourced biomechanics rather than invented
numbers.

---

## OPERATING CONTRACT — obey throughout, outranks convenience

**C1 — Visible or it didn't happen.** Every section ends with captures from
**the build the player actually launches**, in `handback/brief-viii/<section>/`.
Screenshots where a pose is the deliverable; a 5–10s clip wherever *motion* is
the deliverable. Tests passing + no capture = **not complete**. If no capture
facility exists, building one is the first thing you build in Section 0.

**C2 — Ship enabled.** No flags defaulting off. No work on unmerged branches.
Default config = intended experience. If the player's run command launches a
stale binary, fixing that is Section 0's first deliverable and blocks all else.

**C3 — Two-layer motion law.** Every system is classified, in a comment above it:
- **SIM** — fixed timestep, seeded RNG, replay-identical. Anything touching
  damage, hit position, projectile flight, plate detachment, score.
- **COSMETIC** — frame-rate driven, may use wall-clock, never feeds back into
  sim state. Breathing, fidgets, look-at, flinch, camera spring, viewmodel sway,
  debris, heat shimmer.
A cosmetic system that writes a sim value is a bug even if it looks correct.

**C4 — Tunables are data.** Every constant here is a *starting value*.
```
config/motion.ron        # breathing, fidget, springs, kinetic-chain timings
config/limbs.ron         # arm/hand/finger limits + grip poses
config/locomotion.ron    # sprint, cut, stop, lean, bob budgets
config/weapons/*.ron     # per-weapon FP data, spear, heavy rifle, minigun
config/camera.ron        # FP + third-person rig states
config/hud.ron           # anchors, colors, thresholds
config/mech.ron          # scale, mobility, damage states, materials
config/forge/*.ron       # part manifests
```
Hardcoded magic numbers in systems code = failed review.

**C5 — Additive blend order + budget.** Cosmetic layers compose in this fixed
order, each clamped to its own budget:
`locomotion → posture state → breathing → weight shift → look-at → fidget →
reaction/flinch → recoil → aim override`
Total additive **translation on the first-person viewmodel** never exceeds the
standing breathing envelope (Section 3's no-bounce rule is absolute). The
third-person body has per-layer clamps but no global translation cap.

**C6 — Player intent wins.** While the player aims or fires, every reactive
layer attenuates ×0.3. Nothing the world does may fight the player's aim.

**C7 — One mechanism, many users.** Where this brief names a shared utility
(the kinetic chain, §1.4; the spring solver, §2.5), build it **once** and route
every consumer through it. Five hand-tuned copies of "agile" is the failure
mode this brief exists to end.

**C8 — Perceptual threshold rule.** If a value is too subtle to see in a
capture, it is not implemented — it is a rounding error. When a tunable's
visible effect can't be identified in the clip, raise it until it can, then
tune down. "Coded but imperceptible" was a root cause last cycle.

**C9 — Report honestly.** Partial work is named as partial, with the reason,
in the handback. Never mark a motion section complete on tests alone.

### The non-negotiables (regression bugs if broken)

1. First person: **the weapon never rises toward the camera.** No ADS, any
   weapon, any input, ever.
2. **Scoped weapons hide the viewmodel entirely** while scoped.
3. **The viewmodel does not translate/bounce** standing, walking, or firing.
   Rotation and mechanism animation only. No climb, no accumulation across a spray.
4. A feature is done when it is **visibly present and reachable in the launched
   build** — not when it compiles.
5. **Characters are never statues** — breathing, weight shift, head-look, grip
   fidget run continuously on every character, always.
6. **The mech is fully textured** — no placeholder gray. CI-checkable (§7).
7. **The mech visibly transforms** — plates part on entry/exit and physically
   drop off as it takes damage.
8. **The mech never flies**, never leaves the ground beyond step height, and is
   **1.15× soldier height** — not 1.5×, not concept scale.
9. **HUD = CS:GO four-corner anatomy exactly.** Center stays empty but for the
   crosshair.
10. **All movement reads athletic** — no instant velocity-to-zero, no power move
    starting from a dead-neutral pose.
11. **The spear throw follows real throwing-athlete mechanics** — hip-shoulder
    separation, planted block leg, proximal-to-distal whip (§5).
12. **The Forge exists**, is reachable from menu and lobby, and what is designed
    there is exactly what spawns (§8).

---

## SECTION 0 — AUDIT BEFORE ANYTHING

Read the code and run the game before changing anything.

### 0.1 The evidence table
`FEATURE | SPEC'D | CODED | TESTED | VISIBLE IN LAUNCHED BUILD | ROOT CAUSE | FIX`

Cover at minimum: viewmodel placement, no-bounce, seeded recoil, HUD four
corners, mech existence/scale/materials/damage-states, spear thrust, spear
throw, idle-life layer, hands/finger rig, third-person camera, Forge.

Root cause must name one of: *branch never merged / run config launches stale
binary / flag defaulted off / system registered but never scheduled / value
below perceptual threshold (C8) / asset missing, silent fallback / code orphaned,
never called.* "Unclear" is not accepted — instrument until it is clear.

### 0.2 Runtime truth, not config truth
Log at startup and paste into the handback: viewmodel FOV and offsets **as read
at runtime**, the git commit hash and build profile the running binary reports,
and every animation system actually scheduled. Reading a config file proves
nothing; the running process is evidence.

### 0.3 Specific probes
1. **Locomotion.** Is the sprint cycle visually distinct from a walk? Does
   turning have plant/lean or is reorientation instant? Does any start or stop
   hit instant zero velocity? Capture a clip of current run/turn/stop.
2. **Spear.** Capture the current throw and thrust now, for frame-by-frame
   comparison against §5.
3. **Idle.** List every animation layer running on a stationary character this
   second. If the answer is "none," write that plainly — it is finding #1 for §1.
4. **Hands.** Does the rig have forearm twist bones and metacarpals? Are fingers
   posed per weapon, or is the hand a static mesh? (§2 depends on this answer.)
5. **Mech.** Which materials are actually bound per submesh? Screenshots
   front/side/rear. Does entry animate or teleport?

### 0.4 HARD GATE — fix delivery first
Before any new feature work: the existing viewmodel work must be **visibly on
screen in the launched build**, proven by a before/after screenshot pair in
`handback/brief-viii/section-0/`. **Do not proceed to §1 until this is green.**

---

## SECTION 1 — THE ATHLETIC ("ACHILLES") MOTION DOCTRINE

**Goal:** every character reads as a trained, explosive athlete — never a
floating mannequin, never a stop-motion puppet.

### 1.1 The doctrine in one sentence
Homer's Achilles carries two epithets: **"swift of foot"** and **"spear-famed."**
Speed and the spear are the same trait in the oldest version of this archetype.
Every powerful action — running, cutting, throwing, thrusting — must read
**fast and controlled at once**: never sluggish, never floaty, never
simultaneous-rigid.

To make that buildable, this section is grounded in measured sprint
biomechanics; §5 is grounded in measured javelin biomechanics. Both cite real
ranges so the values are defensible rather than vibes.

### 1.2 Sprint-derived locomotion (third-person body, both rigs)
Must **not** leak into first-person viewmodel translation (§3 owns that).

- **Acceleration lean.** Sprint research shows a pronounced forward trunk lean
  during early acceleration that progressively rises to near-upright at top
  speed. Track starts from blocks reach ~45°; a standing game start should use
  **28–32°** over the first 0.3–0.5s, easing to **8–12°** at sustained top
  speed. Never pop instantly to run posture.
- **Knee drive.** Swing-leg knee flexion peaks near **130°** in elite sprinting
  — author the swing to read as *drive*, not shuffle. Knee rises toward hip
  height at top speed.
- **Arm drive.** Elbows ~90°, opposite-arm-to-leg, hands travelling roughly
  hip-to-chin, synced to leg cadence. Arm drive does perceptual work that
  balance-capped travel speed cannot — this is the cheapest "he's fast" cue in
  the game.
- **Cadence over reach.** Speed comes from stride frequency *and* length rising
  together with **ground contact time falling**. Practically: as speed
  increases, shorten contact-pose duration in the cycle rather than lengthening
  the stride — a long-reaching stride reads as floaty.
- **Ground contact under the centre of mass.** Feet plant under the hips, not
  reaching ahead (overstriding = the "ice skater" read). Slight toe-down snap
  on contact, never a flat stamp.
- **Vertical-oscillation budget.** Cap hip bob at **≤4cm** at top speed. Motion
  reads as horizontal drive, not up-down bounce. A bouncy run reads cheap even
  when everything else is right.
- **Composed effort.** Shoulders and face stay controlled at max effort —
  controlled-fast reads as *more* athletic than strained-fast.

### 1.3 Agile stop / turn / cut
- **No instant reorientation.** A direction change >90° at speed triggers a
  **plant-and-cut**: outside foot plants wide and slightly ahead of centre,
  **hips rotate into the new direction before the torso/shoulders follow**,
  with a 5–8% height drop during the cut. Duration **0.15–0.25s**, scaled by speed.
- **Full stops never hit instant zero.** Universal across both rigs, every stop
  and every landing. This is the single largest remaining source of "floaty" —
  hunt it everywhere in the state machine, not just where a brief named it.
- **Starts get 2–3 frames of anticipation** — a small coil before the first
  step. No power move, including "just start running," begins from a dead-neutral
  pose on frame 0.

### 1.4 The Universal Kinetic-Chain Rule — build it ONCE (C7)

> **Proximal-to-distal sequencing:** the core/hips move first; energy whips
> outward — hips → torso → shoulder → elbow → wrist/weapon-tip — each segment
> beginning to accelerate only once the previous one is already moving. The
> result reads as one connected whip instead of a simultaneous, stiff,
> all-joints-at-once rotation.

This is not a metaphor — it is measured: in throwing, as one joint reaches the
end of its range and decelerates, the next is rapidly accelerated, transferring
momentum outward through the chain.

**Implementation shape** (one utility, `motion::kinetic_chain`):
```
KineticChain {
  segments: [Hips, Torso, Shoulder, Elbow, Wrist, Tip],
  onset_offsets_s: [0.00, 0.035, 0.065, 0.090, 0.110, 0.125], // tunable
  peak_scale:      [1.00, 1.15, 1.35, 1.60, 1.85, 2.10],      // velocity gain outward
  deceleration_gate: true,  // a segment's peak triggers the next segment's onset
}
```
Route **every** power move through it: spear throw release (§5), spear thrust,
dodge/roll launch, mech braced side-step (heavier offsets ×2.2), melee shove,
sprint start. Because they share one mechanism, one test (§1.6.1) validates
them all.

### 1.5 Named anti-patterns — hunt as bugs, not aesthetic notes
- **"The mannequin spin"** — simultaneous full-body rotation, zero inter-segment delay.
- **"The wall stop"** — any stop or landing hitting instant zero velocity.
- **"The ice skater"** — locomotion with no lean and no knee drive; a static
  pose translated across the ground.
- **"The switch flip"** — a power move beginning from neutral with zero anticipation.
- **"The floating gun"** — viewmodel translating independently of a rotation cause.

Grep for these names in review notes and code comments.

### 1.6 Tests + captures — completion gate
1. **Kinetic-chain timing test:** for each power move, log angular-velocity
   peak times per segment; assert hip < torso < shoulder < distal with a minimum
   inter-segment offset. Failure names the segment that fired early.
2. **Lean-and-cut test:** direction-change fuzz; assert plant precedes hip
   rotation precedes torso rotation, no orientation snap beyond a per-frame cap.
3. **Zero-instant-stop sweep:** scan every stop/landing/turn state on both rigs
   for velocity discontinuity above threshold. Regression sweep, not one-off.
4. **Vertical-bob budget test:** hip-height trace over a scripted sprint stays
   under cap.
5. **Captures:** OLD vs NEW side-by-side of sprint-start, sprint-stop, hard cut.

---

## SECTION 2 — ARMS, JOINTS, HANDS ★ RESTORED (was missing from v1)

Hands are what the player sees most in first person and what makes a
third-person soldier read as a person. Clamps below come from measured human
active range of motion, so procedural motion can never produce a broken limb.

### 2.0 Reference gathering (do first, ~20 min)
If web access exists, save 10–15 stills to
`handback/brief-viii/section-2/reference/` using: *"FPS viewmodel hands rifle
grip"*, *"hand rig topology finger joints game"*, *"tactical glove close-up"*,
*"hand anatomy MCP PIP DIP diagram"*, *"javelin grip hand"*. Write
`NOTES.md` extracting per image: where the thumb wraps, how many fingers contact
the grip, wrist deviation, elbow direction, how the glove breaks light at the
knuckles. If no web access, say so and proceed — the spec below is self-sufficient.

### 2.1 Bone hierarchy (both rigs, mirrored)
```
clavicle → upper_arm → forearm → [twist_01, twist_02] → hand
hand → thumb_01 → thumb_02 → thumb_03
hand → {index,middle,ring,pinky}_meta → _01 → _02 → _03
```
- **Twist bones mandatory** — two forearm twists at 0%/50%/100% of wrist roll,
  one upper-arm twist at 50% of shoulder roll. Without them the forearm
  candy-wraps on every grip pose: the #1 cause of "cheap-looking hands."
- **Metacarpals mandatory** for index and pinky minimum — the palm must cup.
  A flat palm reads as a mitten.

### 2.2 Joint limits — clamp all procedural poses (`config/limbs.ron`)

| Joint | Axis | Range |
|---|---|---|
| Shoulder | flex/ext | −60° … +170° |
| Shoulder | abduction | 0° … +170° |
| Shoulder | rotation | −80° … +90° |
| Elbow | flexion | **0° … +145°**; hyperextension clamped at −5° |
| Forearm | pronation/supination | −85° … +85° (across twist bones) |
| Wrist | flex/ext | −70° … +80° |
| Wrist | radial/ulnar | −20° … +30° |
| Finger MCP | flexion | 0° … **+90°** (ext −25°) |
| Finger PIP | flexion | 0° … **+110°** (pinky +135°) |
| Finger DIP | flexion | 0° … **+80°** |
| Finger MCP | spread | ±15° (index/pinky more, middle least) |
| Thumb CMC | opposition | 0° … +45° |
| Thumb MCP / IP | flexion | 0…+55° / 0…+80° |

**Coupling rules (cheap realism):** DIP ≈ **0.7 × PIP** (tendon-linked —
independent DIP motion looks robotic); curl reduces spread toward 0; ring+pinky
curl together at 0.85 coupling; phalanx ratio proximal:middle:distal ≈
**1.0 : 0.62 : 0.42**.

### 2.3 Arm solving
- **Two-bone IK + pole vector** both arms (root=shoulder, mid=elbow, tip=hand);
  the pole target sets the elbow plane. This is the standard limb solver.
- **Pole rule:** elbow points down-and-outward **15–25°** off the vertical plane
  for a weapon grip — never straight down (limp), never 90° out (chicken-wing).
  `elbow_pole_offset` is per-weapon data.
- **Soft-clamp the last 5% of reach** so the arm eases into straight. Hard IK
  pops read as broken.
- **The support hand is IK'd to a socket on the weapon**, never animated free.
  Move the weapon and the hand follows — this is what keeps the grip glued
  through recoil, reloads, and fidgets.

### 2.4 Grip pose library (`config/limbs.ron`)
One named pose per interaction as MCP/PIP/DIP triples per digit + wrist offset.
Minimum: `rifle_primary`, `rifle_support`, `pistol`, `spear_shaft`,
`spear_overhead`, `shield_grip`, `open_relaxed`, `climb`, `mech_control_yoke`.
- Blend between poses over **0.12–0.18s**. Never snap.
- **Trigger finger is independent** — leaves the grip pose, travels to the
  trigger in 0.06s on fire, returns in 0.10s. Highest visual value per line of
  code in the whole rig.

### 2.5 Procedural secondary motion — the spring solver (build once, C7)
```
x'' = -k(x - target) - c·x'      c = 2√k  (critical damping)
```
| Element | k | Note |
|---|---|---|
| Hand follow (weapon lag) | 120 | slight trail behind aim = the "weight" cue |
| Elbow pole | 60 | elbows settle late |
| Finger settle | 220 | snappy; fingers are light |
| Shoulder/clavicle | 45 | heaviest, slowest |
| Camera boom (§6) | 90 | |
| Mech hull | 22 | mass reads as low stiffness |

- **Follow-through:** on fast rotation (>180°/s), fire, or throw, the hand target
  lags **0.035s** and overshoots ≤3°. Under-damp the *return* only (ζ ≈ 0.7) so
  it settles with one small overshoot — perfectly damped returns read dead.
- **Inertia coupling:** during sprint/dodge, arm targets take acceleration ×0.02
  as positional offset, clamped 4cm. Third-person always; first-person **as
  rotation only** (C5).
- All of this is **COSMETIC** (C3). A spring output must never feed a hit position.

### 2.6 Art direction — arms and hands
- **Silhouette:** gauntlet plates break the forearm into three readable masses
  (wrist cuff, mid-plate, elbow cop). A tube reads as a mitten at 30m.
- **Model actual knuckle bumps** — do not rely on normal maps. The knuckle line
  catches rim light and is the main "living hand" cue in first person.
- **Materials:** glove leather roughness 0.65–0.85 with fine grain normal; plate
  roughness 0.35–0.55, metallic 1.0; **edge wear masked to plate borders and
  knuckles only** — uniform wear reads as noise. Palm slightly glossier than the
  back (sweat under glove).
- Seams and stitching at finger bases; wrist strap with a small buckle; tiny
  asymmetries (one strap looser) kill the CG-perfect look.
- **Budget:** ~6–9k tris per FP hand, ~2k third-person LOD0. **3 edge loops per
  finger joint** — 2 loops crease badly at 110° PIP flexion.

### 2.7 Tests + captures — completion gate
- **Joint-limit fuzz:** 10,000 seeded random pose targets; no joint exceeds its
  clamp, no NaN, no flip.
- **Coupling test:** DIP ≈ 0.7×PIP and ring/pinky coupling hold across full curl.
- **Grip-attachment test:** per grip pose, every contacting fingertip within 8mm
  of the grip surface; no finger bone penetrates the weapon mesh.
- **Twist distribution test:** roll wrist 85°; twists carry 50%/100%, no vertex
  exceeds a candy-wrap threshold.
- **No-bounce regression:** §3's viewmodel translation test still passes with all
  §2 systems live.
- **Captures:** FP close-up clip of (a) idle finger fidget, (b) trigger-finger
  travel on fire, (c) reload with support hand staying glued, (d) third-person
  arm swing at sprint; plus a still at 110° curl for silhouette review.

---

## SECTION 3 — FIRST PERSON (complete authoritative spec)

**Goal:** the weapon sits low and right, CS:GO-style. It never rises to the
face. It does not bounce. All aiming is the fixed centre crosshair.

### 3.1 The two defining rules
**Rule 1 — no ADS, ever.** Left-click fires from the fixed carry position
permanently. Right-click does nothing on standard weapons except alt-functions
(burst toggle, silencer, zoom on scoped-class only). No state, for any weapon,
translates the model toward the camera.

**Rule 2 — scoping hides the weapon.** For scoped-class weapons, right-click
zooms the **camera** and sets the viewmodel invisible: zoomed world + circular
vignette mask + thin black hairline cross spanning the screen. No weapon model
renders. This is the actual mechanism behind "the gun never comes to your face."

### 3.2 Placement
- Separate viewmodel render pass, own FOV **68°**; world FOV independent.
- Offsets **+0.11m right, −0.13m down, +0.32m forward**; x/y/z tunable, clamps
  ±0.1m.
- Both fingered hands visible (§2). Weapon occupies the lower-right quadrant;
  the muzzle may approach centre, the receiver and hands never cross the vertical
  midline.
- **On-weapon ammo display:** emissive segmented bar on the left receiver face,
  mirroring magazine fraction, segments extinguishing as rounds are spent, one
  pulse on reload complete. Colour driven by the Forge accent (§8).

### 3.3 The no-bounce specification
The bob clock advances **only while moving**. Standing = frozen = zero positional motion.
- **Standing (<5% run speed):** zero bob. Permitted: the §1 breathing layer and
  rotational mouse-sway lag (viewmodel rotation interpolates toward camera with
  ~0.1s lag, amplitude ≤**0.3°**). **No translation.**
- **Walking/running:** speed-scaled bob. Cycle ≈0.21–0.22s by weapon speed class;
  vertical amplitude `speed_frac × 0.25` tunable; lateral ×0.4 at half frequency;
  at full sprint the weapon pulls **down and back** ≤6cm — away from the face, always.
- **Airborne:** amplitude ÷5.
- **Firing — the critical rule:** zero positional translation. Only (a) a small
  rotational kick about the weapon pivot, magnitude = camera aimpunch ×
  `viewmodel_recoil` (default 1.0); (b) bolt/slide mechanism animation; (c) a
  back-slide along the barrel axis ≤**1.5cm** returning in ≤**120ms** via
  critically damped spring. No climb, no drift, no accumulation across a
  30-round spray. Rest pose restored within **0.4s** of the last shot.
- Scoped: no viewmodel exists, so nothing can bounce.

### 3.4 Weapon handling states ★ NEW — the "feel" layer between the rules
These are what make shooting feel like a modern shooter rather than a static prop:
- **Sprint carry:** at >85% run speed the weapon lowers **18°** and rotates
  8° inward (rotation only, C5). **Sprint-out time** (sprint → able to fire)
  **0.20s**; this is a real skill lever — expose it per weapon class
  (SMG 0.15s / rifle 0.20s / heavy 0.30s).
- **Ready-up on stop:** returns over 0.15s with one small overshoot (ζ ≈ 0.7).
- **Reload craft:** tactical reload (round in chamber) and empty reload are
  **separate clips** with different durations; the support hand IK'd to the
  magazine well throughout (§2.3); the trigger finger leaves the trigger during
  reload and returns on completion.
- **Low-ready / obstruction:** approaching a wall within 0.6m rotates the muzzle
  up-and-in 22° (rotation only) so the barrel never visually enters geometry.
- **Inspect:** an idle-triggered or bound inspect that rotates the weapon in
  hand — pure rotation, shows off the Forge finish (§8).
- **Hit feedback:** crosshair hitmarker (2 frames, scaled by damage), distinct
  headshot tone, and a kill-confirm exhale + re-grip on the hands (§1.3).

### 3.5 Recoil — three channels, fixed crosshair
1. **Bullet deviation (truth):** eye direction + aimpunch × `recoil_scale`
   (**2.0**) + inaccuracy cone + fixed spread.
2. **Camera (what you see):** eye angles + aimpunch × 2.0 ×
   `view_recoil_tracking` (**0.45**) + cosmetic view punch (×**0.055**/shot).
   The camera shows only 45% of true deflection.
3. **Viewmodel (cosmetic):** rotational kick per §3.3. Never positional.

**The crosshair stays pinned to screen centre.** Impacts drift above/beside it
during a spray — that is the skill expression. `crosshair_follow_recoil` default OFF.

**Deterministic spray tables:** per weapon, a fixed integer seed generates
`(angle°, magnitude)` per shot index at load. Full-auto smoothing: lerp
consecutive entries by **0.55**. First-shot suppression: shots 0–3 scale
**0.75 → 1.0**. Per shot add `(−cos·mag, −sin·mag)` to punch **velocity**.
All remaining randomness draws from the sim's seeded stream so replays
reproduce every bullet.

**Recovery:** per tick punch angle ×= `e^(−8·dt)` then −`18°·dt` toward zero;
punch velocity ×= `e^(−4.5·dt)`. Camera at rest 0.3–0.5s after fire stops.
Spray index decays once `time_since_last_shot > cycletime × 1.1`, one decade
per **0.5s**.

**Inaccuracy states:** stand/crouch/walk/run/jump/land per weapon; movement
penalty ramps from zero at **34%** of max speed to full at **95%** — this is
what makes counter-strafing work.

### 3.6 Tests + captures — completion gate
1. **Screen-intrusion sweep:** every weapon × stance × 5s continuous fire —
   nothing crosses the vertical midline leftward, nothing enters the central
   circle of radius 12% screen height (muzzle tip exempt), screen-space area
   never exceeds rest +10%.
2. **Bounce meter:** standing, full magazine — translation ≤**2cm** peak,
   ≤**2mm** 0.4s after last shot. Per-frame trace on failure.
3. **Scope-hide test:** zero viewmodel draw calls while scoped; restore in one frame.
4. **Golden-file spray test:** fixed seed, 30-shot spray, impacts within 1mm of
   golden; replay reproduces a magazine bit-identically.
5. **Sprint-out test:** time from sprint release to first legal shot matches the
   per-class value ±1 frame.
6. Crosshair provably never moves; camera at rest within 0.5s.
7. **Captures:** rest, mid-spray, scoped, sprint-carry, both reloads.

---

## SECTION 4 — THE HUD (complete authoritative spec)

Four corners, one info cluster each; centre empty but for the crosshair and
transient bars; three semantic colours — white nominal, red danger, faction hue
identity.

Global: UI scale **0.85** (0.5–0.95 slider). Translucent dark rounded pills
behind text clusters, alpha **0.5**. Font **Saira SemiCondensed**, three weights;
ALL-CAPS labels, sentence-case names. Safe area 5% of screen dimension from any
edge. Layout data-driven (anchor + offset per element) in `config/hud.ron`.

- **4.1 Bottom-left — vitals.** Health cross + number (largest text, ~34px
  @1080p) + depleting bar; armor shield + number + bar to its right. Shield icon
  gains a helmet variant when a helmet is owned. Red at **≤25 HP**, pulse at
  **≤20**. `hud_vitals_style`: 0 numbers+bars (default), 1 numbers only.
- **4.2 Bottom-right — ammo + loadout.** `26 / 90`, magazine large/bright,
  reserve small/dim; magazine red at ≤25%. Above: vertical loadout strip, active
  weapon brightest and offset left, grenades as glyphs with counts.
- **4.3 Top-left — minimap + resources.** Circular, rotates with facing
  (tunable), scale 0.25–1.0 (default 0.7). Self = white arrow; squadmates = dots
  in five fixed colours; spotted enemies = red dots ghost-fading to last known.
  Objective icon beneath. Resource counter below; award toasts stack above it,
  each fading after 2.5s.
- **4.4 Top-centre — timer + score.** `M:SS` centre chip; faction scores
  flanking (defender blue left, raider orange right) with alive counters. Timer
  red at final 0:10.
- **4.5 Top-right — killfeed.** Newest at bottom, right-aligned:
  `Killer [+Assist] [weapon glyph][modifiers] Victim`. Modifiers: headshot,
  penetration, noscope, through-smoke, blind, flash-assist. Names blue
  **#71A6FF** / orange **#ECCE51**. Local-player rows get a 2px **#B50000**
  border on rgba(0,0,0,0.5), radius 4px. Max 5 rows, ~36px, 5s lifetime (+50%
  when involved).
- **4.6 Crosshair.** Full settings family: size (5), gap (0, negatives allowed),
  thickness (1), dot (off), outline (on, 1), colour presets + custom RGB
  (default green 50,250,50), alpha 200, T-shape option, static/dynamic (default
  **classic static**). Scoped-class weapons draw no crosshair when unscoped.
- **4.7 State overlays.** Flash: white plate + frozen afterimage, intensity by
  angle (<53° ≈ 2s hold, 53–72° ≈ 0.5s, 72–101° ≈ 0.1s, behind ≈ negligible),
  ≤5s total, exponential fade. Damage direction: red edge wedge, 0.8s fade
  (audio stays the primary channel). Context progress bar centred ~58% down,
  ALL-CAPS label, release cancels. Death: brief killer-cam then spectate with
  "SPECTATING <name>". Low HP = colour change only, no persistent vignette.
- **4.8 Scoreboard + loadout.** TAB: two faction blocks, columns
  K/A/D/DMG/Score/Ping, local row highlighted, minimap switches to square
  overview while held. Loadout menu: flat grid, all categories visible at once,
  prices shown, refunds during prep only.

**4.9 Tests:** layout snapshots at **1920×1080, 2560×1440, 1280×720** — every
element inside safe area, no overlaps. State tests: health colour flips at
exactly 25/20; ammo red at 25%; killfeed renders all six modifier glyphs from a
scripted stream; crosshair settings round-trip through the settings file.
**Capture:** one screenshot per corner plus a full-HUD combat still.

---

## SECTION 5 — SPEAR: THE ATHLETIC REWORK

A hand-thrown spear is mechanically a javelin, and javelin technique is among
the most rigorously studied throwing motions in sport. The full competition
technique (13–17 approach strides) is too slow for combat — below is that
technique compressed to game beats while keeping the biomechanical *read*.

### 5.1 What the research actually says (and what it buys us)
- Greater release speed correlates with **greater runway speed at the plant**
  and **greater hip-shoulder separation** — this is the direct justification for
  the running-throw bonus (§5.4) being a *mechanic*, not decoration.
- Energy transfers **legs → hips → torso → arm → javelin**; as each joint
  reaches end of range and decelerates, the next accelerates. That is §1.4.
- **A soft/bent block knee loses energy** — the plant leg must read firm and
  braced, never a slide or a squat.
- Optimal release angle sits around **30–36°** above flat.

### 5.2 The throw cycle
- **Momentum carry (mechanic).** If moving ≥70% run speed with ≥2 prior steps
  when raise is pressed, up to 2 steps of momentum feed the throw as a
  compressed approach.
- **Raise / bow position (0.4s, hold to aim).** Arm cocks the spear overhead
  beside the head; hips rotate toward the target while shoulders and throwing
  arm stay rotated away — **hip-shoulder separation ("X-factor"), target 35–45°
  of differential**. Torso arches slightly back (the **bow**), chest open,
  off-hand extended toward the target as the aim line. Move speed 80%. Camera
  FOV −4° (camera-only). This pose **is allowed** to bring the spear high on
  screen — wind-up, not ADS; §3's rules are untouched.
- **Aim:** hold indefinitely. **No arc, no landing marker** — the arc is learned,
  and that is the skill. Reticle: small dot.
- **Release (0.25s), sub-timed:**
  - **Plant (~0.05s):** lead foot plants firmly, slightly ahead of centre, turned
    30–45° into the throw line — **the block**. It converts forward momentum into
    rotational/vertical release energy. Must read *braced*: knee stiff, not soft.
  - **Whip (~0.08s, the fastest beat):** fires through the §1.4 chain in strict
    order — plant leg drives hips, hips rotate torso, torso drives shoulder,
    shoulder extends elbow, elbow leads wrist, spear leaves the fingertips last
    at peak linear velocity. Release angle **30–36°** above flat (tunable).
  - **Follow-through / reverse (~0.12s):** back leg swings through past the plant
    leg to decelerate rotation. This is why real throwers don't stay rigid — it
    is what makes the motion read as controlled power rather than a stopped
    swing. Ends slightly rotated past the throw line, then settles (§1.3 ease-out).
- **Cancel:** lowers the spear without throwing.
- **Melee thrust:** unaffected on primary input, but now routes through the same
  §1.4 utility — a shorter, sharper version of the same rule.

### 5.3 Physics, damage, stick, retrieval
- Launch **22 m/s** along the camera ray (×1.15 with the running bonus) + full
  gravity in the deterministic sim, same integrator as everything else.
- Impact ≥30° to surface → **embeds**, quivers 0.4s (§2.5 spring, cosmetic);
  shallower → bounces with a clatter. Sticks in bodies; the corpse carries it.
- Damage **85 body, ×2 head (170), ×0.75 legs**. Against the mech, angle armor
  applies as usual.
- Retrieval: interact returns it to loadout. Carry max 2. Persist ≥60s.
- Human rig only — the mech fires its launcher instead, reusing the aim-hold
  input with no throw animation.

### 5.4 The running-throw bonus (mechanic)
A throw initiated at ≥70% run speed with ≥2 steps of momentum gets **velocity
×1.15**. Athletic movement gets a mechanical payoff, exactly as an approach run
rewards a real thrower over a standing throw.

### 5.5 Screen profile
`spear_raised`: shaft may cross top-centre; hand/grip stays right of the vertical
midline; nothing enters the lower-centre reticle zone. Guns keep §3's strict profile.

### 5.6 Tests + captures — completion gate
1. **Kinetic-chain timing** applied to the release: hip → torso → shoulder →
   elbow → wrist peaks in strict order with minimum offsets.
2. **Hip-shoulder separation test:** at peak wind-up, differential is within the
   35–45° band.
3. **Deterministic flight test:** 5 angle/distance combos match golden file.
4. **Running-throw bonus test:** release velocity with and without the condition;
   ×1.15 applies exactly when met, never otherwise.
5. Stick-vs-bounce threshold (25°/30°/35°). Retrieval round-trip. Screen profile.
   Replay bit-identical.
6. **Captures:** full cycle clip (raise/bow → plant → whip → follow-through →
   stick → retrieve) **and** a standing-vs-running side-by-side showing the
   approach and the arc difference.

---

## SECTION 6 — THIRD PERSON: CAMERA + POSITIONING ★ RESTORED (was missing from v1)

**Goal:** modern over-shoulder third person — the character visibly
repositioning while strafing, the camera tightening on aim, scoped weapons
handing off to the full-screen optic.

### 6.1 Camera rig (`config/camera.ron`)
- **Hip:** boom **2.2m** back, **+0.45m** right of head, **+0.12m** up. FOV = world FOV.
- **Sprint:** boom eases to **2.5m** with 0.12s positional lag; FOV +4° (speed cue).
- **Aim (hold RMB — camera state only, legal under §3):** boom **1.35m**,
  **+0.55m** right, FOV **−12°**, 0.15s transition. The character raises the
  weapon to a proper shoulder mount on the **third-person model** — no viewmodel
  is involved, so no conflict with the no-ADS rule.
- **Scoped weapons:** aiming the heavy rifle from third person hands off to the
  §3 full-screen scope overlay; unscoping returns to third person.
- **Shoulder swap:** bindable, on the controls screen, 0.18s ease.
- **Collision:** spring against geometry (k=90, §2.5); never clips, never pops
  through walls. On hard occlusion, pull in rather than cut.
- Crosshair stays centre; hip fire uses the same §3.5 inaccuracy states — third
  person is a camera choice, never an accuracy buff.

### 6.2 Character positioning (what makes it read)
- **Aiming:** character faces the aim direction; **8-way strafe blend**;
  upper-body additive aim to **±60°** before legs turn-in-place catches up.
- **Not aiming:** character faces velocity.
- Start/stop get §1.3's anticipation and foot plant; every new blend obeys the
  no-instant-velocity invariant.
- **Camera-relative movement** — input maps to camera space, not character space,
  so strafing reads as strafing.
- Toggle stays on **V**. Works on foot and in the mech.

### 6.3 Tests + captures
Camera fuzz sweep (10-min scripted run against walls/corners; never intersects
geometry). Offset assertion per state. Scope handoff (TP → overlay → back, one
frame each way, HUD intact). Torso-limit test (±60° additive, beyond triggers
turn-in-place). **Capture:** walk → strafe-aim → zoom → scope handoff → unscope.

---

## SECTION 7 — THE MECH (complete authoritative spec)

### 7.1 Scale
**1.15 × soldier height** (1.8m soldier → **2.07m** mech). Concept art reads
~1.7×; deliberately compressed — keep the silhouette language, not the scale.
At 1.15× it shares doors, nav, and cover with soldiers, which keeps hit model
and pathfinding sane.

### 7.2 Silhouette and materials
- **Legs:** reverse-joint digitigrade, exposed piston/hydraulic detail at knee
  and ankle, wide splayed two-or-three-toed feet with a rear heel spur. Hunched,
  forward-leaning stance.
- **No head.** An angular recessed sensor visor slit across the hull front,
  emissive (red or cyan, consistent). Fiction: unmanned/neural-linked frame.
- **Armor:** faceted gunmetal plates, chamfered edges, matte (roughness
  0.6–0.8), subtle edge wear. Emissive only on visor and pod status lights.
- **Hazard chevrons ONLY** on shoulder-pod cover and knee plates — ≤10% of
  surface; an accent, not a paint job.
- **Left shoulder:** 4-tube missile pod with per-tube status lights.
  **Right arm:** weapon hardpoint (minigun or heavy rifle, §7.8). Antenna mast
  rear-left. Ground decals + dust puff per footfall.
- **Mandatory full PBR pass:** albedo, normal, roughness/metallic, AO per
  submesh. Add hydraulic cylinders at knee/ankle, cable runs at joints,
  intake/exhaust vents, decals (unit number, faction emblem), dust gradient up
  the lower legs.
- **Material audit test:** iterate every submesh; assert none uses a
  default/placeholder material and every material has non-empty texture slots.
  **Untextured gray is a CI failure, not an opinion.**
- **Silhouette check:** render front/side/¾ beside the concept art in the
  handback. Still reads "old robot" → the plates are too flat; add bevel and
  inset variation until light breaks across facets.

### 7.3 Grounded — flight is deleted
Every flight/jet/hover state, input, and code path removed or hard-disabled.
Never leaves the ground beyond a **0.4m** step, cannot jump, no air control
anywhere.

### 7.4 Mobility kit ★ EXPANDED (the "improve mech mobility" ask)
All ground-bound, all mass-honest:
- **Walk:** 85% of soldier run speed.
- **Power stride (new):** a 2.5s sustained push to **110%** of soldier run speed,
  costs heat (§7.8) and locks the missile pod while active. Wind-up 0.35s with
  visible hull pitch-forward; cannot turn faster than 90°/s while striding.
  This is the mech's answer to being outrun, without ever leaving the ground.
- **Braced side-step:** ≈3m in 0.9s, near-zero steering. Its wind-up routes
  through §1.4 with **×2.2 timing offsets** — hull/hips lead, legs follow.
- **Pivot turn:** hull yaw capped **180°/s**; a soldier circling close should
  *feel* the mech lag. Turning >120° plays a plant-and-pivot with a visible foot
  reposition, never a slide.
- **Brace stance (new):** hold to plant both feet — movement 0, incoming front-arc
  damage ×0.7 on top of angle armor, minigun spread halved. A real reason to
  stand still, and a real risk.
- **Step-up:** obstacles ≤0.4m are stepped over with a distinct clip, never
  vaulted or hopped.
- **No roll** — a walker tumbling reads wrong.

### 7.5 Rig, animation, and WEIGHT
Digitigrade legs are a 3-joint chain; no runtime IK in v1 — author the clips:
walk, turn-in-place L/R, braced side-step L/R, power stride, brace enter/exit,
weapon-fire additive, damage flinch, death.
- **Mass reads through timing, not scale.** Every mech action gets ~2.2× the
  anticipation and settle duration of the soldier equivalent, and the §2.5 hull
  spring runs at **k=22** (low stiffness = heavy).
- **Overlap and drag:** antenna, cables, and pod covers lag the hull by
  0.08–0.15s — secondary motion is what separates "heavy machine" from "big man."
- Root-motion vs physics displacement within **3cm/frame**.
- Servo/actuator audio on wind-up and landing of every step; a **0.2-intensity
  camera shake** for soldiers within 6m of a footfall.
- Interpenetration sweep covers every mech clip.

### 7.6 Entry / exit / idle life
- **Enter (1.6s, committed, no cancel):** chest plates part and swing open,
  pilot steps in, plates seal, visor sweeps on with ignition flicker + servo
  chorus. **Exit (1.2s):** reverse; powers down into a hunched dormant stance,
  visor dark.
- Occupied vs empty readable at a glance: visor lit + active stance vs dark +
  slumped. **No teleporting pilots.**
- **Alive idle:** visor scan-sweep every 7s, heat shimmer from vents, hull
  micro-sway ±0.5cm. The §1 statue test applies to the mech.

### 7.7 Damage model — armor that drops
Angle-based: incoming damage ×**0.15** front 120°, ×**0.30** sides, ×**1.00**
rear 90°. Visor weak point ×**2.0** applied *after* the angle multiplier (frontal
visor = 0.15×2.0 = 0.30 — rewarded, not dominant; the rear arc is the kill zone).
Fire/pyro ignores angle reduction. Mech HP **1000**.

**Plates are separate meshes and physically detach:**
- **70% HP:** shoulder-pod cover + left knee plate shear off as physics debris
  (despawn 20s); frame glows faint amber at the gaps.
- **40% HP:** side skirts + right knee plate drop; spark loop at a joint; servo
  whine pitch rises.
- **15% HP:** chest plate chips, visor flickers, gait gains a limp hitch
  (animation-only), smoke trail.
- Exposed under-frame takes **×1.25** after angle armor — stripping a mech is
  visible progress and the attacker sees it happen.
- Detach events run in the **SIM** layer (replay-identical); debris is cosmetic.

Soldiers' height-fraction hit zones do **not** apply to the mech; the angle model
replaces them.

### 7.8 Piloting and weapons
Context-interact to enter/exit (bound, on the controls screen, with a prompt).
First person = visor view: §3 grammar with the mech's arms/weapon as viewmodel
(offsets ×1.15), a subtle visor-frame vignette, same crosshair rules.
Third-person toggle (V) identical to §6. HUD swaps the ammo cluster to the
mounted weapon and adds a 4-segment missile indicator above it.

**Minigun:** spin-up 0.4s; heat **0→100% in 4s**; at 100% forced vent lockout
**3s** (barrel glow, vent steam, distinct audio); full cooldown from 99% in 6s.
Spread widens with heat (1.2° → 3.5° half-angle). ~1000 RPM, ~8 dmg/round vs
soldiers. Tracer every 3rd round; casings persist ≥10s. No scope. Movement 70%
while spun up.

**Heavy rifle (AWP-class):** **115 base**. Head ×4 (one-shot regardless of
armor), chest/arm ×1, stomach ×1.25, legs ×0.75 with no armor reduction (never
one-shots, 85). Vs mech, angle armor applies (front ≈17, rear 115, front visor
≈35 — the rifle deletes soldiers; flanks kill mechs). Zoom two levels FOV **40°**
then **10°**, 0.05s/step, viewmodel hidden while scoped, 0.3s scope-in blur,
auto-unscope per shot, re-zoom after bolt. Cycle **1.455s**; deploy 1.27s;
reload 3.7s (firing locked). Mag 5, reserve 10. Scoped move 50%. Inaccuracy:
scoped-standing 0.002, crouched 0.0015, unscoped standing 0.081, moving 0.176.
Recoil kick scoped 25, unscoped 78, angle variance ±20°.

**Targeted missile pod (lock-on):** valid targets **mechs, deployables/turrets,
marked structures ONLY** — no lock on infantry (dumb-fires straight against
soldiers; the anti-oppression rule, do not soften). Acquisition: dedicated input,
target within a 6° half-angle cone and 250m, **1.3s** to full lock; the victim is
warned from lock **start** (HUD flash + audio), not launch. Flight: proportional
navigation N=3, 60 m/s, 50 m/s², turn cap **250°/s**, TTL **7s**. LOS break
>0.4s → ballistic. Damage **270** before angle armor (rear ≈27% of mech HP;
frontal ≈4%). 4 tubes, 1.5s between launches, resupply at base/objective only.
Simulated in the deterministic layer with seeded streams.

### 7.9 Tests + captures — completion gate
Existence test (spawns via a reachable route, enterable, drivable, killable —
**write this one first**). Scale test (1.15× ±2%). Grounded test (10-min fuzz
drive, never exceeds 0.45m). Angle-armor test (8 compass directions + visor,
exact multipliers). Material audit green. Damage-state matrix (HP → 70/40/15%,
exact plates detach, multiplier zones active). Mobility tests: power-stride speed
and heat cost, brace-stance damage reduction, pivot cap, step-up ≤0.4m.
Entry/exit sweep. Statue test on the mech. Minigun heat curve golden test.
Heavy-rifle one-shot matrix. Scope-state test. Lock-on suite (cone, range, LOS
timing, warning at lock start, LOS break → ballistic, no-infantry-lock assert,
PN turn cap). Replay determinism on a recorded engagement.
**Captures:** entry clip, damage progression 100→15% under scripted fire,
power-stride and brace clips, concept-art side-by-side, front/side/rear/visor stills.

---

## SECTION 8 — THE FORGE (complete authoritative spec)

A modular **part-swap** system, not a sculptor. Scope stays honest.

### 8.1 Content (v1 counts)
- **Soldier:** helmet (5 incl. none), torso (4), arm/gauntlet (3), leg/greave
  (3), shield face (4). Per part: primary + secondary colour
  (faction-constrained palette), finish (matte/worn/polished), one decal slot
  (8 emblems), battle-wear slider 0–1 driving roughness + edge-wear.
- **Body:** height ±5%, build slider (slim↔heavy blend shape), skin tone (8),
  face preset (6 — visible only with the helmet off in preview).
- **Weapons:** stock/grip/sight cosmetic variants (3 each), finish (4), emissive
  accent colour — which also drives the §3.2 on-weapon ammo bar.
- **Mech:** plate colour, visor colour (red/cyan), decal, wear slider. **The
  §7.7 detachable plates ARE the customizable plates** — one system, two payoffs.
- **Gauntlet choice drives the §2 first-person hands** — this is where the hand
  craft pays off, and it must be visible in the FP preview.

### 8.2 The editor
Left: category list → part grid with thumbnails. Right: large turntable preview
(drag rotate, scroll zoom). **One click = one visible change.**
- **First-person preview toggle** — see your own hands/viewmodel exactly as in match.
- Randomize. Reset-to-default per part.
- **3 saved appearance slots** in a RON/TOML profile in the save directory.
  Reachable from the **main menu AND the lobby prep phase**.
- Data-driven: parts are manifest entries (id, display name, mesh path, attach
  socket, palette mask). A future part = one glTF + one manifest line, zero code.

### 8.3 In-match application
Applies to the third-person model, the first-person hands/viewmodel, and the
mech. **Faction-readability guard:** silhouette and team-colour regions are
locked; customization can never make an enemy read as a friendly.

### 8.4 Tests + captures
Save/load round-trip (every field). Random-combo interpenetration sweep (200
seeded outfits × idle/run/sprint/cut/thrust — including §1's new motion states).
Preview-equals-match (same profile, compare preview pose vs in-match screenshot
within tolerance). First-person test (gauntlet choice visibly changes in-match
hands). Part-swap <100ms. **Capture:** designing a soldier, then spawning into a
match wearing exactly that.

---

## SECTION 9 — HANDBACK FOR PLAYTEST

1. **The §0 table updated to "after"** — every row VISIBLE = yes, with its
   capture path. This table is the receipt.
2. Exact test commands and pass/fail output for every gate.
3. **The full capture set:** locomotion old-vs-new (sprint-start / stop / cut),
   hand craft clips, viewmodel rest/spray/scoped/sprint-carry, each HUD corner,
   spear full cycle + standing-vs-running, third-person walk→aim→scope handoff,
   mech entry + damage progression + mobility + concept side-by-side, Forge
   design-to-match.
4. **Feel questions, answered concretely:**
   - Does running read as an athlete or a slide?
   - Does the direction-cut read as controlled or as a skid?
   - Do the hands read as hands — is trigger-finger travel visible?
   - Does the whip-release feel fast, or still like a mechanical swing?
   - Does a running throw visibly go further than a standing one?
   - Does the weapon feel planted while spraying, or does anything drift?
   - Does third-person aim read as *focus* when the camera tightens?
   - Does the mech feel heavy — or just slow? (Different things.)
   - Does the damaged mech look wounded, or like a broken asset?
   - Is the Forge one click = one visible change?
5. **Every tunable introduced**, with current value and file location — so one
   feel note becomes a one-line change.
6. **Anything not done**, named plainly, with the reason (C9).

Do not mark §1, §2, §5, §6, §7, or §8 complete on tests alone. Motion, throw
feel, camera feel, mech weight, and Forge usability are judged by playing. The
playtest note that comes back is the real completion signal.

---

## APPENDIX A — Deliberately deferred (do not silently drop)

**The bow** (piercing-bow spec from the prior brief: hold-to-draw, 0.7s to full,
letdown under 0.15s, sway ramp at 4s/8s/10s, 55 m/s, pierce 3 targets at
90/68/45, quiver 12) is **not** in this brief's scope. It is parked, not
cancelled. Re-request it as its own brief once §1–§8 are visibly landed —
adding it mid-cycle would repeat the pattern of half-landed features this brief
exists to end.

---

*Source basis: CS:GO/CS2 shipped data (viewmodel, recoil, HUD anatomy); DICE
BF4 lock-on data; Sons of the Forest / Abyssus / CoD Black Ops 6 reference
gameplay; measured sprint biomechanics (trunk lean through acceleration, swing
knee flexion ~130°, contact-time/frequency relationships); measured javelin
biomechanics (runway speed and hip-shoulder separation correlate with release
speed; proximal-to-distal sequencing; firm block leg; 30–36° release);
human active joint ROM literature for §2 clamps. All numbers are starting
tunables, not final balance.*
