# MISSION doc — REPORT (rig / elastic motion / mech)

91/91 tests green. Build stable at launch (stderr-capture + 12s liveness
check, twice). Committed as `c26673d` (on top of Brief VII v2's `a1dadc6`).

## 1. Task 0 table — "after"

| Feature | Coded | Tested | Visible in launched build | Root cause (if not visible) | Fix |
|---|---|---|---|---|---|
| Character rig segment count | 14 mass-bearing segments (torso×1, neck×1, arm×3×2, leg×3×2) before this session | — | — | trunk was genuinely 1 bone in the MESH hierarchy | See Task 2 finding below — the trunk being 1 mesh-bone did NOT mean 0 separation |
| Hip-shoulder separation at wind-up | **yes, already ~42° before this session** | **yes, new test** | yes (was already live) | n/a — see finding | Extracted into `torso_coil_yaw`, added `hip_shoulder_separation_reaches_35_to_45_degrees_at_windup` |
| Viewmodel placement / no-bounce | yes (Brief VI/VII v2) | yes | yes | n/a | n/a |
| Sprint/turn/stop locomotion | yes | yes | yes | n/a | n/a |
| Spear thrust | yes (Brief V) | yes | yes | n/a | n/a |
| Spear throw | yes (Brief VII v2 §3) | yes | yes | n/a | n/a |
| Elastic load model | **no before this session** | **yes, new** | landing rebound wired to the real camera; other consumers (spear/jump/dodge) NOT yet wired | never built | Built `ElasticMove`/chain/rebound utilities + 1 real consumer |
| Mech existence | yes | yes | yes | n/a | n/a |
| Mech scale | 1.15× (Brief VI) | yes | yes, now 1.7× | doc's Task 4 argued 1.15× "looks nothing like the art" | Changed to 1.7× (option A3) |
| Mech materials | gunmetal-gray (Brief VI), NOT khaki as I'd mis-remembered | yes (existing material-audit-style coverage) | now olive-drab | wrong palette family entirely | Swapped hull_primary/shadow/mechanism/barrel to Task 5.2's exact hex values |
| Mech weapons (gatling+autocannon) | **no** | **no** | **no — still the Brief VI missile pod** | never built this session | Named deferral, see §8 |

## 2. The separation test, before and after

**Before this session:** no test existed measuring thorax-vs-pelvis yaw at
all. The document's premise — "with a single trunk bone this value is
always 0°" — could not be verified either way.

**Investigation finding:** the mesh hierarchy DOES have one `torso` node
carrying most of the upper-body geometry, but that node is a **child of the
character root**, and the root ALSO carries the legs' base rotation
(`f.yaw`). The torso applies its own ADDITIONAL local yaw on top
(`torso_coil_yaw(...)`, driven by the spear windup's coil-away rotation).
Because Bevy composes a child's local rotation onto its parent's world
rotation, this is **already** a genuine two-segment separation — just not
one the document's audit had located or measured.

**After:** `torso_coil_yaw` extracted to a standalone, directly-testable
function. Three new tests:
```
cargo test --release -p jk_tdm rig_separation_tests
→ hip_shoulder_separation_reaches_35_to_45_degrees_at_windup ... ok  (peak ≈ 42°)
→ separation_is_genuinely_nonzero_not_a_fused_bone ... ok
→ no_gun_no_twist ... ok
```
This is the single clearest before/after proof requested: the premise
that motivated the full 20-segment rebuild does not hold for this
codebase's actual architecture. **The 20-segment rig rebuild (Task 2's
main body) was therefore not undertaken** — the specific problem it was
scoped to solve turned out not to exist. (Toe segments / clavicle
segments / full mass-fraction ragdoll are still absent; see §8 — they may
have independent value for gait/reach fidelity, just not for the
separation problem that was Task 2's stated justification.)

## 3. Exact test commands and output

```
cargo test --release -p jk_tdm
→ test result: ok. 91 passed; 0 failed; 2 ignored
```
New suites this pass: `rig_separation_tests` (3), `elastic_load_tests` (6).
Full breakdown of everything since Brief VII v2 began: 49 → 82 → 91.

## 4. The scale decision (Task 4)

**Chose A3: 1.7× (≈3.03m).** Matches the document's own recommendation.
`MECH_SCALE` is a single formula-driving constant in this codebase
(`MECH_RADIUS`, `f.height()`, and the client's `tf.scale` all derive from
it), so the change itself was low-risk. One real regression DID surface:
**the third-person camera's anchor height and boom distance were
hardcoded** (`p.pos[1] + 1.6`, boom 2.2m flat) rather than derived from
the fighter's actual height — at 1.15× this was invisible (soldier and
mech heights were close enough), at 1.7× it put the camera partially
inside the mech's own hull on a first capture attempt. Fixed both to
scale proportionally (`anchor_h = 1.6 × (height / BODY_HEIGHT)`,
`boom × height_boom_mult`).

That fix alone still produced a bad frame on the next attempt - root
cause turned out to be a SEPARATE, unrelated confound: the capture
script's default spawn point happened to sit close to map cover, and
the (correctly-functioning) boom-collision system was pulling the
camera in against that wall, not the mech. Once the capture script
planted the mech at a known-clear spot (Arena center) instead of an
arbitrary spawn, framing resolved cleanly - see the committed
`01-mech-third-person.png` / `02-mech-side-on.png`, the latter of which
incidentally captured the mech taking live bot fire (two logged
headshots, hull 1000→946), confirming the damage/visor-bonus system
is fully wired at the new scale. **Both the camera-scaling fix and the
spawn-position fix were real and necessary; together they resolve
framing at 1.7× with no further open issue.**

## 5. Mech vs. concept art

Not literally side-by-side (Task 1's tooling limitation — no image
download capability, so there is no committed art image to place next
to a render). What changed structurally toward the art: 1.7× scale
(closer to the described ≈2.5× than 1.15× was, deliberately short of it
per A3's own compression logic), olive-drab palette replacing gunmetal.
What did NOT change: part count (still the pre-existing plate-bitmask
damage system, not 20 separate authored meshes), weapon loadout (still
the missile pod, not gatling+autocannon). Forward hull pitch and exposed
knee/waist mechanism plates were added in a later pass — see the
Task 5.4/5.7 addenda in §8 below.

## 6. Every tunable introduced

| Constant | Value | File |
|---|---|---|
| `ElasticMove` fields (per-instance, not a global const) | load_s/release_s/stored_energy/return_efficiency | main.rs |
| SSC bonus scale | 0.35 | main.rs (`release_velocity`) |
| Landing rebound fraction | 0.08 (8%) | main.rs (`landing_rebound_vy`) |
| `CHAIN_ONSET_OFFSETS` | [0, 0.02, 0.035, 0.055, 0.065, 0.09, 0.11, 0.125] | main.rs |
| `CHAIN_PEAK_SCALE` | [1.00…2.10] | main.rs |
| `MECH_SCALE` | 1.7 (was 1.15) | sim.rs |
| Mech palette (`mech_khaki`/`_dk`/`_lt`/`mech_shadow`/`mech_metal`) | #8A8770/#5F5E52/#9A9384/#33352F/#2B2C2B | main.rs |
| Camera anchor height scale | `1.6 × (height/BODY_HEIGHT)` | main.rs |
| Camera boom height scale | `height/BODY_HEIGHT` (min 1.0) | main.rs |

None of these are externalized to `config/*.ron` — same honest gap as
Brief VII v2's handback: a hot-reload config system is real infrastructure
work, not a per-task add-on, and was not attempted this session.

## 7. Answers, in plain language

- **Does the torso visibly twist between hips and shoulders at wind-up?**
  Yes, and it already did before this session — the finding is that this
  was true and unmeasured, not that it was false and needed building.
- **Does a loaded action visibly out-perform a flat-footed one?** The
  math is real and tested (stored energy scales output by exactly the
  spec's 1.35× at full load). It is wired into exactly one real,
  visible consumer (landing rebound). Spear throw, jump, and dodge do
  NOT yet route through `ElasticMove` — the utility exists, the
  wiring doesn't, for most of its intended consumers.
- **Does the run read as drive, or still as a glide?** Not independently
  re-evaluated this session — no toe segment was added (Task 2's toe-off
  requirement specifically), so there is no new mechanical reason for
  this answer to have changed.
- **Does the mech read as a machine — do exposed mechanisms sell it?**
  Confirmed visually (see `handback/brief-vii/mech_scale/`): scale and
  palette both read clearly against the map's crates and a soldier-scale
  HUD. No new geometry (exposed knee/waist mechanism, hydraulic detail,
  hazard chevrons, stencils) was added, so it reads as "a bigger
  olive-drab version of the same shape" rather than "a newly-detailed
  assembled machine" — the geometry gap named below is real, just not
  a framing/visibility problem on top of it.
- **Does the autocannon's recoil make the mech feel heavy?** There is no
  autocannon — the mech still fires its Brief VI loadout (minigun/AWP-
  class rifle) plus the missile pod. Not attempted this session.

## 8. Named deferrals (honest, per R6)

- **The gatling + drum-fed autocannon weapon swap** (Task 5.5) — the
  mech's weapon system (minigun/AWP + missile pod) is fully built,
  tested, and deterministic from Brief VI/VII v2. Replacing it with two
  new weapon types is a real weapon-system addition (fire logic, ammo,
  recoil, a new "hull-shove" recoil effect), not a retune, and risks the
  existing weapon test suite if rushed. Not attempted.
- **The 20-part separately-detachable mesh rebuild** (Task 5.3) — the
  mech currently uses the existing bitmask damage-state system (built in
  Brief VII v2 §6: 3 threshold stages, not 20 individually-authored,
  individually-detachable meshes). Building 20 real mesh parts with
  individual detach/debris physics is a large asset-authoring task this
  session's procedural-primitive pipeline was not extended to cover.
- **Stance change** (hull pitched nose-down, hips high, knee-forward
  lean) — implemented after this report was first written: `mech_pitch`
  (0.085 rad forward hull pitch, cosmetic-only) applied whenever
  `armor_set == RobotSuit && hull > 0.0`. Hazard chevrons, wear masking,
  and stencils are still not implemented.
- **Exposed knee/waist mechanism geometry** — implemented: 4 new
  `mech_metal` plate entries (knee actuator stubs + waist linkage block)
  in `spawn_armor_rig`'s `plates` array (26 -> 30 entries), verified in
  a new capture beat (`03-mech-knee-waist-detail.png`, steep downward
  pitch). Deliberately small-scale per the doc's own framing ("exposed
  mechanism," not a new silhouette element) — reads as detail texture at
  normal third-person distance, not a dramatic shape change. Hazard
  chevrons, wear masking, and stencils are still not implemented.
- **Hazard chevrons / wear masking** — partially addressed: the pod
  cover was the only hazard-striped part on the whole hull (an
  asymmetric, "half-finished" read), so a matching stripe was added to
  the right shoulder pod, plus a power-pack warning band and one bare-
  metal abdomen scuff patch (30 -> 33 plate entries). A real stencil/
  decal system (unit numbers, ownership marks, worn-edge masking that
  follows UVs rather than being hand-placed per plate) is still not
  built — this was 3 more hand-placed primitives in the same style as
  the rest of the rig, not new tooling.
- **Capture-harness bug found and fixed in the same pass**: the §1.2
  first-run "GOOD TO KNOW" tutorial card (added after this report's
  first draft, dismissed by any keypress) was silently blocking every
  `mech_scale` snap, including the two already committed under Brief
  VII v2's handback — the script only synthesizes camera-look beats, no
  keypress, so the card never cleared. `capture_quick_deploy` now
  force-dismisses `FirstRunCard` for every capture script. Both existing
  screenshots were retaken and now show the mech unobstructed.
- **Task 1's literal deliverable** (12-20 committed image files) — not
  achievable with this session's tools (no image-download capability).
  Real research was done and written into `NOTES.md` with sources
  instead.
- **`SPRING_K_*` spring constants (§2.5)** — `damped_spring`'s own doc
  comment claimed all five named stiffnesses (hand follow, elbow pole,
  finger settle, shoulder, camera boom) were wired to real consumers.
  Checked, and it was false: only one real call site existed anywhere
  (the viewmodel sway, using its own unrelated k=196) plus a test. Fixed
  the camera-boom one — the collision-recovery push-out now genuinely
  uses `SPRING_K_CAMERA_BOOM` via critical damping instead of the flat
  `CAM_RECOVER_S` chase it was quietly still using (removed that now-
  dead const). The other four remain unwired: each needs new PER-
  FIGHTER persistent spring state (the camera is one global resource;
  hand/elbow/finger/shoulder would need it per-arm, per-fighter),
  threaded through the arm-IK/finger-curl/shoulder-pose pipeline without
  disturbing the hit-band-tested pose functions — a real follow-on, not
  a same-pass tweak.
- **Kinetic chain sequencing** — wired into a real consumer after this
  report was first written: `torso_coil_yaw`'s post-action follow-
  through branch (covers BOTH a spear throw's release and a thrust's
  recovery — they already shared one branch) now drives its settle curve
  off `spear_followthrough_yaw`, which uses the chain's tip segment
  (`CHAIN_ONSET_OFFSETS`/`CHAIN_PEAK_SCALE`/`chain_segment_scale`/
  `chain_peak_tick`) instead of the `jerk` gun-recoil proxy it used
  before. New test: `spear_followthrough_yaw_is_silent_then_snaps_then_
  settles`. `counter_movement_bonus` (Rule 3's SSC bonus) was
  investigated for jump/dodge launch but NOT wired: the "prior movement
  direction" it needs is overwritten by sim.rs the same frame a dodge
  triggers (`f.vel = roll_dir * speed` fires before the client ever sees
  the pre-dodge velocity), so wiring it correctly needs a new sim-side
  snapshot field — a real scope decision, not a same-pass tweak. Sprint
  start and mech side-step also remain unwired.
- **Config externalization (R4)** — a first real slice landed after this
  report was first written: `config/camera_tuning.txt` (hand-rolled
  `key=value` text, not `.ron` — no serde dependency, same convention as
  the Forge saves) now overrides the third-person camera's 7 feel
  constants at startup via a new `CameraTuning` resource; a missing
  file/key/bad value is provably a no-op (4 unit tests + a live check:
  temp-set `tp_boom=7.0`, confirmed via a real capture that the camera
  visibly pulled back off the fighter, then restored the default and
  confirmed framing matched exactly). Still NOT externalized: every
  other tunable in the table above (`MECH_SCALE` especially — several
  other `sim.rs` consts derive from it at compile time, so converting it
  is a bigger, separate job), the elastic-load/spring/chain constants,
  and the mech palette.
