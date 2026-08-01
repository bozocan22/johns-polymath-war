# Thor's log — the master verification agent

Thor's one job: check everyone else's work, multiple times, and never
take a "done" claim at face value without evidence. Every audit,
every verification pass, every confirmed-or-disputed finding gets
recorded here as it happens — this file is Thor's memory, not a
retrospective summary written after the fact.

Rule Thor follows: a claim is DONE only when Thor (or an agent acting
under Thor) can point at file:line evidence. A gap is only real when
a SECOND, independent check confirms the first one wasn't simply
wrong. "Disputed" findings — where the first pass was wrong and the
code was already fine — get logged just as visibly as confirmed gaps,
because a false alarm wastes real work if nobody notices it was false.

---

## 2026-08-01 — Session start: retroactive record of prior verification work

Before Thor existed as a named role, this session already ran four
adversarial audit waves (~78+ agents) against `jk_tdm`, finding and
fixing 53 real defects, using the same discipline Thor now formalizes:
finder agents propose, verifier agents try to REFUTE, defaulting to
"not a real bug" when uncertain. Named failure patterns caught and
now tracked in `ANTI_PATTERNS.md`: the confident narrator, the split
brain, the one-way mirror, the shrinking-list index, the loyal ghost.

Also on record: a parallel session (visible via git, not this one)
independently caught a **fabricated WebFetch summary** before it
entered a research ledger — a real near-miss, logged in
`aiming/SOURCES.md`. This is the exact failure mode Thor exists to
catch systematically rather than by luck.

## 2026-08-01 — Thor's first formal action: discover+verify workflow across all 9 briefs

**Action:** Launched a workflow (`brief-audit-discover-verify`,
run id `wf_9b5f1aa1-187`) — 6 discovery agents, one per brief-area not
yet cross-checked this session (BRIEF_VII, BRIEF_VIII-B, BRIEF_VIII
§4/5/8, BRIEF_IX, PROMPT_brief_X_research, PROMPT_mech_rebuild), each
reading the brief in full and checking every concrete claim against
actual code via Grep/Read — not assumed, verified. Every claim flagged
PARTIAL or MISSING then goes to an independent second agent whose
default assumption is "the first agent was wrong," and which must
produce its own file:line evidence to confirm a gap is real.

**Status:** running. Results will be appended below the moment it
completes, before any implementation work starts on what it finds —
Thor checks before anyone builds, not after.

## 2026-08-01 — Workflow complete: 143 findings, and a caught misattribution

**Run stats:** 152 agents spawned, 106 completed, 46 errored (session
rate limit, not a code problem), 179 total claims checked across 6
brief-areas, 33 already DONE.

**A real problem Thor caught in its own process:** 46 of the Verify
agents failed with "You've hit your session limit," not because the
findings were wrong. The workflow script's own post-processing bucket
logic treats a failed verify agent's null result the same as
`confirmed_gap: false` — which would have silently filed 46 real,
never-actually-checked findings as "disputed / false alarm." This is
the EXACT misattribution this session already caught once before, in
an earlier audit wave, and documented as a named risk. Caught it again
here before trusting the bucket: a follow-up agent cross-referenced
the raw journal.jsonl, separated "verify genuinely ran and disagreed"
(3 real false alarms, skipped) from "verify never ran" (46 findings,
relabeled PROVISIONAL below, carrying the discover agent's own verdict
and evidence rather than a fabricated disposition).

**Meta-finding, not a code gap:** `PROMPT_brief_X_research.md` is
explicitly marked SUPERSEDED by `briefs/README.md`'s own index —
"Paste `PROMPT_MASTER_research_build.md`... Nothing else." Findings
tagged against it below are kept for completeness (mostly they
duplicate what the master prompt already tracks) but this file should
not be executed as its own prompt.

### DOUBLE-VERIFIED (97 total — independently re-checked, high confidence)

Full itemized list archived in this commit's PR description / commit
message (97 lines is too much to duplicate here twice). Summary by
brief and priority:

- **BRIEF_VIII_B_addendum.md** (rig/mech rebuild spec): 7 Critical, 9
  High, 6 Medium, 1 Low. The headline finding: the brief's "20-segment
  kinetic chain" describes a real 20-*segment mass-bearing body rig*
  (pelvis/lumbar/thorax trunk, clavicles, toe/forefoot); the codebase
  has an 8-*float timing curve* (`CHAIN_ONSET_OFFSETS`) driving
  follow-through velocity scaling on the EXISTING ~14-transform rig.
  Same name, different thing — the timing curve is real and tested,
  but it is not the rig rebuild the brief specifies. Mech rebuild
  (D.1-D.7: walking-weapons-platform silhouette, gatling+autocannon
  as the core kit replacing the missile pod, named part-by-part damage
  states) is also confirmed not undertaken — the mech is still a
  scaled humanoid.
- **BRIEF_VII_optimized.md**: 1 Critical (config/*.ron convention),
  13 High, 19 Medium, 9 Low. Mostly presentation-layer gaps (Forge
  editor UI, bone twist/metacarpal detail, several capture/test
  completeness items) on top of mechanics that mostly DO exist in
  simplified form.
- **BRIEF_VIII_master.md §4/5/8** (HUD/spear/Forge — the three
  sections this session hadn't audited yet): §4 HUD is the single
  richest vein of small, concrete, buildable gaps (minimap enemy dots,
  killfeed modifiers, crosshair settings, death/spectate flow, health
  bar geometry) — 13 findings. §5 spear: the 1.15x running-throw bonus
  is real and missing; spear gravity is deliberately reduced
  (GRAV_FACTOR_SPEAR=0.72) against the brief's "full gravity" spec -
  flagging as a design decision to revisit, not obviously a bug. §8
  Forge: confirmed there is no cosmetic customization system at all
  (2 flat color fields exist; the brief wants a full editor).

### PROVISIONAL (46 total — session-limit-affected, carrying discover verdict only, needs spot-check before acting)

- **BRIEF_IX_castle_grenade_customization.md**: the castle map itself,
  3 grenade types with distinct arm sequences, the 4-class/26-piece
  armour system, and the Forge integration are all confirmed absent
  by the discover pass (consistent with everything else this session
  has found about texture/class-system gaps) but NOT independently
  re-verified — treat as "very likely real" given how well it matches
  already-known gaps, but don't cite as double-confirmed.
- **PROMPT_mech_rebuild.md**: heavily overlaps BRIEF_VIII_B_addendum's
  mech findings (same 20-segment rig, same gatling/autocannon ask) -
  expected, they're the same underlying rework described twice.
- **PROMPT_brief_X_research.md**: mostly moot (superseded), except
  P29/P30 usefully confirm this session's OWN process changes (depth
  floor over breadth quota) are the right call, arrived at
  independently by a fresh read of the old quota-based draft.

### Thor's triage decision

143 findings is not a today-sized backlog — several (the 20-segment
rig rebuild, the mech visual rebuild, the Forge editor, the castle map,
the 26-piece armour/class system) are each their own multi-session
undertaking, not a bug fix. Thor's next actions: (1) pull the
genuinely small, well-scoped, safely-buildable findings and implement
them for real (build+test+commit+push+merge, per section, as
instructed) — starting now; (2) write the large architectural items
into `BACKLOG.md` as properly-scoped entries with their real size
stated honestly, the same way mech hull-climbing got a design doc
instead of a rushed build in Cycle 3; (3) keep this log updated as
each piece lands.

## 2026-08-01 — First two implementations off the confirmed-gap list

**§5.4 running-throw bonus** (`baf50ca`): built, tested, pushed. Along
the way the new test caught a real bug in THIS session's own earlier
work - the spear had been mapped to the rifle-tier sprint-out gate
(Section H), which HOLDS while speed is above 85% of sprint, making
"release a throw while still at running speed" structurally
unreachable. Exempted the spear (its own windup already pays the same
cost the gate exists for). Fixed before commit, not after - the test
did its job.

**§4.3 minimap enemy spotting** (`5fbb3c7`): built, verified live
(release build + baseline capture, exit 0, 0 panics - no sim-layer
logic exists here to unit-test, this is render/HUD state). Reused the
sim's own `los_clear` (made `pub(crate)`, was private) rather than
writing a second visibility check that could drift from the real one.

Both: build clean, tests/live-verify green, committed, pushed to
`bozocan22/johns-kingdom-polymath` main. Two down, ~95 real findings
left plus several multi-session architectural items (see BACKLOG.md).

**§4.1 bow full-draw sway** (`054a283`): built and tested. One
finding along the way, logged for anyone picking up work later rather
than silently skipped: `ElasticMove.return_efficiency` (an earlier
audit item) turned out NOT to be a gap - the struct's own doc comment
already says "SPEC FIXTURE, NOT A LIVE PATH," written by an earlier
pass of this session that deliberately graduated the REAL mechanics
(`chain_segment_scale`, `landing_rebound_vy`, `counter_movement_bonus`)
out into their own production functions instead of routing everything
through the shared struct. Thor's audit workflow flagged it as
"MISSING" without reading that comment closely enough; re-verifying by
hand caught it before wasting a cycle "fixing" something already
correctly documented as intentional. Recorded here so the same finding
doesn't get re-attempted from BACKLOG.md later without this context.

Three real features shipped this pass: running-throw bonus, minimap
enemy spotting, bow sway. 141/141 tests green throughout, each commit
independently verified (build + test, two of the three also live
smoke-tested since they touch render/HUD code with no sim-side unit
coverage of their own).

<!-- Thor appends every subsequent action below this line, in order -->

## 2026-08-01 - Thor live verification: Achilles / Mech / First Person / HUD

Owner-requested state check on four systems. Every row below was re-derived from
the code this pass, not carried over from the 143-finding audit. Verdicts:
BUILT / PARTIAL / MISSING / DELIBERATE (intentional deviation with a code
comment saying so - not a failure).

### 1. Achilles motion doctrine (VIII §1, VIII-B §B/§C)

| Sub-feature | Verdict | Evidence |
|---|---|---|
| Accel lean 28-32° -> 8-12° top speed | PARTIAL | main.rs:6120 lean_target clamped ±0.07 rad (4°); main.rs:874 caps total torso pitch at 0.185 rad (10.6°). Top-speed band met; the 28-32° start spike is clamped BY the §0.2 head-band law, explained main.rs:877-885 |
| Knee drive ~130° peak | PARTIAL | main.rs:6195 `shin = 0.10 + lift*1.25` -> 77° peak |
| Arm drive (elbows 90°, hip-to-chin) | PARTIAL | main.rs:6365 "net ±1.5° - which is also all the arm swing an armed carry gets"; arms IK'd to the weapon (main.rs:6743). No unarmed sprint arm cycle |
| Cadence over reach (distance-driven phase) | BUILT | main.rs:6113-6114 `phase += speed*dt*PI/STRIDE_M` |
| Vertical-oscillation ≤4cm | BUILT, untested | main.rs:872 hip bob max 3.5cm; grep `fn .*bob` = 0 test hits |
| Ground contact / no overstride | PARTIAL | main.rs:6189-6198 elliptical foot path + ankle roll authored; no overstride assertion |
| Plant-and-cut >90°, hips before torso | PARTIAL | main.rs:5935 LEG_TURN_RATE 6.5 + 60° torso clamp (main.rs:5941-5959) gives hips-lead/torso-follow. No foot plant, no 5-8% height drop, no >90° trigger. Sim yaw is instant: sim.rs:3702 `yaw = cmd.yaw` |
| **Full stops never hit instant zero** | **MISSING** | sim.rs:3688 `self.fighters[p].vel = vel` - velocity written raw from input every tick, zero accel/decel model. Releasing input = 0 m/s in one tick. Only stop cue is the render-side lean transient (main.rs:6118-6121) |
| Starts get 2-3 frames anticipation | PARTIAL | main.rs:6395-6396 head lags the pelvis lean (test main.rs:11110). No body-level pre-step coil |
| Kinetic chain utility, built once | BUILT | main.rs:396-413 CHAIN_ONSET_OFFSETS / CHAIN_PEAK_SCALE / chain_segment_scale; tests main.rs:11475, 11491 |
| Routed through EVERY power move | PARTIAL | Live consumers: spear follow-through (main.rs:458-471), sprint-start head lag (main.rs:6395). Not dodge launch, melee shove, or mech side-step |
| 5 named anti-patterns greppable in code | MISSING | grep -i "mannequin / wall stop / ice skater / switch flip / floating gun" over sim.rs+main.rs = **0 hits** |
| ElasticMove load/release model | DELIBERATE | main.rs:350-358 doc: "SPEC FIXTURE, NOT A LIVE PATH" - real mechanics deliberately graduated out to chain_segment_scale / landing_rebound_vy / counter_movement_bonus |
| C.2 rule 3 counter-movement bonus | BUILT | sim.rs:70 + applied at dodge trigger sim.rs:3732-3735; test sim.rs:7747 |
| C.2 rule 5 landing rebound 8% | PARTIAL | main.rs:388 landing_rebound_vy, ONE call site (main.rs:7439) = camera only. No sim-side landing rebound |
| C.2 rule 4 landings recharge / flow bonus | MISSING | no stored-energy carry between moves anywhere |
| §1.6 tests 2/3/4 (cut, zero-stop sweep, bob budget) | MISSING | none of the 142 tests cover these; test 1 (chain timing) exists |

### 2. Mech (VIII §7, VIII-B §A/§D)

| Sub-feature | Verdict | Evidence |
|---|---|---|
| Scale | DELIBERATE | sim.rs:2340 `MECH_SCALE = 1.7` - addendum §A option A3 chosen, reasoning written into the comment (supersedes §7.1's 1.15×). Test sim.rs:9497 |
| Grounded, flight deleted | BUILT | sim.rs:3796 "FLIGHT IS DELETED"; 60s fuzz-drive test with jump+thrust inputs sim.rs:9516-9530 |
| Walk 85% / power stride | BUILT | sim.rs:2429-2434 windup 0.35s, 2.5s, ×1.10, turn cap 90°/s; heat shared with minigun; pod locked while striding (sim.rs:3829); test power_stride_winds_up_bursts_locks_and_costs_heat |
| Pivot cap 180°/s | BUILT | sim.rs:2418 MECH_TURN_RATE = 3.1416, applied sim.rs:3698-3700 |
| Braced side-step ≈3m/0.9s ×scale | PARTIAL | sim.rs:91-93 MECH_STEP_S 0.30 × MECH_STEP_SPEED 6.5 = 1.95m in 0.3s. Not 3m/0.9s, not ×1.7, does not route through §1.4 at ×2.2 offsets |
| **Brace stance (movement 0, ×0.7 front, half spread)** | **MISSING** | `f.brace` is the Folk Shieldwall Brace only (sim.rs:4023, explicitly zeroed for every other set). grep "mech.*brace" = no mech stance |
| Step-up ≤0.4m ×scale | PARTIAL | sim.rs:54 STEP_UP = 0.55, one global constant shared with soldiers; no mech-specific value, no test |
| Entry 1.6s / exit 1.2s committed | BUILT (sim) / MISSING (visual) | sim.rs:2350-2351 + 8 named stages sim.rs:2387-2402, test sim.rs:9648. main.rs references mech_transition_t exactly once (main.rs:2242, to SKIP it for captures) - no plates-part / pilot-steps-in / visor-ignition presentation |
| Alive idle life (visor sweep 7s, heat shimmer, hull micro-sway) | MISSING | grep "scan.sweep / heat shimmer / micro.sway" in mech context = 0 hits. Only the static hull pitch main.rs:6088-6092 |
| Angle armor 0.15/0.30/1.00 + visor ×2.0 | BUILT | sim.rs:2409-2416, hull 1000 sim.rs:2342; test mech_arcs_follow_body_facing_and_eject sim.rs:8555 |
| Plate detach at 70/40/15% + exposed ×1.25 | BUILT (sim) / MISSING (debris) | sim.rs:6611-6629 bitmask, sim.rs:2405-2408; tests damage_state_matrix_fires_each_stage_once_in_order, exposed_frame_takes_the_1_25x_bonus_only_after_a_plate_drops. No physics debris and no per-plate mesh hiding in main.rs |
| Silhouette / D.3 20 named parts | PARTIAL | main.rs:3487 `plates: [...; 33]` - anonymous plates worn over the humanoid rig. Visor slit, hazard chevrons, 4-tube pod, antenna, exposed knee/waist actuators all modelled (main.rs:3515-3560). No digitigrade reverse-joint legs (grep = 0 hits), no named/separable part list |
| D.2 palette correction (olive/khaki) | BUILT | main.rs:1468-1477 mech_khaki/_dk/_lt, mech_shadow, mech_metal, mech_hazard - the gunmetal spec was already replaced |
| Material audit + part-count tests | MISSING | no such tests among the 142 |
| Minigun heat curve | BUILT | sim.rs:2476-2488 spin 0.4s, 1.5 heat/shot (0->100 in ~4s), vent 3.0s, spread 1.2°->3.5°; test minigun_heat_cycle_is_deterministic |
| Heavy rifle (AWM) matrix | BUILT | sim.rs:389-395 (115 base, reload 3.7); test awp_matrix_one_shot_rules sim.rs:9731 |
| Missile pod lock-on | BUILT | sim.rs:1884-1897 all numbers exact (4 tubes, 1.3s, cos6°, 250m, 60 m/s, 250°/s, 7s TTL, 0.4s LOS, 270 dmg, PN N=3); no-infantry lock sim.rs:3841 + test sim.rs:9817 |
| **D.5 gatling + autocannon core kit** | **MISSING** | grep "autocannon / gatling / drum magazine" = **0 hits**. Missile pod is still the default rather than the swappable |
| Mech first-person visor view (offsets ×scale, visor vignette) | MISSING | grep MECH_SCALE in main.rs = 2 hits, neither a viewmodel offset; only vignette is HealthVignette |
| Footfall shake 0.2 within 6m | BUILT | main.rs:7470-7488 (not scaled ×1.7 per addendum §A) |

### 3. First person (VIII §3)

| Sub-feature | Verdict | Evidence |
|---|---|---|
| Rule 1 - no ADS translation, ever | BUILT | main.rs:7849-7857: ads_capable = scoped OR projectile - standard guns get a hard-zero aim shift as a LOCAL guarantee, reasoning in the comment |
| Rule 2 - scoping hides the weapon | BUILT | main.rs:1198 vm_hidden_while_scoped, applied main.rs:7934; test vm_hides_while_scoped |
| Placement (68° VM FOV, +0.11/-0.13/+0.32) | BUILT | main.rs:1141 VM_FOV_DEG = 68.0; main.rs:4627 Vec3::new(0.11, -0.13, -0.32) |
| On-weapon emissive ammo bar | BUILT | main.rs:1202-1219 AmmoBarSeg (8 segs, left face) + ammo_bar_sync main.rs:8995 with reload pulse |
| No-bounce spec (frozen clock, ≤0.3° sway, ÷5 air, ≤1.5cm/120ms slide) | BUILT | main.rs:1143-1152 constants; carry_offset main.rs:1168-1193; tests vm_never_bounces, vm_envelope_clears_midline_and_center |
| Sprint carry 18° down / 8° in | PARTIAL | main.rs:7955-7963: sp*0.61 rad pitch (35°) and sp*0.35 rad yaw (20°) - roughly double the specced angles |
| Sprint-out per class | BUILT | sim.rs:2497-2514 (0.15/0.20/0.30, minigun & spear exempt with written reasons); test sprint_out_gates_fire_by_weapon_class |
| Ready-up 0.15s with ζ≈0.7 overshoot | PARTIAL | main.rs:7801 out-blend 0.14s, monotone ease - no overshoot |
| Reload craft (tactical vs empty, hand IK, trigger finger) | PARTIAL | sim.rs:2521 RELOAD_EMPTY_MULT = 1.35 + test; reload_pose main.rs:7562; trigger_finger_press main.rs:1061 + test. But empty is a time MULTIPLIER, not a separate clip, and no explicit magwell IK target |
| **Low-ready / obstruction 22° muzzle raise** | **MISSING** | cam.blocked (main.rs:5619) drives only a crosshair colour (main.rs:8953). No viewmodel rotation on wall approach |
| Inspect | BUILT | main.rs:7837-7848 (T), rotation + drift, cancelled by any combat input |
| Hit feedback | PARTIAL | main.rs:8948-8955 crosshair colour by zone (gold headshot / red body / orange kill) + crosshair_kill_pop main.rs:9031. No 2-frame marker scaled by damage, no distinct headshot audio tone |
| Recoil 3 channels + fixed crosshair | BUILT | sim.rs:1934-1936 RECOIL_SCALE 2.0 / VIEW_RECOIL_TRACKING 0.45; camera channel main.rs:7306-7312; viewmodel rotational-only main.rs:7955-7963. Crosshair is screen-anchored text, so crosshair_follow_recoil is OFF by construction |
| Cosmetic view punch ×0.055/shot | MISSING | no such constant; channel 2 is the 0.45 tracking only |
| Deterministic spray tables (seed, lerp 0.55, first-shot 0.75->1.0) | BUILT | sim.rs:1951-1988 spray_entry; sim.rs:5039-5051 suppression + lerp; tests a_thirty_shot_ak_spray_is_bit_identical_on_replay, spray_replays_exactly_climbs_and_recovers |
| Recovery constants 8 / 18°/s / 4.5, decay per 0.5s | BUILT | sim.rs:1938-1945 |
| Inaccuracy 34% -> 95% ramp | BUILT | sim.rs:1948-1949, applied sim.rs:4864-4866 |

### 4. HUD (VIII §4.0-§4.9)

| Sub-feature | Verdict | Evidence |
|---|---|---|
| Four-corner data-driven layout + 5% safe area | BUILT | main.rs:1278-1285 HUD_ANCHORS; test hud_layout_holds_at_three_resolutions (1920/2560/1280) main.rs:10423 |
| §4.1 vitals number + colour thresholds | BUILT | main.rs:1288-1298 vitals_color (red ≤25, pulse ≤20); test main.rs:10471 |
| §4.1 depleting bars, armor shield icon, hud_vitals_style | MISSING | main.rs:8842-8845 is a text "+ {hp}" line; no bar geometry, no armor number for non-mech sets, grep hud_vitals_style = 0 hits |
| §4.2 ammo "26 / 90" + low-mag red | BUILT | main.rs:8908-8916; ammo_is_low main.rs:1301 + test |
| §4.2 vertical loadout strip / grenade glyphs | PARTIAL | throwable name+count on a second line only; no strip, no active-weapon offset |
| §4.3 minimap + enemy ghost-fade | BUILT | main.rs:1701-1751 + minimap_system main.rs:8196; MINIMAP_GHOST_FADE_S 3.0, 8 slots (this session's 5fbb3c7) |
| §4.3 rotates with facing, 0.25-1.0 scale slider | MISSING | to_map main.rs:8240 is world-fixed north-up; MINIMAP_PX is a const; settings carry only an on/off bool (main.rs:653) |
| §4.3 resource counter + award toasts | MISSING | LobbyToast (main.rs:2479) is Intro-state only |
| §4.4 timer + score, red final 0:10 | BUILT | main.rs:8720-8737; hud_colors main.rs:8984 |
| §4.4 alive counters, faction blue/orange flanking chips | MISSING | one text line "BLUE nn - nn RED" |
| §4.5 killfeed: 5 rows, newest bottom, assist, headshot glyph, local marker | PARTIAL | main.rs:8757-8779. Marker is a "\| " prefix, not a 2px #B50000 border; no name colours, no pills, no lifetime bonus when involved |
| §4.5 other five modifier glyphs | DELIBERATE | main.rs:1305-1308 doc: the remaining glyphs "need sim events this game does not track yet - deferred, documented" |
| §4.6 no crosshair for scoped-class when unscoped | BUILT | main.rs:8947 noscope_hidden |
| **§4.6 crosshair settings family** | **MISSING** | crosshair is a "+" text glyph (CrosshairText, main.rs:9031). GameSettings (main.rs:650-664) has 5 fields total, none crosshair. No size/gap/thickness/dot/outline/RGB/T-shape/static-dynamic - so §4.9's round-trip test cannot exist |
| §4.7 flash overlay | PARTIAL | main.rs:7055-7060 white plate quantised to 8 steps off blind_t. No angle-banded hold times (2s/0.5s/0.1s), no frozen afterimage |
| §4.7 damage direction wedge | BUILT | damage_indicator main.rs:9163-9204, 4 edge strips, ttl/2.2 fade |
| §4.7 context progress bar | MISSING | contextual_prompts main.rs:9397 is a text line only |
| §4.7 death -> killer-cam -> "SPECTATING <name>" | MISSING | grep "SPECTATING / killer.cam / spectate" = 0 hits; death shows "DOWN - respawn in Ns" (main.rs:8822) |
| §4.7 low HP = colour only, no vignette | DELIBERATE/BUILT | main.rs:2465 "OFF - it is held fully transparent by health_vignette" - matches the brief exactly |
| §4.8 scoreboard | PARTIAL | scoreboard_system main.rs:9154: columns NAME/K/D/HITS/WEAPON. Missing Assists, DMG, Score, Ping; no local-row highlight; minimap does not switch to square overview while held |
| §4.8 loadout menu (flat grid, prices, refunds) | MISSING | no in-match buy/loadout menu |
| §4.9 tests | PARTIAL | layout-at-3-resolutions and threshold/glyph tests exist (main.rs:10423, 10471). Crosshair round-trip impossible; the "six modifier glyphs" stream test covers the one implemented glyph |

### What's genuinely left in these four systems

**Small, one-session-buildable (ranked by value):**
1. §4.6 crosshair settings family - the largest concrete HUD gap; needs a settings-struct extension plus a real drawn crosshair replacing the text glyph.
2. §4.7 death -> killer-cam -> spectate flow, and the §4.7 context progress bar. Both self-contained UI states.
3. §7.4 mech brace stance - the only mech mobility verb with no code at all; the damage-multiplier and spread hooks it needs already exist.
4. §4.8 scoreboard columns (assists/damage/score) + local-row highlight - the sim already tracks ev.assist and per-hit damage.
5. §3.4 low-ready obstruction (22° muzzle raise) and the ready-up overshoot - cam.blocked is already computed; only the viewmodel rotation is missing.
6. §4.1 vitals bars + armor cluster; §4.3 minimap rotate/scale options.
7. §7.4 mech step-up and side-step distance scaled by MECH_SCALE - addendum §A explicitly requires this and it was missed when the scale moved to 1.7.
8. §1.6 missing tests: zero-instant-stop sweep, vertical-bob budget, lean-and-cut ordering.

**Large / architectural (do not rush):**
1. **Sim-side acceleration model.** sim.rs:3688 assigning velocity raw from input is the root cause of "the wall stop" and of plant-and-cut being unimplementable. Fixing it touches every movement consumer, every bot, and every replay-determinism test - the highest-leverage and highest-risk item here.
2. **Mech visual rebuild to D.1-D.6** - digitigrade legs, 20 named separable parts, gatling+autocannon core kit, physics debris on plate detach. Confirmed still open (already in BACKLOG.md).
3. **Mech presentation layer** - entry/exit visuals (BACKLOG #16), idle life, first-person visor view. The sim staging exists with nothing rendering it.
4. **Kinetic chain routed through every power move** rather than two consumers - depends on the rig work already in BACKLOG.md.

Not gaps, confirmed intentional and correctly documented: MECH_SCALE 1.7
(addendum §A option A3), ElasticMove as a spec fixture, the five deferred
killfeed glyphs, the always-transparent low-HP vignette, and the khaki/olive
mech palette (D.2's correction is already in).
