# Trevor's log — the archivist's record

**Append-only. Never rewritten.** One dated entry per run: IDs opened, IDs
whose status moved and why, IDs I could not check, and what I got wrong
last time. This is my memory across sessions — I will not remember today
tomorrow.

The ledger (`TREVOR_LEDGER.md`) is current truth. This file is history.
The task file (`TREVOR_TASKS.md`) is the deliverable. All three are
rewritten or appended each run; the ledger and tasks are rebuilt in full,
this file only ever grows.

Standing rules live in `.claude/agents/trevor.md`.

---

## 2026-08-10 — RUN 1. The data bank, from zero.

Repo at start: `e2866a9`. Repo at end: `f10be3a` — **two commits landed
from a concurrent session while I swept.** Working tree at end: modified
`.claude/agents/thor.md` and `research/OPERATION.md`, untracked
`.claude/agents/trevor.md`. None of those are mine and I touched none of
them.

### What the owner asked for

> *"create all the task that needs to be worked on, check every prompt i
> have inputed, projects that are undone or needs to be implemented, also
> add trevor database, also the research information to be categorized as
> well."*

Four deliverables. Three files written: `TREVOR_LEDGER.md` (the data bank
— rows, threads, research index), `TREVOR_TASKS.md` (the ranked,
lane-assigned work — the one they will actually open), and this log. The
research categorisation lives inside the ledger at §C, on the charter's
three axes.

### IDs OPENED — all 266 of them

`TRV-0001` .. `TRV-0266`. First run, so everything is new. No gaps, no
duplicates — verified mechanically by extracting every `^| TRV-####` and
checking the sequence.

Allocation, and I chose it so the blocks stay meaningful forever:

| Block | Source | Count |
|---|---|---|
| 0001-0021 | `WHATS_MISSING.md` §0-SPEC15 (owner's 15-section mech spec, P1-P4 + code quality) | 21 |
| 0022-0053 | §0-QUEUE Tiers 0-4 | 32 |
| 0054-0065 | §0-NOW (the post-six-agent list) | 12 |
| 0066-0077 | `BRIEF_VII_optimized.md` | 12 |
| 0078-0100 | `BRIEF_VIII_master.md` | 23 |
| 0101-0115 | `BRIEF_VIII_B_addendum.md` | 15 |
| 0116-0141 | `BRIEF_IX_castle_grenade_customization.md` (A/B/C) | 26 |
| 0142-0157 | `PROMPT_MASTER_research_build.md` — **one row per numbered task, 13 tasks, plus R9 and the branch** | 16 |
| 0158-0166 | `PROMPT_brief_X_research.md` (superseded — indexed, not voided) | 9 |
| 0167-0173 | `PROMPT_mech_rebuild.md` (superseded — indexed) | 7 |
| 0174-0187 | `PROMPT_RND_CYCLE.md` + `BACKLOG.md` | 14 |
| 0188-0190 | `PROMPT_motion_system_research.md` | 3 |
| 0191-0234 | **Chat asks recorded second-hand** — `§owner` doc comments, log quotes, commit messages, README lines | 44 |
| 0235-0255 | Agent-origin: Friday's stated deferrals, Friday's least-sure items, Thor's findings, the scouts' | 21 |
| 0256-0266 | Images — uploaded reference, missing reference, generated artefacts | 11 |

**The block that surprised me is 0191-0234.** Forty-four owner asks exist
nowhere in any brief. They survive only because somebody wrote `§owner`
in a doc comment next to the code. That convention is the single best
archival practice in this repo and I have said so in the task file.

### Headline

266 rows · 111 `DELIVERED` (4 contested) · 43 `PARTIAL` · 61
`NOT STARTED` · 17 `BLOCKED` · 13 `SUPERSEDED` · **21 `UNVERIFIED`**.
Open = 104. Origin: 231 owner, 29 agent, 6 agent-found-then-adopted.

### IDs whose status MOVED — against what the record claimed

This is the point of the run. Every one of these was re-derived against
the code today; none was carried on trust.

**Moved to DELIVERED (the plan said open):**

- `TRV-0022` `ROBOT_SPEED_MULT` — **deleted.** `sim.rs:437` now holds a
  `§owner (defect pass)` comment where the constant was.
- `TRV-0025` `pod_aim_held` — **deleted.** Same pattern, `sim.rs:3394`.
- `TRV-0023` `MECH_SHIELD_ARC_COS` — **wired.** `sim.rs:12841`
  `barrier_arc = cos > MECH_SHIELD_ARC_COS;` and `:12828` records that the
  bare literal had exactly one site.
- `TRV-0058` `SCOUT_SCALE` — **wired into `height()`** via
  `ArmorSet::chassis_scale` (`sim.rs:4893`), with a test at `:15743`. The
  0-NOW claim "read by nothing" is closed. (Its *value* question is a
  different row — `TRV-0059`, blocked on the owner.)
- `TRV-0142` Task 0's five before-clips — `TASK0_AUDIT.md` says the
  traversal clip (c) and the map lap (e) are missing. **Both exist now**:
  `traversal/01..04.png`, `map_lap/01..07.png`.
- `TRV-0177` melee depth — `BACKLOG.md` #4 says "Not started". **False.**
  Melee v2 shipped `a99af96` with directional swing lines, line-matched
  parry, and three captures.
- `TRV-0178` AI retreat — `BACKLOG.md` #5 says retreat "is the
  remainder". **Stale.** Shipped `e5431a4` with hysteretic fear,
  class-derived `ROUT_TOLERANCE`, and a test at `sim.rs:17320`.
- `TRV-0186` per-piece armour geometry — the backlog's "stripping a
  gauntlet changes nothing" was already corrected by the gap scout;
  recorded as a row so it cannot be re-opened by a future sweep.
- `TRV-0134` the armour-weight formula — `BACKLOG.md` #11 calls it
  "unwired". **False.** Wired in the 24-plate pass with a live
  weight-against-ceiling readout.
- `TRV-0238` — **this one moved while I was writing.** Friday's last log
  entry (2026-08-10) states `mount_kick_axes` and `turret_chaos_of` are
  "published for the visual half; nothing reads them yet". Commit
  `fe07c19` landed mid-run and `mech_recoil.rs:97-119`
  (`SWING_RAD_PER_DEG`) now reads `punch[1]`. A stated deferral closed by
  another session inside my own sweep window.

**Moved to NOT STARTED / confirmed still open (I re-derived rather than
assumed):**

- `TRV-0024` `TDM_TARGET_CHOICES` — still declared, still unread. Its only
  other mention in the crate is a doc comment at `sim.rs:4922` naming it
  as a known mistake.
- `TRV-0028` `FORGE_SLOTS` — still declared at `main.rs:1372`, no reader.
- `TRV-0026` `shot_handgun.wav` — generated by `gen_sfx.py:73`, on disk,
  loaded by nothing.
- `TRV-0030` explosions are still silent. `main.rs:14096-14116` is the
  complete `Sfx` list: 20 wavs, no boom.
- `TRV-0031` the Field Manual still prints the constant `TDM_TARGET`
  (`main.rs:24303`) and not the chosen target — while every other number
  on that screen was correctly moved to live constants.
- `TRV-0032` `BIND_REGISTRY`'s only `U` row is "Dismount the mech".
- `TRV-0033` the `Q` row still omits the medic's second flip and air jump.
- `TRV-0036` **zero** client references to `armor_stage_of`,
  `armor_wear_of` or `ArmorStage`. The whole damage-state feature is
  invisible.
- `TRV-0040` capture beats are still compile-time consts.
- `TRV-0055` `gatling_heat` still carries two scales — three call sites
  print `×100` under a `%` and one prints raw under the same `%`.
- `TRV-0010` the Royal arrow launcher — grepped `crossbow`,
  `arrow_launcher`, `ArrowLauncher` across the whole tree: **zero hits in
  any weapon path.** This is the last open P1/P2 row.

**Moved to PARTIAL (the plan overstated in one direction or the other):**

- `TRV-0037` mech boarding — the plan says "7 render a `debug!` line".
  Half right now: the client *does* read `mech_enter_stage_for`
  (`main.rs:19198`), plays a rising `click` per beat, and drives
  `visor_ready`. But `:19243-19254` is still eight `debug!` arms.
  Sequencing landed; presentation did not.
- `TRV-0057` `visor_ready` — the claim survives. It is still a field of a
  `Local<MechStageState>`, written three times and read by nothing outside
  its own system. What changed is that it is now correct and tested
  (`visor_ready_after` is pure; `main.rs:25838`). Correct, and still
  unreachable.
- `TRV-0039` Cliffhold — client half gone (`4152240`), **49 references
  survive in `sim.rs`** including `build_cliffhold` and five tests.
- `TRV-0043` bot navigation — BOT ROUTING genuinely landed for Cliffhold
  with published up-links, a named `BOT_PROBE_Y`, and five tests; but
  `sim.rs:27649` states the flat maps were deliberately left alone.
- `TRV-0049`/`TRV-0224` texture pipeline — see the correction below.
- `TRV-0013`/`0014`/`0015` the Royal and the opposition — **paint landed,
  geometry did not.** `main.rs:11414` says it in the code's own words:
  "Same machine, same 53 plates."

### The correction I am most confident about, and it corrects two documents

`BACKLOG.md` #12 and `research/SOURCES.md` both state, as the blocker for
an entire tier of work: *"Zero texture pipeline. **All 21
`asset_server.load` calls are `.wav`.** Unblocker: any image loading at
all."*

**That has been false since `03085b1` on 2026-08-03.** I counted the load
calls: **24**, of which 20 are `.wav` and **4 are `.png`**
(`branding.rs:303-306`, the key art, wordmark and two emblems). Image
loading exists.

The true remaining blocker is narrower and more useful, and I have written
it that way in `TRV-0048` / `TRV-0049`: **no imported image reaches a
WORLD material, and `jk_tdm` has no glTF loader at all.** The 2026-08-08
scout got this exactly right — this is not "the owner points at the
assets", it is "the shipping crate cannot load a mesh if pointed at one".

Recording it here because a blocker that is stated too broadly stops work
that is not actually blocked.

### The oldest untouched ask, and it is the one that cannot be fixed from inside

`TRV-0260` — **the mech concept art is not in this repository.**

`BRIEF_VIII_B` §D opens with "The art is the spec". §D.7 makes the
side-by-side against the art the *stated completion criterion* for the
whole mech section. `PROMPT_mech_rebuild.md` Task 1 asked for 20-32
committed reference images and said why: *"reference that lives only in a
chat log is lost work."*

`handback/reference/NOTES.md` is honest about what happened — the session
had no image-download capability and said so — but the consequence is that
**four rows across two briefs are permanently unsatisfiable**
(`TRV-0075`, `0100`, `0115`, `0172`), and every judgement about whether
the machine matches the art is one person's memory of a chat.

I checked whether it was ever committed and later removed:
`git log --diff-filter=D` across `*.png *.jpg *.jpeg *.gif *.webp *.glb
*.gltf *.fbx` returns only the deleted Cliffhold captures and two
superseded capture generations. **It never arrived.** Open since
`aefd16f` (2026-07-31) put the briefs in the repo — 10 days.

The medic reference art (`TRV-0261`) is the same story with a shorter
history.

### What I got wrong LAST time

Nothing. This is run 1 and there is no prior entry. What I got wrong
*during* this run, corrected before writing:

1. **I briefly concluded that mech boarding was fixed** because
   `mech_enter_stage_for` now has client readers. I opened
   `main.rs:19191-19256` and found eight `debug!` arms. The plan was
   right and I was about to close a live row on a grep result.
   *Lesson: a reader existing is not the same as the reader doing
   anything.*
2. **I nearly filed `TRV-0055` as delivered** because three of four call
   sites scale `gatling_heat` correctly. The fourth (`main.rs:21781`)
   prints it raw under the same `%`. Both branches are live.
3. **I assumed `powershell.exe` would be reachable** from the Bash tool
   and burned two calls finding it is not.

### What I could NOT check — plainly

I am stating this as a section rather than burying it, because implying a
complete sweep is the failure this role exists to prevent.

- **I did not build and did not run the suite.** Trevor is read-only and
  never invoked `cargo`. Every `DELIVERED` means "there is evidence at
  this path", never "it compiles" and never "it works". `TRV-0020` and
  `TRV-0021` exist solely to record that.
- **I did not open a single PNG.** I indexed 155 capture paths and read
  none of them. Rule 8 says the capture is the instrument; I catalogued
  the instruments.
- **Read whole:** `WHATS_MISSING.md`, `OPERATION.md`, `briefs/README.md`,
  all four briefs, all five prompts, `DECISIONS.md`, `DESIGN_MAP.md`,
  `GAME_STATUS_REPORT.md`, root `README.md`, root `FRIDAY_LOG.md`,
  `BACKLOG.md`, `ANTI_PATTERNS.md`, `TASK0_AUDIT.md`,
  `research/SOURCES.md`, `handback/reference/NOTES.md`,
  `handback/brief-ix/REPORT.md`, the last 310 lines of
  `research/FRIDAY_LOG.md`, and the headers of `held_grenade.rs`,
  `mech_recoil.rs`, `mech_lineup.rs`.
- **Sampled by grep only:** `THOR_LOG.md` (3,002 lines — I read ~200,
  around owner-voice hits), `TOTO_LOG.md` (685 lines — **headings only**),
  `sim.rs` (~27k lines — targeted greps + ~8 regions), `main.rs` (~29k
  lines — targeted greps + ~5 regions), `cockpit.rs`, `map_look.rs`,
  `menu_ui.rs`, `branding.rs`.
- **Not read at all:** `handback/ACCOMPLISHMENTS.md`, `AUDIT.md`,
  `CHANGES.md`, `REPORT.md`, `brief-vii/HANDBACK.md`, the eleven
  per-topic `research/*/SOURCES.md`, `motion-architecture/NOTES.md`,
  `body-rig/SPEC_20_SEGMENT_RIG.md`, `mech-climb/DESIGN.md`, the three
  `CYCLE_*_REPORT.md`, every source file in `jk_wall` / `jk_bevy` /
  `jk_client` / `jk_spike` / `jk_core`, and `export/` (gitignored).

**The 21 `UNVERIFIED` rows are the honest total of the above.** Each one
says in its own Evidence cell what I did and did not do. None carries a
fabricated disposition. None was bucketed as a negative result because a
check did not finish — that is the failure mode this project has hit
twice (46 verify agents killed by a rate limit and filed as "disputed";
three agents' research discarded by a missing `await`), and I would rather
write 21 honest gaps than one false zero.

The `UNVERIFIED` set, for the next Trevor to attack first: `TRV-0004`,
`0020`, `0021`, `0027`, `0029`, `0034`, `0035`, `0042`, `0044`, `0062`,
`0085`, `0098`, `0109`, `0110`, `0129`, `0135`, `0201`, `0237`, `0249`,
`0252`, `0253`, `0254`. (`TRV-0068`, `0086`, `0114` carry partial
`UNVERIFIED` halves inside a `PARTIAL` status.)

### Concurrency, and a near-miss worth recording

Two commits landed from another session mid-run: `fe07c19` ("The mount
learns to walk sideways; a chassis scale the client kept a copy of") and
`f10be3a` ("A slot is one line: the fixed-width tile was the wrong
lever"). Both touched files I had already grepped.

`fe07c19` closed `TRV-0238` — a deferral Friday had written *that same
day*. If I had written the row from the log alone, it would have shipped
false.

**Line numbers in the ledger were true at `f10be3a`.** I anchored to
symbol names wherever I could and quoted the line so a shifted number is
recoverable. The next Trevor should re-anchor on symbols, not on the
numbers, and `git fetch` first.

I did not commit. There was a large diff from another lane in the tree and
Trevor's job does not include running git commands that could touch it —
`OPERATION.md` rule 7 and 7b exist because that has already cost this
project real work once. My three files are on disk and untracked; whoever
commits next should stage **only** `TREVOR_LEDGER.md`, `TREVOR_TASKS.md`
and `TREVOR_LOG.md`.

### Notes for the next Trevor

1. **`git` is the only real tool in the Bash shell.** This machine's
   `bash` has no coreutils — `ls`, `find`, `head`, `sort`, `wc`, `cat`,
   `grep` all exit 127 — and `powershell.exe` is not on the Bash tool's
   PATH either. Do `export PATH="$PATH:/c/Program Files/Git/cmd"` and then
   `git grep`, `git ls-files`, `git log --diff-filter=D` do everything.
   Use the Grep and Glob tools for the rest.
2. **`§owner` is the search that finds the vanishing asks.**
   `git grep -n "§owner" -- 'engine/crates/jk_tdm/src/*'` returned 432
   lines and is where 44 of my rows came from. Run it first, every time.
3. **Read the comment before calling something absent.** Three of the
   things I nearly filed as dead were deliberate documented retentions
   (`ROBOT_SPEED_MULT`'s tombstone, `pod_aim_held`'s tombstone, the light
   chassis refusing to crouch). The scouts already confirmed a set of
   these as NOT gaps and I gave them a row (`TRV-0255`) so no future
   sweep re-opens them.
4. **The plan's own warning is correct and I hit it twice.**
   `WHATS_MISSING.md` has gone stale three times; today I found it stale
   in *both* directions — it under-claims (three Tier 0 items are done)
   and it over-claims (§0-NOW #11's `visor_ready` claim is still true, and
   §0-SPEC15 P2's grenade note is not).
5. **Check whether an ask ships against a document that then goes stale.**
   Five documents in this repo now describe a game that no longer exists.
   They are listed in ledger §D. None of them is wrong on purpose; every
   one was true when written. That is the whole reason this ledger exists.

— **TREVOR**, run 1.

### Postscript, written on the last check of the tree

Between finishing the ledger and finishing this log, the other session
moved again: `main.rs` is modified, all eleven `mech_gallery` PNGs have
been re-captured, and **a new untracked module has appeared —
`engine/crates/jk_tdm/src/frontend.rs`.** It is not in any commit, not in
any log, and nothing in the ledger knows about it.

I am not indexing it: an uncommitted file from a lane still writing is not
a record, and reading it would be reading a moving target. **The next
Trevor should look for it first** — if it lands, it is almost certainly a
`main.rs` extraction in the pattern `branding.rs`, `cockpit.rs`,
`held_grenade.rs`, `mech_lineup.rs` and `mech_recoil.rs` already set, and
several ledger rows anchored to `main.rs` line numbers will have moved
into it. That is exactly the shift rule 5 warns about, happening in real
time.

Nothing in the ledger is invalidated by it. Several line numbers in §B may
now point at the wrong file, which is why every row also names its symbol.

---

# 2026-08-11 — run 3 (register run 2): two new owner specs, five rows moved, one thing I had backwards

HEAD at sweep start and end: `5473d06`. `git fetch` first; `origin/main`
== local `main`. Working tree: `sim.rs` (+279/−10) and `frontend.rs`
(+20) modified by live builders. I wrote only
`ISSUED_VS_DELIVERED.md` and this file.

## Counts

300 rows → **344**. `DONE` 137 → **142** · `PARTIAL` 42 → **48** ·
`NOT STARTED` 65 → **87** · `BLOCKED` 16 → **22** · `SUPERSEDED` 12 →
12 · `CONTRADICTED` 7 → **8** · `UNVERIFIABLE` 3 → **4** · `UNVERIFIED`
18 → **21**. Sums to 344.

Open work went **up 28** while five rows closed. That is two owner specs
arriving in one day, not a regression.

## IDs OPENED — 44

- **`TRV-0301`..`TRV-0317`** — the BOW & SPEAR spec (19 sections + an
  acceptance checklist), **not on disk**. Six sections recoverable
  (§4/§8/§17, §9, §10, §11, plus checklist §B and §D); thirteen are
  invisible and `TRV-0314` says so rather than guessing. Includes the two
  rules the owner stated three times each — the RMB/LMB input split and
  "the crosshair is sacred" — and the four reference images, none of
  which are on disk.
- **`TRV-0318`..`TRV-0341`** — BRIEF XI, all 19 sections plus §0's four
  live constraints, §19's checklist and the proof standard.
- **`TRV-0342`..`TRV-0344`** — the LEARN BACK button (owner-reported,
  fixed `5473d06`), the App-level test that fix admits it still owes, and
  `config/settings.txt` carrying `fov_idx = 4` against `FOV_DEFAULT_IDX = 3`.

## IDs THAT MOVED — 5, each re-derived from code

| Row | Was → Now | Why |
|---|---|---|
| `TRV-0289` | `NOT STARTED` → `DONE` | 12 `agile_moves` PNGs committed in `8de5e93`, re-run in `9b108b2`. Run 1's loudest "the script exists and has produced no PNGs" is answered. |
| `TRV-0206` | `NOT STARTED` → `DONE` | `the_enemy_agile_never_out_luminates_the_ally` (`agile_mech.rs:744`), plus a gamma-decode anchor test so the guard cannot pass on a broken formula. SPEC15's own trap 4, guarded at last. |
| `TRV-0292` | `DONE`(contested) → `PARTIAL` | `09-squint-derived.png` is honestly labelled a 3.33× downsample with hue removed, not a matte. Still no three-tier silhouette. |
| `TRV-0291` | `UNVERIFIED` → `PARTIAL` | Part/mesh/material counts published; no draw-call measurement. |
| `TRV-0296` | `CONTRADICTED` → `CONTRADICTED`, description **corrected** | See below. |

## WHAT I GOT WRONG LAST RUN

**`TRV-0296` / C5. I wrote that the Agile Mech is orange on BOTH sides.
It is not.** `ARMOR_FOE = [0.075, 0.125, 0.265]`, commented "faction dark
blue" (`agile_mech.rs:179`). The enemy Agile ships a **dark-blue primary**
with the chassis's orange on its layered plates only.

I read the `_foe` material **names** in the client and inferred what the
values must be. I never opened the constants. That is precisely the
failure I opened run 1 by naming in someone else's work — reading the
first half of a thing and not checking the second — committed in the
loudest row of the file, and in the more alarming of the two directions.

The contradiction itself survives: BRIEF X §2 states orange is the
Agile's identity and states no faction split; the build splits it. The
builder honoured both instructions by role and said so in `FRIDAY_LOG.md`
before I ever looked. **I could have read that log entry and did not.**

Two smaller ones: run 1's §8 claimed "the 14 `UNVERIFIED` rows" while its
own headline said 18 — after I had criticised the ledger for carrying
three numbers for one set. And `TRV-0289` was true when written and false
90 minutes later.

## THE TRAP I ALMOST WALKED INTO TODAY

`dd4fced`'s subject is **"BRIEF XI: finish the Agile Mech - motion,
limbs, hands, guns"**. The dispatch that sent me here described it as "the
Agile Mech finished". `git show --stat` says it adds **one file, 299
lines, and changes no code.** It is the brief being written to disk before
any build touches it — the good pattern, not a delivery.

Had I filed those 24 rows off the subject line I would have marked an
entire unbuilt brief as done. This is the same shape as run 1's headline
finding and it is why the rule is: **derive from code, never from the
commit message's claim about itself.**

## WHAT I REFUSED TO BANK

`sim.rs` is +279 uncommitted, and it is §9/§10/§11 of the bow/spear spec:
the 3-second charge window, the 7-second maximum-charge bonus, the
`SpearStance` accessors. It is good work and it is not committed.
`TRV-0303`, `0304`, `0305` are `UNVERIFIED` with the reason stated. A
commit that has not happened is not a delivery.

One judgement call inside that work is worth the owner's attention and I
flagged it rather than absorbing it: the spec says "~3 s reaches the
normal maximum, holding longer must not keep growing power" **and** "7 s
grants a bonus". Those cannot both sit on one ramp, so the builder split
them onto two axes — velocity caps at 3 s, damage steps at 7 s. That is a
reading of the owner's words, not a fact about them.

## RE-DERIVED AND STILL TRUE (checked, not carried)

Silent explosions (21 `asset_server.load`, all `.wav`, no boom) ·
`armor_stage_of` 0 readers in `main.rs` against 66 in `sim.rs` ·
`struct CapBeat` still a compile-time const array at `main.rs:5280` ·
Royal arrow launcher zero in every weapon path · all four TIER 0 dead
values (`TDM_TARGET_CHOICES`, `shot_handgun.wav`, `FORGE_SLOTS`, the
`9000.0` literal + the client's copy) · `gait_pose` still infantry-derived
· `arc_preview` still gated on `cam_ctl.ads` for exactly the two forbidden
weapons.

**New this run:** `aim_phase` is `pub` and has **zero readers in
`main.rs`**; `main.rs:16987` still documents RMB as "a draw on projectile
weapons (bow/spear, Brief II grammar)", which the sim killed 12 hours
ago; `solve_arm_ik` has 10 call sites and all ten are arms; no capture in
193 holds RMB with a bow or a spear.

## WHAT THE NEXT TREVOR SHOULD DO FIRST

1. **Re-derive `TRV-0303`/`0304`/`0305`** once the spear work commits.
   Expect three rows to close in one commit.
2. **`WHATS_MISSING.md` has not been touched in eleven commits** and is
   now stale for the fourth time. It knows nothing of BRIEF X's handback,
   BRIEF XI, the bow/spear spec, or the front end. It is still described
   as "the live plan" in `OPERATION.md`. That gap is itself worth a row if
   it survives another run.
3. **Read `FRIDAY_LOG.md`'s newest entry BEFORE re-deriving any row it
   touches.** Today it contained the correction to my own headline
   finding, written eleven hours before I published the error.
4. `briefs/README.md` does not list BRIEF XI.

— **TREVOR**, run 3.
