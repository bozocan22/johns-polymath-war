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
