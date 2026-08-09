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

---

## 2026-08-01 — Thor's #1 architectural finding, acted on same session

Thor's live verification (above) ranked **"sim-side acceleration model"**
as the single highest-leverage AND highest-risk item across all four
systems, with the exact evidence: `sim.rs:3688` assigning velocity raw
from input. That is now **built and shipped** (`a1fb256`).

What landed: `approach_velocity()` — one pure, shared function, called
by BOTH the player path and the bot path so they cannot drift (Thor has
flagged bot/player parity as a repeated defect class in this file; the
mech turn-rate comment is the previous instance). Two rates,
`GROUND_ACCEL` 55 and `GROUND_DECEL` 40 m/s².

**The counter-strafe emerged rather than being special-cased.**
Releasing input → target speed 0 → DECEL (slow). Pressing the opposite
direction → target of equal magnitude → ACCEL (fast). So tapping back
beats letting go, which is the CS-family mechanic, and it falls out of
the two-rate model with no branch for it. The test asserts *that
relationship* rather than either constant, so it survives retuning.

**Thor's own risk assessment was correct and worth recording.** Thor
called this "touches every movement consumer, every bot, and every
replay-determinism test." In practice one test broke — the
running-throw bonus — and it broke for exactly the predicted reason:
reaching the 70%-of-sprint threshold now takes real time, so a run-up
sized for an instant-acceleration world no longer qualifies. The
assertion was correct and unchanged; only the setup was stale. Fixed
the run-up and documented why. **Thor's risk call was accurate, and
the blast radius was smaller than feared — worth knowing for the next
architectural item, so future estimates aren't over-cautious.**

## 2026-08-01 — Thor logs its OWN tooling failure (process, not code)

The research workflow dispatched for the 20-segment rig **crashed on a
bug in the orchestration script I wrote**: `const research = parallel([...])`
with no `await`, so `research` was a Promise and `research.filter(...)`
threw. Four agents had been spawned; one (the rig audit) completed, the
rest were abandoned mid-flight. ~397k subagent tokens spent on a run
that returned nothing usable.

Recorded here deliberately, because Thor's remit is *checking work*, and
the work includes Thor's own instrumentation:
- **The failure was silent until the very end.** The script ran the
  audit agent successfully, then died at the synthesis step. A crash
  *after* real work completes is the expensive kind.
- **Recovery worked as designed.** Fixing the one missing `await` and
  resuming with `resumeFromRunId` replays completed agents from cache
  rather than re-running them.
- **Lesson for future workflow scripts:** every `parallel()` and
  `pipeline()` result must be awaited before it is treated as an array.
  This is now the second orchestration-layer defect this session (the
  first: session-limit failures being silently bucketed as "disputed").
  **Both were in the harness, not the game.** The pattern worth naming:
  *the instrument fails more quietly than the thing it measures.*


## 2026-08-03 — Verification pass on "Rig Step 1: the kinetic chain, from measurement instead of by feel" (787f6ff)

**What shipped.** `CHAIN_ONSET_OFFSETS` (main.rs:422-423) went from the
by-feel `[0.000, 0.020, 0.035, 0.055, 0.065, 0.090, 0.110, 0.125]` to a
measurement-derived `[0.000, 0.016, 0.035, 0.040, 0.070, 0.094, 0.114,
0.130]`; `CHAIN_PEAK_SCALE` (main.rs:441-442) likewise; two new tests;
new `JAVELIN_ANCHOR_S` (main.rs:427-428). **The commit's load-bearing
claim:** "the entire blast radius of this commit is ONE production
behaviour: `chain_lag_chase`'s head-lag time constant, 0.125 -> 0.130,
a 4% change."

Thor read the code rather than the commit message, enumerated every
read of both tables across the crate, and compiled a standalone f32
probe (40,001 samples) to test the cancellation empirically instead of
trusting the algebra on paper.

### 1. The blast-radius claim — **UPHELD.** No missed consumer.

Every read of either table in the whole crate (grep over `engine/`;
neither name appears in `sim.rs` or any other file):

| site | function | classification |
|---|---|---|
| main.rs:445,446 | `chain_segment_scale` | production, but its ONLY production caller is main.rs:524 → invariant |
| main.rs:457 | `chain_peak_tick` | **TEST-ONLY** — sole caller is main.rs:11536, despite compiling into the binary |
| main.rs:467 | `chain_lag_chase` | **the one live behaviour** — main.rs:6451 → `ls.lean_lag` → `chain_lag_rx` (6452) → `head_rx` (6455) → neck rotation |
| main.rs:523,524 | `spear_followthrough_yaw` | production via `torso_coil_yaw` main.rs:550 → invariant |
| main.rs:11171, 11555, 11566-11574, 11579-11580, 11599-11601, 11615-11617 | tests | test-only |

The cancellation was verified, not assumed. Over 40,001 samples of
`release_t ∈ [0, 0.40]`: the `elapsed_s < onset` branch is taken **0
times** (IEEE round-to-nearest is monotonic, so `fl(release_t + onset)
≥ onset` for every `release_t ≥ 0`; and `release_t < 0` is already
short-circuited at main.rs:520). `drive` is exactly `0.0` at
`release_t = 0` and exactly `1.0` for `release_t ≥ RAMP_S` (the clamp
makes it `peak * 1.0 / peak`, exact in IEEE).

**Correction to the claim's wording.** The cancellation is exact in ℝ,
**not in f32**. 8,290 of 40,001 samples (20.7%) leave a non-zero
residual, worst **1.788e-7** in `drive`, at `release_t = 0.0936`. Cause:
`(release_t + onset) - onset ≠ release_t` when the addition rounds
(worst ≈ ulp(0.14)/2 ≈ 7.5e-9, amplified by `/0.12`), plus the
`peak * x / peak` round-trip. Worst yaw consequence: **1.8e-8 rad** —
utterly invisible, so the *behavioural* claim stands. But main.rs:508
says the function "reduces **exactly** to `(release_t/RAMP_S).clamp(0,1)`"
and that word is wrong. It should read "to within 2e-7". This matters
because the new test's tolerance is `1e-6` (main.rs:11603) — only a
**5.6x margin** over the measured worst case, and the residual is
table-dependent. Anyone tightening that to 1e-7 gets a red suite.

**A defect the commit shipped:** main.rs:462 still reads "*it arrives at
a new acceleration lean ~one tip-onset (**0.125 s**) behind the pelvis*"
— five lines above main.rs:467, which now reads 0.130. A commit whose
own thesis is "*a reader must not infer a dependency that is not there*"
(main.rs:512) changed the constant and left the doc quoting the old
value. Direction, for the record: `dt / ONSET[7]` with a larger
denominator is a **slower** chase — the head now lags ~4% longer.

**Also unguarded:** `the_head_trails_a_sprint_start_then_settles`
(main.rs:11165) derives `onset_ticks` from `CHAIN_ONSET_OFFSETS[7]`
itself (main.rs:11171), so it self-adjusts and cannot detect this
retune. The single real behaviour change in the commit is pinned by no
test at all.

### 2. The arithmetic — **mostly sound; the word "exactly" is false twice.**

**Re-basing — EXACT, confirmed.** Campos 2004 Table 3 values −0.130 /
−0.090 / −0.060 / 0.000, plus 0.130 → 0.000 / 0.040 / 0.070 / 0.130,
which is `JAVELIN_ANCHOR_S` at main.rs:427-428 verbatim. The
marker→segment mapping (hip→pelvis, shoulder→clavicle, elbow→upper arm,
release→tip) is internally consistent with `CHAIN_SEGMENTS[7] = None`
("the weapon, the chain's output").

**Trunk split — CONFIRMED.** Recomputed in f64:
`I_MPT = 0.1633·(0.468·0.2155)² = 0.001661011`,
`I_UPT = 0.1496·(0.659·0.1707)² = 0.001893082`, shares
`0.46735 / 0.53265`, so 35 ms → **16.357 / 18.643 ms**. Spec §3.3
line 323-324 says 16.36 / 18.64. ✔

**Geometric compression — the spec's `q` is WRONG in the 4th decimal.**
`q = 0.8107` gives `q + q² + q³ = 2.00075449`, not 2. The actual root of
`q + q² + q³ = 2` is **0.8105357**. So `30q + 30q² + 30q³ = 60.0226 ms`,
not 60, and the geometric chain lands the tip at **130.023 ms**, not on
the anchor. Spec §3.3 line 329/332 claims it is "solved to land
**exactly** on the measured 130 ms release anchor" and line 336 asserts
"Tip = 0.1300 ✔ hits the measured anchor" — while the spec's own line
335 prints "Σ = **60.03** ms", contradicting itself two lines earlier.
**Immaterial to what shipped**: recomputing with the true root gives
94.316 / 114.025 / 130.000 ms, which rounds to the identical shipped
`0.094 / 0.114 / 0.130`. But "exactly" is not true, and a spec that
says "exactly" about a number its own line disproves is a spec that
will be trusted somewhere it shouldn't be.

**Shipped constants vs derivation — all match to 3 dp**, with two silent
round-downs worth naming: lumbar derives to 0.016357 and ships 0.016
(−0.36 ms); forearm derives to 0.09432 and ships 0.094 (−0.32 ms). Both
are ~1/50th of the disclosed 20 ms precision ceiling (main.rs:418-421),
so they are fine — but they are rounding, not derivation, and nothing
says so.

**The commit message overstates one corroboration.** "*The by-feel
author was within 15 ms everywhere and **exact on the thorax**.*" The
thorax (index 2 = 0.035) is **not independently derived** — it is
`clavicle 0.040 − the 5 ms floor`, and that floor is an authored
constant the spec itself calls "a monotonicity device, not a claim of
5 ms accuracy" (SPEC §3.3 line 318, echoed at main.rs:420-421). A 4 ms
or 6 ms floor yields 0.036 or 0.034. The by-feel value was not confirmed
by data; it was matched by a chosen constant. Reads as independent
corroboration in the commit message. It is not.

### 3. The new tests — **one is genuine but mis-named; one contains a provably dead assertion.**

**`spear_followthrough_is_invariant_to_the_chain_tables`
(main.rs:11591-11609): NOT vacuous, but it does not test the thing its
name claims.** `predicted_drive` (11597) is built from `RAMP_S` alone,
independent of both tables, so the test genuinely fails if
`chain_segment_scale` stops multiplying by `peak`, stops subtracting
`onset`, or stops being linear. That part of the commit's claim holds.

**But it never calls `spear_followthrough_yaw`.** Lines 11599-11601
*retype* production line 524. The coupling between the tables and the
production function is a **copied literal, not a call**. Every one of
these production breakages leaves the test green:
- main.rs:524 → `chain_segment_scale(TIP, release_t, RAMP_S)` (drop the
  `+ onset`). **This is the exact bug this codebase has already shipped
  once** — `handback/AUDIT.md:38`: "`chain_segment_scale` returned 0.0
  for the first 0.125 s". The one regression with a track record here is
  not covered.
- dividing by `CHAIN_PEAK_SCALE[6]`, or dropping the division.
- `const TIP` changing from 7.

And the spec asked for better: SPEC line 491 — "*Refactor
`spear_followthrough_yaw` so the peak scale is a parameter, **purely so
the inertness test can run**.*" That refactor was skipped and the
expression duplicated instead. The shipped test is weaker than the
spec's own design, in precisely the way the spec was trying to prevent.

Coverage is also thinner than "40 sample points" suggests: `RAMP_S =
0.12` and the step is 0.01, so **11 of 40 points** are in the
non-trivial ramp; 1 is the trivially-exact 0.0 and 28 are the
trivially-exact clamped 1.0.

**`the_kinetic_chain_still_hits_every_measured_javelin_anchor`
(main.rs:11552-11585): the anchor loop and the interpolation-window
asserts are genuine; the third assert is DEAD.**
- 11554-11561 (anchors vs `JAVELIN_ANCHOR_S`): real tripwire. Two
  separate consts (422-423 vs 427-428). Weak-ish — both are literals
  five lines apart in one file — but it is not a constant against
  itself. Commit's claim upheld.
- 11565-11576 (indices 1,2,5,6 inside their brackets): real; those four
  are not otherwise pinned.
- **11579-11584 (`gap_arm < gap_trunk`) cannot fail.** `gap_trunk` and
  `gap_arm` are computed only from indices 0, 3 and 4 — all three
  already pinned to 0.000/0.040/0.070 within 1e-6 by the loop 20 lines
  above. Given that loop passes, `0.030 < 0.040` is forced. The
  assertion is unreachable as a failure and carries zero information.
- Its stated meaning is also wrong at segment granularity: `gap_trunk`
  spans **three** hops (pelvis→lumbar→thorax→clavicle), `gap_arm` spans
  **one** (clavicle→upper_arm). Per hop that is 13.3 ms then 30 ms — the
  chain **expands** there. The comment "the chain's gaps must NARROW
  toward the tip, never widen" (11577-11578) is not what the shipped
  table does per segment, and is not what the code measures.

### 4. Regression check — **CONFIRMED, exactly as claimed.**

`cargo test --release -p jk_tdm` →
`test result: ok. 145 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out`.

### 5. Step 2's no-op claim — **credible in its architecture, false in its "bit-exact" wording, and its test list has a hole where the real risk is.**

Credible parts, verified against the real code:
- SPEC §0.3's rotation form matches main.rs:6430-6436 **exactly**:
  `Ry(0.045·sin·amp + spear_yaw + torso_aim) · Rx((torso_pitch + flinch
  + 0.07·settle + relaxed_e·0.05).min(0.185)) · Rz(sway_r)`.
- There is **exactly one** writer of the torso transform in the whole
  crate (main.rs:6418). No second path to keep in sync.
- The torso spawns at `(0, 0.63, 0)` under `root` (main.rs:3951-3954)
  with ~13 direct `.set_parent(torso)` sites (3961-4216) plus
  `armor_rig` (4212-4214), which carries the mech visor at local
  `(0, 0.885, 0.115)` (main.rs:3575). Co-locating the three pivots does
  leave every one of those locals valid. §0.2's central call holds.

**Concrete risks, in priority order:**

**R1 — "BIT-EXACT" is false, and I measured both ways it fails.**
(a) *Translation.* SPEC line 547 puts `breath - crouch_drop` on the
lumbar, so the composed Y becomes `hip_y + (breath - drop)`. Today
main.rs:6424-6425 computes `hip_y - drop + breath`. f32 addition is not
associative: 110 of 1000 realistic samples differ, worst **5.96e-8 m**.
Not bit-exact **even at Step 2**.
(b) *Rotation.* At Step 2 it *is* bit-exact, but only because
`yaw_pelvis = yaw_thorax = 0` makes two quats the exact identity. The
moment Step 3 splits the yaw, `Ry(a)·Ry(b)` vs `Ry(a+b)` differ in
**1360/2000** samples, worst component **1.19e-7**. SPEC line 50's
"bit-for-bit the legacy single-`torso` rotation whenever the yaw sum is
preserved" is therefore false as stated — and the spec's own Step 2 test
tolerance of 1e-6 (line 569) already concedes it. Land the claim as
"no-op within 1e-6", never "bit-for-bit", or the first person who writes
`assert_eq!` on a quaternion burns a day chasing nothing.

**R2 — the death-branch instruction is itself a visible change, inside
the step that claims to be a no-op.** Today the dead branch
(main.rs:6063-6077) `continue`s **before any rig part is written**, so a
corpse freezes with its last-alive torso rotation, mid-coil included.
SPEC line 564 says to add `pelvis.rotation = IDENTITY` and
`lumbar.rotation = IDENTITY` there. At Step 2 the lumbar carries the
**entire** legacy torso yaw (SPEC line 557), so this zeroes the corpse's
trunk yaw on the first dead frame — shoulders snap square on death. That
is new on-screen behaviour and it is nowhere in Step 2's no-op proof.
Also, the dead branch has **no `parts` access at all** today (only `tf`,
`root_vis`, `life`), so this is not the two-line add it is described as.
Either drop it from Step 2, or reset the thorax too and ship it as an
intentional, captured change.

**R3 — every line number in the spec is stale.** The spec was written
against 38c8ecc; Step 1 (787f6ff) shifted everything below it. Verified
against `git show 38c8ecc`:
`CHAIN_ONSET_OFFSETS` spec:272 says 396, actual **422** (+26);
`chain_lag_chase` spec:274 says 422, actual **466** (+44);
`spear_followthrough_yaw` spec:275 says 467, actual **514** (+47);
`leg[0].translation.y = hip_y` spec:528 says 6201, actual **6257** (+56);
the `rig.torso` block spec:528 says 6362-6381, actual **6418-6437** (+56).
An implementer applying Step 2 by line number edits the wrong lines.
Re-anchor the spec by symbol before Step 2 starts.

**R4 — three of the seven tests Step 2 lists as guard rails are blind to
the change, and the four new ones test the maths rather than the
plumbing.** `head_never_leaves_its_band_in_any_gait` (main.rs:10638)
samples the pure `head_base_y`/`gait_pose`;
`rig_joints_bridge_with_no_daylight` (main.rs:10587) asserts arithmetic
on hard-coded literals (`let yoke_top = 0.625 + 0.07;`). Neither touches
an entity, a parent link, or a composed transform — both pass whether or
not Step 2 is wired correctly. Of the four new tests, three exercise the
extracted pure `trunk_locals`. **The entire risk of Step 2 lives in the
spawn wiring** — reparenting ~13 `.set_parent(torso)` sites
(main.rs:3961-4216), `armor_rig` (4214), and both legs — **and not one
listed test would catch a missed or wrong reparent.** The no-op proof
covers the algebra; the failure mode is the plumbing. Add at least one
test (or a headless capture) that composes real `GlobalTransform`s for
head, `weapon_root`, `armor_rig` and `leg[0]`.

**R5 — the pitch clamp is inside the rotation expression and the spec's
sweep never reaches it.** main.rs:6434 applies `.min(0.185)` inline;
`trunk_locals` takes `pitch` pre-clamped, so the clamp must move to the
call site by hand. SPEC line 569 sweeps `pitch ∈ 0..0.185` — it stops
exactly at the cap and never exercises it, so dropping or
mis-transcribing `.min(0.185)` passes every listed test and silently
breaks the §0.2 head band that `gait_pose` exists to protect. The inputs
do exceed the cap: `torso_pitch_base` is **1.0** while rolling
(main.rs:6218) and **0.90** crouched (main.rs:924), plus `flinch` up to
~0.09 (main.rs:6399-6416). Sweep past the cap.

**R6 — legs under the pelvis: the spec anticipates the yaw but not the
swing axis.** Step 3b (SPEC line 651) correctly flags that legs-as-
pelvis-children means `PELVIS_LEAD_FRAC` twists the feet, and asserts
"`leg[0]` composed world yaw shifts by **exactly** `sep · 0.15`". But
leg[0]'s rotation is `Quat::from_axis_angle(swing_axis, thigh)`
(main.rs:6262), where `swing_axis` is a movement-frame vector built at
main.rs:6190 — an **off-axis** rotation. Composed with a pelvis `Ry(a)`
the result is not decomposable as a yaw shift; the swing *plane* rotates
with the pelvis. That assertion is not well-defined as written. Harmless
at Step 2 (pelvis = identity), which is exactly why it should be
redesigned now while it is cheap.

**R7 (minor) — SPEC line 572's mech-visor margin is wrong.** The
pass/fail is right: `(0.63 + 0.885 − 0.014)/1.78 = 0.84326 ≥ 0.82`, and
`MECH_SCALE` cancels out of the fraction. But the stated "margin 9.4 cm
at mech scale" is not: it is `(0.84326 − 0.82)·1.78·1.7 = 0.0704 m` =
**7.0 cm** (`MECH_SCALE = 1.7`, sim.rs:2386; `BODY_HEIGHT = 1.78`,
sim.rs:28). Comfortable pass, wrong number.

**R8 (minor) — the F4 joint-debug overlay** (main.rs:8434-8442)
enumerates `[rig.torso, rig.neck, rig.weapon_root]`. If `pelvis`/
`lumbar` join `FighterRig`, add them here or the overlay silently stops
showing the trunk chain it exists to show.

### What the commit got WRONG (the short list)

1. **main.rs:462 still says "0.125 s"** for the one constant this commit
   changed to 0.130. Shipped doc defect, in the file the commit was
   editing for documentation accuracy.
2. **"reduces exactly" (main.rs:508) is false in f32.** 20.7% of samples
   carry a residual, worst 1.788e-7. Behaviourally irrelevant; the word
   is still wrong, and it sits 5.6x from the new test's own tolerance.
3. **The spec's `q = 0.8107` is not the root of `q + q² + q³ = 2`**
   (true root 0.8105357), so "solved to land **exactly** on the 130 ms
   anchor" is false — it lands at 130.023 ms, as the spec's own "Σ =
   60.03 ms" admits. Shipped table unaffected.
4. **"exact on the thorax" is not corroboration.** Index 2 = 0.035 is
   `0.040 − the 5 ms authored floor`, not a derived or measured value.
5. **The dead assertion at main.rs:11579-11584** cannot fail, and its
   comment describes a distal compression the table does not have per
   hop (13.3 ms then 30 ms — it expands).
6. **`spear_followthrough_is_invariant_to_the_chain_tables` never calls
   `spear_followthrough_yaw`.** It duplicates line 524 instead. The
   commit says it means "the argument that licensed this retune fails
   loudly" — it does, but only for changes to `chain_segment_scale`, not
   for changes to the function in its own name. The spec asked for the
   refactor (line 491) that would have closed this; it was skipped.

**Net:** the load-bearing claim (item 1) is TRUE and no consumer was
missed — the riskiest part of the commit is the part that was checked
most carefully. Everything wrong here is in the *wording* of certainty
("exactly", "bit-for-bit", "invariant") and in one test that guards a
lemma while carrying the name of the theorem.

---

## 2026-08-03 — Verification pass 2 on rig Step 1: Friday's fix of the 7 defects (commit `1e774e1`)

Scope: `engine/crates/jk_tdm/src/main.rs` + `handback/AUDIT.md` only.
`SPEC_20_SEGMENT_RIG.md` deliberately NOT read — Toto/Friday were editing
it concurrently, and analysing a shifting file produces shifting findings.

**Suite confirmed by running it, not by reading a claim:**
`cargo test --release -p jk_tdm` -> **148 passed; 0 failed; 2 ignored**.
One test binary only (`--bin jk_tdm`); there is no second target hiding
tests. The 2 ignored are `sim.rs:9214 autoplay_report` and
`sim.rs:9387 diag_bailey`, on-demand diagnostics unrelated to the rig.
`main.rs` verified byte-identical to `1e774e1` (`git diff 1e774e1 --` empty)
even though HEAD has since moved to `0288ff0` — other agents committed
research files, not source, so this verdict is against the commit named.

### Every mutation reproduced. All eight rows: exact match.

I applied each mutation to the real file, ran the real suite, and reverted.

| # | Mutation | Friday claimed | **Thor measured** |
|---|---|---|---|
| M1 | drop `+ tip_onset` (main.rs:583) | 3 FAIL | **3 FAIL** — `carries_past`, `is_invariant`, `matches_its_hand_computed_curve` |
| M2 | same, on PRE-change code (`787f6ff`) | old invariance test ok | **ok** — the old test passed |
| M3 | forearm idx 5 `0.094`->`0.100` (main.rs:432) | 2 FAIL | **2 FAIL** — `geometric_root`, `hits_every_measured_javelin_anchor` |
| M4 | same, on PRE-change code | 145/145 passed | **145 passed; 0 failed** |
| M5 | drop `.min(1.0)` (main.rs:498) | 1 FAIL | **1 FAIL** — `head_lag_chase_pins_the_measured_tip_onset` |
| M6 | tip onset `0.130`->`0.125` | 3 FAIL | **3 FAIL** |
| M7 | hardcode `/ 2.611` for `/ tip_peak` | 1 FAIL | **1 FAIL** — `is_invariant` |
| M8 | hand idx 6 `0.114`->`0.115` (one ms) | 1 FAIL | **1 FAIL** — `geometric_root` |

Zero discrepancies. Friday's falsifiability table is **accurate as
written** — the first such table this session that survived reproduction
without a correction. The headline result is M3 vs M4: a 6 ms move of a
derived arm onset was **completely invisible** to the pre-change suite
and is now caught by two independent tests. That is a real gain.

Every mutation reverted via `git checkout --`; revert verified empty
after each one; final `git status` shows main.rs clean.

### The declared hole is REAL — and behaviourally inert. Friday's argument reaches the right answer for the wrong reason.

Constructed the two-sided mutation Friday described:
`spear_followthrough_yaw`'s `const TIP: usize = 7` -> `6` (main.rs:564)
**and** the test's `variants[0]` -> index 6 (main.rs:11831).

- One-sided (wrapper only): **1 FAIL** — the `to_bits()` assertion at
  main.rs:11843 fires. Friday's claim holds.
- Two-sided: **148 passed, 0 failed.** Escapes completely, as declared.

But Friday justified the escape being acceptable with the wrong argument
("bit-equality catches the realistic single-sided case"). The actual
reason it does not matter is stronger and Friday did not state it: **the
TIP index is not observable.** The whole point of the invariance proof is
that the output does not depend on `(onset, peak)` — so a wrapper reading
index 6 produces the same curve, which is exactly why the golden test also
stayed green. There is no behavioural regression available through that
mutation. Friday declared a hole that cannot leak.

The mutation class that *can* leak is structural, and it is closed:
`spear_followthrough_matches_its_hand_computed_curve` (main.rs:11878-11897)
reads **no table at all** — it calls the wrapper and compares to literals.
It is therefore immune to any edit of the invariance test's variant list.
M1 confirms it fires on the AUDIT bug. That independence, not the
bit-equality assertion, is what makes D6's fix sound.

### What Friday's table omits: the pre-change suite was NOT blind to the AUDIT.md #1 bug.

Friday's M2 row is correctly scoped ("old invariance test: ok") and is
true. But run the whole pre-change suite under that mutation and it is
**144 passed; 1 failed** — `spear_followthrough_carries_past_the_release_then_settles`
catches it, because with the onset dropped the curve's peak is exactly
`SPEAR_RELEASE_YAW` and the assertion demands `peak > SPEAR_RELEASE_YAW`.

This matters because main.rs:11871-11874 calls the drop "the bug in
AUDIT.md #1, and it is what the old test could not see", and
main.rs:11803-11806 calls D6 "the worst of the seven". Both sentences are
literally true of *that test*. A reader will still come away believing
the bug could have shipped again unnoticed. It could not have. **D6 took
detection of the AUDIT bug from one test to three — a real hardening, not
a rescue from zero.** Friday should have said so; it is the kind of
deflation that makes the rest of the report more credible, not less.

### Friday's three volunteered uncertainties: two are overstated, one is right.

Measured independently in a standalone f32 harness mirroring the consts.

**(a) `exp()` bit-stability — OVERSTATED. Not a genuine portability
hazard.** Friday called it "the most fragile thing it added". Measured:

- Worst `|f32 - pinned|` across the 7 golden rows is **5.96e-8** at
  t=0.06, i.e. **16.8x** headroom — not the 5.5e-8 / "~18x" the comment
  at main.rs:11869 states. Friday's own margin is slightly generous;
  the doc figure should read 5.96e-8 and 16.8x.
- Three of the seven rows (t=0.00, 0.03, 0.05) sit at or before `HOLD_S`,
  so `decay = exp(-0.0) = 1.0` **exactly** in every conforming libm.
  They have *zero* exp exposure. Friday's framing implies all seven are
  at risk.
- For the four rows that do call `exp`, I computed how far `expf` would
  have to be wrong to break the 1e-6 tolerance: **39, 37, 146 and 9399
  ulp** respectively. The binding constraint is ~37 ulp. glibc's `expf`
  is correctly rounded (<=0.5 ulp); musl and UCRT are within ~1 ulp. A
  toolchain would need a **catastrophically** broken `expf` to move this
  test. The test does not require bit-stability; it requires ~37 ulp,
  which is two orders of magnitude of slack on the axis that actually
  varies. Verdict: real mechanism, negligible risk, wrong risk ranking.

**(b) the 1.6x geometric-root margin — OVERSTATED, and misframed.**
Measured: index 5 lands 3.161e-4 from the table against 5e-4 (**1.58x**);
index 6 is 2.511e-5 (**19.9x**). Friday's numbers are right. The framing
is not. A margin is only a *risk* if the quantity varies — and this one
cannot. `the_arm_onsets_reproduce_an_independently_solved_geometric_root`
(main.rs:11746-11796) is pure f64 add/mul/div/compare: a 200-step
bisection over IEEE-754 doubles, **no transcendental, no libm call, no
platform-dependent operation anywhere in it.** It is bit-deterministic on
every conforming target. The 1.58x is not a stability margin; it is the
distance of the true 0.0943161 from the 0.0945 rounding boundary — a fact
about the anthropometric data, not about the test.

And the tightness is a **virtue**: it is precisely what makes M3 and M8
fail. Loosening that tolerance would delete the only test that catches a
1 ms move of index 6 (see D9 below). Leave it.

**(c) 7 numbers to retune — correct engineering. But Friday missed the
failure mode that actually threatens it.** Pinning a feel curve is the
right response to a bug (AUDIT.md #1) that was a *silent behavioural
inversion* — property-only tests are what let it ship, and 7 numbers is
cheap insurance. Agreed.

The unnamed risk: main.rs:11866-11868 says the golden values "were
computed in f64 outside the crate ... a table of results, not a
re-derivation." **Nothing enforces that.** The next person to retune will
most cheaply regenerate GOLDEN by printing the code's own f32 output — at
which point the test silently degenerates into a change-detector that
pins whatever the code does, bug included. That is the same class of
defect as the D6 test it replaced. A comment is the only guard. Note the
inconsistency: the geometric-root test solved exactly this problem
correctly, by deriving its reference *inside* the test from independent
inputs; the golden test carries literals. Both choices are defensible for
their subject matter, but the golden test's guard is the weaker one and
Friday listed the wrong worry (a) as its most fragile property. **(c)'s
real hazard, not (a)'s, is where this will break.**

### Two defects Friday did not flag

**D8 (minor, doc) — main.rs:552-554 compares a drive-term residual to a
yaw tolerance.** It says the 1.788e-7 residual "is 1.8e-8 rad of yaw once
scaled by `OVERSHOOT_RAD` ... but only ~5.6x below the 1e-6 tolerance the
invariance test uses." Mismatched units: 5.6x is `1e-6 / 1.788e-7`, i.e.
the *drive* residual measured against a tolerance applied to *yaw*. The
sentence itself supplies the correct conversion one clause earlier and
then does not use it. Measured end-to-end worst divergence across the six
variants is **2.98e-8 rad -> 34x** headroom, which matches main.rs:11821's
"33x inside the tolerance" — so the file states both 5.6x and 33x for the
same margin, six thousand lines apart. The error is conservative (it
argues *against* tightening, which is the right advice), so nothing is
operationally wrong. But this is a doc block whose entire purpose is
numerical precision, in a section written specifically to correct a
previous imprecision.

Verified: 12,000 samples on the 10 us grid, **8,290** with non-zero
residual, worst **1.788139e-7 at release_t = 0.09359**, and **exactly 0**
after saturation. Friday's counts are all correct.

**D9 (minor, comment) — main.rs:11718-11719 overstates what the new
per-hop assertion guards.** Its message reads "indices 5 and 6 are the
interpolated ones", implying both are covered. Measured:

- idx 5 `0.094`->`0.100`: per-hop assertion **fires** (main.rs:11716,
  hops `[0.030, 0.030, 0.014, 0.016]` — monotonicity broken).
- idx 6 `0.114`->`0.115`: hops become `[0.030, 0.024, 0.021, 0.015]`,
  still strictly decreasing — the per-hop assertion **passes**. Only
  `geometric_root` catches it.

Coverage is not lost (D3's test catches both, which is why M8 still
fails), but the D5 replacement guards index 5 and not index 6 at the
table's 1 ms resolution, and its own failure message says otherwise. This
is a much milder version of the sin D5 was raised to fix — an assertion
whose text claims more scope than it has.

### Production call sites re-verified (a test that guards dead code is not coverage)

- `sync_fighters` registered as a real Bevy system at main.rs:2779.
- `chain_lag_chase` main.rs:6510 -> `chain_lag_rx` -> `head_rx`
  main.rs:6512 -> `Quat::from_rotation_x` on the neck at main.rs:6569.
- `torso_coil_yaw` main.rs:6361 -> `spear_yaw` -> torso transform at
  main.rs:6490; its final branch is `spear_followthrough_yaw`
  (main.rs:609). Both chains reach a `Transform`. Live.

### Verdict

D1, D2, D5, D6, D7 fixes are **real and falsifiable**, proved by mutation
rather than by inspection. Friday's falsifiability table reproduced 8/8
with zero corrections, and its numeric claims (8,290 / 1.788e-7 / 2.98e-8
/ 3.16e-4 / 1.6x / <=8e-8) all verified to the digit. That is the best
evidence quality any agent has handed me this session.

What Friday got wrong is smaller than usual and all of one kind — **risk
ranking, not fact**: it flagged (a) and (b), which measurement shows are
near-zero risks, and did not flag (c)'s regeneration hazard, D8, or D9,
which are where this will actually erode. And it declared a "remaining
hole" (the two-sided mutation) that is real as a mutation-testing fact
but cannot produce a behavioural defect, while under-reporting that the
pre-change suite already caught the AUDIT bug by another route.

**Net: the fixes are sound. Friday's self-assessment is honest and
slightly miscalibrated — it was hardest on the parts that hold up best.**
Nothing here blocks Step 2. Recommended follow-ups, all minor: correct
main.rs:11869 to 5.96e-8/16.8x, reconcile the 5.6x-vs-33x contradiction
(D8), and fix the per-hop assertion's message (D9).

All mutations reverted; `git status` clean for `engine/crates/jk_tdm/src/main.rs`.

## 2026-08-03 — VERIFY §C: the hull gatling + autocannon (`9b26280`, `5bd2ab7`)

Baseline re-verified by me before and after every mutation: **161 passed,
0 failed, 2 ignored**. All source mutations reverted with targeted
`git checkout --`; `sim.rs` and `jk_core/src/timestep.rs` clean at exit.

### 1. The defect Friday found — fix is COMPLETE. Agree.

`hitscan_burst` (sim.rs:5485) is the **only** production caller of the
hit chain, and it passes `damage` through to `apply_hit_dmg`. I searched
every `gun(` in the file: no other re-derivation survives on the path.
`apply_armor`/`apply_armor_tagged` take `base` as a parameter (6905-6913,
7070) and never re-read; `damage_zombie` takes `dmg`; the zombie branch
(5477, `damage * mult`) already carried the parameter.

Two corrections, both minor and one in Friday's favour:

- **Under-claim.** The commit describes ONE re-derivation site. There
  were **two** — the zone multiplier (now 5699, `base_dmg * ...`) and the
  armour floor (now 5711, `apply_armor(j, dmg, base_dmg, ...)`, formerly
  its own `let base_dmg = gun(self.fighters[i].gun).damage`). Both were
  live, both were fixed in the same commit.
- **Overclaim, cosmetic.** "all 20-odd existing call sites keep their
  meaning and read unchanged" is true, but **every one of them is inside
  `mod tests`** (which starts at 7588). After this change `apply_hit` has
  *zero* production callers. The wrapper exists for the test suite only.
  Not a defect; the commit message implies a production surface that is
  not there.

### 2. The heat-decay gate — Friday's INSTINCT is right, its MECHANISM description is wrong. HIGHEST-VALUE ITEM.

Measured, trigger held through `step()`, SIM_HZ=120:

| configuration | time to forced vent | × minigun |
|---|---|---|
| **shipped (gated on `fire_cd`)** | **9.083 s** (122 shots, 1090 ticks) | **2.06×** |
| decay ungated | 29.79 s | 6.74× |
| no decay at all | 7.78 s | 1.76× |
| minigun, measured the same way | 4.417 s (67 shots) | 1.00× |

So the design intent ("sustains ~2× the minigun") **is met** — 2.06×. And
Friday's ~7.8 s / ~30 s figures are correct. But:

- **"mirroring how the minigun gates its decay on the trigger hold-timer"
  is wrong.** `spin_cmd` is set at the TOP of `try_fire` (5216), *before
  every early return including `fire_cd > 0.0`*, so while the trigger is
  held it is refreshed every tick and the decay branch never runs —
  **100 % suppression**. `fire_cd <= 0.0` is true for exactly **one tick
  per fire cycle** (decrement 3225 → gate 3318 → fire 4131, same tick).
  The gatling therefore cools by `GATLING_HEAT_DECAY * DT` = 0.0792 per
  shot — **88.9 % suppression, not 100 %**. Hence 122 shots to reach 100
  heat instead of 111.
- **"a barrel group under fire does not cool"** is not what the code does.
- **"~8×" the minigun ungated is an overclaim** — measured **6.74×**.
- **"~9.4 s" sustain** — measured **9.083 s**.

**Tick-granularity dependence: CONFIRMED, and worse than Friday guessed.**
Because the surviving decay is one tick's worth per shot, it is *linear
in DT*. Measured by rebuilding at three tick rates:

| SIM_HZ | sustain |
|---|---|
| 240 | 8.221 s |
| **120 (shipped)** | **9.083 s** |
| 60 | 11.183 s |

A **36 % swing** in the headline number from the tick rate alone. Note
the irony: the *ungated* version is the tick-rate-stable one (ramp and
decay are both per-second rates); **the gate is what introduces the DT
dependence.** Related and undisclosed: `ticks_per_shot` measured 8.93,
not the 8.4 the constant implies — the real fire period is
`ceil(0.07/DT)*DT` = **0.075 s**, so the gatling's ROF is 800 RPM, not
the 857 `GATLING_FIRE_PERIOD` reads as.

**Is `fire_cd` shared in a way that can misfire? Yes — four vectors.**

- **(i) Live — the dismount throttle.** Measured: after one gatling round
  `fire_cd = 0.07`, and `try_fire` on foot immediately after returns
  **false**. The hull mount's cycle clock throttles the pilot's carried
  gun for up to 0.07 s after ejection. `autocannon_cd` exists as its own
  field *precisely to avoid this*, and the file says so at
  **sim.rs:1399-1401**: *"Its own field, not `fire_cd`, so the two mounts
  cannot silently share a cooldown."* The gatling is the lone violator of
  a principle stated three lines above the field it violates.
- **(ii) Live, and NOT disclosed by Friday — the mounts freeze the
  pilot's carried-gun spray index.** `try_fire_gatling` (5554) and
  `try_fire_autocannon` (5598) both write `f.last_shot_at`. `step`
  decays the carried gun's `spray_i` only when
  `t_now - f.last_shot_at > gun(f.gun).fire_period * 1.1` (3370-3374).
  The gatling refires every 0.075 s; every carried gun but the minigun
  has a threshold above that (M4 0.099, AK 0.1155, Deagle 0.462). So
  **`spray_i` never decays while the hull gatling fires.** Mech entry
  (3592-3616) resets the mech fields but not `spray_i`. Board mid-burst,
  hold the gatling, dismount → the carried rifle resumes at its old spray
  index where an idle pilot would get a full reset. Same defect class as
  (i), second instance, undisclosed. (The autocannon's 1.35 s cycle is
  slower than every threshold, so it is harmless here.)
- **(iii) Latent — stale heat on a dismounted pilot.** Measured: a
  dismounted pilot carrying 50.0 stale `gatling_heat` cools to **40.50**
  in 1 s idle but only to **49.13** while firing his carried rifle — the
  carried gun's `fire_cd` suppresses 91 % of the cooling. Not observable
  today because the only production route into a chassis is the
  `RobotArmor` pickup (3592), which zeroes the mount state, as does death
  (3452). It goes live the moment a second boarding route (the
  `research/mech-climb` work) lands without that reset.
- **(iv) Bots — benign today, armed for tomorrow.** A bot in a mech calls
  `try_fire` (7392, ungated on `in_mech()`), setting `fire_cd` from its
  *carried* gun. Its `gatling_heat` is 0, so the gate is a no-op. The
  instant item 4 is fixed, that carried gun's `fire_cd` will gate both
  the gatling's fire and its cooling. **Item 2 must land before item 4.**

**Weapon switching mid-heat behaves sanely** — this part is clean. Key
1/2 writes only `mech_weapon` (3756-3757); the decay/vent block (3312-
3320) sits outside the `mech_weapon` match, so heat keeps cooling and a
forced vent keeps draining while the autocannon is selected. Correct, but
undocumented and untested.

**RECOMMENDATION.** Keep the gate's intent; replace its mechanism.

1. **Yes, add the dedicated field — but as a trigger-HOLD timer, not a
   cooldown**, set at the top of `try_fire_gatling` before every early
   return, exactly as `spin_cmd` is at try_fire:5216. That is the only
   shape that delivers Friday's own stated rationale, and it removes the
   DT dependence: suppression becomes 100 %, sustain becomes
   `ceil(0.07/DT)*DT * ceil(100/0.9)` ≈ **8.4 s = 1.90×** the minigun —
   still "~2×", and stable to within one tick across tick rates.
2. **Also move the cycle clock off `fire_cd` onto its own `gatling_cd`**,
   mirroring `autocannon_cd`. Not gold-plating: it enforces the rule the
   file already wrote at 1399-1401, and it is what kills (i).
3. **Decide (ii) separately** — cheapest correct fix is to stop writing
   `last_shot_at` from the mounts unless something mech-side reads it.
4. **Do item 3 (the test) FIRST.** None of these numbers is currently
   observable to the suite, so any change here is unfalsifiable.

*Rejected alternative, recorded so it is not re-proposed:* gating decay on
`t - last_shot_at` needs no new field and is tick-rate stable, but
`last_shot_at` is written by `try_fire` too (5307) — it reproduces exactly
the sharing defect being fixed.

### 3. The missing sustain test — AGREE, and the hole is far bigger than Friday stated.

Confirmed absent. `gatling_vent_t` is never set to a non-zero value
anywhere in `mod tests` (only zeroed, 10007 and 10109). Two of my own
mutations prove the scope:

- Ungate the decay (3318, `} else if f.fire_cd <= 0.0 {` → `} else {`) —
  the exact 9.08 s → 29.79 s regression Friday's whole argument is
  about → **161 passed, 0 failed. SURVIVES.**
- Delete the forced-vent latch (5556-5559) *and* the `gatling_vent_t >
  0.0` lockout (5532) — i.e. remove the entire forced-vent mechanism and
  make the gatling infinite-sustain → **161 passed, 0 failed. SURVIVES.**

**The vent does not exist as far as the suite is concerned.** What the
test should assert, stepping `step()` with `shoot: true` held on a sealed
chassis:

1. `gatling_vent_t` latches to `GATLING_VENT_FORCED_S` after a bounded
   elapsed time — pinned as a **range** (e.g. 8.0..10.5 s), not a point,
   so it survives a tick-rate change but still fails on 7.8 s or 29.8 s.
2. That time is `> 1.8×` and `< 2.4×` the **measured** minigun
   time-to-vent, through the same held-trigger loop. Measuring both sides
   is what keeps it from becoming another constants-ratio assertion.
3. `try_fire_gatling` returns **false** for the whole vent window, and
   `gatling_heat` is exactly 0.0 on the tick the vent clears.
4. The pilot's carried `gun`, `ammo` and **`spray_i`** are unchanged
   across the burst — this is the regression test for cross-talk (ii),
   and it would have caught it.

### 4. Bot parity — the call is defensible as SCOPING; the description understates the damage badly.

Measured: seeded a bot into a sealed chassis, ran 2 s of `step()`. Result:
`gatling_heat = 0`, `hits_dealt = 4`, `ammo = 26/30`, carried gun
`Ak47`. It fires its **carried AK-47 (13.5 dmg)** from inside a 1000-hull
chassis and never touches either mount. Concretely, a bot mech:

- **Reloads.** 2.2 s of total inactivity per 30 rounds. Hull mounts never
  reload.
- **Runs out.** `bot_act` gates firing on `ammo > 0` (7378) and
  `try_reload` needs reserve; AK-47 is 30 + 120 = **150 rounds**. After
  that a bot mech is **permanently disarmed while sitting in a
  full-health chassis** — a 1000-hull obstacle that cannot fight. The
  hull mounts consume no ammo at all.
- **Cannot threaten another mech.** 13.5 through the front 85 % cut is
  2.0/shot against the autocannon's 21.75. ~50 s of uninterrupted fire to
  kill a player mech.
- Bots do reach mechs in production — the pickup loop is
  `for i in 0..self.fighters.len()` (3556), not player-only.

So it is not "the bot fires the wrong gun"; it is "**a bot mech is a
rifleman with 1000 HP who eventually disarms himself permanently**."
Given this file's record — turn rate, acceleration and §A's brace all
shipped player-only first and all had to be re-wired — I judge this the
**highest-priority §C follow-up after the sustain test**. Friday's
determinism concern is legitimate (wiring the mounts into `bot_act`
consumes RNG in `hitscan_burst` and moves every seeded replay), but that
argues for doing it deliberately and soon with the seeds rebaselined in
one commit, not for deferring it behind more §C work that enlarges the
reseed. **And it must land after item 2**, or the `fire_cd` sharing
becomes live for bots (see 2(iv)).

### 5. The unasked noise emission — wired CORRECTLY, and it needs a test.

Verified: `try_fire_gatling` 5563-5568 and `try_fire_autocannon`
5612-5615 both call `emit_noise(at, gun_noise_m(GunKind::Minigun))` under
`mode == Mode::Extraction`, after the hitscan, with
`at = [pos[0], pos[2]]` — the `[x, z]` shape `emit_noise` expects
(6455-6465). Identical in form to `try_fire`'s own call at 5377-5381.
`gun_noise_m(Minigun) = 95.0` (2788). Friday's framing is right: a radius
*lookup*, read-only, not an entry into the `GunKind` pipeline.

**It needs a test, and mutation proves why.** Deleting the entire
Extraction/`emit_noise` block from BOTH mounts — a completely silent mech
against the horde director, the exact defect Friday added this to prevent
— gives **161 passed, 0 failed. SURVIVES.** A one-line test (fire a mount
in Extraction with a zombie inside 95 m, assert `z.alerted`) closes it,
and also catches the likelier future slip: someone "tidying"
`GunKind::Minigun` to a quieter weapon.

### 6. Mutation table — MY OWN runs, not Friday's claims

| # | mutation | site | result | Friday claimed |
|---|---|---|---|---|
| a | `apply_hit_dmg(i,j,hit_y,end,damage)` -> `apply_hit(i,j,hit_y,end)` (restore the real defect) | 5485 | **160/1** — `hull_mounts_carry_their_own_damage_down_the_shared_hit_path` | 159/1 |
| b | `!f.in_mech()` -> `false`, both fire fns | 5528, 5583 | **160/1** — `mech_weapons_refuse_to_fire_for_non_mech_fighters` | 157/1 |
| c | delete the `mech_weapon != Gatling` clause | 5529 | **160/1** — `autocannon_and_gatling_are_mutually_exclusive_by_mech_weapon` | 157/1 |
| d | *(mine)* ungate heat decay | 3318 | **161/0 — SURVIVES** | — |
| e | *(mine)* delete both mounts' Extraction noise | 5563, 5612 | **161/0 — SURVIVES** | — |
| f | *(mine)* delete the forced-vent latch + lockout | 5532, 5556-5559 | **161/0 — SURVIVES** | — |

Friday's 157/1 and 159/1 are consistent with the suite as it stood at
`9b26280` (149 tests) and `5bd2ab7` (160); the baseline is 161 after
`d6aa356`. **Every reproduced mutation killed exactly one test, and the
right one.** The counts moved; the conclusions did not.

### 7. Attacking the tests

Four of six are genuinely falsifiable and mutation-proven above. Two have
problems, and one of those explains why §C's headline mechanic went
unguarded.

**`gatling_heat_ramps_slower_than_minigun_in_absolute_terms` — its third
block is self-referential AND its comment is false** (10083-10092):

```rust
let gat_s = 100.0 / (GATLING_HEAT_PER_SHOT / GATLING_FIRE_PERIOD);
let mini_s = 100.0 / (MINIGUN_HEAT_PER_SHOT / gun(GunKind::Minigun).fire_period);
assert!(gat_s > mini_s * 1.5, "hull gatling cooks off after {gat_s}s of fire ...");
```

It steps nothing. It rebuilds `heat_per_shot / fire_period` from four
constants and compares. The comment calls it *"The same relationship
expressed as TIME TO A FORCED VENT"* and the message says *"cooks off
after {gat_s}s"* — both false: `gat_s` = **7.78 s**, the shipped time to
a forced vent is **9.08 s**, because the expression ignores both the
decay and the tick granularity. A constants-ratio assertion wearing a
measurement's label — and it is the *only* thing in the suite that
mentions time-to-vent, which is very likely why mutations (d) and (f)
survive. The test's first two blocks are fine and do measure off the real
fire paths.

**`autocannon_kick_is_damped_exactly_by_mech_brace_recoil_damp` is partly
tautological but acceptable.** `(braced - unbraced *
MECH_BRACE_RECOIL_DAMP).abs() < 1e-6` rebuilds the source expression at
5603-5607 and cannot fail for any value of the constant. It does catch
real things — `mech_brace` unread, wrong sign, wrong `punch_vel` axis, a
second independent braced constant appearing — and measures through the
real fire path; the companion assertions are the falsifiable half. Weak,
not worthless; worth knowing it pins the derivation, not a behaviour.

**`gatling_spread_widens_with_heat` is the strongest of the six** — 300
samples off real tracer geometry, heat pinned at both ends, constants
used as *bounds* rather than as expected values.

**One gap in the otherwise-excellent test 6:** it exercises only the
fighter/hull branch. `hitscan_burst`'s zombie branch (5477,
`damage * mult`) stays untested — which is precisely the branch Friday
says "would have hidden the split indefinitely." Cheap to add; it is the
branch that hid the bug.

### Verdict — what Friday got wrong or overclaimed

1. **"mirroring how the minigun gates its decay on the trigger
   hold-timer"** — wrong. `spin_cmd` suppresses decay 100 % while held;
   `fire_cd <= 0.0` lets one decay tick through per shot (88.9 % at
   120 Hz).
2. **"a barrel group under fire does not cool"** — not what the code
   does; it cools 0.0792 per shot.
3. **"~8×" the minigun ungated** — overclaim; measured **6.74×**.
4. **"~9.4 s" sustain** — measured **9.083 s**.
5. **"all 20-odd existing call sites"** — all of them are in `mod tests`;
   `apply_hit` has zero production callers.
6. **Missed:** the `last_shot_at` cross-talk — both mounts freeze the
   pilot's carried-gun spray index (2(ii)).
7. **Missed:** the real fire period is 0.075 s, not 0.07 s — 800 RPM, not
   857.
8. **Understated:** the missing test. Not just the sustain number — the
   entire forced-vent mechanism can be deleted with a green suite.
9. **Understated:** bot mech behaviour. Not "fires the wrong gun" but
   "permanently disarms itself after 150 rounds inside a 1000-hull
   chassis."
10. **Under-claimed, in its favour:** it fixed two re-derivation sites and
    described one.

Where Friday deserves credit: **the defect it found is real and the fix
is complete** — I could not find a surviving re-derivation anywhere on
the hit path. Test 6's ratio design is the best-built test in §C. Keeping
`MechWeapon` out of `GunKind` holds up under reading, as does the refusal
to add `AUTOCANNON_BRACED_KICK`. And its instinct that the `fire_cd` gate
is the weakest thing it shipped is exactly right — it simply
under-diagnosed *why*: not the balance number (2.06× is on intent), but
that the number is accidental, tick-rate dependent, and riding a field
the file itself forbids the sibling mount from riding.

**Net: §C's hit path is sound and its gates are proven. §C's heat/vent
system — the gatling's entire identity — is unproven end-to-end, and the
one line holding it up is the one Friday flagged.** Ship order:
(3) sustain test → (2) dedicated trigger field + `gatling_cd` →
(4) bot parity → (5) noise test.

All mutations reverted; `git status` clean for
`engine/crates/jk_tdm/src/sim.rs` and `engine/crates/jk_core/src/timestep.rs`.

---

## 2026-08-03 — §4.6 crosshair: Friday's #1 uncertainty resolved from source

Friday shipped the crosshair settings family (180 tests, 20 mutations
applied and all 20 killed, verified by counting PIXELS in a real
capture rather than trusting the suite — 44 green pixels at exactly
(50,250,50) at defaults, and a non-default `config/settings.txt` with
size 10 / gap **−4** / T-shape produced device-space arm positions
matching its prediction exactly).

It flagged five uncertainties and named one as the thing it most wanted
checked:

> **The kill-pop X is uncaptured.** It relies on Bevy UI honouring
> `Transform.rotation`. No capture script reaches a player kill, so
> there is no PNG of the X. If rotation no-ops, the kill confirm
> degrades silently to just the orange flash.

**RESOLVED — it works, verified from the Bevy 0.15.3 source, not
assumed:**
- `bevy_ui/src/layout/mod.rs:413-414` writes **only**
  `transform.translation`; rotation is never touched by the layout
  system.
- `bevy_ui/src/render/mod.rs:290,381,459,492` extract via
  `transform.compute_matrix()`, and :653 via `global_transform.affine()`
  — the FULL affine, rotation included.

So a rotated UI node renders rotated. The kill-pop X is real.

**Still genuinely open from Friday's list, recorded rather than
waved through:** `crosshair_render` itself is Bevy-side and untested
(a swapped arm index would pass the whole suite — every pure function
it calls is proven, but the piece-index→rect mapping is not);
`CROSS_SPREAD_PX_PER_RAD` replaced a bare literal in a system no test
covers; and the dynamic bloom rate, the outline's 0.75x alpha coupling
and all four clamp ranges are Friday's own judgement, not specified.

**Process note worth keeping:** Friday's menus capture caught a defect
that its own code comment had asserted away — a 51-character row it had
claimed fit was still wrapping. It re-measured, fixed it, and changed
the comment to say "measured, not assumed". That is the confident-
narrator anti-pattern being caught by evidence in the same session it
was written.

---

# 2026-08-08 - THOR verdict on the ~19-commit visual/gameplay session (HEAD 568a42a)

Scope: the five items dispatched - visual claims, claimed bug fixes,
the S7 FP-aim audit, a test-quality sweep with real mutations, and a
hunt for unverified claims dressed as verified.

**Method note, read this first.** Two builders were writing `sim.rs` and
`main.rs` throughout this run (sim.rs mtime moved twice while I worked;
line numbers drifted ~180 lines in sim.rs and ~18 in main.rs mid-audit).
I therefore did **all** mutation work in an isolated copy of the
workspace at `<scratchpad>/eng`, restoring from `<scratchpad>/pristine/*.rs`
between mutations. **The repo's source files were never written to.**
Baseline in the copy: 328 passed / 0 failed / 2 ignored, identical to
the repo.

Second method note: the shared `engine/target` dir was under memory
pressure from three concurrent cargo processes. Two mutation runs came
back with bogus "could not compile bevy_reflect_derive (472 errors)" and
"rustc-LLVM ERROR: out of memory". **Those were the instrument, not the
code** - both re-ran clean. Any compile error seen in this environment
must be re-run before it is believed.

## THE ONE THING THAT IS WRONG: the LMB grenade throw is dead

`main.rs:15446-15447` (live repo):

    if game.nade_ready && buttons.just_pressed(fire_btn) {
        game.nade_ready = false;
    }

...runs **before** `main.rs:15483`:

    throw_hold: (game.nade_ready && buttons.pressed(fire_btn))
        || keys.pressed(KeyCode::KeyH)
        || buttons.pressed(MouseButton::Back),

In Bevy, `just_pressed` is a subset of `pressed`: on the frame LMB goes
down both are true. So the clear fires first, `nade_ready` is false by
line 15483, and `throw_hold` is false on that frame and on every held
frame after (nothing re-sets `nade_ready` except G). `cook_t` never
accrues, so the sim's release branch (`else if cook_t > 0.0`) never runs
and `grenades[sel] -= 1` never executes.

**Net: press G, hold LMB, release - nothing is thrown, ever. The grenade
stays in hand and you fire your rifle instead.**

This is the exact sequence commit `6a46f61` documents as the new
contract ("G puts it in your hand and does nothing else, LMB winds,
releasing LMB throws").

Confirmed twice, independently:
1. Code reading + Bevy `ButtonInput` semantics.
2. `git show 6a46f61 -- crates/jk_tdm/src/main.rs`: the diff changed
   **only** `!buttons.just_pressed(fire_btn)` to `buttons.pressed(fire_btn)`.
   The clear-on-just_pressed block was pre-existing and is what made the
   OLD expression correct (it produced the falling edge). Left in place,
   it kills the new one on frame one. Classic: the fix moved the edge and
   left the edge-consumer behind.

Still working: `H`, `Mouse4`, and the accidental order **hold LMB first,
then tap G** (G is handled at 15384-15392, before the clear, and
`just_pressed(LMB)` is false by then). That last one is why this could
survive a casual test.

No test covers this. Every sim test sets `throw_hold: true` on a
`PlayerCmd` directly (sim.rs:16708, 16946, 17619, 17727, 21326), so the
client wiring is entirely unguarded. That is the real gap.

## S7 first-person aim: the audit is OVERSTATED, not wrong

The commit says hypothesis (1) is cleared because "`muzzle_origin`
returns the EYE" and "when muzzle == camera the second stage is the
identity". That is true for exactly one case: an on-foot fighter
standing up.

FP camera eye (`main.rs:17833-17845`) vs `muzzle_origin`
(`sim.rs:8158`, `pos[1] + EYE_REL.min(height()-0.12)`):

| stance | camera eye offset | muzzle offset | delta |
|---|---|---|---|
| standing on foot (h 1.78) | h-0.16 = 1.62 | min(1.62,1.66) = 1.62 | **0.000 m** |
| crouched (CROUCH_HEIGHT 1.15) | 0.99 | min(1.62,1.03) = 1.03 | **0.04 m** |
| rolling (ROLL_HEIGHT 0.95) | 0.79 | 0.83 | **0.04 m** |
| heavy mech standing (h 3.026) | visor_eye_y = 2.723 | 1.62 | **1.103 m** |
| heavy mech kneeling (S21, h 2.179) | 1.961 | 1.62 | **0.341 m** |

The mech row is the one that matters. `visor_eye_y = pos + height()*0.90`
(sim.rs:3008 / MECH_VISOR_Y_FRAC sim.rs:4620) but every mech weapon
fires from `muzzle_origin`: gatling (~sim.rs:8800), autocannon (~8886),
plasma (~8945, ~9013), rockets (~9162), repair-beam pick (~9088).
**A pilot in first person is looking from 2.72 m and shooting from
1.62 m.** With nothing in the aim ray's path, `crosshair_aim_dir` falls
back to `t_hit = 200.0` (main.rs:1197ff) and the shot converges only at
200 m - at a target 40 m out that is ~0.88 m low. The barrel is also
below the top of low cover the camera clearly sees over. That is
precisely the "crosshair lies about where the bullet goes" failure the
audit declares cleared.

I am not calling this a regression - the two-stage aim is deliberate and
this offset predates the audit. I am calling the **claim** wrong: "FP aim
is geometrically exact" is true for a standing soldier, false for a
crouched one, and badly false for a pilot.

### The surviving test is NOT self-referential in the way that matters

`a_first_person_shot_goes_exactly_where_the_crosshair_points`
(main.rs:24001). Mutation-proved, both directions:

- **M8** - add `+ 0.40` to `muzzle_origin`'s Y (the barrel offset the
  test exists to catch): **KILLED**. The fix to the first, vacuous
  version is real.
- **M7** - `EYE_REL` 1.62 -> 1.20: **SURVIVED**. The test rebuilds
  `EYE_REL.min(f.height()-0.12)` verbatim, so both sides move together.

So it pins the **formula**, not the **constant**. Weaker than advertised,
but it does bite on the stated threat model. Note also it sets
`lean = 0.0` and never crouches or boards a mech - it cannot see any row
of the table above except the first.

## The visual claims

**(a) Four medic trims render distinguishably - PARTLY SUPPORTED.**
Code is real and consumed: `MechTrim::ALL[i % 4]` drives
`spawn_scout_chassis`, which reads `trim.limb_scale()` (as `lt`) and
`trim.wears(...)` for the four optional plates. Plate counts 0/1/3/4 are
all distinct, so no two trims can be identical.
But the CAPTURE does not establish it.
`handback/brief-vii/trims/01-trims-front.png` cuts the fourth machine off
at the right frame edge and buries the lower halves under the
"+100 MEDIC HULL 210" panel; `02-trims-quarter.png` strings the four
along a depth gradient so apparent size is dominated by perspective, and
the two furthest read as identical. The script's own comment claims the
lineup is the instrument ("if this lineup ever shows two identical
machines, the indexing is what is wrong"). At this framing the
instrument cannot resolve that.

**(b) Hull gatling barrels spin in third person - OVERSTATED, and the
S32 defect it claims to fix has been MOVED, not removed.**
`spawn_armor_rig` (called per fighter, main.rs:12773) gives **every**
mech hull a `MechTurretSpinner` node. `spin_mech_turret_barrels`
(main.rs:10321) queries `With<MechTurretSpinner>` with **no per-fighter
filter** and reads only `game.sim.fighters[game.sim.player]`. Therefore:
- player on foot -> `rate = 0.0` -> early return -> **every bot mech's
  barrels are frozen even while it is firing**;
- player holds the trigger -> **every bot mech's barrels spin up**,
  firing or not;
- the `RobotArmor` pickup totem (main.rs ~11541, "stage parts never
  hide") spins with the player's trigger too.

The commit says this fixes "a weapon correct in one view and broken in
the other". It fixed it for the player's own body and recreated it for
everyone else's.

**(c) Fingers on the PLAYER, mitten on bots - HOLDS.**
`is_player = i == sim.player` (main.rs:12576) -> `weapon_detail` ->
`spawn_world_hand_fingered` under `if weapon_detail` (main.rs:12462),
`else` a single ball. Arm hardware gated the same way at main.rs:12490.
The Forge turntable passes `true` deliberately.
**But the commit's stated verification is false - see item 5.**

**(d) Royal variant 1-in-4 by slot index - HOLDS.**
`spawn_armor_rig(commands, kit, i % 4 == 1)` at main.rs:12773, inside
`for (i, f) in sim.fighters.iter().enumerate()`. Index-derived, never
rng, so replay-safe as claimed.

## The claimed bug fixes - all five in place, one of them non-functional

1. `weapon_root` hidden in a mech - main.rs:17030,
   `for e in [rig.neck, rig.arm_l[0], rig.arm_r[0], rig.weapon_root]`
   set from `f.armor_set.is_mech()`. HOLDS.
2. `let in_mech = f.in_mech();` - main.rs:15952, replacing the
   `== ArmorSet::RobotSuit` that excluded the medic. HOLDS.
3. Medic no longer wears the heavy's leg armour - the leg-armour loop
   just below main.rs:17030 is gated on
   `f.armor_set == ArmorSet::RobotSuit` specifically while the body hide
   uses `is_mech()`. The distinction is deliberate and correct. HOLDS.
4. Grenade wind-up at the trigger - the *expression* is in place at
   main.rs:15483 but **the feature is broken**, see the top of this
   entry. Counts as NOT HELD.
5. Spear halving deleted - no `SPEAR_V0_MIN` branch survives in
   `try_fire`; the only remaining speed modifiers are the S5.4 running
   bonus and `spear_power`, both applied to player and bot alike. The
   commit's claim that `SPEAR_V0_MIN` now equals the full speed checks
   out numerically: both are
   `22.0 * MISSILE_SPEED_MULT * SPEAR_SPEED_EXTRA` (sim.rs:350 vs the
   Spear `GunSpec.projectile`). HOLDS.

## Test-quality sweep (all mutations run in the isolated copy)

| # | mutation | result | reading |
|---|---|---|---|
| M1 | barrier FILL alpha 0.085 -> 0.90 | **SURVIVED** | vacuous |
| M2 | `BARRIER_SCALE` 1.60 -> 1.20 | KILLED | real |
| M3 | barrier field disc 1.70*SCALE -> 0.80*SCALE (production only) | **SURVIVED** | vacuous |
| M4 | `MechTrim::Field` limb_scale 1.00 -> 0.84 | KILLED | real |
| M6 | `MechTrim::wears()` -> always `true` | **SURVIVED** | uncovered |
| M7 | `EYE_REL` 1.62 -> 1.20 | **SURVIVED** | formula-only |
| M8 | `muzzle_origin` +0.40 barrel offset | KILLED | real |
| M9 | `MECH_CROUCH_HEIGHT_FRAC` 0.72 -> 1.00 | KILLED | real |
| M10 | `chassis_kneeling` drops the jump phases | KILLED | real |

M5 - stubbing `let lt = trim.limb_scale()` - could not be made to compile
cleanly under the OOM conditions above. **PROVISIONAL / never verified.**
M6 covers the same claim from the other dial and survived, so the
conclusion below stands on M6 alone.

**The S21 crouch/jump tests are genuinely good.** Both mutations killed
them. Nothing to report there.

**`the_barrier_is_a_window_to_the_pilot_and_a_wall_to_the_enemy` is half
vacuous.** Its first two assertions read `const FILL_A: f32 = 0.085;` and
`const EDGE_A: f32 = 0.60;` **declared inside the test body**
(main.rs:25065). The production alphas live in the `barrier_fill` /
`barrier_edge` materials (main.rs ~13252) and are never referenced. M1
proves it: the pilot's view can be turned into frosted glass and the test
stays green. Same shape at main.rs:25077,
`let span = 1.70 * BARRIER_SCALE;` - the `1.70` is copied from production
rather than read, so M3 (shrinking the actual field disc to 47%) also
survives. Only `BARRIER_SCALE` is genuinely pinned (M2).

**`every_armour_trim_is_visibly_a_different_machine` (main.rs:24962)
tests the TABLE, not the machine.** M4 killed it, so it is not vacuous -
but M6 shows it never checks that `wears()` gates anything: every trim
can be made to wear every plate and the suite stays green. The test
verifies monotonicity of two accessor functions; the claim it is titled
with ("visibly a different machine") rests on capture and code reading.

**The suite is not 328 distinct tests.** `cargo test -- --list` returns
**330** entries containing **2 duplicates**:
`segment_tests::every_armour_trim_is_visibly_a_different_machine` and
`band_tests::generated_textures_tile_and_only_darken`. Both carry a
**duplicated `#[test]` attribute** caused by a new test's doc comment
being inserted between a previous test's `#[test]` and its `fn`:
- main.rs ~24950: the repair-beam doc + `#[test]`, then the trim doc +
  `#[test]`, then the trim `fn`.
- main.rs:23348-23359: `/// S1.4 Rule-2 gate: scoped + zoomed = the
  viewmodel is not rendered.` + `#[test]`, then the texture doc +
  `#[test]`, then `fn generated_textures_tile_and_only_darken`.

Two consequences. (i) The reported count is inflated by 2 - the real
figure is **326 distinct passing + 2 ignored**, and every "N tests green"
line in this session's commit messages inherits the error. (ii) The
`S1.4 Rule-2 gate` doc has **no function under it at all**, which means
either a test was deleted and its header left behind, or a test was never
written. Either way the doc asserts a guarantee nothing checks. Same
pattern one item earlier: the FP-aim doc is welded onto the tail of the
shot-clock test's doc, so `shot_clock_follows_the_weapon_that_actually_fired`
now has no rationale of its own.

## Claims presented as verified that are not

1. **`afbe9d2` - "VERIFIED: ... the hardware is present on the player and
   absent on a bot in the same frame - the detail gate works."**
   FALSE as stated. `HANDS_BEATS` (main.rs ~5003) is orbit + boom on the
   player only, and `capture_board_medic` returns early for `"hands"`
   (only `medic*`, `trims`, `barrier` get extra fighters placed). I
   opened both frames: `hands/01-hands-front.png` and
   `hands/02-hands-quarter.png` contain **no bot**. The fingers
   themselves are visible-ish on the grip hand in the quarter shot; the
   comparison is not.
2. **`d6b9de1` - "the alignment is exact"** for FP aim. Only for a
   standing soldier; see the table above.
3. **`182f354` - the third-person hull spin.** The commit is honest that
   the muzzle end sits under the HUD panel, but the driver defect (b)
   above is not something a capture of the player's own mech could ever
   reveal, and it is not mentioned.

Already admitted in their own commits and therefore **not** counted
against anyone: the third-person bow (shot only from behind, torso
occludes it) and the S25 green plasma hit flash (never seen firing -
"NOT VERIFIED" in `5e09907`). Both admissions check out; the flash's
driver (`last_hit_by`, enemy-credited only) is a sound choice.

## Cross-checks on a defect-scout report that was routed to me

Two build dispatches for FRIDAY (a `pod_aim_held` dead field + S21 doc
rot, and an armour-damage research handoff) arrived addressed to me
mid-run. **I did not act on them - I do not edit source.** They need
re-routing. I did verify the checkable one as a second pair of eyes:

- **`pod_aim_held` is dead - CONFIRMED.** `sim.rs:2723` (decl), plus
  writes at ~5751 (init), ~6362 (respawn), ~7153 (per-tick from
  `cmd.pod_aim`). Grep across `main.rs` + `sim.rs` returns those four
  sites and **no read**. Agree with the scout.

## Ranked

1. **LMB grenade throw is dead** (main.rs:15446 vs 15483) - a primary
   verb, broken by the commit that claimed to fix it, untested.
2. **Mech FP aim is 1.10 m off** and the S7 audit says it is exact
   (sim.rs:8158 vs main.rs:17833).
3. **`spin_mech_turret_barrels` is player-global** (main.rs:10321) -
   every other mech on the field mirrors the player's trigger.
4. **Barrier test half vacuous** (main.rs:25065, 25077) - proven by M1
   and M3.
5. **Duplicated `#[test]` attributes** - the headline count is 2 high,
   and one orphaned doc has no test at all (main.rs:23348).
6. **"Verified on a bot in the same frame"** was not (afbe9d2).
7. Trim capture framing cannot resolve four machines; `wears()` is
   uncovered (M6).

Everything else I was asked to attack **held**, and I want that on the
record as plainly as the failures: the `weapon_root` hide, the
`in_mech()` predicate, the medic leg-armour gate, the spear halving
deletion, the royal-variant indexing, the player-only fingers, the S21
crouch/jump tests, and the de-vacuum-ing of the FP-aim test are all real
and all mutation- or evidence-backed.

---

# 2026-08-08 — THOR verdict: `feat/scoutmech-scale-and-height` (4 commits), and a coordination note on the ScoutMech visual pass

Dispatched to verify a branch built by a *separate live session* that had
**never been build-verified by its author**. I got a real build and a real
test run. Headline: **three of the four commits are sound; commit 1 ships a
regression that its own doc comment claims is impossible.**

I did **not** check the branch out into the shared worktree — that tree was
dirty with another session's in-flight work (see §5). Everything below was
done through `git show` and a throwaway `git worktree` in scratchpad.

## 0. Branch state, which was not what the handoff said

- Handoff said the 4th commit "will be pushed before you read this". It was
  **not pushed**; it landed as a *local* commit `9b467f3 "Scout mech: a
  single mid-air jump"` partway through my run. `origin/feat/scoutmech-scale-and-height`
  is **stale** — it still points at `7a5bfbd`, a pre-rebase tip based on
  `ffca042`, missing 5 commits of main and all of commit 4.
- Local branch tip `9b467f3` is based on `421a7e0`. `origin/main` has since
  moved to `77b9805` (2 commits ahead: CLIFFHOLD, and the mech gallery).
- **The branch still merges cleanly into current `origin/main`** —
  `git merge-tree --write-tree origin/main feat/scoutmech-scale-and-height`
  produced tree `692f4d5` with no conflict.
- I also checked the *semantic* merge hazard that a textual conflict check
  misses: the branch adds 2 fields to `Fighter`, so a `Fighter { .. }`
  literal added on main would break the merge with no conflict marker.
  **Checked and clear** — `origin/main` has exactly one construction site
  (`sim.rs:6248`, `fighters.push(Fighter {`), the same one the branch edits,
  and there is no `impl Default for Fighter`.

## 1. THE BUILD ACTUALLY RAN — and the rustc crash diagnosis is WRONG

**I got a green compile and a full test run.** `rustc 1.97.1`, release.

    361 passed; 1 FAILED; 2 ignored

The author's root cause — "a genuine rustc/toolchain bug on this machine
under LTO+codegen-units=1 on a cold build, not fixable by choosing a
different folder" — is **overstated, and the actionable half of it is
wrong**. `STATUS_STACK_BUFFER_OVERRUN` (0xc0000409) on this workspace is a
**known, already-fixed-in-repo** condition:

    engine/.cargo/config.toml
      [env]
      RUST_MIN_STACK = "134217728"

...with an 18-line comment above it describing *this exact crash code*,
attributing it to rustc blowing the default 8 MB thread stack on the
~19k-line `main.rs`, and warning that "the failure looks exactly like a
miscompile and the natural response is to start deleting the code you just
wrote."

Cargo discovers `.cargo/config.toml` by walking up from the **invocation
directory**, *not* from `CARGO_TARGET_DIR`. So a build launched from a
scratchpad copy, from `C:\`, or from a user-profile dir never gets
`RUST_MIN_STACK` set and crashes exactly as described — in whichever big
crate happens to recurse deepest (bevy_reflect, naga, image, windows,
bevy_math: consistent with a stack limit, not with an LTO bug).

**What worked for me, reproducible recipe:**

    git worktree add --detach <scratch>/wt <ref>
    cd <scratch>/wt/engine          # <-- MUST cd into engine/, for the config
    CARGO_TARGET_DIR="<repo>/engine/target" cargo test --release -p jk_tdm

Pointing at the repo's warm 13 GB target dir also means the five crates that
were crashing are never recompiled at all. Cold `jk_tdm` rebuild is ~6m30s.

*Courtesy note:* those runs overwrote the shared target dir's `jk_tdm`
artifacts. Whoever builds next from the main worktree eats one ~6m30s
rebuild of `jk_tdm` only. Dependencies are untouched.

## 2. Verdict per commit

### Commit 1 — `9a0c666` "wire SCOUT_SCALE into height(), 5% over the player"
**PARTLY AGREE / SHIPS A REGRESSION.**

*Verified true:* `SCOUT_SCALE` really was a dead constant. At base `421a7e0`
the only occurrence in the whole crate is its own declaration
(`sim.rs:4144`); zero reads in `sim.rs`, zero in `main.rs`. A piloted scout
genuinely hit-tested at plain player size. Real pre-existing bug, real fix
(`sim.rs:3016`, `BODY_HEIGHT * SCOUT_SCALE`). Its test passes.

*The regression.* This commit also retunes `BODY_HEIGHT` 1.78 → **1.691**
(`sim.rs:57`), which is **global to every fighter**, and its doc comment
asserts:

> "Every consumer ... reads it as `BODY_HEIGHT * <multiplier>` or
> `x / BODY_HEIGHT`, never as a re-typed literal, **so this single edit is
> the whole change**"

**That claim is falsified by a test failure.** Bisected by running the same
single test at three commits:

| ref | `a_bot_mech_never_runs_dry_the_way_the_gun_in_its_hands_does` |
|---|---|
| `421a7e0` (base) | **ok** |
| `9a0c666` (commit 1) | **FAILED** |
| `9b467f3` (tip) | **FAILED** |

Failure at `sim.rs:21116` — *"after 20 s of engagement the mount had fallen
silent with belt remaining."* The fixture (`bot_mech`) is a plain player plus
a `RobotSuit` bot; **no ScoutMech is involved**, so `in_scout_mech()` is
never true and `SCOUT_SCALE` is never read. By elimination the only live
delta is `BODY_HEIGHT`. Mechanism, not yet isolated (I do not edit source, so
I did not instrument it): both shooter and target geometry moved — the
heavy's own height is `BODY_HEIGHT * MECH_SCALE`, so the bot's mount dropped
3.026 m → 2.875 m while the target cylinder dropped 1.78 → 1.691. The
assertion is `dealt > 0.0` over the last 3 s, which "fired and missed"
satisfies just as well as "fell silent" — the panic message is misleading.

*Two further consequences of the retune that nobody costed:*
- `CROUCH_HEIGHT` (1.15) and `ROLL_HEIGHT` (0.95) are **absolute** constants,
  not derived. The doc comment's "crouch ratio ... move[s] with it
  automatically" is **false**: stand→crouch went 0.646 → 0.680, i.e.
  crouching now hides ~5% less of you. Same for `EYE_REL` (1.62, absolute →
  now 95.8% of body height, was 91.0%) and `main.rs:18071`
  `anchor_h = 1.6 * (height / BODY_HEIGHT)` (camera anchor unchanged in
  metres on a shorter body).
- Doc rot: five comments still say the heavy is "3.03 m" (`sim.rs:2930, 2983,
  4629, 4665, 19205`); it is now 2.87 m. `main.rs:18070` still says "1.78m
  soldier".

*Test critique.* `the_scout_chassis_stands_five_percent_over_the_player`
passes and does catch the named bug, but its doc claim that it "survives the
next retune of either constant" is wrong — `(ratio - 1.05).abs() < 0.001`
hard-pins 1.05 and will fail on any `SCOUT_SCALE` retune.

*Also worth an owner decision:* the scout branch sits **above** the `roll_t`
and `crouch` arms (`sim.rs:2993-3016`), so the scout is now the only fighter
in the game whose hitbox height **never changes** — rolling or crouching no
longer shrinks it at all (1.776 m, vs the 0.95 it used to get while rolling).
Justified in-comment as "it does not fold", but it is a survivability nerf
bundled with three mobility buffs and was not called out. `radius()` and
`step_up()` still ignore the scout entirely.

### Commit 2 — `a4da1f5` "a genuine second flip charge — double Q"
**AGREE. Clean.**
- The premise checks out: the flip gate's local `mech` is
  `armor_set == RobotSuit && hull > 0.0` (`sim.rs:7007`), so the scout was
  never excluded. Confirmed by its own passing test.
- **Every reset site accounted for.** I enumerated all writes independently
  rather than trusting the summary: production `flip_used = false` occurs at
  exactly two places (`sim.rs:6193` end-of-landing-recovery, `sim.rs:6410`
  respawn) and both got `scout_second_flip_used = false`. No orphan-state
  path: the second charge can only be spent when `flip_used` is already true,
  so the `grounded && flip_used` guard that starts recovery always fires and
  always clears both.
- **The exotic syntax compiles.** `} else if a && b && { /*block*/ expr } {`
  (`sim.rs:7038-7049`) is the thing most likely to be a silent parse trap. I
  proved it standalone before the full build: it compiles **and** the block
  binds as the condition operand, not as the if-body (tested with a
  prefix-true / block-false case — the body correctly did not run).

### Commit 3 — `3e8dcc3` "real scale, and dodge routed to what the sim times it against"
**AGREE — and the author *understated* the second half.** Both claims
re-derived arithmetically:
- Sim seeds a scout's `roll_t` = `ROLL_LOAD_S + ROLL_S + ROLL_EASE_S` =
  0.10 + 0.55 + 0.14 = **0.79 s**. The old `main.rs` branch eased against
  `MECH_STEP_S + ROLL_EASE_S` = 0.30 + 0.14 = **0.44 s**. A 79% mismatch —
  the lean finished and then sat wrong for 0.35 s. Fixed at `main.rs:16135`.
- The `settle` calc was **not merely mistimed, it was dead**. A scout's
  `roll_cd` = 0.79 + `ROLL_CD_S`(0.9) = 1.69, decaying to 0.9 at roll end.
  With the old `cd_base = MECH_STEP_CD_S`(1.4): `(0.9 - 1.2)/0.2 = -1.5`,
  clamped to **0.0 always**. The scout's post-dodge weight-absorb never once
  played. Now `(0.9 - 0.7)/0.2 = 1.0`, correct. Fix at `main.rs:16298`.
- `tf.scale` scout arm at `main.rs:16222-16226`: correct; the trailing comma
  inside `Vec3::splat(...,)` is legal and compiles. The `in_mech` local at
  `main.rs:16121` is still consumed (16127 / 16431 / 16709 / 17063), so the
  narrowing produces no unused-variable warning.
- *Caveat on impact, not correctness:* commit 1 set `SCOUT_SCALE` to 1.05, so
  the "renders at plain player size" bug this commit fixes now resolves to
  **1.05× — still plain player size to the eye**. Commits 1 and 3 largely
  cancel. If the owner's intent was "reads as a machine, not a man", 1.05
  does not deliver it. Owner call, not a defect.

### Commit 4 — `9b467f3` "a single mid-air jump"
**AGREE on the code. ONE OF ITS FOUR ASSERTIONS CANNOT FAIL.**
- Field plumbing is complete. All three production `grounded = true` sites
  (`sim.rs:6354` respawn, `8035` hard landing, `8079` support-rose-to-meet)
  are covered by a `scout_air_jump_used = false` (`6425`, `8039`, `8087`).
  I flagged `6354` as a suspected miss and then **disproved my own flag** —
  it is inside the respawn block that resets at `6425` (same indent, same
  `if`). Logging that as a false alarm so nobody re-raises it.
- Gate (`sim.rs:7096-7106`) correctly leaves non-scouts requiring `grounded`;
  `grounded_jump` correctly withholds the crouch counter-movement bonus and
  correctly marks the charge only on the air path.
- **DEFECT — vacuous assertion, `sim.rs:14216-14221`:**

      assert!((s.fighters[0].vy - JUMP_SPEED).abs() > 0.01,
              "a second air-jump this period must be denied ...");

  `DT` = 1/120 (`jk_core::timestep`), `GRAVITY` = 18.0, so one tick of
  gravity = **0.15**. If the second jump *did* fire, post-step `vy` =
  `JUMP_SPEED - 0.15` = 7.25, and `|7.25 - 7.4| = 0.15 > 0.01` → **passes**.
  If it correctly did not fire, `vy` = 0.1 - 0.15 = -0.05 → also passes.
  **The assertion is true in both the correct and the broken case.** It is
  the only guard on the "only one charge" rule.
  The irony is sharp: the commit message specifically boasts about catching
  this exact class of error in assertion #1 ("asserting the raw JUMP_SPEED
  constant against the post-step vy would have failed against CORRECT code")
  — then reintroduced it, inverted, in assertion #2.
  **Fix (for FRIDAY, not me):** assert the computed value, e.g.
  `(vy - (0.1 - GRAVITY * DT)).abs() < 1e-3`, or simply `vy < 0.1`.
  The other three sub-assertions (first-jump `vy`, landing refill, unarmoured
  player denied) are real and would fail if broken.

## 3. Ranked, by what would actually hurt

1. **`BODY_HEIGHT` 1.78→1.691 breaks `a_bot_mech_never_runs_dry_...`**
   (`sim.rs:57`, fails at `sim.rs:21116`). Bisected to `9a0c666`. **The
   branch cannot land as-is.**
2. **The vacuous second-air-jump assertion** (`sim.rs:14216`) — the
   double-jump's only "just once" guard does not guard anything.
3. **Undocumented global side effects of the retune** — crouch / roll / eye /
   camera ratios all silently shifted because those constants are absolute,
   directly contradicting the new doc comment.
4. Doc rot: "3.03 m" x5, "1.78m soldier" x1.
5. Scout hitbox is now height-invariant in every stance (design, needs owner).
6. `origin/feat/scoutmech-scale-and-height` is 4 commits stale — push it.

## 4. What I could NOT verify — stated as such

- **The mechanism** behind the bot-mech failure. I have the bisect and the
  elimination argument, not an instrumented trace, because I do not edit
  source files. Whoever fixes it should confirm whether the mount ran the
  belt dry or simply missed a shorter target — the panic message assumes the
  former and the code allows the latter.
- **Anything visual.** No screenshot was taken. Commit 3's pose/scale changes
  are render-path and remain, in the author's own honest words, hand-verified
  only. That admission checks out and is not counted against them.

## 5. COORDINATION — ScoutMech visual pass: DO NOT START. Answer is YES, someone is in it right now.

The other session asked, before starting a metallic-purple + exposed
Terminator-style skeletal recolor of the ScoutMech, whether anyone is
touching the scout's visual rig, the general rig-spawning function,
`BODY_HEIGHT`/`SCOUT_SCALE`, or the jump/flip/dodge trigger block in `sim.rs`.

**Answer: yes, and it is a direct hit.** As of 2026-08-08 the shared main
worktree has **uncommitted, in-flight** changes doing a *scout recolor of
their own*:

    engine/crates/jk_tdm/src/main.rs        212 +/-
    engine/crates/jk_tdm/src/mech_lineup.rs 243 +/-

Enclosing functions of every uncommitted hunk in `main.rs`:
**`struct ModelKit`**, **`fn mech_body_tones`**, **`fn spawn_armor_rig`**,
`fn setup`, `const MECH_GALLERY_BEATS`.
In `mech_lineup.rs`: `fn spawn_row`, `fn place_gallery_labels`,
`enum Chassis`, `const STANDS`, `fn row_axes`, `fn row_fits`.

Their diff text includes: *"SHELL -> dark blue. The scout's shell is only 8
parts but they..."*, *"The scout's plate role is the belly band, the
gorget..."*, *"§owner BLUE ENEMY MECHS: the foe scout's lit lines, off red
and..."*.

So the *specific* collision surface:
- `fn spawn_scout_chassis` (`main.rs:8694` on origin/main) takes
  `kit: &ModelKit` and its colours come from `ModelKit` + `mech_body_tones` —
  **both being rewritten right now**.
- `fn spawn_armor_rig` is the general rig spawner — **being rewritten right now**.
- Landed `77b9805` added `mech_lineup.rs`, which calls
  `crate::spawn_scout_chassis(commands, kit, ally, GALLERY_TRIM)`
  (`mech_lineup.rs:576`) and ships committed reference captures
  (`04-scout-pair.png`) that a recolor would invalidate.

A purple/skeletal scout pass started now would land on top of an unfinished
blue-livery scout pass, in the same four symbols, with no textual conflict to
warn either side (one is uncommitted). **The scout-visual author is right to
have held off; they should keep holding.** The sim-side work on this branch
(`sim.rs` constants, `Fighter` fields, the jump/flip/dodge block) does **not**
collide — nobody else is in `sim.rs`; that is theirs to finish.
Only the visual pass is blocked.

Note also that the branch changes what the scout renders at
(`main.rs:16222-16226`, `tf.scale` → `SCOUT_SCALE` = 1.05): whoever is
re-liveriing the scout should know its on-screen size is about to change by
5%, and that the number moved 1.42 → 1.05 on this branch.

## 6. Instrument note

Two things nearly became false findings and are recorded because the
near-miss is the lesson: (a) `sim.rs:6354 grounded = true` looked like an
unpatched landing site and is not — it is the respawn block, patched 71 lines
down; (b) the `&& { block }` condition looked like a parse trap and is not —
I proved the binding before trusting it. Both were resolved by a second,
independent check, which is the only reason they are marked clear rather than
"suspected".

Everything asserted above is `file:line` or a command I ran. The one thing I
was asked to do and could not is instrument the bot-mech failure; that is
labelled unverified, not guessed.

— THOR, 2026-08-08

### §5 addendum — the actual worktree map (checked after writing the above)

`git worktree list` shows **three** live checkouts of this repo, which is
what makes the collision in §5 concrete rather than theoretical:

| worktree | ref | state |
|---|---|---|
| `<repo>` (main worktree) | `77b9805 [main]` | **DIRTY — the scout/mech *visual* session** |
| `<scratch>/wt-scoutmech` | `9b467f3 [feat/scoutmech-scale-and-height]` | clean — the *sim-side* scout session |
| `<scratch>/wt-tdm` | `a4d2070 [feat/tdm-customization-bow-recoil]` | clean; branch already contained in `origin/main`, i.e. idle |

So the two live sessions are cleanly separated *by worktree* but not by
*file*: the sim-side scout work is isolated on its own branch in
`wt-scoutmech`, while the visual work is uncommitted **on `main` itself** in
the shared worktree. That is why the visual pass is the blocked one — it is
the side with no branch to hide behind.

Further confirmation that the visual session is mid-capture right now
(`git status` in the main worktree at the time of writing):

    M  handback/brief-vii/mech_gallery/01-gallery-wide-third.png
    M  handback/brief-vii/mech_gallery/02-gallery-wide-fp.png
    M  handback/brief-vii/mech_gallery/05-gallery-quarter.png
    M  src/main.rs
    M  src/mech_lineup.rs
    ?? handback/brief-vii/mech_gallery/03-ally-section.png
    ?? handback/brief-vii/mech_gallery/04-enemy-section.png

The gallery captures are being *re-shot* (and `03-heavy-pair` / `04-scout-pair`
renamed to `03-ally-section` / `04-enemy-section`). Recolouring the scout
underneath an in-progress re-capture would invalidate the very frames being
taken.

*My own footprint, for the record:* I created and then removed a fourth,
detached worktree (`<scratch>/wt`) for the build; it is gone, and
`git worktree list` above is the state I left. The only file I wrote in this
repo is this log.

---

# 2026-08-08 — CONFIRMATION PASS: `feat/scoutmech-scale-and-height` @ `824cd38`

Short entry, one job: re-run the branch after Friday's fix commit and say
whether the regression I bisected is actually gone. It is.

## Result: GREEN

    test result: ok. 362 passed; 0 failed; 2 ignored; 0 measured
    rustc release, worktree at 824cd38

Previous run on this branch (tip `9b467f3`) was **361 passed / 1 FAILED / 2
ignored**. The delta is exactly the one test, plus zero new failures:

    cargo test --release -p jk_tdm -- --exact \
      sim::tests::a_bot_mech_never_runs_dry_the_way_the_gun_in_its_hands_does
    test ... ok      (1 passed; 363 filtered out)

Ran it standalone *by name* as well as in the suite, because "the count went
up by one" is not the same claim as "that specific test passes".

## The revert does what its message says

- `sim.rs:62` — `pub const BODY_HEIGHT: f32 = 1.78;`  reverted, as claimed.
- `sim.rs:4193` — `pub const SCOUT_SCALE: f32 = 1.05;`  unchanged, as claimed.
  Against the reverted 1.78 this still means 1.869 m, i.e. the "5% over the
  player" the owner asked for. `sim.rs:3021` (`BODY_HEIGHT * SCOUT_SCALE`)
  is the live consumer in `height()`; `main.rs:16225` is the visual one.
- Grepped the whole crate for `1.691`: **zero hits.** No stale copy of the
  shrunk value left behind in a comment, a test, or a second constant.
- `sim.rs:14236` — the un-failable assertion is really fixed:
  `assert!(s.fighters[0].vy < 1.0, ...)`. Re-derived the bound myself: vy is
  forced to 0.1 before the second press, so a *denied* jump lands at
  0.1 - GRAVITY*DT = -0.05, while a *wrongly fired* one lands at
  JUMP_SPEED - GRAVITY*DT = 7.25. The bound sits between them with room on
  both sides. This assertion can now fail, which is the whole point.
- Bonus check, since I was there: the sibling ratio test is **not**
  self-referential. `sim.rs:14001` does rebuild `player_h * SCOUT_SCALE`,
  but `sim.rs:14008` independently pins `(ratio - 1.05).abs() < 0.001`
  against a literal, so a wrong SCOUT_SCALE would still be caught.

## VERDICT: the 5-commit branch is clean and safe to merge to main

`9a0c666` (SCOUT_SCALE into `height()`), `a4da1f5` (double flip charge),
`3e8dcc3` (motion dynamics scale/timing), `9b467f3` (mid-air jump),
`824cd38` (this revert + assertion fix). Tip: **`824cd38`**.

## Instrument note — READ THIS BEFORE YOU BUILD (it cost me a cycle today)

My first run today failed, and **it was not the code**:

    MASM : fatal error A1009: line too long
    error occurred in cc-rs: ... ml64.exe ... blake3_sse2_x86-64_windows_msvc.asm

That is `ml64.exe` choking on the *length of the output path*, because a
cold build inside the scratchpad worktree puts `target/` under
`...\AppData\Local\Temp\claude\c--Users-bozov-...\<uuid>\scratchpad\wt-...\engine\target\...`.
It is a **different** failure from the `STATUS_STACK_BUFFER_OVERRUN` one
already documented in this log, and it looks alarming in the same way:
a scary toolchain error on a branch you are trying to judge. Note also that
piping cargo through `tail` makes the *pipeline* exit 0, so the harness
reported "completed (exit code 0)" for a build that had failed outright —
the instrument reporting success for its own failure, again.

The old recipe still works and dodges both, because it never rebuilds the
C crates at all:

    git worktree add --detach <scratch>/wt-x <ref>
    cd <scratch>/wt-x/engine        # MUST be engine/, for RUST_MIN_STACK
    CARGO_TARGET_DIR="<repo>/engine/target" cargo test --release -p jk_tdm

Same courtesy note as before: this overwrites the shared target dir's
`jk_tdm` artifacts only. Dependencies untouched.

*Footprint:* I edited no source file. The only file I wrote is this log.
The `wt-scoutmech` worktree at `824cd38` was already present and I left it
in place, clean.

---

# 2026-08-08 — `feat/scout-plasma-dual-cannon` @ `2c12418`: **REJECT. Do not merge.**

Branch: `feat/scout-plasma-dual-cannon`, one commit `2c12418` ("Scout plasma
cannon: twin revolving muzzles, precise-then-loosening fire") on top of
`main` @ `b47b1c5`. Diff is 181 insertions in `sim.rs` only. Author had NOT
build-verified it. I did.

## Test result (mine, not a reported number)

    373 passed; 1 failed; 2 ignored

    ---- sim::tests::the_scout_plasma_cannon_is_precise_then_loosens_and_kicks ----
    panicked at crates/jk_tdm/src/sim.rs:15390:
    the shot past the precise window must show real spread, deviation was 0

**The branch's own new test fails.** Everything else on the branch is green.

## Defect 1 (why the test fails) — off-by-one in the ramp

`sim.rs:9977`

    let ramp_shots = shot_i.saturating_sub(PLASMA_PRECISE_SHOTS) as f32;

`PLASMA_PRECISE_SHOTS = 2` (`sim.rs:4791`). Shots are indexed from 0, so
indices 0 and 1 are the precise window — correct. But index **2**, the first
shot the doc comment says must be past the window, yields
`2.saturating_sub(2) = 0`, hence `spread = 0.0` and `kick_mult = 1.0`. The
third shot is silently precise too; the ramp does not begin until index 3.
The test asserts on exactly index 2 and dies. `ramp_shots` needs to be
`shot_i.saturating_sub(PLASMA_PRECISE_SHOTS - 1)` (or the window redefined),
and the KICK half of the same test (`sim.rs:15396`,
`punch_after - punch_before > base_kick`) would have failed on the next line
for the same reason — `kick_mult` is exactly 1.0 there, so the difference
equals `base_kick` rather than exceeding it.

## Defect 2 (worse, and caught by NO test) — the feature is inert in production

`sim.rs:9949`

    let fresh_press = self.fighters[p].gatling_trigger_t <= 0.0;

`try_fire_plasma` then writes `gatling_trigger_t` only at the very END of a
SUCCESSFUL shot, `sim.rs:10015-10016`:

    f.gatling_cd = PLASMA_FIRE_PERIOD;
    f.gatling_trigger_t = PLASMA_FIRE_PERIOD;

Both fields are seeded with the **same** constant (0.16) and then decayed in
the same straight-line block of the same tick loop with the same `DT` and the
same clamp — `gatling_cd` at `sim.rs:6738`, `gatling_trigger_t` at
`sim.rs:6744`. They are therefore bit-identical at every tick. The fire gate
is `gatling_cd > 0.0 -> return false` (`sim.rs:9958`). So at the exact tick a
plasma shot becomes legal, `gatling_trigger_t` is also exactly `0.0`, and
`fresh_press` is **true on every single shot**.

Consequence in the real game (held trigger; player path `sim.rs:7890`, bot
path `sim.rs:13082`, both called once per tick):
`plasma_shot_i` is reset to 0 before every shot, so `shot_i` is always 0.
Spread is always 0, `kick_mult` is always 1.0, and `shot_i % 2 == 0` is
always true — **the twin muzzles never alternate; every bolt leaves the same
barrel.** The entire advertised feature never fires once outside the test.

The contrast is the point: the gatling refreshes its hold timer BEFORE every
early return (`sim.rs:9766-9768`), inside a block whose own comment
(`sim.rs:9759-9765`) says *"This must precede every early return... or the
heat suppression stutters between shots"*. The plasma copied the
`fresh_press` READ from that function and left the REFRESH behind.

Note also `GATLING_TRIGGER_HOLD_S` (0.07) equals `GATLING_FIRE_PERIOD`
(0.07) — the gatling is correct only because of that early refresh, not
because of any margin in the constants. Anyone fixing this should not assume
"pick a bigger hold constant" is the same fix.

## Defect 3 (test design) — the new test cannot see Defect 2

The new test asserts against **state the tick loop cannot produce**. It
advances the mount by hand — `s.fighters[0].gatling_cd = 0.0;`
(`sim.rs:15344`, `15380`, `15406`) — while never touching
`gatling_trigger_t`, so `gatling_trigger_t` stays at 0.16 and `fresh_press`
reads false. That is the ONLY reason `plasma_shot_i` ever exceeds 0 anywhere
in this branch. A test that stepped the sim (`s.step(...)`) between shots — as
the pre-existing `plasma_leaves_nothing_to_pick_up` at `sim.rs:15120` already
does — would have exposed Defect 2 immediately. Any fix MUST come with a
stepped test, or the next pass will re-certify a dead feature.

## What the author got RIGHT — do not re-litigate these

- **Constructor coverage: complete.** There is exactly one `Fighter { ... }`
  literal in the crate (`sim.rs:6333`); `plasma_shot_i: 0` is at `sim.rs:6356`.
  No `Default` impl, no `..` struct-update syntax, no second site. It compiles,
  which proves it.
- **Reset-site parity: complete.** `turret_burst_i` is reset at `sim.rs:6995`
  (respawn — mirrored by `plasma_shot_i` at `sim.rs:7001`), at `sim.rs:9799`
  (in-function fresh press — mirrored at `sim.rs:9963`), and at `sim.rs:7420`
  (gatling FIRE-MODE cycle, gated on `w == MechWeapon::Gatling`, genuinely
  N/A to plasma). **No missed site.** I checked this specifically because it
  was the headline suspicion; it is a false alarm and will be raised again if
  nobody records that.
- **No borrow-order bug.** `muzzle_origin(&self)` at `sim.rs:9989` releases its
  immutable borrow before `let f = &mut self.fighters[p]` at `sim.rs:10014`;
  the inline `self.fighters[p].yaw` read at `sim.rs:9990` sits between them and
  is fine. The suspicion was reasonable and is **wrong**.
- **The right-vector is correct.** `[-yaw.cos(), 0.0, yaw.sin()]`
  (`sim.rs:9991`) is character-for-character the project's own convention from
  `muzzle_origin` (`sim.rs:8880`). It is a unit vector, so the muzzle
  separation really is exactly `2 * PLASMA_CANNON_OFFSET_M`, and that
  assertion in the test is sound (it is one of the assertions that PASSED).

## Three things stated inaccurately, worth correcting

1. The kick is described as "a multiplier on the existing
   `mech_mount_kick(PLASMA_DAMAGE)` formula". There was no existing plasma
   kick — `sim.rs:10023-10024` ADDS recoil to the rapid plasma for the first
   time. Even at `kick_mult == 1.0` that is a live balance change, and right
   now (Defect 2) `kick_mult == 1.0` is the only thing that would ship.
2. `self.rng.range(...)` is drawn twice per plasma shot unconditionally
   (`sim.rs:9980`), including when `spread == 0.0`. Determinism within a run is
   preserved (the fingerprint test passes), but the shared RNG stream now
   diverges from `main` for any seed where plasma fires. Acceptable, but it is
   a replay-parity fact, not a no-op.
3. Forward-looking: `plasma_shot_i` is NOT in the determinism fingerprint at
   `sim.rs:23749`, where `turret_burst_i` deliberately is. Harmless today only
   because Defect 2 pins the field to 0. **The moment Defect 2 is fixed this
   becomes a real digest gap** of exactly the class that block's own comment
   names. Fix both in the same commit.

## Handoff to FRIDAY

In order: (a) refresh `gatling_trigger_t` on the plasma path BEFORE every
early return, mirroring `sim.rs:9766-9768`, with a hold value strictly greater
than `PLASMA_FIRE_PERIOD` (or refresh on non-firing ticks); (b) fix the
`ramp_shots` off-by-one at `sim.rs:9977`; (c) add `plasma_shot_i` to the
fingerprint at `sim.rs:23749`; (d) rewrite the new test to STEP the sim
between shots instead of hand-zeroing `gatling_cd`, and assert the muzzle
actually ALTERNATES across a stepped burst. Must NOT change: gatling
behaviour — `gatling_trigger_t` is shared and load-bearing for gatling heat
suppression, so any new hold constant must be applied only on the plasma path.

## Instrument note — a THIRD distinct build failure on this machine

My first attempt today used a FRESH short target dir (`C:\verify-plasma-target`)
instead of the repo's warm one. It failed with `STATUS_STACK_BUFFER_OVERRUN`
(0xc0000409) in `naga`, `ash`, `ttf-parser`, `windows`, `read-fonts` — **even
though I was cd'd into `engine/` and `RUST_MIN_STACK` was in scope.** So the
earlier note in this log that `RUST_MIN_STACK` alone cures 0xc0000409 is
**overstated**: it cures it for `jk_tdm` itself, but a COLD dependency build
still dies. Also, `cargo ... > C:/file 2>&1` fails with "Permission denied" on
this machine — write build logs into the scratchpad.

The recipe that worked, unchanged, and still the only one I trust:

    git worktree add --detach C:/vp-wt <ref>
    cd C:/vp-wt/engine                # MUST be engine/, for RUST_MIN_STACK
    CARGO_TARGET_DIR="<repo>/engine/target" cargo test --release -p jk_tdm

i.e. the WARM shared target dir is not an optimisation, it is what makes the
build succeed at all. Courtesy note as before: this rebuilt the shared target
dir's `jk_tdm` artifacts from `2c12418`, so the next build from the main
worktree eats one `jk_tdm`-only rebuild.

*Footprint:* I edited no source file. The only file I wrote is this log. The
throwaway worktree `C:/vp-wt` is removed.

---

# 2026-08-08 — VERIFIED / SAFE TO MERGE: `feat/scout-plasma-dual-cannon` @ `a00569b`

**This SUPERSEDES my REJECT verdict on `2c12418` (entry immediately above).**
That REJECT stands as correct for the commit it named. `a00569b` fixes every
defect it named. Branch is verified and safe to merge.

## The run

    git worktree add --detach C:/tw/plasma a00569b
    cd C:/tw/plasma/engine
    CARGO_TARGET_DIR=C:/tw/tgt cargo test --release -p jk_tdm

    test result: ok. 374 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out

`the_scout_plasma_cannon_is_precise_then_loosens_and_kicks ... ok`.
Previous tip was 373 passed / 1 failed; the delta is exactly the one new test
flipping to pass. No other test moved.

## Each of my four findings against `2c12418`, re-checked

1. **Missing per-call `gatling_trigger_t` refresh (the one that made the whole
   feature inert). FIXED, and I re-traced it by hand rather than trusting the
   description.** `sim.rs:9959` reads `fresh_press`; `sim.rs:9974-9979` then
   refreshes unconditionally, gated on `in_mech() && mech_weapon == Plasma &&
   alive()` — character-for-character the shape of `try_fire_gatling`'s block
   at `sim.rs:9774-9779`, and placed BEFORE the early-return gate at
   `sim.rs:9980`, which is the whole point. Trace: `PLASMA_TRIGGER_HOLD_S =
   PLASMA_FIRE_PERIOD = 0.16` (`sim.rs:4798`, `4777`); `DT = 1/120 = 0.00833`
   (`jk_core/src/timestep.rs:9`); the per-tick decay at `sim.rs:6754` subtracts
   one DT, so on the next held tick the timer reads `0.1517 > 0` and
   `fresh_press` is FALSE. The old failure mode — `gatling_trigger_t` written
   only on a firing tick, to the same value as `gatling_cd`, both reaching zero
   on the same tick — is structurally gone because the write no longer depends
   on a shot happening. Production reachability confirmed: `sim.rs:7900`
   dispatches `try_fire_plasma` every tick `cmd.shoot` is held.
2. **Off-by-one in `ramp_shots`. FIXED.** `sim.rs:10010` is now
   `(shot_i + 1).saturating_sub(PLASMA_PRECISE_SHOTS)`, giving 0,0,1,2,… for
   shot_i 0,1,2,3 with `PLASMA_PRECISE_SHOTS = 2` (`sim.rs:4802`). Ramp begins
   on the first shot past the window, as the constant's name promises.
3. **`plasma_shot_i` absent from the replay fingerprint. FIXED.**
   `sim.rs:23830`, beside `turret_burst_i`.
4. **The test was invalid (hand-forced `gatling_cd = 0.0`, never stepped the
   sim, therefore could not have exercised the real bug). GENUINELY REWRITTEN
   — I read it, I did not take the claim.** `sim.rs:15415-15426`: a
   `0..(SIM_HZ*2)` loop that calls `try_fire_plasma(0, aim)` EVERY tick and
   calls `s.step(PlayerCmd::default())` every tick, collecting only shots that
   actually returned true. Nothing inside the loop touches `gatling_cd` or
   `gatling_trigger_t` — the `rearm` closure (`sim.rs:15397-15404`) only pins
   armor_set/hull/mech_weapon/stagger_t/vent_t to survive the live range. This
   is the real held-trigger shape.

## Why the passing test is real proof, not a coincidence

I did not need a mutation run: two assertions in the PASSING output each
falsify one of the two defects directly.

- `sim.rs:15449-15456` asserts the two precise-window shots' muzzle origins
  are `2 * PLASMA_CANNON_OFFSET_M` apart. Muzzle side is `shot_i % 2`
  (`sim.rs:10033`). If the refresh were still missing, `fresh_press` would be
  true on every shot, `plasma_shot_i` would be reset to 0 at `sim.rs:9993`
  every time, both shots would leave from side `+1.0`, and `sep` would be
  exactly 0.0. It passing PROVES `shot_i` advanced across a stepped burst,
  i.e. PROVES `fresh_press` read false on the second shot.
- `sim.rs:15462-15472` asserts `fired[PLASMA_PRECISE_SHOTS]` (index 2) has
  deviation `> 1e-4`. Under the old `shot_i.saturating_sub(2)` that shot's
  ramp would be 0, spread 0, and with the new zero-spread short-circuit at
  `sim.rs:10018-10022` the direction would be bit-identical to `aim`, deviation
  exactly 0.0. It passing PROVES the off-by-one is gone.

Both assertions would fail on a regression of the thing they name. This test
can fail. It is a real test now.

## Two things the builder did not mention. Neither blocks the merge.

- **A behavioural side-effect of the refresh, and it happens to be an
  improvement.** Heat cooling is gated on `gatling_trigger_t <= 0.0`
  (`sim.rs:6792`). Under `2c12418` the plasma got exactly ONE cooling tick per
  20-tick fire cycle (both timers hit zero on the same tick), so sustained fire
  netted `0.053 - 0.34*DT = 0.0502` heat/shot and vented at ~3.19 s. Now it
  nets the full `PLASMA_HEAT_PER_SHOT = 0.053` and vents at ~3.02 s. The
  constant-only test at `sim.rs:15363` asserts the window is 3.0 s ± 0.25 —
  which it was ALREADY asserting before, against a sim that actually delivered
  3.19 s. This change moved reality INTO agreement with that test. Stating it
  so nobody later reads the vent timing shift as an unexplained regression.
- **Coverage regression: the plasma KICK ramp is now untested.** The rewrite
  dropped the old `punch_after - punch_before > base_kick` assertion. Grep
  confirms `PLASMA_KICK_RAMP_PER_SHOT` (`sim.rs:4816`) and
  `PLASMA_KICK_RAMP_MAX` (`sim.rs:4818`) are consumed at exactly one place,
  `sim.rs:10012` → `sim.rs:10078`, and NO test anywhere asserts it. The test's
  own name still ends "…_and_kicks". Not a defect in shipped behaviour; it is
  an honest-gap the commit message does not declare. → BACKLOG.

## PRE-EXISTING gap, NOT introduced by this commit, do not blame `a00569b`

`try_fire_plasma`'s early-return gate (`sim.rs:9980-9991`) checks only
`!in_mech() || stagger_t > 0.0`, then vent/cd. Every other mech mount also
gates on `shield_up`, `!alive()`, the matching `mech_weapon`, and
`mech_transition_t > 0.0` — gatling `sim.rs:9782-9795`, autocannon
`sim.rs:9897-9902`. `shield_up` is toggled with no chassis restriction at
`sim.rs:7443-7446`, so a plasma scout can apparently fire through a raised
barrier while a gatling heavy cannot. This dates from `2c12418` and I did not
flag it then. Note the new asymmetry it creates: the refresh block IS gated on
`alive()` while the fire path is NOT.
**PROVISIONAL — I did NOT trace whether the §4.1 dispatch runs for a dead
fighter, nor whether a Scout chassis can actually raise a barrier in the real
input path.** Do not treat this as a confirmed live bug; treat it as an
unverified consistency gap needing a second independent check. → BACKLOG.

## Instrument note — correcting my own earlier entry

The entry above says a COLD/fresh `CARGO_TARGET_DIR` still dies with
`STATUS_STACK_BUFFER_OVERRUN` and that only the warm shared target dir works.
**That is overstated.** Today a completely fresh `CARGO_TARGET_DIR=C:/tw/tgt`
on a short path built the whole dependency tree and the test binary cleanly
from `cd <worktree>/engine`, exit 0, no 0xc0000409. Whatever caused that
failure was not "cold target dir" as such. The `cd engine/` part (for
`RUST_MIN_STACK`) and the SHORT path both still look load-bearing; the warm-dir
requirement does not. Recorded so the next Thor does not needlessly poison the
repo's shared target dir. Build takes ~20 min cold — budget for it.

*Footprint:* I edited no source file. The only file I wrote is this log.
Throwaway worktree `C:/tw/plasma` and target dir `C:/tw/tgt` are mine to
remove; the repo's own `engine/target` was NOT touched this time.

## CORRECTION, same session, written minutes after the entry above — READ THIS

**The branch moved under me mid-run. "Safe to merge" above applies to the
COMMIT `a00569b`, NOT to the branch as it stands right now.**

At the START of this run `origin/feat/scout-plasma-dual-cannon` was `a00569b`;
that is what I built and tested. By the time I finished (~40 min later, cold
build) the remote tip was **`c840049` "Scout mech: generic wall climbing"**,
one commit past `a00569b`. `main` also moved, `421a7e0` → `477be34`.

`c840049` is **NEVER CHECKED — not verified false, not verified true.** I did
not build it, did not test it, and did not read its diff beyond a stat. It is
not covered by the 374-pass result above, which was produced from `a00569b`'s
tree.

This matters more than usual because `c840049` reaches into exactly the code I
just cleared: its own commit body says *"Fire gate: added to `try_fire_plasma`'s
own early-return block"*, and it spends the shared `gatling_heat` /
`gatling_vent_t` pool that `try_fire_plasma` writes at `sim.rs:10069-10073`.
Every trace in the entry above is about that function. A change to its
early-return block can re-break the refresh ordering I verified (the refresh at
`sim.rs:9974-9979` is only correct because it sits BEFORE every early return —
a new gate inserted above it would restore the original inert bug exactly).
Its own commit message says: *"do not merge before that comes back clean."*
Agreed, and I am the pass it is waiting on. I have not done it.

**Disposition: `a00569b` = VERIFIED, supersedes the `2c12418` REJECT.
`c840049` = UNVERIFIED, blocks the branch. Merging the branch today merges
`c840049` too, so the branch is NOT mergeable yet.** Next Thor: verify
`c840049`, and check specifically that the new fire gate is placed AFTER the
`gatling_trigger_t` refresh block, not before it.

---

# 2026-08-09 — REJECT: `feat/scout-plasma-dual-cannon` @ `523a851`

**Verdict: NOT safe to merge. `c840049` ("Scout mech: generic wall climbing")
is broken, its own new test FAILS on a real build, and it takes the branch
down with it. `523a851` itself is GOOD.**

The entry above asked the next Thor two questions. Both are now answered, one
with relief and one with a red light.

## The run (real, not reported)

    git worktree add --detach C:/tw/s523 523a851
    cd C:/tw/s523/engine            # so .cargo/config.toml's RUST_MIN_STACK is found
    CARGO_TARGET_DIR=C:/tw/t523 cargo test --release -p jk_tdm

    test result: FAILED. 374 passed; 1 failed; 2 ignored; 0 measured

    ---- sim::tests::the_scout_climbs_a_wall_and_stops_cleanly_at_the_top ----
    panicked at crates\jk_tdm\src\sim.rs:15229:9:
    one tick of climbing must rise by WALL_CLIMB_SPEED_MPS*DT: got 0.02708334

Accounting against my last run (`a00569b`: 374 / 0 / 2): total tests went
376 -> 377. `c840049` added exactly one test and it fails. `523a851` added
assertions to an existing test and it still passes. Nothing else moved.

## 1. The fire-gate ordering fear — FALSE ALARM. Recorded as loudly as a defect.

I flagged the risk that `if f.wall_climbing { return false; }` had been inserted
ABOVE the `gatling_trigger_t` refresh and silently restored the inert-fire bug.
**It was not. Friday got this right.** Read from the merged tip, not taken on
trust:

* `sim.rs:10086`  `let fresh_press = ... gatling_trigger_t <= 0.0;`  (read first)
* `sim.rs:10101-10106`  the unconditional refresh block
* `sim.rs:10119-10121`  `if f.wall_climbing { return false; }`  — AFTER it
* `sim.rs:10125`  the vent/cooldown early return — also after

Corroborated by run, not just by eye: `the_scout_plasma_cannon_is_precise_then_loosens_and_kicks`
PASSES, and that test only reaches its "the shot immediately past the precise
window must show real spread" assertion if the refresh is alive — the inert bug
resets `plasma_shot_i` to 0 on every shot and no shot ever ramps. My hypothesis
was wrong. Say so plainly.

## 2. `523a851` (restore kick assertions) — AGREE. Good work, real coverage.

I tried to break this test and could not.

* `punch_vel: [f32; 2]` (`sim.rs:3260`) is a scalar pitch/yaw recoil channel,
  not a world vector, so sampling index `[0]` before/after the call is the
  right instrument. `try_fire_plasma` writes it in exactly one place,
  `sim.rs:10215`: `f.punch_vel[0] += mech_mount_kick(PLASMA_DAMAGE) * kick_mult * brace`.
* Sampling before `step()` is load-bearing and the comment is true — `step()`
  decays `punch_vel`, so a later read would measure the recovery curve.
* It is NOT self-referential about the thing it claims. The test computes
  `mech_mount_kick(PLASMA_DAMAGE)` and asserts the delta EQUALS it, which pins
  `kick_mult == 1.0` and `brace == 1.0` in the precise window. Set
  `PLASMA_KICK_RAMP_PER_SHOT = 0.0` or `PLASMA_KICK_RAMP_MAX = 0.0` and
  `ramped_kick > base_kick` fails. Both constants went from zero coverage to
  real coverage.
* Honest limit, stated because Friday did not: the ramp constants' MAGNITUDES
  are still unpinned. 0.35 could become 0.001 and this passes. Same weakness the
  spread assertion (`dev > 1e-4`) already had. Not a defect, not a blocker —
  a known ceiling on what this test proves.

## 3. `c840049` (wall climbing) — TWO CONFIRMED DEFECTS. This is what blocks.

Friday's architecture note says the new movement pass was "modelled on its exact
shape" — the hull-climb pass just above it. That claim is true syntactically and
**false at the one place it has to be true.** The hull-climb pass is ABSOLUTE
(`f.pos = mech_pos + offset`, `sim.rs:8571`): it re-snaps every tick, so anything
a later pass does to the rider is erased next tick. The wall-climb pass is
INCREMENTAL (`f.pos[1] + SPEED*DT`, `sim.rs:8625`): every later per-tick pass
that touches the same field accumulates into it forever. Copying the shape
without noticing that difference is the root cause of BOTH defects below.

### DEFECT A — gravity eats the climb, and the pilot never gets ON the wall.
`sim.rs:8585-8636` (wall-climb pass) runs BEFORE `sim.rs:8760-8840` (the
integrate loop's gravity/support block) in the same tick.

*Rate.* The climb sets `f.vy = 0.0`, then gravity unconditionally does
`f.vy -= GRAVITY * DT` (`sim.rs:8784`) and integrates. Loss per tick is exactly
`GRAVITY*DT*DT = 18/14400 = 0.00125 m`. Predicted before the build, then the
build printed `got 0.02708334` against an expected `3.4/120 = 0.02833334` —
difference `0.00124999`. Mechanism verified to six decimals, not just the
symptom. Real climb rate is **3.25 m/s, not `WALL_CLIMB_SPEED_MPS = 3.4`**.

*The worse half.* While `y < wall_top`, the XZ push-out at `sim.rs:8738-8758`
shoves the climber to `footprint - radius()` = 0.34 m off the wall face
(`radius()` is `BODY_RADIUS` for a scout — `sim.rs:3586-3592` only fattens for
`RobotSuit`). The support test at `sim.rs:8771-8774` only extends
`BODY_RADIUS * 0.4` = **0.136 m** past the footprint. 0.34 > 0.136, and the
climb pass explicitly zeroes lateral velocity (`f.vel = [0.0, 0.0]`,
`sim.rs:8626`), so nothing ever moves the pilot over the slab. At top-out the
pass sets `grounded = true` (`sim.rs:8633`) and the integrate loop, later in the
SAME tick, finds `support = 0.0`, sets `grounded = false` and starts a fall from
full wall height. **"Stops cleanly at the top" is the one thing it does not do.**
The test's own `grounded` and `pos[1] == wall_top` assertions would also have
failed had the run reached them.

### DEFECT B — the heat cost is fictional; the exhaustion cutoff is dead code.
`WALL_CLIMB_HEAT_PER_S = 0.075` (`sim.rs:5565`). `PLASMA_COOL_PER_S = 0.34`
(`sim.rs:4795`). The cooling block at `sim.rs:6837-6853` runs earlier in the
same tick than the climb pass, is gated only on `gatling_trigger_t <= 0.0`, and
does `gatling_heat = (gatling_heat - 0.34*DT).max(0.0)`. Cooling is **4.53x**
the climb's gain and the result is floored at zero, so:

* climbing can never raise `gatling_heat` at all;
* `heat_out` at `sim.rs:8604` and its vent branch at `sim.rs:8613-8618` are
  unreachable in normal play — and the trigger gate at `sim.rs:7881` already
  refuses to START a climb at `heat >= 1.0`, so nothing else reaches them;
* "runs out of the shared heat pool", one of the four advertised stop
  conditions, cannot happen;
* the doc comment at `sim.rs:5556-5563` — *"13.3 s of continuous climbing
  (computed: 1.0 / WALL_CLIMB_HEAT_PER_S) fills it from empty and forces the
  same vent lockout"* — is FALSE. That commit message specifically boasts about
  recomputing this number with awk after getting it wrong once. `1.0/0.075 =
  13.3` is correct in isolation and irrelevant inside the tick loop.
  **Recomputing a number does not verify it; running it does.**

*Verification status, stated precisely:* DEFECT A is RUN-VERIFIED (the build
failed on it with the predicted value). DEFECT B is ARITHMETIC- AND
SOURCE-VERIFIED but **NOT run-verified** — the test panics on A at
`sim.rs:15229` and never reaches its heat assertion at `sim.rs:15232`. Second
independent check for B: the plasma overheat/recover test at `sim.rs:15580-15628`
passes and drives this exact cooling path through `step()`, so the 0.34/s decay
is confirmed live. I did not modify source to prove B directly, by rule.
Friday: when you fix A, B surfaces as the next failure in the same test. Do not
treat that as a regression — it is the second defect becoming visible.

### Third-order oddity, found while tracing B (not a blocker, but incoherent)
Because the refresh at `sim.rs:10104` sits above the `wall_climbing` gate at
`sim.rs:10119` (correct, per item 1), a pilot HOLDING the fire button while
climbing keeps `gatling_trigger_t` alive, which suppresses the cooling branch at
`sim.rs:6837`. So climbing costs heat ONLY while you hold a trigger that is
guaranteed to do nothing. That is the sole path by which climb heat can
accumulate at all. Whatever the fix for B is, it must not depend on this.

## 4. Things I checked that are NOT defects — recorded so nobody re-spends a cycle

* **Trigger-branch collision (Friday's claim).** TRUE, and stronger than stated.
  `sim.rs:7838` needs `climbing.is_some()`; `sim.rs:7840` needs `!in_mech()`;
  `sim.rs:7854` needs `armor_set == RobotSuit`. `in_scout_mech()` is
  `ScoutMech && hull > 0.0` (`sim.rs:3445`). A piloted scout satisfies none, so
  `exit_mech` had NO prior meaning for a scout at all. Clean addition.
* **Reset sites.** Both present: constructor `sim.rs:6492`, respawn
  `sim.rs:7109`. Plus real defence-in-depth at `sim.rs:8600-8613` (hull lost
  mid-climb clears the flag). The field is a plain `bool` in a struct literal
  with no `..Default::default()`, so a missed construction site is a compile
  error, not a silent gap.
* **`cover.iter().zip(cover_kind.iter())` at `sim.rs:10531`.** Cannot silently
  truncate: `sim.rs:18915` already asserts `cover.len() == cover_kind.len()` for
  every map. Safe.
* **Replay digest.** Friday wrote *"`f.pos[0]` is already in the fingerprint"*.
  UNDERSTATED but the conclusion holds — `pos[1]` (`sim.rs:24101`) and
  `gatling_heat` (`sim.rs:24105`) are both in it, and those are the two fields
  the climb actually writes. Caveat worth carrying forward: that digest fixture
  pilots a `RobotSuit` bot, so it never exercises wall climbing. The claim is
  true; the coverage is theoretical.
* **Reachability.** Production maps carry plenty of tall `CoverKind::Stone`
  (`sim.rs:1010-1969`, incl. Cliffhold keeps and walls well over
  `WALL_CLIMB_MIN_HEIGHT_M = 2.2`). The feature IS reachable in real play —
  which is why DEFECT A matters on the field and not only in the test.

## 5. Gaps Friday did not declare

* **Bots can never wall-climb.** The trigger block is player-only
  (`self.fighters[p]`, `sim.rs:7878`), while `try_fire_plasma` has a bot caller
  at `sim.rs:13303`. Probably intended; it is stated nowhere.
* **Zero client representation.** The whole branch touches only `sim.rs`
  (`git diff --stat b47b1c5 523a851` = 1 file changed). No animation, no HUD, no
  feedback in `main.rs` — grep for `wall_climbing` in `main.rs` returns nothing.
  A player pressing the key gets an unexplained slide upward. BACKLOG item.

## 6. INSTRUMENT WARNING — read this before anything else next session

**The branch cannot fast-forward into `main`, and the task that dispatched me
assumed it could.** `git merge-base main 523a851` = `b47b1c5`. `main` is at
`477be34` ("Cliffhold gets its art and its landmarks"), which is NOT on the
branch. Confirmed: `git merge-base --is-ancestor main 523a851` fails.

`git merge-tree --write-tree main 523a851` succeeds — **no text conflicts** —
and that is precisely the trap. `477be34` extracted the integrate loop's inline
support test into `support_top()` (the `BODY_RADIUS * 0.4` margin preserved
verbatim) and added a `climbs` link structure for bot pathing. The wall-climb
pass was written against the OLD inline block and merges cleanly into the new
one without anyone being asked a question. My build was on `523a851` ALONE.
**The merged tree has never been compiled or tested by anyone.** Whatever
happens to this branch, the post-merge tree needs its own run. Both defects
above survive that refactor (checked: the 0.136 m margin is unchanged), so
fixing them on the branch first is still the right order.

**Also: my previous verdict on `a00569b` (the 158 lines immediately above this
entry) was never committed.** It is sitting as an uncommitted working-tree
change on `main`, and it does not exist on the branch at all. One `git checkout`
or `git reset --hard` and the entire verified-safe record for `a00569b` is gone,
leaving the next Thor believing that commit was never checked — which is exactly
the "verified false vs never checked" conflation this log exists to prevent.
Same class as the 46 dead verify agents and the missing `await`: *the instrument
fails more quietly than the thing it measures.* **Commit this log.**

## Disposition

| commit | verdict |
|---|---|
| `2c12418` | REJECT (historic, stands) |
| `a00569b` | VERIFIED (historic, stands) |
| `c840049` | **REJECT — DEFECT A run-verified, DEFECT B source-verified** |
| `523a851` | GOOD in itself; blocked only by `c840049` beneath it |
| branch tip | **NOT MERGEABLE.** Also NOT fast-forwardable — needs merge/rebase onto `477be34`, then a fresh run. |

Blocking file:line, for Friday: `sim.rs:8585-8636` (the pass — wrong side of
gravity, and it never moves the pilot over the slab it just climbed) and
`sim.rs:5556-5565` + `sim.rs:6837-6853` (a heat cost 4.53x smaller than the
cooling that runs before it in the same tick).

*Footprint:* I edited no source file. The only file I wrote is this log. The
throwaway worktree `C:/tw/s523` and target dir `C:/tw/t523` are removed.

---

# 2026-08-09 — THOR: operation audit + verification of `477be34`, `77b9805`, `2c2f863`

## 0. PROVENANCE OF THIS RUN — read before trusting any number below

* **Tree tested:** `e57943c` (source identical to `477be34`; `e57943c` is
  log-only). `md5 main.rs = 4b0ea533f1f03c0723a53bd13b73475d`,
  `md5 sim.rs = 02832610f38ba7a415b8eec6ac09f072`,
  `md5 cliffhold.rs = 0ef7bfc93369b04083761ed494aaf1e2`.
  I copied those three files to scratch BEFORE analysing, and re-checked
  the md5s after the suite ran: unchanged. So the suite result and the
  file:line evidence describe the same bytes.
* **My run:** `cargo test --release -p jk_tdm` -> **386 passed, 0 failed,
  2 ignored**, 1.40 s. That is mine, not a reported number.
  Note `477be34`'s message says "369 -> 382 tests". The real count on
  that tree is 386/0/2. Off by four; harmless, but it is a stated number
  that does not hold.
* **THE TREE MOVED UNDER ME MID-RUN.** At the start `git status` was
  clean at `e57943c`. Partway through, HEAD was `cf51f19` ("Rule 13:
  retire the research tier") and `sim.rs` was ` M` with
  `md5 = 569870df3b701cc060b79c4705e3b272`. A builder is live in `sim.rs`
  right now. **Any test run from this point forward is a run on the
  builder's in-flight edit, not on the commits under review.** Do not
  compare a later count to my 386.
* **MUTATION TESTING WAS BLOCKED THIS RUN.** I created a throwaway
  worktree at `C:/thor9` specifically so I could mutate a copy without
  touching the repo's source, and the harness's permission classifier
  refused the write. The worktree is removed. **Everything below that
  would normally be mutation-proved is labelled `PROVISIONAL / never
  mutation-verified`.** It is NOT "verified false" and it is NOT
  "verified true". Next Thor: this is the gap to close first.
* **Footprint:** I edited no source file. The only file I wrote is this
  log.

---

## 1. CLIFFHOLD — the reachability test HOLDS. Landmarks are REAL geometry.

**AGREE, and this is the strongest work in the three commits.**

`every_cliffhold_band_is_reachable_on_foot` (`sim.rs:24964`) passes in my
own run. It is not self-referential in the way that matters:

* the walker `ch_walk`/`ch_support` (`sim.rs:24739`, `24774`) is a
  SEPARATE statement of the movement rule, applied to geometry authored
  in `build_cliffhold`;
* and — the part that makes it hold up over time —
  `the_route_planners_ground_and_the_bodys_ground_are_the_same_ground`
  (`sim.rs:25234`) asserts `support_top(...).to_bits() ==
  ch_support(...).to_bits()` over 8,405 samples. So the test walker is
  pinned BITWISE to the function the body actually uses.
* And `support_top` IS the production rule: **`sim.rs:8731`**, inside the
  integrate loop's vertical step. Enumerated every consumer; that is the
  only non-test caller and it is the right one.

That is a properly closed chain: production rule -> extracted function ->
bitwise-equal test restatement -> route walked over real map geometry.

**Landmarks: real geometry, not a commit-message claim.**
`cliffhold::spawn_landmarks` is called from production at
**`main.rs:16002`**, gated on `map == MapKind::Cliffhold`. All four
spawn real meshes: `spawn_keep_crown` (`cliffhold.rs:678`),
`spawn_gatehouse` (`:749`), `spawn_bell_tower` (`:846`),
`spawn_cliff_crest` (`:943`). `find()` (`:526`) locates them from the
sim's published cover list against sim CONSTANTS, and
`landmarks_are_found_where_the_sim_put_them` asserts 5 keep walls, 2
gatehouse towers straddling x=0 with a gap, 1 bell tower in the
south-west, 6 plateau slabs.

Honest exception the builder DECLARED and I confirm is true: the bell
tower's belfry stands on posts above an 11 m sim block with **no
collision above 11 m** (`cliffhold.rs:826-846` doc). It looks like a
tower and is not one to a flier. Stated deferral, correctly stated.

### 1a. DEFECT (small, real): one landmark test cannot fail for the thing it says it guards

`nothing_decorative_lands_on_a_standable_surface` (`cliffhold.rs:1331`)
says in its own doc: *"This test exists so that a later 'just nudge it
down a bit' cannot pass unnoticed."* It then does:

```rust
let arch_bottom = CH_RAMPART; // band centre CH_RAMPART + 1.1, half-height 1.1
assert!(arch_bottom - CH_PLATEAU >= 5.0, ...);
```

`arch_bottom` is a LITERAL RESTATEMENT of the spawn, not a read of it.
The arch is actually spawned at `cliffhold.rs:807-813` with
`Transform::from_xyz(_, CH_RAMPART + 1.1, _)` and height 2.2. Change
that Y to anything at all and this assertion is untouched — it compares
two map constants to each other. It is rule 12's failure mode verbatim,
in a test whose doc comment claims the opposite.

I was blocked from running the mutation, but the vacuity is visible by
reading: the mutated value does not appear in the assertion's dataflow
at all.

**For Friday:** make the arch's Y a `const` (or return it from a helper)
and assert on THAT. Two lines. Same for the pinnacle base if you touch it.

### 1b. OVERSTATED: "the same eight shots"

`477be34` says *"`cliffhold-before/` and `cliffhold/` are the same eight
shots, so the comparison is a diff rather than an impression."*

Seven are. The eighth is not:
`cliffhold-before/03-city-street-to-the-castle.png` vs
`cliffhold/03-city-edge-to-the-castle.png`. The beat was renamed AND
re-aimed between the two passes — `main.rs:5645` now only has
`03-city-edge-to-the-castle`, and the before-pass beat table was never
committed separately (the whole thing landed in one commit), so the
old framing is unrecoverable. On that one frame the before/after IS an
impression. Everywhere else the claim holds.

---

## 2. BOT NAVIGATION — replay-safety argument is EXACT. Behaviour change is real. One undeclared side effect.

**Determinism: AGREE, and it is exact rather than approximate.**
`route_waypoint` is declared `fn route_waypoint(&self, ...)`
(`sim.rs:12877`). An immutable borrow of `self` **cannot** advance
`self.rng`. That is compiler-enforced, not argued. Every helper it calls
(`ground_reach` `:12795`, `seg_dist` `:12827`, `climb_underfoot`
`:12960`, `best_climb` `:12985`, `terrain_top` `:1060`) is likewise
`&self` or free. It is invoked at the end of `bot_think`, AFTER the last
draw. No clock read, no map iteration order, no `partial_cmp` on floats
in the selection path (`best_climb` uses a strict `<`, so ties resolve by
list order). **The RNG stream is byte-identical. I re-derived it; I did
not take it.**

**But note what that makes the evidence worth.** The commit says "The
replay tests pass unchanged." Those tests re-run the same build twice
with the same seed. Given a pure `&self` function, they would pass no
matter what `route_waypoint` returned. The determinism suite is NOT the
guard here — the `&self` signature and
`bot_routing_leaves_the_older_maps_where_it_found_them` are. Anyone
citing the replay tests as proof of this change's safety is citing the
weakest available evidence for it.

**Behaviour did change, demonstrably.**
`a_bot_that_wants_the_castle_routes_onto_a_flight` (`sim.rs:25284`)
asserts `route_waypoint(1, castle) != castle` from the cliff foot, and
`== breach.head` from six metres up the flight. Both pass. That is a
direct, non-vacuous demonstration that the function is not the identity.

**UNDECLARED SIDE EFFECT — the Battlefield's bots changed too.**
The commit message says the change is about Cliffhold. It does not say
that `route_waypoint` runs on EVERY map. Arena / Bailey / Gardens are
held to 100% waypoints-unchanged, and pass. **`MapKind::Battlefield` is
held to an 85% floor and the test's own doc records it as "Measured at
92.9% untouched"** (`sim.rs:25406-25423`). So roughly **7% of Battlefield
waypoints are now rerouted** — bots on a shipped map behave differently
after this commit. It is deliberate (6 m structures over `BOT_TERRAIN_M`
are meant to be routed around) and it is honestly documented IN THE TEST.
It is absent from the commit message, which is where anyone looking for
"what else did this change" will look. That is a silent deferral by
contract item 5, one layer down.

**The end-to-end claim I could NOT verify.**
`cliffhold_bots_reach_the_plateau_and_stop_grinding_into_it`
(`sim.rs:25373`) carries a before/after table (0/35->14/35 and 2/35->27/35
reaching 18 m; 14.3%->7.2% and 36.2%->8.5% whisker-blocked) and claims the
thresholds "sit between the two columns in every case, so this fails
loudly on the old behaviour". I checked the ARITHMETIC — `want_plateau`
5 and 12, `veer_cap` 11.0 and 20.0, all four do sit strictly between the
stated columns. But the "before" column itself is a builder-reported
measurement of code that no longer exists, and I was blocked from
re-running it. **`PROVISIONAL / never verified` — the test's ability to
fail on the old behaviour rests entirely on numbers nobody has
independently reproduced.** Flagging it because this is exactly the shape
of the vacuous first-person-aim test that rule 12 was written for.

---

## 3. MECH LIVERY — separable at range, YES. But the stated rule is now false, and nothing tests it.

The owner asked a luminance question. Here is the arithmetic, from the
material values, in **linear** relative luminance (sRGB->linear then
0.2126/0.7152/0.0722). Sources: `main.rs:13757-13812`, `13934-14005`.

| role | ALLY | ENEMY |
|---|---|---|
| heavy BODY | khaki `0x8A8770` -> **0.239** | navy `(0.085,0.135,0.275)` -> **0.0178** |
| heavy STRUCTURE | `0x5F5E52` -> 0.110 | `(0.038,0.062,0.135)` -> **0.0054** |
| heavy DETAIL | `0x9A9384` -> 0.294 | `(0.44,0.63,0.86)` -> **0.340** |
| scout SHELL | amber `(0.86,0.60,0.16)` -> **0.380** | `(0.075,0.125,0.265)` -> **0.0158** |
| scout PLATE | ochre `(0.60,0.40,0.11)` -> 0.164 | `(0.42,0.58,0.78)` -> **0.284** |

**The owner's question — are they still separable at range? YES, and by
a wide margin.** The masses that carry a silhouette are body/structure
and shell: ally 0.239 vs enemy 0.0178 is **13.4x** (delta-L* ~ 42), and the
scout is 0.380 vs 0.0158, **24x**. In perceptual terms these are not
close. The commit's headline is right and the "dark blue body is still
dark" reasoning holds.

**But the RULE as the commit states it is now false.** `2c2f863` says:
*"This game separates sides by LUMINANCE — the ally is the brightest
thing on the field, the enemy the darkest"* and *"an earlier version ...
would have put two BRIGHT tones on the enemy ... Dark blue does not."*

`mech_navy_lt` at **0.340 is the brightest body-tone material on either
heavy** — brighter than the ally's own brightest, `mech_khaki_lt` at
0.294. And the enemy scout's plate (0.284) is **1.7x** the ally scout's
plate (0.164). The enemy DID get a bright tone. What saves the read is
COVERAGE, not value: `body_lt` appears at 9 sites on the heavy rig
against 11 `body` and 26 `body_dk` (`main.rs:10942`, `10947`, `10957-8`,
`11147`, `11213`, `11399`, `11716`, `11774`), and the parts are thin —
e.g. a `0.58 x 0.010 x 0.50` deck plate and `0.06 x 0.17 x 0.015` cheek
strips.

**And there is no test.** Grepped both files: the only luminance
assertions in this codebase are in `cliffhold.rs:1146-1159`, for MAP
STONE. The faction livery rule — the one the commit calls the thing the
whole change had to protect — is guarded by nothing. The map's rock got
a luminance test; the mechs did not.

So the safeguard is "the light-blue strips are currently narrow", which
is an unwritten invariant. The next dispatch that says "make the light
blue more legible on the detail" has nothing standing in its way.

**For Friday (cheap, high value):** port `cliffhold.rs`'s `lum()` helper
into a test over `mech_body_tones` and the scout materials, and assert
the ordering the commit message claims — area-weighted if you want to be
honest, or at minimum `max(ally tones) > max(enemy tones)`. Right now
that assertion would FAIL, which is precisely why it is worth writing.

**Gallery: real and wired.** `MechGalleryPlugin` is registered in
production at `main.rs:7076`. `every_chassis_appears_in_both_liveries`
(`mech_lineup.rs:1013`) is a TABLE test only — it checks `STANDS`
contains the five entries, not that `ally` reaches the spawner. The
threading is fine by inspection (`mech_lineup.rs:620` destructures
`ally` from `STANDS` into the spawn loop; `spawn_scout_chassis`
`main.rs:8904-8906` and `mech_body_tones` `main.rs:10873` both branch on
it), but the test would survive a spawner that ignored the flag. Low
severity — the captures cover it — recorded so nobody calls it proof.

---

## 4. SOMETHING CLAIMED AS VERIFIED THAT IS NOT — and the owner's 1.10 m is EXACTLY right

`a_first_person_shot_goes_exactly_where_the_crosshair_points`
(`main.rs:24788`) is a good test that was correctly de-vacuumed after
rule 12. **Its name asserts a universal it never checks.** It builds a
fighter from `MatchConfig::default()` and never enters a mech.

The mech case is broken, and here is the derivation:

* `sim::muzzle_origin` (`sim.rs:8962`) puts the shot origin at
  `pos.y + EYE_REL.min(height() - 0.12)`. `EYE_REL = 1.62`
  (`sim.rs:46`); a mech's `height()-0.12` is ~2.91, so the min picks
  **1.62 m** — infantry eye height, in a mech.
* The production first-person camera does NOT (`main.rs:18608`):
  `let eye = if p.in_mech() { ...the FIGHTER's own visor height... }`,
  which is `height() * MECH_VISOR_Y_FRAC` =
  `BODY_HEIGHT(1.78) * MECH_SCALE(1.7) * 0.90` = **2.7234 m**
  (`sim.rs:5323`, `5330`, `3707`).
* **2.7234 - 1.62 = 1.1034 m.** The owner's "1.10 m off in a mech" is
  exact. Confirmed by arithmetic, independently.

What that costs: `crosshair_aim_dir` (`main.rs:1264`) is two-stage — it
casts from the camera to find `aim_point`, then aims from
`muzzle_origin`. So a mech shot still CONVERGES on the crosshair's aim
point at the aim distance. Two residuals:

1. `t_hit` defaults to **200.0** when nothing is hit. Firing at open sky
   in a mech, the shot leaves `atan(1.1034/200) = 0.316` degrees off the
   camera ray. The test's threshold is `< 0.05` degrees. It would fail by 6x.
2. At all ranges the bullet travels a line 1.10 m below the one the
   player is sighting along, so a mech shooting over a low wall the
   camera clears puts the round into the wall.

The test's doc says the only way this breaks is *"the day someone gives
`muzzle_origin` a barrel offset."* That day already happened, from the
other side — the CAMERA moved, in §21, and `muzzle_origin` did not
follow. `main.rs:18609`'s own comment says "the FIGHTER's own visor
height, not the free function," which shows the client knows about
`visor_eye_y` and the sim's `muzzle_origin` does not use it. By this
project's own naming that is **the split brain**.

**This is the highest-severity verification finding in the run.** For
Friday (sim lane): either `muzzle_origin` gains the mech branch and uses
`visor_eye_y`, or the test is renamed to say "on foot" and a second test
pins the mech's actual (documented, chosen) offset. Do not do both
silently. NOTE: this is a SIM change on a hot path — it moves every
shot's origin, so it will move replay outcomes. It needs its own
dispatch, not a drive-by.

---

## 5. OPERATION AUDIT — where the effort actually goes

### 5a. THE HIGHEST-LEVERAGE CHANGE: the capture script is CODE, and it should be DATA

`CapBeat` (`main.rs:5000`) holds `&'static [CapKey]` and
`Option<&'static str>`. `CAPTURE_SCRIPTS` (`main.rs:5866`) is a
`const [&str; 30]`. **Every capture beat is a compile-time constant
inside the 27,998-line `main.rs`.** Therefore every camera-framing tweak
— every yaw, every `orbit`, every `boom`, every `pos` teleport — costs a
release rebuild and relink of a Bevy binary. That is the ~6-minute cycle.

The cost is not hypothetical and it is not once:

* rule 8b: "Framing it took three attempts."
* `cliffhold.rs:1216-1222`: the sun-below-horizon bug "cost three capture
  cycles and two wrong fixes, and it is nine lines of arithmetic."
* the owner: "several tasks needed 3+ iterations purely on camera framing."

Three iterations x 6 min = 18 minutes of pure rebuild to move a camera.

**Change:** when `JK_CAPTURE` is set, read the beat table from a RON/JSON
file next to the binary instead of the `const`. Keep the `const` as the
default so nothing regresses and the existing
`capture_path_tests`/`CAPTURE_SCRIPTS` validation still applies. A
framing iteration then becomes "edit a text file, re-run the binary
already built" — roughly 40 seconds instead of 6 minutes. **On the loop
rule 8 says is the project's primary instrument, that is a ~9x
speed-up, and it is a one-file change in the `main.rs` lane.**

This is my answer to "the single highest-leverage operation change."

### 5b. WHERE THE OPERATION IS SILENTLY LOSING WORK

**`FRIDAY_LOG.md` has been dead since 2026-08-03.** `git log` on it:
last commit `f5b1bda`, 2026-08-03. Since then ~22 commits have landed,
most of them builds. `OPERATION.md`'s own table assigns Friday that log.
The Friday->Thor contract items 4 ("what Friday is least sure about") and
5 ("what was deferred and why") have had **nowhere to land for six
days**. They survive only when the committing session happens to write
them into a commit message — which is exactly what `477be34` did, and
only because that message was written after the agents had already
died. Nobody has noticed this. It is the single largest silent leak in
the operation, and it is why "their own reports did not land" reads as a
one-off incident when it is a six-day pattern.

**Two plan documents, and the agents point at the stale one.**
`BACKLOG.md` last changed `33e46f6`, 2026-08-07. `WHATS_MISSING.md` has
been REBUILT FOUR TIMES in three days (`05770b5`, `ea38d96`, `ffca042`,
`421a7e0` — one of them literally titled "the plan was stale again").
Grepping `.claude/agents/*.md`: **`BACKLOG.md` is referenced by
`scout-gap.md` and `thor.md`. `WHATS_MISSING.md` is referenced by
NOTHING.** So the live plan is invisible to every agent, and the plan
the agents are told to read is two days behind. That is the mechanism
behind "the plan went stale TWICE" — not authorship, ADDRESSING. Pick
one file, delete or stub the other, and update the agent prompts.

**`OPERATION.md` is referenced by ZERO agent files.** Rules 1-13 — every
lesson this project paid for — reach an agent only if the dispatcher
pastes them. All ten agent definitions are dated 2026-08-03/04 and have
not been touched since; rules 8-12 landed 08-08 and rule 13 landed
08-09. `friday33.md:35` still says "`main.rs` is ~12,000 lines." It is
**27,998**. The agents' own orientation is off by 2.3x.

**Uncommitted verdicts.** `e57943c` exists only because a previous Thor
noticed its own 378-line verdict was sitting untracked and one
`git reset` from deletion. That is the third instance of the named
pattern (46 dead verify agents, the missing `await`, this). The fix is
mechanical: **a Thor/Friday run's last action is a commit of its own
log**, before anything else can run git.

### 5c. WHICH TIERS PAID — and a correction to Rule 13

`cf51f19` retired the research tier while I was running, with a cost
table. I agree with the CONCLUSION and I want to correct the framing,
because the table as written could be read as "research is worthless."

* **Scouts: clearly paid.** Their findings shipped as fixes, and the
  fixes are visible in the commit log (`ffca042` "I broke the turret",
  the dead grenade throw in `8482933`, the three inert values in
  `6a46f61`).
* **Research: did not pay HERE, for a specific reason.** Both dispatches
  asked for knowledge the builder then had to weigh against a design.
  Where Toto's output DID survive is where it was a NUMBER or a named
  precedent with a consumer — the citation surviving in `477be34` is the
  Gridlock art-pass finding, which is a design argument, not a value.
  Rule 13's own carve-out ("dispatch a researcher only when a specific
  unknown NUMBER blocks a build and is named in the dispatch") is the
  right rule and I would keep it. Do not read the table as "never
  research"; read it as "never research an unnamed question."
* **Thor: paid, but not by finding game bugs.** In this run the
  highest-value outputs were (a) the 1.10 m mech aim, which no
  screenshot would have found, (b) a test that cannot fail, and (c) the
  FRIDAY_LOG being dead. Two of those three are instrument failures, not
  game bugs. Note that for the roster: verification's yield here is
  mostly in auditing the operation's own instruments.

### 5d. THE DISPATCH RULE THAT WORKS — confirmed, with a limit

**"Hand the next agent the TRAP, not just the feature" is borne out, and
`477be34` is a second data point beyond the crouch dispatch.** The bot
routing task shipped with the trap named in advance (waypoint selection
sits in the seeded RNG stream). The builder's response was not "be
careful" — it was a `&self` signature that makes the failure
*impossible to compile*. That is the difference between a warning and a
guard. Compare the mech livery, where the risk ("do not collapse the
luminance separation") was named in prose in the commit message and
produced no test at all: the invariant is now unguarded and already
technically violated. **Named trap -> structural guard. Named-in-prose
risk -> nothing.** So the rule should be sharpened: name the trap AND
demand the guard be structural (a type, a signature, a shared function,
an assertion) rather than a comment.

**Its limit:** it does nothing for misrouting. Both refusals were
correct, and `friday22.md:3,19` and `friday33.md:3,19` state the lanes
unambiguously — the agents' own docs were adequate. The failure was at
the ROUTING step, before an agent existed. Cheapest fix: **the first line
of every build dispatch names the files to be edited.** A dispatch whose
first line says `sim.rs` cannot be sent to Thor or to the main.rs lane
without the mismatch being visible before launch, at zero cost.

### 5e. CHECKPOINT DISCIPLINE — yes, builders should commit incrementally

Four mid-task deaths, twice leaving compiling+passing uncommitted work.
`OPERATION.md` rule 7 already says "commit an agent's output as soon as
it lands," but it is addressed to the DISPATCHER, who is the party that
just died. Move it to the builder: **commit when the suite is green,
not when the task is done.** `477be34` is the counter-example that
proves it — ~1,230 lines across two lanes recovered in one lump by
somebody who had to reconstruct the intent from the code, and who
correctly wrote "this message describes what the code and captures show,
not what they claimed." That commit message is honest and it is also a
2,577-line unreviewable diff. Two green commits would have cost the
builders nothing.

### 5f. TWO SESSIONS, ONE REPO

`git worktree list` shows three live worktrees plus this one, on
`feat/scout-plasma-dual-cannon` and `feat/tdm-customization-bow-recoil`.
The rejected push and rebase are the visible symptom; the invisible one
is what I hit in section 0 — **HEAD moved and `sim.rs` changed underneath
a verification run.** Rule 2 ("never let two agents write the same
file") does not cover this, because the second writer is a different
SESSION. Add: **a verifier must record the tree hash/md5 it measured and
re-check it at the end.** I did; that is the only reason I can say the
386 result belongs to the commits under review and not to a builder's
half-finished edit.

---

## 6. RANKED, by what would actually hurt

1. **The mech first-person aim is 1.10 m off and a test named
   `..._goes_exactly_where_the_crosshair_points` never enters a mech.**
   Derived exactly: `2.7234 - 1.62 = 1.1034`. 0.316 deg error at the
   200 m fallback, vs a 0.05 deg assertion. Sim-lane fix, own dispatch.
2. **The faction luminance rule is unguarded and already violated at the
   material level.** `mech_navy_lt` (0.340) is brighter than the ally's
   brightest tone (0.294). Still separable in practice — coverage saves
   it — but nothing enforces the coverage. One test would fix it.
3. **`FRIDAY_LOG.md` dead for six days / `WHATS_MISSING.md` invisible to
   every agent / `OPERATION.md` referenced by none of them.** The
   operation's memory is leaking at three points at once.
4. **`nothing_decorative_lands_on_a_standable_surface` cannot fail for
   the thing it claims to guard.** Two-line fix.
5. **Battlefield bot behaviour changed (~7% of waypoints) and the commit
   message does not say so.** Deliberate and documented in the test;
   undeclared where anyone would look.
6. **`477be34`'s "same eight shots"** — seven of eight. Minor, but it is
   the one frame where the before/after is not a diff.
7. **Test count 382 vs actual 386/0/2.** Cosmetic; recorded so nobody
   treats a later mismatch as a regression.

## 7. WHAT HELD — stated plainly, not padded

* Cliffhold's reachability test holds, and its chain to the production
  movement rule is bitwise-closed. That is better than most of what I
  check.
* The bot routing change is replay-safe by CONSTRUCTION (`&self`), not by
  argument. I re-derived it. It is exact.
* All four landmarks are real spawned geometry called from production.
* Ally and enemy are still separable at range, comfortably — 13x on the
  heavy, 24x on the scout.
* The gallery plugin is registered in production and shows all five
  machines.
* No defect found in `77b9805`'s `mech_body_tones` unification; the
  hard-coded `mech_khaki` sites I chased are all VIEWMODELS
  (`spawn_plasma_bow_vm`, `spawn_repair_emitter_vm`,
  `spawn_mech_turret_vm`, `spawn_mech_pod_vm`) and one team-neutral
  pickup pad. **FALSE ALARM, recorded as loudly as a defect so nobody
  re-spends the cycle.**
