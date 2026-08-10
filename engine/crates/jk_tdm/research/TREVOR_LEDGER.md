# TREVOR LEDGER — the issued-vs-delivered data bank

**Rebuilt in full: 2026-08-10 (run 1, first ever).** This file is CURRENT
TRUTH, not history. History lives in `TREVOR_LOG.md`. Every `TRV-####` is
permanent: never renumbered, never reused, never deleted.

Repo HEAD when this run started: `e2866a9`. Two commits landed from a
concurrent session while I swept — `fe07c19` and `f10be3a` — and both are
folded in. `git fetch` before trusting a line number.

**What `DELIVERED` means here.** "There is evidence at this path." It does
NOT mean "it works." Thor decides that. Anything I doubt is tagged
`DELIVERED (contested)` and listed for Thor in §E.

**What `UNVERIFIED` means here.** I did not personally re-derive it this
run, or the check did not complete. It is an honourable answer and there
are 21 of them. A zero I did not verify is not a zero.

**Last checked.** Every row in this file was derived today, 2026-08-10.
The `Chk` column carries that date explicitly; where a row's status rests
on somebody else's evidence rather than mine, the Evidence cell says whose.

---

## HEADLINE

| | count |
|---|---|
| **Rows total** | **266** |
| `DELIVERED` (of which 4 contested) | **111** |
| `PARTIAL` | 43 |
| `NOT STARTED` | 61 |
| **Open (`PARTIAL` + `NOT STARTED`)** | **104** |
| `BLOCKED` | 17 |
| `SUPERSEDED` | 13 |
| `UNVERIFIED` | **21** |

**Origin: 231 `owner` · 29 `agent` · 6 agent-found-then-adopted-by-owner.**
Owner rows outrank agent rows at equal severity, always.

**Layer: 83 `sim` · 108 `cosmetic` · 34 `doc` · 19 `asset` · 22 both lanes.**
The 22 both-lane rows are the ones that need friday22 and friday33
coordinated, or a `match` breaks.

Counted mechanically from §B, not estimated. Every one of the 266 IDs
`TRV-0001`..`TRV-0266` is present exactly once, with no gaps.

---

## §A — THE THREADS

A row says a thing was asked. A thread says what happened to it. These are
built from what the material actually clusters into, not from a taxonomy I
brought with me.

### THREAD 01: the mech tier system (Agile / Big / Royal, both sides)
Rows: TRV-0001, 0003, 0009, 0013, 0015, 0017, 0058, 0059, 0093, 0099, 0101, 0191, 0192, 0248
First asked:  BRIEF_VIII §7.1 (scale 1.15×), 2026-07-xx
Restated:     BRIEF_VIII_B §A (decide: 1.15 / 1.7 / 2.5×), then
              WHATS_MISSING.md 0-SPEC15 **P1** "Six mech variants
              registered", 2026-08-09 (owner)
Research:     `research/mech-entry/CYCLE_1_REPORT.md`; TOTO_LOG 2026-08-03
              (body rig, which the chassis hangs on)
Built:        `sim.rs` `CHASSIS_TIERS` (:4924), `ROYAL_MULT` (:4838),
              `chassis_scale` (:4886), `in_heavy_mech()`;
              `mech_lineup.rs` STANDS (:236-241); commits `614bf03`,
              `4714ba0`, `1904bc7`, `a0f67ae`, `fe07c19`
Pictures:     `handback/brief-vii/mech_gallery/01..11.png` (11 captures —
              wide, ally section, enemy section, quarter, two enemy
              close-ups, four sentinels)
Verified:     THOR_LOG 2026-08-08 (SCOUT_SCALE bisect, `b47b1c5` revert)
State:        **PARTIAL** — three tiers, two sides, real pads, real hull
              pools, a team-blind test and a photographed lineup all
              shipped. What did not: the Royal is still the Big's 53
              plates times 1.10 (`main.rs:11414`), so "its own body and
              silhouette, NOT a scaled Big Mech" is unbuilt, and the
              opposition machines differ from the ally machines only in
              paint.
Next:         friday33 — give the Royal one silhouette element the Big
              does not have (SPEC15's own trap 3), then do the same for
              the opposition hull. Before that, TRV-0013 needs an owner
              ruling: the spec asked for subtle neon-blue accents and the
              build shipped gold.

### THREAD 02: the grenade as a held item
Rows: TRV-0006, 0007, 0063, 0121, 0122, 0123, 0124, 0125, 0126, 0127, 0128, 0129, 0130, 0131, 0148, 0175, 0184, 0185
First asked:  BRIEF_IX-B (fuse, falloff, bounce), 2026-07-xx
Restated:     WHATS_MISSING.md 0-SPEC15 **P2**, 2026-08-09 (owner) —
              "inventory → equip → HOLD IN HAND → aim → release →
              throwable physics → decrement stock"
Research:     `research/grenade/SOURCES.md` (S-09 de Carpentier, READ),
              `research/grenade/CYCLE_2_REPORT.md`
Built:        `held_grenade.rs` (whole module — four models, fist and
              forearm, `hold_pose`, `grenade_in_hand`); `sim.rs:8727`
              `f.grenades[sel] -= 1`; commits `4714ba0`, `70a4222`,
              `a0f67ae`
Pictures:     `handback/brief-vii/grenade_hold/01..07.png` (7 captures —
              rifle before G, frag at rest, winding, full wind, hand
              empty after throw, molotov bottle, third person)
Verified:     test `a_held_grenade_never_lights_its_fuse_and_the_clock_starts_at_release`
              (`sim.rs:20686`) asserts nothing detonates held, nothing is
              spent held, and release spends exactly one
State:        **DELIVERED** — the SPEC15 half is complete and
              photographed. What remains is BRIEF_IX-B's long tail:
              distinct percussion/gas types, water, enclosed-space
              amplification, height-on-airtime, kick-the-grenade
              counterplay. All of those were named-and-refused, not
              dropped, in `handback/brief-ix/REPORT.md`.
Next:         Nothing in this thread is P1/P2. Leave it.

### THREAD 03: the recoil envelope ("increase the recoil")
Rows: TRV-0008, 0064, 0193, 0195, 0201, 0238, 0239, 0240, 0241
First asked:  chat, quoted in FRIDAY_LOG as `§owner: "increase the recoil"`
              on the heavy mech's minigun turret
Restated:     WHATS_MISSING.md 0-SPEC15 **P2** — "controlled for 1-2 s,
              then progressively chaotic. Must read as intentional, not
              random"
Research:     NONE. Say so — this was measured in-engine, not researched.
Built:        `sim.rs` `TURRET_FELT_FLOOR` 24.0 (:5367), `mount_punched_aim`,
              `turret_chaos` + `turret_spray_entry` (:5528);
              `mech_recoil.rs` `SWING_RAD_PER_DEG` (:97-119);
              commits `d6e35d1`, `879c95a`, `fe07c19`
Pictures:     **NONE.** This is the single largest visible change in the
              game with no capture behind it.
Verified:     tests `sim.rs:22766` (one round moves the camera),
              `:22828` (and the half the bots pay), `:22943`, `:23083`,
              `:23168`, `:23216`
State:        **DELIVERED (contested)** — both halves shipped and are
              tested, and the client now reads the second axis the sim was
              already writing. Contested because Rule 8 says a visual
              claim with no screenshot is a hope: nobody has photographed
              the controlled window becoming chaotic.
Next:         A capture script beat: hold the turret trigger for the full
              burst and snap at 0.5 s / 1.5 s / 3 s / vent. That is the
              cheapest evidence in the whole ledger. Also three of
              Friday's own least-sure items (TRV-0239/0240/0242) are
              balance changes nobody asked for and want an owner playtest.

### THREAD 04: mech visual identity (the P3 block)
Rows: TRV-0011, 0012, 0013, 0014, 0015, 0016, 0111, 0112, 0113, 0114, 0115, 0172, 0194, 0198, 0205, 0206
First asked:  BRIEF_VIII_B §D (read from the concept art), 2026-07-xx
Restated:     WHATS_MISSING.md 0-SPEC15 **P3 — VISUAL**, 2026-08-09
              (owner). Five bullets, of which the Agile upgrade is
              explicitly "the largest visual item".
Research:     `handback/reference/NOTES.md` — and it opens by stating the
              image folder was NOT achievable that session
Built:        paint only. `main.rs:14288-14338` (four Royal materials),
              `:14361`/`:14527`/`:14577` (blue enemy), `branding.rs:96-152`
              (team identity). Commit `a0f67ae` split the two Royals.
Pictures:     `mech_gallery/03-ally-section.png`, `04-enemy-section.png`,
              `06-enemy-heavy-close.png`, `07-enemy-scout-close.png`
Verified:     THOR_LOG §3 — ally/enemy separable at 13.4× and 24× relative
              luminance. **Thor also states the rule is now false and
              nothing tests it** (TRV-0206).
State:        **PARTIAL, and it is the biggest open block the owner named.**
              Four of the five P3 bullets ask for GEOMETRY and received
              PAINT. The Agile upgrade has not been started under SPEC15
              at all (the 2026-08-07 medic redesign answered a different
              ask). The rocket launcher got a detail pass in `5e09907`
              and has not been revisited.
Next:         friday33, one bullet per session, in the owner's order:
              Agile → Rocket Launcher → Royal body → opposition body →
              opposition Royal. Hand each one SPEC15's own trap 3
              ("silhouette beats paint") and trap 4 (the luminance rule
              is unguarded).

### THREAD 05: mech boarding and the cockpit
Rows: TRV-0037, 0057, 0074, 0174, 0179
First asked:  BRIEF_VII §6.2 / BRIEF_VIII §7.6 (enter 1.6 s committed,
              exit 1.2 s, no teleporting pilots)
Restated:     `PROMPT_RND_CYCLE.md` Appendix A Critical #1 — "turns
              entering the mech from a state toggle into an *event*"
Research:     `research/mech-entry/CYCLE_1_REPORT.md` (aviation checklist
              design, why interruptible sequences fail)
Built:        `sim.rs` `mech_enter_stage` / `mech_enter_stage_for`, 8
              named stages, tested; `main.rs:19191` `mech_stage_presentation`;
              `cockpit.rs` (whole module); commits `b116a46`, `8482933`
Pictures:     `handback/brief-vii/cockpit/01..08.png`,
              `medic_cockpit/01..08.png` (16 captures)
Verified:     `visor_ready_tells_fully_entered_apart_from_never_boarded`
              (`main.rs:25838`)
State:        **PARTIAL** — and WHATS_MISSING is right about it. The
              client now *reads* the stage and fires one dry `click` per
              beat, but all eight visual beats are still `debug!` strings
              (`main.rs:19243-19254`). Seven of eight stages render
              nothing a player can see. `visor_ready` is still a field of
              a Bevy `Local`, so nothing outside its own system can read
              it — the 0-NOW §B.11 claim survives re-derivation.
Next:         friday33. The strings already exist verbatim inside the
              `debug!` calls; the work is turning each into a visible
              beat (plates, seam lights, servo audio) and the camera cut.

### THREAD 06: armour — pieces, weight, damage states
Rows: TRV-0036, 0106, 0133, 0134, 0135, 0137, 0140, 0183, 0186, 0235
First asked:  BRIEF_IX-C (26 pieces, weights, Fresh/Scuffed/Cracked/Severed)
Restated:     WHATS_MISSING 0-QUEUE Tier 2 #15 — "Armour damage states are
              invisible... `armor_stage_of` has ZERO client readers"
Research:     `research/armor-damage/SOURCES.md`, TOTO_LOG 2026-08-08 —
              the finding that reframed the dispatch was a NULL, and the
              degradation curve is concave not linear
Built:        `sim.rs` `ArmorPiece::struck`, `ArmorStage`, `armor_stage_of`,
              `armor_wear_of`, `wear_plate`, `plate_condition_survives_a_respawn`;
              `armor_weight_movement_penalty` WIRED; commits `192bd8d`,
              `d320af2`, `33e46f6`
Pictures:     `handback/brief-vii/trims/01-trims-front.png`, `02-trims-quarter.png`
Verified:     11 mutations, each killed a named test (FRIDAY_LOG §C)
State:        **PARTIAL, and the split is exact: the sim half shipped and
              the client half does not exist.** I grepped `main.rs` for
              `armor_stage_of`, `armor_wear_of` and `ArmorStage`: zero
              hits. Fresh, Scuffed and Cracked render identically. Only
              Severed shows, and only because it removes the piece.
Next:         friday33 — the single highest-value invisible-system fix in
              the game after boarding. Also carry Friday's own stated
              deferral (TRV-0235): plate wear fires only on the zoned
              hitscan path, so grenades, melee, claws and gas neither
              wear plate nor are reduced by it.

### THREAD 07: the Forge and character customisation
Rows: TRV-0028, 0045, 0052, 0076, 0095, 0136, 0138, 0139, 0181
First asked:  BRIEF_VII §7, restated in full as BRIEF_VIII §8
Restated:     BRIEF_IX-C Forge integration; `PROMPT_MASTER` Task 4 (L0-L4)
Research:     NONE for the flow. `PROMPT_MASTER` Task 4 demanded two
              peer-reviewed avatar papers; `research/character-creation/`
              does not exist.
Built:        turntable (`62cd0e5`), SAVE/LOAD/RANDOMIZE rows, the armour
              page, four classes with a Forge picker (`2af8a01`), helmet
              shapes (`5c1eb2d`)
Pictures:     `menus/02-soldier-page.png`, `03-armoury-page.png`
Verified:     `armor_weight` round-trip; class-trade test (`sim.rs:15058`)
State:        **PARTIAL** — real editor surface, real classes, real 24-plate
              grid. Absent: the per-piece category grid, the first-person
              preview toggle, cosmetic paint/decal layers, 5 loadouts per
              class, the class-swap warning, and the whole L0-L4 flow.
              `FORGE_SLOTS` is still declared and read by nothing.
Next:         Not P1-P3. Note for the record that BACKLOG.md #9's stated
              blocker ("no class system") is **known false** — the class
              system shipped 2026-08-05.

### THREAD 08: maps — the castle, the sightline rule, Cliffhold
Rows: TRV-0039, 0041, 0042, 0116, 0117, 0118, 0119, 0120, 0149, 0196, 0197, 0219, 0234
First asked:  BRIEF_IX-A (three tiers, 40 m rule, crossfire anchor)
Restated:     WHATS_MISSING 0-QUEUE Tier 3 #20 — "+10% larger, real
              elevation, high ground, randomised structures", with the
              seeding trap attached; and Tier 2 #18 "Delete Cliffhold"
Research:     `research/maps/SOURCES.md` (S-10 Level Design Book, READ),
              `research/vertical-maps/SOURCES.md` (TOTO33, tier-V finally
              non-zero)
Built:        `MAP_SCALE` +25% centrally (`46851ec`); Cliffhold
              (`830446e`, `477be34`) then its client half deleted
              (`4152240`); `max_unobstructed_sightline` validator
Pictures:     `map_lap/01..07.png`; the 16 Cliffhold PNGs were deleted
              with the client half
Verified:     all four maps measured against the 40 m rule —
              **none pass**, worst case 80-510 m (GAME_STATUS_REPORT)
State:        **NOT STARTED** for the castle the brief asks for, and
              **PARTIAL** for the Cliffhold deletion: 49 `Cliffhold`
              references survive in `sim.rs` including `build_cliffhold`
              and its five reachability tests, against 3 in `main.rs`.
              The two lying centrepieces (Bailey's keep, Gardens' gazebo)
              are `UNVERIFIED` — I did not re-derive them this run.
Next:         Tier 3 work; not P1-P3. If it is picked up, the seeding trap
              is the thing to hand the builder: randomised structures must
              be seeded at map-BUILD time from the match seed, or every
              later draw shifts and replay breaks for every other system.

### THREAD 09: traversal and bot navigation
Rows: TRV-0043, 0051, 0145, 0180, 0233
First asked:  `PROMPT_MASTER` Task 3 (dodge/jump/flips/mantle/vault/climb)
Restated:     WHATS_MISSING 0-QUEUE Tier 3 #21 — "Bot navigation,
              properly. Waypoints are 2D with no height and no
              reachability check. Bots cannot choose to climb."
Research:     `research/traversal/SOURCES.md`, `research/mech-climb/DESIGN.md`
Built:        hull climbing (`7e164d4`, 7-item checklist, 4 tests);
              §owner BOT ROUTING — published up-links, `BOT_PROBE_Y`,
              `route_waypoint` (`sim.rs:1009`, `:6776`, `:13608`,
              tests `:27338-27700`)
Pictures:     `traversal/01-jump-apex.png` .. `04-landing-recovery.png`
Verified:     `every_cliffhold_band_is_reachable_on_foot` — which is
              attached to the map whose client half was then deleted
State:        **PARTIAL** — climb/vault/mantle as a general verb was never
              built (`TASK0_AUDIT.md` row: "❌ none"). Bot routing landed
              for Cliffhold and `sim.rs:27649` says the flat maps were
              deliberately left where they were. Whether `waypoint` is
              still a bare `[f32; 2]` I did not re-derive: **UNVERIFIED**.
Next:         Blocked on map metrics (`MAP_METRICS.md` never written).
              Do not research it ahead of that — the ledger's own rule.

### THREAD 10: audio
Rows: TRV-0026, 0030, 0120, 0230(part), 0037(part)
First asked:  BRIEF_VII/VIII throughout ("servo/actuator audio", "blast
              audible at 60 m")
Restated:     WHATS_MISSING 0-QUEUE Tier 1 #9 — "**Every explosion is
              silent.** The sim publishes `Boom`; no sound exists."
Research:     NONE
Built:        20 `.wav` loads at `main.rs:14096-14116`. `gen_sfx.py` is in
              the repo and is the named unblocker.
Pictures:     n/a
Verified:     by me, this run: no explosion/boom sound exists in `Sfx`;
              `shot_handgun.wav` is generated at `gen_sfx.py:73` and
              loaded by nothing.
State:        **NOT STARTED.** Also open: plasma, repair beam, barrier and
              precision charge all play `shot_mp5` (`main.rs:21311`
              marks it a placeholder at the call site); every boarding
              stage plays `click`.
Next:         friday33 + `gen_sfx.py`. One session buys the explosion, the
              four placeholders and the eight boarding beats. There is no
              owner blocker on any of it and there never was.

### THREAD 11: HUD, menus and the things two screens disagree about
Rows: TRV-0024, 0031, 0032, 0033, 0055, 0061, 0087, 0088, 0089, 0090, 0210
First asked:  BRIEF_VIII §4 (four-corner anatomy)
Restated:     WHATS_MISSING 0-QUEUE Tier 1 #10/#11/#12
Research:     NONE
Built:        four corners, crosshair settings family (20 mutations),
              killfeed modifiers, scoreboard, death→killer-cam→spectate,
              suppression edge-glow, grip bar, mech vitals
Pictures:     `menus/01..08` (12 files across two naming generations)
Verified:     by me, this run: the Field Manual's MODES line prints the
              constant `TDM_TARGET` (`main.rs:24303`), not the target the
              player chose. `TDM_TARGET_CHOICES` (`sim.rs:430`) has no
              reader but a doc comment. `BIND_REGISTRY`'s `U` row
              (`main.rs:5077`) says "Dismount the mech" and never mentions
              grabbing the hull. The `Q` row (`:5060`) names roll and
              flip and never mentions the medic's second charge or its
              mid-air jump.
State:        **PARTIAL.** The anatomy is done; four small honesty defects
              survive, each one line.
Next:         friday33, all four in one sitting. `gatling_heat` carrying
              two scales in one field (`main.rs:21781` prints raw under a
              `%` while `:21763`/`:21769`/`:21772` print ×100) belongs in
              the same pass.

### THREAD 12: dead constants and doc rot
Rows: TRV-0022, 0023, 0024, 0025, 0026, 0027, 0028, 0029, 0249
First asked:  scout-defect sweep, 2026-08-08 (agent-origin)
Restated:     WHATS_MISSING 0-QUEUE **TIER 0 — MINUTES**, 2026-08-09
              (owner adopted them into the plan, so they are owner rows)
Research:     NONE
Built:        `ROBOT_SPEED_MULT` deleted (`sim.rs:437` records it);
              `pod_aim_held` deleted (`:3394`); `MECH_SHIELD_ARC_COS` now
              read at `:12841`
State:        **PARTIAL — 3 of 8 done.** Still dead, re-derived today:
              `TDM_TARGET_CHOICES`, `FORGE_SLOTS` (`main.rs:1372`),
              `shot_handgun.wav`. Two more (`SCOUT_SCALE`'s doc wording,
              the 9000.0 spray scale) are `UNVERIFIED` — I did not check.
Next:         The cheapest rows in the ledger. Bundle with THREAD 11.

### THREAD 13: the capture instrument
Rows: TRV-0040, 0056, 0154, 0204, 0243, 0251, 0266
First asked:  BRIEF_VII C1 "Visible or it didn't happen", 2026-07-xx
Restated:     owner, quoted in THOR_LOG: *"several tasks needed 3+
              iterations purely on camera framing"*; then WHATS_MISSING
              0-QUEUE Tier 2 #19 — "**Make capture scripts DATA, not
              code** — Thor's highest-leverage finding"
Research:     THOR_LOG operation audit (`df887dd`): the capture loop is
              the bottleneck; ~6 min per cycle, ~40 s if scripts were data
Built:        `CapBeat` (`main.rs:5203`) with `orbit`, `boom` and `home`;
              155 committed PNGs across 30 script directories
State:        **NOT STARTED.** Beats are still compile-time constants
              inside a 29k-line file, which is exactly why a framing tweak
              costs a full release rebuild.
Next:         friday33. This is the row that makes every other visual row
              cheaper, which is why Thor ranked it first.

### THREAD 14: the asset pipeline (textures, glTF, uploaded assets)
Rows: TRV-0048, 0049, 0224, 0262
First asked:  BRIEF_VIII §7.2 "Mandatory full PBR pass... untextured gray
              is a CI failure, not an opinion"
Restated:     WHATS_MISSING 0-QUEUE Tier 4 #26 — "**Your uploaded gun
              assets: `jk_tdm` HAS NO glTF LOADER.** Writing that loader
              is the actual task."
Research:     none needed; the blocker is named and mechanical
Built:        procedural textures + normal maps generated at startup
              (`main.rs:14160`, `:25101`; commits `07f923d`, `52684be`)
Verified:     by me, this run. `jk_tdm` makes exactly 24 `asset_server.load`
              calls: 20 `.wav` and **4 `.png`, all of them branding UI**
              (`branding.rs:303-306`). No image reaches a world material.
              No glTF/GLB loader exists. `engine/assets/characters/`
              contains one file: `.gitkeep`.
State:        **BLOCKED, and the blocker is NOT the owner.** This corrects
              two documents at once: `BACKLOG.md` #12 and
              `research/SOURCES.md` both say "all 21 `asset_server.load`
              calls are `.wav`" and "zero image loads". That has been
              false since `03085b1` (2026-08-03). The true remaining
              blocker is narrower and worth stating precisely: no imported
              image is used as a WORLD texture, and the shipping crate
              cannot load a mesh if pointed at one.
Next:         Write the glTF loader in `jk_tdm`, or accept that uploaded
              character/gun models cannot enter this game. That is an
              owner-facing choice, not a task.

### THREAD 15: the research programme (8 topics, the quotas)
Rows: TRV-0143 .. 0157, 0158 .. 0166, 0188, 0189, 0190, 0203
First asked:  `PROMPT_brief_X_research.md` (3 topics, 16 sources each)
Restated:     `PROMPT_MASTER_research_build.md` (8 topics, 12 each,
              ≥3 P and ≥3 V), which supersedes it
Superseded by: OPERATION.md **rule 13**, from the owner's own words —
              *"dont be doing reserch try to be more built and friday
              orienatted"* and *"cancel research but also you can tell
              them to become scouts to help you build"*, 2026-08-09
State:        **SUPERSEDED, deliberately, by the owner.** Recording it
              rather than deleting it, because the ledger's job is that
              nothing disappears quietly. Quota reality at retirement:
              `research/SOURCES.md` self-reports 2/16 counted on topic 1,
              2/16 on topic 3, **0/16 on topic 2**, and **0/4 video on
              every topic**. Eight tier-P seeds in the master prompt's
              §1.5 are still `SNIPPET-ONLY` and were never read.
Next:         Nothing. Do not restart it. `toto*` only when a specific
              unknown NUMBER blocks a build, named in the dispatch.

### THREAD 16: the motion architecture decision
Rows: TRV-0188, 0189, 0190
First asked:  `PROMPT_motion_system_research.md`, whole file
Research:     `research/motion-architecture/{SOURCES,NOTES}.md` — session
              1 read 5 of 14 core sources (`e69e454`)
Built:        NOTHING, correctly — the prompt's §4 says "Then stop. Do not
              implement."
Verified:     NEVER
State:        **PARTIAL / effectively abandoned.** `DECISION.md` — the
              entire deliverable — was never written. The one durable
              output is the LaFAN1 licence trap (CC BY-NC-ND 4.0),
              recorded and still true.
Next:         **This is the largest piece of orphaned research in the
              repo.** Either write the one-page decision from what was
              read, or close the thread explicitly. Leaving it at 5/14 is
              the worst of the three options.

### THREAD 17: the body rig and the motion doctrine
Rows: TRV-0079, 0080, 0081, 0082, 0102 .. 0110, 0044, 0046, 0169, 0170
First asked:  BRIEF_VIII_B §B (20 segments) and §C (elastic load model)
Restated:     `PROMPT_mech_rebuild.md` Tasks 2 and 3
Research:     `research/body-rig/SPEC_20_SEGMENT_RIG.md`; TOTO_LOG
              2026-08-03 twice (clavicle stays ASSUMED — terminal; toe
              split corroborated; one order-of-magnitude error in Toto's
              own ledger, caught by Toto)
Built:        `787f6ff`, `1e774e1`, `07e48ad`, `c6f55af`; all 20 segments,
              mass/length/inertia, five §B.6 tests including mass closure
Pictures:     `hands/01-02.png`, `idle_life/00s..16s.png`
Verified:     the separation test fails before and passes after — the
              proof the fix landed
State:        **DELIVERED**, with two payoffs still unclaimed: ragdoll and
              hit-reaction impulse read the inertia column nowhere
              (TRV-0046), and §4 soldier finger ANIMATION still poses from
              one spawn-time `curl` (TRV-0044, `UNVERIFIED` — not
              re-derived this run).
Next:         Tier 3. Not now.

### THREAD 18: first-person, aiming, and the 1.10 m question
Rows: TRV-0053, 0059, 0060, 0065, 0084, 0085, 0086, 0144, 0250
First asked:  BRIEF_VIII §3 (no ADS, scope hides the viewmodel, no bounce)
Restated:     WHATS_MISSING 0a Tier 1 #1 "First-person aiming [§7]. The one
              Priority-1 item still open"; then 0-QUEUE #29, reclassified
              by the plan itself as **"A DECISION FOR THE OWNER, not a
              task"**
Research:     `research/aiming/SOURCES.md` — the CHI 2014 aim-assist paper,
              READ in full, and the place this project caught a fabricated
              WebFetch extraction (five invented numbers and an invented
              technique name)
Built:        `d6e35d1`... no: `d6b9de1` — five hypotheses raised, all five
              cleared, and a test shipped pinning the property because it
              held by COINCIDENCE (muzzle == eye) rather than by
              construction
Verified:     THOR_LOG §4 — 2.7234 − 1.62 = **1.1034 m**, the owner's
              "1.10 m" confirmed exactly by independent arithmetic
State:        **BLOCKED on the owner.** The aim is geometrically exact for
              infantry. In a mech the camera sits at the visor and the
              weapon fires from `EYE_REL`, a metre below. A hull turret
              genuinely IS a metre below the visor, so this may be correct
              and merely unstated. Changing it moves every mech
              engagement, hit test, cover line and tracer in the game.
Next:         **Ask the owner.** Nothing else in this thread should move
              first. `AIM_SPEC.md`, the prompt's actual deliverable, was
              never written.

### THREAD 19: the weapons (spear, bow, the gun pass)
Rows: TRV-0071, 0072, 0091, 0097, 0199, 0213, 0214, 0215, 0216, 0217, 0218, 0225
First asked:  BRIEF_VII §3/§4, BRIEF_VIII §5
Restated:     a long run of chat asks preserved as `§owner` comments —
              "the war bow is held UPRIGHT, and it CURVES"; "a JAVELIN,
              not a broomstick with a wedge on it"; "the minigun had NO
              sights of any kind"; "IT HAS TO READ AS A MINIGUN"; "AIMED,
              THE WEAPON DOES NOT MOVE. Not 'moves less' — does [not]"
Research:     `research/spear-throw/SOURCES.md`
Built:        `8933748`, `419e52d`, `68d325b`, `a890766`, `c5a6d3b`,
              `53be066`, and the GUN PASS surface vocabulary
              (`main.rs:3906`, `:8415-8740`)
Pictures:     `spear_flight/00..05`, `arrow_flight/00..05`,
              `bow_draw/01..05`, `bow_draw_fp/01..05`,
              `sights_a|b|c/01..04` (12), `minigun_check/01..05`
Verified:     dead-code findings, not tuning: the spear's silent 2× speed
              halving fired on every player throw while the preview drew
              the full arc
State:        **DELIVERED.** The most completely evidenced thread in the
              ledger — 43 captures behind it.
Next:         Nothing open above Tier 3.

### THREAD 20: bot AI — squads, suppression, overwatch, retreat
Rows: TRV-0178, 0206, 0239, 0242, 0252
First asked:  `PROMPT_RND_CYCLE.md` Appendix A High #5
Restated:     BACKLOG.md #5
Research:     none new — `jk_wall`'s morale pass was the named precedent
Built:        focus fire + anchor/flank (`0b48693`), suppression by
              geometry (`df2a4b0`), bounding overwatch (`df2a4b0`),
              **RETREAT** with hysteretic fear (`e5431a4`; `sim.rs:2954`,
              `:13367`, `:13399`, test `:17320`)
State:        **DELIVERED.** `BACKLOG.md` #5 still says "RETREAT is the
              remainder" — **known false.** Every part shipped.
Next:         Nothing. But TRV-0242 (a bot chassis now never raises its
              barrier) is a live consequence of a recent fix and is owner-
              visible behaviour nobody chose.

### THREAD 21: networking and the things blocked on nothing existing
Rows: TRV-0050, 0125, 0182, 0185
State:        **BLOCKED**, all four, each with a named unblocker: zero
              networking deps; no water volume; no destruction design
              reason; no mud/sand/ice surface in any map.
Next:         Nothing. Do not research them "in preparation" — the
              research goes stale before the blocker clears.

### THREAD 22: powered armour (research only, by instruction)
Rows: TRV-0150
First asked:  `PROMPT_MASTER` Task 8, marked **RESEARCH ONLY, DO NOT
              BUILD**, with a build-readiness checklist as the deliverable
Research:     NONE. `research/powered-armour/` does not exist.
Built:        Nothing — which is correct.
State:        **NOT STARTED.** The instruction was obeyed in the half that
              said "do not build" and not in the half that said "produce
              the spec". Recording it so it does not vanish: the owner
              said they want this *in the future*.
Next:         Nothing now. Rule 13 retired the tier that would write it.

### THREAD 23: the in-game console and runtime image import
Rows: TRV-0147, 0163
First asked:  owner, paraphrased in `PROMPT_brief_X_research.md`'s own
              change table as *"a console that allows me inside the game
              or images I put"*
Restated:     `PROMPT_MASTER` Task 5C, with the refusal list as the feature
Research:     none. `CONSOLE_SPEC.md` never written.
Built:        Nothing. `TASK0_AUDIT.md`: "Console / command system — ❌ none".
State:        **BLOCKED upstream** on THREAD 14 — there is no world-texture
              path to import into. The console CORE (cvar registry,
              autocomplete, history) is not blocked by anything and could
              ship alone.
Next:         If the owner wants this, the honest first slice is the cvar
              registry with no import path at all.

### THREAD 24: the reference imagery the owner supplied
Rows: TRV-0256 .. 0266, 0070, 0075, 0083, 0100, 0115, 0168
First asked:  `PROMPT_mech_rebuild.md` Task 1 — "Save everything into
              `handback/reference/` and commit it — reference that lives
              only in a chat log is lost work."
Built:        Four owner PNGs are in the repo and wired: `key_art.png`,
              `wordmark.png` (+`_on_light`), `emblem.png` (+`_small`),
              plus `app_icon.ico`. Commit `923358d`, `03085b1`.
Missing:      **The mech concept art is not in this repository.**
              `BRIEF_VIII_B` §A and §D are written *from* it — "the art is
              the spec" — and §D.7 makes "place it next to the concept art
              in the handback" the section's stated completion criterion.
              The medic reference art ("a squat utility robot, rounded
              masses, one big camera lens, worn amber over near-black")
              is likewise absent; the medic was built to it and cannot be
              re-checked against it. `handback/reference/NOTES.md` opens
              by stating the image folder was not achievable that session.
              No image was ever committed and later deleted — I checked
              `git log --diff-filter=D` across all image types.
State:        **BLOCKED on the owner, and it is the oldest live ask in the
              ledger** — ~11 days, since `aefd16f` on 2026-07-31 put the
              briefs in the repo.
Next:         Ask the owner to drop the mech and medic reference images
              into `engine/crates/jk_tdm/handback/reference/`. Until then
              three completion criteria in two briefs are unsatisfiable,
              and every judgement about whether the machine matches the
              art is one person's memory.

---

## §B — THE ROWS

Columns: **ID · Ask (the owner's words, quoted) · Issued at · O**rigin
(o=owner, a=agent) **· L**ayer (S=sim, C=cosmetic, D=doc, A=asset) **·
Status · Evidence · Thread · Links · Chk**.

`Chk` is the date I personally re-derived the status. Every row in this
file reads `08-10` because this is run 1; nothing was carried on trust.

### B.1 — `WHATS_MISSING.md` §0-SPEC15 — THE MECH SYSTEM SPEC (owner, 2026-08-09)

The owner's own 15-section spec. It SUPERSEDES the queue. Priority order
is theirs, preserved exactly.

**P1 — ARCHITECTURE**

| ID | Ask | Issued at | O | L | Status | Evidence | Thread | Links | Chk |
|---|---|---|---|---|---|---|---|---|---|
| TRV-0001 | "Remove PYRO ARMOUR completely. Models, inventories, training, UI, menus, spawn logic, equipment logic, references, dead assets... Grep the whole tree." | WHATS_MISSING.md:18-22 | o | S+C | DELIVERED | `sim.rs:4775` (records the removal), zero `FLAME_*` constants tree-wide, `main.rs:5064` (bind row deleted), `:5103` (prompt deleted), `:12745` (pad model deleted); commits `b11b7de`, `5bf474a`. 8 residual mentions, all documentary comments. | 01 | supersedes TRV-0038 | 08-10 |
| TRV-0002 | "Training Mode: ONE fixed scenario. No settings menu, no rules config, no setup screen. Entering it starts the scenario. Hardcode the ruleset rather than exposing it." | WHATS_MISSING.md:23-25 | o | C | DELIVERED | `main.rs:23383` `§2 THE TRAINING SCENARIO, HARDCODED`, `:23405` `mode: Mode::Training`, `:23219` (training removed from the mode rows); test `main.rs:29015` compares a plain and a fiddled config | 01 | — | 08-10 |
| TRV-0003 | "Six mech variants registered: player Agile/Big/Royal and opposition Agile/Big/Royal." | WHATS_MISSING.md:26-27 | o | S | DELIVERED | `sim.rs:4924` `CHASSIS_TIERS`; test `sim.rs:16173` `six_chassis_variants_are_registered_and_team_blind`; `mech_lineup.rs:236-241` six stands; `handback/brief-vii/mech_gallery/01..11.png` | 01 | depends-on TRV-0009 | 08-10 |
| TRV-0004 | "Inventory/equipment stability preserved." | WHATS_MISSING.md:28 | o | S | UNVERIFIED | I did not check. No named regression test found for this phrasing; carrying no disposition. | 01 | — | 08-10 |

**P2 — GAMEPLAY**

| ID | Ask | Issued at | O | L | Status | Evidence | Thread | Links | Chk |
|---|---|---|---|---|---|---|---|---|---|
| TRV-0005 | "Shield in inventory as `Shield [4]`, an interactive slot showing quantity." | WHATS_MISSING.md:31-33 | o | C | DELIVERED (contested) | `main.rs:4909-4948` (the shield branch, `("SHIELD", "x1", p.shield_up)`), `:15517-15560` (four bordered tiles, sized to content); commits `70a4222`, `f10be3a`. Contested: **no capture frames the strip as four slots.** The quantity is `x1` by a documented decision (the plate is never consumed and has no pool). | 11 | — | 08-10 |
| TRV-0006 | "Grenade on G reuses the BOW/SPEAR architecture: inventory → equip → HOLD IN HAND → aim → release → throwable physics → decrement stock. A held item, not an instant projectile." | WHATS_MISSING.md:34-37 | o | S+C | DELIVERED | `held_grenade.rs` (whole module: 4 models + fist/forearm, `hold_pose`, `grenade_in_hand`); `sim.rs:8727` `f.grenades[sel] -= 1`; test `sim.rs:20686`; `handback/brief-vii/grenade_hold/01..07.png` | 02 | supersedes TRV-0063 partially | 08-10 |
| TRV-0007 | "Grenade-specific physics." | WHATS_MISSING.md:35 | o | S | DELIVERED | `sim.rs` `grenade_tick`, `surface_restitution`/`surface_friction`, tests `:25818`, `:25856`, `:25882` (bit-identical replay with thrown grenades) | 02 | — | 08-10 |
| TRV-0008 | "Big Mech recoil: controlled for 1-2 s, then progressively chaotic. Straight/heavy/absorbed first, then rising instability. Must read as intentional, not random." | WHATS_MISSING.md:38-40 | o | S+C | DELIVERED (contested) | `sim.rs:5528` §4 block, `turret_chaos(i)`, `turret_spray_entry(i)`, `mount_kick_axes`; tests `:22943`, `:23083`, `:23168`, `:23216`; client half `mech_recoil.rs:97-119` `SWING_RAD_PER_DEG` (`fe07c19`). **Evidence for the visual half is code, not a PNG.** | 03 | depends-on TRV-0193 | 08-10 |
| TRV-0009 | "Royal tier ~10% larger and ~10% stronger than Big." | WHATS_MISSING.md:41 | o | S | DELIVERED | `sim.rs:4838` `ROYAL_MULT = 1.10` feeding `chassis_scale`, `mech_hull_max`, `mech_shield_max`, a derived `armor_spec` row; tests `sim.rs:15812`, `:15881`, `mech_lineup.rs:1516` `the_royal_stands_taller_than_the_big` | 01 | — | 08-10 |
| TRV-0010 | "Royal ARROW LAUNCHER: minigun + 3 crossbows. Compact minigun silhouette, rotating mechanism, three crossbow assemblies around a central weapon, bolt ammunition, mechanical loading." | WHATS_MISSING.md:42-44 | o | C+S | **NOT STARTED** | Grepped the whole `src/` tree for `crossbow`, `arrow_launcher`, `ArrowLauncher`: **zero hits in any weapon path.** `MechWeapon` carries Gatling / Rockets / Autocannon / Plasma / Repair and nothing else. Every "bolt" hit is a rivet, a rifle bolt, or the medic's plasma bolt. | 01 | depends-on TRV-0009 | 08-10 |

**P3 — VISUAL**

| ID | Ask | Issued at | O | L | Status | Evidence | Thread | Links | Chk |
|---|---|---|---|---|---|---|---|---|---|
| TRV-0011 | "Agile Mech major upgrade — the largest visual item. Must read FASTER, LIGHTER, more advanced, more mechanically detailed, and clearly distinct from Big and Royal *in silhouette*. Plates, joints, legs, hydraulics, torso, shoulders, head/cockpit, weapon mounting, small components, surfaces, energy details." | WHATS_MISSING.md:47-51 | o | C | **NOT STARTED** | No `§P3` marker anywhere touches the scout chassis. The medic redesign (`72a93f3`, 2026-08-07) predates SPEC15 and answered a different ask — a squat utility robot, not a faster/lighter/more advanced one. `main.rs:9149` `§owner AGILE SUPPORT MECH: the light chassis, redesigned` is that earlier pass. | 04 | — | 08-10 |
| TRV-0012 | "Rocket Launcher redesign — chambers, mounting, reload mechanism, barrel detail, materials, VFX, firing animation, recoil." | WHATS_MISSING.md:52-53 | o | C | PARTIAL | Shipped: ribbed casing, mouth jaws, backblast vents, feed arm mid-travel, boxed seeker head (`aa14bcf` FLAGSHIP PASS, `main.rs:10988`), targeting electronics (`5e09907`). Absent: reload mechanism as a visible action, muzzle VFX, firing animation, recoil answer. Not revisited under SPEC15. | 04 | — | 08-10 |
| TRV-0013 | "Royal Mech: its own body and silhouette, NOT a scaled Big Mech, with SUBTLE neon-blue energy accents (channels, seams, reactor) — accent, not a coating." | WHATS_MISSING.md:54-56 | o | C | PARTIAL | Body: **not started** — `main.rs:11414` "§22 THE ROYAL VARIANT. Same machine, same 53 plates", geometry is the Big × `ROYAL_MULT`. Accents: **shipped but DEVIATED** — `main.rs:14288-14307` records the builder choosing "deep bronze-graphite carrying GOLD" over the spec's neon blue, on a stated argument. Owner has not ruled. | 04 | depends-on owner ruling | 08-10 |
| TRV-0014 | "Opposition mechs are NOT recolours. Own armour design, body structure, silhouette, mechanical detail, weapon styling — while keeping the faction colour language: neon red, dark red, neon blue, dark blue." | WHATS_MISSING.md:57-60 | o | C | PARTIAL | Colour language DELIVERED: `branding.rs:96-152`, `main.rs:14361`, `:14421`, `:14527`, `:14577`; captures `mech_gallery/04-enemy-section.png`, `06`, `07`. Structure: **the enemy machines are the ally machines with a different material table.** | 04 | — | 08-10 |
| TRV-0015 | "Opposition Royal: unique body, neon-red/dark-red primary with dark-blue complements. Must not read as a recoloured player Royal." | WHATS_MISSING.md:61-62 | o | C | PARTIAL | Palette split DELIVERED (`a0f67ae`; `main.rs:14326-14344` `mech_royal*` vs `mech_royal_ally*`), and `main.rs:11361-11372` records the defect it fixed — the two Royals differed "only by a lamp". Body: identical to the player Royal, which is identical to the Big. | 04 | duplicate-of TRV-0013 (body half) | 08-10 |

**P4 — POLISH, and the code-quality clauses**

| ID | Ask | Issued at | O | L | Status | Evidence | Thread | Links | Chk |
|---|---|---|---|---|---|---|---|---|---|
| TRV-0016 | "P4 — POLISH: Animations, VFX, weapon feedback, materials, lighting, UI consistency, performance." | WHATS_MISSING.md:64-66 | o | C | NOT STARTED | No pass named against P4. Ongoing polish is real but is not this row. | 04 | depends-on TRV-0011..0015 | 08-10 |
| TRV-0017 | "Shared systems over duplicated mech logic." | WHATS_MISSING.md:69 | o | S | DELIVERED | All 24 production `== ArmorSet::RobotSuit` comparisons replaced by `in_heavy_mech()` / `is_heavy_chassis()` in `614bf03`; recorded FRIDAY_LOG 2026-08-10 §2 | 01 | — | 08-10 |
| TRV-0018 | "faction visuals kept as DATA" | WHATS_MISSING.md:70 | o | C | DELIVERED | `branding.rs:150` `Side::steel()`, `:96-124` the four tone constants; `mech_body_tones` three-tone table | 04 | — | 08-10 |
| TRV-0019 | "training config isolated; no new settings" | WHATS_MISSING.md:70-71 | o | C | DELIVERED | `main.rs:23383` `match_config` for `Mode::Training` ignores the settings surface entirely; test `:29043` | 01 | duplicate-of TRV-0002 | 08-10 |
| TRV-0020 | "compile clean; fix warnings caused by the change" | WHATS_MISSING.md:71 | o | D | UNVERIFIED | I did not build. Trevor is read-only and did not run `cargo`. | 01 | — | 08-10 |
| TRV-0021 | "test every new interaction" | WHATS_MISSING.md:71 | o | S | UNVERIFIED | I did not run the suite. Last recorded count: 417 (FRIDAY_LOG 2026-08-10), plus whatever `fe07c19`/`f10be3a` added. | 01 | — | 08-10 |

### B.2 — `WHATS_MISSING.md` §0-QUEUE — the owner's size-ordered list (2026-08-09)

**TIER 0 — MINUTES**

| ID | Ask | Issued at | O | L | Status | Evidence | Thread | Links | Chk |
|---|---|---|---|---|---|---|---|---|---|
| TRV-0022 | "`ROBOT_SPEED_MULT` (1.12) is dead and states the OPPOSITE of the live 0.85 beside it. Delete it." | WHATS_MISSING.md:100-101 | o | S | DELIVERED | `sim.rs:437` — the constant is gone; a `§owner (defect pass)` comment stands where it was, recording the deletion and why | 12 | — | 08-10 |
| TRV-0023 | "`MECH_SHIELD_ARC_COS` is named, documented, and unread — the barrier's real arc test uses a bare `cos > 0.5` literal 40 lines away. Point one at the other." | WHATS_MISSING.md:102-104 | o | S | DELIVERED | `sim.rs:12841` `barrier_arc = cos > MECH_SHIELD_ARC_COS;`, with `:12828` recording that the literal had exactly one site | 12 | — | 08-10 |
| TRV-0024 | "`TDM_TARGET_CHOICES` is unread; the menu hand-types `30`/`60`." | WHATS_MISSING.md:105 | o | C | **NOT STARTED** | Re-derived today: `sim.rs:430` declares it; the only other occurrence in the crate is a doc comment at `sim.rs:4922` naming it as a known mistake. `main.rs` reads `sim::TDM_TARGET` (the scalar) at `:1477`, `:23412`, `:24303` and never the array. | 12 | — | 08-10 |
| TRV-0025 | "`pod_aim_held` is written every tick and read nowhere. Delete or wire." | WHATS_MISSING.md:106 | o | S | DELIVERED | `sim.rs:3394` — deleted; the `§owner (defect pass)` comment records it | 12 | — | 08-10 |
| TRV-0026 | "`shot_handgun.wav` is generated and loaded by nothing." | WHATS_MISSING.md:107 | o | D | **NOT STARTED** | `engine/assets/audio/gen_sfx.py:73` writes it; the file is on disk and tracked; `main.rs:14096-14116` loads 20 wavs and this is not one of them | 12, 10 | — | 08-10 |
| TRV-0027 | "`SCOUT_SCALE`'s doc says 'slimmer and SHORTER' for a chassis that is 5% TALLER than a man." | WHATS_MISSING.md:108-109 | o | S | UNVERIFIED | I confirmed the constant is `1.05` (`sim.rs:5045`) and is now consumed by `chassis_scale` (`:4893`). I did **not** read the full doc comment to check the wording, so I am not calling this either way. | 12 | — | 08-10 |
| TRV-0028 | "`FORGE_SLOTS` is unread; both slot UIs hardcode the list." | WHATS_MISSING.md:110 | o | C | **NOT STARTED** | `main.rs:1372` `const FORGE_SLOTS: usize = 3;` — the only other `forge_slot` hits are `forge_slot_path()` (`:1456`) and its callers, none of which read the constant | 12, 07 | — | 08-10 |
| TRV-0029 | "Export a `pub const` for the `9000.0` spray scale — the client copies it today and guards the copy with a test." | WHATS_MISSING.md:111-112 | o | S | UNVERIFIED | Not checked this run. | 12 | — | 08-10 |

**TIER 1 — AN HOUR**

| ID | Ask | Issued at | O | L | Status | Evidence | Thread | Links | Chk |
|---|---|---|---|---|---|---|---|---|---|
| TRV-0030 | "**Every explosion is silent.** The sim publishes `Boom`; no sound exists. Unblocker is `gen_sfx.py`, already in the repo." | WHATS_MISSING.md:115-116 | o | C | **NOT STARTED** | Re-derived today: `main.rs:14096-14116` is the complete `Sfx` load list — 20 wavs, no explosion, no boom. `gen_sfx.py` generates no explosion either. | 10 | — | 08-10 |
| TRV-0031 | "**The Field Manual quotes the DEFAULT score target, not the one you chose** — two screens on one pause menu disagree." | WHATS_MISSING.md:117-119 | o | C | **NOT STARTED** | Re-derived today: `main.rs:24300-24307`, the `modes` string interpolates the constant `TDM_TARGET`, not `sim.tdm_target`. Everything else on that screen was correctly moved to live constants — this line was missed. | 11 | depends-on TRV-0024 | 08-10 |
| TRV-0032 | "**Hull-climbing `U` is missing from the 'full bind list.'**" | WHATS_MISSING.md:119 | o | C | **NOT STARTED** | Re-derived today: `main.rs:5077` — the registry's only `U` row is "Dismount the mech (chassis is scrapped; the pad respawns)". The in-world prompt says "U - GRAB THE HULL" (`:22802`) and no bind row mentions it. | 11 | — | 08-10 |
| TRV-0033 | "**The medic's mid-air jump and second flip are undocumented** — two movement options a pilot will never discover. Put them on the Controls screen, NOT the equip hint (that line already overflows)." | WHATS_MISSING.md:120-122 | o | C | **NOT STARTED** | Re-derived today: `main.rs:5060` `Q` = "Ground: dodge roll - Air + direction: FLIP". No mention of the scout's second flip charge (`sim.rs:3427`) or its single mid-air jump (`sim.rs:3436`). The equip hint at `:5128` was correctly cut to two facts and does not carry them either. | 11 | — | 08-10 |
| TRV-0034 | "`gait_pose` bakes the INFANTRY crouch ratio (0.646) against the sim's `MECH_CROUCH_HEIGHT_FRAC` (0.72) — kneeling, the x2.0 visor weak point lands on rendered neck and shoulder." | WHATS_MISSING.md:123-125 | o | C | UNVERIFIED | Partial derivation only: `MECH_CROUCH_HEIGHT_FRAC = 0.72` at `sim.rs:5919`, and it appears **nowhere** in `main.rs`. The literal `0.646` also appears nowhere in `main.rs`. I could not locate the current site, so I will not call it either done or open. | 01 | — | 08-10 |
| TRV-0035 | "The barrier test is half vacuous: its alpha and span constants are copied INTO the test body, so mutating the real material or the real disc size leaves the suite green." | WHATS_MISSING.md:126-128 | o | C | UNVERIFIED | Not re-derived. Rule 12 says a test cited as evidence that cannot fail is not evidence — so this row also caveats TRV-0207's evidence. | 04 | — | 08-10 |

**TIER 2 — A SESSION**

| ID | Ask | Issued at | O | L | Status | Evidence | Thread | Links | Chk |
|---|---|---|---|---|---|---|---|---|---|
| TRV-0036 | "**Armour damage states are invisible.** Four stages ship and are tested; `armor_stage_of` has ZERO client readers, so Fresh, Scuffed and Cracked render identically. Only Severed shows, and only because it removes the piece." | WHATS_MISSING.md:131-134 | o | C | **NOT STARTED** | Re-derived today: grep of `main.rs` for `armor_stage_of`, `armor_wear_of`, `ArmorStage` returns **zero hits**. The sim publishes all three (FRIDAY_LOG §C) and the client reads none. | 06 | depends-on TRV-0137 | 08-10 |
| TRV-0037 | "**Mech boarding: 8 tested sim stages, 7 render a `debug!` line no player sees.** The largest built-but-invisible system in the game. The strings already exist verbatim inside the debug calls." | WHATS_MISSING.md:135-137 | o | C | PARTIAL | `main.rs:19191` `mech_stage_presentation` now reads `mech_enter_stage_for` (so the "ZERO client references" era is over), plays a rising `click` per beat, and drives `visor_ready`. But `:19243-19254` is eight `debug!` arms and nothing else. Stage 8 alone has a visible consequence. | 05 | — | 08-10 |
| TRV-0038 | "**Pyro armour is unobtainable on every map** while the controls screen still sells its flame ability — and TWO per-map relocation tables still place a Pyro pad that never spawns." | WHATS_MISSING.md:138-140 | o | S | SUPERSEDED | Closed by TRV-0001. Keeping the row: this is where the ask started. | 01 | superseded-by TRV-0001 | 08-10 |
| TRV-0039 | "**Delete Cliffhold.** Half-done and reverted. Salvage first: the +25% scale trap, the flight-joint bug, and the reachability-test shape are all reusable." | WHATS_MISSING.md:141-143 | o | S+C | PARTIAL | Client half deleted in `4152240` ("its art findings stay and now dress all maps"), 16 Cliffhold captures deleted with it. Re-derived today: `sim.rs` still holds **49** `Cliffhold` references including `build_cliffhold` (`:1690`) and five named tests (`:26970-27700`); `main.rs` holds 3. | 08 | — | 08-10 |
| TRV-0040 | "**Make capture scripts DATA, not code** — Thor's highest-leverage finding. Beats are compile-time constants in a 28k-line file, which is WHY every framing tweak costs a 6-minute rebuild. ~40 s after." | WHATS_MISSING.md:144-146 | o | C | **NOT STARTED** | `main.rs:5203` `struct CapBeat` and every script is a compile-time const array. Re-derived by grep: no external script file, no parser. | 13 | unblocks TRV-0008, 0056, 0243 | 08-10 |

**TIER 3 — MULTI-SESSION**

| ID | Ask | Issued at | O | L | Status | Evidence | Thread | Links | Chk |
|---|---|---|---|---|---|---|---|---|---|
| TRV-0041 | "**The three core maps: +10% larger, real elevation, high ground, randomised structures.** The trap: 'randomised' must be seeded at map-BUILD time from the match seed." | WHATS_MISSING.md:149-153 | o | S+C | NOT STARTED | No `+10%` pass exists; the live scale change is the earlier central `MAP_SCALE` +25% (`sim.rs:1509`). No randomised-structure path in `build_map`. | 08 | — | 08-10 |
| TRV-0042 | "Fix the two lying centrepieces (Bailey's keep is argument-for-argument Dust Arena's tower; Gardens' gazebo is a solid block)." | WHATS_MISSING.md:153-155 | o | S | UNVERIFIED | Not re-derived. Carrying the scout's original evidence (0-SCOUT #3) forward rather than inventing a disposition. | 08 | — | 08-10 |
| TRV-0043 | "**Bot navigation, properly.** Waypoints are 2D with no height and no reachability check. Bots cannot choose to climb." | WHATS_MISSING.md:156-158 | o | S | PARTIAL | §owner BOT ROUTING shipped: published up-links (`sim.rs:1009`), `BOT_PROBE_Y` (`:167`), `route_waypoint` (`:13608`), five tests (`:27338-27700`). But `sim.rs:27649` states the FLAT maps were deliberately left where they were, and I did **not** re-derive whether `waypoint` is still `[f32; 2]`. | 09 | — | 08-10 |
| TRV-0044 | "**Soldier finger ANIMATION.** The hand poses from one `curl` that is still a spawn-time argument." | WHATS_MISSING.md:158-159 | o | C | UNVERIFIED | Not re-derived. `afbe9d2` built the instrument that could check the fingers; whether `curl` is still spawn-time-only I did not confirm. | 17 | — | 08-10 |
| TRV-0045 | "**Weapon crafting station** — shares a front end with the Forge per-piece grid. Build both or neither." | WHATS_MISSING.md:160-161 | o | C | NOT STARTED | No crafting surface in `menu_ui.rs` or the intro pages. | 07 | depends-on TRV-0076 | 08-10 |
| TRV-0046 | "**Ragdoll + hit-reaction impulse.** The rig's mass/inertia data is complete, tested, and read by nothing." | WHATS_MISSING.md:162-163 | o | S+C | NOT STARTED | `segment_data` / `segment_inertia` exist and are tested; `derived_spring_k` is the only consumer. | 17 | depends-on TRV-0106 | 08-10 |
| TRV-0047 | "**Mech control scheme** — worth doing now that jump, crouch and the cockpit exist to be controlled." | WHATS_MISSING.md:164-165 | o | C | NOT STARTED | No mech-specific control layer; the chassis reuses the infantry bindings with three mech rows in `BIND_REGISTRY`. | 01 | — | 08-10 |

**TIER 4 — BLOCKED BY A NAMED THING**

| ID | Ask | Issued at | O | L | Status | Evidence | Thread | Links | Chk |
|---|---|---|---|---|---|---|---|---|---|
| TRV-0048 | "**Your uploaded gun assets: `jk_tdm` HAS NO glTF LOADER.** Only `jk_bevy` does. A GLB in `assets/characters/` changes nothing in the shipping game. Writing that loader is the actual task." | WHATS_MISSING.md:169-171 | o | A | BLOCKED | Unblocker, named: **write a glTF loader in `jk_tdm`**. Re-derived today: 24 `asset_server.load` calls in the crate — 20 `.wav`, 4 branding `.png`, zero mesh. `engine/assets/characters/` contains only `.gitkeep`. | 14 | — | 08-10 |
| TRV-0049 | "**Texture pipeline** — procedural textures exist now (Cliffhold rock, metal, wood); no IMPORTED image is used as a world texture, and the older maps' cover materials are still flat." | WHATS_MISSING.md:172-174 | o | C | PARTIAL | Procedural generation shipped (`main.rs:14160` "generated once at startup, then shared"; `:14186` tangents; `:25101` the generators are pure functions). No imported image reaches a world material — confirmed by the load list above. | 14 | — | 08-10 |
| TRV-0050 | "Networking (zero deps)" | WHATS_MISSING.md:175 | o | S | BLOCKED | Unblocker: a networking dependency and a decision to have one. `Cargo.toml` has none. Scoreboard deliberately omits a Ping column for this reason. | 21 | — | 08-10 |
| TRV-0051 | "traversal (blocked on map metrics)" | WHATS_MISSING.md:175 | o | S | BLOCKED | Unblocker, named: `MAP_METRICS.md` (TRV-0149), which was never written. | 09 | depends-on TRV-0149 | 08-10 |
| TRV-0052 | "full character customisation" | WHATS_MISSING.md:175-176 | o | C | PARTIAL | Helmets are 5 shapes × 4 tints, a real part library (`5c1eb2d`); 24 armour plates with a Forge page; four classes. Body/face sliders and weapon cosmetics: 0. | 07 | — | 08-10 |
| TRV-0053 | "**Mech first-person aim sits 1.10 m above the muzzle** — camera at the visor (2.72 m), weapon fires from `EYE_REL` (1.62 m)... Changing it moves every mech engagement, hit test, cover line and tracer in the game." | WHATS_MISSING.md:178-182 (filed by the plan itself as "A DECISION FOR THE OWNER, not a task") | o | S | BLOCKED | Unblocker, named: **an owner ruling.** Arithmetic independently confirmed by Thor: `BODY_HEIGHT 1.78 × MECH_SCALE 1.7 × MECH_VISOR_Y_FRAC 0.90 = 2.7234`; `2.7234 − 1.62 = 1.1034 m` (THOR_LOG §4). Also raised by Friday unprompted (FRIDAY_LOG §C, `sim.rs:8165` and `:11084`). | 18 | — | 08-10 |

### B.3 — `WHATS_MISSING.md` §0-NOW — the post-six-agent list (2026-08-08)

| ID | Ask | Issued at | O | L | Status | Evidence | Thread | Links | Chk |
|---|---|---|---|---|---|---|---|---|---|
| TRV-0054 | "The jump telegraph has no client reader... ~0.95 s of model/hitbox disagreement per jump. Key the rig on `chassis_kneeling()` + `mech_jump_compression_of()`, not raw `f.crouch`." | WHATS_MISSING.md:191-198 | a→o | C | DELIVERED (contested) | `main.rs:17532` `§owner THE MECH JUMP HAS A LOAD POSE`, `:19300` and `:19528` the camera's half, test `:26181`; captures `mech_jump/01..06.png`. Contested: I did not confirm the rig keys on `chassis_kneeling()` rather than `f.crouch`. | 01 | — | 08-10 |
| TRV-0055 | "`gatling_heat` carries TWO scales in one sim field — 0..100 for the heavy's gatling, 0..1 for the medic's plasma... Split it or document it." | WHATS_MISSING.md:214-216 | a→o | S+C | **NOT STARTED** | Re-derived today: `main.rs:21763`, `:21769`, `:21772` print `p.gatling_heat * 100.0` under a `%`; `:21781` prints the **raw** `p.gatling_heat` under the same `%`. Both branches are live. | 11 | — | 08-10 |
| TRV-0056 | "**Capture verification owed** for four fixes that landed unphotographed: the per-owner turret spinners, the world minigun spin, the kneeling crouch drop, and the grenade throw." | WHATS_MISSING.md:210-213 | a→o | C | PARTIAL | Grenade throw: **closed** — `grenade_hold/01..07.png`. Kneeling crouch drop: partially covered by `mech_jump/*`. Per-owner turret spinners and world minigun spin: **still no capture**. | 13 | depends-on TRV-0040 | 08-10 |
| TRV-0057 | "`visor_ready` promises a camera that does not exist — it is a Bevy `Local`, so nothing outside its own system CAN read it." | WHATS_MISSING.md:237-238 | a→o | C | PARTIAL | Re-derived today, and the claim **survives**: `main.rs:19193` `mut st: Local<MechStageState>`, and `visor_ready` is a field of that struct (`:19145`). It is written at `:19204`/`:19215`/`:19222` and read by nothing outside `mech_stage_presentation`. What changed since the claim was written is that it now has a pure function (`visor_ready_after`) and a real test (`:25838`) — so it is correct, just still unreachable. | 05 | — | 08-10 |
| TRV-0058 | "`SCOUT_SCALE = 1.42` is read by nothing — the Mechanical Medic renders man-sized despite a constant documented as the reason it 'reads as a different silhouette from across a map'." | WHATS_MISSING.md:230-232 | a→o | S | DELIVERED | Wired: `sim.rs:4893` `ArmorSet::ScoutMech => SCOUT_SCALE` inside `chassis_scale`, consumed by `height()` (`:3710`); test `sim.rs:15743`. Value is now **1.05**, not 1.42 — see TRV-0059. | 01 | superseded-by TRV-0059 (the intent question) | 08-10 |
| TRV-0059 | (Thor) "If the owner's intent was 'reads as a machine, not a man', 1.05 does not deliver it. Owner call, not a defect." | THOR_LOG.md:1741-1742 | a | S | BLOCKED | Unblocker, named: **an owner ruling on `SCOUT_SCALE`.** 1.05 × 1.78 = 1.869 m — a big man, not a machine. Thor also flags (`:1696-1699`) that the scout is now the only fighter whose hitbox height never changes in any stance. | 01 | depends-on owner | 08-10 |
| TRV-0060 | "§16/§17 projectile origin audit — all five projectile types, both views, never checked together." | WHATS_MISSING.md:245-246 | o | C | NOT STARTED | Historically partial (mech fire used to leave screen centre; fixed in `77a5898`). No pass has covered bullets / rockets / arrows / spears / grenades in both views at once. | 18 | — | 08-10 |
| TRV-0061 | "§19 HUD redesign — deliberately last; it must unify readouts that are still being added." | WHATS_MISSING.md:246-247 | o | C | NOT STARTED | Deliberate ordering, recorded at the source. Not a gap yet. | 11 | depends-on TRV-0036, 0037, 0055 | 08-10 |
| TRV-0062 | "§29 throwing consistency" | WHATS_MISSING.md:256 | o | S | UNVERIFIED | Not re-derived. | 02 | — | 08-10 |
| TRV-0063 | "§28 arc-attached distance readout" | WHATS_MISSING.md:256 | o | C | NOT STARTED | 0-SCOUT corrects the parent claim: the max-range indicator and the metre readout both exist in the status line; only the ARC-ATTACHED distance is missing. | 02 | — | 08-10 |
| TRV-0064 | "§11 turret recoil: muzzle climb and the hull answering the shot (the camera-kick half is live and tested)." | WHATS_MISSING.md:257-258 | o | C | DELIVERED | `mech_recoil.rs` — `CLIMB_RAD_PER_DEG` (:78) is the muzzle climb, `SHOVE_M_PER_DEG_S` (:57) + `SHOVE_MAX_M` is the hull answering the shot, and both read the sim's own `punch`/`punch_vel` rather than a client table (the split-brain the module exists to prevent). | 03 | — | 08-10 |
| TRV-0065 | "§32 a real first/third-person consistency pass. Three separate instances of that defect were found in ONE session." | WHATS_MISSING.md:259-260 | o | C | NOT STARTED | No systematic pass. Individual instances fixed (`182f354`, `77a5898`). | 18 | — | 08-10 |

### B.4 — `briefs/BRIEF_VII_optimized.md` (spec; superseded by VIII for content, kept because the operating contract originates here)

| ID | Ask | Issued at | O | L | Status | Evidence | Thread | Links | Chk |
|---|---|---|---|---|---|---|---|---|---|
| TRV-0066 | §0 "AUDIT: WHY DID NOTHING VISIBLY CHANGE? ... Root cause must name one of: branch never merged / stale binary / flag defaulted off / registered but never scheduled / below perceptual threshold / asset missing / code orphaned." | BRIEF_VII §0.1-0.4 | o | D | DELIVERED | `handback/brief-vii/HANDBACK.md`; `handback/AUDIT.md`; before/after viewmodel pair in `baseline/01..05.png` | 13 | superseded-by TRV-0078 | 08-10 |
| TRV-0067 | §1 "no character is ever a statue. Soldiers breathe, shift, look, react." (breathing 12→30/min, weight shift 6-12 s, head look ±25°, grip fidget 8-15 s) | BRIEF_VII §1.1 | o | C | DELIVERED | `handback/brief-vii/idle_life/00s|04s|08s|12s|16s.png` — the statue test's own capture | 17 | — | 08-10 |
| TRV-0068 | §1.2 "Posture states: Relaxed (10 s no combat) / Alert / Suppressed (projectile within 2 m) / Low HP (<25%) posture sag." | BRIEF_VII §1.2 | o | C | PARTIAL | Suppressed: DELIVERED (`main.rs:20568` `§owner SUPPRESSION, the PLAYER's half`, `:22533`; `sim.rs:15448` test). Relaxed / Low-HP sag: **UNVERIFIED**, not re-derived. | 17 | — | 08-10 |
| TRV-0069 | §2 "ARMS, JOINTS, AND HANDS: THE CRAFT PASS — twist bones mandatory, metacarpals mandatory, joint limits from measured human active ROM, DIP ≈ 0.7×PIP, trigger finger independent." | BRIEF_VII §2.1-2.5 | o | C | DELIVERED | `c394208` "§1 the mech gets hardware; §2 the SOLDIER gets hands"; `ELBOW_FLEX_MIN/MAX_DEG`, `DIP_PIP_COUPLING` in `main.rs`; captures `hands/01-hands-front.png`, `02-hands-quarter.png` | 17 | — | 08-10 |
| TRV-0070 | §2.0 "Search and save 10-15 stills into `handback/brief-vii/section-2/reference/` ... Write `reference/NOTES.md`." | BRIEF_VII §2.0 | o | A | **NOT STARTED** | The directory does not exist. `handback/reference/NOTES.md` exists at a different path and opens by declaring the image deliverable "**not achievable** with the tools available this session". Honest, and still an open ask. | 24 | duplicate-of TRV-0083, 0168 | 08-10 |
| TRV-0071 | §3 "SPEAR THROW — hold to raise overhead, release to throw with the whole body, learnable arc, spear sticks in and is retrievable. **No trajectory arc. No landing marker.**" | BRIEF_VII §3.1-3.3 | o | S+C | DELIVERED | `sim.rs` charged javelin (`:6601`, `:9997`); captures `spear_flight/00-charging.png`..`05-landed.png`; test `sim.rs:18414` (winding must buy something) | 19 | — | 08-10 |
| TRV-0072 | §4 "BOW — full draw in 0.7 s, letdown under 0.15 s, sway ramp ±0.4°→±1.2° over 4 s, forced letdown at 10 s, 55 m/s, pierce 3 at 90/68/45, quiver 12." | BRIEF_VII §4.1-4.2 | o | S+C | DELIVERED | `054a283` (full-draw sway); captures `bow_draw/01..05.png`, `bow_draw_fp/01..05.png`, `arrow_flight/00..05.png` | 19 | — | 08-10 |
| TRV-0073 | §5 "THIRD-PERSON CAMERA + POSITIONING — hip boom 2.2 m/+0.45 right/+0.12 up, sprint 2.5 m, aim 1.35 m/+0.55/FOV −12°, shoulder swap, spring collision k=90, ±60° upper-body additive." | BRIEF_VII §5.1-5.2 | o | C | DELIVERED (contested) | `CameraTuning`, `boom_step`, `step_leg_yaw` (turn-in-place); `config/camera_tuning.txt`. Contested: the §5.3 capture — "walk → strafe-aim → zoom → scope handoff → unscope" — has no clip. | 18 | — | 08-10 |
| TRV-0074 | §6 "MECH OVERHAUL — full PBR per mesh, entry 1.6 s committed / exit 1.2 s, damage states at 70/40/15% with plates detaching as physics debris, alive idle." | BRIEF_VII §6.1-6.3 | o | C | DELIVERED | `f0b6b46`, `939df7c`, `b116a46`, `d6aa356`; captures `mech_scale/01..03.png`, `cockpit/01..08.png` | 05 | — | 08-10 |
| TRV-0075 | §6.1 "**Silhouette check:** render front/side/¾ beside the concept art in the handback." | BRIEF_VII §6.1 | o | A | BLOCKED | Unblocker, named: **the concept art is not in this repository.** See TRV-0260. | 24 | depends-on TRV-0260 | 08-10 |
| TRV-0076 | §7 "THE FORGE — left category list → part grid with thumbnails, right large turntable, **one click = one visible change**, first-person preview toggle, randomize, reset-per-part, 3 saved slots reachable from the main menu AND the lobby." | BRIEF_VII §7.1-7.2 | o | C | PARTIAL | Shipped: turntable (`62cd0e5`), SAVE/LOAD/RANDOMIZE, the armour page (four rows by body region, live weight-vs-ceiling readout), 3 slots. Absent: the per-piece category grid with thumbnails, the first-person preview toggle, reset-to-default per part. | 07 | duplicate-of TRV-0095 | 08-10 |
| TRV-0077 | §8 "HANDBACK FOR PLAYTEST — the §0 table updated to 'after', every gate's pass/fail, the full capture set, feel questions answered concretely, every tunable with value and location, anything not done named plainly." | BRIEF_VII §8 | o | D | DELIVERED | `handback/brief-vii/HANDBACK.md`, `handback/REPORT.md`, `handback/CHANGES.md`, `handback/ACCOMPLISHMENTS.md` | 13 | — | 08-10 |

### B.5 — `briefs/BRIEF_VIII_master.md` (the master brief, 51 KB)

| ID | Ask | Issued at | O | L | Status | Evidence | Thread | Links | Chk |
|---|---|---|---|---|---|---|---|---|---|
| TRV-0078 | §0 "AUDIT BEFORE ANYTHING — the evidence table, runtime truth not config truth, five specific probes, and the HARD GATE: existing viewmodel work visibly on screen before §1." | BRIEF_VIII §0 | o | D | DELIVERED | `research/TASK0_AUDIT.md` — 17 rows, each verified against code, with root causes | 13 | supersedes TRV-0066 | 08-10 |
| TRV-0079 | §1.2 "Sprint-derived locomotion — acceleration lean 28-32° easing to 8-12°, knee drive ~130°, arm drive elbows ~90° hip-to-chin, cadence over reach, contact under the centre of mass, hip bob ≤4 cm." | BRIEF_VIII §1.2 | o | C | DELIVERED | `main.rs:2303` (lean cap), `:2311` (knee flexion), `:2316` (shoulder swing), `:17470` (the sprinter driving off the line), `:17606` (KNEE DRIVE), `:18163` (ARM DRIVE) — all `§owner ATHLETIC MOTION` | 17 | — | 08-10 |
| TRV-0080 | §1.3 "No instant reorientation... **Full stops never hit instant zero.** Starts get 2-3 frames of anticipation." | BRIEF_VIII §1.3 | o | S | DELIVERED | `a1fb256` "the acceleration model — full stops never hit instant zero"; `925a2e2` turn-in-place; `step_leg_yaw` + its convergence test | 17 | — | 08-10 |
| TRV-0081 | §1.4 "The Universal Kinetic-Chain Rule — build it ONCE. Route every power move through it: spear throw release, spear thrust, dodge/roll launch, mech braced side-step, melee shove, sprint start." | BRIEF_VIII §1.4 | o | S | DELIVERED | `01ebf21` (kinetic chain wired into spear throw release + thrust recovery); `3c7059f` (sprint-start head lag); `c44407e` (mech side-step) | 17 | — | 08-10 |
| TRV-0082 | §2 "ARMS, JOINTS, HANDS ★ RESTORED — bone hierarchy, joint limits table, two-bone IK + pole vector, grip pose library, the spring solver built once." | BRIEF_VIII §2.1-2.5 | o | C | DELIVERED | See TRV-0069. Spring solver: `b725674` "wire the four orphaned secondary-motion springs" | 17 | duplicate-of TRV-0069 | 08-10 |
| TRV-0083 | §2.0 "save 10-15 stills to `handback/brief-viii/section-2/reference/`... If no web access, say so and proceed." | BRIEF_VIII §2.0 | o | A | **NOT STARTED** | Directory does not exist. The "say so" half was honoured in `handback/reference/NOTES.md`. | 24 | duplicate-of TRV-0070 | 08-10 |
| TRV-0084 | §3 "FIRST PERSON — no ADS ever; scoping hides the weapon entirely; the viewmodel does not translate/bounce; recoil in three channels with the crosshair pinned to screen centre; deterministic spray tables." | BRIEF_VIII §3.1-3.5 | o | C | DELIVERED | `9b92808`, `9d29d4c`, `23379e5`; screen-intrusion sweep + `weapon_bounded_extent` (`e5431a4`); captures `baseline/02..05.png`, `shield_fp/01-02.png`, `mech_fp/04-pause-no-viewmodel.png` | 18 | — | 08-10 |
| TRV-0085 | §3.2 "**On-weapon ammo display:** emissive segmented bar on the left receiver face, mirroring magazine fraction, segments extinguishing as rounds are spent, one pulse on reload complete. Colour driven by the Forge accent." | BRIEF_VIII §3.2 | o | C | UNVERIFIED | Not re-derived. I did not find a matching symbol by grep and did not search exhaustively. | 19 | — | 08-10 |
| TRV-0086 | §3.4 "Weapon handling states — sprint carry (18° lower, sprint-out 0.15/0.20/0.30 s by class), ready-up on stop, reload craft (tactical vs empty as separate clips), low-ready/obstruction 22° up-and-in within 0.6 m, inspect, hit feedback." | BRIEF_VIII §3.4 | o | C | PARTIAL | Sprint-out gate DELIVERED (`d6259b8`); empty-vs-tactical reload cost DELIVERED (`sim.rs:9534`, test `:19346`); inspect on `T` DELIVERED; hitmarker + headshot tone + kill-pop DELIVERED. **Low-ready/obstruction: UNVERIFIED.** | 19 | — | 08-10 |
| TRV-0087 | §4 "THE HUD — four corners, one info cluster each; centre empty but for the crosshair; three semantic colours; UI scale 0.85; Saira SemiCondensed; safe area 5%." | BRIEF_VIII §4 | o | C | DELIVERED | `f35b5b6`, `1523123`, `77a5898` (the UI gets its own Camera2d at order 2); captures `menus/01..08` | 11 | — | 08-10 |
| TRV-0088 | §4.6 "Crosshair — full settings family: size, gap (negatives allowed), thickness, dot, outline, colour presets + custom RGB, alpha, T-shape, static/dynamic (default classic static)." | BRIEF_VIII §4.6 | o | C | DELIVERED | `30bd463`; FRIDAY_LOG crosshair entry — 20 mutations, 20 killed; pixel-measured against a non-default settings file (predicted x −8..+7, y −5..+7; measured identical); capture `menus/04-settings.png` | 11 | — | 08-10 |
| TRV-0089 | §4.3 "Resource counter below [the minimap]; award toasts stack above it, each fading after 2.5 s." | BRIEF_VIII §4.3 | o | C | PARTIAL | Toasts DELIVERED (`00e185e`). The **resource** half has no economy in TDM/KOTH to award from — recorded as waiting on a mode that has one, rather than faked. | 11 | — | 08-10 |
| TRV-0090 | §4.8 "Scoreboard columns K/A/D/DMG/Score/Ping" | BRIEF_VIII §4.8 | o | C | PARTIAL | K/A/D/DMG DELIVERED (`6d29217`, `08b3ebb`). **Ping deliberately absent** — no netcode, documented at the site. A decision, not a gap. | 11 | depends-on TRV-0050 | 08-10 |
| TRV-0091 | §5 "SPEAR: THE ATHLETIC REWORK — momentum carry, raise/bow 0.4 s with 35-45° hip-shoulder separation, release 0.25 s sub-timed into plant/whip/follow-through, 22 m/s, ×1.15 running bonus, 85 body ×2 head ×0.75 legs." | BRIEF_VIII §5.1-5.5 | o | S+C | DELIVERED | `baf50ca` (running-throw bonus, and its test caught a real conflict with the sprint-out gate before it shipped); `4840784` (charged throw); separation test in the rig suite | 19 | — | 08-10 |
| TRV-0092 | §6 "THIRD PERSON ★ RESTORED — camera rig states, 8-way strafe blend, upper-body additive ±60°, camera-relative movement, toggle on V, works on foot and in the mech." | BRIEF_VIII §6 | o | C | DELIVERED | See TRV-0073. `V or O` in `BIND_REGISTRY` (`main.rs:5081`) | 18 | duplicate-of TRV-0073 | 08-10 |
| TRV-0093 | §7 "THE MECH — scale, silhouette and materials, grounded (flight deleted), mobility kit, rig/animation/weight, entry/exit/idle life, angle-based damage with plates that drop, piloting and weapons." | BRIEF_VIII §7 | o | S+C | DELIVERED | `2f8356b` (flight deleted), `f0b6b46`, `9b26280`, `d845250`, `f9f6eef` (power stride), `7e501b6` (the pilot sees from the visor) | 01 | §7.1 superseded-by TRV-0099 | 08-10 |
| TRV-0094 | §7.8 "Targeted missile pod (lock-on): valid targets mechs, deployables/turrets, marked structures ONLY — **no lock on infantry** (the anti-oppression rule, do not soften)." | BRIEF_VIII §7.8 | o | S | DELIVERED | `2f8356b`; `main.rs:5078` the `Y (hold)` bind row states it verbatim; `sim.rs:4287` `§owner rocket table` | 01 | — | 08-10 |
| TRV-0095 | §8 "THE FORGE — content counts, the editor, in-match application, faction-readability guard." | BRIEF_VIII §8 | o | C | PARTIAL | See TRV-0076. Faction-readability guard: DELIVERED in spirit — team colour rests on luminance, not hue — but **unguarded by any test** (TRV-0206). | 07 | duplicate-of TRV-0076 | 08-10 |
| TRV-0096 | §9 "HANDBACK FOR PLAYTEST — the §0 table updated to 'after'... **This table is the receipt.**" | BRIEF_VIII §9 | o | D | DELIVERED | `handback/REPORT.md`, `handback/ACCOMPLISHMENTS.md`, `TASK0_AUDIT.md` | 13 | — | 08-10 |
| TRV-0097 | Appendix A "**The bow** ... is **not** in this brief's scope. It is parked, not cancelled." | BRIEF_VIII App. A | o | S | DELIVERED | Built anyway across `c727fd0`, `a4d2070`, `4be8701`, `68d325b`, `054a283`. Recording it because a parked ask that ships is still an ask that was tracked. | 19 | — | 08-10 |
| TRV-0098 | §7.2 "**Material audit test:** iterate every submesh; assert none uses a default/placeholder material and every material has non-empty texture slots. **Untextured gray is a CI failure, not an opinion.**" | BRIEF_VIII §7.2 | o | C | UNVERIFIED | Not re-derived. The procedural texture pass (`52684be`) likely satisfies the spirit; whether the CI assertion exists I did not check. | 14 | — | 08-10 |
| TRV-0099 | Non-negotiable 8: "**The mech ... is 1.15× soldier height** — not 1.5×, not concept scale." | BRIEF_VIII non-neg. 8 | o | S | SUPERSEDED | Superseded by `BRIEF_VIII_B` §A, option A3 = 1.7×. Live value `MECH_SCALE = 1.7`. Keeping the row: this is why two documents disagree about the mech's height. | 01 | superseded-by TRV-0101 | 08-10 |
| TRV-0100 | §7.9 "**Captures:** entry clip, damage progression 100→15% under scripted fire, power-stride and brace clips, **concept-art side-by-side**, front/side/rear/visor stills." | BRIEF_VIII §7.9 | o | A | PARTIAL / BLOCKED | Delivered: `cockpit/*`, `mech_scale/*`, `mech_jump/*`, `barrier/*`. **The concept-art side-by-side is unsatisfiable** — the art is not in the repo. Damage progression 100→15% under scripted fire: no capture. | 24 | depends-on TRV-0260 | 08-10 |

### B.6 — `briefs/BRIEF_VIII_B_addendum.md` (corrections after concept art; where it disagrees with VIII, **this file wins**)

| ID | Ask | Issued at | O | L | Status | Evidence | Thread | Links | Chk |
|---|---|---|---|---|---|---|---|---|---|
| TRV-0101 | §A "Pick one, explicitly, and write the choice into `config/mech.ron` ... **Recommendation: A3 (1.7×).**" | BRIEF_VIII_B §A | o | S | PARTIAL | Decision DELIVERED: A3 taken, `MECH_SCALE = 1.7` (`c6f55af`, Task 4). The `config/mech.ron` half was **not** done — this repo uses hand-rolled `key=value` text files by a documented deliberate convention (`BACKLOG.md`: "introducing a new dependency and file format for its own sake"). Recording the divergence rather than pretending either side is wrong. | 01 | supersedes TRV-0099 | 08-10 |
| TRV-0102 | §B.1 "The 20 segments" (head+neck, thorax, lumbar, pelvis, 2 clavicles, arms, forearms+twists, hands, thighs, shanks, feet, toes) | BRIEF_VIII_B §B.1 | o | C | DELIVERED | `research/body-rig/SPEC_20_SEGMENT_RIG.md`; commits `38c8ecc`, `787f6ff`, `1e774e1`; segment-count test | 17 | — | 08-10 |
| TRV-0103 | §B.2 "**The critical fix: three trunk segments** ... Build the three-part trunk first — it unblocks the rest of the brief." | BRIEF_VIII_B §B.2 | o | C | DELIVERED | Lumbar added; twist shared 38/62 rather than landing on one hinge. The separation test **fails before and passes after** — the stated proof. | 17 | unblocks TRV-0091 | 08-10 |
| TRV-0104 | §B.3 "Segment mass fractions ... Whole-body check: = 1.000" | BRIEF_VIII_B §B.3 | o | C | DELIVERED | Mass-closure test live, and it catches the brief's own trap — the clavicles are carved FROM the thorax, not added beside it | 17 | — | 08-10 |
| TRV-0105 | §B.4 "Segment lengths (fraction of total height H) ... **Use these to validate the existing rig**" | BRIEF_VIII_B §B.4 | o | C | DELIVERED | Proportion test: every segment within ±5% of its fraction | 17 | — | 08-10 |
| TRV-0106 | §B.5 "Inertial properties ... Feed them into the §2.5 spring solver so the spring stiffness per segment is **derived from mass, not hand-guessed**." | BRIEF_VIII_B §B.5 | o | C | PARTIAL | Data + `derived_spring_k` DELIVERED (`07e48ad`). The two named payoffs — **ragdoll and hit-reaction impulse** — read the inertia column nowhere. | 17 | see TRV-0046 | 08-10 |
| TRV-0107 | §B.6 "Tests — segment-count, separation, proportion, mass-closure, toe-off" | BRIEF_VIII_B §B.6 | o | S | DELIVERED | All five live (`787f6ff`, `1e774e1`) | 17 | — | 08-10 |
| TRV-0108 | §C "The Elastic Load Model — `ElasticMove { load_s, release_s, stored_energy, return_efficiency: 0.92 }`; release 2-3× faster than load; stored energy scales output ×(1 + e×0.35)." | BRIEF_VIII_B §C.1-C.2 | o | S | DELIVERED | `c6f55af` (`ElasticMove`, counter-movement bonus, kinetic chain sequencing, landing rebound wired into the real landing camera) | 17 | — | 08-10 |
| TRV-0109 | §C.2 rule 3 "A counter-movement beats a dead start ... **the same rule should govern the jump, the dodge launch, and the melee thrust**, not just the throw." | BRIEF_VIII_B §C.2 | o | S | PARTIAL | Dodge launch DELIVERED (`a_counter_movement_dodge_launches_harder`, `TASK0_AUDIT.md`). Jump and melee thrust: **UNVERIFIED** — Thor's 143-finding audit named "counter-movement bonus not reaching jump" and I did not re-derive it. | 17 | — | 08-10 |
| TRV-0110 | §C.2 rule 4 "**Landings recharge.** A landing that flows into the next move carries stored energy forward." | BRIEF_VIII_B §C.2 | o | S | UNVERIFIED | Not re-derived. Thor's audit also named `ElasticMove.return_efficiency` as an assigned-but-never-read field. | 17 | — | 08-10 |
| TRV-0111 | §D.2 "**CORRECTION to Brief VIII §7.2** ... The art is **olive drab / khaki / field tan** ... `hull_primary #8A8770`, `mechanism_dark #33352F`, `barrel_metal #2B2C2B` ... Emissive: **minimal**. The art has no glowing visor." | BRIEF_VIII_B §D.2 | o | C | SUPERSEDED | Superseded by the owner's later TEAM IDENTITY and BLUE ENEMY MECHS asks, which the whole live palette is built on (`branding.rs:96-152`, `main.rs:14361`). The build has an emissive visor and a faction colour language; the addendum asked for neither. **Recording it because nobody wrote down that the olive-drab spec was retired** — it was overtaken, not decided against. | 04 | superseded-by TRV-0194, 0198 | 08-10 |
| TRV-0112 | §D.3 "Part inventory (mirrors the 20-segment body — these are also the Forge's swappable/detachable plates)" — the 20 named parts | BRIEF_VIII_B §D.3 | o | C | PARTIAL | The heavy carries ~53 plates and three detach stages (`main.rs:11414`), and they are group nodes so a damage stage sheds a whole cluster. They are **not** D.3's 20 named parts, and no part-count test asserts the list. | 04 | — | 08-10 |
| TRV-0113 | §D.5 "**D5-a (matches the art):** left arm gatling, right arm autocannon; the missile pod becomes an optional swappable hardpoint, **absent by default**." | BRIEF_VIII_B §D.5 | o | S+C | PARTIAL | The autocannon exists as a real `MechWeapon` with its own recoil (`sim.rs:23271`). The missile pod is still **core** — bound to `Y`, ten tubes per chassis (`sim.rs:4274`), not optional and not absent. The reconciliation the brief demanded ("write the decision down") was not written. | 04 | — | 08-10 |
| TRV-0114 | §D.6 "Damage states mapped to the actual parts — 70%: hip skirts + left thigh plate; 40%: shin plate + one rear hull drum, antenna bends; 15%: drum casing chips, sensor plate cracks, foot cleat row tears away." | BRIEF_VIII_B §D.6 | o | C | PARTIAL | Three detach stages exist and are tested; `main.rs:13946-13956` names `skirt_l`, `skirt_r`, `drum_r`, `antenna` — so the parts are close. The mapping to D.6's exact thresholds and the limp/smoke half: UNVERIFIED. | 04 | — | 08-10 |
| TRV-0115 | §D.7 "**Palette audit:** assert no mech material uses the old gunmetal values ... **Part-count test:** all 20 named parts exist as separate meshes ... **Silhouette test:** ... place it next to the concept art in the handback. **This side-by-side is the completion criterion.**" | BRIEF_VIII_B §D.7 | o | A | BLOCKED | Palette audit + part-count test: NOT STARTED. Silhouette side-by-side: **BLOCKED — the art is not in the repo.** | 24 | depends-on TRV-0260 | 08-10 |

### B.7 — `briefs/BRIEF_IX_castle_grenade_customization.md`

**IX-A — Castle Map Design**

| ID | Ask | Issued at | O | L | Status | Evidence | Thread | Links | Chk |
|---|---|---|---|---|---|---|---|---|---|
| TRV-0116 | "**Architecture: three elevation tiers** — Tier 1 ground (0 m), Tier 2 wall/tower walk (4.0-6.0 m), Tier 3 tower summit (12.0-14.0 m)." | BRIEF_IX-A §Architecture | o | S | NOT STARTED | No castle map with these bands exists. Cliffhold built occupied bands at 0/5/6/7/8/11/12/18/24/25/32 m (`FRIDAY_LOG` root) and then had its client half deleted (`4152240`). | 08 | see TRV-0039 | 08-10 |
| TRV-0117 | "**The 40 m unobstructed rule** — no two player positions can see each other across more than 40 m of open space without cover." | BRIEF_IX-A §Sightline | o | S | NOT STARTED | The validator exists (`max_unobstructed_sightline`) and all four maps were measured against it: **none pass**, worst-case 80-510 m (`GAME_STATUS_REPORT.md`). Measured, named, unfixed. | 08 | — | 08-10 |
| TRV-0118 | "**The crossfire anchor** ... visible from Tier 2 LEFT and RIGHT simultaneously, but only at angles more than 60° apart" + three-material layering + detail density by tier | BRIEF_IX-A | o | S+C | NOT STARTED | No map is authored against these rules. | 08 | depends-on TRV-0116 | 08-10 |
| TRV-0119 | "**Primary: the Castle Heart** (Tier 1.5, 6 m radius, 12 s capture, a well connecting to a Tier 0 sub-level)" + "**Secondary: the Gatehouse Signal**" + "**Inversion at 5:00 game time**" | BRIEF_IX-A §Objectives | o | S | NOT STARTED | KOTH has one hill; there is no two-objective, tier-inverting mode. | 08 | depends-on TRV-0116 | 08-10 |
| TRV-0120 | "**Vertical movement audio callouts** — stairs soft, climbing routes stone-shifting, vaults metallic ring, drops rubble crunch" | BRIEF_IX-A | o | C | NOT STARTED | No surface-aware movement audio. | 10 | — | 08-10 |

**IX-B — Grenade Dynamics**

| ID | Ask | Issued at | O | L | Status | Evidence | Thread | Links | Chk |
|---|---|---|---|---|---|---|---|---|---|
| TRV-0121 | "**Grenades open mechanically** — not instant detonation. Fuse time of 3-5 s from release to blast, and that fuse is visible and audible in-world." | BRIEF_IX-B non-neg. 1 | o | S | DELIVERED | Fuse is 5 s timed from release; test `sim.rs:20686` proves the clock starts at release and not at equip — that test exists because the opposite was once true and killed the thrower | 02 | — | 08-10 |
| TRV-0122 | "**Opening sequences differ per type** — percussion detonates on impact (0.2 s), time grenades hiss for 4 s, gas grenades pop and spread over 2 s." | BRIEF_IX-B §Types | o | S | PARTIAL | Four `ThrowKind`s (Frag/Flash/Smoke/Molotov) cover the three conceptual roles. The distinct percussion and 3-phase gas variants were **named and refused** with a stated reason in `handback/brief-ix/REPORT.md` — a deferral, not a silence. | 02 | — | 08-10 |
| TRV-0123 | "**Blast radius scales, falloff is smooth** — 0-2 m 100%, 2-6 m 100→50%, 6-12 m 50→15%, 12-20 m 15→0%. No hard edge cliffs." | BRIEF_IX-B §Falloff | o | S | DELIVERED | `frag_falloff_frac`; tests `frag_falloff_matches_the_brief_ix_b_breakpoints` (4 exact breakpoints + a 200-point monotonicity sweep) and `frag_damage_reaches_the_full_20m_range`; commit `d60a61d` | 02 | — | 08-10 |
| TRV-0124 | "**Grenades interact with geometry** — bounce predictably off stone (0.40), wood/metal 0.50, organic 0.05 (sticks), water 0.30." | BRIEF_IX-B §Bounce | o | S | DELIVERED | `surface_restitution` + `surface_friction`; tests `sim.rs:25818`+; commits `605d88c`, `a3b89aa` | 02 | water half → TRV-0125 | 08-10 |
| TRV-0125 | "Water 0.30 — sinks at 0.3 m/s; blast radius reduced 30%." | BRIEF_IX-B §Bounce | o | S | BLOCKED | Unblocker, named: **no water volume exists in any map.** Recorded in `handback/brief-ix/REPORT.md` and `BACKLOG.md` #18. | 21 | — | 08-10 |
| TRV-0126 | "**Enclosed space amplification** — rooms under 3 m wide: blast radius +20%, falloff steeper; rooms over 10 m: −10%, gentler." | BRIEF_IX-B | o | S | NOT STARTED | Needs a "how enclosed is this point" query that does not exist. Recorded as buildable-but-not-rushed. | 02 | — | 08-10 |
| TRV-0127 | "**Height effects** — thrown downward +0.15 s air time per 2 m of drop; thrown upward −0.10 s per 2 m." | BRIEF_IX-B | o | S | NOT STARTED | **Flagged rather than guessed**: `handback/brief-ix/REPORT.md` records that the brief's own mechanism is underspecified and implementing it means inventing it. That is the right call and the row stays open. | 02 | — | 08-10 |
| TRV-0128 | "**Denial** — players can kick or melee a grenade away, moving it 3-5 m and resetting its bounce trajectory. Works only within 1 m and only if the grenade is visible." | BRIEF_IX-B §Counter-play | o | S | NOT STARTED | No such input path. | 02 | — | 08-10 |
| TRV-0129 | "**Suppression** — a thrower under active fire suffers +0.5 s ADS time before release (flinch)." | BRIEF_IX-B §Counter-play | o | S | UNVERIFIED | Suppression exists as a mechanic (`sim.rs:15448`); whether it taxes the throw specifically I did not check. | 02 | — | 08-10 |
| TRV-0130 | "**Environmental destruction** — grenades destroy weak scaffolding and hanging barriers, opening new routes or closing choke points." | BRIEF_IX-B §Counter-play | o | S | NOT STARTED | No destructible geometry. `BACKLOG.md` #10 has it as Medium with no design reason yet. | 08 | — | 08-10 |
| TRV-0131 | "**Grenades per loadout: 2.** ... Assault 1 percussion + 1 time; Scout 2 time; Heavy 1 gas + 1 percussion; Support 1 gas + 1 time. **Resupply:** refreshes on objective capture or on team spawn." | BRIEF_IX-B §Loadout | o | S | PARTIAL | `GRENADE_PRESETS` and `grenade_preset` exist (`sim.rs:7200`); they are **not** bound to the four shipped classes, and the class names differ from the brief's anyway (see TRV-0132). | 02 | depends-on TRV-0132 | 08-10 |

**IX-C — Character Customization**

| ID | Ask | Issued at | O | L | Status | Evidence | Thread | Links | Chk |
|---|---|---|---|---|---|---|---|---|---|
| TRV-0132 | "**Tier 1 — Class selection**: Assault 6.2 m/s ceiling 25 kg; Scout 6.8 / 20; Heavy 5.0 / 32; Support 5.8 / 24." | BRIEF_IX-C §Tier 1 | o | S | PARTIAL / DIVERGED | A four-class system **shipped** (`2af8a01`) — but as **LINE / SKIRMISHER / WARDEN / MARKSMAN**, hooked to health, movement, spread and swap speed, with a per-class silhouette, a Forge picker, and a test proving each of the three non-baseline classes trades (`sim.rs:15058`). Different names, different axes, same intent. Nobody wrote down that the brief's four were superseded. | 07 | supersedes the brief's names | 08-10 |
| TRV-0133 | "**Tier 2 — Armour customization**: piece weights, 15 rows summing to 26 pieces." | BRIEF_IX-C §Tier 2 | o | S | DELIVERED as **24** | Built to the table, not to the title: **the brief's own table sums to 24, not the 26 its heading claims** — recorded rather than reconciled by inventing two plates. Commits `33e46f6`, `d320af2`. | 06 | — | 08-10 |
| TRV-0134 | "**Movement penalty:** for each 1 kg over the class ceiling, movement drops 0.15 m/s." | BRIEF_IX-C §Tier 2 | o | S | DELIVERED and WIRED | `armor_weight_movement_penalty`, built pure in `03a5d10` and **wired** in the 24-plate pass. `BACKLOG.md` #11 still calls it "unwired" — **known false.** | 06 | corrects BACKLOG #11 | 08-10 |
| TRV-0135 | "**Tier 3 — Weapon selection**: weapon weight consumes the same budget as armour." | BRIEF_IX-C §Tier 3 | o | S | UNVERIFIED | Not re-derived. | 06 | — | 08-10 |
| TRV-0136 | "**Tier 4 — Cosmetic customization**: skin (4), palette (primary/secondary/tertiary), weapon paint (4), decals (team emblem / personal sigil / rank marker)." | BRIEF_IX-C §Tier 4 | o | C | NOT STARTED | The Forge saves hat colour, tunic colour, melee choice, grenade preset, helmet shape/tint and class. No paint, no decal, no skin. | 07 | — | 08-10 |
| TRV-0137 | "**Armour damage states** — 100% Fresh / 70% Scuffed (+5% resist) / 40% Cracked (+10%, piece tilts) / 15% Severed (detaches on next hit). Exposed segments take ×1.25." | BRIEF_IX-C §Damage states | o | S+C | PARTIAL | **Sim half DELIVERED**: 80-point pools, the exact thresholds and resistances, `ArmorPiece::struck` resolving the piece from geometry the hit already carries, detach as the unequipped path, 11 mutations each killing a named test (`192bd8d`, FRIDAY_LOG §C). **Client half absent** — see TRV-0036. "Piece tilts" at Cracked is unbuilt. | 06 | blocks TRV-0036 | 08-10 |
| TRV-0138 | "**Forge integration** — loadout preview turnaround, piece repair at 50 in-match points, free repainting, save/load 5 loadouts per class, class-swap confirmation." | BRIEF_IX-C §Forge | o | C | PARTIAL | Repair exists sim-side (§C repairs plate at 50 points, and condition deliberately survives respawn — pinned by `plate_condition_survives_a_respawn` so it reads as a decision, not the recurring loyal-ghost defect). Turnaround exists. 5-per-class, repaint and the class-swap warning: absent. | 07 | — | 08-10 |
| TRV-0139 | "**Preset loadouts** — 12 named presets across the four classes (Aggressive/Duelist/Fieldfare, Vanguard/Ghost/Archer, Tank/Reaper/Sentinel, Paladin/Surgeon/Bombardier)." | BRIEF_IX-C §Presets | o | C | NOT STARTED | No named presets exist. | 07 | depends-on TRV-0132 | 08-10 |
| TRV-0140 | "**Test gates** — customization readability at 30 m in black silhouette; weight and movement; damage state visibility photographed at 70/40/15%; armour coverage hierarchy Ghost vs Tank." | BRIEF_IX-C §Test gates | o | S | PARTIAL | Weight/movement: tested. Coverage hierarchy: the trim system makes coverage **monotonic in the trim**, which is the property a cheap test can pin, and it is mutation-proven. Damage-state visibility: **impossible until TRV-0036**. Squint test at 30 m: not automated. | 06 | depends-on TRV-0036 | 08-10 |
| TRV-0141 | "**Captures required** — Forge screen with 5 presets; light vs heavy silhouette at 30 m; damage progression in four frames; one character with three weapons at 30 m; the asymmetric anchor close-up." | BRIEF_IX-C §Captures | o | A | NOT STARTED | Only `trims/01-trims-front.png` and `02-trims-quarter.png` are in this family — and that lineup exists because reading the table only proves the code branches, not that the branches are worth having. None of the five listed captures exist. | 06 | depends-on TRV-0040 | 08-10 |

### B.8 — `briefs/PROMPT_MASTER_research_build.md` — **13 tasks, one row each** (the one the owner is told to paste)

| ID | Ask | Issued at | O | L | Status | Evidence | Thread | Links | Chk |
|---|---|---|---|---|---|---|---|---|---|
| TRV-0142 | Task 0 — "AUDIT ... Report what exists, as a table ... Launch the build. Capture five before-clips into `research/before/`: (a) walk/sprint/look FP, (b) ADS and fire, (c) every traversal move, (d) throw a grenade, (e) one map lap showing every elevation change. **Gate:** all five clips on disk and committed before proceeding." | PROMPT_MASTER §2 | o | D | DELIVERED | Table: `research/TASK0_AUDIT.md`. Clips: not in `research/before/` but the coverage exists under `handback/brief-vii/` — (a) `baseline/01-02`, (b) `baseline/03-04`, (c) `traversal/01..04`, (d) `grenade_hold/01..07`, (e) `map_lap/01..07`. **`TASK0_AUDIT.md`'s own note that (c) and (e) are missing is now stale** — both landed later. | 13 | corrects TASK0_AUDIT | 08-10 |
| TRV-0143 | Task 1 — "First-person dynamics · slug `fp-dynamics`" — 6 sub-systems, 12 counted sources ≥3 P ≥3 V, plus `SYNTHESIS.md` and a decision table mapping 1:1 to a config file | PROMPT_MASTER Task 1 | o | D | **NOT STARTED** at quota | `research/fp-dynamics/` does not exist. The closest artefact is `research/SOURCES.md` Topic 1, which self-reports **2/16 counted, 0 P, 0 V** and adds "What this changes in `jk_tdm`: nothing — already correct". No `SYNTHESIS.md`. | 15 | superseded-by rule 13 | 08-10 |
| TRV-0144 | Task 2 — "Proper aiming · slug `aiming` ... **Deliverable:** ledger files plus `AIM_SPEC.md` — the full input chain from raw stick or mouse delta to final view rotation, as an ordered pipeline with every stage named." | PROMPT_MASTER Task 2 | o | D | PARTIAL | `research/aiming/SOURCES.md` exists and contains the project's single best research artefact: the Vicencio-Moriera CHI 2014 aim-assist paper, READ end to end, **and the record of catching a fabricated WebFetch summary that invented five numbers and a fifth technique name**. `AIM_SPEC.md` was never written. | 15, 18 | — | 08-10 |
| TRV-0145 | Task 3 — "Traversal: dodge, jump, flips, climbing, obstacles · slug `traversal` ... plus `TRAVERSAL_FSM.md` and `LEDGE_DETECTION.md`" | PROMPT_MASTER Task 3 | o | D | PARTIAL | `research/traversal/SOURCES.md` exists; `research/mech-climb/DESIGN.md` covers hull climbing specifically and **was built** (`7e164d4`). `TRAVERSAL_FSM.md` and `LEDGE_DETECTION.md`: never written. `TASK0_AUDIT.md` row for climb/vault/mantle: "❌ none". | 15, 09 | — | 08-10 |
| TRV-0146 | Task 4 — "Layered character creation · slug `character-creation` ... **The commit-boundary question is the core of this task** ... find at least **two peer-reviewed papers** on avatar customization ... plus `FLOW_SPEC.md`" | PROMPT_MASTER Task 4 | o | D | **NOT STARTED** | `research/character-creation/` does not exist. `research/SOURCES.md` Topic 2 records **0/16, no searches run**, with a stated blocker ("no class system and no per-piece armour system") that is **now false** — both shipped. | 15, 07 | blocker now false | 08-10 |
| TRV-0147 | Task 5 — "Weapon systems: reload, render, console · slug `weapon-systems` ... plus `RELOAD_FSM.md` (with an `ammo commit` column), `RENDER_SPEC.md`, `CONSOLE_SPEC.md` (with the explicit refusal list)" | PROMPT_MASTER Task 5 | o | D | **NOT STARTED** | None of the three files exist. `research/SOURCES.md` Topic 3 has 2/16 counted and a real finding: the S-05 reload-cancel exploit **cannot occur here** because ammo commits at the END of the timer and `switch_slot` grants no ammo. That is a genuine negative result, not a gap. | 15, 23 | — | 08-10 |
| TRV-0148 | Task 6 — "Grenade aiming and physics · slug `grenade-physics` ... plus `GRENADE_SOLVER.md` — the chosen solve mode with its equations, the preview contract, and the determinism guarantees with the integration scheme named" | PROMPT_MASTER Task 6 | o | D | PARTIAL | `research/grenade/{SOURCES,CYCLE_2_REPORT}.md` exist; seed [D] (de Carpentier) is READ. `GRENADE_SOLVER.md` never written. The prompt's **failure condition** "the grenade preview uses a different solver than the throw" is satisfied anyway — one `grenade_tick` serves both, asserted to 0.1 mm across 200 throws. | 15, 02 | — | 08-10 |
| TRV-0149 | Task 7 — "Map and level design · slug `map-design` ... plus `MAP_METRICS.md` — our single authoritative metric table ... and `BLOCKOUT_PROCESS.md`" | PROMPT_MASTER Task 7 | o | D | **NOT STARTED** | `research/maps/SOURCES.md` exists with S-10 (Level Design Book) READ and real cross-engine numbers extracted. Neither deliverable was written. **This is the file TRV-0051 is blocked on.** | 15, 08 | blocks TRV-0051 | 08-10 |
| TRV-0150 | Task 8 — "Powered armour · slug `powered-armour` · **RESEARCH ONLY, DO NOT BUILD** ... Produce the spec, stop there ... End the file with a **build-readiness checklist**." | PROMPT_MASTER Task 8 | o | D | **NOT STARTED** | `research/powered-armour/` does not exist; `POWERED_ARMOUR_SPEC.md` was never written. The "do not build" half was obeyed. The owner said they want this *in the future* — recording it so the want does not vanish. | 22 | — | 08-10 |
| TRV-0151 | Task 9 — "SYNTHESIS ... `research/SYNTHESIS.md` — one document, eight sections, containing only *decisions* ... plus `research/TUNABLES.md`. **Gate:** every conflict has a written resolution." | PROMPT_MASTER §4 | o | D | **NOT STARTED** | Neither file exists. The nine named cross-topic collisions (physique vs viewmodel, ledge bands vs map metrics, i-frames vs blast, etc.) have no written resolution. | 15 | — | 08-10 |
| TRV-0152 | Task 10 — "IMPLEMENT: 10.1 config first · 10.2 map metrics · 10.3 aiming pipeline · 10.4 FP dynamics · 10.5 traversal FSM · 10.6 grenade solver · 10.7 reload FSM · 10.8 character creation flow · 10.9 weapon render · 10.10 console and import" | PROMPT_MASTER §5 | o | S+C | **NOT STARTED** as specified | 10.4 and 10.6 are satisfied by other routes (FP dynamics and the grenade are real and tested). 10.1, 10.2, 10.3, 10.5, 10.7-as-an-FSM, 10.8, 10.9, 10.10 were never built. | 15 | — | 08-10 |
| TRV-0153 | Task 11 — "TESTS. Each test must **fail on the pre-change code** and **pass after**." (a 36-row table: `research_quota`, `deadzone_shape`, `coyote_time`, `input_buffer`, `iframe_window`, `grenade_determinism`, `import_rejects_traversal`, `console_cvar_roundtrip`, …) | PROMPT_MASTER §6 | o | S | **NOT STARTED** | Two rows are satisfied incidentally: `grenade_determinism` (1000 seeded throws, raw-bit) and `preview_matches_throw` (200 throws, sub-mm). The other 34 have no test. | 15 | — | 08-10 |
| TRV-0154 | Task 12 — "CAPTURES. From the build you actually launched." (12 named captures: before/after FP feel, aiming, assist proof, traversal reel, dodge i-frames, grenade bank shot, three reload paths, character creation, physique extremes, weapon render, console session, map lap) | PROMPT_MASTER §7 | o | A | PARTIAL | Map lap: DELIVERED. Traversal: partial (4 stills, not one unbroken take). Grenade: partial (hold captured, bank shot and seed-repeat not). The other eight have no capture, and five of them photograph systems that do not exist. | 13 | — | 08-10 |
| TRV-0155 | Task 13 — "REPORT ... 9 numbered sections ending in **What you did not do, and why. Plainly.**" | PROMPT_MASTER §8 | o | D | **NOT STARTED** | No such report. `handback/brief-ix/REPORT.md` and `handback/REPORT.md` are its nearest relatives and predate this prompt. | 15 | — | 08-10 |
| TRV-0156 | R9 — "**Research is committed.** `research/` is a real directory. Ledgers, notes, extracted numbers and screenshots go there and get committed. Nothing lives only in your context." | PROMPT_MASTER §0 R9 | o | D | DELIVERED | 22 markdown files under `engine/crates/jk_tdm/research/`, all tracked. The single most successful rule in the whole prompt. | 15 | — | 08-10 |
| TRV-0157 | "Work on branch `claude/master-research` ... push with `git push -u origin claude/master-research`. Do not open a pull request unless asked." | PROMPT_MASTER preamble / §8 | o | D | NOT STARTED | No such branch. All work landed on `main`. Recording the divergence, not litigating it. | 15 | — | 08-10 |

### B.9 — `briefs/PROMPT_brief_X_research.md` — ***Superseded* by the master prompt, "which absorbs all three of its topics"** (`briefs/README.md:25`)

Superseded ≠ void. Indexed, with the superseder named on each row.

| ID | Ask | Issued at | O | L | Status | Evidence | Thread | Links | Chk |
|---|---|---|---|---|---|---|---|---|---|
| TRV-0158 | Task 0 — "AUDIT WHAT EXISTS ... capture 3 before-clips into `research/before/`" | PROMPT_brief_X §2 | o | D | SUPERSEDED | — | 15 | superseded-by TRV-0142 | 08-10 |
| TRV-0159 | Task 1 — "RESEARCH FIRST-PERSON DYNAMICS ... 16 counted, ≥4 P, ≥4 V ... `NUMBERS.md` has ≥25 rows" | PROMPT_brief_X §3 | o | D | SUPERSEDED | 2/16 at retirement | 15 | superseded-by TRV-0143 | 08-10 |
| TRV-0160 | Task 2 — "RESEARCH LAYERED CHARACTER CREATION ... at least **two peer-reviewed papers** ... `FLOW_SPEC.md`" | PROMPT_brief_X §4 | o | D | SUPERSEDED | 0/16 at retirement | 15 | superseded-by TRV-0146 | 08-10 |
| TRV-0161 | Task 3A — "Reload as a state machine: `IDLE → INITIATE → MAG_RELEASE → MAG_DROP → MAG_INSERT → SEAT → CHARGE/BOLT → RECOVER → IDLE`, with the interrupt boundary and the ammo-commit frame" | PROMPT_brief_X §5.1 | o | S | SUPERSEDED | The audit result stands and is worth keeping: this codebase commits ammo at the END and cancels rather than pauses — a stated design choice, not a bug | 15 | superseded-by TRV-0147 | 08-10 |
| TRV-0162 | Task 3B — "Weapon rendering ('gun pixel dynamics'): material layering, wear/edge damage, decal projection for a player-supplied image, viewmodel depth pass, texture budget" | PROMPT_brief_X §5.2 | o | C | SUPERSEDED | Blocked upstream on THREAD 14 in any case | 15, 14 | superseded-by TRV-0147 | 08-10 |
| TRV-0163 | Task 3C — "In-game console and runtime image import ... **What is refused:** anything outside the whitelist, oversize, or resolving outside the import directory. Refusals print a reason." | PROMPT_brief_X §5.3 | o | C | SUPERSEDED | `TASK0_AUDIT.md`: "Console / command system — ❌ none". The refusal-list instinct is recorded as correct in `research/SOURCES.md`. | 23 | superseded-by TRV-0147 | 08-10 |
| TRV-0164 | Task 4 — "SYNTHESIS AND CONFLICT RESOLUTION ... `TUNABLES.md`" | PROMPT_brief_X §6 | o | D | SUPERSEDED | — | 15 | superseded-by TRV-0151 | 08-10 |
| TRV-0165 | Tasks 5-7 — implement, tests (21 rows), captures (7 items) | PROMPT_brief_X §7-9 | o | S+C | SUPERSEDED | — | 15 | superseded-by TRV-0152..0154 | 08-10 |
| TRV-0166 | Task 8 — "REPORT ... **What you did not do, and why.** State them plainly (R10)." | PROMPT_brief_X §10 | o | D | SUPERSEDED | — | 15 | superseded-by TRV-0155 | 08-10 |

### B.10 — `briefs/PROMPT_mech_rebuild.md` — ***Superseded,* kept for the mech-specific detail not carried into the master prompt** (`briefs/README.md:24`)

| ID | Ask | Issued at | O | L | Status | Evidence | Thread | Links | Chk |
|---|---|---|---|---|---|---|---|---|---|
| TRV-0167 | Task 0 — "AUDIT AND FIX DELIVERY (blocks everything else) ... **How many segments does the current character rig have, and is the trunk one bone or several?**" | PROMPT_mech_rebuild Task 0 | o | D | DELIVERED | `handback/REPORT.md`; `c6f55af` audited and **empirically disproved** the single-trunk-bone premise (separation was already ~42%) — a spec premise proven wrong, which OPERATION rule 11 says is worth more than a tick | 17 | — | 08-10 |
| TRV-0168 | Task 1 — "GATHER AND COMMIT VISUAL REFERENCE ... Save 12-20 mech images and 8-12 body/hand images into `handback/reference/` and commit it — **reference that lives only in a chat log is lost work**." | PROMPT_mech_rebuild Task 1 | o | A | **NOT STARTED / BLOCKED** | `handback/reference/NOTES.md` exists and is honest: *"there is no image-download capability... that specific deliverable is **not achievable** with the tools available this session."* Zero images. The design principles were captured instead and did drive Task 5. | 24 | duplicate-of TRV-0070, 0083 | 08-10 |
| TRV-0169 | Task 2 — "REBUILD THE BODY AS 20 SEGMENTS ★ do this first, it unblocks the rest ... **Four features are blocked on one missing bone.**" | PROMPT_mech_rebuild Task 2 | o | C | DELIVERED | `787f6ff`, `1e774e1`, `07e48ad`; the separation test run both ways as the prompt demanded | 17 | duplicate-of TRV-0102 | 08-10 |
| TRV-0170 | Task 3 — "THE ELASTIC LOAD MODEL ('Achilles motion', literally)" | PROMPT_mech_rebuild Task 3 | o | S | DELIVERED | `c6f55af` | 17 | duplicate-of TRV-0108 | 08-10 |
| TRV-0171 | Task 4 — "MECH SCALE DECISION (answer before rebuilding). Pick one, write it into `config/mech.ron`, and state the choice in your report." | PROMPT_mech_rebuild Task 4 | o | S | DELIVERED | A3 = 1.7× taken and stated (`c6f55af`, `handback/REPORT.md`); `8f77f01` scaled the step-up with the chassis because the addendum required it | 01 | duplicate-of TRV-0101 | 08-10 |
| TRV-0172 | Task 5 — "REBUILD THE MECH FROM THE ART — the 20 parts, the olive-drab palette, damage states mapped to real parts. **Capture — this is the completion criterion:** render the mech beside a 1.8 m soldier at the art's ¾ angle, next to the concept art." | PROMPT_mech_rebuild Task 5 | o | C | PARTIAL / BLOCKED | Rebuilt repeatedly (`939df7c`, `f0b6b46`, `aa14bcf`, `40acc69`, `aedc8db`). The **soldier-beside-machine** half is now delivered by `mech_lineup.rs` — `Chassis::Soldier` exists precisely as "the RULER the row is measured against" — captured in `mech_gallery/01`, `05`. The **concept-art** half is unsatisfiable. | 04, 24 | depends-on TRV-0260 | 08-10 |
| TRV-0173 | Task 6 — "REPORT ... including 'Anything not done, named plainly, with the reason.'" | PROMPT_mech_rebuild Task 6 | o | D | DELIVERED | `handback/REPORT.md` — "honest deferral list (weapon swap, 20-part mesh rebuild, mech stance/geometry, camera framing still imperfect at new scale, config externalization)" | 13 | — | 08-10 |

### B.11 — `briefs/PROMPT_RND_CYCLE.md` — the repeatable R&D cycle and its Appendix A backlog

`BACKLOG.md` is the live descendant of Appendix A. **It is HISTORICAL and
several entries are known false** — indexed here, never ranked from.

| ID | Ask | Issued at | O | L | Status | Evidence | Thread | Links | Chk |
|---|---|---|---|---|---|---|---|---|---|
| TRV-0174 | Appendix A Critical #1 — "**Mech entry sequence** — approach, identification, cockpit open, climb-in, harness, power-up, servo sync, gyro calibration, weapon diagnostics, HUD boot, camera transition. Turns entering the mech from a state toggle into an *event*." | PROMPT_RND_CYCLE App. A | o | S+C | PARTIAL | Sim half DELIVERED (`research/mech-entry/CYCLE_1_REPORT.md`, `b116a46`). Presentation split out as `BACKLOG.md` #16 and is still 7-of-8 invisible. | 05 | duplicate-of TRV-0037 | 08-10 |
| TRV-0175 | Appendix A Critical #3 — "**Grenade surface interaction** — per-material restitution and friction, rolling, spin, angular momentum" | PROMPT_RND_CYCLE App. A | o | S | DELIVERED | `research/grenade/CYCLE_2_REPORT.md`, `a3b89aa`; spin split out as #17 | 02 | — | 08-10 |
| TRV-0176 | Appendix A Critical #2 — "**Infantry vs. giant mech** — climbing the hull, joint strikes, hydraulic failure, cable cutting, sensor destruction, weak-point exposure, coordinated squad attacks" | PROMPT_RND_CYCLE App. A | o | S+C | DELIVERED (partial scope) | Design `research/mech-climb/DESIGN.md` (Cycle 3), build `7e164d4` — zone grabs on stripped plates, position parenting, asymmetric grip drain, involuntary detach, 1.6× climbing strike on both melee paths, 4 tests, plus the grip HUD and prompt (`f69e7a9`). Joint strikes / hydraulic failure / cable cutting: not built. | 09 | — | 08-10 |
| TRV-0177 | `BACKLOG.md` #4 — "Melee depth: parry, deflection, directional attack, stagger, armour penetration, weak points" | BACKLOG.md:19 | o | S | DELIVERED | **`BACKLOG.md` says "Not started" — known false.** Melee v2 shipped `a99af96`: strafe picks the swing's LINE (left/right/overhead), latched at the wind; a parry only meets a blade on the same line; both viewmodel and third-person silhouettes cock to the chosen line. Parry + `PARRY_STAGGER_S` in `71d27a5`. Captures `melee_dirs/01..03.png`. Test `sim.rs:15276`. | 19 | corrects BACKLOG #4 | 08-10 |
| TRV-0178 | `BACKLOG.md` #5 — "AI squad coordination: flanking, suppression, bounding overwatch, retreat" | BACKLOG.md:20 | o | S | DELIVERED | **`BACKLOG.md` says RETREAT "is the remainder" — stale.** Retreat shipped `e5431a4`: `fear` from witnessed falls, suppression, being outnumbered and your own blood; hysteretic threshold; `ROUT_TOLERANCE` from CLASS; a broken man holds fire, keeps facing the enemy, and reloads; the player is never routed. `sim.rs:2954`, `:13367`, `:13399`; test `:17320`. | 20 | corrects BACKLOG #5 | 08-10 |
| TRV-0179 | `BACKLOG.md` #6 — "Mech operation feel: weight, mechanical inertia, cockpit vibration, heat, internal damage, emergency shutdown/eject" | BACKLOG.md:21 | o | S+C | PARTIAL | Weight/inertia (hull spring k=22), cockpit (`cockpit.rs`, 16 captures), heat (`stride_heat`, gatling heat, plasma lockout) all DELIVERED. Internal damage and emergency shutdown/eject: NOT STARTED. | 05 | — | 08-10 |
| TRV-0180 | `BACKLOG.md` #7 — "Traversal: climb, vault, mantle, ledge bands" | BACKLOG.md:22 | o | S | BLOCKED | Unblocker, named: `MAP_METRICS.md`. Hull climbing proved the attach/parent/stamina mechanic but is mech-specific. | 09 | depends-on TRV-0149 | 08-10 |
| TRV-0181 | `BACKLOG.md` #9 — "Character creation layers (L0-L4)" | BACKLOG.md:29 | o | C | NOT STARTED | **The stated blocker — "no class system and only 5 whole-body armour presets" — is known false.** Four classes shipped 2026-08-05; 24 per-piece plates shipped 2026-08-07. The row is unblocked and nobody has noticed. | 07 | corrects BACKLOG #9 | 08-10 |
| TRV-0182 | `BACKLOG.md` #10 — "Destruction and environmental interaction" | BACKLOG.md:30 | o | S | NOT STARTED | "rapier supports some; needs a design reason before a technique" — that reason still does not exist. Correctly parked. | 08 | — | 08-10 |
| TRV-0183 | `BACKLOG.md` #11 — "Injury, fatigue, equipment weight, dynamic centre of gravity" | BACKLOG.md:31 | o | S | PARTIAL | **The stated blocker — "the armour-weight formula exists but is unwired" — is known false.** It was wired in the 24-plate pass, with a live weight-against-ceiling readout in the Forge. Injury, fatigue and dynamic CoG remain unbuilt. | 06 | corrects BACKLOG #11 | 08-10 |
| TRV-0184 | `BACKLOG.md` #17 — "Grenade rotational dynamics — real spin/angular momentum affecting bounce direction" | BACKLOG.md:47 | a | S | NOT STARTED | **Deliberate deferral with a stated reason**: real rotational dynamics in a fixed-timestep replay-critical integrator is a much larger determinism risk than the tangential-friction win already banked. A decision, not a gap. | 02 | — | 08-10 |
| TRV-0185 | `BACKLOG.md` #18 — "Additional grenade surface materials — mud, sand, snow, ice, water" | BACKLOG.md:48 | a | S | BLOCKED | Unblocker, named: any map adding one of these as real cover geometry. | 21 | — | 08-10 |
| TRV-0186 | `BACKLOG.md` #24 — "Per-piece armour GEOMETRY — the soldier's mesh is unchanged when you strip a gauntlet" | BACKLOG.md:54 | a | C | DELIVERED | **Corrected by the gap scout, 2026-08-08:** "the 24 plate groups exist and visibility follows the loadout. The backlog's 'stripping a gauntlet changes nothing' is no longer true." | 06 | corrects BACKLOG #24 | 08-10 |
| TRV-0187 | §5 — "ROTATING CODEBASE REVIEW. Each cycle, review **four** categories, rotating through this list so each is examined roughly every five cycles." | PROMPT_RND_CYCLE §5 | o | D | NOT STARTED | No cycle records a four-category rotation. The scouts (`scout-defect`, `scout-gap`, `scout-map`) do the equivalent work ad hoc and find real things, which is arguably better — but the rotation itself was never run. | 15 | — | 08-10 |

### B.12 — `briefs/PROMPT_motion_system_research.md` (research-only; ends in a decision, not a survey)

| ID | Ask | Issued at | O | L | Status | Evidence | Thread | Links | Chk |
|---|---|---|---|---|---|---|---|---|---|
| TRV-0188 | §4 — "`research/motion-architecture/DECISION.md`: the recommended architecture concretely (player / nearby / crowd, and the LOD handoff), every rejected alternative with the axis that killed it, the nine-axis scoring table, a migration path, cost, determinism verdict, extend/replace/leave-alone per existing system, and the honest risk." | PROMPT_motion §4 | o | D | PARTIAL | `research/motion-architecture/{SOURCES,NOTES}.md` exist; `e69e454` records **session 1, 5/14 core sources READ**. `DECISION.md` — the entire deliverable — was never written. **The largest orphaned research in the repo.** | 16 | — | 08-10 |
| TRV-0189 | R3 — "**Licence before recommendation.** No repository, dataset, or model may be *recommended* until its licence is recorded verbatim ... **Known trap:** LaFAN1 is CC BY-NC-ND 4.0." | PROMPT_motion §0 R3 | o | D | PARTIAL | The LaFAN1 trap is recorded and remains the single durable output of this thread. No crate was evaluated to the standard the rule demands (`bevy_animation_graph`, `bevy_motion_matching`, `bevy_mod_inverse_kinematics` — Bevy-version compatibility never checked). | 16 | — | 08-10 |
| TRV-0190 | §5 — the eight tests (`source_quota`, `every_repo_has_licence`, `no_noncommercial_recommended`, `bevy_version_recorded`, `determinism_classified`, `rejections_have_reasons`, …) | PROMPT_motion §5 | o | D | NOT STARTED | None exist. | 16 | depends-on TRV-0188 | 08-10 |

### B.13 — CHAT ASKS RECORDED SECOND-HAND

**These are the ones that vanish.** None of them is in a brief. Every one
survives only because somebody wrote it next to the code, in a commit
message, or inside a log. The project's `§owner` doc-comment convention is
the single best archival practice in this repo and it is why this section
can exist at all.

| ID | Ask | Issued at | O | L | Status | Evidence | Thread | Links | Chk |
|---|---|---|---|---|---|---|---|---|---|
| TRV-0191 | *"I want you to have 2 other mechs models also displayed both in ally and enemy in training mode, this will help me visually describe the player."* | quoted verbatim in `mech_lineup.rs:6-8` | o | C | DELIVERED | `mech_lineup.rs` (whole module); captures `mech_gallery/01-gallery-wide-third.png`, `02-gallery-wide-fp.png`, `03-ally-section.png`, `04-enemy-section.png`, `05-gallery-quarter.png`; commits `77b9805`, `2c2f863`, `1904bc7` | 01 | — | 08-10 |
| TRV-0192 | *"have mechs displayed in training mode please in their original sizes"* | quoted verbatim in `mech_lineup.rs:20-21` | o | C | DELIVERED | `mech_lineup.rs:127-160` `chassis_scale` reads the sim's own `ArmorSet::chassis_scale` rather than restating it; `Chassis::Soldier` added as the ruler; test `mech_lineup.rs:1516`; commit `fe07c19` "a chassis scale the client kept a copy of" | 01 | — | 08-10 |
| TRV-0193 | *"increase the recoil"* (the heavy mech's minigun turret) | quoted in `FRIDAY_LOG.md:1330` as `§owner`; constant doc at `sim.rs:5307`, `:5313` | o | S | DELIVERED | Measured first: one round of SINGLE moved the camera by **exactly 0.0000 degrees** before the change. `TURRET_FELT_FLOOR = 24.0` (`sim.rs:5367`), `TURRET_AIM_STABILISER = 0.25`. Felt kick 8.29 → 24.00 °/s; AUTO plateau 4.09 → 16.22°; bot damage +5.4%. Tests `sim.rs:22766`, `:22828`. **No capture.** | 03 | supersedes TRV-0201 (turret only) | 08-10 |
| TRV-0194 | *"make enemy mechs colour dark blue and [light blue lines]"* | quoted in `main.rs:3675` as `§owner BLUE ENEMY MECHS` | o | C | DELIVERED | `main.rs:3675`, `:3706`, `:14361`, `:14421`, `:14527`, `:14577`; captures `mech_gallery/04-enemy-section.png`, `06-enemy-heavy-close.png`, `07-enemy-scout-close.png`; commit `2c2f863` | 04 | — | 08-10 |
| TRV-0195 | *"the first 1-2-3 shots go straight, like a normal gun"* | quoted in `sim.rs:15163` (the test's own doc) | o | S | DELIVERED | `sim.rs:4453` (how many rounds at the start of a burst leave essentially straight); test `sim.rs:15163` | 03 | — | 08-10 |
| TRV-0196 | *"put more effort in last 3 maps"* | quoted in `main.rs:16567` as `§owner` | o | C | PARTIAL | The quote survives against a capture-script change. The map content work it names is TRV-0041 and is NOT STARTED. **This is a good example of an ask being recorded next to the wrong thing** — it is filed against a camera beat, not against the maps. | 08 | see TRV-0041 | 08-10 |
| TRV-0197 | *"what you see is what you get"* — with flight coming, a roof you can see is a roof you can reach | `sim.rs:2001` as `§owner` | o | S | DELIVERED then orphaned | Delivered on Cliffhold — `every_cliffhold_band_is_reachable_on_foot` walks eight routes to prove it. Cliffhold's client half was then deleted, so the principle now lives only in a map you cannot select. | 08 | see TRV-0039 | 08-10 |
| TRV-0198 | `§owner TEAM IDENTITY` — ally = white-and-gold over clean mid grey; enemy moved **down** from dark oxide red to near-black iron with a red cast; the two factions must differ in their FRAMES, not only their accents | `branding.rs:96`, `:107`, `:122`, `:150` | o | C | DELIVERED | `branding.rs` (whole module); `Side::steel()` and `Side::accent()` keep it as DATA; verified by Thor's luminance arithmetic (13.4× and 24×) | 04 | supersedes TRV-0111 | 08-10 |
| TRV-0199 | *"the war bow is held UPRIGHT, and it CURVES"* / `§owner THE BOW STANDS UP AGAIN` | `main.rs:371`, `:2985`, `:8612` | o | C | DELIVERED | `a890766` "The bow stands up: §6, and two instruments that could not see a cant" — and it exposed two instruments welded to the old decision (`bow_string_half` could only aim in one plane; the screen-intrusion sweep discarded `vm_carry`'s ROLL). Captures `bow_draw_fp/01..05.png`. | 19 | — | 08-10 |
| TRV-0200 | `§owner`: *"a raised shield is NOT an answer to war projectiles. An arrow slips past its edge entirely; a thrown spear punches through"* | `sim.rs:289-291` | o | S | DELIVERED | `7d9f42a` "war projectiles beat shields"; test `sim.rs:18908` (three hits of either war projectile fell a shield-bearer) | 19 | — | 08-10 |
| TRV-0201 | *"Recoil HALVED across the arsenal (owner request)"* | `README.md:209-210` (v5 section) | o | S | UNVERIFIED / partly SUPERSEDED | For the mech turret: superseded by TRV-0193, which raised it 2.9×. For the infantry arsenal: I did not re-derive whether the halving is still in force. | 03 | superseded-by TRV-0193 (turret) | 08-10 |
| TRV-0202 | *"four owner-decided rules locked in: baseline 2 headshots / 8 body shots; AWM = only the head is instant; 'check back' = capturable respawn checkpoints; battles cap at 8v8."* | `README.md:223-225` (v6 section) | o | S | PARTIAL | First three DELIVERED and tested (`main.rs:24291-24296` derives the damage line from the live constants). **8v8 was later withdrawn from the menu** (`main.rs:23200`, `ab4bf55`) — an owner rule overturned by a later decision, which is exactly the kind of thing that needs a row. | 11 | superseded-by TRV-0212 | 08-10 |
| TRV-0203 | *"dont be doing reserch try to be more built and friday orienatted"* and *"cancel research but also you can tell them to become scouts to help you build"* | quoted verbatim in `OPERATION.md:262-263`, commit `cf51f19` | o | D | DELIVERED | OPERATION.md **rule 13**, with the measured evidence table behind it (two toto dispatches ≈398k tokens producing knowledge that mostly did not survive a builder's judgement; two scout sweeps ≈277k tokens producing findings that shipped as fixes) | 15 | supersedes THREAD 15 | 08-10 |
| TRV-0204 | *"several tasks needed 3+ iterations purely on camera framing"* | quoted in `THOR_LOG.md:2829` | o | C | NOT STARTED | The fix is TRV-0040. Three iterations × 6 min = 18 minutes of pure rebuild to move a camera, and it recurs. | 13 | duplicate-of TRV-0040 | 08-10 |
| TRV-0205 | The owner's luminance question — *are the mech liveries still separable at range?* | `THOR_LOG.md:2700`, `:2712` | o | C | DELIVERED (answered) | Thor computed it in linear relative luminance from the live material values: ally 0.239 vs enemy 0.0178 = **13.4×** (ΔL* ≈ 42); scout 0.380 vs 0.0158 = **24×**. Answered, in arithmetic, not opinion. | 04 | spawns TRV-0206 | 08-10 |
| TRV-0206 | (Thor) "the stated rule is now false, and **nothing tests it**" — ally/enemy separation rests on luminance, not hue, and the enemy's light blue is already brighter than the ally's brightest tone | `THOR_LOG.md:2698`; restated as WHATS_MISSING trap 4 | a→o | C | **NOT STARTED** | No test pins ally/enemy luminance separation. SPEC15's own trap list carries it: "The luminance rule is unguarded... Neon on dark is safe; neon over large areas is not." | 04 | — | 08-10 |
| TRV-0207 | *"the owner asked for the original back: just the blue transparent hologram, at 60% bigger"* (the mech barrier, after a pass bolted an emitter ring and feed conduits around it) | `WHATS_MISSING.md:511` | o | C | DELIVERED | `BARRIER_SCALE = 1.60` against the original 1.7 m field = 2.72 m, mid-band of the file's own 2.4-3.4 m bracket; all the added hardware removed; `main.rs:9589`, `:9602`, `:10176`; captures `barrier/01..04.png`; commit `6442a2d` | 04 | evidence caveated by TRV-0035 | 08-10 |
| TRV-0208 | *"the owner brought reference art: a squat utility robot, rounded masses, one big camera lens, worn amber over near-black"* | `WHATS_MISSING.md:513` | o | C | DELIVERED (build) / BLOCKED (the art) | Built to it: ~90 exposed struts/pistons/louvres reduced to ~45 nameable masses; one LENS eye with the team accent as its iris at visor height; plantigrade thick legs; amber ally shell. Captures `medic/01..09.png`; commit `72a93f3`. **The reference image is not in the repo**, so nothing can be re-checked against it. | 24 | depends-on TRV-0261 | 08-10 |
| TRV-0209 | `§owner`: *"the standing CLASS pick — who you are for the match"* / *"the FOUR CLASSES - a standing choice made in the Forge, not a spawn roll"* | `main.rs:1351`, `sim.rs:2239`, `:3207` | o | S | DELIVERED | `2af8a01`; test `sim.rs:15058` (each class must actually trade) | 07 | — | 08-10 |
| TRV-0210 | `§owner`: *"kills to win a TDM - a quick 30 or a long 60"* | `main.rs:1353`, `sim.rs:3048` | o | C | DELIVERED (with a defect) | Selectable per match (`ab4bf55`). The Field Manual still prints the default — TRV-0031. | 11 | spawns TRV-0031 | 08-10 |
| TRV-0211 | `§owner`: *"ZOMBIE EXTRACTION is withdrawn from the menu"* | `main.rs:23214` | o | C | DELIVERED | Deliberate and documented at the site. Confirmed by the gap scout as NOT a gap. | 11 | — | 08-10 |
| TRV-0212 | `§owner`: *"8v8 withdrawn"* | `main.rs:23200` | o | C | DELIVERED | Deliberate; the battle-size row keeps its place. Overturns TRV-0202's fourth rule. | 11 | supersedes TRV-0202 (part) | 08-10 |
| TRV-0213 | `§owner`: *"every firearm carries a 1x red-dot optic, and the [illuminated centre]"* — and the earlier note that a CROSS of two long bars "read badly" | `main.rs:4233`, `:4274`, `:16133`, `:25014` | o | C | DELIVERED | `419e52d`; two guns had NO rear sight at all and five declared the front post as their sight line — all fixed, then superseded by the red dot. Captures `sights_a|b|c/01..04.png` (12 frames). | 19 | — | 08-10 |
| TRV-0214 | `§owner`: *"AIMED, THE WEAPON DOES NOT MOVE. Not 'moves less' - does [not move]."* | `main.rs:20371` | o | C | DELIVERED | `9b92808` "the aimed gun stops moving" | 19 | — | 08-10 |
| TRV-0215 | `§owner`: *"focused = STILL. Sway and breathe all but stop"* / *"focus ALIGNS THE SIGHTS"* | `main.rs:20242`, `:20420` | o | C | DELIVERED | `7d9f42a` "Focus means STILL"; `995c655` "focus reaches every gun" | 19 | — | 08-10 |
| TRV-0216 | `§owner`: *"a JAVELIN, not a broomstick with a wedge on it"* | `main.rs:8678` | o | C | DELIVERED | `8933748`, `c5a6d3b`; leaf blade, collar, haft, butt spike | 19 | — | 08-10 |
| TRV-0217 | `§owner`: *"the minigun had NO sights of any kind"* | `main.rs:8736` | o | C | DELIVERED | `419e52d` | 19 | — | 08-10 |
| TRV-0218 | `§owner`: *"IT HAS TO READ AS A MINIGUN"* | `main.rs:10289` | o | C | DELIVERED | `53be066` "The turret is a minigun again, and the gallery tells the truth about size" — and the defect behind it is OPERATION rule 8's own example: a barrel shroud written as "a half-cowl, open below" was built from cylinders facing down the barrel axis, i.e. **discs**, which capped the gun and hid all six barrels. Captures `minigun_check/01..05.png`. | 19 | — | 08-10 |
| TRV-0219 | `§owner MAP EXPANSION`: every map 25% bigger, applied centrally | `sim.rs:25`, `:1509` | o | S | DELIVERED | `46851ec` — POSITIONS scale, extents and heights do not, plus an infill pass because a bigger map is a worse map empty | 08 | trap recorded at `FRIDAY_LOG` root | 08-10 |
| TRV-0220 | `§owner`: *"the four classes must actually TRADE - each one better [at something]"* | `sim.rs:15058` | o | S | DELIVERED | LINE is 1.0 across the board; a test proves the other three each trade | 07 | — | 08-10 |
| TRV-0221 | `§owner`: *"the RANGE's promise is 'nothing shoots back'. A training mode where you can still be killed mid-lesson is not a training [mode]"* | `sim.rs:15234-15239` | o | S | DELIVERED | Test `the_training_range_never_shoots_back` | 01 | — | 08-10 |
| TRV-0222 | `§owner`: *"ten tubes per chassis (was 4), and ammo pads now resupply"* | `sim.rs:4274` | o | S | DELIVERED | `d845250` "a real ammo economy" | 01 | — | 08-10 |
| TRV-0223 | `§owner`: *"the mech's FRONT is winnable now. Ten AWM body shots or [...]"* | `sim.rs:18834` | o | S | DELIVERED | Test at the cited line; rebalance recorded at `sim.rs:5875` | 01 | — | 08-10 |
| TRV-0224 | `§owner TEXTURE PIPELINE` | `main.rs:3333`, `:14160`, `:25101` | o | C | PARTIAL | Procedural generators + tangents (`07f923d`, `52684be`). No imported image reaches a world material. | 14 | duplicate-of TRV-0049 | 08-10 |
| TRV-0225 | `§owner GUN PASS`: the shared surface vocabulary — slide serrations, ports, grips, magazines, handguard slots, receiver seams, rail slots, takedown pins, dust-cover seams, tube bands, bolt handles, cheek risers, feed trays, motor ribs, heat shrouds | `main.rs:3906`, `:8415-8740` | o | C | DELIVERED | `c19ca92` "Give every gun a machined surface" | 19 | — | 08-10 |
| TRV-0226 | `§owner ARM PASS`: the articulation | `main.rs:12092` | o | C | DELIVERED | `9eee7de` "Heavy mech: arms that could move" — pauldron→elbow→cradle chain replacing bare hardpoints | 04 | — | 08-10 |
| TRV-0227 | `§owner FLAGSHIP PASS`: the turret and the launcher — *"make the mounts look like weapons a flagship would carry"* | `main.rs:10701`, `:10988` | o | C | DELIVERED | `aa14bcf`; spinner-mounted clamps, muzzle collars, a STATIC drive housing, heat-sink stack with a glowing seam, belt box with gold feed links | 04 | — | 08-10 |
| TRV-0228 | `§owner MECH REFIT`: *"the hull is built at full size and worn at 85%"*, plus the DENSITY pass and the LEG density pass | `main.rs:941`, `:11687`, `:12554` | o | C | DELIVERED | `f0b6b46` — uniform root scale because the hull is full of rotated cylinders and a non-uniform parent scale shears every one; ~120 new parts buy the size back as MASS | 04 | — | 08-10 |
| TRV-0229 | `§owner SHIELD PASS`: *"it has to actually COVER the machine"* | `main.rs:26929` | o | C | DELIVERED | Test at the cited line | 04 | — | 08-10 |
| TRV-0230 | `§owner MECH SENSOR OVERLAY`: the optic housing, the target box, the two bars that open away from the crosshair as a precision charge builds | `main.rs:4662`, `:4667`, `:4673`, `:9782`, `:14935` | o | C | DELIVERED | `cd24c4c` "a bracket that finds the target"; captures `medic/08-precision-charging.png` | 11 | — | 08-10 |
| TRV-0231 | `§owner MELEE v2`: *"the swing has to SHOW its line, or the [defender cannot answer it]"* | `main.rs:18357`, `:20488`; `sim.rs:6436`, `:3325` | o | S+C | DELIVERED | `a99af96`; captures `melee_dirs/01-left-cock.png`, `02-right-cock.png`, `03-overhead-cock.png` | 19 | duplicate-of TRV-0177 | 08-10 |
| TRV-0232 | `§owner SUPPRESSION`: the player-facing half — *"the mechanic shipped with [no read]"* | `main.rs:20568`, `:22533`; `sim.rs:15448` | o | C | DELIVERED | `e5431a4` — the screen edge facing the last close round lights pale gold, reusing the DAMAGE flash's own strips (red 0.55 alpha, gold 0.25, so a hit wins on alpha alone with no priority rule) | 20 | — | 08-10 |
| TRV-0233 | `§owner BOT ROUTING`: *"the one rule that tells terrain from furniture"* | `sim.rs:168`, `:1009`, `:6776`, `:13608`, `:27338-27700` | o | S | DELIVERED (scoped) | `BOT_PROBE_Y = 0.75` extracted from a literal, value unchanged; every flight publishes itself as a link; five tests. `sim.rs:27649` states the flat maps were deliberately left alone. | 09 | see TRV-0043 | 08-10 |
| TRV-0234 | `§owner CLIFFHOLD`: *"a castle on a cliff over half a city, built for fliers"* — 600×600 m, every altitude band reachable on foot | `sim.rs:949`, `:1690`; root `FRIDAY_LOG.md` | o | S+C | SUPERSEDED | Built (`830446e`, `477be34`), then the owner's plan said "Delete Cliffhold" and the client half went (`4152240`). The sim half and its five tests remain. **A whole map's worth of owner want, half-retracted, half-alive.** | 08 | superseded-by TRV-0039 | 08-10 |

### B.14 — AGENT-ORIGIN ROWS (`origin: agent`)

Thor's findings and Friday's stated deferrals belong in the bank too, but
they never outrank an owner row at equal severity.

| ID | Ask / finding | Issued at | O | L | Status | Evidence | Thread | Links | Chk |
|---|---|---|---|---|---|---|---|---|---|
| TRV-0235 | (Friday, deferred) "plate wear fires only where plate PROTECTION already fires (the zoned hitscan path), so grenades, melee, claws and gas still neither wear plate nor are reduced by it" | FRIDAY_LOG.md:85-87 | a | S | NOT STARTED | Stated deferral with one gate (`in_mech`) for both, so they cannot drift. A deliberate limit, recorded. | 06 | — | 08-10 |
| TRV-0236 | (Friday, deferred) "`armor_spec`'s flats are unreachable for a piloted HEAVY chassis... **Worth someone deciding on deliberately.**" | FRIDAY_LOG.md:1466-1470 | a | S | BLOCKED | Unblocker, named: a decision on whether the flats should be reachable or removed. True of the Big Mech's row too, and long-standing. | 01 | — | 08-10 |
| TRV-0237 | (Friday, deferred) "`mech_visor_eye_y(pos_y)`, the free function, still hard-codes `MECH_SCALE` and so is wrong for a Royal. Every client caller needs moving — that is Friday33's, and I did not reach across." | FRIDAY_LOG.md:1475-1478 | a | C | UNVERIFIED | Commit `fe07c19` is titled "a chassis scale the client kept a copy of" and may have closed exactly this. I did not re-derive it. | 01 | — | 08-10 |
| TRV-0238 | (Friday, deferred) "`mount_kick_axes` and `turret_chaos_of` are published for the visual half; **nothing reads them yet**." | FRIDAY_LOG.md:1479-1482 | a | C | DELIVERED | **Moved since it was written.** `mech_recoil.rs:97-119` `SWING_RAD_PER_DEG` now tracks `punch[1]`, landed in `fe07c19`. | 03 | closes a Friday deferral | 08-10 |
| TRV-0239 | (Friday, least sure) "A bot mech's sustained output at 10 m fell about a third (492 → 331 damage over 17 s)... a third is a real balance change, not a rounding error." | FRIDAY_LOG.md:1454-1460 | a | S | BLOCKED | Unblocker, named: **an owner playtest.** The direction is intended; the magnitude is not something a builder should ratify alone. | 03 | depends-on owner | 08-10 |
| TRV-0240 | (Friday, least sure) "Braced fire is no longer perfectly accurate... it is a nerf nobody asked for." | FRIDAY_LOG.md:1389-1394 | a | S | BLOCKED | Same unblocker. Player-only (bots never brace with the gatling). | 03 | depends-on owner | 08-10 |
| TRV-0241 | (Friday, least sure) "The player/bot asymmetry in `punched_aim_stabilised` is untouched. **It is the real root** and it applies to every recoiling weapon in the game." | FRIDAY_LOG.md:1409-1412 | a | S | NOT STARTED | Correctly scoped out of a commit titled "the turret kicks harder". Still the root. | 03 | — | 08-10 |
| TRV-0242 | (Friday, least sure) "A bot-piloted chassis now **never raises its barrier at all.**" | FRIDAY_LOG.md:1405-1408 | a | S | NOT STARTED | The honest consequence of fixing the reload gate. Barrier discipline for a bot mech is a new behaviour with its own rule, and the builder declined to invent it. | 20 | — | 08-10 |
| TRV-0243 | (Friday, least sure) "no capture script has a beat where the player gets a kill, so I have **no PNG of the X**... **This is the single thing I would most want checked.**" | FRIDAY_LOG.md:1279-1286 | a | C | NOT STARTED | If Bevy UI silently no-ops `Transform.rotation` on a UI node, the kill confirm degrades to a colour flash and nothing warns you. | 13 | depends-on TRV-0040 | 08-10 |
| TRV-0244 | (Friday, least sure) "`crosshair_render` itself is untested... A swapped arm index or an inverted `shown` term would pass the whole suite." | FRIDAY_LOG.md:1287-1293 | a | C | NOT STARTED | The capture is the only thing standing behind it, and it exercises two configurations. | 11 | — | 08-10 |
| TRV-0245 | (Friday, least sure) "**no test covers `stability_bracket`**, so if I had fat-fingered the number nothing would have caught it." | FRIDAY_LOG.md:1301-1306 | a | C | NOT STARTED | — | 11 | — | 08-10 |
| TRV-0246 | (Friday, least sure) "The dynamic crosshair's px-per-radian is inherited, not derived. 2400 came from the stability bracket, where it was tuned against a bracket glyph, not against arm travel... the number is ASSUMED, not measured." | FRIDAY_LOG.md:1307-1311 | a | C | NOT STARTED | Not on the critical path (the spec defaults to static). | 11 | — | 08-10 |
| TRV-0247 | (Friday) "**No existing determinism test ever puts a bot in a mech, and I can prove it.** 0 bot-mech ticks out of ~4,800 ticks × 9 bots." | FRIDAY_LOG.md:1101-1104 | a | S | DELIVERED | Closed by `2d726ba` "Extend the replay guarantee to cover BOT MECHS - the path it did not reach" | 01 | — | 08-10 |
| TRV-0248 | (Thor) "the scout branch sits **above** the `roll_t` and `crouch` arms, so the scout is now the only fighter in the game whose hitbox height **never changes** — rolling or crouching no longer shrinks it at all (1.776 m, vs the 0.95 it used to get while rolling). *Also worth an owner decision.*" | THOR_LOG.md:1696-1699 | a | S | BLOCKED | Unblocker, named: **an owner decision.** A dodge that does not duck the head band changes what a roll is for. | 01 | depends-on owner | 08-10 |
| TRV-0249 | (Thor) "Doc rot: '3.03 m' ×5, '1.78m soldier' ×1." | THOR_LOG.md:1785 | a | D | UNVERIFIED | Not re-derived. Now higher-stakes than when written: with a Royal at 1.10× the Big, "3.03 m" is wrong for two of three tiers. | 12 | — | 08-10 |
| TRV-0250 | (Thor) "a first-person aim test placed its camera at `muzzle_origin` — the very function under test — so the mutation moved both together and the assertion could never fail." | OPERATION.md rule 12; THOR_LOG.md:2762-2782 | a | S | DELIVERED | De-vacuumed, and a test now pins the property because it held by **coincidence** (muzzle == eye) rather than by construction. This is the project's canonical example of rule 12. | 18 | see TRV-0053 | 08-10 |
| TRV-0251 | (Thor) the operation audit — "the capture loop is the bottleneck" | commit `df887dd`; THOR_LOG.md:2826-2832 | a | C | NOT STARTED | ~6 min per capture cycle; ~40 s if beats were data. Three iterations to move a camera = 18 minutes of pure rebuild, and it recurs. | 13 | duplicate-of TRV-0040 | 08-10 |
| TRV-0252 | (scout-defect) "**Every mech's hull turret spins from the PLAYER's trigger.** One rate computed from `fighters[player]` is written to every `MechTurretSpinner`... On foot, nothing spins at all." | WHATS_MISSING.md:333-337 | a | C | UNVERIFIED | Claimed fixed; **no capture**, and TRV-0056 lists exactly this as capture-verification owed. Carrying the scout's evidence forward rather than a disposition. | 13 | — | 08-10 |
| TRV-0253 | (scout-defect) "**The carried minigun's WORLD model never spins** — the spinner is only tagged when `with_hands` is true, which is the viewmodel flag." | WHATS_MISSING.md:338-339 | a | C | UNVERIFIED | Same: claimed fixed, no capture. | 13 | — | 08-10 |
| TRV-0254 | (scout-defect) "Third-person `crouch_drop` is double-counted for a kneeling mech." | WHATS_MISSING.md:345 | a | C | UNVERIFIED | Not re-derived. FRIDAY22 handed exactly this to FRIDAY33 (root `FRIDAY_LOG.md:29-32`: "`main.rs:17359` adds `MECH_BRACE_STANCE_DROP` on top of a hull the sim now sinks itself"). | 01 | — | 08-10 |
| TRV-0255 | (scout-gap) "Both scouts also confirmed what is **NOT** a gap, which is worth as much: the light chassis refusing to crouch, the medic having no power core, the armour-pip HUD excluding the medic, and 8v8/Extraction being withdrawn from the menu are all **DELIBERATE and documented at the site**." | WHATS_MISSING.md:348-351 | a | D | DELIVERED | Recorded as a row so no future sweep re-opens them. OPERATION rule 11's corollary: read the comment before you call something absent. | 12 | — | 08-10 |

### B.15 — IMAGES (uploaded reference and generated evidence)

**An image the owner uploaded is a SPEC in picture form.** Each row states
what the picture asks for in words.

| ID | Image / ask | Issued at | O | L | Status | Evidence | Thread | Links | Chk |
|---|---|---|---|---|---|---|---|---|---|
| TRV-0256 | `key_art.png` — **owner upload.** What it asks for in words: the game's whole visual identity — the palette every menu, HUD and seal is themed from, and the mood the splash sets before a player sees a single frame of play. | `engine/assets/branding/key_art.png`, added `923358d` (2026-08-04) | o | A | DELIVERED | Loaded `branding.rs:303`; drove `77fa134` "Retheme the UI to the key art - one palette instead of 30 magic literals". Captures `splash/01-hold.png`, `02-fade-out.png`, `menus/01-title-page.png`. | 24 | — | 08-10 |
| TRV-0257 | `wordmark.png` + `wordmark_on_light.png` — **owner upload.** Asks for: the game's name set as a mark, with a light-background variant so it can sit on a parchment plate as well as a dark one. | same commit | o | A | DELIVERED | `branding.rs:304`; menu titles and seal footers | 24 | — | 08-10 |
| TRV-0258 | `emblem.png` + `emblem_small.png` — **owner upload.** Asks for: a single faction/house mark that reads at two sizes — full plate and inline chip. | same commit | o | A | DELIVERED | `branding.rs:305-306` | 24 | — | 08-10 |
| TRV-0259 | `app_icon.ico` — **owner upload.** Asks for: the game to look like a real desktop application, not a cargo binary. | `engine/assets/branding/app_icon.ico` | o | A | DELIVERED | `468a9b8` "Package the game as a real desktop app: icon, no console, shortcuts" | 24 | — | 08-10 |
| TRV-0260 | **THE MECH CONCEPT ART — MISSING.** What it asks for, in the words of the two documents written *from* it: a **walking weapons platform**, ≈2.5× soldier height (hull top ~4.5 m, antenna ~5.2 m, the soldier's helmet reaching only the knee); hull cantilevered forward over **reverse-joint legs**, pitched nose-down, hips high and set back; **no head, no cockpit glass, no visible pilot opening**; arms mounted **low on the hull sides**, not shoulder-top; a **gatling on one arm** and a **drum-fed autocannon** on the other, the drum being "the most recognisable shape on the machine"; **knee and waist mechanisms deliberately exposed**; **wide flat splayed feet with a rear spur and visible cleats**; olive-drab / khaki / field tan with minimal emissive; a stencilled "3" on the hull. | `BRIEF_VIII_B` §A + §D (whole section — "**The art is the spec**"); `PROMPT_mech_rebuild.md` Task 1 and Task 5's completion criterion | o | A | **BLOCKED — on the owner** | Verified absent: `git ls-files` across `*.png *.jpg *.jpeg *.gif *.webp *.glb *.gltf *.fbx` returns five branding files and 155 handback captures, nothing else. `git log --diff-filter=D` shows no image of this kind was ever committed and later deleted. `handback/reference/` contains one file: `NOTES.md`, which opens by stating the image deliverable was not achievable. | 24 | blocks TRV-0075, 0100, 0115, 0172 | 08-10 |
| TRV-0261 | **THE MEDIC REFERENCE ART — MISSING.** What it asks for, in the words of the record: "a squat utility robot, rounded masses, one big camera lens, worn amber over near-black." | quoted in `WHATS_MISSING.md:513` | o | A | **BLOCKED — on the owner** | The chassis WAS rebuilt to it (`72a93f3`) and photographed (`medic/01..09.png`). But the image is not in the repo, so every future judgement about whether the machine still matches it is one person's memory of a chat. | 24 | see TRV-0208 | 08-10 |
| TRV-0262 | **"YOUR UPLOADED GUN ASSETS" — MISSING.** Referenced by the owner as §23 of the 36-section spec. | `WHATS_MISSING.md:169-171`, `:466-467` | o | A | **BLOCKED — twice over** | (1) No unused image or model asset is anywhere in the repo — `engine/assets/characters/` holds only `.gitkeep`. (2) Even if supplied, **`jk_tdm` has no glTF loader** (TRV-0048), so a GLB would change nothing in the shipping game. The 2026-08-08 scout sharpened this correctly: this is not "the owner points at the assets", it is "the shipping crate cannot load a mesh if pointed at one". | 24, 14 | depends-on TRV-0048 | 08-10 |
| TRV-0263 | `output/concept_commander.png` — **generated, not uploaded.** Produced by `concept/commander_concept.py`. Belongs to the `jk_wall` battle-sim line, not to `jk_tdm`. | `output/concept_commander.png` | a | A | DELIVERED | No action. Indexed so it is never mistaken for owner reference art. | 24 | — | 08-10 |
| TRV-0264 | `output/battle_40v40.gif`, `third_person.gif`, `dbg_tp10.png`, `frames/*.png` (56 files) — **generated spike artefacts** from the `jk_wall` 40v40 milestone. | `output/` | a | A | DELIVERED | No action. Indexed for the same reason. | 24 | — | 08-10 |
| TRV-0265 | `GAME_STATUS_REPORT.pdf` / `.md` — the 2026-08-01 full-project brief, "verified against actual code". | repo root | a | D | **DELIVERED, now materially stale** | Its "honest remaining scope" names five large rebuilds; three of them have since shipped (the 20-segment rig, the 26-piece armour, the 4-class system). Its Armor Sets section still lists **Pyro**. Anyone reading it today gets a picture ten days old. Flagged, not edited — Trevor does not edit other people's documents. | 12 | corrects nothing; is corrected by this ledger | 08-10 |
| TRV-0266 | **The 155 committed handback PNGs across 30 script directories** — the project's entire visual evidence base, and the reason "visible or it didn't happen" is enforceable at all. | `engine/crates/jk_tdm/handback/brief-vii/**` | a | A | DELIVERED | Index row. Largest sets: `mech_gallery` (11), `medic` (9), `cockpit` + `medic_cockpit` (16), `sights_a|b|c` (12), `menus` (12), `grenade_hold` (7), `map_lap` (7), `mech_jump` (6), `spear_flight` + `arrow_flight` (12), `bow_draw` + `bow_draw_fp` (10). **Directories with zero captures for a shipped system: the recoil envelope, the armour damage states, the weapon strip as slots, the boarding beats.** | 13 | — | 08-10 |

---

## §C — THE RESEARCH INDEX

Everything under `engine/crates/jk_tdm/research/`, categorised on the
three axes. Eleven topic directories, one source ledger, one researcher
log.

### C.1 — AXIS 1: TOPIC (which thread does it serve?)

**Research that serves no thread is the finding, not a filing problem.**

| Directory | Files | Serves thread | Verdict |
|---|---|---|---|
| `body-rig/` | `SPEC_20_SEGMENT_RIG.md`, `SOURCES.md` | 17 (body rig) | **The best-served research in the repo.** Every value reached code. |
| `grenade/` | `SOURCES.md`, `CYCLE_2_REPORT.md` | 02 (grenade) | Served. Bounce coefficients and the falloff table both shipped. |
| `mech-entry/` | `SOURCES.md`, `CYCLE_1_REPORT.md` | 05 (boarding) | Served sim-side; the presentation half it argued for is still unbuilt. |
| `mech-climb/` | `DESIGN.md`, `SOURCES.md`, `CYCLE_3_REPORT.md` | 09 (traversal) | Served. Design → build in one hop (`7e164d4`). |
| `spear-throw/` | `SOURCES.md` | 19 (weapons) | Served. Javelin biomechanics reached the throw. |
| `armor-damage/` | `SOURCES.md` | 06 (armour) | **Half-served.** The sim half consumed it; the client half does not exist, so the visual half of the research has no consumer. |
| `aiming/` | `SOURCES.md` | 18 (first person) | **Served as a negative result**, which counts. Also the site of the fabrication incident. `AIM_SPEC.md` never written. |
| `maps/` | `SOURCES.md` | 08 (maps) | **Orphaned.** Real cross-engine metrics extracted; `MAP_METRICS.md` never written; traversal is blocked on that file. |
| `traversal/` | `SOURCES.md` | 09 | **Orphaned.** No general traversal system exists. |
| `vertical-maps/` | `SOURCES.md` | 08 | **Orphaned by deletion.** Its one usable line ("what you see is what you get") reached Cliffhold, whose client half was then removed. |
| `motion-architecture/` | `SOURCES.md`, `NOTES.md` | 16 | **Orphaned.** 5 of 14 core sources read; `DECISION.md` — the entire deliverable — never written. |
| `SOURCES.md` (root) | the master ledger | 15 | Honest about its own shortfall, which is its main virtue. |
| `TOTO_LOG.md` | four dispatches | 06, 08, 17 | Served on the rig; half-served on armour; orphaned on vertical maps. |

**Missing entirely** (a topic the prompts demanded and no directory
exists for): `fp-dynamics`, `character-creation`, `weapon-systems`,
`map-design`, `powered-armour`, `grenade-physics` as its own slug.

### C.2 — AXIS 2: TIER AND SOLIDITY

Per `SOURCES.md`'s own counts, unedited:

| Topic | Counted | Tier P | Tier V | Target |
|---|---|---|---|---|
| fp-dynamics | 2 | **0** | **0** | 16 / 4 / 4 |
| ballistics + maps (owner-supplied) | 2 | 2 | **0** | — |
| character-creation | **0** | **0** | **0** | 16 / 4 / 4 |
| weapon-systems | 2 | **0** | **0** | 16 / 4 / 4 |

**Tier V — video with timestamped quotes — is ZERO on every topic in the
root ledger.** The one place it is not zero is `vertical-maps` (TOTO33,
2026-08-08: "THE HEADLINE: tier-V is no longer 0"), and that is the topic
whose product was deleted.

**Solidity of the values that DID reach the code**, labelled per the
project's own MEASURED / DERIVED / ASSUMED discipline:

| Value | Label | Where it is recorded | Note |
|---|---|---|---|
| Segment mass fractions, lengths, CoM, radii of gyration | **MEASURED** | `body-rig/SPEC_20_SEGMENT_RIG.md` | Dempster / Winter / de Leva. Mass closure asserted to ±0.001. |
| The clavicle's mass split | **ASSUMED — and terminal** | TOTO_LOG 2026-08-03 (second pass) | Toto retried Hatze, could not read the paper, and declared the label terminal rather than upgrading it. Exemplary. |
| The toe's mass split | **DERIVED, corroborated** | TOTO_LOG 2026-08-03 | A new corroborating measurement was found and recorded. |
| Grenade bounce coefficients | **MEASURED** (brief table) | `grenade/CYCLE_2_REPORT.md` | Stone 0.40 / crate 0.50 / organic 0.05, each behaviourally tested. |
| Blast falloff breakpoints | **MEASURED** (brief table) | `handback/brief-ix/REPORT.md` | Shape matched, not raw numbers — the brief's 80 HP baseline does not transfer to this game's 100. Recorded, not silently averaged. |
| Armour wear rate | **ASSUMED** | FRIDAY_LOG §C, labelled at `wear_plate` | "The one number [the brief] does not give." Baseline rifle scuffs on round 2, cracks on 4, severs on 6, strips on 7. |
| Armour degradation curve shape | **DERIVED** — concave, not linear | TOTO_LOG 2026-08-08 | Arithmetic in the ledger. |
| `TURRET_FELT_FLOOR = 24.0` | **DERIVED from measurement** | `sim.rs:5367` | Midpoint between the felt floor and the M4's 28.8, which an existing test already caps the mount at. |
| `TURRET_AIM_STABILISER = 0.25` | **MEASURED, and the algebra was wrong** | FRIDAY_LOG `d6e35d1` | The reciprocal-of-the-lift version would have let a bot's rounds walk 38% further, because punch angle is **superlinear** in the impulse. Caught only by measuring the plateau. Written into the constant's doc so nobody re-derives the wrong thing. |
| Crosshair clamp ranges (size 1..12, gap −5..12, thickness 1..5) | **ASSUMED** | FRIDAY_LOG, stated | "the clamp RANGES are mine, not the brief's". |
| Dynamic crosshair 2400 px/rad | **ASSUMED, inherited** | FRIDAY_LOG | Tuned against a bracket glyph, not against arm travel. |
| `BARRIER_SCALE = 1.60` | **DERIVED, with the arithmetic shown** | `WHATS_MISSING.md:511` | 60% against the ORIGINAL 1.7 m, not against the already-1.55× tree — reading it the other way gives 4.2 m, which the file's own test rejects as "a building, not a shield". |
| Recoil channel split (45% camera tracking) | **MEASURED** (CS:GO shipped data) | `SOURCES.md` S-01/S-02 | Validated after the fact rather than derived from it. |
| de Carpentier analytical solver | **READ, deliberately NOT adopted** | `SOURCES.md` S-09 | Swapping it in would change every existing throw and invalidate the golden tests. The *solve-for-launch* capability is recorded as a real future opportunity for throw assist and bot grenade aim. |

**The fabrication incident, which is why this axis exists at all.** A
`WebFetch` extraction of the Vicencio-Moriera CHI 2014 aim-assist paper
returned "~87% hit rate", "2.5 degrees visual angle", "1.8× standard
reticle diameter", "0.7 normalized units" and "20 participants", plus an
invented fifth technique name. **None of those numbers appear anywhere in
that paper.** It was caught only because the raw PDF was read and checked
against the summary. Recorded in `research/aiming/SOURCES.md` and cited by
`PROMPT_RND_CYCLE.md` as the proof that a breadth quota is a fabrication
generator.

### C.3 — AXIS 3: CONSUMED OR ORPHANED

Did any value in this research reach the code?

**CONSUMED — a value from this research is live in the build:**
`body-rig` (20 segments, mass closure, spring stiffness derived from
`m·(k·L)²`), `grenade` (bounce coefficients, falloff breakpoints,
determinism test shape), `mech-entry` (the 8-stage sequence and the
argument that it must be committal), `mech-climb` (grip-fatigue model →
`CLIMB_GRIP_MAX` and the asymmetric drain), `spear-throw` (release angle,
runway-speed correlation → the ×1.15 running bonus).

**PARTLY CONSUMED — the research landed, half of its product did not:**
- `armor-damage` — the four-stage model is live sim-side; the client
  reads none of it, so every visual finding in that dispatch is inert.
- `aiming` — the CHI 2014 paper's "correct the outcome, never the
  player's hand" became OPERATION rule R9; the five §7 hypotheses it
  informed all came back clear, which is a real result. `AIM_SPEC.md`,
  the actual deliverable, does not exist.

**ORPHANED — nothing in the code reads it:**
1. **`motion-architecture/`** — the largest. 5/14 core sources read, no
   `DECISION.md`, no crate ever evaluated against Bevy 0.15. The one
   durable output is the LaFAN1 licence trap (CC BY-NC-ND 4.0:
   NonCommercial *and* NoDerivatives — papers readable, data unshippable).
2. **`maps/`** — real cross-engine metrics extracted and READ (S-10:
   player box, eye height, min hallway 2.0 m, stairs 15×25 cm at 30-35°,
   TF2's 256/1024 u range bands). `MAP_METRICS.md` never written, so
   TRV-0051 and TRV-0180 are both blocked on a file that exists as
   research and not as a deliverable.
3. **`traversal/`** — no traversal verb exists to consume it.
4. **`vertical-maps/`** — orphaned by deletion, which is the saddest
   category. The dispatch cost ~211k tokens; its one line that reached
   code reached Cliffhold's client half, and that was removed.
5. **The eight tier-P seeds in `PROMPT_MASTER` §1.5** — Weihs (GDC 2013
   aim assist), Brink's SMART, the Bournemouth parkour thesis, Yoder's
   multiplayer level design, Reitich on projectile prediction, Wagar on
   i-frames. All still `SNIPPET-ONLY`. Never read, never counted, never
   retired. The prompt's own honesty note says so plainly and that note
   is still accurate 10 days later.

**The measured verdict on the tier, from OPERATION rule 13's own table:**

| tier | cost | what reached the code |
|---|---|---|
| `toto` armour | ~187k tokens | a positional-damage model the builder weighed and did **not** adopt |
| `toto33` vertical maps | ~211k tokens | one usable line + a bot finding a builder could have grepped in a minute |
| `scout-defect` | ~150k | dead grenade throw, turret spinning off the wrong fighter, 3 inert constants, a doc-rot cluster |
| `scout-gap` | ~127k | 8 boarding stages rendering nothing, the medic rendering man-sized, two castle centrepieces that are solid boxes |

That table is why the research tier is retired, and I am not arguing with
it. I am recording that **five research artefacts have no consumer and
nobody has decided whether to close them or feed them.** Leaving
`motion-architecture` at 5/14 forever is the worst of the three options.

---

## §D — WHERE I DISAGREE WITH THE RECORD

Four documents in this repo make claims that did not survive today's
re-derivation. I do not edit other people's files; I record the
disagreement here and hand it over.

| Document | Claim | What I found |
|---|---|---|
| `BACKLOG.md` #4 | Melee depth "Not started" | **False.** Melee v2 shipped `a99af96` with directional lines, line-matched parry, and three captures. |
| `BACKLOG.md` #5 | "RETREAT is the remainder" | **Stale.** Retreat shipped `e5431a4` with hysteretic fear, class-derived `ROUT_TOLERANCE` and a test. |
| `BACKLOG.md` #9 | Character creation blocked because "no class system and only 5 whole-body armour presets" | **Blocker is false.** Four classes shipped 2026-08-05; 24 per-piece plates shipped 2026-08-07. The row is unblocked and nobody noticed. |
| `BACKLOG.md` #11 | "The armour-weight formula exists but is unwired" | **False.** Wired in the 24-plate pass, with a live weight-against-ceiling readout in the Forge. |
| `BACKLOG.md` #12 / `research/SOURCES.md` | "Zero texture pipeline. **All 21 `asset_server.load` calls are `.wav`.** Unblocker: any image loading at all." | **False since `03085b1` (2026-08-03).** There are 24 loads: 20 `.wav` and **4 `.png`**. The correct, narrower blocker is: no imported image reaches a WORLD material, and `jk_tdm` has no glTF loader. |
| `TASK0_AUDIT.md` §Before-clips | "Missing vs the letter of the brief: a dedicated traversal clip (c) and a full map lap (e)" | **Stale.** Both exist now — `traversal/01..04.png` and `map_lap/01..07.png`. |
| `handback/brief-ix/REPORT.md` | IX-C's class system, 26-piece armour and damage states "not attempted" / "doesn't exist yet in any form" | **All three shipped since.** The report is a correct snapshot of 2026-07-30 and a misleading one today. |
| `GAME_STATUS_REPORT.md` | Lists **Pyro** among the five armour sets; names the 20-segment rig, 26-piece armour and 4-class system as unbuilt | **All four out of date.** Pyro was deleted `b11b7de`; the other three shipped. |
| `DESIGN_MAP.md` | Cited by `PROMPT_MASTER` §Read-first as "what is actually built versus specified" | **It maps the wrong game.** Every row is about `jk_wall`, the shieldwall battle sim. It says nothing about `jk_tdm`, which is what the prompt then goes on to target. A session following the prompt's reading order gets a confident, detailed, irrelevant answer. `TASK0_AUDIT.md` line 5 already noticed this and said so. |
| Root `FRIDAY_LOG.md` vs `research/FRIDAY_LOG.md` | Two files, same name, same agent | **They do not contradict each other — they are disjoint.** The root file holds three entries (the §21 sim half, the §C armour sim half, and CLIFFHOLD) and stops. The research file holds the full append-only history through 2026-08-10. The root file is an orphan that a later session did not know about. Nothing is lost, but a reader who opens the root one gets 3 of ~20 entries and no signal that more exist. |

---

## §E — HANDOFFS

### E.1 — Rows that need **THOR** (claimed done, evidence thin or contested)

| Row | Why |
|---|---|
| TRV-0008 | The recoil envelope is the biggest feel change in the game and has **no capture**. Prove the controlled window and the chaotic ramp are visible, not just tested. |
| TRV-0005 | `Shield [4]` as an interactive slot: no capture frames the strip as four slots. |
| TRV-0054 | Jump telegraph: I did not confirm the rig keys on `chassis_kneeling()` rather than raw `f.crouch`. |
| TRV-0035 | The barrier test's alpha and span constants are copied into the test body. If it is vacuous, TRV-0207's evidence is too. |
| TRV-0057 | `visor_ready` is still inside a `Local`. Does the camera actually cut to the visor, or does the flag go nowhere? |
| TRV-0237 | `fe07c19` may have closed the `mech_visor_eye_y` free-function bug. Confirm or reopen. |
| TRV-0252, TRV-0253 | Turret spinners and world minigun spin — claimed fixed, never photographed, and TRV-0056 lists both as capture-verification owed. |
| TRV-0250 | Already de-vacuumed once. Re-check it still cannot pass by coincidence now that a third chassis exists. |
| TRV-0134 | `armor_weight_movement_penalty` is claimed wired. Mutation-prove it. |

### E.2 — Rows that need **TOTO** (blocked on a number nobody has)

Rule 13 says `toto*` only when a specific unknown NUMBER blocks a build,
named in the dispatch. **By that test, exactly two rows qualify today:**

| Row | The number nobody has |
|---|---|
| TRV-0010 | The Royal arrow launcher's rate of fire, bolt velocity and per-bolt damage against a mech hull. Nothing in the game fires a bolt from a mount; there is no neighbouring value to interpolate from. Everything else about it is art. |
| TRV-0126 | "How enclosed is this point" — the query itself is the unknown, not a coefficient. If it is dispatched, ask for the **method** (ray fan? nearest-wall distance? cell occupancy?) and its cost at 120 Hz, not for a percentage. |

Everything else that looks like a research need is really a decision
(TRV-0013, 0053, 0059, 0236, 0239, 0240, 0248) or a build.

### E.3 — Rows ready for **FRIDAY** now, with the lane

`sim.rs` → **friday22**. `main.rs` + client modules → **friday33**.

**friday33** (the busy lane): TRV-0010, 0011, 0012, 0013, 0014, 0015,
0024, 0028, 0030, 0031, 0032, 0033, 0036, 0037, 0040, 0055, 0061, 0136,
0139, 0206, 0243, 0244, 0245.

**friday22** (`sim.rs`): TRV-0026 (with `gen_sfx.py`), 0039 (the Cliffhold
sim half — coordinate, or the `match` breaks), 0055 (the sim side of the
scale split), 0235, 0241, 0242.

**Both lanes, coordinate or it breaks**: TRV-0039 only.

### E.4 — Rows blocked on the **OWNER**, and nobody else

| Row | The question, in one sentence |
|---|---|
| TRV-0260 | Please drop the mech concept art into `handback/reference/` — three completion criteria in two briefs are unsatisfiable without it. |
| TRV-0261 | Same for the medic reference art. |
| TRV-0053 | Should the mech's first-person aim leave the visor or the hull turret? A hull turret genuinely IS a metre below the visor, so this may be correct and merely unstated — but changing it moves every mech engagement in the game. |
| TRV-0013 | The Royal's accents shipped **gold**; the spec asked for **subtle neon-blue**. Which? |
| TRV-0059 | `SCOUT_SCALE = 1.05` makes the medic 1.87 m — a big man, not a machine. Is that the intent? |
| TRV-0239, TRV-0240 | The recoil envelope cost bot mechs about a third of their sustained output, and braced fire is no longer perfectly accurate. Both are real balance changes nobody asked for. Playtest and rule. |
| TRV-0248 | The scout's hitbox now never shrinks when rolling or crouching. Is a dodge that does not duck the head band still a dodge? |
| TRV-0236 | `armor_spec`'s flat values are unreachable for any piloted heavy chassis. Keep them scaled for table consistency, or delete them? |

---

## §F — WHAT I COULD NOT CHECK, AND WHY

Stated plainly. This sweep was **not** complete and I will not imply it was.

**I did not build and did not run the suite.** Trevor is read-only and did
not invoke `cargo`. Every `DELIVERED` in this file means "there is
evidence at this path", never "it compiles" and never "it works". Two rows
(TRV-0020, TRV-0021) exist solely to record that.

**I did not open a single PNG.** Capture rows cite paths and filenames.
Whether `04-wind-full.png` actually shows a grenade cocked past the
shoulder, I do not know. Rule 8 says the capture is the instrument — I
indexed the instruments, I did not read them.

**Read whole:** `WHATS_MISSING.md`, `OPERATION.md`, `briefs/README.md`,
all four `BRIEF_*.md`, all five `PROMPT_*.md`, `DECISIONS.md`,
`DESIGN_MAP.md`, `GAME_STATUS_REPORT.md`, root `README.md`, root
`FRIDAY_LOG.md`, `BACKLOG.md`, `ANTI_PATTERNS.md`, `TASK0_AUDIT.md`,
`research/SOURCES.md`, `handback/reference/NOTES.md`,
`handback/brief-ix/REPORT.md`, the last 310 lines of
`research/FRIDAY_LOG.md`, and the headers of `held_grenade.rs`,
`mech_recoil.rs`, `mech_lineup.rs`.

**Sampled by grep only, and I am naming them so nobody mistakes this for
coverage:** `THOR_LOG.md` (3,002 lines — I read roughly 200 of them,
around owner-voice hits), `TOTO_LOG.md` (685 lines — **headings only**),
`sim.rs` (~27k lines — targeted greps plus about eight regions),
`main.rs` (~29k lines — targeted greps plus about five regions),
`cockpit.rs`, `map_look.rs`, `menu_ui.rs`, `branding.rs` (grep only).

**Not read at all:** `handback/ACCOMPLISHMENTS.md`, `handback/AUDIT.md`,
`handback/CHANGES.md`, `handback/REPORT.md`,
`handback/brief-vii/HANDBACK.md`, the eleven per-topic
`research/*/SOURCES.md` files, `research/motion-architecture/NOTES.md`,
`research/body-rig/SPEC_20_SEGMENT_RIG.md`, `research/mech-climb/DESIGN.md`,
the three `CYCLE_*_REPORT.md` files, every source file in `jk_wall`,
`jk_bevy`, `jk_client`, `jk_spike`, `jk_core`, and `export/` (gitignored;
two files, `john_kingdom_game_source.md` and `STATUS.md`). Rows citing
those documents cite them as artefacts, not as things I verified.

**The 34 `UNVERIFIED` rows are the honest total of the above.** Each one
says in its Evidence cell what I did and did not do. None of them carries
a fabricated disposition, and none of them was bucketed as a negative
result because the check did not finish.

**Concurrency.** Two commits (`fe07c19`, `f10be3a`) landed from another
session while this sweep ran, and both changed files I had already
grepped. TRV-0238 moved from Friday's open deferral to `DELIVERED`
because of one of them. Line numbers in this file were true at
`f10be3a`; anchor to the symbol names, not the numbers.

**Tooling note for the next Trevor.** On this machine `bash` has no
coreutils — `ls`, `find`, `head`, `sort`, `wc`, `cat` all exit 127, and
`powershell.exe` is not on the Bash tool's PATH either. `git` works after
`export PATH="$PATH:/c/Program Files/Git/cmd"`. Use the Grep and Glob
tools for everything else. `git grep`, `git ls-files` and
`git log --diff-filter=D` did all the heavy lifting in this run.
