# TREVOR LEDGER — the issued-vs-delivered data bank

**Rebuilt in full: 2026-08-11 (run 4).** Current truth, not history.
History is `TREVOR_LOG.md`. The queue is `TREVOR_TASKS.md`. Every
`TRV-####` is permanent: never renumbered, never reused, never deleted.

Repo HEAD at sweep start and end: **`46d6dbb`** ("Thor's bow/spear verdict:
two rules held, and a pose that was never posed"). `git fetch` before
trusting any line number — three sessions push here.

**Working tree at sweep time, and I banked none of it:**

| Path | State | How I treated it |
|---|---|---|
| `src/main.rs` | modified | Read for greps. Uncommitted lines NOT counted as delivered. |
| `src/muzzle_flash.rs` | modified; the module itself IS committed | Indexed as `TRV-0375`. |
| `handback/brief-vii/muzzle_flash/` | **untracked** | Captures that exist on this disk and in no commit. `TRV-0375`. |

---

## §0 — HEADLINE

| | run 2 (2026-08-11 am, `5473d06`) | **run 4 (now, `46d6dbb`)** |
|---|---|---|
| **Rows total** | 344 | **379** — `TRV-0001`..`TRV-0379`, no gaps |
| `DELIVERED` | 142 | **156** |
| `PARTIAL` | 48 | **59** |
| `NOT STARTED` | 87 | **96** |
| **Open (`PARTIAL`+`NOT STARTED`)** | 135 | **155** |
| `BLOCKED` | 22 | **24** |
| `SUPERSEDED` | 12 | 12 |
| `CONTRADICTED` | 8 | 8 |
| `UNVERIFIABLE` | 4 | 4 |
| `UNVERIFIED` | 21 | **20** |

Sums to 379 exactly. Origin: **345 owner · 28 agent · 6
agent-found-then-adopted-by-owner.** Owner rows outrank agent rows at
equal severity, always.

**Open work went up 20 while 14 rows closed.** Two briefs (XII, XII-A)
were issued AND built inside one day; the rows they opened outnumber the
rows they closed. That is a spec arriving, not a regression.

### Where the row TEXT lives — read this before you go looking for a row

The ID space `TRV-0001`..`TRV-0344` was allocated across runs 1-2 and its
full per-row text (quoted ask, issued-at, evidence) lives in
**`ISSUED_VS_DELIVERED.md`** — run 2's register, in this same ID space.
I did **not** re-transcribe 344 rows I did not personally re-derive today.
Copying a status forward under a fresh date is precisely the fabrication
this role exists to prevent.

What is in THIS file, all derived today at `46d6dbb`:

- **§A** the threads — the whole life of each ask.
- **§B** every row whose status **MOVED**, in full, with the evidence.
- **§C** the **35 rows opened this run** (`TRV-0345`..`TRV-0379`), in full.
- **§D** the research index, on the three axes.
- **§E** rows for Thor, rows for Toto, and what I could not check.

**Any row in `ISSUED_VS_DELIVERED.md` that does not appear in §B below was
not re-derived by me today.** Its run-2 status stands at its run-2 date.
It is not dated 2026-08-11 and must not be quoted as if it were.

**What `DELIVERED` means here.** "There is evidence at this path." Not "it
works." Thor decides that. I did not build and did not run `cargo`.

---

## §A — THE THREADS

### THREAD 01 — the bow & spear spec (the loudest thread in the file)
Rows: TRV-0294, 0301..0317, 0367, 0368, 0369, 0371, 0372
First asked:  chat, 2026-08-10. **STILL NOT ON DISK.** `git ls-files briefs/`
              returns 14 files and no bow/spear brief. `TRV-0313`.
Restated:     the two load-bearing rules were each stated THREE times —
              *"RIGHT MOUSE IS PRE-AIM ONLY — it never starts a charge.
              The attack button charges. PRE_AIM and CHARGING are never
              combined"* and *"the crosshair is sacred — nothing obstructs
              it"*.
Research:     `research/spear-throw/SOURCES.md`
Built:        sim half `a95b48c` (`enum AimPhase`, `fn aim_phase`), charge
              curve `5ad7519` (`SPEAR_CHARGE_FULL_S` 3.00,
              `SPEAR_MAX_CHARGE_S` 7.00, `SPEAR_MAX_CHARGE_DMG` 1.10),
              physics `eff8fbf`, **visual half `2575909`** — wood, leather,
              cord, and the weapon moved to the right of frame
Pictures:     `spear_fp/01..06` — **including `02-fp-spear-preaim.png`, the
              first frame in this repository that holds RMB** —
              `bow_draw_fp/01..05`, `bow_draw/01..05`, `spear_flight/00..05`,
              `arrow_flight/`. **`handback/brief-vii/spear_wind/` does not
              exist** although `SPEAR_WIND_BEATS` (`main.rs:6011`) is
              registered as `"spear_wind"` (`:6970`) — `TRV-0368`.
Verified:     THOR 2026-08-11 at `1713fe1`, suite re-run by him not taken
              on report: **478 pass / 0 fail / 2 ignored**. Verified
              against PIXELS, not code.
State:        **PARTIAL, and Thor found the seam.** Rules 1 and 2 hold in
              first person for both weapons, photographed. What did not
              ship: `spear_wind_frac_of` has exactly **ONE** client reader
              (`main.rs:18705`, the third-person rig); `spear_stance_of`
              and `spear_max_charged_of` have **ZERO**; and `main.rs:21536`
              still hand-rolls `1.0 - p.spear_wind_t / SPEAR_WINDUP_S` off
              the **release** clock — which reads 0 for the whole charge.
              **So in first person the javelin does not move at all through
              a seven-second charge**, and the commit message says it does.
              Thor proved it from the shipped binary: `02-fp-spear-preaim`
              and `04-fp-spear-full-charge` put the spear head at the same
              pixel while the HUD reads `JAVELIN FULL [#########]`.
Next:         friday33 — `TRV-0367`. Thirteen of nineteen sections remain
              invisible to this ledger (`TRV-0314`) and will stay invisible
              until the spec is pasted into `briefs/`.

### THREAD 02 — the HUD (BRIEF XII + XII-A): issued and built in one day
Rows: TRV-0345..0362; TRV-0031/0032/0033/0055 fold into its §8 audit
First asked:  `briefs/BRIEF_XII_hud_human_vs_mech.md`, then
              `briefs/BRIEF_XII_A_hud_consolidation.md` — **both written to
              disk before any build touched them. That is the standard.**
Research:     NONE, by design. The reference is two owner images.
Built:        **new module `src/hud.rs`, 1,742 lines** (`1713fe1`,
              `2ad5d42`, `85e0667`, `d73a6d0`). The legacy `weapon_strip`
              and `shield_readout` are deleted. Heat and hull
              de-duplicated behind one helper, `hud.rs:190 fn heat_pct`.
              `44115e7` stopped the selected mount printing its number
              underneath it; `7218e37` and `1515ac9` fixed the armoury and
              soldier pills.
Pictures:     `hud_contrast/`, `menus/`
Verified:     `d73a6d0`'s own subject — *"two things the capture caught
              that reading could not"*. Rule 8 paying again.
State:        **DELIVERED for the consolidation, BLOCKED for the art
              direction.** The two owner reference images are **not on
              disk**. `handback/reference/hud/NOTES.md` is the mitigation
              and it is the best archival act in this repo this week:
              it describes both images in words *written while they were on
              screen*, states plainly *"until the PNGs land here, this
              description IS the reference"*, and **cites `TRV-0260` as
              the reason it exists**. The ledger's oldest open row was used
              to prevent its own repetition.
Next:         owner drops `img1-mech-pov.png` and `img2-human-pov.png`
              into `handback/reference/hud/`. **No builder is blocked** —
              the written description is workable. The *acceptance check*
              is what is blocked. `TRV-0361`, `TRV-0362`.

### THREAD 03 — the Agile Mech: BRIEF X built, BRIEF XI blocked at three points
Rows: TRV-0282..0293, 0296, 0318..0341
First asked:  `briefs/BRIEF_X_agile_mech.md` (`efe1428`)
Restated:     `briefs/BRIEF_XI_agile_mech_motion_limbs_guns.md`, 299 lines,
              `dd4fced` — owner's framing: *"Finish the Agile Mech rather
              than continually redesigning unrelated systems, while making
              the shared limb/hand/weapon system explicit."*
Research:     `research/motion-architecture/DECISION.md` — written
              2026-08-10 (`632824a`), and **BRIEF XI §0.2 makes its item 3
              BINDING**. Orphaned research that stopped being orphaned.
Built:        `src/agile_mech.rs`, 798 lines. The palette is `pub const`
              data with a guard test
              (`the_enemy_agile_never_out_luminates_the_ally`, `:744`).
              **BRIEF XI: nothing built. `dd4fced` adds one file and
              changes no code.**
Pictures:     `agile_body/01..09`, `agile_moves/01..12`
State:        **BRIEF X closed but for one row. BRIEF XI blocked at three
              places — all three re-derived by me today from source:**

1. **Climb — the Agile Mech cannot.** `climb_target` (`sim.rs:11940`)
   skips any candidate where `!m.in_mech()`, i.e. **the TARGET must be a
   mech**, and then requires `m.mech_plates_dropped & zone.bit() != 0`.
   The climber's own gate is at the caller: `main.rs:23868` —
   `if !p.in_mech() && p.alive() && game.sim.climb_target(...).is_some()`.
   **Hull-climbing is a verb for a pilot ON FOOT against a stripped enemy
   hull.** It is something you do *to* a mech, not *in* one. BRIEF X §0's
   climb row was struck as FALSE in `553c425`. `TRV-0318`, `TRV-0326`.
2. **Crouch — `sim.rs` forbids it.** `set_crouch` (`sim.rs:4134`):
   `if self.in_heavy_mech() { self.crouch = want && self.grounded } else
   { self.crouch = want && !self.in_mech() }`. The scout chassis is not
   `in_heavy_mech`, falls to the else arm, and `!in_mech()` refuses it
   outright. **BRIEF XI §4 is not buildable as written.** `TRV-0325`.
3. **Pace — the client crouch is still infantry.**
   `MECH_CROUCH_HEIGHT_FRAC` (`sim.rs:6386`) is read once at `sim.rs:4172`
   and **zero times in `main.rs`**; `gait_pose` (`main.rs:2518`) still
   computes from the infantry ratio. `TRV-0034`, restated by the owner as
   BRIEF XI §4.

Next:         **OWNER RULING before any dispatch.** BRIEF XI §0.1 states
              the failure mode correctly and I am quoting it because it is
              the right instruction for every agent here: *"do not quietly
              animate a verb that never fires, and do not quietly skip it
              either."*

### THREAD 04 — armour damage states: still the largest built-and-invisible system
Rows: TRV-0036, 0106, 0133..0141, 0183, 0186, 0235
Re-derived today: `armor_stage_of` / `armor_wear_of` / `ArmorStage` →
**`sim.rs`: 66. `main.rs`: 0. `hud.rs`: 0.** A brand-new 1,742-line HUD
module was written this week and did not pick this up either.
State:        **PARTIAL, and the split is exact: the sim half shipped and
              the client half does not exist.** Fresh, Scuffed and Cracked
              render identically; only Severed shows, and only because it
              removes the piece. `ArmorStage::tilts` is a published
              accessor for a brief requirement with **no reader**.
Next:         friday33, one session. Fully specified in `TREVOR_TASKS.md`.

### THREAD 05 — the Royal: still a Big Mech ×1.10, and now measurably invisible
Rows: TRV-0010, 0013, 0014, 0015, 0370
First asked:  SPEC15 **P2** (arrow launcher) and **P3** (its own body)
Re-derived today: `arrow_launcher` / `ArrowLauncher` / `crossbow` across
all of `src/` → **two hits, both prose comments about a bow reading like a
crossbow in first person (`main.rs:396`, `:9648`). Zero in any weapon
path.** `MechWeapon` is Gatling / Autocannon / Rockets / Plasma / Repair.
**New, and worse (THOR, today).** The two Royals' hull luminance is ally
**0.0567** against foe **0.0563** — **1.007×**. The Agile's own guard test
demands **≥ 8×**. The strict form fails 6.7× the wrong way. There is **no
guard test on the Royal palette at all**, and the colours are inline
`Color::srgb` literals inside a `materials.add` block
(`main.rs:15380`/`:15398`), so they cannot be tested without being lifted
to consts first. **That lift is the prerequisite, not the recolour.**
State:        **NOT STARTED** for the arrow launcher — the last open P2 row
              — and **PARTIAL** for the body. Owner ruling Q4 made colour
              stop separating the two Royals, so the squint test is now the
              only separator, and it is unguarded.

### THREAD 06 — the invisible-system backlog nobody has touched in three days
Rows: TRV-0024, 0026, 0028, 0029, 0030, 0040
All re-derived today at `46d6dbb`:
- **25 `asset_server.load` calls: 21 `.wav` + 4 `.png`.** No `boom`, no
  explosion. The sim publishes `Boom` and nothing plays. *(The old
  blocker claim "all 21 loads are `.wav`, unblocker is any image loading
  at all" is **STRUCK** — `branding/key_art.png`, `wordmark.png`,
  `emblem.png`, `emblem_small.png` all load. Corrected in `7719296`.)*
- `FORGE_SLOTS` (`main.rs:1520`) — declared; its only other mention in the
  crate is `sim.rs:5389`, a doc comment calling it a known mistake. **No
  executable reader.**
- `TDM_TARGET_CHOICES` (`sim.rs:430`) — same shape: `sim.rs:5389` and
  `frontend.rs:321`, both doc comments. **No executable reader.**
- `struct CapBeat` (`main.rs:5531`) — still a compile-time const array
  inside a 29k-line file. ~6 min per framing tweak vs ~40 s.
State:        **NOT STARTED, all of them, with no owner blocker on any one
              and the unblocker named on day one.**

### THREAD 07 — traversal and maps: TWO BLOCKERS CLEARED
Rows: TRV-0043, 0051, 0117, 0149, 0180, 0374, 0376, 0377, 0379
**This is the good news of the run.** `research/maps/MAP_METRICS.md`
(1,025 lines) and `research/motion-architecture/DECISION.md` (775 lines)
were both written 2026-08-10 in `632824a`, *"The two orphaned research
threads finally produce their deliverables"*.
- `TRV-0051` traversal — **`BLOCKED` → `NOT STARTED`.** Its named blocker
  was `MAP_METRICS.md`. §4.3 "THE LEDGE BANDS" exists.
- `TRV-0180` ledge bands — **`BLOCKED` → `NOT STARTED`.** Same file.
- `TRV-0043` bot navigation — unblocked for the same reason.
**And `MAP_METRICS.md` corrected the record it was written from.** §6.1
re-ran the shipping validator rather than quoting the ledger:
Arena 80.2 → **102.9 m**, Bailey 93.4 → **120.2 m**, Gardens 92.0 →
**115.0 m**, Battlefield 509.9 → **637.4 m**, Cliffhold **577.1 m**.
*The drift is `MAP_SCALE`* — every old baseline was taken before the +25%
expansion. `SOURCES.md` is corrected.
It also names its own instrument's defect rather than hiding it: **the
sightline validator is flat-map-only and samples a grid at `half/10`**
(`sim.rs:19453`), so Battlefield quantises at ±35.4 m and Cliffhold at
±42.4 m. **Do not quote those figures to 0.1 m.** `TRV-0374`.
State:        the 40 m sightline rule is **unsatisfiable on every shipped
              map** and was retired by an AGENT's arithmetic. The
              arithmetic is right; the authority is wrong. One owner
              sentence closes it. `TRV-0117` / `TRV-0377`.
              The Bailey got its rebuild (`e7f408e`, +30%, randomised, and
              a chapel that was sitting on a pickup). Arena and Gardens did
              not. `TRV-0379`.

### THREAD 08 — the record about the record (BALANCE joins the operation)
Rows: TRV-0249, 0255, 0363..0366
`BALANCE_BOARD.md` / `BALANCE_LOG.md` are new agent files (`553c425`,
`8fb5228` — *"Balance's first board: the trap, the verdict, and an order I
am arguing against"*). Four of its findings are about **the record**,
which is my lane, so I have adopted them as rows:
- **S1** — Q4 is ratified, and two live comments still argue against it in
  the present tense. A builder reading either concludes the gold is an
  unresolved deviation. `TRV-0363`.
- **S3 — was CRITICAL.** The corrected `climb_target` comment lived only
  in an uncommitted working tree: *"one bad git command from being lost."*
  **It has since landed** — I verified the gate and both callers at
  `46d6dbb`. Recording the near-miss, because the near-miss is the record.
  `TRV-0364`.
- **S4 — my own error.** `TREVOR_TASKS.md` run 1 said *"friday22 and
  friday33. Those are the only two. A third builder has nowhere to go."*
  **False at HEAD: `git ls-files src/` returns thirteen modules.** It was
  idling builders. Corrected in this run's task file. `TRV-0365`.
- **S6** — `dd4fced`'s subject *"BRIEF XI: **finish** the Agile Mech"*
  ships 299 lines of markdown and zero code. Second commit this week whose
  subject marks an unbuilt brief as done. `git log --oneline` is the
  cheapest status instrument in the repo and it is being corrupted.
  `TRV-0366`.

### THREAD 09 — the missing images (unchanged, and now four more)
Rows: TRV-0260, 0261, 0309..0312, 0361, 0362
**Eight owner-supplied images. `git ls-files handback/reference/` returns
two files and both are `NOTES.md`.**
`TRV-0260`, the mech concept art, is **eleven days old** and remains the
oldest untouched ask in the ledger. `BRIEF_VIII_B` §D opens with *"The art
is the spec"* and §D.7 makes the side-by-side the mech section's **stated
completion criterion**. `PROMPT_mech_rebuild` Task 1 said why in plain
words: *"reference that lives only in a chat log is lost work."*
Four rows across two briefs (`TRV-0075`, `0100`, `0115`, `0172`) are
permanently unsatisfiable without it.
**The pattern is now understood and being mitigated, which is new.** The
HUD folder (`8bbb954`) wrote both images down in prose before they
arrived, and cited `TRV-0260` as the reason. Do that for the four
bow/spear images too and the class of failure closes.

---

## §B — EVERY ROW WHOSE STATUS MOVED TODAY

Fourteen. Each re-derived from code or from a committed artefact at
`46d6dbb`. Nothing here is carried on trust.

| Row | Was | Now | Evidence, today |
|---|---|---|---|
| `TRV-0303` | `UNVERIFIED` | **`DELIVERED`** | The uncommitted spear-charge work run 2 refused to bank has **landed** — `5ad7519`, *"The javelin wind becomes three seconds, and seven buys ten percent"*. `SPEAR_CHARGE_FULL_S` = 3.00, curve deliberately linear, shares argued in the comment. |
| `TRV-0304` | `UNVERIFIED` | **`DELIVERED`** | Same commit. `SPEAR_MAX_CHARGE_S` = 7.00, `SPEAR_MAX_CHARGE_DMG` = 1.10, resolved as a threshold on a **different axis** — velocity caps at 3 s, damage steps at 7 s. **That split is a builder's reading of the owner's words, not a fact about them**, and it is carried into the decision list as D12. |
| `TRV-0305` | `UNVERIFIED` | **`DELIVERED`** | Same commit; the checklist's damage bonus hangs off the charge. |
| `TRV-0302` | `NOT STARTED` | **`PARTIAL`** | **Which half shipped:** Thor verified no client path can start a charge from RMB — `cam.ads` is set from `aim_btn` alone (`main.rs:17922-17929`), consumed as `cmd.ads`, resolved through `aim_phase(true, cmd.ads, cmd.shoot)` (`sim.rs:8865`, `:8872`). **Which half did not:** `aim_phase`/`AimPhase` has **1** hit in `main.rs` and it is a comment (`:17655`) — **zero executable readers**. The HUD still derives state from raw timers. |
| `TRV-0306` | `NOT STARTED` | **`PARTIAL`** | The overhead javelin wind pose **exists in third person** — `javelin_wind_pose`, driven by the sim's `spear_wind_frac_of` at `main.rs:18705` — and **does not exist in first person**, where `main.rs:21536` hand-rolls off the release clock. Thor proved the FP half in pixels. |
| `TRV-0308` | `PARTIAL` | `PARTIAL` — **evidence upgraded to a photograph** | The FP viewmodel half holds for both weapons and is now photographed clean. **The obstruction is not the viewmodel:** `spear_fp/02-fp-spear-preaim.png` shows the arc preview's landing ring **on the exact crosshair pixel** with a red range readout directly under it, against `01-fp-spear-idle.png`'s clean `[ + ]`. The screen-intrusion sweep measures `weapon_parts` geometry only (`weapon_bounded_corners`, `main.rs:3096`) and **cannot see a world-space ring or a UI text node**. |
| `TRV-0317` | `NOT STARTED` | **`PARTIAL`** | Run 2's *"no capture in this repository holds RMB with a bow or a spear"* is **half closed**: `spear_fp/02-fp-spear-preaim.png` exists and it is the frame that settled C1. **`BOW_DRAW_FP_BEATS` (`main.rs:6294`) is still Mouse-Left-only, so no bow pre-aim frame exists.** |
| `TRV-0294` | `CONTRADICTED` | `CONTRADICTED` — **now photographed, and urgent** | `fn arc_preview` (`main.rs:21657`) still gates on `let show = cam_ctl.ads && p.alive() && spec.projectile.is_some() && p.roll_t <= 0.0;`. Exactly two guns declare `projectile: Some(..)` — Bow and Spear, the two weapons the ban names. RMB is now formally PRE-AIM, so **the forbidden arc is bound to the very button the newest spec elevates**, and there is finally a frame of it. |
| `TRV-0051` | `BLOCKED` | **`NOT STARTED`** | Its named blocker was `MAP_METRICS.md`. That file exists — 1,025 lines, `632824a` — with §4.3 "THE LEDGE BANDS". |
| `TRV-0180` | `BLOCKED` | **`NOT STARTED`** | Same file, same section. |
| `TRV-0149` | `NOT STARTED` | **`DELIVERED`** | `research/maps/MAP_METRICS.md`, `632824a`. And it **corrected the document it was derived from** — see THREAD 07. |
| `TRV-0188`, `0189`, `0190` | open | **`DELIVERED`** *(closed run 2, re-verified today)* | `research/motion-architecture/DECISION.md`, 775 lines, `632824a`. The deliverable that was never written is written, and BRIEF XI §0.2 makes item 3 binding. |
| `TRV-0055` | `NOT STARTED` | **`PARTIAL`** | **Which half shipped:** `hud.rs:190 fn heat_pct(gatling_heat, scout_chassis)` now **names the bug in its own doc comment** and gives the HUD one policy. **Which half did not:** `main.rs:22825` still prints `HEAT {:.0}%` from the **raw** value while `:22807`/`:22813`/`:22816` print `× 100` under the same `%`. The sim-side split is untouched. |
| `TRV-0289`, `TRV-0206` | — | unchanged `DELIVERED` | Re-confirmed: twelve `agile_moves` PNGs on disk; `the_enemy_agile_never_out_luminates_the_ally` at `agile_mech.rs:744`. |

### And one row the dispatch told me had moved — which HAS NOT

> Dispatch: *"The Field Manual score lie appears fixed — the constant it
> misread is gone from that path. Confirm."*

**It is not fixed. `TRV-0031` stands, and it is now demonstrably wrong.**
`main.rs:25489`:

```rust
let modes = format!(
    "TDM first to {:.0} - KOTH hold the center {:.0} s -\n\
     {:.0}-min clock, {:.0} s sudden-death overtime.",
    TDM_TARGET,          // <-- the CONSTANT, not game.sim.cfg.tdm_target
```

Every other number on that screen *was* correctly moved to live constants
— the comment twelve lines above says so in the code's own words: *"Every
number below is the LIVE constant, never a retyped copy. The old prose
hardcoded all of them."* This one line was missed. It reads
`sim::TDM_TARGET` = 30 while `frontend::INTRO_TDM_TARGET` = 25 is what an
introductory match actually plays to (`main.rs:7657`). **The screen tells
a player in the intro match the wrong target.** One line, friday33.

---

## §C — ROWS OPENED THIS RUN — `TRV-0345`..`TRV-0379`

All `Last checked: 2026-08-11`, all derived at `46d6dbb`.

### C.1 — BRIEF XII + XII-A, the HUD (origin: owner)

| ID | Ask | Issued at | Layer | Status | Evidence |
|---|---|---|---|---|---|
| `TRV-0345` | The HUD must read differently as a HUMAN and as a MECH — *"one layout discipline at two densities"* | `BRIEF_XII_hud_human_vs_mech.md` | cosmetic | **DELIVERED** | `src/hud.rs`, 1742 lines, `1713fe1` — *"the HUD stops being one layout with two extra numbers"* |
| `TRV-0346` | §1 reuse the existing visual language; do not invent a second | BRIEF XII §1 | cosmetic | **DELIVERED** | `frontend::palette` GOLD/INK/INK_SOFT/NEON_BLUE/NEON_RED + the `T_TITLE..T_MICRO` ramp, reused |
| `TRV-0347` | One heat readout, not three | XII-A | cosmetic | **DELIVERED** | `hud.rs:190 heat_pct`; `2ad5d42` — *"heat three times, hull twice, and a strip that vanishes on sand"* |
| `TRV-0348` | One hull readout, not two | XII-A | cosmetic | **DELIVERED** | `85e0667` — *"one heat, one hull, and the strip folded into the HUD"* |
| `TRV-0349` | The weapon strip must not vanish on sand | XII-A | cosmetic | **DELIVERED** | `2ad5d42` |
| `TRV-0350` | Delete the legacy `weapon_strip` and `shield_readout` | XII-A | cosmetic | **DELIVERED (contested)** | `git grep` finds 1 residual mention in `hud.rs`, 1 in `cockpit.rs`, 5 in `main.rs`. I read none of the seven. **Contested until someone confirms all seven are prose and not a live path.** → Thor |
| `TRV-0351` | The selected mount must stop printing its number underneath it | owner-reported; `44115e7` | cosmetic | **DELIVERED** | `44115e7` |
| `TRV-0352` | The armoury pills were a fixed height, so half the plate names fell out | owner-reported; `7218e37` | cosmetic | **DELIVERED** | `7218e37` |
| `TRV-0353` | The pill padding cost the soldier page two lines it did not have | `1515ac9` | cosmetic | **DELIVERED** | `1515ac9` |
| `TRV-0354` | §8 the HUD audit — every screen tells the truth | BRIEF XII §8 | cosmetic | **PARTIAL** | Consolidation shipped. **Five of the six honesty defects the audit exists to find are still open** (`TRV-0031`, `0032`, `0033`, `0024`, `0028`); `TRV-0055` is half-done. |
| `TRV-0355` | Rule 1: *"the centre of the screen is never touched"* | `handback/reference/hud/NOTES.md` | cosmetic | **PARTIAL** | Viewmodel enforced and tested; **the arc preview's landing ring violates it.** duplicate-of `TRV-0308` |
| `TRV-0356` | Rule 2-3: health and ammo the two biggest glyphs in opposite bottom corners; current/reserve a **big/small pair**, never equal weight | ibid | cosmetic | **UNVERIFIED** | **I opened no HUD PNG this run.** Stated, not checked. |
| `TRV-0357` | Rule 4: transient messages get little or no panel; permanent readings get one | ibid | cosmetic | **UNVERIFIED** | ditto |
| `TRV-0358` | Rule 5: *"objectives belong in the world, not in a list"* | ibid | sim+cosmetic | **NOT STARTED** | KOTH has one hill; no world-space objective marker symbol exists |
| `TRV-0359` | Rule 6: colour carries meaning — one hue for threat, one for systems | ibid | cosmetic | **PARTIAL** | `NEON_RED`/`NEON_BLUE` exist as palette data; **no test pins the semantics**, so a repaint can silently break it — the same shape as SPEC15 trap 4 |
| `TRV-0360` | Mech POV gets targeting / heat / integrity / lock; human POV gets none of them | ibid, the density table | cosmetic | **PARTIAL** | Heat, hull and integrity shipped in `hud.rs`. **Lock and targeting: no symbol in `src/`.** |
| `TRV-0361` | **Reference image 1 — save as `img1-mech-pov.png`.** Modern military shooter (grey sky, glass office block, iron sights, red *"ENEMY CONTROLS ALMOST ALL SECTORS"* banner). **This is the MECH POV reference.** | owner, chat; folder created `8bbb954` | asset | **BLOCKED (owner)** | `git ls-files handback/reference/hud/` returns **`NOTES.md` and nothing else** |
| `TRV-0362` | **Reference image 2 — save as `img2-human-pov.png`.** Stylized cartoon shooter (desert town, wooden CORNWELL / JENKIN COAL buildings, shotgun, red engineer portrait). **This is the HUMAN POV reference.** | ibid | asset | **BLOCKED (owner)** | ditto. duplicate-of the `TRV-0260` failure mode |

**On `TRV-0361`/`0362`, and it belongs in the record:** these are the
best-handled missing images in this project's history. `8bbb954` — *"The
HUD reference folder, with both images written down before they arrive"* —
created the folder, described both images in words while they were still
on screen, declared the description the working reference until the PNGs
land, and cited `TRV-0260` as the reason. **A builder is not blocked. The
acceptance check is.** That distinction is worth copying to the four
bow/spear images (`TRV-0309`..`0312`), where it has not been done.

### C.2 — Findings adopted from BALANCE (origin: agent; about the record)

| ID | Finding | Issued at | Layer | Status | Evidence |
|---|---|---|---|---|---|
| `TRV-0363` | Q4 is ratified and **two live comments still argue against it in the present tense** | `BALANCE_BOARD.md` §4 S1 | doc | **NOT STARTED** | `main.rs:15301` *"asks for the player Royal to carry SUBTLE NEON-BLUE energy"*; `main.rs:4005` *"why it is not the spec's neon blue"*. A builder reading either concludes the gold is an unresolved deviation. **It is ratified.** |
| `TRV-0364` | The `climb_target` correction lived only in an uncommitted tree — *"one bad git command from being lost"* | ibid §4 S3 | doc | **DELIVERED** | **It landed.** I verified the gate (`sim.rs:11944`) and both callers (`main.rs:23868`, `sim.rs:9188`) at `46d6dbb`. Recording the near-miss, not a defect. |
| `TRV-0365` | **My own run-1 lane map was FALSE**: *"friday22 and friday33. Those are the only two. A third builder has nowhere to go."* | ibid §4 S4, against `TREVOR_TASKS.md:18-21` | doc | **DELIVERED** | `git ls-files src/` returns **13 modules**. The claim was idling builders. Corrected in this run's task file. |
| `TRV-0366` | Commit subjects mark unbuilt briefs as done — `dd4fced` *"BRIEF XI: **finish** the Agile Mech"* adds one file and changes no code | ibid §4 S6 | doc | **NOT STARTED** | Second occurrence this week. `git log --oneline` is what every new session reads and it is the cheapest status instrument in the repo. |

### C.3 — Findings adopted from THOR's bow/spear verdict (origin: agent)

| ID | Finding | Layer | Status | Evidence |
|---|---|---|---|---|
| `TRV-0367` | **There is no first-person charge pose, and the doc comment that says otherwise is the dangerous part.** `main.rs:21536` reads `1.0 - p.spear_wind_t / SPEAR_WINDUP_S` — the RELEASE clock, which is 0 for the entire charge | cosmetic | **NOT STARTED** | Proved in pixels: `spear_fp/02` and `/04` place the spear head at approx (1045, 535) in **both**, while the HUD reads `JAVELIN FULL`. `/05-plant` moves — the signature of a release-keyed pose. A second copy survives at `main.rs:1403` (`torso_coil_yaw`). `spear_stance_of` and `spear_max_charged_of` have ZERO client readers. |
| `TRV-0368` | **The flagship pose of the whole pass has never been photographed.** `SPEAR_WIND_BEATS` (`main.rs:6011`) is registered as `"spear_wind"` (`:6970`) and the output directory does not exist | cosmetic | **NOT STARTED** | Confirmed by me: no `handback/brief-vii/spear_wind/` in `git ls-files`. The 8 named snaps include `02-preaim-not-a-wind`, **the builder's own stated proof for half of input rule 2**. Not verified, not disproven — **never checked.** |
| `TRV-0369` | **The mech-entry charge leak, recorded a third time.** `sim.rs:8103` (Scout) and `:8136` (Robot\|Royal) clear heat, vent, plasma charge and dropped plates on boarding and never touch `bow_draw_t`, `spear_charge_t`, `spear_power`, `spear_max_charge` | sim | **NOT STARTED** | Board mid-draw, dismount, and an arrow leaves — or a javelin at 1.3× still carrying the +10% flag. The dismount teardown (`sim.rs:7563`) clears `mech_jump_phase` **for precisely this argument, in a comment that says so.** Fix it at the ENTRY, not the exit. |
| `TRV-0370` | **The two Royals separate by 1.007×, and nothing can test it.** ally hull 0.0567 vs foe hull 0.0563; the Agile's guard demands ≥ 8× | cosmetic | **NOT STARTED** | `main.rs:15380`/`:15398` are inline `Color::srgb` literals inside a `materials.add` block. **Lifting the six literals to consts is the actual prerequisite, not the recolour.** Strict form: brightest foe 0.1196 vs dimmest ally 0.0178 — 6.7× the wrong way. |
| `TRV-0371` | **Both pre-aim tests hard-code the production `ads_shift` instead of reading it** | cosmetic | **NOT STARTED** | `Vec3::new(0.050,0.020,0.050)` / `(0.020,0.010,0.060)` at `main.rs:21665-21668` are re-typed at `:26278` and `:26346`. Change production and both tests still pass, having verified a vector the game no longer uses. **OPERATION rule 12.** |
| `TRV-0372` | Hands ARE modelled on the bow in the shipped frame, and **there is no grip/nock anchor to build against** | cosmetic | **PARTIAL** | `weapon_hand_specs` (`main.rs:4853`) returns a hand for Bow and Spear; `bow_draw_fp/03` shows a fully-fingered glove on the riser. What exists is `GRIP_WINDOW_M` (a test window, `:3026`) and `bow_nock_local()` (`:446`) — **a function, not a socket.** If the owner is building hands separately he collides with this. |

### C.4 — Research, assets and the rest

| ID | Ask / finding | Issued at | Origin | Layer | Status | Evidence |
|---|---|---|---|---|---|---|
| `TRV-0373` | `DECISION.md` §9 specifies **BM-1**, a pose-kernel microbenchmark, as the instrument that closes axis 5 — *"measurement beats a citation"* | `motion-architecture/DECISION.md` §9.2 | agent | sim | **NOT STARTED** | The spec is written in full; BM-2 is ~1 hour on the existing crowd bench. **No benchmark has been run.** The decision was made without its own named instrument, and §9.4 says so honestly. |
| `TRV-0374` | The sightline validator is **flat-map-only and grid-quantised** — Battlefield ±35.4 m, Cliffhold ±42.4 m | `maps/MAP_METRICS.md` §6.2, marked ⚠ *"a defect, not a caveat"* | agent | sim | **NOT STARTED** | `sim.rs:19453` samples at `half/10`. **Do not quote the sightline figures to 0.1 m.** |
| `TRV-0375` | `src/muzzle_flash.rs` (548 lines) and its capture directory | commits + working tree | agent | cosmetic | **PARTIAL** | The **module is committed**. `handback/brief-vii/muzzle_flash/` is **untracked** and exists only on this disk. A bare `git stash` or `git checkout` destroys it. `OPERATION` rule 7b. |
| `TRV-0376` | `DECISION.md` §7 MIGRATION PATH — adopt it, in what order, or leave the closed-form poser alone | `DECISION.md` §5-§7 | agent→owner | sim+cosmetic | **BLOCKED (owner)** | Five families rejected on named axes (motion matching, learned/LMM, two Bevy crates, active ragdoll); §0 gives the decision in one paragraph; §7 gives the path. **Nobody has ruled whether to start it.** |
| `TRV-0377` | The IX-A **40 m sightline rule** needs a replacement number or a formal retirement | `BRIEF_IX-A` vs `MAP_METRICS.md` §6 | owner | sim | **BLOCKED (owner)** | All five maps fail: Arena 102.9, Bailey 120.2, Gardens 115.0, Cliffhold 577.1, Battlefield 637.4 m. Unsatisfiable above ~15 m half-extent. **Retired by an agent's arithmetic — the arithmetic is right and the authority is wrong.** |
| `TRV-0378` | *"The outbuildings were dead code, and a mutation is the only reason I know"* | commit `9e5ff59` | agent | cosmetic | **DELIVERED** | Dead code removed **and mutation-proven**. OPERATION rules 9 and 12 honoured in one commit. Recorded as the pattern to copy, not as an open item. |
| `TRV-0379` | The Bailey rebuild — *"30% bigger, randomised, and the chapel was sitting on a pickup"* | commit `e7f408e`, against 0-QUEUE Tier 3 #20 | owner | sim | **PARTIAL** | **One** of the three core maps got the size/elevation/randomisation pass. **Arena and Gardens did not.** The spawn row is still 8 wide (`d1e9e06`). **The seeding trap still applies:** randomised structures must be seeded at map-BUILD time from the match seed, or every later draw shifts and replay breaks for every other system. → Thor: which stream does the Bailey draw from? |

---

## §D — THE RESEARCH INDEX

11 topic directories, `SOURCES.md`, `TOTO_LOG.md`, and five agent logs.
Categorised on the charter's three axes.

### D.1 Topic → thread → deliverable → consumed?

| Topic dir | Thread | Deliverable written? | Consumed — and by what constant? |
|---|---|---|---|
| `maps/` | 07 | **`MAP_METRICS.md`, 1025 lines, `632824a`** | **Partly.** It *corrected* `SOURCES.md`'s stale sightline baselines and unblocked two rows. **No value from it has reached a constant yet.** |
| `motion-architecture/` | 03 | **`DECISION.md`, 775 lines, `632824a`** | **YES, bindingly.** BRIEF XI §0.2 elevates item 3 (generalise `solve_arm_ik` to the legs; no second solver, no crate) to a build constraint. That is `TRV-0319`, still `NOT STARTED`. |
| `mech-entry/` | 05 | `CYCLE_1_REPORT.md` | YES — `mech_enter_stage`'s eight committal stages |
| `mech-climb/` | 03, 07 | `DESIGN.md`, `CYCLE_3_REPORT.md` | **Contested.** The design exists; the verb it designs for **does not exist for a mech** (THREAD 03). |
| `grenade/` | 02 | `CYCLE_2_REPORT.md` | YES — fuse, falloff, bounce |
| `armor-damage/` | 04 | `SOURCES.md` | **Half.** The concave degradation curve reached `sim.rs`. **Zero of it reached `main.rs` or `hud.rs`.** |
| `body-rig/` | 03 | `SPEC_20_SEGMENT_RIG.md` | YES — the 20-segment rig. Fingers **deliberately omitted and documented as such** (`main.rs:852`). A decision, not a gap — but §8's sub-rig it defers to does not exist (`TRV-0329`). |
| `spear-throw/` | 01 | `SOURCES.md` | YES — charge curve and flight physics |
| `traversal/` | 07 | `SOURCES.md` | **ORPHANED — and, as of `632824a`, no longer blocked.** |
| `aiming/` | 01, 05 | `SOURCES.md` | **ORPHANED.** No value from it names a constant anywhere in `src/`. |
| `vertical-maps/` | 07 | `SOURCES.md` (TOTO33 — the only non-zero Tier-V) | **ORPHANED.** |

### D.2 ORPHANED RESEARCH — work already paid for that a builder could use TODAY

**This is my highest-value output, and this week is the first time the list
has got SHORTER.**

1. **`maps/MAP_METRICS.md` §4.3 THE LEDGE BANDS** — measured jump apexes,
   step heights, drop rules, and the band-separation rule the code asserts
   *plus the stronger one it needs*. **Two rows (`TRV-0051`, `TRV-0180`)
   were BLOCKED on this exact file and are now merely NOT STARTED.**
   Nobody has been told. Also §3.1 corridor widths, §3.3 the three cover
   tiers that already ship, §5 stairs and ramps.
2. **`motion-architecture/DECISION.md` §9.2 BM-1** — a fully specified
   pose-kernel microbenchmark, plus BM-2 at ~1 hour on the existing crowd
   bench. `TRV-0373`. And §5's five rejections, each with a named axis,
   which save a future session from re-litigating two Bevy crates.
3. **`vertical-maps/SOURCES.md`** — the only Tier-V (video/talk) source
   this project has ever landed. It serves a thread nobody is working.
4. **`aiming/SOURCES.md`** — never consumed by anything, ever.
5. **`traversal/SOURCES.md`** — same, and now unblocked.

### D.3 The inverse — constants with no research behind them, where a real number was required

- **`SCOUT_SCALE = 1.05`** — frozen in two briefs pending an owner ruling.
  Authored by feel, wearing a researched value's clothes. `TRV-0059`.
- **`SPEAR_MAX_CHARGE_DMG = 1.10` and the 3 s / 7 s split** — the builder
  argued it in a comment, honestly, but the spec's two sentences cannot
  both hold on one ramp, so **the resolution is a reading of the owner's
  words, not a measurement.** `TRV-0304` → decision D12.
- **The Royal palette's six inline `Color::srgb` literals** — not consts,
  therefore not testable, therefore 1.007× separation went unnoticed for
  days. `TRV-0370`.
- **The arrow launcher's rate of fire, bolt velocity and per-bolt damage
  against a mech hull** — the one genuine toto number in the file.
  `TRV-0010`.

### D.4 Tier and solidity

Per `SOURCES.md`'s own honest counts, the structure is unchanged: most
extracted values are **DERIVED** (a cited source plus this game's own
arithmetic), a minority are **MEASURED** (in engine, by a validator), and
**Tier-V remains one source project-wide.**

`MAP_METRICS.md` is the first document here to label every row
MEASURED / DERIVED / ASSUMED inline **and** to publish its own
instrument's precision ceiling and its own instrument's defect. **That is
the standard the other ten `SOURCES.md` should be held to**, and it is a
cheap doc task for anyone idle.

---

## §E — WHO NEEDS WHAT

### E.1 Rows for THOR — claimed done, evidence thin or contested

- `TRV-0345`..`0353` — the nine HUD consolidation rows are `DELIVERED` on
  **commit evidence, and I opened no HUD PNG.** A 1,742-line rewrite of
  the thing the player looks at, unphotographed by me.
- `TRV-0350` — five residual `weapon_strip`/`shield_readout` mentions in
  `main.rs`. Prose, or a live path?
- `TRV-0356`, `TRV-0357` — `UNVERIFIED` for the same reason.
- `TRV-0379` — **the Bailey randomisation.** Is it seeded at map-BUILD
  time from the match seed, or drawn from the gameplay RNG stream? If the
  latter, replay is broken for every other system and nobody will notice
  for weeks.
- `TRV-0371` — Thor's own finding: two pre-aim tests that cannot fail.
- `TRV-0008` — the recoil envelope, still the biggest feel change in the
  game with **no capture**.
- Carried from run 1, still unchecked: `TRV-0035`, `0054`, `0057`, `0134`,
  `0237`, `0250`, `0252`, `0253`.

### E.2 Rows for TOTO — blocked on a number nobody has

Rule 13: `toto*` only when a specific unknown **NUMBER** blocks a build,
and it must be named in the dispatch. **Two rows qualify. Still two.**

- **`TRV-0010`** — the Royal arrow launcher's rate of fire, bolt velocity,
  and per-bolt damage against a mech hull. Nothing in the game fires a
  bolt from a mount, so there is no neighbouring value to interpolate
  from. Everything else about that weapon is art.
- **`TRV-0126`** — "how enclosed is this point". **The METHOD is the
  unknown**, not a coefficient. Ask for the method (ray fan? nearest-wall
  distance? cell occupancy?) and its cost at 120 Hz. Do not ask for a
  percentage.

`TRV-0373` (BM-1) looks like research and is not: it is a **benchmark**,
it belongs to friday22, and `DECISION.md` §9.1 argues that point at
length.

### E.3 What I could not check, plainly

Stated as a section rather than buried, because implying a complete sweep
is the failure this role exists to prevent.

- **I did not build and did not run the suite.** Read-only; never invoked
  `cargo`. Every `DELIVERED` means "there is evidence at this path", never
  "it compiles" and never "it works". The last reported figure is Thor's,
  re-run by him: 478 pass / 0 fail / 2 ignored at `1713fe1`.
- **I opened ZERO PNGs.** There are 188 committed captures plus an
  untracked `muzzle_flash/` directory. `TRV-0356` and `TRV-0357` are
  `UNVERIFIED` for exactly this reason and say so in their own cells. Every
  pixel measurement in this file is **Thor's**, and is attributed to him.
- **I did not re-derive the 330 rows absent from §B.** Their run-2
  statuses stand at their run-2 date. Stated at the top of this file so
  nobody reads a 2026-08-11 header as a 2026-08-11 check.
- **I derived the BRIEF XII / XII-A rows from the commits and from
  `handback/reference/hud/NOTES.md`, NOT from the brief text.** That is a
  weaker derivation than reading the briefs and I am saying so rather than
  letting nineteen fresh rows look equally solid.
- **Thirteen of nineteen bow/spear sections remain invisible** (`TRV-0314`),
  as do FRONT END P7+ (`TRV-0297`). Not guesses, and must not be turned
  into guesses.
- **Read whole this run:** `TREVOR_TASKS.md`, `TREVOR_LOG.md`,
  `ISSUED_VS_DELIVERED.md`, `handback/reference/hud/NOTES.md`,
  `THOR_LOG.md`'s newest entry in full, `BALANCE_BOARD.md` (outline plus
  §4 and §6), `MAP_METRICS.md` §6.1-§6.2, the heading structure of
  `DECISION.md` and `MAP_METRICS.md`, run 1's `TREVOR_LEDGER.md`.
- **Sampled by targeted grep + region reads:** `sim.rs` (`climb_target`,
  `set_crouch`, `height`, `step_up`, the `!in_mech` gates), `main.rs`
  (`arc_preview`, the Field Manual `modes` block, `BIND_REGISTRY`,
  `gatling_heat`, `CapBeat`, the asset load list, `aim_phase`), `hud.rs`,
  `agile_mech.rs`, `frontend.rs`.
- **Not read at all:** `FRIDAY_LOG.md` (1,620 lines — **and run 3's own
  note said to read it first; I did not, again**), `TOTO_LOG.md`, the
  bodies of `DECISION.md` and `MAP_METRICS.md` beyond the cited sections,
  the briefs XII and XII-A themselves, the eleven per-topic `SOURCES.md`,
  `WHATS_MISSING.md` this run, everything outside `jk_tdm`.
- **Concurrency.** `main.rs` and `muzzle_flash.rs` were modified in the
  tree throughout. Line numbers were true at `46d6dbb` with that tree.
  **Every row also names its symbol; re-anchor on the symbol, `git fetch`
  first.**

---

*Written by TREVOR, run 4. I do not edit source, briefs, or another
agent's log. Where a document is wrong I record the disagreement here and
hand it over. Where I was wrong — `TRV-0365`, the two-lane claim that
idled a builder — §C.2 says so before anything else.*
