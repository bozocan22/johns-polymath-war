# BRIEF VII (v2, optimized) — Living Motion, Limb & Hand Craft, Spear Throw, Bow, Third-Person Camera, Mech Overhaul, and the Forge

**How to run this:** paste from `OPERATING CONTRACT` to the end into Claude Code.
Work sections in order. Do not stop between sections to ask permission to
continue. The three reference videos are already translated into written specs —
do not attempt to watch them.

---

## OPERATING CONTRACT (read once, obey throughout)

These rules exist because the last session produced green tests and an
unchanged-looking game. They outrank convenience everywhere.

**C1 — Visible or it didn't happen.** Every section ends with captures from
**the build the player actually launches**, written to
`handback/brief-vii/<section>/`. Screenshots where a pose is the deliverable;
a 5–10s clip wherever *motion* is the deliverable. A section with passing
tests and no captures is NOT complete. If no capture facility exists, building
one is in scope and is the first thing you build in Section 0.

**C2 — Ship enabled.** No feature flags defaulting off. No work parked on an
unmerged branch. Default config = the intended experience. If the player's run
command launches a stale or different binary, fixing that is Section 0's first
deliverable and blocks everything else.

**C3 — Two-layer motion law.** Every motion feature is classified and lives in
exactly one layer:
- **SIM layer** — deterministic, fixed timestep, seeded RNG, replay-identical.
  Anything that touches damage, hit position, projectile flight, plate
  detachment, or score.
- **COSMETIC layer** — frame-rate-driven, may use wall-clock, never feeds back
  into sim state. Breathing, fidgets, look-at, flinch, camera spring, viewmodel
  sway, debris.
State the layer in a comment above every new system. A cosmetic system that
writes a sim value is a bug even if it looks right.

**C4 — Tunables are data.** Every constant in this brief is a *starting value*.
Each lands in a versioned data file, hot-reloadable where feasible:
```
config/motion.ron      # breathing, fidget, flinch, spring constants
config/limbs.ron       # arm/hand/finger rig limits + grip poses
config/weapons/*.ron   # per-weapon: spear, bow, guns
config/camera.ron      # third-person rig states
config/mech.ron        # damage states, materials, entry/exit
config/forge/*.ron     # part manifests
```
Hardcoded magic numbers in systems code = failed review.

**C5 — Additive budget.** Cosmetic layers compose additively over locomotion in
this fixed order, each clamped to its own budget:
`locomotion → posture state → breathing → weight shift → look-at → fidget →
reaction/flinch → aim override`
Total additive translation on the **first-person viewmodel** must never exceed
the Brief IV breathing amplitude (this is the Brief VI no-bounce rule, kept
absolute). Third-person body has no such translation cap, only per-layer clamps.

**C6 — Player intent wins.** While the player is aiming or firing, all reactive
layers attenuate ×0.3. Nothing the world does may fight the player's aim.

**C7 — Report honestly.** If a section is partially done, say which part and
why, in the handback. Do not mark motion sections complete on tests alone.

### Reconciliation with Brief VI — nothing here breaks it

| Brief VI rule | Status under this brief |
|---|---|
| No ADS ever; gun never rises to face (VI §1.1 R1) | **Absolute, all guns, first person.** Spear raise (§3) and bow draw (§4) are throw/draw grammar with their own screen profiles, not ADS. Third-person aim (§5) is camera-only, no viewmodel. |
| No viewmodel bounce; standing = frozen bob (VI §1.3) | **Absolute.** Living motion (§1–2) adds rotational and finger-pose life only; translation stays inside the Brief IV envelope. |
| Scoped weapons hide the viewmodel (VI §1.1 R2) | Unchanged, and reused: third-person scoping hands off to the same overlay. |
| Screen-intrusion test (VI §1.4) | Extended with per-weapon allowance profiles (`spear_raised`, `bow_drawn`). The strict gun profile must keep passing. |
| Brief V grenade trajectory arc | **Grenade-only.** Spear and bow get NO arc and NO landing marker — the learnable arc IS the skill. |
| Brief V spear thrust / dodge weight | Stands. §3 adds the throw on top of the thrust. |
| Brief VI mech spec (1.15×, grounded, angle armor, weapons) | Stands. §6 is a visual/feel overhaul plus damage states on top. |

---

## SECTION 0 — AUDIT: WHY DID NOTHING VISIBLY CHANGE?

**Read code and run the game before changing anything.**

### 0.1 The evidence table
Produce this for every headline feature of Briefs V and VI — spear thrust,
dodge weight, grenade arc, viewmodel placement, no-bounce, seeded recoil, HUD
corners, mech existence, mech weapons:

`FEATURE | SPEC'D | CODED | TESTED | VISIBLE IN LAUNCHED BUILD | ROOT CAUSE | FIX`

Root cause must name one of: *branch never merged / run config launches stale
or different binary / feature flag defaulted off / system registered but never
scheduled / values below perceptual threshold / asset missing so silent
fallback / code orphaned, never called.* "Unclear" is not an accepted answer —
instrument until it is clear.

### 0.2 Runtime truth, not config truth
Log at startup, and paste into the handback:
- viewmodel FOV, position offset, rotation offset **as read at runtime**
- the git commit hash and build profile the running binary reports
- the full list of registered animation systems/layers actually scheduled
Reading the config file is not evidence. The running process is evidence.

### 0.3 Per-area probes
1. **Brief VI landing.** Do the Brief VI tests exist and pass *right now*?
   Capture the "before" set: rest / walking / firing / scoped.
2. **Brief V spear.** Do thrust states exist and are they bound to a reachable
   input? Capture a clip of whatever currently happens when you press it.
3. **Animation inventory.** What runs on an idle character this second? List
   every active layer. If the answer is "nothing," write that plainly — it is
   finding #1 for §1.
4. **Mech visuals.** Which materials are actually bound to each mech submesh?
   Placeholder gray counts as none. Screenshots front / side / rear.
5. **Entry/exit.** Does entering the mech play any animation, or teleport?

### 0.4 Fix delivery FIRST — hard gate
Before any new feature work: the Brief VI viewmodel must be **visibly on
screen in the launched build**, proven by a before/after screenshot pair in
`handback/brief-vii/section-0/`. If the run configuration is the culprit, fix
the run configuration and document the change. **Do not proceed to §1 until
this gate is green.**

**Done when:** table complete with root causes, runtime values logged, run
config provably launches current code, before/after viewmodel captures exist.

---

## SECTION 1 — LIVING MOTION LAYER (the "emotion")

**Goal:** no character is ever a statue. Soldiers breathe, shift, look, react.
Faces stay helmeted — the feeling comes from posture and micro-motion.

### 1.1 Always-on idle micro-motion (both rigs)
- **Breathing** — chest/shoulder rise. 12 cycles/min calm; ramps to 30/min
  after sprinting, decays back over 8s. Drives an additive layer of ±0.5cm at
  the shoulder. Mech equivalent: hull micro-sway ±0.5cm + vent cycling.
- **Weight shift** — hip/foot weight transfer every 6–12s, seeded RNG,
  deterministic selection, cosmetic playback.
- **Head look** — idle characters glance toward the nearest moving entity or
  last loud sound every ~4s; ±25° yaw max, 0.3s ease-in-out. Suppressed while
  aiming.
- **Grip fidget** — re-grip / finger adjustment on the held weapon every
  8–15s. Third-person AND first-person hands. In first person this is a
  **rotational and finger-pose flourish only** — zero added translation
  (see §2 for the finger mechanics that make this read).

### 1.2 Posture states
- **Relaxed** (10s no combat): weapon lowered ~8°, wider stance, shoulders down.
- **Alert** (combat): current stance.
- **Suppressed** (projectile within 2m): involuntary flinch/duck, intensity
  0.2, 0.3s decay. ×0.3 while the player is firing (C6).
- **Low HP (<25%)**: posture sag, labored breathing (audio + amplitude up),
  heavier footfalls. **No movement-speed change** — the HUD already signals it.

### 1.3 Reactions (all additive, all attenuated per C6)
- Explosion within 8m → shield-eyes flinch, 0.4s.
- Ally death within 6m → head snap toward them, 0.5s.
- Kill confirmed → subtle exhale + re-grip. No long taunts.

### 1.4 Tests + captures — completion gate
- **Statue test (automated).** Record 30s of a stationary character; assert
  (a) the third-person skeleton never has all-joint velocity ≈ 0 for >2s
  continuously, and (b) first-person viewmodel translation stays inside the
  Brief IV breathing envelope — proving life was added *without* bounce.
- Scripted trigger test per reaction: spawn stimulus, assert the layer fires
  and decays on schedule.
- Breathing-rate golden curve: sprint 10s → sampled rate over the next 20s
  matches the curve within tolerance.
- **Captures:** 20s idle clip, 20s combat clip.

---

## SECTION 2 — ARMS, JOINTS, AND HANDS: THE CRAFT PASS ★ NEW

**Why this section exists:** hands and arms are what the player sees most in
first person and what makes a third-person soldier read as a person instead of
a mannequin. This is the deepest-craft section in the brief. Numbers below come
from human joint biomechanics — use them as clamps so procedural motion can
never produce a broken-looking limb.

### 2.0 Reference gathering (do this first, ~20 min)
You likely have web access; the brief author did not have a way to paste images
into this repo. So **gather your own reference and commit it**:
- Search and save 10–15 stills into `handback/brief-vii/section-2/reference/`
  using these terms: *"FPS viewmodel hands rifle grip screenshot"*, *"hand rig
  topology finger joints game"*, *"tactical glove gauntlet close-up"*,
  *"hand anatomy MCP PIP DIP joints diagram"*, *"two-bone IK elbow pole vector"*.
- Write `reference/NOTES.md` extracting, for each image: where the thumb wraps,
  how many fingers actually contact the grip, how far the wrist deviates, where
  the elbow points, and how the glove's material breaks light at the knuckles.
- If web access is unavailable, say so in the handback and proceed from the
  numeric spec below — it is self-sufficient.

### 2.1 Bone hierarchy (canonical, both rigs, mirrored L/R)
```
clavicle → upper_arm → forearm → [forearm_twist_01, forearm_twist_02] → hand
hand → thumb_01 → thumb_02 → thumb_03
hand → {index,middle,ring,pinky}_meta → _01 → _02 → _03
```
- **Twist bones are mandatory.** Two forearm twists distributing 0%/50%/100%
  of wrist roll, plus one upper-arm twist taking 50% of shoulder roll. Without
  them the forearm candy-wraps on any grip pose — the single most common
  "cheap-looking hands" failure.
- **Metacarpals are mandatory** for index and pinky at minimum: the palm must
  be able to cup. A flat palm reads as a mitten.

### 2.2 Joint limits (clamp every procedural pose to these)
Sourced from measured human active range of motion; store in `config/limbs.ron`.

| Joint | Axis | Range |
|---|---|---|
| Shoulder | flex/ext | −60° … +170° |
| Shoulder | abduction | 0° … +170° |
| Shoulder | rotation | −80° … +90° |
| Elbow | flexion | **0° … +145°**, hyperextension hard-clamped at −5° |
| Forearm | pronation/supination | −85° … +85° (spread across twist bones) |
| Wrist | flex/ext | −70° … +80° |
| Wrist | radial/ulnar dev | −20° … +30° |
| Finger MCP | flexion | 0° … **+90°** (ext to −25°) |
| Finger PIP | flexion | 0° … **+110°** (little finger to +135°) |
| Finger DIP | flexion | 0° … **+80°** |
| Finger MCP | spread | ±15° (index/pinky more, middle least) |
| Thumb CMC | opposition | 0° … +45° |
| Thumb MCP/IP | flexion | 0° … +55° / 0° … +80° |

**Coupling rules that sell realism cheaply:**
- DIP flexion ≈ **0.7 × PIP flexion** (they are tendon-linked; independent DIP
  motion looks robotic).
- Curling a finger reduces its spread toward 0 (fingers converge in a fist).
- Ring and pinky curl **together** at 0.85 coupling — humans cannot isolate them.
- Phalanx length ratio proximal : middle : distal ≈ **1.0 : 0.62 : 0.42**.

### 2.3 Arm solving
- **Two-bone IK + pole vector** for both arms. Root = shoulder, mid = elbow,
  tip = hand; the pole target sets the elbow plane.
- **Pole placement rule:** elbow points *down-and-outward* at 15–25° from the
  vertical plane through the arm for a weapon grip — never straight down (limp)
  and never flared 90° out (chicken-wing). Expose `elbow_pole_offset` per
  weapon in `config/limbs.ron`.
- **Soft-clamp near full extension:** blend the last 5% of reach so the arm
  eases into straight instead of snapping — hard IK pops read as broken.
- **Left hand is IK'd to a socket on the weapon**, never animated free. Move
  the weapon and the support hand follows automatically; this is what keeps
  the grip glued during recoil, fidgets, and reloads.

### 2.4 Grip pose library (`config/limbs.ron`)
Author one named finger pose per interaction, as MCP/PIP/DIP triples per digit
plus a wrist offset. Minimum set: `rifle_primary`, `rifle_support`,
`pistol`, `spear_shaft`, `spear_overhead`, `bow_riser`, `bow_string_draw`
(three-finger Mediterranean draw — index above, middle+ring below), `shield_grip`,
`open_relaxed`, `climb`.
- Blend between poses over 0.12–0.18s. Never snap.
- **Trigger finger is independent**: it leaves the grip pose, travels to the
  trigger over 0.06s on fire, returns over 0.10s. This single detail does more
  for "the hands are alive" than any other.

### 2.5 Procedural secondary motion (physics of the limb)
Every cosmetic offset runs through a **critically-damped spring** so nothing
teleports:
```
x'' = -k*(x - target) - c*x'      c = 2*sqrt(k)   (critical damping)
```
Starting constants (`config/motion.ron`):

| Element | k | Notes |
|---|---|---|
| Hand follow (weapon lag) | 120 | Slight trail behind the aim; the "weight" cue |
| Elbow pole | 60 | Lazier than the hand — elbows settle late |
| Finger settle | 220 | Snappy; fingers are light |
| Shoulder/clavicle | 45 | Heaviest, slowest |
| Camera boom (§5) | 90 | |

- **Follow-through:** on any fast rotation (turn > 180°/s, fire, throw), the
  hand target lags the goal by `0.035s` and overshoots ≤ 3° on arrival.
  Under-damp the *return* only (ζ ≈ 0.7) so it settles with one small
  overshoot — a perfectly critically damped return reads dead.
- **Inertia coupling:** during sprint and dodge, arm targets receive
  acceleration × `0.02` as a positional offset, clamped to 4cm. Applies to the
  third-person body always; to the first-person viewmodel **only as rotation**
  (C5 keeps translation frozen).
- **Determinism:** these are all COSMETIC (C3). Never let a spring output feed
  a hit position.

### 2.6 Graphic design of the arm and hand (art direction)
- **Silhouette first:** gauntlet plates must break the forearm into three
  readable masses (wrist cuff, mid-plate, elbow cop). A tube reads as a mitten
  at 30m.
- **Knuckle geometry:** model actual knuckle bumps; do not rely on normal maps
  alone — the knuckle line catches rim light and is the main "living hand" cue
  in first person.
- **Materials (full PBR):** glove leather roughness 0.65–0.85 with a fine
  grain normal; armor plate roughness 0.35–0.55 metallic 1.0; **edge wear
  masked to plate borders and knuckles only** — uniform wear reads as noise.
  Palm slightly glossier than the back of the hand (sweat/oil) — subtle but
  it makes the palm read as skin-under-glove.
- **Seams and stitching** on the glove at finger bases; a wrist strap with a
  small buckle. Tiny asymmetries (one strap looser) kill the CG-perfect look.
- **Color:** the gauntlet carries the faction secondary color; the glove stays
  neutral. This is also the §7 Forge customization surface.
- **Poly budget guidance:** hands ~6–9k tris each for first-person viewmodel,
  ~2k for third-person LOD0. Loops: 3 edge loops per finger joint (bend, and
  one either side) — 2 loops crease badly at 110° PIP flexion.

### 2.7 Tests + captures — completion gate
- **Joint-limit fuzz:** drive 10,000 seeded random pose targets through the
  solver; assert no joint ever exceeds its §2.2 clamp and no NaN/flip occurs.
- **Coupling test:** assert DIP ≈ 0.7 × PIP and ring/pinky coupling hold across
  the full curl range.
- **Grip-attachment test:** for each grip pose, assert every contacting
  fingertip is within 8mm of the weapon's grip surface, and no finger bone
  penetrates the weapon mesh.
- **Twist distribution test:** roll the wrist 85°; assert forearm twist bones
  carry 50%/100% and no vertex on the forearm exceeds a candy-wrap threshold.
- **No-bounce regression:** the Brief VI viewmodel translation test still passes
  with all §2 systems live.
- **Captures:** first-person close-up clip of (a) idle finger fidget,
  (b) trigger-finger travel on fire, (c) a reload showing the support hand
  IK staying glued, (d) third-person arm swing at sprint. Plus a still of the
  hand at 110° curl for silhouette review.

---

## SECTION 3 — SPEAR THROW (Reference A: *Sons of the Forest* grammar)

**Goal:** hold to raise the spear overhead, release to throw with the whole
body, learnable arc, spear sticks in and is retrievable.

### 3.1 The throw cycle
- **Raise** (hold aim input with spear equipped): 0.4s — arm cocks the spear
  overhead beside the head, torso coils, off-hand points forward at the target.
  Move speed 80%. Camera FOV −4° (camera-only focus cue). This raised pose is
  **allowed** to bring the spear high on screen: it is wind-up, not ADS.
- **Aim:** hold indefinitely. **No trajectory arc. No landing marker.** The arc
  is learned — that is the entire point of Reference A. Reticle: small dot.
- **Throw** (release): 0.25s full-body throw obeying Brief V ordering —
  **hips drive → shoulder → arm → release**, with visible follow-through.
  Launch 22 m/s along the camera ray, full gravity, in the deterministic sim.
- **Cancel:** the Brief V cancel input lowers the spear without throwing.
- **Melee:** the Brief V thrust stays on the primary input, untouched.

### 3.2 Impact, stick, retrieval
- Impact angle ≥30° to the surface → **embeds**, quivering 0.4s (spring from
  §2.5, cosmetic). Shallower → bounces off with a clatter.
- Sticks in bodies; the corpse carries it.
- **Damage:** 85 body, ×2 head (170 — a lethal skill shot), ×0.75 legs. Against
  the mech, angle armor applies as usual — a spear should not threaten a mech
  frontally, and that is correct.
- **Retrieval:** interact on a stuck spear returns it to the loadout. Carry max
  2. Thrown spears persist ≥60s.
- Mech does not throw spears. Human rig only.

### 3.3 Screen profile
New `spear_raised` profile: shaft may cross top-center; hand/grip stays right
of the vertical midline; nothing enters the lower-center reticle zone. Guns keep
the strict Brief VI profile unchanged.

### 3.4 Tests + captures — completion gate
Deterministic flight test (5 angle/distance combos vs golden file, Brief V
standard). Stick-vs-bounce threshold test at 25°/30°/35°. Retrieval round-trip
(throw → stick → retrieve → count restored). `spear_raised` screen profile
passes. Replay reproduces a throw bit-identically.
**Capture:** one clip of raise → aim → throw → stick → retrieve.

---

## SECTION 4 — BOW (Reference B: *Abyssus* piercing-bow grammar)

**Goal:** a fast, punchy FPS bow — not an archery simulator.

### 4.1 Draw cycle
- **Draw** (hold fire): full draw in 0.7s. Release before 0.15s = **letdown**,
  no shot. Power maps draw fraction linearly: 35% → 100% of velocity and damage.
- **Full-draw hold:** steady 4s; then rotational aim sway ramps ±0.4° → ±1.2°
  over the next 4s (camera-space, not viewmodel translation); forced letdown at
  10s total. Crouching halves sway.
- **Release:** string snap, small **rotational** kick through the Brief VI
  recoil channels, arrow away with a thin tracer trail. Re-nock 0.6s.
- **Cancel:** reload key = letdown without firing.
- Camera FOV −5° at full draw only. The bow does **not** rise to the face.
- **Crosshair:** none below 50% draw; dot fades in at 50%, sharpens at full
  draw. Full draws are rewarded with precision.
- Hands: `bow_riser` grip left, `bow_string_draw` three-finger right (§2.4);
  the string-hand anchor point sits at the jaw and must stay glued through sway.

### 4.2 The arrow
- Full draw: 55 m/s, gravity, deterministic seeded sim, replay-exact.
- **Piercing:** passes through up to **3** soldiers — damage 90 → 68 → 45
  (×0.75 per pierce), headshot ×2 at each pierce. Lining up enemies is the
  fantasy; reward it.
- Against the mech: arrow stops (no pierce), angle armor applies.
- Arrows stick visually in surfaces and bodies. No retrieval in v1 (note as
  future work). Quiver 12; resupply at base/crates.
- Movement 85% while drawing; jump-draw allowed with Brief VI §2.4 inaccuracy.

### 4.3 Tests + captures — completion gate
Draw-power golden curve (`power(t)` exact). Pierce test: 3 aligned dummies +
1 behind → hits 3 for 90/68/45, fourth untouched. Letdown-no-fire test. Sway
envelope telemetry at the 4s/8s/10s marks. Replay determinism. `bow_drawn`
screen profile (limbs left/below, string may approach center, grip never
crosses the midline inward).
**Capture:** clip of draw → hold → sway onset → release → triple pierce.

---

## SECTION 5 — THIRD-PERSON CAMERA + POSITIONING (Reference C: BO6 grammar)

### 5.1 Camera rig (all tunable, `config/camera.ron`)
- **Hip:** boom 2.2m back, +0.45m right of head, +0.12m up. FOV = world FOV.
- **Sprint:** boom eases to 2.5m with 0.12s positional lag.
- **Aim** (hold RMB in third person — camera state only, legal under Brief VI):
  boom 1.35m, +0.55m right, FOV −12°, 0.15s transition. The character raises the
  weapon to a proper shoulder mount on the **third-person model** — no viewmodel
  is involved, so no Brief VI conflict.
- **Scoped weapons:** aiming the heavy rifle from third person hands off to the
  Brief VI full-screen scope overlay. Unscoping returns to third person.
- **Shoulder swap** input, bindable, shown on the controls screen.
- **Camera collision:** spring against geometry (k = 90, §2.5); never clips,
  never pops through walls.
- Crosshair stays screen-center; hip fire uses the same Brief VI §2.4
  inaccuracy states — third person is a camera choice, never an accuracy buff.

### 5.2 Character positioning (what makes Reference C read well)
- **Aiming:** character faces the aim direction; 8-way strafe locomotion blend;
  upper-body additive aim to **±60°** before legs turn-in-place catches up.
- **Not aiming:** character faces velocity.
- Start/stop get 2–3 frame lean anticipation and a foot plant on stop. Brief V's
  no-instant-velocity invariant applies to every new blend.
- Toggle stays on **V** (Brief III). Works on foot and in the mech.

### 5.3 Tests + captures — completion gate
Camera fuzz sweep (10-min scripted run against walls and corners; camera never
intersects geometry). Offset assertion per state. Scope handoff test (TP →
overlay → back, one frame each way, HUD intact). Torso-limit test (±60°
additive, beyond triggers turn-in-place).
**Capture:** clip of walk → strafe-aim → zoom → scope handoff → unscope.

---

## SECTION 6 — MECH OVERHAUL: FROM "OLD ROBOT" TO ALIVE WAR MACHINE

### 6.1 Material and detail pass
- Apply Brief VI §4.2 for real: full PBR per mesh — albedo, normal,
  roughness/metallic, AO. Faceted gunmetal plates, chamfered edges, matte
  0.6–0.8 roughness, edge wear masked to borders, panel-line normal detail,
  hazard chevrons **only** on pod cover and knee plates.
- Add implied secondary detail: hydraulic cylinders at knee and ankle, cable
  runs at joints, intake/exhaust vents, antenna mast, decals (unit number,
  faction emblem), dust gradient up the lower legs.
- **Material audit test:** iterate every mech submesh; assert none uses a
  default/placeholder material and every material has non-empty texture slots.
  "Untextured gray" becomes a CI failure, not an opinion.
- **Silhouette check:** render front/side/¾ beside the concept art in the
  handback. If it still reads "old robot," the plates are too flat — add bevel
  and inset variation until light breaks across the facets.

### 6.2 Entry/exit transformation
- **Enter (1.6s, committed, no cancel):** chest plates part and swing open,
  pilot steps in, plates seal, visor sweeps on with ignition flicker + servo
  chorus. **Exit (1.2s):** reverse; the mech powers down into a hunched dormant
  stance, visor dark.
- Occupied vs empty must be readable at a glance: visor lit + active stance
  versus dark + slumped. **No teleporting pilots, ever.**
- Interpenetration sweep covers both animations.

### 6.3 Damage states — armor that DROPS
Plates are separate meshes from day one (this also feeds §7):
- **70% HP:** shoulder-pod cover + left knee plate shear off as physics debris
  (despawn 20s). Underlying frame glows faint amber at the gaps.
- **40% HP:** side skirts + right knee plate drop; spark loop at one joint;
  servo whine pitch rises.
- **15% HP:** chest plate chips, visor flickers, gait gains a limp hitch
  (animation-only), smoke trail.
- Exposed under-frame takes **×1.25 damage**, applied *after* angle armor —
  stripping a mech is progress, and the attacker sees it happen.
- Plate-detach events run in the **sim layer** (replay-identical). Debris
  physics is cosmetic.
- **Alive idle:** visor scan-sweep every 7s, heat shimmer from vents, hull
  micro-sway (§1.1). A parked occupied mech must never look like a statue.

### 6.4 Tests + captures — completion gate
Material audit green. Damage-state matrix (drive HP to 70/40/15 → assert exact
plates detached and multiplier zones active). Entry/exit sweep green. Statue
test applied to the mech.
**Capture:** entry clip, damage progression clip (100→15% under scripted fire),
concept-art side-by-side renders.

---

## SECTION 7 — THE FORGE: PRE-MATCH CHARACTER DESIGNER

**Scope honestly:** this is a **modular part-swap** system, not a sculptor.

### 7.1 Content (v1 counts)
- **Soldier:** helmet (5, incl. none), torso armor (4), arm/gauntlet set (3),
  leg/greave set (3), shield face (4). Per part: primary + secondary color
  (faction-constrained palette), finish (matte/worn/polished), one decal slot
  (8 emblems), battle-wear slider 0–1 driving roughness + edge-wear intensity.
- **Body:** height ±5%, build slider (slim↔heavy blend shape), skin tone (8),
  face preset (6 — visible only when the helmet is off in preview).
- **Weapons (per weapon):** stock/grip/sight cosmetic variants (3 each), finish
  (4), emissive accent color — which also drives the Brief VI on-weapon ammo
  bar color.
- **Mech:** plate color, visor color (red/cyan per Brief VI), decal, wear
  slider. The §6.3 detachable plates **are** the customizable plates — one
  system, two payoffs.
- **Gauntlet choice drives the §2 first-person hands** — this is where the
  hand craft pays off and must be visible in the FP preview.

### 7.2 The editor (simple mode is the requirement)
- Left: category list → part grid with thumbnails. Right: large turntable
  preview (drag rotate, scroll zoom). **One click = one visible change.**
- **First-person preview toggle:** see your own hands/viewmodel exactly as in
  match.
- Randomize button. Reset-to-default per part.
- **3 saved appearance slots**, stored as RON/TOML in the save directory.
  Reachable from the main menu AND the lobby prep phase.
- Fully data-driven: parts are manifest entries (id, display name, mesh path,
  attach socket, palette mask). Adding a part later = drop in a glTF + one
  manifest line, zero code.

### 7.3 In-match application
Choices apply to the third-person model, the first-person hands/viewmodel, and
the mech. **Faction-readability guard:** silhouette and team-color regions are
locked; customization can never make an enemy read as a friendly.

### 7.4 Tests + captures — completion gate
Save/load round-trip (every field). Random-combo interpenetration sweep (200
seeded outfits × idle/run/thrust — no part clips through another).
Preview-equals-match test (same profile, compare preview pose vs in-match
screenshot within tolerance). First-person test (gauntlet choice visibly
changes in-match hands). Part-swap performance <100ms.
**Capture:** clip of designing a soldier, then spawning into a match wearing
exactly that.

---

## SECTION 8 — HANDBACK FOR PLAYTEST

1. **The §0 table, updated to "after"** — every row VISIBLE = yes, each with its
   capture path. This table is the receipt that the invisible-work problem is
   fixed.
2. Exact test commands and current pass/fail for every gate above.
3. **The full capture set:** idle-life clips, hand/finger craft clips, spear
   cycle, bow triple-pierce, third-person clip, mech entry + damage progression
   + concept side-by-side, Forge round-trip clip.
4. **Feel questions, answered concretely:**
   - Does the idle read as alive, or as twitchy?
   - Do the hands read as hands — is the trigger-finger travel visible?
   - Does the spear arc feel learnable within ~10 throws?
   - Does the bow release feel punchy at full draw and weak at minimum draw?
   - Does the camera tightening read as focus? Is the scope handoff seamless?
   - Does the damaged mech look wounded, not like a broken asset?
   - Is the Forge one click = one visible change?
5. **Every tunable introduced**, with current value and file location.
6. **Anything not done**, named plainly, with the reason (C7).

---

*Reference basis: Sons of the Forest spear-throw grammar (A), Abyssus piercing
bow (B), CoD Black Ops 6 third-person (C), CS:GO grammar per Brief VI, Brief V
motion-weight invariants. Joint ranges from measured human active ROM
literature. All numbers are starting tunables.*
