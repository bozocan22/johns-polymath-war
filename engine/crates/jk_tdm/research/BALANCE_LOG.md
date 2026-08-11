# BALANCE LOG — append only

The board is the picture. This is the diary: what changed, what I stopped,
and what I missed. Misses are recorded with the same weight as catches. A
coordinator that logs only its successes is measuring nothing.

---

## RUN 1 — 2026-08-11

**Role created by the owner today:** *"check everybody's work in sync, in the
same timeline, without conflict — make sure each section communicates with
each other, use Trevor's and Thor's brain."* First run. No prior board.

**Ground truth taken:** `git fetch` (no new remote commits), HEAD `a3b20f0`,
`git status --porcelain`, `git diff --stat`, `git log --oneline -25`.
No `cargo` command run — a build races the live writing lane.

### State at run start

- **Two live lanes**, neither announced: a bow/spear/muzzle-flash builder in
  `main.rs` (+ untracked `muzzle_flash.rs`), and thor in `THOR_LOG.md` (+250).
- `sim.rs` **clean and unowned** — contrary to what two documents still say.
- HEAD's last five commits are three research/doc commits and two build
  commits. `553c425` created this role and struck BRIEF_X's climb row.

### COLLISIONS PREVENTED THIS RUN

1. **The `git add -A` trap, still armed.** `main.rs` contains
   `+mod muzzle_flash;` (diff line 12) and `+.add_plugins(muzzle_flash::MuzzleFlashPlugin)`
   (diff line 1040) while `muzzle_flash.rs` is **untracked**. Confirmed
   `muzzle_flash` is absent from HEAD's eleven `mod` lines. Any other lane
   running `git add -A` publishes a non-compiling tree. **This exact failure
   already cost this repo two broken commits** via `mod agile_mech;`.
   Board now carries a standing prohibition on `git add -A`, bare
   `git stash`, and `git checkout -- main.rs` until that lane commits.

2. **A guaranteed textual conflict in BRIEF_XII's wiring.** The two lines
   `hud.rs` needs in `main.rs` — the `mod` list and the plugin chain — are
   the **same two hunks** the bow/spear lane is currently editing (diff `+`12
   and `+`1040). Had the HUD builder added them this session it would not
   have been a possible conflict, it would have been a certain one. Deferred
   to Session B with a named clearing event.

3. **The HUD migration mistaken for "safe because it's a new module".**
   BRIEF_XII §0.1 proposes `hud.rs`, which is right, but the HUD's current
   home is `main.rs` — `HudRoot` at `:7212` with 30+ spawn sites through
   `:16486`. Retiring those is a large edit to the most contested file. Split
   into Session A (new file only, zero `main.rs` contact) and Session B
   (wiring + migration, after the commit lands). Without the split, a "safe"
   dispatch becomes the exact failure the role exists to prevent.

4. **Two lanes racing over `gatling_heat`.** BRIEF_XII and Trevor's Task 9
   both rewrite `main.rs:21763/21769/21772/21781` — `TRV-0055`'s two scales
   under one `%` — for different reasons. Ratified as ONE lane, with the sim
   half (`TRV-0024`, `sim.rs:430`) split out to friday22 where it does not
   collide and where it usefully lands *before* Session B needs it.

5. **An idle sim lane, twice over.** `BRIEF_XII` §0.1:100 and the general
   folklore both say `sim.rs` is "dirty with another lane's spear work". It
   landed in `eff8fbf`/`a3b20f0`. `sim.rs` is clean. A whole builder lane was
   being held by a stale sentence.

6. **A false ceiling on parallelism.** `TREVOR_TASKS.md:18-21` — *"Those are
   the only two. A third builder has nowhere to go."* HEAD has eleven client
   modules. Board publishes a three-lane configuration that is safe today:
   `hud.rs` + `sim.rs` + `agile_mech.rs`.

### CONTRADICTIONS FILED, NOT RESOLVED

- **OD-1, the enemy Agile livery.** `BRIEF_X:100-104` industrial orange
  primary vs `WHATS_MISSING.md:62-65` SPEC15 P3 faction language (neon red /
  dark red / neon blue / dark blue). Both current, both the owner's, and
  **newness does not settle it** because P3 received a *later* amendment on
  the same date. Recommended player-orange / enemy-blue-with-orange-accents
  on trap-4 luminance grounds (thor's measured 17.3× separation). Escalated.
  Noted that the builder shipped dark blue and **refused to pick** — correct
  behaviour, and recorded as such rather than as an unfinished task.
- **OD-2, BRIEF_XI's three unbuildable sections.** Confirmed `set_crouch`
  myself by direct read (`sim.rs:3681-3687`) because the priority ranking
  turns on it: the heavy crouches when grounded, the Agile takes the `else`
  branch and is unconditionally refused. Leg bones and pace gates taken as
  reported. Filed as ONE question with three parts, to be answered together.

### STALE DOCUMENTS NAMED

S1 (`main.rs:15301`, `:4005` still argue against the superseded neon-blue
spec) · S2 (the Royal colour-no-longer-separates consequence has not reached
`mech_lineup.rs`) · **S3 (the climb-comment correction is sitting UNCOMMITTED
inside another lane's dirty `main.rs` and is one bad git command from being
lost — the most urgent row on the board)** · S4 (the two-builder ceiling) ·
S5 (`sim.rs` described as dirty) · S6 (`dd4fced`'s subject says "finish" over
299 lines of markdown and zero code).

**Single highest-value action available in this repo right now:** the
bow/spear lane commits `main.rs` and `muzzle_flash.rs` **together**, even as
WIP. That defuses the `add -A` trap, rescues S3, and clears W1 in one command.

### CONFIRMED FROZEN

`SCOUT_SCALE` = 1.05, untouched by any lane. `sim.rs` clean, no
`SCOUT_SCALE` hunk in the dirty `main.rs` diff. Q5 still open.

### WHAT I MISSED, OR COULD NOT SEE — recorded so run 2 can close it

- **I cannot identify the bow/spear lane or tell "still working" from
  "abandoned".** W1's clearing event is therefore unschedulable by me. This
  is the biggest hole in this run and everything downstream inherits it.
- **I did not verify the tree compiles.** No `cargo`. "HEAD compiles" is
  unverified this run; I assert only the structural fact about the untracked
  module.
- **17 modified `.wav` and ~39 modified `handback/brief-vii/*.png` have no
  owner and I did not diff them.** Probably CRLF/regeneration churn
  (`gen_sfx.py` is also modified), but **I recorded it as unexplained rather
  than benign.** Those PNGs are the project's evidence base, and thor already
  found three Agile frames whose labels do not match their pixels. If run 2
  finds those images changed in content, this run was blind to it.
- **I filtered `git status --porcelain` for readability** on the asset noise.
  I checked directly that only one untracked `.rs` exists. Any untracked
  non-`.rs` file outside `src/` was not enumerated.
- **I recommended against the owner's own written priority order** (SPEC15
  ranks the Agile P3 above UI at P4) in putting BRIEF_XII ahead of BRIEF_XI.
  Flagged in the board rather than applied silently. If the owner overrules,
  that is the system working, and this line is the audit trail.
