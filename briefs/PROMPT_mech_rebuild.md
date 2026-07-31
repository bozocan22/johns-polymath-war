# CLAUDE CODE PROMPT — Body Rebuild, Elastic Motion, and the Mech From Concept Art

*Copy everything below the line into Claude Code. Attach the mech concept art to
the same message if you can — if not, Task 1 reconstructs it from the written
spec.*

---

## MISSION

You are working on a Bevy/Rust game with first- and third-person combat. Three
things are broken and this session fixes all three, in order:

1. **The character rig cannot perform motions the design already requires.** It
   needs to be rebuilt as a 20-segment body. Four already-written features are
   silently blocked on this and cannot work until it is done.
2. **Explosive motion has no load phase**, so every powerful action reads
   robotic. Build the elastic load model.
3. **The mech in the code does not match the concept art** — wrong scale, wrong
   palette, wrong weapons. Rebuild it from the art.

Work the tasks in order. Do not stop between tasks to ask whether to continue.
If something is genuinely ambiguous enough that guessing wastes work, state the
question, then keep working on whatever is unblocked.

## OPERATING RULES

**R1 — Visible or it didn't happen.** Every task ends with a capture from *the
build I actually launch*, saved to `handback/<task>/`. Screenshots where a pose
is the deliverable, a 5–10s clip where motion is the deliverable. Tests passing
with no capture = task not complete. If no capture facility exists, build one
first — that is in scope.

**R2 — Ship enabled.** No feature flags defaulting off, no work left on unmerged
branches. Default config = intended experience.

**R3 — Two layers, declared.** Every system is commented as either **SIM**
(fixed timestep, seeded RNG, replay-identical — anything touching damage, hit
position, projectile flight, score) or **COSMETIC** (frame-rate driven, never
feeds back into sim state). A cosmetic system that writes a sim value is a bug
even if it looks right.

**R4 — Data, not constants.** Every number below goes in a hot-reloadable file:
`config/limbs.ron`, `config/motion.ron`, `config/locomotion.ron`,
`config/mech.ron`. Hardcoded magic numbers = failed review.

**R5 — Perceptual threshold.** If a value's effect cannot be seen in the
capture, it is not implemented — it is a rounding error. Raise it until visible,
then tune down.

**R6 — Report honestly.** Partial work is named as partial, with the reason.

---

## TASK 0 — AUDIT AND FIX DELIVERY (blocks everything else)

Before changing anything, run the game and report this table:

`FEATURE | CODED | TESTED | VISIBLE IN LAUNCHED BUILD | ROOT CAUSE | FIX`

Cover: character rig segment count, trunk segmentation, viewmodel placement and
no-bounce, sprint/turn/stop locomotion, spear thrust, spear throw, mech
existence/scale/materials.

Then answer with evidence from the **running process**, not from config files:
- Log at startup: git commit hash, build profile, and every animation system
  actually scheduled. Paste it in.
- **How many segments does the current character rig have, and is the trunk one
  bone or several?** This determines the size of Task 2.
- Capture a "before" clip of run → turn → stop, and of the current mech from
  front/side/rear.

**Root cause must name one of:** branch never merged / run config launches a
stale binary / flag defaulted off / system registered but never scheduled /
value below perceptual threshold / asset missing with silent fallback / code
orphaned and never called. "Unclear" is not an accepted answer — instrument
until it is clear.

**Gate:** if my launch command does not build current code, fix that first and
prove it with a before/after screenshot pair. Do not start Task 1 until this is
green.

---

## TASK 1 — GATHER AND COMMIT VISUAL REFERENCE

You likely have web access. Use it. Save everything into
`handback/reference/` and commit it — reference that lives only in a chat log
is lost work.

**1a — Mech reference.** The target design is a *military walking weapons
platform*, not a hero robot: olive-drab paint, reverse-joint legs, boxy hull, no
head, exposed knee and waist mechanism, a gatling on one arm and a drum-fed
autocannon on the other. Search and save 12–20 images using these terms:

- `John Park mech design` (closest match to the target's painterly military style)
- `Front Mission wanzer concept art` (closest genre match: utilitarian,
  drum-magazine, boxy military walkers)
- `Titanfall Legion titan` and `Titanfall Scorch` (chunky grounded proportions)
- `Chappie Blomkamp exosuit` and `District 9 mech` (real-world military plausibility)
- `reverse joint digitigrade mech legs concept`
- `military walker olive drab concept art`
- `drum magazine autocannon` and `gatling gun barrel cluster` (weapon detail)
- `hard surface panel line greeble reference`

**1b — Body/hand reference.** Save 8–12 more:
- `FPS viewmodel hands rifle grip screenshot`
- `hand rig topology finger joints game`
- `javelin thrower release hip shoulder separation photo`
- `sprint acceleration trunk lean photo sequence`

**1c — Write `handback/reference/NOTES.md`.** For each mech image extract: where
armour is smooth vs where mechanism is exposed, how many distinct masses the leg
reads as, where the paint is worn, how the weapon mounts to the arm. For each
body image: where the thumb wraps, how far the wrist deviates, where the elbow
points, how much the torso twists at wind-up.

**Design principle to apply throughout** (from hard-surface practice): greebles
must read as *engineering logic* — structural segmentation, maintenance access,
energy routing, mechanical connections — not as surface noise. Every panel line
should look like it opens, bolts, or routes something.

**Done when:** images committed, `NOTES.md` written. If web access is
unavailable, say so plainly and proceed — Tasks 2–5 are self-sufficient.

---

## TASK 2 — REBUILD THE BODY AS 20 SEGMENTS ★ do this first, it unblocks the rest

### Why
The design requires **hip-shoulder separation** (35–45° of twist between hips and
shoulders at a throw's wind-up). **With a single trunk bone this value is always
0° and the feature cannot exist.** Same for hips-leading-shoulders in a
direction cut, ±60° upper-body aim over turning legs, and torso coil as
anticipation. Four features are blocked on one missing bone.

### 2.1 The segments

| # | Segment | Count | Purpose |
|---|---|---|---|
| 1 | Head + neck | 1 | head-look layer |
| 2 | **Thorax** (upper trunk) | 1 | rotates with the shoulders |
| 3 | **Lumbar** (mid trunk) | 1 | **the twist segment — unblocks separation** |
| 4 | **Pelvis** (lower trunk) | 1 | rotates with the hips, drives the chain |
| 5–6 | Clavicle / shoulder girdle | 2 | the shoulder must travel, not only rotate |
| 7–8 | Upper arm | 2 | |
| 9–10 | Forearm + 2 twist bones each | 2 | twists prevent forearm candy-wrap |
| 11–12 | Hand | 2 | root of the finger sub-rig |
| 13–14 | Thigh | 2 | |
| 15–16 | Shank | 2 | the Achilles spring lives here (Task 3) |
| 17–18 | Foot (hindfoot) | 2 | |
| 19–20 | **Toe / forefoot** | 2 | enables the toe-off snap in the sprint cycle |

Fingers are a **sub-rig** on segments 11–12, not counted here — these 20 are the
mass-bearing, physics-relevant segments.

### 2.2 Mass fractions → `config/limbs.ron`
Published body-segment-parameter values. These drive ragdoll mass, follow-through
weight, and hit-reaction impulse.

```
head_neck   0.081
thorax      0.216      lumbar 0.139      pelvis 0.142     # trunk total 0.497
clavicle    0.005 each (carved from thorax)
upper_arm   0.028      forearm 0.016     hand 0.006       # arm total 0.050
thigh       0.100      shank  0.0465
hindfoot    0.011      toe    0.0035                      # foot total 0.0145
```
Closure check: `0.081 + 0.497 + 2(0.050) + 2(0.161) = 1.000`. Assert this in a test.

### 2.3 Segment lengths as a fraction of height H
```
upper_arm 0.186H   forearm 0.146H   hand 0.108H
thigh     0.245H   shank   0.246H   foot_length 0.152H
shoulder_width 0.259H   hip_width 0.191H   shoulder_height 0.818H
```
At H=1.8m: upper arm 33cm, forearm 26cm, thigh 44cm, shank 44cm, foot 27cm.
**Validate the existing rig against these** — proportions drifting from these
values are the usual reason a character reads as "off" without anyone being able
to say why.

### 2.4 Inertia → drives spring stiffness automatically
Centre of mass as a fraction of segment length from the **proximal** joint:
`upper_arm 0.436, forearm 0.430, hand 0.506, thigh 0.433, shank 0.433, foot 0.50`

Radius of gyration about the CoM, as a fraction of segment length:
`upper_arm 0.322, forearm 0.303, hand 0.297, thigh 0.323, shank 0.302, foot 0.475`

**Derive spring stiffness per segment from `m·(k·L)²` instead of hand-tuning it.**
This removes most of the "why does this limb feel wrong" iteration loop.

### 2.5 Joint limits — clamp every procedural pose
```
shoulder  flex/ext -60..170   abduction 0..170   rotation -80..90
elbow     flexion  0..145     hyperextension hard-clamped at -5
forearm   pronation/supination -85..85   (distributed across twist bones)
wrist     flex/ext -70..80    radial/ulnar -20..30
MCP       flexion 0..90 (ext -25)   spread ±15
PIP       flexion 0..110 (pinky 135)
DIP       flexion 0..80
thumb     CMC opposition 0..45   MCP 0..55   IP 0..80
```
**Coupling rules (cheap realism):** `DIP = 0.7 × PIP` (tendon-linked — independent
DIP motion looks robotic); curl drives spread toward 0; ring and pinky curl
together at 0.85 coupling; phalanx ratio proximal:middle:distal = 1.0 : 0.62 : 0.42.

### 2.6 Tests
- **Segment-count test:** all 20 named segments exist and are addressable.
- **Separation test:** drive a throw wind-up; assert `yaw(thorax) − yaw(pelvis)`
  reaches 35–45°. *This test failing before the rebuild and passing after is the
  proof the fix landed — run it both ways and show me both results.*
- **Proportion test:** every segment length within ±5% of its 2.3 fraction.
- **Mass-closure test:** segment masses sum to 1.000 ± 0.001.
- **Toe-off test:** in the sprint cycle the toe segment rotates through
  plantar-flexion at contact exit. No toe rotation = the run is still a glide.
- **Joint-limit fuzz:** 10,000 seeded random pose targets, no joint exceeds its
  clamp, no NaN, no flip.

**Capture:** third-person clip of a throw wind-up showing the torso twisting
between hips and shoulders, and a sprint clip showing toe-off.

---

## TASK 3 — THE ELASTIC LOAD MODEL ("Achilles motion", literally)

### Why
The Achilles tendon is the mechanical reason human movement looks explosive.
Tendons return **90–95% of stored elastic energy**, scaling from ~1.3 J per
walking step to ~38 J in a one-leg jump, via the **stretch-shortening cycle**:
pre-activation → loaded stretch (eccentric) → explosive shortening (concentric).

**Translation to animation:** explosive motion is never muscle firing from rest.
It is *load, then release*, and the release is faster than the load that produced
it. Motion that skips the load phase reads robotic no matter how fast the
release is.

### 3.1 Build one shared utility
```rust
// COSMETIC drives visuals; the velocity bonus is SIM — keep them separate.
struct ElasticMove {
    load_s: f32,              // eccentric coil / counter-movement
    release_s: f32,           // concentric whip
    stored_energy: f32,       // 0..1, accumulates during load
    return_efficiency: f32,   // 0.92 human, 0.55 mech (steel is worse than tendon)
}
```

### 3.2 The rules
1. **Release is 2–3× faster than load.** Wind-up 0.4s → release 0.15–0.20s. A
   release slower than its wind-up reads as a shove, never a strike.
2. **Stored energy scales output:** `release_velocity = base × (1 + stored_energy × 0.35)`.
   A fully loaded move is measurably stronger — and visibly so, because the
   wind-up was longer.
3. **A counter-movement beats a dead start.** Any move preceded by motion in the
   opposite direction gets the bonus. Apply this to the **jump, dodge launch,
   melee thrust, and spear throw** — not just the throw.
4. **Landings recharge.** A landing that flows into the next action carries
   stored energy forward; stopping first loses it. This is the mechanical core of
   feeling like an athlete instead of a state machine.
5. **Never fully damp a landing.** Soften touchdown over 2–3 frames with ~8% of
   impact velocity returned upward. Never clamp to zero in one frame.

### 3.3 Pair it with proximal-to-distal sequencing
Build **one** kinetic-chain utility and route every power move through it:
```
segments:      [Pelvis, Lumbar, Thorax, Clavicle, UpperArm, Forearm, Hand, Tip]
onset_offsets: [0.000,  0.020,  0.035,  0.055,    0.065,    0.090,   0.110, 0.125]
peak_scale:    [1.00,   1.08,   1.15,   1.25,     1.35,     1.60,    1.85,  2.10]
deceleration_gate: true   // each segment's peak triggers the next segment's onset
```
Consumers: spear throw release, spear thrust, dodge launch, sprint start, mech
side-step (offsets ×2.2). One mechanism, one test covering all of them.

### 3.4 Tests
- **Load-release ratio:** every `ElasticMove` satisfies `release_s ≤ load_s / 2`.
- **SSC bonus:** same move from a dead stop vs out of a counter-movement; the
  output difference matches `stored_energy × 0.35`.
- **Landing rebound:** no landing reaches exactly zero vertical velocity in one frame.
- **Chain timing:** angular-velocity peaks occur in strict order
  pelvis → lumbar → thorax → shoulder → elbow → wrist → tip, with minimum offsets.
  A failure must name which segment fired early.

**Capture:** side-by-side of a flat-footed action and the same action performed
out of a flow. Per R5, if the difference isn't visible in the clip, it isn't done.

---

## TASK 4 — MECH SCALE DECISION (answer before rebuilding)

Measured from the concept art with the soldier at 1.8m: the hull top sits near
**4.5m**, antenna near 5.2m, and the soldier's helmet reaches only the mech's
**knee**. That is **≈2.5× soldier height**. The current spec says 1.15×, which
would be a machine barely a head taller than a man — throwing away everything the
art sells (walking under it, sheltering behind its legs).

Pick one, write it into `config/mech.ron`, and state the choice in your report:

| Option | Height | Gain | Cost |
|---|---|---|---|
| A1 — honour the art | 2.5× (4.5m) | Full presence; legs become cover | Needs bespoke nav width, doorway rules, cover system |
| **A3 — recommended** | **1.7× (3.06m)** | Keeps the tower read and leg-cover fantasy | Widened nav radius only, no bespoke system |
| A2 — current spec | 1.15× (2.07m) | Shares all soldier nav and cover | Looks nothing like the art |

**Whatever is chosen, scale these with it** (do not leave at soldier values):
footfall camera-shake radius, step-up height, side-step distance, visor height,
viewmodel offsets, interpenetration clearance margins.

---

## TASK 5 — REBUILD THE MECH FROM THE ART

### 5.1 Overall read
A **walking weapons platform** — a light armoured vehicle on legs, not a humanoid
robot. Hull cantilevered forward over reverse-joint legs, arms mounted **low on
the hull sides** (not shoulder-top), no head, no cockpit glass. Utilitarian: every
shape looks like it houses something.

**Stance:** hull pitched slightly nose-down, hips high and set back, knees
carrying weight forward — leaning into its own mass. A level upright hull loses
the silhouette entirely.

### 5.2 Palette — replaces the current gunmetal spec
The art is **olive drab / khaki / field tan**, not gray.
```
hull_primary   #8A8770   olive-khaki, matte, roughness 0.72
hull_shadow    #5F5E52   recessed panels
mechanism_dark #33352F   joints, actuators, cabling
barrel_metal   #2B2C2B   roughness 0.45
wear_metal     #9A9384   edge chips ONLY at plate borders and foot cleats
marking_white  #D8D4C6   stencils (the "3" on the hull)
```
**Emissive is minimal** — the art has no glowing visor. One sensor-lens glint and
per-tube status lights. If team colour is needed, a thin lens line, never a lightbar.

### 5.3 The 20 parts (these double as damage-detach plates and Forge swap parts)

**Hull (5):** main hull box (flat angled top, chamfered front, stencil panel) ·
front sensor plate (dark, recessed, inset) · upper rear pod pair (two horizontal
cylinders) · antenna mast (thin whip right + short ball-tipped stalk left) ·
dorsal hatch/vent plate

**Arms (6):** left shoulder housing · **gatling arm** (4–6 barrels in a sleeve,
muzzle ring) · gatling feed housing · right shoulder housing · **autocannon arm**
(long barrel, muzzle device, angled down) · **drum magazine** (the big cylinder
on top — the most recognisable shape on the machine; give it a seam, a latch, and
visible rotation on firing)

**Hip (3):** waist actuator block (the busiest area — pistons, linkages, exposed
cable) · hip yaw ring · hip armour skirts (small; must not hide the mechanism)

**Legs ×2 (6):** thigh plate (largest flat surface, dominant read) · knee
mechanism (**exposed**, deliberately uncovered) · shin plate · ankle actuator
cluster · foot pad (wide, flat, splayed) · **foot cleats** (tread blocks on the
sole — do not smooth these out)

### 5.4 Proportion and detail rules
- Hull footprint is **wider than tall** — a slab, not a tower.
- The thigh plate is the largest flat surface: its silhouette and one strong panel
  line matter more than its texture.
- **Knee and waist mechanisms stay exposed.** The contrast between smooth armour
  and dense mechanical clutter is what makes it read as real machinery. Covering
  them kills the design.
- Feet wide and flat with a rear spur — not claws, not blades.
- Every panel line must look like it opens, bolts, or routes something (Task 1's
  engineering-logic principle).

### 5.5 Weapons — replaces the missile-pod spec
The art shows **no missile pod**: it carries a gatling and a drum-fed autocannon.
Ship that as the default; make the missile pod an **optional swappable hardpoint,
absent by default**.

**Autocannon:** 15-round drum, ~200 RPM burst, 45 damage/round vs soldiers,
visible drum rotation per shot, heavy shell ejection, strong muzzle flash, and a
**hull-shove recoil that visibly rocks the machine back on its hips** — mass reads
through recoil far more than through texture.

### 5.6 Damage states mapped to real parts
- **70% HP:** hip skirts + left thigh plate shear off → exposes the waist actuator
  block (already the most mechanical area, so the exposure reads instantly).
- **40% HP:** shin plate + one rear hull drum drop; antenna bends or snaps; sparks
  at the knee.
- **15% HP:** drum casing chips and stops rotating smoothly; sensor plate cracks;
  a foot cleat row tears away producing a visible limp; smoke from the waist block.
- Exposed mechanism takes **×1.25** damage after angle armour. Detach events run
  in the SIM layer (replay-identical); debris is cosmetic.

### 5.7 Tests
- **Material audit:** iterate every submesh; assert none uses a
  default/placeholder material and every material has non-empty texture slots.
  Untextured gray becomes a CI failure, not an opinion.
- **Palette audit:** sample rendered pixels against the 5.2 values within tolerance.
- **Part-count test:** all 20 parts exist as separate meshes (this is what makes
  both damage-detach and customization work).
- **Damage matrix:** drive HP to 70/40/15% and assert the exact parts detach and
  the multiplier zones activate.
- **Scale test:** chosen ratio ±2%.

**Capture — this is the completion criterion:** render the mech at the chosen
scale beside a 1.8m soldier at the concept art's ¾ angle, and place it next to the
concept art in the handback. If the shapes don't match, the task is not done. Plus
front/side/rear/underside stills and a firing clip showing drum rotation and hull
recoil shove.

---

## TASK 6 — REPORT

Produce `handback/REPORT.md` containing:

1. **The Task 0 table updated to "after"** — every row VISIBLE = yes, with its
   capture path.
2. **The separation test, before and after** the rig rebuild — the single clearest
   proof the body work landed.
3. **Exact test commands** and current pass/fail output for every test above.
4. **The scale decision** you took in Task 4 and why.
5. **The mech side-by-side** against the concept art.
6. **Every tunable introduced**, with its current value and file location, so one
   feel note becomes a one-line change.
7. **Answers to these, in plain language:**
   - Does the torso visibly twist between hips and shoulders at wind-up?
   - Does a loaded action visibly out-perform a flat-footed one?
   - Does the run read as drive, or still as a glide?
   - Does the mech read as a *machine* — do the exposed mechanisms sell it?
   - Does the autocannon's recoil make the mech feel heavy?
8. **Anything not done**, named plainly, with the reason.

Do not mark Tasks 2, 3, or 5 complete on tests alone. Motion and mass are judged
by watching — the captures exist so I can judge without launching a debugger.
