# BALANCE BOARD — the live sync picture

**Run 1. 2026-08-11.** Rewritten in full every run. Ground truth taken at
this run's start: `git fetch` (clean, no new remote commits), HEAD =
`a3b20f0`, `git status --porcelain`, `git diff --stat`.

Anyone should be able to dispatch from this file alone. If a check did not
complete, it is named in §7 rather than reported as a negative result.

---

## 1. THE VERDICT SOMEONE IS WAITING ON — BRIEF XII

**BRIEF_XII CAN START NOW, in a NEW file `engine/crates/jk_tdm/src/hud.rs`,
by ONE builder (`friday33` lane), with the two-line wiring edit to `main.rs`
DEFERRED to the end of the session and sequenced behind the bow/spear lane's
commit.**

That is the whole answer. The reasoning and the exact sequencing follow.

### 1.1 The obstacle is real and it has NOT landed. It has grown.

Verified this run, not inherited:

```
 M engine/crates/jk_tdm/research/THOR_LOG.md
 M engine/crates/jk_tdm/src/main.rs
?? engine/crates/jk_tdm/src/muzzle_flash.rs
```

```
 engine/crates/jk_tdm/research/THOR_LOG.md |  250 +++++
 engine/crates/jk_tdm/src/main.rs          | 1649 ++++++++++++++++++-----
 2 files changed, 1674 insertions(+), 225 deletions(-)
```

`main.rs` was **1396 insertions** at the previous check and is **1649** now.
The lane is not finishing — it is still writing. Nothing has landed.

Contents confirmed in the diff, by line of diff hunk:

| Evidence | Diff line | Meaning |
|---|---|---|
| `+mod muzzle_flash;` | diff `+`12 | tracked file declares an untracked module |
| `+const BOW_TIP_Z: f32 = -0.100;` | diff `+`30 | recurve geometry, was `-0.088` |
| `+ .add_plugins(muzzle_flash::MuzzleFlashPlugin)` | diff `+`1040 | plugin registration |
| `+ "muzzle_flash" => MUZZLE_FLASH_BEATS,` and 5 more | diff `+`955-1028 | a new capture script |
| javelin/bow tip comment at `+`1113 | | overhead wind pose |

`git show HEAD:.../main.rs | grep "^mod "` returns eleven modules —
`agile_mech`, `branding`, `cockpit`, `held_grenade`, `frontend`, `map_look`,
`mech_lineup`, `mech_recoil`, `menu_ui`, `sim` — and **`muzzle_flash` is not
among them.** So the module exists only in the working tree.

### 1.2 THE TRAP, stated plainly for whoever commits next

**A tracked `main.rs` currently declares `mod muzzle_flash;` while
`muzzle_flash.rs` is untracked. Any `git add -A` by any other lane publishes
a tree that cannot compile.** This is not hypothetical: this repo has already
paid for it once, when a half-written `main.rs` declaring `mod agile_mech;`
was swept into another agent's commit and **HEAD could not compile for two
commits.**

**Standing instruction until the bow/spear lane commits:**
- **No lane other than the bow/spear lane may run `git add -A` or
  `git commit -a` at this repo root.** Stage by explicit path only.
- **No `git stash` bare.** It would take 1649 lines of another agent's
  uncommitted work with it. That has also already happened here once.
- **No `git checkout -- main.rs`.** Same reason, irreversible.

### 1.3 Does a new `hud.rs` actually buy safety? — YES, ALMOST ENTIRELY

The honest answer, which is the one BRIEF_XII §0.1 asked for:

**A new `hud.rs` is collision-free for ~99% of the work and carries exactly
two lines of unavoidable contact with the contested file:**

```rust
mod hud;                                   // beside the other 11 mod lines
.add_plugins(hud::HudPlugin)               // in the app builder
```

That is the entire intersection. It is not zero, and pretending it is zero
is how the `agile_mech` breakage happened. But it is two lines, in two places
whose surrounding text the bow/spear lane is *also* editing — both hunks are
live in the current dirty diff (`+`12 and `+`1040 are literally the module
list and the plugin chain). So a naive concurrent edit lands in the **same
two hunks** the other lane is touching: a guaranteed textual conflict, not a
merely possible one.

**Therefore the sequencing is:**

1. **NOW** — dispatch the HUD builder. It creates `hud.rs` and does all
   design, layout, systems, components and tests inside that one new file.
   Zero collision: the file does not exist for anyone else. This is real
   work, likely most of a session.
2. **NOW, in parallel** — the builder may READ `main.rs`, `frontend.rs`,
   `menu_ui.rs`, `palette` freely. Reading a dirty file is safe; it will read
   the bow/spear lane's in-progress text, which is harmless for design work
   but means **it must not quote `main.rs` line numbers as evidence**.
3. **BLOCKED until the specific event below** — the two wiring lines, plus
   any *migration* of existing HUD code out of `main.rs`.

**The event that clears the block:** the bow/spear lane commits `main.rs`
**together with** `muzzle_flash.rs` in one commit, and `git status --porcelain`
returns `main.rs` clean. Not "when main.rs looks quiet" — when that commit
exists in `git log`. As of this run it does not.

### 1.4 The migration question — the one that is NOT two lines

BRIEF_XII §8 says *"remove redundant or outdated UI elements"*, and the HUD's
current home is `main.rs`: `struct HudRoot` at `main.rs:7212` (HEAD),
`hud_visible` `:7219`, `hud_visibility` `:7233`, `mech_hud_sync` `:9943`, and
**30+ `HudRoot` spawn sites** from `:15195` through `:16486` and beyond.

**Moving those is a large edit to the most contested file in the repo, and it
is NOT covered by "a new module is safe".** Say it out loud in the dispatch:

- **Session A (now):** build `hud.rs` as the new surface — new components,
  new layout, new systems, tests, all self-contained. Do not delete anything
  from `main.rs`.
- **Session B (after the bow/spear commit lands):** wire the two lines, then
  retire the superseded `HudRoot` spawns. This is where the old HUD dies.

Attempting the migration in Session A converts a safe dispatch into the exact
failure mode this board exists to prevent.

### 1.5 Should BRIEF_XII and Trevor's Task 9 run in ONE lane? — YES, ONE LANE

BRIEF_XII §0.1 already claims the overlap; I am ratifying it, with the split.

Two lanes racing here would be **strictly worse**, and not for the usual
reason. Task 9's six defects are not adjacent to the HUD — four of them
**are** HUD surfaces:

| Row | Site | Relationship to BRIEF_XII |
|---|---|---|
| `TRV-0031` Field Manual prints `TDM_TARGET` | `main.rs:24303` | a screen BRIEF_XII §8 audits |
| `TRV-0024` `TDM_TARGET_CHOICES` unread | `sim.rs:430` | **sim-side — NOT this lane** |
| `TRV-0032` `U` missing from `BIND_REGISTRY` | vs `main.rs:22802` | Controls surface, §8 |
| `TRV-0033` medic 2nd flip + midair jump undocumented | `sim.rs:3427`,`:3436` | Controls surface, §8 |
| `TRV-0028` `FORGE_SLOTS` unread | `main.rs:1372` | Forge surface, §8 |
| `TRV-0055` `gatling_heat` two scales | `main.rs:21763/69/72` vs `:21781` | **live HUD readout — same code the HUD rewrites** |

`TRV-0055` settles it. `gatling_heat` prints `×100` under a `%` at three
sites and **raw** under the same `%` at a fourth, and all four branches are
live. A HUD redesign rewrites those readouts. If Task 9 runs in a second lane
it is rewriting the same lines at the same time, in the same file, for a
different reason. **Merge them.**

**But split the sim half out.** `TRV-0024` and the source-side half of
`TRV-0055` are `sim.rs` — friday22's, and `sim.rs` is not currently dirty, so
that half is separately dispatchable right now (see §3).

### 1.6 A caution the brief does not carry

BRIEF_XII's own proof standard demands captures at two aspect ratios across
ten states. **A capture cycle is ~6 minutes** (`TREVOR_TASKS.md` TASK 10:
6 minutes vs ~40 seconds if beats were data). Ten states × two ratios,
with iteration, is not a tail — it is a second session. Budget it, or land
TASK 10 first. I am not recommending TASK 10 first, because the owner asked
for the HUD; I am recommending the capture budget be stated in the dispatch
so nobody reports "brief complete" with four frames.

---

## 2. FILE OWNERSHIP TABLE — as of this run

| File | Current owner | Dirty? | Queued next | Dispatch |
|---|---|---|---|---|
| `src/main.rs` | **bow/spear lane** (uncommitted, growing) | **YES — 1649+/225-** | HUD wiring (2 lines), then HUD migration | **WAIT** |
| `src/muzzle_flash.rs` | same lane | **UNTRACKED** | — | **DO NOT `git add -A`** |
| `src/sim.rs` | *nobody* | **clean** | friday22: `TRV-0024`, `TRV-0055` src half, `TRV-0235`, `TRV-0241` | **SAFE NOW** |
| `src/hud.rs` | *does not exist* | — | BRIEF_XII builder | **SAFE NOW — create it** |
| `src/agile_mech.rs` | *nobody* | clean | BRIEF_XI (blocked, §5) | **SAFE — but see §5** |
| `src/frontend.rs` | *nobody* | clean | — | SAFE (read-only for HUD lane) |
| `src/mech_lineup.rs` | *nobody* | clean | Task 2 / Task 5 captures | SAFE |
| `src/branding.rs`, `menu_ui.rs`, `cockpit.rs`, `held_grenade.rs`, `mech_recoil.rs`, `map_look.rs` | *nobody* | clean | — | SAFE |
| `research/THOR_LOG.md` | **thor** (uncommitted, +250) | **YES** | — | **thor only** |
| `research/TREVOR_*.md` | trevor | clean | — | trevor only |
| `research/BALANCE_*.md` | **balance** | — | — | balance only |
| `briefs/*` | owner | clean | — | read-only to all agents |
| `engine/assets/audio/*.wav`, `handback/brief-vii/*.png` | — | modified (binary/CRLF churn) | — | **see §7 — unexplained** |

**Live lanes right now: two.** The bow/spear builder (`main.rs` +
`muzzle_flash.rs`) and thor (`THOR_LOG.md`). Nobody holds `sim.rs`.

**The lane map is wider than `TREVOR_TASKS.md:18-21` says.** That file states
*"Those are the only two. A third builder has nowhere to go."* That is stale.
There are eleven client modules at HEAD plus every new module anyone creates.
**Three builders are safe today**: the bow/spear lane in `main.rs`, a HUD
lane in a brand-new `hud.rs`, and a sim lane in `sim.rs`. I am flagging that
line as a document to correct, not silently overriding it (§4).

---

## 3. DISPATCH VERDICTS

### SAFE TO DISPATCH NOW

**D1 — `friday33`-class builder: BRIEF_XII, Session A, in `hud.rs` only.**
Touches: **creates** `src/hud.rs`. Reads (does not write) `main.rs`,
`frontend.rs`, `menu_ui.rs`, `branding.rs`, `handback/reference/hud/NOTES.md`.
No collision: the file does not exist for any other lane.
**Explicit prohibitions in this dispatch:** no edit to `main.rs` (not even
the `mod` line, this session); no `git add -A`; no `git stash`; no `sim.rs`;
do not delete any existing `HudRoot` spawn.
**Warn it:** `cargo test` compiles `main.rs`, which another lane is
mid-write. A suite failure during this session may be that lane's transient.
Re-run before believing a failure, per `TREVOR_TASKS.md:23-26`.

**D2 — `friday22`: the sim half of Task 9, in `sim.rs`.**
`TRV-0024` (`TDM_TARGET_CHOICES`, `sim.rs:430`) and the source-side split of
`gatling_heat`'s two scales (`TRV-0055`). `sim.rs` is clean and unowned; this
does not touch `main.rs`. **It unblocks the client half D1 will meet in
Session B**, so running it now is a genuine dependency win, not filler.
Caveat: it is a shared-signature change, so its client-side print sites in
`main.rs` must be left for the HUD lane's Session B — report, don't reach.

**D3 — the owner, not an agent:** the four decisions in §6.

### MUST WAIT

**W1 — BRIEF_XII Session B: the two wiring lines + the HUD migration out of
`main.rs`.**
Waiting on: **the bow/spear lane committing `main.rs` and `muzzle_flash.rs`
in one commit.** Clears when that commit appears in `git log` and
`git status --porcelain` shows `main.rs` clean. It holds 1649 insertions of
uncommitted work as of this run and is still growing.

**W2 — BRIEF_XI §5 (climbing), §4 (crouch), §3 (walk/run/sprint).**
Waiting on: **an owner decision**, not on a file. See §5. This is *not* a
conflict block — `agile_mech.rs` is free. The unbuildable sections are
unbuildable for a reason no sequencing fixes.

**W3 — anything that re-photographs the Agile captures thor rejected
(`03-agile-profile`, `08-agile-legs-profile`, the jump trio).**
Not blocked by a file — blocked by the ~6-minute capture cycle competing
with D1's own capture needs. Sequence it after D1, or it doubles D1's cost.

### NOT BLOCKED AT ALL — say this loudly

- **`sim.rs` is free.** Both the ledger and the briefs describe it as "dirty
  with another lane's spear work" (`BRIEF_XII` §0.1). **That was true; it is
  not true now.** The spear sim half landed in `eff8fbf`/`a3b20f0`. A sim
  lane has been idle on a stale warning.
- **BRIEF_XII does not need TASK 10 (capture-as-data) first.** It would be
  cheaper after, but nothing blocks it. Do not let "do TASK 10 first" idle
  the HUD.
- **BRIEF_XII does not need the two reference PNGs.** `NOTES.md` and
  BRIEF_XII §0 are written as a substitute spec, deliberately, and both say
  a builder may work from them. The images improve the result; they do not
  gate the start.
- **BRIEF_XII does not conflict with BRIEF_XI.** Different files
  (`hud.rs` vs `agile_mech.rs`), different lanes. They could run together —
  if BRIEF_XI were not blocked on the owner.

---

## 4. STALE DOCUMENTS — the ruling that has not arrived

Each row names the ruling and the specific file that has not caught up.

| # | The ruling | Where it landed | Where it has NOT | Severity |
|---|---|---|---|---|
| S1 | **Q4, owner 2026-08-10** — player Royal keeps GOLD; SPEC15's neon-blue line is superseded | `WHATS_MISSING.md:55-59` (struck through, marked SUPERSEDED — correctly), `TREVOR_TASKS.md:70-98` | **`main.rs:15301`** still reads *"asks for the player Royal to carry SUBTLE NEON-BLUE energy"* in the present tense, and **`main.rs:4005`** still says *"note at its construction for why it is not the spec's neon blue"* — both argue against a spec that no longer exists. A builder reading either concludes the gold is an unresolved deviation. **It is ratified.** | Medium — reopens a closed decision |
| S2 | **Q4 consequence** — opposition Royal is RED + YELLOW + NEON-BLUE, so **colour no longer separates the two Royals** | `WHATS_MISSING.md:66-74`, `TREVOR_TASKS.md:88-93` (Tasks 4 and 5 both carry it) | Nothing in `mech_lineup.rs` — it is the module that produces the three-tier comparison frame and it carries no note that the squint test is now the *only* separator. Also: thor found `mech_royal` is lacquer red `(0.52,0.09,0.09)` and **not yet yellow** — the palette half of the ruling is unimplemented, which is a task, not rot. | High — it is the acceptance criterion for Tasks 4/5 |
| S3 | **The climb correction** — `BRIEF_X` §0's false climb row struck in `553c425`; the *code comment* correcting `climb_target`'s gate is written | `BRIEF_X` (committed), `BRIEF_XI` §0.1:15-31 (gated correctly) | **UNLANDED AND AT RISK.** The corrected comment lives at diff `+`852-859 of the bow/spear lane's **uncommitted** `main.rs` — *"said `climb_target` \"requires `!p.in_mech()` and a dropped plate bit\", ... real gate deleted. `climb_target` contains NO test on the climber at ..."*. If that working tree is ever stashed, reverted or lost, **the correction dies with it and the false claim is the only surviving text in the source.** | **Critical — one bad git command from being lost** |
| S4 | **Lane map** — nine-plus client modules are nine-plus lanes | this board | **`TREVOR_TASKS.md:18-21`**: *"Those are the only two. A third builder has nowhere to go."* False at HEAD (eleven modules). It is idling a builder. | Medium — costs parallelism |
| S5 | **The spear sim half landed** (`eff8fbf`, `a3b20f0`) | git history | **`BRIEF_XII` §0.1:100**: *"`sim.rs` ... is currently dirty with another lane's spear work."* Not true now. A HUD builder reading its own brief will believe a sim lane is live when none is. | Low for the HUD lane, Medium for the sim lane it idles |
| S6 | `dd4fced` subject line *"BRIEF XI: **finish** the Agile Mech"* ships **299 lines of markdown, zero code, zero captures** (thor §3; Trevor run 2 independently) | thor's log, uncommitted | `git log --oneline` — which is what every new session reads. **Second commit this week whose subject marks an unbuilt brief as done.** | Medium — corrupts the cheapest status instrument in the repo |

**S3 is the one to act on today.** The single cheapest mitigation: the
bow/spear lane commits, even as WIP, staging `main.rs` **and**
`muzzle_flash.rs` together. That clears S3, clears W1, and defuses the
`git add -A` trap in one action. **It is the highest-value single command
available in this repo right now.**

---

## 5. BRIEF XI vs BRIEF XII — the priority recommendation for the owner

### 5.1 BRIEF_XI's three blockers, verified as a shape, not re-verified as facts

I did not re-derive these; that is thor's job and he has them. I checked
only that they are of one shape, because that determines the ranking. One
I confirmed by direct read because the ranking turns on it:

```rust
// sim.rs:3681
pub fn set_crouch(&mut self, want: bool) {
    if self.in_heavy_mech() {
        self.crouch = want && self.grounded;
    } else {
        self.crouch = want && !self.in_mech();
    }
}
```

The **heavy** mech may crouch when grounded. The **Agile** takes the `else`
branch, is `in_mech()`, and therefore `crouch` is unconditionally `false`.
`sim.rs:8985` states it in the file's own words: *"`set_crouch` refuses it
outright (`want && !in_mech()`)"*. So BRIEF_XI §4 CROUCHING (`:116-121`)
animates a state that cannot occur.

The other two are the same shape: **no leg bones** (~110 parts flat under one
root, so §1's IK cannot attach), and both pace gates `!in_mech` so
walk/run/sprint (§3, `:100`) are states the Agile can never enter.

**All three are the climb question again.** `BRIEF_XI` §0.1 already wrote the
rule for exactly this: *"they become an owner decision — 'do you want mech
climbing built?' — which is a new feature and a `sim.rs` change, not a polish
pass."* The same sentence governs crouch and pace verbatim.

### 5.2 THE RECOMMENDATION — **BRIEF XII FIRST. Not close.**

Four reasons, in order of weight:

1. **BRIEF_XI is blocked on you; BRIEF_XII is blocked on nobody.** Three of
   XI's sections cannot be built until the owner rules, and the ruling is
   *"authorise a `sim.rs` feature"* — the opposite of the polish pass XI was
   framed as (*"do not redesign unrelated game systems"*). BRIEF_XII needs
   one new file and two deferred lines.
2. **BRIEF_XI would be built blind.** Its §5/§4/§3 are animations of verbs
   that never fire. Building them yields motion nobody can trigger — a HUD
   readout with no signal behind it, in animation form. The project has a
   name for shipping that: `ANTI_PATTERNS.md`'s *"the confident narrator"*.
3. **The owner asked for the HUD today, and said it must look good.** SPEC15
   files UI under **P4 POLISH** while the Agile is **P3** — so the owner's
   own standing order ranks XI above XII. **I am recommending against the
   written order, and saying so out loud rather than quietly reordering:**
   the P3/P4 ranking was written before BRIEF_XII existed and before XI's
   recon found three sections unbuildable. A P3 that cannot be built does
   not outrank a P4 that can. **Owner may overrule; the written order is
   preserved here so the overrule is visible either way.**
4. **The whole of BRIEF_XI is not blocked.** Its §2 limbs/hands, §6 weapon
   mounting and §18 reuse work are buildable in `agile_mech.rs`, which is
   clean and unowned. If the owner wants Agile progress *in parallel*, that
   is the safe slice — and it genuinely parallelises with D1, because
   `agile_mech.rs` and `hud.rs` are different files. **This is the strongest
   available three-lane configuration today: D1 (hud.rs) + D2 (sim.rs) +
   BRIEF_XI's unblocked slice (agile_mech.rs).**

**Rank: BRIEF_XII Session A → BRIEF_XI's unblocked slice → (owner rules) →
BRIEF_XII Session B → BRIEF_XI §3/§4/§5.**

---

## 6. OWNER DECISIONS OUTSTANDING — escalated, not resolved

### OD-1 — THE ENEMY AGILE LIVERY. Two current owner instructions, incompatible.

**Side A —** `briefs/BRIEF_X_agile_mech.md:100-104`:

> `## 2. COLOR IDENTITY — orange + metallic blue`
> **Primary armour — orange.** Dominant. Burnt orange, industrial orange,
> warm orange, slightly darker orange for secondary plates.

and `:224`: *"**Agile** — Fast / Light / Orange + Metallic Blue / Compact"*,
and `:269` makes *"Orange + metallic blue becomes the Agile Mech's
recognizable identity"* an acceptance checkbox.

**Side B —** `research/WHATS_MISSING.md:62-65`, SPEC15 P3:

> **Opposition mechs are NOT recolours.** Own armour design, body structure,
> silhouette, mechanical detail, weapon styling — while **keeping the faction
> colour language: neon red, dark red, neon blue, dark blue.**

**Both are current. Both are yours.** BRIEF_X never says "player Agile only",
so its orange claims the whole chassis; SPEC15 P3 says every opposition mech
keeps the faction language. They cannot both govern the *enemy* Agile.

**What shipped:** dark-blue primary. Thor measured it —
`ARMOR_FOE = [0.075, 0.125, 0.265]` — and recorded it as a **FALSE ALARM**
against Trevor's C5 claim that the Agile is orange on both sides. So the
builder followed **Side B** and **refused to pick**, filing it upward
instead. That was correct behaviour and it should be said so.

**Which is newer:** BRIEF_X (2026-08-10, and BRIEF_XI the same day) is newer
than SPEC15's P3 text. But SPEC15 P3 received a *later* owner amendment on
2026-08-10 (the Royal palette), which means P3 is live and being edited, not
superseded wholesale. **Newness does not settle this one.** That is precisely
why it is escalated rather than recommended-and-closed.

**A builder with no ruling would** paint the enemy Agile orange — because
BRIEF_X is the newer, more specific, more detailed document and reads like
the spec of record. That would delete faction colour separation on the
fastest chassis in the game.

**BALANCE RECOMMENDS: Side B for the enemy, Side A for the player.**
BRIEF_X's orange becomes the **player** Agile's identity; the enemy Agile
keeps dark blue with **orange-family accents** to carry the shared chassis
identity without breaking the faction read. Grounds: SPEC15 trap 4 — the
ally/enemy read rests on **value**, not hue, and thor measured the Agile pair
at **17.3× luminance separation** with a non-vacuous guard test already
protecting it. Orange-on-orange throws that away for a wheel-position
argument (`BRIEF_X:232`) that trap 3 says is worth less than silhouette.

**This is an OWNER DECISION. Do not let a builder resolve it.**
Note it also resolves `BRIEF_XI:231`'s explicit carve-out — *"(Does not
resolve §0.4's enemy-livery question.)"* — which has now been deferred
across two briefs.

### OD-2 — BRIEF_XI's three unbuildable sections. One question, three parts.

*Do you want the Agile Mech to crouch, to walk/run/sprint as distinct paces,
and to have leg bones?* Each is a `sim.rs` and/or rig change, i.e. a feature,
not the polish pass BRIEF_XI was framed as. **Answer all three at once** —
they are one architectural question wearing three hats, and answering them
one at a time costs three round trips. See §5.

### OD-3 — Q5, `SCOUT_SCALE`. **CONFIRMED STILL 1.05 AND STILL FROZEN.**

Verified this run: `SCOUT_SCALE` is defined in `sim.rs` (referenced at
`sim.rs:3725` under `§owner AGILE SUPPORT MECH`) and consumed by
`mech_lineup.rs:1379` / `:1400` and guarded by that module's own test at
`:1398`. **No lane has touched it** — `sim.rs` is clean and the dirty
`main.rs` diff contains no `SCOUT_SCALE` change. `BRIEF_XI` §0.3:59 restates
the freeze: *"`SCOUT_SCALE` remains frozen (open owner question Q5)."*
The freeze is holding. The question is still open: 1.05 × 1.78 m = **1.87 m**,
a big man, not a machine, and the constant was once 1.42.

### OD-4 — The two Royals separate by value, or by a lamp?

Thor's dispatch 4, unchanged and now numeric. Carried forward here so it does
not live only in an uncommitted log.

*(Q1, Q2, Q3, Q6, Q7, Q8 in `TREVOR_TASKS.md` Band 0 remain open and are not
duplicated here — Trevor holds them.)*

---

## 7. WHAT I COULD NOT CHECK — plainly

1. **The bow/spear lane's identity and intent.** I know a lane is writing
   1649 lines into `main.rs` and creating `muzzle_flash.rs`. I do **not**
   know which session it is, whether it is still running, or when it intends
   to commit. **W1's clearing event is therefore unschedulable by me.**
   Everything downstream inherits that: if that lane has silently died, its
   work is stranded and someone must decide whether to adopt or discard it —
   and I cannot tell the difference between "still working" and "abandoned"
   from a working tree.
2. **Whether the bow/spear tree compiles.** I did not build. I ran no
   `cargo` command — a build takes ~6 minutes and would have raced the
   writing lane. So "HEAD compiles" is **unverified by me this run**; I
   assert only the structural fact that `mod muzzle_flash;` + untracked file
   would not survive a partial commit.
3. **BRIEF_XI's leg-bone and pace-gate findings.** I confirmed `set_crouch`
   by direct read. The ~110-flat-parts claim and both pace gates I took from
   the recon **as reported** and did not re-derive. That is thor's lane. If
   the owner is about to rule on OD-2, **the leg-bone claim should be
   thor-confirmed first** — it is the one that decides whether §1's IK work
   is possible at all.
4. **`engine/assets/audio/*.wav` (17 files) and ~39 `handback/brief-vii/*.png`
   show as modified with no lane claiming them.** These are binary; I did not
   diff them. Most likely cause is checkout/CRLF or a regeneration side
   effect, and `gen_sfx.py` is also modified, which supports the audio half.
   **But I cannot confirm it is benign, and 39 modified capture PNGs are the
   project's evidence base.** Someone should establish whether those images
   still show what their filenames claim, because thor's §3 already found
   three Agile frames whose labels do not match their pixels.
5. **Untracked files outside `src/`.** `git status --porcelain` was filtered
   for readability on the assets/handback noise. Only one untracked `.rs`
   exists (`muzzle_flash.rs`) — that I checked directly and it is the one
   that matters.
6. **`git fetch` returned no new commits**, so no other SESSION has pushed
   since `a3b20f0`. This is a real negative result, not a failed check.
