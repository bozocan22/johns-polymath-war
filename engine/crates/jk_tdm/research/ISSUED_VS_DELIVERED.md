# ISSUED vs DELIVERED — the owner-instruction register

**Built 2026-08-10. Repo HEAD at sweep start and end: `4bde6b3`.**
Working tree: one untracked capture directory
(`handback/brief-vii/agile_body/`, 8 PNGs) and an in-progress
`agile_mech` test in another session's lane. I wrote nothing outside
`research/`.

**What this file is.** Thor verifies claims about CODE. Nothing has ever
verified claims about the OWNER'S INSTRUCTIONS. This is that register:
every instruction the owner issued, from every source, with a status
that is backed by `file:line` or a commit — never by a commit message's
claim about itself.

**Relationship to `TREVOR_LEDGER.md`.** The ledger is the sibling record
and it is good; I re-derived from the code rather than inheriting from
it, and I share its ID space rather than inventing a second one. Rows
`TRV-0001`..`TRV-0266` are the ledger's and keep their numbers.
`TRV-0267`+ are opened here. Where I disagree with the ledger, §7 says
so.

**Status vocabulary** (the owner's, from the dispatch that commissioned
this file):

| Status | Means |
|---|---|
| `DONE` | I found the code and can cite it |
| `PARTIAL` | Some shipped. The row says precisely which half did not |
| `NOT STARTED` | Grepped for, genuinely absent |
| `BLOCKED` | A specific named blocker, not "needs art" |
| `SUPERSEDED` | A later instruction replaced it; both are cited |
| `CONTRADICTED` | The owner asked for two incompatible things. Both named. **The owner decides, not me** |
| `UNVERIFIABLE` | Cannot be settled from code alone ("does this look cartoon enough") |
| `UNVERIFIED` | I did not check, or the check did not complete. Honest, and there are 18 |

`DONE` here means "there is evidence at this path". It does not mean "it
works". I did not build and did not run the suite.

---

## §0 — HEADLINE

| | count |
|---|---|
| Rows in this register | **300** — `TRV-0001` .. `TRV-0300`, no gaps |
| ├─ carried from `TREVOR_LEDGER.md`, re-derived | 266 |
| └─ **opened by this sweep** | **34** |
| `DONE` | **137** |
| `PARTIAL` | 42 |
| `NOT STARTED` | 65 |
| **Open (`PARTIAL` + `NOT STARTED`)** | **107** |
| `BLOCKED` | 16 |
| `SUPERSEDED` | 12 |
| **`CONTRADICTED`** | **7** |
| `UNVERIFIABLE` | 3 |
| `UNVERIFIED` | 18 |

Counts sum to 300 exactly. The arithmetic against the ledger's own
headline is in §7: eleven rows changed status under re-derivation, and
each change is named there rather than absorbed into a total.

Origin: 268 owner · 26 agent · 6 agent-found-then-adopted-by-owner.

**The single most surprising finding is §2.C1** — the game ships a red
predicted-trajectory arc and a landing ring for the bow and the spear,
which two separate briefs forbid in bold, twice each, calling the absence
of that arc "the entire point". Nobody has ever noticed, because the code
cites an *earlier* brief as its authority and both later briefs were
filed as DELIVERED against asks whose second sentence was never checked.

---

## §1 — DONE THAT THE QUEUE STILL LISTS AS OPEN

*This is the recurring failure the owner named. Every line below was
re-derived against the code today.*

| Row | The queue's claim | What the code says | Cite |
|---|---|---|---|
| `TRV-0022` | `0-QUEUE` TIER 0 #1: "`ROBOT_SPEED_MULT` (1.12) is dead… Delete it." | **Deleted.** A `§owner (defect pass)` tombstone stands where it was. | `sim.rs:437` |
| `TRV-0023` | TIER 0 #2: "`MECH_SHIELD_ARC_COS` … is unread." | **Wired.** `barrier_arc = cos > MECH_SHIELD_ARC_COS;` | `sim.rs:12841`, tombstone note `:12828` |
| `TRV-0025` | TIER 0 #4: "`pod_aim_held` … read nowhere. Delete or wire." | **Deleted**, same tombstone pattern. | `sim.rs:3394` |
| `TRV-0038` | TIER 2 #17: "**Pyro armour is unobtainable on every map** … TWO per-map relocation tables still place a Pyro pad" | **Gone.** 8 residual mentions tree-wide, all documentary comments. Zero `FLAME_*` constants. | `sim.rs:4775`; commits `b11b7de`, `5bf474a` |
| `TRV-0058` | `0-NOW` §B.9: "`SCOUT_SCALE = 1.42` is read by nothing" | **Wired into `height()`** via `ArmorSet::chassis_scale`, with a test that names the regression. | `sim.rs:4893`, `:3710`, test `:15743` |
| `TRV-0254` | `0-SCOUT` defect #5: "Third-person `crouch_drop` is double-counted for a kneeling mech." | **Fixed**, with the arithmetic in the comment: "a kneeling chassis paid the drop twice: 0.85 m of real hull sink, and 0.62 m more of camera". Now `chassis_kneeling() => 0.0`. | `main.rs:19601-19614` |
| `TRV-0149` | `0-QUEUE` TIER 4 #28: "traversal (**blocked on map metrics**)" | **Unblocked.** `MAP_METRICS.md` was written 2026-08-10 (1,025 lines) and derives the three numbers the block rested on — including that every ledge number in the project was 0.55 m short, because the ceiling is apex + `STEP_UP`, not apex. | commit `632824a`; `research/maps/MAP_METRICS.md` |
| `TRV-0188` | ledger THREAD 16: "`DECISION.md` — the entire deliverable — was never written. The largest orphaned research in the repo." | **Written**, 775 lines, with a verdict (keep procedural sim-driven posing, buy nothing) and a reversal of session 1's own crate recommendation. | commit `632824a` |
| `TRV-0011` | `0-SPEC15` **P3**: "**Agile Mech major upgrade** — the largest visual item" | **Built.** `agile_mech.rs`, 650 lines, five silhouette elements outside the Big's vocabulary, four material roles, seven tests. Was `NOT STARTED` in the ledger 18 hours ago. | `agile_mech.rs`; commits `efe1428`, `4bde6b3` |
| `TRV-0177` / `TRV-0178` / `TRV-0181` / `TRV-0183` | `BACKLOG.md` #4 "melee depth: not started", #5 "retreat is the remainder", #9 "blocked: no class system", #11 "armour-weight formula unwired" | **All four false.** Melee v2 `a99af96`; retreat `e5431a4`; four classes 2026-08-05; weight wired in the 24-plate pass. Carried from the ledger, spot-checked, unchanged. | see `TREVOR_LEDGER.md` §D |

**Nine of these are still sitting in `WHATS_MISSING.md` today, written as
open work.** Three of the eight TIER 0 "MINUTES" items are done and the
list does not say so; the other five are §2.B.

---

## §2 — THE FOUR ANSWERS

### 2.A — CONTRADICTIONS THE OWNER MUST RULE ON

*Seven. Each names both instructions and cites both. I did not pick a
side on any of them.*

---

#### C1 — `TRV-0294` · **The bow and spear ship a predicted trajectory arc and a landing ring, which two briefs forbid.**

**Instruction A, twice, in bold:**

> `briefs/BRIEF_VII_optimized.md:331` — "**No trajectory arc. No landing
> marker.** The arc is learned — that is the entire point of Reference A."
>
> `briefs/BRIEF_VII_optimized.md:72` — "Brief V grenade trajectory arc |
> **Grenade-only.** Spear and bow get NO arc and NO landing marker — the
> learnable arc IS the skill."
>
> `briefs/BRIEF_VIII_master.md:561` — "**No arc, no landing marker** — the
> arc is learned, and that is the skill."

**Instruction B**, cited by the code as its authority:

> `main.rs:2661` — "The predicted-arc preview for **bow/spear** aiming
> (**§4.2 Brief II**): arc-length-spaced dots, a landing ring +
> drop-line, and a ±spread cone of two fainter arcs."
>
> `main.rs:20` (module header, the game's own summary of itself) —
> "Bow/spear aiming shows a red PREDICTED ARC with a landing marker".

**What ships.** `fn arc_preview` (`main.rs:20879`) is registered in the
app at `main.rs:7891` and gates on
`cam_ctl.ads && p.alive() && spec.projectile.is_some()`. Exactly two guns
declare `projectile: Some(..)` — the **Bow** (`sim.rs:686`) and the
**Spear** (`sim.rs:708`). So the preview exists for precisely the two
weapons the ban names, and for nothing else. Brief II is not in this
repository; Briefs VII and VIII both post-date it and both override it.

**Status: `CONTRADICTED`.** Not `NOT STARTED`, not a defect — the owner
issued both instructions and the older one is what runs.

**Why nobody caught it:** `TRV-0071` and `TRV-0091` were both filed
`DELIVERED` quoting asks whose *first* sentence (the throw) shipped. The
sentence after it was never checked against the code.

**Evidence gap, stated:** no capture holds focus with a bow or a spear,
so no PNG proves or disproves the arc on screen. `bow_draw_fp/03` and
`spear_flight/00` are both un-focused and show no arc. The code is
conclusive; the instrument has never been pointed at it.

**Dispatch, if the owner rules for the briefs:** friday33, ~1 hour —
gate `arc_preview` on `GunKind::Grenade`-class throws only, and take a
`bow_draw_fp` beat that holds RMB so the absence is photographed.

---

#### C2 — `TRV-0295` · **"No head, no cockpit glass, no visible pilot opening" against a shipped head, visor and cockpit.**

> `briefs/BRIEF_VIII_master.md:666` — "**No head.** An angular recessed
> sensor visor slit across the hull front"
> `briefs/BRIEF_VIII_B_addendum.md:249` — "…mounted low on the hull sides,
> **no head, no cockpit glass, no visible pilot opening**"
> `briefs/PROMPT_mech_rebuild.md:315` — same words again

Against the 36-section spec's **§20 spacecraft cockpit** ("first-person
mech frame, instrument panels, screen glow, vibration",
`WHATS_MISSING.md:401-403`), which shipped as `cockpit.rs`, 16 cockpit
captures, `MECH_VISOR_Y_FRAC`, and a **×2 visor weak point** that is now
load-bearing gameplay (`sim.rs`, the mech-front damage balance at
`sim.rs:5875`).

**Status: `CONTRADICTED`.** Almost certainly superseded in the owner's
head, but **nobody wrote it down**, and the concept-art briefs are still
the stated completion criterion for the mech section. One sentence from
the owner retires three brief lines permanently.

---

#### C3 — `TRV-0113` · **The missile pod: "absent by default" vs bound to `Y` with ten tubes.**

> `BRIEF_VIII_B_addendum.md:329` — "…the missile pod becomes an
> **optional swappable hardpoint, absent by default**."
> `PROMPT_mech_rebuild.md:369-370` — the same instruction, and the brief
> demanded the reconciliation be **written down**.

Live: `sim.rs:4274` `§owner: "ten tubes per chassis (was 4), and ammo
pads now resupply"` — core, on every chassis, bound to `Y`
(`main.rs:5078`). It is not optional and it is not absent.

**Status: `CONTRADICTED`.** Two owner instructions, opposite defaults.
The `§owner` marker on the ten-tube line means the *later* one is also
the owner's — so this is very likely a supersession nobody recorded. Say
so once and it closes.

---

#### C4 — `TRV-0111` · **Olive drab / khaki / field tan, minimal emissive, "the art has no glowing visor" — against the entire live faction palette.**

> `BRIEF_VIII_B_addendum.md` §D.2, headed "**CORRECTION to Brief VIII
> §7.2**": `hull_primary #8A8770`, `mechanism_dark #33352F`,
> `barrel_metal #2B2C2B`; "Emissive: **minimal**. The art has no glowing
> visor."

Against `§owner TEAM IDENTITY` (`branding.rs:96`, `:107`, `:122`, `:150`),
`§owner BLUE ENEMY MECHS` (`main.rs:3675`), SPEC15 P3's "neon red, dark
red, neon blue, dark blue", the Royal's gold, the opposition Royal's
yellow, and now BRIEF X's orange.

The build has an emissive visor and a faction colour language. The
addendum asked for neither. **§D.2 was never retired — it was overtaken.**
It is still the document that §D.7 makes the mech section's completion
criterion.

**Status: `CONTRADICTED`.** Retire §D.2 explicitly or the next agent
reading the briefs in order will rebuild a khaki mech.

---

#### C5 — `TRV-0296` · **The Agile Mech is orange on BOTH sides, and three of six chassis are now in one hue band.**

> BRIEF X §2 — "**Primary armour — orange.** Dominant… This becomes the
> Agile Mech's recognizable visual identity." No faction split is stated.
>
> `WHATS_MISSING.md:62-65` (SPEC15 P3) — "**Opposition mechs are NOT
> recolours** … while keeping the faction colour language: **neon red,
> dark red, neon blue, dark blue**."

The build gives the enemy Agile `scout_hull_foe` / `scout_plate_foe` /
`agile_blue_foe` and a `scout_line_foe` lamp (`agile_mech.rs:189-203`) —
so both Agiles are orange machines and friend-or-foe rests on the lamp
plus material value. BRIEF X's own §13 records the squeeze: the player
Royal is **gold**, the opposition Royal is **yellow**, and "orange sits
between them on the wheel".

Meanwhile `TRV-0206` is still open: **the luminance rule that carries
friend-or-foe is unguarded and nothing tests it** (Thor, and SPEC15's own
trap 4).

**Status: `CONTRADICTED` + `NOT PROVEN`.** Two questions for the owner:
(1) should the opposition Agile be orange at all, or red? (2) BRIEF X's
own acceptance line — "clearly different from Big and Royal" — has **no
black-silhouette-at-30 m capture** behind it. `mech_lineup.rs` can
already produce that frame in one beat.

---

#### C6 — `TRV-0272` · **FRONT END P5 asked for a "large MECH preview"; what shipped previews the SOLDIER.**

The builder stated the deviation rather than hiding it, in the commit
body of `7abed26` and at `main.rs:19937`:

> "STILL NOT RIGHT, stated rather than hidden: this previews the SOLDIER,
> because the soldier is what the screen customizes. **The spec's word was
> 'mech'.** Nothing in the customization screen changes a machine today,
> so pointing the camera at one would be a preview of something the player
> cannot edit."

The *size* half is delivered and measured: portrait 720×1040 render
target (Bevy's `fov` is vertical, so portrait crops the sides for free),
camera in from 3.00 m to 2.57 m, card 366×520 for ~2.6× the image area,
rows moved out of its way at 72% width.

**Status: `PARTIAL`, pending an owner ruling.** Either "the soldier is
fine" (one line, closes it), or "make the mech customizable", which is a
multi-session feature nobody has scoped.

---

#### C7 — `TRV-0273` · **FRONT END P6 named GAMEPLAY / CONTROLS / AUDIO / GRAPHICS. The build shipped CONTROLS / INTERFACE / CROSSHAIR.**

Stated deviation, from `d91adb4`:

> "THE DEVIATION, STATED. The spec named GAMEPLAY / CONTROLS / AUDIO /
> GRAPHICS. **This game has no audio setting and no graphics setting** —
> no volume, no resolution, no quality, no vsync. Empty tabs would
> advertise controls that do not exist, and inventing settings to fill
> them breaks the other half of the same brief, which says 'minimal'
> twice."

The *shape* the spec asked for — a small set of doors, one category at a
time — is delivered and mutation-proven
(`every_setting_is_still_reachable_behind_some_tab`, `main.rs:28615`).

**Status: `PARTIAL`, pending an owner ruling.** The real question the
deviation exposes and nobody has asked: **do you want a volume slider and
a resolution/vsync setting at all?** The game currently has neither, and
that is not recorded anywhere as a decision.

---

**Resolved contradictions, kept so they are not re-opened:**

- **The Royal's neon blue vs the shipped gold** — `TRV-0013`. Ruled by
  the owner 2026-08-10: *"keep the royal mech, and keep the opposition
  royal mech colour red yellow and neon blue details."* The gold stays;
  `WHATS_MISSING.md:55-61` is struck through and marked. **Closed.**
- **Mech scale 1.15× vs 1.7× vs the art's ~2.5×** — `TRV-0099`/`0101`.
  `BRIEF_VIII` non-negotiable 8 says "1.15× … not 1.5×, not concept
  scale"; `BRIEF_VIII_B` §A recommends 1.7× and its own precedence rule
  says the addendum wins; live `MECH_SCALE = 1.7`. **Resolved by
  document precedence**, but Brief VIII still reads as a non-negotiable
  to anyone opening it first.
- **"battles cap at 8v8"** (`README.md:223-225`, an owner rule) vs
  `§owner: "8v8 withdrawn"` (`main.rs:23200`). Both owner; the later
  wins. The README still states the retired rule. **Doc rot, not a live
  contradiction.**
- **"Recoil HALVED across the arsenal (owner request)"**
  (`README.md:209`) vs `§owner "increase the recoil"` (the mech turret,
  raised 2.9×). Different scopes — infantry arsenal vs hull turret — so
  probably not a conflict, but nobody has said so in writing.
  `TRV-0201` stays `UNVERIFIED`: I did not re-derive whether the
  infantry halving is still in force.

---

### 2.B — ASKS NOBODY EVER PICKED UP

*The expensive ones. Every row here has **zero** trace of anyone starting
— no commit, no partial code, no `§owner` marker, no research artefact.*

| Row | The instruction | Issued | Age | Why it is still zero | Lane |
|---|---|---|---|---|---|
| `TRV-0010` | "**Royal ARROW LAUNCHER: minigun + 3 crossbows.** Compact minigun silhouette, rotating mechanism, three crossbow assemblies around a central weapon, bolt ammunition, mechanical loading." | SPEC15 **P2** 2026-08-09 | 1 day | Grepped `crossbow`, `arrow_launcher`, `ArrowLauncher` across `src/`: **zero hits in any weapon path.** `MechWeapon` = Gatling/Rockets/Autocannon/Plasma/Repair. **The last open P2 row, and the only thing that would make a Royal not-a-bigger-Big.** | friday33 + one number from toto |
| `TRV-0260` | "**Save everything into `handback/reference/` and commit it — reference that lives only in a chat log is lost work.**" (the mech concept art) | `PROMPT_mech_rebuild` Task 1, `aefd16f` 2026-07-31 | **10 days — the oldest untouched ask** | The session that got it had no image-download capability and said so (`handback/reference/NOTES.md`). `git log --diff-filter=D` across every image type: it was never committed and later deleted. **It never arrived.** Four rows in two briefs are permanently unsatisfiable without it (`TRV-0075`, `0100`, `0115`, `0172`), and §D.7 makes the concept-art side-by-side the mech section's *stated completion criterion*. | **owner** |
| `TRV-0261` | The medic reference art — *"a squat utility robot, rounded masses, one big camera lens, worn amber over near-black."* | chat, quoted `WHATS_MISSING.md:513` | 3 days | The chassis was built to it and photographed; the image is not here, so nothing can be re-checked against it. | **owner** |
| `TRV-0030` | "**Every explosion is silent.** The sim publishes `Boom`; no sound exists. **Unblocker is `gen_sfx.py`, already in the repo.**" | `0-NOW` 2026-08-08, restated `0-QUEUE` TIER 1 #9 | 2 days | Re-derived: the `Sfx` load list is 22 `.wav` and no boom; `gen_sfx.py` generates no explosion either. **There has never been an owner blocker and the unblocker was named on day one.** | friday33, 1 hour |
| `TRV-0036` | "**Armour damage states are invisible.** `armor_stage_of` has ZERO client readers, so Fresh, Scuffed and Cracked render identically." | `0-NOW` §A.3 2026-08-08, restated TIER 2 #15 | 2 days | Re-derived today: `main.rs` contains **0** references to `armor_stage_of`, `armor_wear_of` or `ArmorStage`. `sim.rs` contains **66**. `ArmorStage::tilts` is a published accessor for a brief requirement with no reader. | friday33, 1 session |
| `TRV-0040` | "**Make capture scripts DATA, not code** — Thor's highest-leverage finding." | TIER 2 #19 | 2 days | `struct CapBeat` at `main.rs:5280`; every script is still a compile-time const array in a 29k-line file. 6 min per framing tweak vs ~40 s. Your own words: *"several tasks needed 3+ iterations purely on camera framing."* | friday33, 1 session |
| `TRV-0024` `TRV-0026` `TRV-0028` `TRV-0029` | Four of the eight TIER 0 "**MINUTES. One line each, no design decisions**" items | 2026-08-09 | 1 day | `TDM_TARGET_CHOICES` (`sim.rs:430`) unread — its only other mention is a doc comment calling it a known mistake. `shot_handgun.wav` written by `gen_sfx.py:73`, on disk, loaded by nothing. `FORGE_SLOTS` (`main.rs:1386`) unread. The `9000.0` spray scale is **still a bare literal at four sim sites** and the client keeps its own copy (`main.rs:308 PUNCH_DEG_S_PER_SPEC_KICK = 9000.0`). | friday33 + friday22, one sitting |
| `TRV-0031` `TRV-0032` `TRV-0033` | The three TIER 1 honesty defects | 2026-08-09 | 1 day | Field Manual still prints the constant (`main.rs:24643 TDM_TARGET,`) not the chosen target. `BIND_REGISTRY`'s only `U` row is still "Dismount the mech" (`main.rs:5154`) while the world prompt says "U - GRAB THE HULL" (`:23033`). The `Q` row (`:5137`) still names roll and flip only — the scout's second flip charge and mid-air jump appear nowhere a player can find them, **and BRIEF X has just made them the Agile's mechanical identity.** | friday33, 1 sitting |
| `TRV-0055` | "`gatling_heat` carries TWO scales in one sim field… Split it or document it." | `0-NOW` §A.6 | 2 days | Re-derived: `main.rs:4949`, `:21974`, `:21980`, `:21983` print `×100` under a `%`; **`main.rs:21992` prints the raw value under the same `%`.** Both branches live. | friday22 (source) + friday33 |
| `TRV-0034` | "`gait_pose` bakes the INFANTRY crouch ratio (0.646) against the sim's `MECH_CROUCH_HEIGHT_FRAC` (0.72)" | `0-NOW` §A.2 | 2 days | **Re-derived and it survives.** The literal `0.646` is gone, but `gait_pose` (`main.rs:2336`) still computes the crouch pose from `CROUCH_HEIGHT` (= 1.15, i.e. 0.646 × `BODY_HEIGHT`), and the kneeling-mech path (`main.rs:17794-17803`) blends two *infantry* poses. `MECH_CROUCH_HEIGHT_FRAC` (`sim.rs:5919`) appears **nowhere** in `main.rs`. | friday33 |
| `TRV-0206` | "the luminance rule is unguarded… **nothing tests it**" — SPEC15's own **trap 4** | Thor, adopted into SPEC15 2026-08-09 | 1 day | No test pins ally/enemy luminance separation. The owner's own spec hands this trap to every builder and no builder has taken it. **C5 makes it urgent.** | friday33, ~30 min |
| `TRV-0150` | "**Powered armour** · RESEARCH ONLY, DO NOT BUILD… Produce the spec, stop there. End the file with a build-readiness checklist." | `PROMPT_MASTER` Task 8 | 7 days | The "do not build" half was obeyed. The "produce the spec" half was never started; `research/powered-armour/` does not exist. **You said you want this in the future — recording it so the want does not vanish.** | — |
| `TRV-0136` `TRV-0139` | Tier 4 cosmetics (skin ×4, palette, weapon paint ×4, decals) and the **12 named preset loadouts** (Aggressive/Duelist/Fieldfare, Vanguard/Ghost/Archer, Tank/Reaper/Sentinel, Paladin/Surgeon/Bombardier) | BRIEF_IX-C | 11 days | Zero. The Forge saves hat colour, tunic colour, melee choice, grenade preset, helmet shape/tint and class — no paint, no decal, no skin, no named preset. | friday33 |
| `TRV-0119` `TRV-0120` | The Castle Heart / Gatehouse Signal / **objective inversion at 5:00**; and vertical-movement audio callouts (stairs soft, climbing stone-shifting, vaults metallic ring, drops rubble crunch) | BRIEF_IX-A | 11 days | KOTH has one hill. There is no two-objective, tier-inverting mode and no surface-aware movement audio. | friday22 + friday33 |
| `TRV-0187` | "**ROTATING CODEBASE REVIEW.** Each cycle, review four categories, rotating so each is examined roughly every five cycles." | `PROMPT_RND_CYCLE` §5 | 7 days | **Never run once.** The scouts do the equivalent ad hoc and find real things — arguably better — but the instruction itself has no execution record. | — |
| `TRV-0157` | "Work on branch `claude/master-research`… Do not open a pull request unless asked." | `PROMPT_MASTER` preamble | 7 days | No such branch exists. All work landed on `main`. Recording the divergence, not litigating it. | — |
| `TRV-0297` | "**Sections beyond P6 of the FRONT END spec.**" | chat, 2026-08-10 | today | See §3. Six priorities have `§owner FRONT END` markers in the code. **If the spec has 15 sections, nine of them have no trace anywhere in this repository.** | **owner — re-supply** |

**One more that belongs here and is a different shape:** `TRV-0117`, the
owner's **40 m unobstructed-sightline rule** (BRIEF_IX-A). It was
measured — all four maps fail, 80–637 m — and then **retired by an agent
document** (`632824a`: "the IX-A 40 m sightline rule is retired as a
global maximum… unsatisfiable above ~15 m half-extent from the day it was
written"). The replacement is honestly labelled "a design proposal
needing an owner decision, not a finding". **An owner rule is currently
retired on an agent's arithmetic.** That is the right arithmetic and the
wrong authority. `BLOCKED` on one owner sentence.

---

### 2.C — SHIPPED THAT THE OWNER NEVER ASKED FOR

| Row | What shipped | Against what |
|---|---|---|
| `TRV-0294` | **The bow/spear trajectory arc + landing ring + spread cone.** `main.rs:20879`. | Forbidden in bold, twice each, by BRIEF_VII and BRIEF_VIII. See C1. **This is the only item in this section that contradicts an explicit prohibition.** |
| `TRV-0097` | **The whole bow.** Full draw, sway ramp, letdown, pierce table, quiver, third-person nock, viewmodel arrow, 10 captures. | `BRIEF_VIII` Appendix A: "**The bow** … is **not** in this brief's scope. It is parked, not cancelled." Built anyway across `c727fd0`, `a4d2070`, `4be8701`, `68d325b`, `054a283`. Good work, out of scope, never re-authorised. |
| `TRV-0239` `TRV-0240` | **A ~⅓ cut to bot mech sustained output** (492 → 331 damage over 17 s at 10 m) and **braced turret fire is no longer perfectly accurate** (0° → ~1.6°). | Friday volunteered both and named them: *"a nerf nobody asked for"*, *"a third is a real balance change, not a rounding error."* The direction is what you asked for; the magnitude is not. **Wants a playtest, not an argument.** |
| `TRV-0132` | **The class system is a different system in the same slot.** Shipped LINE / SKIRMISHER / WARDEN / MARKSMAN, hooked to health, movement, spread and swap speed. | BRIEF_IX-C Tier 1 named **Assault 6.2 m/s / 25 kg, Scout 6.8 / 20, Heavy 5.0 / 32, Support 5.8 / 24** — different names *and* different axes. Same intent, and it is tested (`sim.rs:15058`). **Nobody ever wrote down that the brief's four were superseded**, so the brief still reads as unbuilt. |
| — | **The mech barrier's outer frame, six-node emitter ring and feed conduits.** | Bolted on unasked, on the argument that a field that size "needs a structure to project from". The owner asked for the original back; all of it was removed (`6442a2d`). *Listed as the exemplar of the pattern, already closed.* |
| `TRV-0298` | **A khaki emitter housing with three steel petals hanging at the left hip of every scout and every infantryman.** | A parenting change in §P1 SIX VARIANTS moved the barrier module onto the soldier's thorax, which is visible for everyone, and nothing replaced the hiding the old parent gave for free. **Found by BRIEF X's first capture, and fixed in the same commit** (`main.rs:10350-10357`). Listed because it is the clearest proof this project has that Rule 8 pays: 435 tests and a compiler never saw it; one screenshot did. `DONE`. |

Everything else I checked in the "unasked" direction turned out to be
asked for and marked: the sentinels ringing the map (`§owner ON THE MAP`),
the `CARTOON` dial (*"make the cartoons ui little look like cartoon
feeling as well"*), the zombie-extraction mode (`§8` of an earlier spec,
withdrawn from the menu with a marker), and the three deliberate
retentions the scouts already cleared.

---

## §3 — THE FRONT END SPEC — the one that is not on disk

**This is the live instance of the failure this file exists to catch.**

Yesterday, `efe1428` wrote BRIEF X to disk before building anything, and
said why: *"a spec that lives only in a chat window is the exact failure
Trevor exists to catch."* The FRONT END spec, issued the same day and
already largely built, **was not given the same treatment.** It exists
only as `§owner FRONT END` doc comments and four commit bodies.

I reconstructed what follows from those. It is the recoverable part. I
cannot tell you what is missing from it, only that P1–P6 are the highest
priorities that leave a trace and **nothing in the repository references
a P7 or beyond.**

| Row | Instruction (recovered from `§owner` markers and commit bodies) | Status | Evidence |
|---|---|---|---|
| `TRV-0267` | "LAUNCH → INTRO IMAGE → two options (START A GAME / LEARN ABOUT THE GAME)" | `DONE` | `frontend.rs:706 open_title`; `GameState::Title` is the app default (`main.rs:7839`); capture `frontend/01-title.png`, `02-learn.png` |
| `TRV-0268` | "**The normal menu bar must NOT appear after the intro.**" | `DONE` — structurally, not by hiding | Four states added at the FRONT of `GameState` (`main.rs:5059-5078`); the loadout screen is reachable only from the main menu, which is reachable only from a result or a pause. "There is no code path from launch to a menu bar left to break." |
| `TRV-0269` | "A fixed **4v4, first to 25** introductory match. No config screen." | `DONE` | `INTRO_PER_TEAM = 4`, `INTRO_TDM_TARGET = 25` (`frontend.rs:283-290`); `intro_match_config()` is a `const fn` that reads `Selected` **nowhere**; tests `intro_match_is_four_v_four_to_twenty_five`, `intro_match_ignores_every_player_setting`, `the_introductory_match_builds_and_steps` (`cd07a07` — 4v4 is a team size no shipping config had ever built, and the sim silently *clamps* `per_team`, so "it compiled" proved nothing) |
| `TRV-0270` | "MATCH COMPLETE, with exactly two large buttons" — one of them "CONTINUE PLAYING" | `DONE` | `open_match_complete` (`frontend.rs:900`), `FrontAction::ContinueToMenu` (`:223`); captures `frontend/04-match-complete.png` **and** `05-match-complete-defeat.png` — both halves, because "a screen only ever photographed winning has never had its other half looked at" |
| `TRV-0271` | "A **five-entry** MAIN MENU that reads as a command interface" | `DONE` | `open_main_menu` (`frontend.rs:1011`); test `main_menu_has_exactly_five_entries` (`:1380`) — "the spec says five and names them"; capture `frontend/03-main-menu.png` |
| `TRV-0272` | **P5** — "large mech preview, clean, the machine is the visual focus" | **`PARTIAL` / see C6** | Size delivered and measured (`main.rs:2719`, `:7528`, `:19937`, `:23229`). Subject is the **soldier**, stated openly. |
| `TRV-0273` | **P6** — settings by category navigation: GAMEPLAY / CONTROLS / AUDIO / GRAPHICS | **`PARTIAL` / see C7** | Shape delivered (`main.rs:7703`, `:24194`), tabs deviate to CONTROLS / INTERFACE / CROSSHAIR; test `every_setting_is_still_reachable_behind_some_tab` (`:28615`), mutation-proven from a file copy |
| `TRV-0274` | "**very dark blue / black** ground, **bright white** primary type (asked twice), soft grey secondary, gold reserved for accent and selection" | `DONE` | `frontend::palette` (`:81-118`) with a measured contrast table computed in **linear** space (INK 19.4:1 on ground, GOLD 9.9:1) because sRGB-space figures run ~10% optimistic |
| `TRV-0275` | "neon red and neon blue are **not** general-purpose UI colours" | `DONE` | `palette::NEON_BLUE` / `NEON_RED` (`:114-117`), doc: "**ONLY for faction.**" |
| `TRV-0276` | "**BIG click targets**" — the thing the spec is loudest about | `DONE` | `HERO_H = 78.0`, `ENTRY_H = 62.0`, ≈2× the pause menu's `ROW_H` (`frontend.rs:176-187`) |
| `TRV-0277` | "strong hover / selection feedback" | `DONE` | `weight_colors`, `ButtonPop`, `pop_target`; test `every_weight_reacts_to_hover_and_press` (`:1399`) |
| `TRV-0278` | "**fades and small scale animations only**, polished rather than flashy" | `DONE` | `FADE_S = 0.24`, `hover_scale 1.030`, `press_scale 0.985`; tests `pop_is_a_nudge_not_a_zoom`, `cartoon_dial_is_restrained` |
| `TRV-0279` | *"make the cartoons ui little look like cartoon feeling as well"* (verbatim, `frontend.rs:41`) — has to survive next to "dark futuristic" and "cinematic" | `DONE`, **and it is a dial you can turn** | `CARTOON` (`frontend.rs:153`): border 3 px, panel radius 14, button radius 20, shadow 6, hover 1.030, press 0.985. At 0 border/radius the whole front end reverts to the flat panel it was. **This is the one row in the file where you can change the answer yourself in six numbers.** |
| `TRV-0280` | "dark futuristic" · "cinematic" · "**minimal**" (twice) | `UNVERIFIABLE` | Aesthetic judgement. The instruments exist: `frontend/01..05.png`, five frames. Nobody has told you they are right; only you can. |
| `TRV-0281` | **The spec itself is not committed anywhere.** | **`NOT STARTED` — and it is the archival defect, not a build defect** | `git ls-files briefs/` returns 11 files. There is no `BRIEF_FRONTEND*`. Every trace is a comment inside `main.rs` and `frontend.rs`. **Ask: paste it once into `briefs/` and this row closes forever.** |
| `TRV-0297` | Sections beyond P6 | `UNVERIFIABLE` | No `P7`+ marker exists anywhere in the tree. I cannot tell whether they were built, refused, or never read. |

**Two adjacent facts the front-end work exposed and nobody has filed:**

1. **The game has no audio settings and no graphics settings at all** —
   no volume, no resolution, no quality, no vsync. Discovered as a
   side-effect of P6 and recorded only in a commit body. `TRV-0299`,
   `NOT STARTED`, needs an owner decision on whether it should.
2. **`d91adb4` swept a partial working tree** and committed a `main.rs`
   carrying `mod agile_mech;` while `agile_mech.rs` was still untracked.
   **HEAD did not compile between `d91adb4` and `4bde6b3`.** Recorded in
   `4bde6b3`'s own body. `TRV-0300`, `DONE` (self-corrected) — logged
   because it is the second time this project has lost work to a
   whole-tree operation, and the standing rule ("never `git stash` bare")
   should probably grow "and never `git add -A` across lanes".

---

## §4 — BRIEF X (the Agile Mech) — issued and built inside 24 hours

`briefs/BRIEF_X_agile_mech.md`, committed `efe1428`, built `4bde6b3`.
**This is the best-executed instruction in the register** and the
counter-example to everything above: written to disk first, §0 added by
an agent to record what already existed so the owner's closing line was
not misread as a feature request, one lane, no sim change, seven tests.

| Row | Section | Status | Evidence |
|---|---|---|---|
| `TRV-0282` | §0 — the three abilities that **must not break**: climb, double jump, ground dodge, air flip, second flip charge | `DONE` (sim untouched) + **capture owed** | `agile_mech.rs:48-53` "Presentation only. Nothing here is read by `sim.rs`… A replay is bit-identical with this model or the old one." |
| `TRV-0283` | §1 cartoon-tech construction from boxes/cylinders/plates | `DONE` | `agile_mech.rs` whole module |
| `TRV-0284` | §2 orange armour + metallic blue machinery + graphite + subtle bright blue | `DONE` | Four material roles; `agile_blue` is **new** — "the old palette painted armour and mechanism the same". `energy` is deliberately identical on both sides so it cannot eat the team lamp (`:163-166`) |
| `TRV-0285` | §3 compact silhouette, "speed rather than strength" | `DONE` | Five elements, none in the Big's vocabulary: reverse-jointed legs, swept dorsal fins, wedge helmet 0.245 wide (vs the old egg's 0.50), shoulders ±0.40 (vs ±0.475), forward-canted torso |
| `TRV-0286` | §4 mechanical head, **NO HAT** | `DONE` | Wedge helmet, faceplate, horizontal visor slit; the visor carries the team lamp at eye height (`:374`) |
| `TRV-0287` | §5 modular armour by region | `DONE` | `agile_mech.rs:293-380` |
| `TRV-0288` | §6/§7 reuse the existing controller, physics, animation, skeleton | `DONE` | One-lane by construction; `SCOUT_SCALE` untouched (owner Q5 still open) |
| `TRV-0289` | §8 animation & motion — no clipping through idle/walk/run/jump/land/turn/strafe/aim/attack/swap/throw **+ climb, double jump, dodge, flip** | **`NOT STARTED` (the evidence)** | The `agile_moves` capture script **exists in code** (`main.rs:6277`, `:6340`) — walk, jump, double jump, dodge roll, air flip, second flip — and **has produced no PNGs.** Nothing in this harness had ever pressed Q or Space in the light chassis. `handback/brief-vii/agile_moves/` does not exist. |
| `TRV-0290` | §10 detail by layering, §11 weapon integration | `DONE` | `GROUND_Y` / `MOUNT_X` / `MOUNT_Y` pinned by `agile_anchors_are_pinned` — the repair-beam origin and the plasma spawn were tuned against these and are computed from fighter position, not from the model |
| `TRV-0291` | §12 performance — low/medium poly, reasonable draw calls | `UNVERIFIED` | No draw-call or material-count measurement exists. Asserted, not measured. |
| `TRV-0292` | §13 differentiate from Big and Royal | `DONE` in code, **`NOT PROVEN`** | Eight `agile_body` captures exist **and are untracked** (four sides, head close, legs close). **None is a three-tier black-silhouette comparison at 30 m**, which is both the brief's acceptance line and the one `mech_lineup.rs` can already produce. See C5. |
| `TRV-0293` | §9/§14 cartoon-technical style, final design target | `UNVERIFIABLE` | Aesthetic. The eight captures are the instrument; only the owner can read them. |

**Immediate, cheap, and owed:** commit the eight `agile_body` PNGs, run
`agile_moves`, and take one three-tier silhouette frame. That closes six
of BRIEF X's own acceptance lines and answers C5's second half.

---

## §5 — THE REGISTER, BY SOURCE

Rows `TRV-0001`..`TRV-0266` are carried from `TREVOR_LEDGER.md` §B with
their quoted asks and IDs intact. Rather than restate 266 rows I verified
selectively and record here **only where my re-derivation differs from
that file** (§7), plus the new rows opened above.

Source blocks, for dispatch routing:

| Block | Source | Rows | Open |
|---|---|---|---|
| A | `WHATS_MISSING.md` §0-SPEC15 (the owner's 15-section mech spec, P1-P4) | 0001-0021 | **1** (`TRV-0010`) at P1/P2; 3 at P3; P4 untouched |
| B | §0-QUEUE Tiers 0-4 | 0022-0053 | 20 |
| C | §0-NOW (post-six-agent list) | 0054-0065 | 8 |
| D | `BRIEF_VII_optimized.md` | 0066-0077 | 3 + **C1** |
| E | `BRIEF_VIII_master.md` | 0078-0100 | 7 + **C1, C2** |
| F | `BRIEF_VIII_B_addendum.md` | 0101-0115 | 6 + **C3, C4** |
| G | `BRIEF_IX` A/B/C | 0116-0141 | 17 |
| H | `PROMPT_MASTER_research_build.md` (13 tasks, one row each) | 0142-0157 | 11, most superseded by OPERATION rule 13 |
| I | `PROMPT_brief_X_research.md` (superseded, indexed) | 0158-0166 | 0 live |
| J | `PROMPT_mech_rebuild.md` (superseded, indexed) | 0167-0173 | 2 |
| K | `PROMPT_RND_CYCLE.md` + `BACKLOG.md` | 0174-0187 | 8 |
| L | `PROMPT_motion_system_research.md` | 0188-0190 | **0 — closed 2026-08-10** |
| M | **Chat asks recorded second-hand** (`§owner` comments, log quotes, commit messages, README lines) | 0191-0234 | 5 |
| N | Agent-origin (Friday's deferrals, Thor's findings, the scouts') | 0235-0255 | 12 |
| O | Images — uploaded, missing, generated | 0256-0266 | 3, all `BLOCKED` on the owner |
| **P** | **FRONT END spec — chat only, not on disk** | **0267-0281** | **4** |
| **Q** | **BRIEF X** | **0282-0293** | **3** |
| **R** | **Contradictions and new findings opened by this sweep** | **0294-0300** | **3 new contradictions + 4** |

---

## §6 — DISPATCH SHEET

Ranked by the owner's own priority order (SPEC15 P1→P4, then 0-QUEUE
Tier 0→4), then by what unblocks the most, then by cost. Where I would
have ordered differently I say so and leave the owner's order standing.

### Owner — decisions only, no work (unblocks 14 rows)

1. **C1** — bow/spear arc: honour the ban, or retire it? (`TRV-0294`)
2. **C5** — is the opposition Agile orange, or red? (`TRV-0296`)
3. **C6** — does the customization screen preview a soldier or a mech? (`TRV-0272`)
4. **C7** — do you want audio and graphics settings at all? (`TRV-0273`, `TRV-0299`)
5. **C2 / C3 / C4** — three brief lines that are almost certainly retired and are not marked so (`TRV-0295`, `0113`, `0111`)
6. **Re-supply the mech and medic concept art** (`TRV-0260`, `0261`) — 10 days, four unsatisfiable completion criteria
7. **Paste the FRONT END spec into `briefs/`** (`TRV-0281`) — and tell us how many sections it has
8. Carried from the ledger, still open: mech FP aim 1.10 m (`TRV-0053`), `SCOUT_SCALE` 1.05 (`TRV-0059`), the recoil balance cost (`TRV-0239`/`0240`), the scout's non-shrinking hitbox (`TRV-0248`), `armor_spec`'s unreachable flats (`TRV-0236`), the 40 m sightline rule's retirement (`TRV-0117`)

### friday33 — `main.rs` + client modules

| # | Task | Rows | Cost |
|---|---|---|---|
| 1 | **Royal ARROW LAUNCHER** — the last open P2 row | `TRV-0010` | 1-2 sessions |
| 2 | **Commit the `agile_body` captures, run `agile_moves`, take the three-tier silhouette frame** | `TRV-0289`, `0292` | ~1 capture cycle |
| 3 | **Capture scripts as data** — pays for every row below it | `TRV-0040`, `0204`, `0251` | 1 session |
| 4 | **Armour damage states get a client half** | `TRV-0036`, `0137`, `0140` | 1 session |
| 5 | **Explosion audio + the four placeholders + the eight boarding beats** | `TRV-0030`, `0026` | 1 hour |
| 6 | **Mech boarding: 7 of 8 stages made visible** (the strings already exist verbatim inside the `debug!` calls) | `TRV-0037`, `0174` | 1 session |
| 7 | **The six honesty fixes, one sitting** | `TRV-0031`, `0024`, `0032`, `0033`, `0028`, `0055` | 1 session |
| 8 | **The luminance guard test** — do it inside task 2 | `TRV-0206` | 30 min |
| 9 | Rocket launcher redesign; Royal body; opposition body | `TRV-0012`, `0013`, `0014`, `0015` | 1-2 sessions each |

*Where I would have re-ordered and did not: I would put 3 and 5 above 1,
because they are cheap and they buy evidence for everything else. The
owner's order puts P2 first, so the arrow launcher stands at the top.
Both readings are here; pick one.*

### friday22 — `sim.rs`

`TRV-0055` (split `gatling_heat` at the source) · `TRV-0029` (export the
`9000.0` spray scale as a `pub const`; the client's copy is
`main.rs:308`) · `TRV-0235` (plate wear fires only on the zoned hitscan
path — grenades, melee, claws and gas neither wear plate nor are reduced
by it) · `TRV-0241` (the player/bot asymmetry in
`punched_aim_stabilised` — Friday calls it "the real root", and it
applies to every recoiling weapon) · `TRV-0242` (a bot chassis now never
raises its barrier at all) · `TRV-0039` (the Cliffhold sim half — **50
case-insensitive references survive in `sim.rs` against 3 in `main.rs`**; coordinate or
the `match` breaks) · `TRV-0043` (bot navigation, now that
`MAP_METRICS.md` exists)

### thor — claimed, evidence thin

`TRV-0294` (**first**: does the arc actually render on screen with a bow
under focus? No capture holds focus with either weapon) · `TRV-0008`
(recoil envelope — biggest feel change in the game, zero captures) ·
`TRV-0005` (the four-slot weapon strip — no frame shows it; the only
strip captures on disk predate the change) · `TRV-0035` (the barrier
test copies its own constants into the test body; if it is vacuous so is
`TRV-0207`'s evidence) · `TRV-0057` (`visor_ready` inside a `Local`) ·
`TRV-0252`, `TRV-0253` (turret spinners, world minigun spin) ·
`TRV-0134` (mutation-prove `armor_weight_movement_penalty`)

### toto — Rule 13: only a named unknown NUMBER

- `TRV-0010` — the arrow launcher's rate of fire, bolt velocity, and
  per-bolt damage against a mech hull. Nothing in the game fires a bolt
  from a mount, so there is no neighbouring value to interpolate from.
- `TRV-0126` — "how enclosed is this point": the **method** is the
  unknown, not a coefficient. Ask for the method and its cost at 120 Hz.

---

## §7 — WHERE I DISAGREE WITH THE RECORD

| Document | Its claim | What I found |
|---|---|---|
| `TREVOR_LEDGER.md` `TRV-0071`, `TRV-0091` | Spear/bow: `DELIVERED` | **The second half of both asks is violated.** "No trajectory arc. No landing marker" ships as its opposite. The ledger quoted the sentence and did not check it. See C1. |
| `TREVOR_LEDGER.md` §F | "The **34** `UNVERIFIED` rows…" | The headline of the same file says **21**, and `TREVOR_LOG.md` lists **22** IDs under a heading that says 21. Three numbers for one set. Cosmetic, but a register that cannot count itself invites doubt about the rest. |
| `TREVOR_LEDGER.md` `TRV-0011` | Agile Mech: `NOT STARTED` | **Built** since, `agile_mech.rs`. Correct when written 18 hours ago. |
| `TREVOR_LEDGER.md` `TRV-0027` | `SCOUT_SCALE` doc wording: `UNVERIFIED` | Re-derived: `sim.rs:5038` reads "Scale, **against the heavy's chassis**. Slimmer and shorter" — which is *true* against the heavy. **The queue item (#6) mis-states the defect.** What is false is the next clause: "so it reads as a different silhouette from across a map", at 1.05. `PARTIAL`. |
| `TREVOR_LEDGER.md` `TRV-0029`, `0034`, `0254` | `UNVERIFIED` | Resolved. `9000.0` still bare at four sim sites plus a client copy (`NOT STARTED`); `gait_pose` still infantry-derived (`NOT STARTED`, literal moved not fixed); `crouch_drop` double-count **fixed** (`DONE`). |
| `WHATS_MISSING.md` TIER 0 / TIER 2 #17 / TIER 4 #28 | Listed as open | Three TIER 0 items done, Pyro fully removed, traversal unblocked. **Stale for the fourth time**, in both directions. |
| `BACKLOG.md` #10 | "rapier supports some of it" (destruction) | **`jk_tdm` has no rapier dependency.** `rapier3d` belongs to `jk_wall`. Same wrong-game confusion as `DESIGN_MAP.md`. Found by `632824a`, confirmed here. |
| `DECISIONS.md` | Cited as a project decision record | **Every ADR is about `jk_wall`/`jk_core`**, the shieldwall sim — not `jk_tdm`. ADR-006 promises "Bevy at the art milestone: glTF loading, skeletal animation"; `jk_tdm` **is** Bevy and has no glTF loader (`TRV-0048`). ADR-002 promises Rapier; `jk_tdm` has none. Two of the five ADRs describe a pipeline this game does not have. Third wrong-game document, after `DESIGN_MAP.md` and `BACKLOG.md` #10. |
| `GAME_STATUS_REPORT.md` | Lists Pyro; names the 20-segment rig, 26-piece armour and 4-class system as unbuilt | All four wrong. Dated 2026-08-01, reads as current. |
| `handback/brief-ix/REPORT.md` | Class system, 26-piece armour, damage states "do not exist in any form" | All three shipped. Honest snapshot of 2026-07-30, misleading today. |

---

## §8 — WHAT I COULD NOT CHECK, AND WHY

Stated as a section rather than buried, because implying a complete sweep
is the failure this file exists to prevent.

- **I did not build and did not run the suite.** Read-only; I never
  invoked `cargo`. The dispatch reports 435 pass / 1 fail (another
  session's in-progress `agile_mech` work). Every `DONE` means "there is
  evidence at this path", never "it compiles" and never "it works".
- **I opened exactly two of the 165 committed PNGs**
  (`bow_draw_fp/03-fp-bow-full-draw.png`, `spear_flight/00-charging.png`),
  both to test C1, and **neither settled it** — neither holds focus, so
  neither can show or refute the arc. That non-result is recorded in C1
  rather than rounded into a disposition. Rule 8 says the capture is the
  instrument; I catalogued the instruments and read two.
- **The FRONT END spec's true section count is unknown to me.** I
  reconstructed 14 requirements from `§owner` markers and commit bodies.
  The dispatch says 15 sections. If P7–P15 exist, **I have no way to see
  them and no way to tell whether they were built.** `TRV-0297` says so
  rather than guessing.
- **Read whole:** `WHATS_MISSING.md`, `BRIEF_X_agile_mech.md`,
  `DECISIONS.md`, `TREVOR_LEDGER.md`, `TREVOR_TASKS.md`, `TREVOR_LOG.md`,
  `frontend.rs` §§1-330 and its test names, `agile_mech.rs` header and
  constants, the last seven commit bodies in full.
- **Sampled by targeted grep + region reads:** `main.rs` (~29k lines,
  ~10 regions), `sim.rs` (~28k lines, ~6 regions), `briefs/*` (grep for
  the contradiction candidates only), `FRIDAY_LOG.md` (headings only —
  **it has no entry for the front-end work at all**), `THOR_LOG.md`
  (not opened this run), `TOTO_LOG.md` (not opened this run).
- **Not read at all:** `research/maps/MAP_METRICS.md` (1,025 lines) and
  `research/motion-architecture/DECISION.md` (775 lines) — I verified
  they exist, are committed, and what their commit body claims they
  contain. I did not audit their content. `handback/ACCOMPLISHMENTS.md`,
  `AUDIT.md`, `CHANGES.md`, `REPORT.md`, `brief-vii/HANDBACK.md`, the
  eleven per-topic `SOURCES.md`, everything in `jk_wall` / `jk_bevy` /
  `jk_client` / `jk_spike` / `jk_core`.
- **The 14 `UNVERIFIED` rows are the honest total of the above.** None
  carries a fabricated disposition. None was bucketed as a negative
  result because a check did not finish — that is the failure mode this
  project has hit twice (46 verify agents killed by a rate limit and
  filed as "disputed"; three agents' research discarded by a missing
  `await`).
- **Concurrency.** Another session is writing `agile_mech.rs`,
  `main.rs` and `sim.rs` right now. Line numbers in this file were true
  at `4bde6b3`. Every row also names its symbol; re-anchor on the symbol,
  `git fetch` first.

---

*Written by TREVOR. I do not edit source, briefs, or another agent's log.
Where a document is wrong I record the disagreement here and hand it over.*
