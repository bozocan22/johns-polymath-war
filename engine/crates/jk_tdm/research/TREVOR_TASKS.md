# TREVOR TASKS — everything that needs to be worked on

**Generated 2026-08-11 (run 4) from `TREVOR_LEDGER.md`, at HEAD `46d6dbb`.**
Rewritten in full each run. Every task carries its `TRV-` rows, so the
ledger is the "why" and this file is the "do".

**379 asks indexed · 156 delivered · 155 open · 20 unverified.**

---

## THE LANE MAP — corrected, and my run-1 version was wrong

Run 1 of this file said: *"friday22 and friday33. Those are the only two.
A third builder has nowhere to go."* **That was false and it was idling
builders.** `git ls-files src/` returns **thirteen modules**. BALANCE
caught it (`BALANCE_BOARD.md` §4 S4); the row is `TRV-0365`.

| Lane | Files | Currently |
|---|---|---|
| **friday22** | `sim.rs` | free |
| **friday33** | `main.rs` | **dirty — another lane is writing.** Coordinate. |
| **friday-hud** | `hud.rs` | free |
| **friday-front** | `frontend.rs`, `menu_ui.rs` | free |
| **friday-mech** | `agile_mech.rs`, `mech_lineup.rs`, `mech_recoil.rs`, `cockpit.rs` | free |
| **friday-fx** | `muzzle_flash.rs`, `held_grenade.rs`, `branding.rs`, `map_look.rs` | `muzzle_flash.rs` dirty |

A task that spans `sim.rs` **and** `main.rs` is not one task. Split it and
say which half comes first. **Warn every dispatch:** both lanes run
`cargo test`, which compiles both big files, so a suite run during another
lane's write fails for no reason. Re-run before concluding a failure is
real.

---

# BAND 0 — THIRTEEN DECISIONS. THIS IS THE REAL STORY OF THE WEEK.

**You are decision-blocked, not work-blocked.** Thirteen questions, zero
build cost, and between them they gate **31 rows** and stop four builders
guessing on your behalf. Ranked: what unblocks the most, first.

Each carries: the question in one line, both options, what it blocks, and
my recommendation. **The recommendation is mine, not a fact. Overrule it
freely — I only need the sentence.**

---

### D1 — Can the Agile Mech climb? Do you want mech climbing built at all?
`TRV-0318`, `TRV-0326` · **blocks BRIEF XI §5 and its §19 checkbox** · gates a `sim.rs` feature

**The facts, re-derived from source today.** `climb_target`
(`sim.rs:11940`) skips every candidate where `!m.in_mech()` — **the target
must be a mech** — and then requires a **dropped plate** on it. The
climber's gate is at the caller: `main.rs:23868`,
`if !p.in_mech() && p.alive() && ...`. So hull-climbing is a verb for a
pilot **on foot** against a stripped enemy hull. Something you do *to* a
mech, not *in* one. BRIEF X §0 asserted the opposite and that row was
struck as FALSE in `553c425`.

- **Option A —** "the Agile cannot climb; drop §5 and the §19 box." Free.
- **Option B —** "build mech climbing." That is a **new `sim.rs` feature**,
  not the polish pass §5 describes. Multi-session.

BRIEF XI §0.1 already says the right thing and nobody may proceed past it:
*"do not quietly animate a verb that never fires, and do not quietly skip
it either."*

**My recommendation: A.** The verb that exists — a dismounted pilot
swarming a stripped hull — is a better and rarer idea than a mech climbing
a wall, and it is already built and tested.

---

### D2 — Can the Agile Mech crouch? `sim.rs` currently forbids it.
`TRV-0325`, `TRV-0034` · **blocks BRIEF XI §4** · gates `sim.rs` + `main.rs`

`set_crouch` (`sim.rs:4134`):

```rust
if self.in_heavy_mech() { self.crouch = want && self.grounded }
else                    { self.crouch = want && !self.in_mech() }
```

The scout chassis is not `in_heavy_mech`, so it falls to the else arm and
**`!in_mech()` refuses the crouch outright.** BRIEF XI §4 — *"crouch with
the whole lower body, do not simply lower the character vertically"* — is
**not buildable as written.**

- **Option A —** the Agile is not supposed to crouch. Drop §4.
- **Option B —** let it. One `sim.rs` line plus a `MECH_CROUCH_HEIGHT_FRAC`
  path for the scout, **plus** the client half — `MECH_CROUCH_HEIGHT_FRAC`
  (`sim.rs:6386`) has **zero readers in `main.rs`**, so `gait_pose`
  (`main.rs:2518`) would still bake the infantry ratio.

**My recommendation: B, and it is cheap on the sim side.** But note that
the hitbox moves with `height()`, so this is a gameplay change, not a pose.

---

### D3 — Do the Agile's pace gates matter? Both are `!in_mech`, so walk/run/sprint are unreachable in a mech.
`TRV-0323`, BRIEF XI §2 · **blocks §2 and the movement half of §19**

BRIEF XI §2 asks the three gaits to be differentiated and *"lighter and
faster than the Big Mech"*. The recon found both pace gates are `!in_mech`.
I did **not** re-derive this one myself and it is `UNVERIFIED` in the
ledger — I will not guess at it.

- **Option A —** ungate the paces for the scout chassis (`sim.rs`).
- **Option B —** the Agile has one pace by design; rewrite §2 to say so.

**My recommendation: get D1, D2 and D3 answered in one sentence each, in
the same message.** They are three faces of one question — *is the Agile
Mech a chassis with a full movement vocabulary, or a fast machine with a
small one?* Answer that and all three fall out.

---

### D4 — Enemy Agile livery: orange, or blue?
`TRV-0296`, `TRV-0320` · **CONTRADICTED since run 1** · gates BRIEF XI §16

Two of your own instructions pull opposite ways. BRIEF X §2: *"Primary
armour — orange. This becomes the Agile Mech's recognizable visual
identity"*, with **no faction split stated**. SPEC15: opposition mechs keep
red/blue. The build split it **by role** and said so:

```rust
pub const ARMOR_ALLY: [f32; 3] = [0.90, 0.42, 0.11];   // industrial orange
pub const ARMOR_FOE:  [f32; 3] = [0.075, 0.125, 0.265]; // faction dark blue
```

BRIEF XI §16's *"do not change the established colour identity"*
explicitly does **not** resolve it (§0.4 says so).

- **Option A —** opposition Agile stays blue-primary. Zero work.
- **Option B —** orange-primary both sides. `scout_hull_foe` and
  `scout_plate_foe` are **the whole change** — the builder named them.

**My recommendation: A.** The luminance guard already passes at 17× and
the silhouette work has not been done yet; taking away the colour
separator before the shape separator exists costs friend-or-foe reads.

---

### D5 — Does the arc preview's landing ring obstruct the sacred crosshair?
`TRV-0294`, `TRV-0308`, `TRV-0355` · **blocks the client half of the input split** · **now photographed**

**This one has become urgent and there is finally a picture.** Two briefs
forbid the arc in bold, twice each — *"No trajectory arc. No landing
marker. The arc is learned — that is the entire point"*. The game ships it
gated on exactly the two weapons named (`arc_preview`, `main.rs:21657`:
`cam_ctl.ads && spec.projectile.is_some()`; only Bow and Spear declare
`projectile: Some(..)`).

**And `cam_ctl.ads` is RMB, which your newest spec has just declared
PRE-AIM.** So the forbidden arc is now bound to the exact button the
newest spec elevates. Thor photographed it: `spear_fp/02-fp-spear-preaim.png`
shows the ring **on the exact crosshair pixel** with a red range readout
under it; `01-fp-spear-idle.png` at the same framing shows a clean `[ + ]`.

- **Option A —** honour the briefs. friday33, ~1 hour: gate `arc_preview`
  on the grenade-class throw only, and take an RMB-held beat so the
  *absence* is photographed too.
- **Option B —** retire the ban. Then the screen-intrusion sweep needs a
  **second instrument** — it measures `weapon_parts` geometry only
  (`weapon_bounded_corners`, `main.rs:3096`) and **cannot see a
  world-space ring or a UI text node at all.**

**My recommendation: A.** You stated "the crosshair is sacred" three times
in the newest spec and the ring is the only thing on the crosshair in the
shipped pre-aim frame. **Rule this before anyone builds the client half of
the input split on top of it.**

---

### D6 — The 7-second charge was resolved onto a second axis. Is that your intent?
`TRV-0304` · already shipped · **one sentence either way**

Your spec says *"~3 s reaches the normal maximum, holding longer must not
keep growing power"* **and** *"7 s grants a maximum-charge bonus"*. Those
cannot both sit on one ramp, so the builder split them: **velocity caps at
3 s, damage steps at 7 s** (`SPEAR_MAX_CHARGE_S` 7.00,
`SPEAR_MAX_CHARGE_DMG` 1.10). It is argued in the comment, honestly.

**That is a reading of your words, not a fact about them, and it is now
live in the game.** Confirm it or redirect it.

**My recommendation: confirm.** It is the only reading that satisfies both
sentences, and it makes the 7 s hold a real risk/reward rather than a
longer ramp.

---

### D7 — Re-supply eight owner images. Six have been missing for up to eleven days.
`TRV-0260`, `0261`, `0309`..`0312`, `0361`, `0362` · **blocks 6+ rows and 3 completion criteria**

`git ls-files handback/reference/` returns **two files and both are
`NOTES.md`.**

| Save into | Which |
|---|---|
| `handback/reference/` | mech concept art (`TRV-0260` — **11 days, the oldest live ask**), medic art (`TRV-0261`) |
| `handback/reference/` | bow layout, bow design, spear design, **spear charging pose** (`TRV-0309`..`0312`) |
| `handback/reference/hud/` | `img1-mech-pov.png`, `img2-human-pov.png` (`TRV-0361`, `0362`) |

`BRIEF_VIII_B` §D opens *"The art is the spec"* and §D.7 makes the
side-by-side the mech section's **stated completion criterion**.
`TRV-0312` is the acceptance criterion for the javelin wind pose — **the
pose cannot be built to a picture nobody has.**

**Good news worth copying:** `8bbb954` wrote the two HUD images down in
prose *while they were on screen*, declared that description the working
reference until the PNGs land, and cited `TRV-0260` as the reason. **No
builder is blocked on the HUD; only the acceptance check is.** Nobody has
done that for the four bow/spear images. Somebody should.

---

### D8 — The ~⅓ bot mech output cut, and braced fire no longer perfectly accurate. Keep it?
`TRV-0239`, `TRV-0240` · **unruled for six days** · wants a playtest, not an argument

Bot mech damage at 10 m fell **492 → 331 over 17 s**; braced turret fire
went **0° → ~1.6°**. The builder volunteered both and called one *"a nerf
nobody asked for"*. The direction is what you asked for; the magnitude is
a balance change nobody signed off.

**My recommendation: play one match and say "fine" or "too far".** This is
the oldest unruled decision in the file and it does not get better by
being reasoned about.

---

### D9 — The 40 m sightline rule was retired by an agent's arithmetic. Ratify or replace.
`TRV-0117`, `TRV-0377` · **blocks the map tier**

Every shipped map fails, and the numbers were **re-measured** this week
from the shipping validator, not quoted: Arena **102.9 m**, Bailey
**120.2 m**, Gardens **115.0 m**, Cliffhold **577.1 m**, Battlefield
**637.4 m**. The old baselines were pre-`MAP_SCALE` and wrong by 1.25×.
The rule is unsatisfiable above ~15 m half-extent.

**The arithmetic is right. The authority is wrong** — an agent retired an
owner's rule.

- **Option A —** ratify the retirement, one sentence.
- **Option B —** name a replacement number. **Note the instrument's own
  defect first** (`TRV-0374`): it is flat-map-only and grid-quantised at
  ±35 m on Battlefield, ±42 m on Cliffhold. **Do not set a rule to 0.1 m
  precision against an instrument with that error bar.**

**My recommendation: A now, B later** — ratify the retirement so the map
tier unblocks, and set a real number when someone fixes the validator.

---

### D10 — Traversal: adopt `DECISION.md`'s migration path, or leave the closed-form poser alone?
`TRV-0376`, `TRV-0319`, `TRV-0322` · **gates BRIEF XI §1**

`research/motion-architecture/DECISION.md` (775 lines, `632824a`) is the
deliverable that was orphaned for weeks. §5 rejects five families on named
axes (motion matching, learned/LMM, two Bevy crates, active ragdoll). §7
gives a migration path. §0 gives the decision in one paragraph.

BRIEF XI §0.2 already makes **item 3 binding** — generalise `solve_arm_ik`
to the legs, **no second solver, no crate**. Verified today:
`solve_arm_ik` (`main.rs:2585`) has **10 call sites and every one is an
arm**; there is no `leg_ik`, `foot_place` or `solve_leg` anywhere in
`src/`.

- **Option A (Project A) —** do item 3 only. 1-2 sessions. Closes §1, most
  of §2, and the foot half of §19.
- **Option B (Project B) —** adopt §7's full migration path. Multi-session,
  and §8.1 costs it honestly.

**My recommendation: A.** It is the single highest-value BRIEF XI item, it
is already binding, and §9's own benchmark (`TRV-0373`) has never been run
— so nobody has the number that would justify B.

---

### D11 — Should the mech's first-person aim leave the visor, or the hull turret?
`TRV-0053` · the plan itself files this as **"A DECISION FOR THE OWNER, not a task"**

The camera sits at the visor (2.7234 m); every mech weapon fires from
`EYE_REL` (1.62 m). The gap is **1.1034 m** — your "1.10 m", confirmed
exactly by independent arithmetic. A hull turret genuinely *is* a metre
below the visor, so this may be correct and merely unstated.

Changing it moves **every mech engagement, hit test, cover line and tracer
in the game.** Nobody should decide that quietly for you.

**My recommendation: leave it and write the sentence down.** It is
physically right; it only reads as a bug because nothing says so.

---

### D12 — Four single numbers, each one sentence.
`TRV-0059`, `TRV-0248`, `TRV-0236`, `TRV-0370`

| # | The question | Row |
|---|---|---|
| a | **`SCOUT_SCALE = 1.05`** makes the medic 1.87 m — a big man, not a machine. The constant was once 1.42. Intent? **Frozen in two briefs pending you.** | `TRV-0059` |
| b | **The scout's hitbox no longer shrinks** when rolling or crouching. It is the only fighter in the game whose height never changes in any stance. A dodge that does not duck the head band is a different verb. | `TRV-0248` |
| c | **`armor_spec`'s flat values are unreachable** for any piloted heavy chassis — `apply_armor` takes the hull/angle branch and returns first. Keep them scaled for table consistency, or delete them? | `TRV-0236` |
| d | **The two Royals separate by 1.007×** (ally hull 0.0567, foe 0.0563) where the Agile's guard demands ≥ 8×. Do the two Royals separate **by value**, or do you accept a lamp-only read on 0.4% of the machine? | `TRV-0370` |

**On (d), one build note that changes the shape of the fix:** the Royal
colours are **inline `Color::srgb` literals inside a `materials.add`
block** (`main.rs:15380`/`:15398`), not consts — so they **cannot be
tested where they live**. Lifting the six literals out is the
prerequisite, not the recolour. Whichever way you rule, that lift happens.

---

### D13 — Two specs still live only in a chat window. Paste them into `briefs/`.
`TRV-0313`, `TRV-0281` · **archival, not build** · closes 14 invisible rows

`git ls-files briefs/` returns 14 files. Neither the **bow-and-spear spec**
(19 sections + acceptance checklist) nor the **FRONT END spec** is among
them. **Thirteen of nineteen bow/spear sections are invisible to this
ledger** (`TRV-0314`) and I cannot tell whether they were built, refused,
or never read. That row is not a guess and must not be turned into one.

`efe1428` said it best while writing BRIEF X to disk *before* building
anything: *"a spec that lives only in a chat window is the exact failure
Trevor exists to catch."* BRIEF X, XI, XII and XII-A all got that
treatment. **Two specs did not.** One paste closes this forever.

---

### ✅ CLOSED — do not re-open

**Q4, the Royal palette. Answered 2026-08-10.** *"Keep the royal mech, and
keep the opposition royal mech colour red yellow and neon blue details."*
Player Royal: **the GOLD STAYS**, the builder's call is ratified, SPEC15's
neon-blue line is superseded. Opposition Royal: **RED + YELLOW +
NEON-BLUE details** — red primary, the other two as accents.

**The consequence that Tasks below must carry:** the player Royal is gold
and the opposition Royal carries yellow, so **colour no longer separates
them**. "Must not read as a recoloured player Royal" now rests **entirely
on body and silhouette** — which is D12(d), measured at 1.007×.

**Two live comments still argue against this ruling** and will mislead the
next builder who reads them: `main.rs:15301` (*"asks for the player Royal
to carry SUBTLE NEON-BLUE energy"*, present tense) and `main.rs:4005`.
`TRV-0363` — delete both lines in whatever pass touches that file next.

---

# BAND 1 — THE THINGS BLOCKED BY NOBODY, WHICH IS WHY THEY ARE FIRST

Nothing in this band needs a decision, a number, an image, or another
lane. Every one has been open for three days or more.

## TASK 1 — The first-person javelin does not move through a seven-second charge
`TRV-0367`, `TRV-0306` · **friday33** · **half a session** · Thor's #1 finding

**Status.** `main.rs:21536`:

```rust
let raw_wind = if p.gun == GunKind::Spear && p.spear_wind_t > 0.0 {
    1.0 - p.spear_wind_t / SPEAR_WINDUP_S
```

`spear_wind_t` is the **RELEASE** clock and reads 0 for the entire charge.
This is one of the exact three hand-written copies that
`TdmSim::spear_plant_frac_of` was published to retire —
`sim.rs:10118-10123` names them, *"currently in three separate places over
there"*. **Two of the three survive.**

**What the thread already knows — this is a fully specified handoff:**
- `spear_wind_frac_of` has exactly **ONE** client reader in the whole
  binary (`main.rs:18705`, the third-person rig). `spear_stance_of` and
  `spear_max_charged_of` have **ZERO**.
- The third-person half is **correct and was built from the sim's own
  accessors**. The builder diagnosed this exact defect for the body and did
  not fix it one function away in `fp_viewmodel`.
- The second copy to retire in the same pass: `main.rs:1403`
  (`torso_coil_yaw`).
- **The commit message claims this is done.** It is true for third person
  and false for first person, and the false half is the view your §6 is
  written about. Thor recorded it as a wrong "verified", not as a gap.

**Thor's own dispatch, quoted:** replace `raw_wind` with
`game.sim.spear_wind_frac_of(player)` for the wind and
`spear_plant_frac_of` for the release, **keeping the
`SPEAR_WIND_RELEASE_S` tail on the release half only.**

**Acceptance check:** the FP spear translation at charge 0.0 and charge 1.0
must **differ**. Mutation-prove it by pinning `spear_wind_frac_of` to 0 —
the test must go red.

## TASK 2 — Run `spear_wind`, and add an RMB hold to the bow beats
`TRV-0368`, `TRV-0317` · **friday33** · **one capture cycle (~6 min)** · Rule 8

**The flagship pose of the whole bow/spear pass has never been
photographed.** `SPEAR_WIND_BEATS` exists (`main.rs:6011`), is registered
as `"spear_wind"` (`:6970`), and `handback/brief-vii/spear_wind/` **does
not exist on disk** — I confirmed it against `git ls-files`.

Eight named snaps were never taken, **including `02-preaim-not-a-wind`,
which is the builder's own stated proof for half of input rule 2.** Not
verified, not disproven — **never checked.**

In the same cycle: `BOW_DRAW_FP_BEATS` (`main.rs:6294`) presses
`MouseButton::Left` only, so **no bow pre-aim frame exists in this
project.** The spear got one (`spear_fp/02-fp-spear-preaim.png`) and it is
the frame that settled D5. The bow deserves the same.

**Acceptance check:** `spear_wind/01..08` on disk, and a bow frame with RMB
held at the snap beat.

## TASK 3 — The mech-entry charge leak. Third time recorded.
`TRV-0369` · **friday22** · **~1 hour**

`sim.rs:8103` (ScoutArmor) and `sim.rs:8136` (Robot|Royal) clear
`gatling_heat`, `gatling_vent_t`, `plasma_charge_t` and
`mech_plates_dropped` on boarding — and never touch `bow_draw_t`,
`spear_charge_t`, `spear_power`, `spear_max_charge`. The whole bow/spear
dispatch sits in the `else` arm of the in-mech branch
(`sim.rs:8859-8873`), so **the clocks freeze on boarding and the first
tick after dismount reads as a release edge.**

Board mid-draw, dismount, and **an arrow leaves** — or a javelin at 1.3×
still carrying the +10% flag.

**Fix it at the ENTRY, not the exit** — Thor is explicit: *"the entry is
where the state stops being simulated."* The dismount teardown
(`sim.rs:7563`) already clears `mech_jump_phase` **for precisely this
argument, in a comment that says so.**

**Acceptance check:** a test that boards mid-draw and dismounts, asserting
nothing launches and no charge survives.

## TASK 4 — Every explosion is silent
`TRV-0030`, `TRV-0026` · **friday33 + `gen_sfx.py`** · **1 hour** · open 3 days, blocker named on day one

Re-derived today: **25 `asset_server.load` calls — 21 `.wav` + 4 `.png`.**
The wav list is bow · click · headshot · hit · hurt · jump · kill · pickup ·
reload · roll · shield · shot_ak · shot_deagle · shot_glock · shot_mg ·
shot_mp5 · shot_rifle · shot_shotgun · shot_sniper · spear · win.
**No boom.** The sim publishes `Boom` and nothing plays.

`gen_sfx.py` generates no explosion either, so this is 1 hour *including*
writing the generator line. In the same sitting: `shot_handgun.wav` is
written by `gen_sfx.py:73`, sits on disk, and is in **no** load list; and
plasma / repair beam / barrier / precision charge all play `shot_mp5`,
marked placeholder at the call site.

*(Struck for the record: the old blocker "all 21 loads are `.wav`,
unblocker is any image loading at all" is **false** — four `.png` loads
ship in `branding.rs`. Corrected in `7719296`.)*

**There has never been an owner blocker on any of this.**

**Acceptance check:** an explosion is audible; `gen_sfx.py` writes
`boom.wav`; the load list is 22 wavs.

## TASK 5 — Armour damage states get a client half
`TRV-0036`, `TRV-0137`, `TRV-0140` · **friday-hud or friday33** · **1 session**

**Re-derived today:** `armor_stage_of` / `armor_wear_of` / `ArmorStage` →
**`sim.rs`: 66. `main.rs`: 0. `hud.rs`: 0.** A brand-new 1,742-line HUD
module shipped this week and did not pick this up either. Fresh, Scuffed
and Cracked render identically; only Severed shows, and only because it
removes the piece.

**What the thread already knows — a fully specified handoff:**
- The sim publishes everything you need: `armor_stage_of` (returns
  `Option` — **`None` is a bare mount, and a client must not draw clean
  steel on a naked shoulder**), `armor_wear_of`, `ArmorStage::label` /
  `::tilts` / `::resist`, `ArmorPiece::struck`, `HitZone::band`,
  `ArmorCondition::{hp,frac,stage,wear,repair}`.
- **`ArmorStage::tilts` exists and nothing draws it.** "Piece tilts at
  Cracked" is a brief requirement with a published accessor and no reader.
- Detach is already the *unequipped* path — `armor_pieces.set(p, false)`,
  the same bit the Forge switch clears — so **you need no second
  visibility rule.** `a_shot_off_plate_is_indistinguishable_from_one_never_worn`
  will notice if that ever grows a second flag.
- Brief IX-C gives you the visual language: Fresh = clean plate; Scuffed =
  light scratches, edge dulling; Cracked = deep gouges, fracture lines,
  loose rivets; Severed = detached or hanging.
- The 24 plate groups already exist as separate geometry and visibility
  already follows the loadout — **you are changing materials, not building
  meshes.**

**Acceptance check:** a capture at 100% / 50% / 15% plate shows three
different surfaces. `CapBeat.hull` already stages it. Right now that
capture is *impossible*, not merely missing.

## TASK 6 — The six honesty fixes, one sitting
`TRV-0031`, `0032`, `0033`, `0024`, `0028`, `0055` · **friday33 + friday22** · **1 session** · BRIEF XII §8's own findings

| Row | The defect, re-derived today at `46d6dbb` |
|---|---|
| `TRV-0031` | **NOT fixed, despite a report that it was.** `main.rs:25489` `let modes = format!(... TDM_TARGET, ...)` — the **constant**, not `game.sim.cfg.tdm_target`. The comment twelve lines above says *"Every number below is the LIVE constant, never a retyped copy"*; this line was missed. It prints 30 while `frontend::INTRO_TDM_TARGET` = 25 is what an intro match plays to. **The screen lies to the player.** |
| `TRV-0024` | `TDM_TARGET_CHOICES` (`sim.rs:430`) — its only other mentions are two doc comments (`sim.rs:5389`, `frontend.rs:321`), one of which calls it a known mistake. **No executable reader.** Fixing this and `TRV-0031` together is one change. |
| `TRV-0032` | `BIND_REGISTRY`'s only `U` row (`main.rs:5414`) is *"Dismount the mech (chassis is scrapped; the pad respawns)"*. The in-world prompt says **"U - GRAB THE HULL"**. Hull climbing is not in the full bind list. |
| `TRV-0033` | The `Q` row (`main.rs:5397`) names roll and flip only. **The Agile's second flip charge and its mid-air jump appear nowhere a player can find them — and BRIEF X made them its mechanical identity.** Put them on Controls, **not** the equip hint; that line already overflowed once and was cut to two facts for exactly that reason. |
| `TRV-0028` | `FORGE_SLOTS` (`main.rs:1520`) — declared, read by nothing; `forge_slot_path` and its callers all hardcode. |
| `TRV-0055` | **Half fixed.** `hud.rs:190 fn heat_pct` now names the two-scale bug in its own doc comment and gives the HUD one policy. **Still live:** `main.rs:22825` prints `HEAT {:.0}%` from the **raw** value while `:22807`/`:22813`/`:22816` print `× 100` under the same `%`. The sim-side split is friday22's. |

**Why these matter more than their size:** every one is the game telling
the player something false. `ANTI_PATTERNS.md` has a name for the class —
**"the confident narrator"** — and it was earned here.

## TASK 7 — Capture scripts as DATA, not code
`TRV-0040`, `TRV-0204`, `TRV-0251` · **friday33** · **1 session** · Thor's highest-leverage finding

`struct CapBeat` (`main.rs:5531`) — every script is still a compile-time
const array inside a 29,000-line file. A framing tweak costs a **full
release rebuild: ~6 minutes, versus ~40 seconds if the beats were data.**

Your own words, quoted in `THOR_LOG.md`: *"several tasks needed 3+
iterations purely on camera framing."* Three iterations × 6 minutes = 18
minutes of pure rebuild to move a camera, **and it recurs on every task in
this file that owes a capture** — TASK 2, TASK 5, TASK 8, and every one of
BRIEF XI §19's 33 proof boxes.

**Three properties of the rig, learned the hard way, that must survive:**
- The boom anchors on the **HEAD**, so closing distance magnifies the
  offset between anchor and subject.
- **Pitch orbits the CAMERA about the anchor** rather than tilting the
  view, so positive pitch photographs the top of a hat.
- `look` turns the PLAYER and the third-person boom is rigidly behind the
  player, so **no yaw ever yields a profile.** `CapBeat.orbit` swings the
  boom around a stationary subject **and re-aims at the anchor** — the
  first attempt did not re-aim and photographed the scenery beside the
  machine.
- Beat times must not run backwards; every script's last beat must set
  `end`. Both are pinned by tests.

**Acceptance check:** a framing change to one beat requires no
`cargo build`.

## TASK 8 — Lift the Royal's six colour literals to consts
`TRV-0370` · **friday-mech** · **~1 hour** · prerequisite for D12(d)

Thor measured the two Royals at **1.007× hull luminance** (ally 0.0567,
foe 0.0563) where the Agile's own guard demands ≥ 8×; the strict form
fails 6.7× the wrong way. **There is no guard test on the Royal palette at
all**, and there cannot be: the colours are inline `Color::srgb` literals
inside a `materials.add` block (`main.rs:15380`/`:15398`).

**Do the lift now regardless of how D12(d) is ruled** — it is the
prerequisite either way, it is mechanical, and it converts an unmeasurable
surface into a testable one. `agile_mech.rs` already shows the shape:
`pub const` palette data with
`the_enemy_agile_never_out_luminates_the_ally` (`:744`) plus a gamma-decode
anchor test so the guard cannot pass on a broken formula.

**Acceptance check:** the six literals are `pub const`s and one test pins
ally/enemy Royal separation. **Do not recolour in the same commit.**

---

# BAND 2 — SPEC15 P2/P3: the owner's own priority order

## TASK 9 — Build the Royal ARROW LAUNCHER
`TRV-0010` · **friday33 + one toto number** · **1-2 sessions** · SPEC15 **P2 — the last open P1/P2 row**

**Your words:** *"Royal ARROW LAUNCHER: minigun + 3 crossbows. Compact
minigun silhouette, rotating mechanism, three crossbow assemblies around a
central weapon, bolt ammunition, mechanical loading."*

**Re-derived today: `arrow_launcher` / `ArrowLauncher` / `crossbow` return
two hits across all of `src/`, and both are prose comments about a bow
reading like a crossbow in first person (`main.rs:396`, `:9648`). Zero in
any weapon path.** `MechWeapon` = Gatling / Autocannon / Rockets / Plasma /
Repair.

**This is the only mech weapon the Royal tier would have that the Big does
not.** Right now a Royal is a Big Mech with more hull — exactly what P3
says it must not be.

**What the thread already knows:**
- `MechWeapon::for_set` is the DATA row that decides which mounts a chassis
  has. **Add there, not in a forked spawn function.** SPEC15 trap 1.
- `a_number_key_selects_the_mount_the_strip_labels_with_it` exists because
  a hardcoded `0 => Gatling, 1 => Rockets` once made the medic's repair
  beam unreachable. **The strip and the key handler must read one list.**
  That test will catch you if they drift.
- The turret already proves "rotating mechanism you can see":
  `main.rs:10289` `§owner: IT HAS TO READ AS A MINIGUN`. **The trap it
  records is the one to avoid on the crossbow assemblies** — a "half-cowl,
  open below" built from cylinders facing down the barrel axis, i.e.
  **discs**, which capped the gun and hid all six barrels.
- SPEC15 trap 3: **silhouette beats paint.** Three crossbows around a
  central minigun *is* a silhouette element. Use it — it is the cheapest
  answer to D12(d) that exists.

**The one number nobody has → toto:** rate of fire, bolt velocity, and
per-bolt damage against a mech hull. Nothing in the game fires a bolt from
a mount, so there is no neighbouring value to interpolate from. **The
dispatch must name that number, not the topic.**

**Acceptance check:** a Royal pilot selects it from the strip by number; it
fires bolts with their own physics; the barrels visibly rotate; there is a
capture of it firing.

## TASK 10 — BRIEF XI §1: generalise `solve_arm_ik` to the legs
`TRV-0319`, `TRV-0322` · **friday33** · **1-2 sessions** · gated on **D10**

`solve_arm_ik` (`main.rs:2585`) has **10 call sites and every one is an
arm** (`:18459`, `:18488`, `:18536`, `:18567`, `:18651`, `:18663`,
`:20184`, `:20187`, `:25936`). There is no `leg_ik`, no `foot_place`, no
`solve_leg` anywhere in `src/`.

`DECISION.md` item 3 specifies this exactly and **BRIEF XI §0.2 makes it
binding: do not write a second solver, do not add a crate.** This is the
single highest-value BRIEF XI item and it closes §1, most of §2, and the
foot half of §19.

**Acceptance check:** feet stay on a slope; **consecutive-frame** capture
shows no sliding. BRIEF XI's proof standard names sliding and clipping as
the two claims most likely to be ticked without evidence, and says
*"capture consecutive frames, not one pose."*

## TASK 11 — The Agile's double jump has no airborne pose
`TRV-0321`, `TRV-0324` · **friday-mech** · **half a session** · BRIEF XI §0.4 calls it *"the highest-value single fix in this brief"*

`try_mech_jump`'s compress/tuck is **heavy-only**, so the apex frame is
indistinguishable from standing. Your Agile's signature mechanic is
currently invisible. **The captures already exist to prove the fix**:
`agile_moves/07-double-jump-kick.png`, `08-double-jump-apex.png`.

Thor's note to carry: the apex frame was not shot at the actual peak —
re-time the jump trio while you are there.

**Acceptance check:** re-run `agile_moves`; frames 07/08 differ from
`01-standing`.

## TASK 12 — SPEC15 P3, one bullet per session, in your order
`TRV-0011` Agile visual upgrade → `TRV-0012` Rocket Launcher → `TRV-0013`/`0015` Royal body → `TRV-0014` opposition body · **friday-mech** · **1-2 sessions each**

**Four of these five bullets asked for GEOMETRY and received PAINT.** Take
one per session. Do not merge them. Hand every one:
- **Trap 3 — silhouette beats paint.** A variant identifiable only by
  colour does not exist at range.
- **Trap 4 — the luminance rule is unguarded** *on the Royal*. The Agile's
  is now guarded (`agile_mech.rs:744`) and that is the pattern; see TASK 8.
- **Trap 5 — capture everything.** Five defects in one week were invisible
  to the compiler and to 478 tests and obvious in a screenshot.

On `TRV-0011` specifically, the decision to revisit **deliberately, not
delete**: the medic pass took ~90 exposed struts down to ~45 nameable
masses and replaced the digitigrade frame with plantigrade thick legs —
*because the digitigrade frame read fast but also skittish, and half its
hardware existed only to explain itself.* If you now want FAST again, that
is the trade to reopen on purpose.

---

# BAND 3 — REAL, OPEN, NOT NOW

Listed so nothing disappears. Do not start these while Bands 0-2 are open.

| Task | Rows | Lane | What the ledger knows |
|---|---|---|---|
| Mech boarding: 7 of 8 stages made visible | `TRV-0037`, `0174` | friday33 | **The strings already exist verbatim inside the `debug!` calls** (`main.rs:19243-19254`) — that is the spec, already written by the person who built the timer. The system fires only on a stage CHANGE, never per frame; **do not break that.** `visor_ready` is still a field of a `Local`, so nothing outside its own system can read it (`TRV-0057`) — if the camera cut is meant to be real, that flag has to leave the `Local`. |
| Traversal / ledge bands | `TRV-0051`, `0180` | friday22 | **UNBLOCKED THIS WEEK.** Their named blocker was `MAP_METRICS.md`; it exists, 1025 lines, with §4.3 "THE LEDGE BANDS". Nobody has been told. |
| Bot navigation, properly | `TRV-0043` | friday22 | Also unblocked by `MAP_METRICS.md`. BOT ROUTING landed for Cliffhold with published up-links and `BOT_PROBE_Y`; `sim.rs:27649` states the flat maps were **deliberately** left alone. |
| Arena + Gardens: the +10%/elevation/randomisation pass | `TRV-0041`, `0042`, `0379` | friday22 | The **Bailey got it** (`e7f408e`). **The trap, and hand it to whoever builds:** randomised structures must be seeded at map-BUILD time from the match seed. Drawing from the gameplay RNG stream shifts every later number and breaks replay for every other system. |
| Finish deleting Cliffhold | `TRV-0039` | **both — coordinate or the `match` breaks** | Client half went in `4152240`; ~50 references survive in `sim.rs` including `build_cliffhold` and five reachability tests. **Salvage first:** the +25% scale trap, the flight-joint bug, and the reachability-test shape are all reusable. |
| Fix the sightline validator | `TRV-0374` | friday22 | It is **flat-map-only** and samples at `half/10`, so Battlefield quantises at ±35.4 m and Cliffhold at ±42.4 m. `MAP_METRICS.md` §6.2 marks it *"a defect, not a caveat"*. Blocks a precise answer to D9. |
| Run BM-1 / BM-2 | `TRV-0373` | friday22 | `DECISION.md` §9.2 specifies the pose-kernel microbenchmark **in full**; BM-2 is ~1 hour on the existing crowd bench. **The architecture decision was made without its own named instrument** and the document says so. |
| The rest of BRIEF XI — §7-§14 limb/hand/grip families and the gun art pass | `TRV-0328`..`0335` | friday33 | §13 wants a shared `Weapon → Grip Point → Hand IK → Arm → Shoulder` chain to *"remove the need to hand-tune hand positions per weapon"*. **No `grip_point`/`GripPoint` symbol exists**, and `main.rs:20184-20187` drives both wrists from per-weapon offsets — the hand-tuning §13 wants to retire. See also `TRV-0372`: **whoever builds hands has no attachment point to build against** (`bow_nock_local()` is a function, not a socket). |
| Two tests that cannot fail | `TRV-0371` | friday33 | `Vec3::new(0.050,0.020,0.050)` / `(0.020,0.010,0.060)` at `main.rs:21665-21668` are re-typed at `:26278` and `:26346`. **Two literals want to be one const.** OPERATION rule 12. |
| Commit the `muzzle_flash` captures | `TRV-0375` | friday-fx | `handback/brief-vii/muzzle_flash/` is **untracked**. One bare `git stash` and it is gone. |
| Delete the two comments that argue against Q4 | `TRV-0363` | friday33 | `main.rs:15301`, `main.rs:4005`, both present tense, both about a spec that no longer exists. |
| Character creation L0-L4 · Forge per-piece grid · weapon crafting | `TRV-0181`, `0045`, `0076`, `0136`, `0138`, `0139` | friday-front | **`BACKLOG.md` #9's stated blocker is known FALSE.** It says "no class system and only 5 whole-body armour presets". Four classes shipped 2026-08-05; 24 per-piece plates shipped 2026-08-07. **Unblocked for six days and nobody noticed.** |
| Injury / fatigue / dynamic CoG | `TRV-0183` | friday22 | **`BACKLOG.md` #11's blocker is known FALSE** — the armour-weight formula was wired in the 24-plate pass. |
| Ragdoll + hit-reaction impulse | `TRV-0046`, `0106` | both | The rig's mass, length and inertia are complete and tested; `derived_spring_k` is the only consumer. |
| glTF loader for `jk_tdm` | `TRV-0048`, `0262` | friday33 | **The blocker is named and it is not the owner:** the shipping crate cannot load a mesh if pointed at one. Only `jk_bevy` can. This is the real task behind "your uploaded gun assets". |
| §19 HUD redesign remainder | `TRV-0358`, `0359`, `0360` | friday-hud | **Deliberately last.** TASK 5 and the boarding beats both add elements; redesigning a growing HUD means doing it twice. |

---

# BAND 4 — BLOCKED. DO NOT START.

Each has a named unblocker that is neither difficulty nor effort.

| Row | Blocked on |
|---|---|
| `TRV-0361`, `TRV-0362` | **Two HUD reference images.** But note: `handback/reference/hud/NOTES.md` describes both in prose and declares itself the working reference until the PNGs land. **A builder may proceed.** Only the acceptance check is blocked. |
| `TRV-0309`..`0312`, `TRV-0260`, `TRV-0261` | Six owner images. **Nobody has written these four bow/spear ones down in prose the way the HUD ones were.** Somebody should, today — it costs ten minutes and it is the difference between blocked and merely unproven. |
| `TRV-0318`, `TRV-0326` | D1. |
| `TRV-0325` | D2. |
| `TRV-0376` | D10. |
| `TRV-0377`, `TRV-0117` | D9. |
| `TRV-0050` Networking | A networking dependency, and a decision to have one. The deterministic sim and bit-identical replay are the right foundation; nothing else exists. The scoreboard deliberately omits a Ping column for this reason. |
| `TRV-0125` Grenades in water | No water volume exists in any map. |
| `TRV-0185` Mud / sand / snow / ice grenade surfaces | None exists as a `CoverKind`. **Do not research the coefficients now** — they go stale before the blocker clears. |
| `TRV-0182` Destruction | rapier supports some of it; there is still no *design reason*, which is the honest blocker. |
| `TRV-0150` Powered armour | Research-only by your instruction. **The "do not build" half was obeyed; the "produce the spec" half was never started** and `research/powered-armour/` does not exist. Recorded so the want does not vanish: you said you want this **in the future**. |
| `TRV-0163` In-game console + image import | The image-import half is blocked upstream on the texture pipeline. **The console CORE — cvar registry, autocomplete, history, scrollback — is blocked by nothing** and could ship alone if you want it. |

---

# LANE SHEETS

**friday33** (`main.rs` — dirty, coordinate first): TASK 1 → TASK 2 →
TASK 6 → TASK 7 → TASK 4 → TASK 9 → TASK 10.

**friday22** (`sim.rs` — free): TASK 3 → TASK 6's sim half (`TRV-0055`,
`TRV-0024`) → `TRV-0029` (the `9000.0` spray literal, still bare at
`sim.rs:4515`/`:5481` with the client keeping its own copy at
`main.rs:308`) → `TRV-0235` (plate wear fires only on the zoned hitscan
path — grenades, melee, claws and gas neither wear plate nor are reduced
by it; **one gate, `in_mech`, for both, so they cannot drift**) →
`TRV-0241` (the player/bot asymmetry in `punched_aim_stabilised`, which
the builder calls *"the real root"* and which applies to every recoiling
weapon) → `TRV-0242` (a bot chassis now never raises its barrier at all)
→ `TRV-0316` (`step_plasma_precision`, `sim.rs:8859`, still charges on
`cmd.ads` — **the one remaining RMB-charge in the file**; cross-lane,
needs the client's mount router to move too, so it needs both Fridays or
an owner "leave it").

**friday-hud** (`hud.rs` — free): TASK 5.

**friday-mech** (`agile_mech.rs`, `mech_lineup.rs`, `mech_recoil.rs`,
`cockpit.rs` — free): TASK 8 → TASK 11 → TASK 12.

**thor**: see `TREVOR_LEDGER.md` §E.1. Top of that list —
`TRV-0345`..`0353` (nine HUD rows delivered on commit evidence, **and I
opened no HUD PNG**) and `TRV-0379` (**does the Bailey's randomisation
draw from the match seed or the gameplay RNG?** If the latter, replay is
broken for every other system and nobody will notice for weeks).

**toto** — **two rows, and only two.** `TRV-0010` (arrow launcher rate of
fire, bolt velocity, per-bolt damage vs a mech hull) and `TRV-0126` ("how
enclosed is this point" — **ask for the METHOD and its cost at 120 Hz, not
a percentage**). Everything else that looks like a research need is a
decision in Band 0 or a build in Bands 1-2.

---

# THE THINGS NOBODY IS TRACKING

Not tasks. Facts about the record that will cost a session if nobody says
them out loud.

1. **`WHATS_MISSING.md` is still described as "the live plan" and has not
   been touched in twenty commits** (last: `7719296`, 2026-08-10). It has
   now gone stale for the **fifth** time and knows nothing of BRIEF XI,
   BRIEF XII, BRIEF XII-A, the bow/spear spec, the front end, `hud.rs`,
   `MAP_METRICS.md` or `DECISION.md`. **`OPERATION.md` still sends every
   new session to it.**
2. **Two commit subjects this week mark unbuilt briefs as done**
   (`TRV-0366`). `dd4fced` — *"BRIEF XI: **finish** the Agile Mech"* —
   adds one file and changes no code. `git log --oneline` is what every
   new session reads and it is the cheapest status instrument in the repo.
   **A brief being written to disk is a good thing; say "BRIEF XI: the
   spec", not "finish".**
3. **`BACKLOG.md` #4/#5/#9/#11 are known false** (melee depth, AI retreat,
   the class system's blocker, the armour-weight wiring — all four
   shipped). Index it; never rank from it.
4. **Five documents describe a game that no longer exists**:
   `GAME_STATUS_REPORT.md` (2026-08-01, still lists Pyro),
   `handback/brief-ix/REPORT.md`, `DECISIONS.md` (every ADR is about
   `jk_wall`/`jk_core`), `README.md:223-225` (the retired 8v8 cap), and
   `briefs/README.md` (lists neither BRIEF XI, XII nor XII-A). None is
   wrong on purpose; every one was true when written. **That is the whole
   reason this ledger exists.**
5. **`MAP_METRICS.md` set a new standard and the other ten `SOURCES.md`
   should be held to it**: every row labelled MEASURED / DERIVED /
   ASSUMED inline, the instrument's precision ceiling published, and the
   instrument's own defect named rather than hidden. Cheap doc work for
   anyone idle.
6. **The `§owner` doc-comment convention is still the best archival
   practice in this repo** — 44 chat asks survive *only* because someone
   wrote `§owner` next to the code. One warning: the marker has
   **collided** (`TRV-0315`). `§8/§9 (owner)` at `sim.rs:357`, `:388`,
   `:6683`, `:21068` belongs to the 30-section gameplay spec of
   2026-08-07; `§9/§10/§11 (owner, BOW & SPEAR)` belongs to a different
   document. Same file, same marker, two specs. **Every `§N (owner)`
   should name its spec.**
7. **Writing an image down in prose before it arrives works** (`8bbb954`).
   It converted two blocked rows into two unproven-but-workable ones. Do
   it every time an image is mentioned and cannot be saved.
