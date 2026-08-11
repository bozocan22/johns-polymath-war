# ISSUED vs DELIVERED — the owner-instruction register

**Run 2. Rebuilt 2026-08-11. Repo HEAD at sweep start and end: `5473d06`.**
`git fetch` run first; `origin/main` == local `main` == `5473d06`.

**Working tree at sweep time — read this before trusting any line below.**
Two files are modified and uncommitted:

| File | Diff | Whose | How I treated it |
|---|---|---|---|
| `src/sim.rs` | +279 / −10 | a live builder, **§9/§10/§11 of the bow-and-spear spec** | Read, **not counted as delivered.** Its rows are `UNVERIFIED` and say why. |
| `src/frontend.rs` | +20 | a live builder | Not read, not indexed. |

A commit that has not happened is not a delivery, and the last thing this
register should do is bank another lane's half-written work as done. Rows
`TRV-0303`, `0304`, `0305` are the ones affected and each says so in its
own line.

**What this file is.** Thor verifies claims about CODE. This verifies
claims about the OWNER'S INSTRUCTIONS: every ask, from every source, with
a status backed by `file:line` or a commit — never by a commit message's
claim about itself.

**Status vocabulary** (the owner's):

| Status | Means |
|---|---|
| `DONE` | I found the code and can cite it |
| `PARTIAL` | Some shipped. The row says precisely which half did not |
| `NOT STARTED` | Grepped for, genuinely absent |
| `BLOCKED` | A specific named blocker, not "needs art" |
| `SUPERSEDED` | A later instruction replaced it; both are cited |
| `CONTRADICTED` | The owner asked for two incompatible things. Both named. **The owner decides, not me** |
| `UNVERIFIABLE` | Cannot be settled from code alone |
| `UNVERIFIED` | I did not check, or the check did not complete |

`DONE` means "there is evidence at this path". It does not mean "it
works". I did not build and did not run the suite.

---

## §0 — HEADLINE

| | run 1 (2026-08-10) | **run 2 (today)** |
|---|---|---|
| Rows | 300 | **344** — `TRV-0001`..`TRV-0344`, no gaps |
| `DONE` | 137 | **142** |
| `PARTIAL` | 42 | **48** |
| `NOT STARTED` | 65 | **87** |
| **Open (`PARTIAL`+`NOT STARTED`)** | 107 | **135** |
| `BLOCKED` | 16 | **22** |
| `SUPERSEDED` | 12 | 12 |
| **`CONTRADICTED`** | 7 | **8** |
| `UNVERIFIABLE` | 3 | **4** |
| `UNVERIFIED` | 18 | **21** |

Counts sum to 344 exactly. **Open work went UP by 28 while five rows
closed** — because two new owner specs arrived (44 new rows) and only six
old rows moved. That is not a regression; it is the register catching up
with two briefs issued in one day.

Origin: 310 owner · 28 agent · 6 agent-found-then-adopted-by-owner.

**The five rows that MOVED, each re-derived from code today:**

| Row | Was | Now | Why |
|---|---|---|---|
| `TRV-0289` | `NOT STARTED` | **`DONE`** | BRIEF X §8 motion captures: **12 `agile_moves` PNGs now committed** (`8de5e93`, re-run `9b108b2`) — roll early/inverted/recover, jump, double-jump kick + apex, air flip first/inverted/second, landed. This was the register's single loudest "the script exists and has produced no PNGs". It has produced twelve. |
| `TRV-0206` | `NOT STARTED` | **`DONE`** | SPEC15 trap 4, the unguarded luminance rule: `the_enemy_agile_never_out_luminates_the_ally` (`agile_mech.rs:744`) pins hull separation ≥ 8× in **linear** light and the strict form — every enemy surface below every ally surface. Backed by a second test that pins the gamma decode itself (`relative_luminance_decodes_gamma`, `:770`), so the guard cannot pass on a broken formula. |
| `TRV-0292` | `DONE` in code / `NOT PROVEN` | **`PARTIAL`** | BRIEF X §13's acceptance line now has *an* instrument — `agile_body/09-squint-derived.png` — but the builder states in `FRIDAY_LOG.md:1549` that it is **not a matte**: it is `mech_gallery/01` downsampled 3.33× and hue-stripped, because the harness has no stencil or depth output. Honest, filename says `derived`, and it is not the three-tier black silhouette the brief asks for. |
| `TRV-0291` | `UNVERIFIED` | **`PARTIAL`** | BRIEF X §12 performance: part/mesh/material counts now published (~150 parts, 3 meshes, 7 materials, 4 shared with the heavy — `FRIDAY_LOG.md:1499`). Still **no draw-call measurement.** Counted, not profiled. |
| `TRV-0296` | `CONTRADICTED` | **`CONTRADICTED` (my description was WRONG — see §7)** | I wrote that the enemy Agile is orange. **It is not.** `ARMOR_FOE = [0.075, 0.125, 0.265]` — "faction dark blue" (`agile_mech.rs:179`). The contradiction is real and still open, but it points the *other* way. |

---

## §1 — WHAT IS LEFT

*The owner has asked this many times and every answer has been partly
wrong. This is the whole answer, ranked, with a lane and the evidence
behind each status. Lanes: **[S]** `sim.rs`/friday22 · **[C]**
`main.rs`+client/friday33 · **[D]** docs · **[OWNER]** a decision only
the owner can make.*

### 1.A — [OWNER] Decisions. Zero build cost. They unblock 21 rows.

| # | The question | Rows | Why it is stuck |
|---|---|---|---|
| 1 | **Bow/spear trajectory arc: honour the ban, or retire it?** | `TRV-0294` | Two briefs forbid it in bold, twice each. The game ships it, gated on exactly the two weapons named. **§2 C1.** Now urgent: RMB was just formalised as PRE-AIM, and the arc is drawn on RMB. |
| 2 | **"The crosshair is sacred." Does the landing ring count as obstruction?** | `TRV-0308` | New spec, stated three times. The FP viewmodel already obeys it and is tested. The arc's **landing ring sits on the impact point**, which on a flat shot is where the crosshair is. Never captured, so never seen. |
| 3 | **Enemy Agile livery: orange or blue?** | `TRV-0296`, `TRV-0320` | BRIEF X §2 "orange becomes the Agile's identity", no faction split stated; SPEC15 says opposition mechs keep red/blue. Build ships **dark-blue primary + rust plates**. BRIEF XI §16 says "do not change the colour identity", which explicitly **does not resolve it** (§0.4). |
| 4 | **Can the Agile Mech climb? Do you want mech climbing built?** | `TRV-0318`, `TRV-0326` | BRIEF X §0 asserts it can; `climb_target` is gated `!in_mech()` **at the climber** and needs a dropped plate on the target. BRIEF XI §0.1 gates §5 and its own §19 checkbox on a written verdict. **Nobody may build or skip it until you say.** |
| 5 | **The ~⅓ bot mech output cut, and braced fire no longer perfectly accurate** | `TRV-0239`, `TRV-0240` | 492 → 331 damage over 17 s at 10 m; braced 0° → ~1.6°. The builder volunteered both and called one *"a nerf nobody asked for"*. **Still unruled after 5 days.** Wants a playtest, not an argument. |
| 6 | **"No head, no cockpit glass, no visible pilot opening"** | `TRV-0295` | Said three times across two briefs and a prompt. The game ships a visor, a cockpit and a ×2 visor weak point that is load-bearing gameplay. Almost certainly superseded in your head; **nobody wrote it down**, and §D.7 still makes the concept-art comparison the mech section's completion criterion. |
| 7 | **The missile pod: "absent by default" or ten tubes on `Y`?** | `TRV-0113` | Both instructions are yours. The ten-tube line carries a `§owner` marker, so it is probably the later one. One sentence closes it. |
| 8 | **Retire addendum §D.2 (khaki/olive, "no glowing visor")?** | `TRV-0111` | Never retired, only overtaken. The next agent reading the briefs in order will rebuild a khaki mech. |
| 9 | **Customization: preview the soldier, or make the mech editable?** | `TRV-0272` | The size half shipped and is measured. The subject is the soldier, stated openly in the commit. "The soldier is fine" is one line; the alternative is a multi-session feature nobody has scoped. |
| 10 | **Do you want audio and graphics settings at all?** | `TRV-0273`, `TRV-0299` | The game has **no volume, no resolution, no quality, no vsync.** Discovered as a side-effect of the settings work and recorded only in a commit body. |
| 11 | **Re-supply four bow/spear reference images + the mech + medic concept art** | `TRV-0309`..`0312`, `TRV-0260`, `TRV-0261` | Six owner-supplied images, none on disk. `handback/reference/` contains one file and it is `NOTES.md`. **Ten days on the mech art.** |
| 12 | **Paste the bow-and-spear spec and the FRONT END spec into `briefs/`** | `TRV-0313`, `TRV-0281` | Two specs exist only in chat. `git ls-files briefs/` returns 12 files and neither is there. See §3. |
| 13 | **The 40 m sightline rule was retired by an AGENT's arithmetic** | `TRV-0117` | The arithmetic is right (all four maps fail, 80–637 m; unsatisfiable above ~15 m half-extent). The authority is wrong. One sentence. |
| 14 | Carried, still open: mech FP aim height 1.10 m (`TRV-0053`), `SCOUT_SCALE` 1.05 (`TRV-0059`, frozen in two briefs pending you), the scout's non-shrinking hitbox (`TRV-0248`), `armor_spec`'s unreachable flats (`TRV-0236`) | — | Each is a single number. |

*Ruled and closed — do not re-open: the Royal's gold vs neon blue
(`TRV-0013`), 2026-08-10: "keep the royal mech, and keep the opposition
royal mech colour red yellow and neon blue details."*

### 1.B — [C] friday33, `main.rs` + client modules. Ranked.

| # | Task | Rows | Cost | Acceptance check |
|---|---|---|---|---|
| 1 | **The bow/spear CLIENT half of the input split.** `aim_phase` is `pub` and has **zero readers in `main.rs`** (grep: 0). The HUD still derives state from raw timers — 24 references to `bow_draw_t`/`spear_charge_t`/`spear_wind_t`. And `main.rs:16987` still tells the reader RMB is *"a draw on projectile weapons (bow/spear, Brief II grammar)"*, which the sim killed 12 hours ago. | `TRV-0302`, `TRV-0315` | 1 session | `main.rs` contains ≥1 `sim::aim_phase(` call and 0 hand-rolled charge-state derivations; the Brief II sentence is gone. |
| 2 | **A capture that HOLDS RMB with a bow and with a spear.** This is the instrument that has never existed. It settles C1 (does the banned arc render?), C2 of the new spec (does anything cross the crosshair?), and photographs pre-aim for the first time. `BOW_DRAW_FP_BEATS` (`main.rs:5699`) presses **Mouse Left only** — correct under the new spec, and it means no frame in this repo shows pre-aim. | `TRV-0317`, `TRV-0294`, `TRV-0308` | ~1 capture cycle | Two new PNGs, one per weapon, with RMB held at the snap beat. |
| 3 | **Capture scripts as DATA, not code.** `struct CapBeat` (`main.rs:5280`) — every script is still a compile-time const array in a 29k-line file. 6 min per framing tweak vs ~40 s. Thor's highest-leverage finding, and it pays for tasks 2, 6 and every BRIEF XI proof below. | `TRV-0040`, `TRV-0204`, `TRV-0251` | 1 session | A framing change to one beat requires no `cargo build`. |
| 4 | **Armour damage states get a client half.** Re-derived today: `main.rs` contains **0** references to `armor_stage_of`, `armor_wear_of` or `ArmorStage`; `sim.rs` contains **66**. Fresh, Scuffed and Cracked render identically. `ArmorStage::tilts` is a published accessor for a brief requirement with no reader. | `TRV-0036`, `TRV-0137`, `TRV-0140` | 1 session | A capture at 100% / 50% / 15% plate shows three different surfaces. `CapBeat.hull` already stages it. |
| 5 | **Explosion audio.** Re-derived: **21** `asset_server.load` calls, all `.wav`, and the list is bow · click · headshot · hit · hurt · jump · kill · pickup · reload · roll · shield · shot_ak · shot_deagle · shot_glock · shot_mg · shot_mp5 · shot_rifle · shot_shotgun · shot_sniper · spear · win. **No boom.** The sim publishes `Boom` and nothing plays. `gen_sfx.py` generates no explosion either — so this is 1 hour, including writing the generator line. | `TRV-0030` | 1 hour | An explosion is audible; `gen_sfx.py` writes `boom.wav`; the load list is 22. |
| 6 | **The Agile's double jump has no airborne pose.** `try_mech_jump`'s compress/tuck is heavy-only, so the apex frame is indistinguishable from standing. The builder flagged it rather than sneaking it in. BRIEF XI §0.4 calls it *"the highest-value single fix in this brief: the Agile's signature mechanic is currently invisible"*. Captures `07-double-jump-kick.png` / `08-double-jump-apex.png` already exist and will show the fix. | `TRV-0321`, `TRV-0324` | ~half a session | Re-run `agile_moves`; frames 07/08 differ from 01-standing. |
| 7 | **BRIEF XI §1 — generalise `solve_arm_ik` to the legs.** `solve_arm_ik` (`main.rs:2585`) has **10 call sites, all arms**; there is no `leg_ik`, no `foot_place`, no `solve_leg` anywhere in `src/`. `DECISION.md` item 3 already specifies this exactly and BRIEF XI §0.2 makes it binding — *do not write a second solver, do not add a crate.* This is the single highest-value BRIEF XI item and closes §1, most of §2, and the foot half of §19. | `TRV-0319`, `TRV-0322` | 1-2 sessions | Feet stay on a slope; consecutive-frame capture shows no sliding. |
| 8 | **The six honesty fixes, one sitting.** Field Manual still prints `TDM_TARGET,` the constant (`main.rs:24643`) not the chosen target; `BIND_REGISTRY`'s only `U` row still says "Dismount the mech" (`main.rs:5154`) while the world prompt says "U - GRAB THE HULL" (`:23033`); the `Q` row (`:5137`) names roll and flip only — the Agile's **second flip charge and mid-air jump appear nowhere a player can find them**, and BRIEF X made them its mechanical identity; `gatling_heat` prints ×100 under a `%` at four sites and raw under the same `%` at `main.rs:21992`. | `TRV-0031`, `0032`, `0033`, `0024`, `0028`, `0055` | 1 session | Each string matches the value it claims to describe. |
| 9 | **Royal ARROW LAUNCHER** — "minigun + 3 crossbows". Re-derived today: `crossbow`, `arrow_launcher`, `ArrowLauncher` return **zero hits in any weapon path**; `MechWeapon` = Gatling / Autocannon / Rockets / Plasma / Repair (`sim.rs:5173-5191`). Owner priority says P2 and P2 outranks everything above it — see the note below the table. | `TRV-0010` | 1-2 sessions + one toto number | A `MechWeapon::ArrowLauncher` variant fires bolts; a capture shows the silhouette. |
| 10 | Mech boarding: 7 of 8 stages made visible (the strings already exist verbatim inside the `debug!` calls) · rocket launcher redesign · Royal body · opposition body | `TRV-0037`, `0174`, `0012`, `0013`, `0014`, `0015` | 1-2 sessions each | — |
| 11 | The rest of BRIEF XI — §7-§14, the limb/hand/grip families and the gun art pass | `TRV-0328`..`0335` | multi-session | §19's own checklist, with consecutive-frame captures |

**Where I would have re-ordered and did not.** I would put 3 and 5 above
1, because they are cheap and buy evidence for everything else, and I
would put 9 (the arrow launcher) after 7. The owner's stated order puts
SPEC15 P2 first, so the arrow launcher keeps its P2 claim. Both readings
are here; pick one.

### 1.C — [S] friday22, `sim.rs`

- `TRV-0055` — split `gatling_heat`'s two scales **at the source**; the client half is task 8 above.
- `TRV-0029` — export the `9000.0` spray scale as a `pub const`. Re-derived: still a bare literal at `sim.rs:4515` and `:5481`, twice more in tests (`:23760`, `:23761`), and the client keeps its own copy at `main.rs:308 PUNCH_DEG_S_PER_SPEC_KICK = 9000.0`.
- `TRV-0316` — `step_plasma_precision` (`sim.rs:8859`) still charges on `cmd.ads`. **It is the one remaining RMB-charge in the file**, its own comment calls it "PLASMA BOW mode 2", and the builder flagged rather than fixed it because moving it needs the client's mount router to move too. Cross-lane; needs both Fridays or an owner "leave it".
- `TRV-0235` — plate wear fires only on the zoned hitscan path: grenades, melee, claws and gas neither wear plate nor are reduced by it.
- `TRV-0241` — the player/bot asymmetry in `punched_aim_stabilised`, which the builder calls "the real root" and which applies to every recoiling weapon.
- `TRV-0242` — a bot chassis now never raises its barrier at all.
- `TRV-0039` — the Cliffhold sim half: **50 case-insensitive references in `sim.rs` against 3 in `main.rs`.** Coordinate or the `match` breaks.
- `TRV-0043` — bot navigation, now that `MAP_METRICS.md` exists.

### 1.D — [C]+[S] The bow/spear spec's own remainder

`TRV-0303`/`0304`/`0305` (the 3-second charge window, the 7-second
maximum-charge bonus, §B's damage bonus) are **being written right now**
in the uncommitted `sim.rs`. Do not dispatch them. When they land, the
client owes: a charge readout that reads `spear_wind_frac_of` /
`spear_max_charged_of` instead of the raw clock, and the **overhead
javelin wind pose** (`TRV-0306`) which is pure presentation and has no
client code at all today.

### 1.E — [D] Docs

- `WHATS_MISSING.md` — **last touched `7719296`, two days and eleven commits ago.** Stale in both directions. It still lists as open: three TIER 0 items that are done, the Pyro relocation tables that are gone, traversal's map-metrics block that is lifted, `TRV-0011` the Agile Mech, and `TRV-0206` the luminance guard. It has now gone stale four times.
- `BACKLOG.md` #4/#5/#9/#11 known false (melee depth, retreat, class system, armour-weight wiring all shipped); #10 cites rapier, which `jk_tdm` does not depend on.
- `DECISIONS.md` — every ADR is about `jk_wall`/`jk_core`, not this game.
- `GAME_STATUS_REPORT.md` (dated 2026-08-01) and `handback/brief-ix/REPORT.md` — both name shipped systems as unbuilt.
- `README.md:223-225` still states the retired 8v8 cap.
- `briefs/README.md` does not list BRIEF XI.

---

## §2 — CONTRADICTIONS THE OWNER MUST RULE ON

*Eight. Each names both instructions and cites both. I did not pick a
side on any of them.*

#### C1 — `TRV-0294` · **The bow and spear ship a predicted trajectory arc and a landing ring, which two briefs forbid.**

> `BRIEF_VII_optimized.md:331` — "**No trajectory arc. No landing marker.** The arc is learned — that is the entire point of Reference A."
> `BRIEF_VII_optimized.md:72` — "Brief V grenade trajectory arc | **Grenade-only.** Spear and bow get NO arc and NO landing marker — the learnable arc IS the skill."
> `BRIEF_VIII_master.md:561` — "**No arc, no landing marker** — the arc is learned, and that is the skill."

against the code's own cited authority:

> `main.rs:2661` — "The predicted-arc preview for **bow/spear** aiming (**§4.2 Brief II**)…"
> `main.rs:20` — "Bow/spear aiming shows a red PREDICTED ARC with a landing marker".

**Re-derived today at `5473d06`.** `fn arc_preview` (`main.rs:20902`)
still gates on:

```rust
let show = cam_ctl.ads && p.alive() && spec.projectile.is_some() && p.roll_t <= 0.0;
```

Exactly two guns declare `projectile: Some(..)` — Bow (`sim.rs:686`) and
Spear (`sim.rs:708`). The preview exists for precisely the two weapons
the ban names and nothing else. Brief II is not in this repository;
Briefs VII and VIII both post-date it.

**What is NEW and makes this urgent.** `cam_ctl.ads` is RMB. The owner's
new spec has just declared RMB to be **PRE-AIM** and said so three times.
So the forbidden arc is now bound to the exact button the newest spec
elevates. Whatever you rule, rule it before the client half of the input
split is built on top of it.

**Evidence gap, unchanged:** no capture in this repository holds RMB with
a bow or a spear. `bow_draw_fp/03` and `spear_flight/00` are un-focused
and show no arc. The code is conclusive; **the instrument has never been
pointed at it.** `BOW_DRAW_FP_BEATS` presses Mouse Left only.

**Dispatch if the owner rules for the briefs:** friday33, ~1 hour — gate
`arc_preview` on the grenade-class throw only, and take the RMB-held beat
so the absence is photographed.

#### C2 — `TRV-0295` · **"No head, no cockpit glass, no visible pilot opening" against a shipped head, visor and cockpit.**

> `BRIEF_VIII_master.md:666` · `BRIEF_VIII_B_addendum.md:249` · `PROMPT_mech_rebuild.md:315` — the same words, three times.

Against the 15-section spec's §20 spacecraft cockpit, which shipped as
`cockpit.rs`, 16 cockpit captures, `MECH_VISOR_Y_FRAC`, and a **×2 visor
weak point that is now load-bearing gameplay**. Unchanged since run 1.

#### C3 — `TRV-0113` · **The missile pod: "absent by default" vs bound to `Y` with ten tubes.**

`BRIEF_VIII_B_addendum.md:329` and `PROMPT_mech_rebuild.md:369-370` both
say optional and absent. Live: `sim.rs:4289` `§owner: "ten tubes per
chassis (was 4), and ammo pads now resupply"`, on every chassis, bound to
`Y`. The `§owner` marker on the ten-tube line means the later one is also
yours — very likely a supersession nobody recorded.

#### C4 — `TRV-0111` · **Olive drab / khaki, "minimal emissive", "the art has no glowing visor" — against the entire live faction palette.**

`BRIEF_VIII_B_addendum.md` §D.2, headed "CORRECTION to Brief VIII §7.2".
Against `§owner TEAM IDENTITY`, `§owner BLUE ENEMY MECHS`, SPEC15's neon
red/blue, the Royal's gold, and now BRIEF X's orange. §D.2 was never
retired, only overtaken — and §D.7 still makes it the mech section's
completion criterion.

#### C5 — `TRV-0296` / `TRV-0320` · **The enemy Agile's livery. I described this BACKWARDS last run.**

**Correction first.** Run 1 said "the Agile Mech is orange on BOTH sides,
and three of six chassis are now in one hue band." **That is false.**
Re-derived from `agile_mech.rs:178-183`:

```rust
pub const ARMOR_ALLY: [f32; 3] = [0.90, 0.42, 0.11];   // industrial orange
pub const ARMOR_FOE:  [f32; 3] = [0.075, 0.125, 0.265]; // faction dark blue
pub const PLATE_ALLY: [f32; 3] = [0.56, 0.235, 0.075];  // burnt orange
pub const PLATE_FOE:  [f32; 3] = [0.40, 0.165, 0.060];  // deep rust
```

The enemy Agile ships **dark-blue primary armour** and carries the
chassis's orange on its **layered plates only**. I read the `_foe`
material *names* and inferred the values instead of reading the values. A
register that does that is doing the thing it exists to catch.

**The contradiction survives, inverted.** BRIEF X §2 says "**Primary
armour — orange.** This becomes the Agile Mech's recognizable visual
identity" and states no faction split. The build gives the opposition a
blue primary. The builder honoured both instructions **by role** and said
so (`FRIDAY_LOG.md:1531-1537`): *"Two owner rulings from the same day
pull opposite ways… If the owner wants the opposition Agile
orange-primary, `scout_hull_foe` and `scout_plate_foe` are the whole
change."* BRIEF XI §0.4 carries it forward unresolved and notes that
§16's "do not change the established colour identity" **does not** settle
it.

**What DID close:** SPEC15's trap 4 is now guarded — `TRV-0206` above.
The luminance question is answered (17× separation in linear light); the
livery question is not.

#### C6 — `TRV-0272` · **FRONT END P5 asked for a "large MECH preview"; what shipped previews the SOLDIER.**

Stated openly by the builder at `main.rs:19937` and in `7abed26`'s body:
*"STILL NOT RIGHT, stated rather than hidden… **The spec's word was
'mech'.**"* The size half is delivered and measured (portrait 720×1040
target, camera 3.00 → 2.57 m, card 366×520, rows at 72% width).
`PARTIAL`, pending a ruling.

#### C7 — `TRV-0273` · **FRONT END P6 named GAMEPLAY / CONTROLS / AUDIO / GRAPHICS. The build shipped CONTROLS / INTERFACE / CROSSHAIR.**

Stated deviation from `d91adb4`: *"**This game has no audio setting and
no graphics setting** — no volume, no resolution, no quality, no vsync."*
The shape the spec asked for is delivered and mutation-proven
(`every_setting_is_still_reachable_behind_some_tab`, `main.rs:28615`).
The unasked question the deviation exposes: **do you want them at all?**

#### C8 — `TRV-0318` / `TRV-0326` · **NEW. "Improve Agile Mech climbing animations" against a verb that may not exist for a mech.**

> BRIEF X §0 asserts the Agile Mech can climb.
> BRIEF XI §5 asks to *"improve Agile Mech climbing animations"* and §19 has a `Climbing` checkbox.
> Friday33: `climb_target` is gated `!m.in_mech()` **at the climber** and requires a **dropped plate on the target** — hull-climbing is a verb for a pilot **on foot** against a stripped enemy hull. Something you do *to* a mech, not *in* one.

BRIEF XI §0.1 is unusually explicit about the failure mode and I am
quoting it in full because it is the correct instruction for every agent
here: *"do not quietly animate a verb that never fires, and do not
quietly skip it either."* If the Agile cannot climb, §5 and the §19 box
are **not buildable as written** and become a new feature and a `sim.rs`
change — not a polish pass.

---

## §3 — TWO SPECS THAT LIVE ONLY IN A CHAT WINDOW

**This is the failure this register exists to catch, and it has now
happened twice more.**

`efe1428` wrote BRIEF X to disk before building anything and said why: *"a
spec that lives only in a chat window is the exact failure Trevor exists
to catch."* `dd4fced` did the same for BRIEF XI — 299 lines, on disk
before any build touched it. **That is the standard.** Two specs did not
get it.

### 3.A — THE BOW-AND-SPEAR SPEC (19 sections + an acceptance checklist)

Issued today. **Not in `briefs/`.** `git ls-files briefs/` returns 12
files; there is no bow/spear brief among them. Everything below was
reconstructed from `§owner` markers in `sim.rs` and two commit bodies.
**I can see six of nineteen sections. I cannot see the other thirteen and
I have no way to tell whether they were built, refused, or never read.**

**The two rules the owner stated THREE TIMES EACH. Flagged load-bearing.**

> **1. "RIGHT MOUSE IS PRE-AIM ONLY — it never starts a charge. The
> attack button charges. PRE_AIM and CHARGING are never combined."**
> The spec states it three separate times and calls it *"extremely
> important"* (quoted at `sim.rs:6620-6624`).
>
> **2. "The crosshair is sacred — nothing obstructs it."**
> Stated three times. Restated independently in BRIEF XI §15.

| Row | Section | Instruction (recovered) | Status | Evidence |
|---|---|---|---|---|
| `TRV-0301` | §4/§8/§17 | **RMB = pre-aim, LMB = charge, never combined** — the sim half | **`DONE`** | `enum AimPhase` + `pub fn aim_phase(ready, pre_aim, attack)` (`sim.rs:6634`, `:6654`). `if attack { Charging } else if pre_aim { PreAim } else { Equipped }` — **there is no input by which `pre_aim` alone reaches `Charging`.** Both step functions take an `AimPhase`, not a bool (`:10327`, `:10415`), so bow and spear cannot end up on different buttons. Test `right_mouse_pre_aims_and_only_the_attack_button_charges` (`:21543`) walks all seven input combinations and asserts three seconds of pure RMB winds no clock, launches nothing and spends no ammo. Commit `a95b48c`, 227 → 232 sim tests, all five mutation-proven from a file copy. |
| `TRV-0302` | §4/§8/§17 | The same rule, **client half** | **`NOT STARTED`** | `grep -c "aim_phase\|AimPhase" main.rs` = **0**. The HUD derives its own state from raw timers (24 references to `bow_draw_t`/`spear_charge_t`/`spear_wind_t`). `main.rs:16987` still documents RMB as *"a draw on projectile weapons (bow/spear, Brief II grammar)"* — the sim builder named this line explicitly as the client's to correct. |
| `TRV-0303` | §9/§10 | "a smooth progression with no visible steps" across **0-3 s**; low/medium/high power by second; **"~3 s reaching the normal maximum"**; holding longer buys no more power | **`UNVERIFIED`** | Being written **right now** in uncommitted `sim.rs`: `SPEAR_CHARGE_FULL_S` 0.85 → 3.00, curve deliberately kept linear with the shares argued in the comment (30.6% / 34.7% / 34.7%). **Not committed. Not counted as delivered.** |
| `TRV-0304` | §11 | **7 s MAXIMUM CHARGE BONUS**, "deliberate high-risk/high-reward, paid for in time and mobility", and it **"must not stack past that"** | **`UNVERIFIED`** | Same uncommitted work: `SPEAR_MAX_CHARGE_S = 7.00`, `SPEAR_MAX_CHARGE_DMG = 1.10`, resolved as a **threshold on a different axis** (velocity stops at 3 s, damage steps at 7 s) because the two spec sentences cannot both hold on one ramp. That reading is a builder's judgement call on your words and **is worth a sentence from you either way.** |
| `TRV-0305` | §B | The acceptance checklist hangs a **damage bonus off the charge** | **`UNVERIFIED`** | Named in `a95b48c`'s body as the reason defect 3 (the banked wind) mattered. Delivered by the same uncommitted work. |
| `TRV-0306` | (§11 / pose) | The charge must read as a real **overhead javelin wind** — arm raised, spear high and angled forward-down, opposite arm out for balance, weight into a wide stance | **`NOT STARTED`** | The sim half (`SpearStance`, `spear_stance_of`, `spear_wind_frac_of`, `spear_plant_frac_of`) is in the uncommitted diff. **The client pose does not exist.** The builder notes three separate places in `main.rs` hand-roll `1.0 - spear_wind_t / SPEAR_WINDUP_S` — a sim constant living on the wrong side of the boundary. |
| `TRV-0307` | §D | "existing player mechanics are preserved" — specifically the **parry / stagger** interactions | **`DONE`** | `a95b48c` defect 4: `try_fire` has always refused a staggered fighter, but **the bow spawns its arrow directly rather than through `try_fire`**, so a staggered archer loosed normally and a staggered thrower kept winding. Both paths now consult the parry. Test at `sim.rs:21475`. |
| `TRV-0308` | (stated ×3) | **"The crosshair is sacred — nothing obstructs it."** | **`PARTIAL`** | The FP viewmodel half already obeys it and is tested: `ScreenProfile::BowDrawn` — *"the whole bow stays below the centre circle, so the crosshair is never covered"* (`main.rs:2825-2829`) — and `SpearRaised`, both enforced by `every_weapon_holds_its_own_screen_profile` (`main.rs:24995`) against the **real** part corners, eleven or twenty per weapon. **What is NOT covered:** (a) the arc preview's **landing ring and drop-line**, which on a flat shot land where the crosshair is — see C1; (b) `main.rs:15512` states plainly *"The mech intrusion sweep does not cover hull mounts"*; (c) no capture holds RMB, so nothing has ever been photographed. |
| `TRV-0309` | follow-up task | Reference image — **bow layout** | **`BLOCKED` (owner)** | Not on disk. `git ls-files` finds no image under `handback/reference/` — the directory contains `NOTES.md` and nothing else. |
| `TRV-0310` | follow-up task | Reference image — **bow design** | **`BLOCKED` (owner)** | ditto |
| `TRV-0311` | follow-up task | Reference image — **spear design** | **`BLOCKED` (owner)** | ditto |
| `TRV-0312` | follow-up task | Reference image — **spear charging pose** | **`BLOCKED` (owner)** | ditto. This one is the acceptance criterion for `TRV-0306`: **the pose cannot be built to a picture nobody has.** |
| `TRV-0313` | — | **The spec itself is not committed anywhere.** | **`NOT STARTED`** | Archival defect, not a build defect. Paste it into `briefs/` once and this row closes forever — and thirteen sections stop being invisible. |
| `TRV-0314` | §1-§3, §5-§7, §12-§16, §18-§19 + checklist §A/§C/§E+ | The thirteen sections and the rest of the acceptance checklist | **`UNVERIFIABLE`** | Only §4, §8, §9, §10, §11, §17 and checklist items §B and §D leave any trace in this repository. I cannot tell whether the rest were built, refused, or never read. **This row is not a guess and must not be turned into one.** |
| `TRV-0315` | — | *(agent-origin)* **The `§N (owner)` marker convention has collided.** | **`NOT STARTED`** | `§8/§9 (owner)` at `sim.rs:357`, `:388`, `:6683`, `:21068` belong to the **30-section gameplay spec of 2026-08-07** (`6a46f61`). `§9/§10/§11 (owner, BOW & SPEAR)` in the uncommitted work belongs to **today's spec**. Same file, same marker, two different documents. The bow/spear work correctly appends `, BOW & SPEAR`; the older markers cannot be told apart. Cheap fix: every `§N (owner)` names its spec. |
| `TRV-0316` | §4/§8/§17 remainder | `step_plasma_precision` **still charges on RMB** | **`NOT STARTED`** | `sim.rs:8859` — `self.step_plasma_precision(p, cmd.aim, cmd.ads)`. Its own comment calls it "PLASMA BOW mode 2". The builder flagged rather than fixed: moving it needs the client's mount router to move with it, so it is cross-lane. **The one surviving RMB-charge in the file.** |
| `TRV-0317` | — | *(agent-origin)* **No capture in this repository holds RMB with a bow or a spear.** | **`NOT STARTED`** | `BOW_DRAW_FP_BEATS` (`main.rs:5699`) presses `MouseButton::Left` — which is *correct* under the new spec — and never presses Right. Five `bow_draw_fp` frames, five `bow_draw`, six `spear_flight`, six `arrow_flight`, and pre-aim appears in none of them. **This one capture settles C1, `TRV-0308` and half of BRIEF XI §15.** |

### 3.B — THE FRONT END SPEC (15 sections claimed, 6 recoverable)

Unchanged from run 1 and still not on disk. `TRV-0267`..`TRV-0281`,
`TRV-0297`, `TRV-0299`, `TRV-0300`. Eleven `DONE`, two `PARTIAL` behind
C6/C7, one `UNVERIFIABLE`, one `NOT STARTED` (the spec is not committed),
one `UNVERIFIABLE` for sections beyond P6.

**One row moved here today.** `TRV-0342`, new: the owner reported the
LEARN screen's BACK button doing nothing. `5473d06` found it was worse —
*"a trap door, not a dead button"*: one `NavReturn` slot served a
two-level path, so after a trip to MANUAL the player could not leave LEARN
**at all except by killing the process**. Fixed with a second slot
(`LearnReturn`), written only by `FrontAction::Learn`. **`DONE`.** The
test states its own limit honestly — it replays the writes rather than
running `front_buttons`, so it catches the slots being collapsed back
into one but would not catch `Back` being rewired to read `NavReturn`
again. That limit is `TRV-0343`, `NOT STARTED`.

---

## §4 — BRIEF XI (the Agile Mech's second brief) — 24 rows, 1 done

`briefs/BRIEF_XI_agile_mech_motion_limbs_guns.md`, 299 lines, committed
`dd4fced` **2026-08-10 22:39**. On disk before any build touched it, with
a §0 written by an agent recording three things the brief cannot know.
Owner's framing, quoted in the file: *"Finish the Agile Mech rather than
continually redesigning unrelated systems, while making the shared
limb/hand/weapon system explicit."*

**Nothing in this brief has been built. `dd4fced` adds one file and
changes no code** — the commit subject "finish the Agile Mech" describes
the brief's intent, not a delivery. That is exactly the misreading this
register exists to prevent, and I nearly made it.

| Row | Section | Status | Evidence / what is actually there |
|---|---|---|---|
| `TRV-0318` | §0.1 climbing is contested, **do not build blind** | **`BLOCKED` (owner)** | See C8. |
| `TRV-0319` | §0.2 `DECISION.md` item 3 **is** §1 and is binding — generalise `solve_arm_ik`, no second solver, no crate | **`NOT STARTED`** | `solve_arm_ik` (`main.rs:2585`) has 10 call sites and every one is an arm (`:18459`, `:18488`, `:18536`, `:18567`, `:18651`, `:18663`, `:20184`, `:20187`, `:25936`). No `leg_ik`, `foot_place` or `solve_leg` anywhere in `src/`. |
| `TRV-0320` | §0.4 enemy Agile livery | **`CONTRADICTED`** | duplicate-of `TRV-0296`. See C5. |
| `TRV-0321` | §0.4 **double jump has no airborne pose** — "the highest-value single fix in this brief" | **`NOT STARTED`** | `try_mech_jump`'s compress/tuck is heavy-only. The apex frame is indistinguishable from standing; the captures now exist to prove it (`agile_moves/08-double-jump-apex.png`). |
| `TRV-0322` | §1 foot & leg motion, no sliding, terrain adaptation | **`NOT STARTED`** | see `TRV-0319` |
| `TRV-0323` | §2 walking / running / sprinting differentiated, "lighter and faster than the Big Mech" | **`UNVERIFIED`** | A gait system and a sprint exist. I did not measure whether the Agile's three gaits differ per the brief's terms, and I will not guess. |
| `TRV-0324` | §3 jump prepares, land absorbs | **`PARTIAL`** | Compression exists **heavy-only**. Nothing for the Agile. |
| `TRV-0325` | §4 crouch with the whole lower body, "do not simply lower the character vertically" | **`PARTIAL`** | This is `TRV-0034` restated by the owner. `gait_pose` (`main.rs:2336`) still computes the crouch from `CROUCH_HEIGHT` (= 1.15, i.e. 0.646 × `BODY_HEIGHT`, the **infantry** ratio) and the kneeling-mech path (`main.rs:17794-17803`) blends two infantry poses. `MECH_CROUCH_HEIGHT_FRAC` (`sim.rs:5919`) appears **nowhere** in `main.rs`. |
| `TRV-0326` | §5 climbing — hands/feet to surface | **`BLOCKED` (owner)** | gated by §0.1 |
| `TRV-0327` | §6 aiming must not freeze the lower body | **`NOT STARTED`** | — |
| `TRV-0328` | §7 **two** limb families — DESIGN A player, DESIGN B mech; "do not create completely separate complicated systems for every character" | **`NOT STARTED`** | — |
| `TRV-0329` | §8 the reusable component tree (Arm → Upper/Elbow/Forearm/Wrist/Hand → Palm/Fingers) | **`NOT STARTED`** | The 20-segment rig **deliberately omits fingers** and says why: *"Fingers are deliberately absent — they are a sub-rig on the hands (§2), and counting them would push this past fifty and miss the point of the list"* (`main.rs:852`). That is a documented decision, not a gap — but the sub-rig it defers to does not exist. |
| `TRV-0330` | §9 mech arm design | **`NOT STARTED`** | — |
| `TRV-0331` | §10 mech hand & fingers must not clip through **guns, bow, spear, grenade** | **`NOT STARTED`** | `held_grenade.rs:189` draws "four finger bars and a thumb" as **geometry**, not articulation. |
| `TRV-0332` | §11 player hand design | **`NOT STARTED`** | — |
| `TRV-0333` | §12 shared Weapon → Grip Point → Hand IK → Arm → Shoulder, "should remove the need to hand-tune hand positions per weapon" | **`NOT STARTED`** | No `grip_point`/`GripPoint` symbol exists. `main.rs:20184-20187` drives both wrists through `solve_arm_ik` from per-weapon offsets — the hand-tuning this section wants to retire. |
| `TRV-0334` | §13 gun graphics pass — "guns are not placeholder geometry" | **`NOT STARTED`** | — |
| `TRV-0335` | §14 gun sits correctly in the mech hand, first **and** third person | **`NOT STARTED`** | — |
| `TRV-0336` | §15 FP weapon visuals — right side, **never covering the crosshair**, minimal sway | **`PARTIAL`** | Infantry weapons: enforced and tested (see `TRV-0308`). **Mech hull mounts: explicitly out of the sweep** — `main.rs:15512`, *"The mech intrusion sweep does not cover hull mounts"*. §15 says the rule applies to the Agile's guns. |
| `TRV-0337` | §16 keep the established colour identity, **do not change it this pass** | **`DONE`** | Satisfied by construction — the palette is unchanged since `8de5e93` and is now `pub const` data with a test pinning it (`TRV-0206`). Recorded as done because it is a *constraint* and the constraint holds, not because work was performed. |
| `TRV-0338` | §17 final integration checklist (20 named subsystems) | **`NOT STARTED`** | — |
| `TRV-0339` | §18 **reuse** — "one good reusable system over many separate systems that do the same thing" | **`NOT STARTED`** | The brief names this the *main technical objective*. |
| `TRV-0340` | §19 final quality check — 33 boxes across Movement / Combat / Character / Presentation / Technical | **`NOT STARTED`** | — |
| `TRV-0341` | PROOF STANDARD — every §19 visual box needs a screenshot; **"capture consecutive frames, not one pose"** for sliding and clipping | **`NOT STARTED`** | The brief names the two claims most likely to be ticked without evidence. Cheaper if `TRV-0040` (capture scripts as data) lands first. |

---

## §5 — BRIEF X — now closed but for one row

Run 1 said *"Immediate, cheap, and owed: commit the eight `agile_body`
PNGs, run `agile_moves`, and take one three-tier silhouette frame."*
**Two of the three happened, in `8de5e93` and `9b108b2`.**

| Row | Section | Status | Change |
|---|---|---|---|
| `TRV-0282` | §0 the three abilities must not break | `DONE` | Capture owed → **capture delivered.** `agile_moves` 12 frames photograph roll, jump, double-jump kick and apex, air flip first/inverted/second, landing. Climb is not among them and now cannot be — see C8. |
| `TRV-0289` | §8 animation & motion, no clipping | **`NOT STARTED` → `DONE`** | The re-timed script found a real defect the first table hid: *"the first table put the dodge roll AFTER the double jump and photographed a machine standing perfectly still twice."* Roll now runs off a walk, timed against `ROLL_LOAD_S + ROLL_S + ROLL_EASE_S`. |
| `TRV-0291` | §12 performance | **`UNVERIFIED` → `PARTIAL`** | ~150 parts / 3 meshes / 7 materials published. No draw-call measurement. |
| `TRV-0292` | §13 differentiate from Big and Royal | **`DONE`(contested) → `PARTIAL`** | `09-squint-derived.png` exists and is honestly labelled: not a matte, a 3.33× downsample with hue removed, because the harness has **no stencil or depth output** and a luminance threshold failed against a dark wall. Still no three-tier comparison. |
| `TRV-0298` | the khaki tripod on every scout and infantryman | `DONE` | Unchanged. Still the clearest proof in this project that Rule 8 pays: 438 tests and a compiler never saw it; one screenshot did. |

**What the roll capture found that nothing else could,** and why it
belongs in a register rather than only a build log: *"the tumble is the
ONLY camera angle in this game that looks at a chassis from underneath,
and the pelvis had no floor."* The pilot's torso showed through the
machine's crotch on every roll, and the pilot's **bright gold waist
stripe** had been reading as two yellow tabs on the machine's hips *in
every frame since the first build*. Four visual defects, all invisible to
the test suite.

---

## §6 — ASKS NOBODY HAS EVER PICKED UP

*Every row re-derived today at `5473d06`, not carried forward on faith.*

| Row | The instruction | Issued | Age | Re-derived today | Lane |
|---|---|---|---|---|---|
| `TRV-0260` | "**Save everything into `handback/reference/` and commit it — reference that lives only in a chat log is lost work.**" | `PROMPT_mech_rebuild` Task 1, `aefd16f` 2026-07-31 | **11 days — the oldest untouched ask** | `git ls-files` under `handback/reference/` returns **one file, `NOTES.md`**. The session that got the task had no image-download capability and said so. It never arrived. Four rows in two briefs are permanently unsatisfiable without it (`TRV-0075`, `0100`, `0115`, `0172`), and §D.7 makes the concept-art side-by-side the mech section's **stated completion criterion**. | **owner** |
| `TRV-0309`-`0312` | The four bow/spear reference images | today | today | Same directory, same answer. **The same failure, eleven days later, with the same directory named in the original instruction.** | **owner** |
| `TRV-0261` | The medic reference art — *"a squat utility robot, rounded masses, one big camera lens, worn amber over near-black"* | chat, quoted `WHATS_MISSING.md:513` | 4 days | Chassis built to it and photographed (9 `medic` frames). The image is not here, so nothing can be re-checked against it. | **owner** |
| `TRV-0030` | "**Every explosion is silent.** The sim publishes `Boom`; no sound exists. **Unblocker is `gen_sfx.py`, already in the repo.**" | `0-NOW` 2026-08-08 | 3 days | **Still true.** 21 `asset_server.load` calls, all `.wav`, list enumerated in §1.B task 5. No boom. `gen_sfx.py` writes no explosion either. **There has never been an owner blocker and the unblocker was named on day one.** | friday33, 1 h |
| `TRV-0036` | "**Armour damage states are invisible.** `armor_stage_of` has ZERO client readers." | `0-NOW` §A.3 | 3 days | **Still true.** `main.rs`: **0**. `sim.rs`: **66**. | friday33, 1 session |
| `TRV-0040` | "**Make capture scripts DATA, not code** — Thor's highest-leverage finding." | TIER 2 #19 | 3 days | **Still true.** `struct CapBeat` at `main.rs:5280`; every script is a compile-time const array. Every BRIEF XI proof and every bow/spear capture pays the 6-minute build tax without it. | friday33, 1 session |
| `TRV-0024` | `TDM_TARGET_CHOICES` unread | 2026-08-09 | 2 days | **Still true.** `sim.rs:430` defines it; the only other mentions are a doc comment at `sim.rs:4937` calling it a known mistake and a doc comment at `frontend.rs:321`. **No executable reader.** | friday33 |
| `TRV-0026` | `shot_handgun.wav` written by nobody's loader | 2026-08-09 | 2 days | **Still true.** `gen_sfx.py:73` writes it; it is on disk; **it is not in the 21-entry load list.** | friday33 |
| `TRV-0028` | `FORGE_SLOTS` unread | 2026-08-09 | 2 days | **Still true.** `main.rs:1386` defines it; the only other mention is `sim.rs:4937`'s doc comment. | friday33 |
| `TRV-0029` | The `9000.0` spray scale is a bare literal | 2026-08-09 | 2 days | **Still true.** `sim.rs:4515`, `:5481` (plus `:23760`, `:23761` in tests) and the client keeps its own copy at `main.rs:308`. | friday22 + friday33 |
| `TRV-0010` | "**Royal ARROW LAUNCHER: minigun + 3 crossbows.**" | SPEC15 **P2** 2026-08-09 | 2 days | **Still zero.** `crossbow` / `arrow_launcher` / `ArrowLauncher`: no hits in any weapon path. `MechWeapon` = Gatling / Autocannon / Rockets / Plasma / Repair (`sim.rs:5173`). **The last open P2 row and the only thing that would make a Royal not-a-bigger-Big.** | friday33 + one toto number |
| `TRV-0034` | `gait_pose` bakes the INFANTRY crouch ratio | `0-NOW` §A.2 | 3 days | **Still true, and now restated by the owner** as BRIEF XI §4 — see `TRV-0325`. | friday33 |
| `TRV-0150` | "**Powered armour** · RESEARCH ONLY, DO NOT BUILD… Produce the spec, stop there." | `PROMPT_MASTER` Task 8 | 8 days | The "do not build" half was obeyed. The "produce the spec" half was never started; `research/powered-armour/` does not exist. | — |
| `TRV-0136` `TRV-0139` | Tier-4 cosmetics and the **12 named preset loadouts** | BRIEF_IX-C | 12 days | Zero. | friday33 |
| `TRV-0119` `TRV-0120` | Castle Heart / Gatehouse Signal / objective inversion at 5:00; surface-aware movement audio | BRIEF_IX-A | 12 days | KOTH has one hill. Neither exists. | friday22 + friday33 |
| `TRV-0187` | "**ROTATING CODEBASE REVIEW**, four categories, each examined every five cycles" | `PROMPT_RND_CYCLE` §5 | 8 days | **Never run once.** The scouts do the equivalent ad hoc and find real things — arguably better — but the instruction has no execution record. | — |
| `TRV-0157` | "Work on branch `claude/master-research`…" | `PROMPT_MASTER` preamble | 8 days | No such branch exists; all work landed on `main`. Recording the divergence, not litigating it. | — |
| `TRV-0344` | *(agent-origin, new)* `config/settings.txt` carries `fov_idx = 4` (100°) against `FOV_DEFAULT_IDX = 3` (`main.rs:1588`) | — | — | **Every capture in this repository was taken through it.** Comparisons within a pass are same-settings and valid; none is what a clean checkout produces. | friday33, minutes |

---

## §7 — WHERE I DISAGREE WITH THE RECORD, AND WHERE I WAS WRONG

**What I got wrong last run** — first, and named:

| Row | What I wrote | What is actually there |
|---|---|---|
| `TRV-0296` / C5 | "The Agile Mech is orange on BOTH sides, and three of six chassis are now in one hue band." | **False.** `ARMOR_FOE = [0.075, 0.125, 0.265]`, *"faction dark blue"* (`agile_mech.rs:179`). I read the `_foe` material **names** and inferred the values instead of reading the values. The contradiction is real; my description of it was backwards, and it was the more alarming of the two directions. |
| `TRV-0289` | "the `agile_moves` capture script exists in code and has produced no PNGs" | True when written. **Twelve frames landed 90 minutes later** (`8de5e93`). |
| §8 of run 1 | "the 14 `UNVERIFIED` rows are the honest total of the above" | The headline of the same file said **18**. I criticised the ledger for having three numbers for one set and then shipped two. Fixed: 21, and §8 no longer restates a count. |

**Where I disagree with other documents** — all re-checked today:

| Document | Its claim | What I found |
|---|---|---|
| `WHATS_MISSING.md` | The live plan | **Last touched `7719296`, eleven commits ago. Stale for the fourth time, in both directions.** Lists as open: three TIER 0 items that are done, Pyro relocation tables that are gone, traversal's map-metrics block, `TRV-0011` the Agile Mech, `TRV-0206` the luminance guard. Knows nothing of BRIEF X's handback, BRIEF XI, the bow/spear spec, or the front end. |
| `briefs/README.md` | What each brief covers | Does not list `BRIEF_XI_agile_mech_motion_limbs_guns.md`. |
| `TREVOR_LEDGER.md` `TRV-0071`, `TRV-0091` | Spear/bow `DELIVERED` | The second half of both asks is violated — see C1. **This was run 1's headline finding and I applied its lesson to `dd4fced` today:** the commit subject is "BRIEF XI: finish the Agile Mech" and the commit **changes no code at all**. |
| `BACKLOG.md` #4/#5/#9/#11 | melee depth / retreat / class system / armour weight unbuilt | All four false; all four shipped. #10 cites rapier, which `jk_tdm` does not depend on. |
| `DECISIONS.md` | A project decision record | Every ADR is about `jk_wall`/`jk_core`. ADR-006 promises glTF + skeletal animation; `jk_tdm` has no glTF loader. ADR-002 promises Rapier; `jk_tdm` has none. |
| `GAME_STATUS_REPORT.md` (2026-08-01) | Lists Pyro; names the 20-segment rig, 26-piece armour and 4-class system as unbuilt | All four wrong, reads as current. |
| `handback/brief-ix/REPORT.md` | Class system, 26-piece armour, damage states "do not exist in any form" | All three shipped. Honest snapshot of 2026-07-30. |
| `README.md:223-225` | "battles cap at 8v8" | Retired by `§owner: "8v8 withdrawn"` (`main.rs:23200`). Doc rot, not a live contradiction. |

---

## §8 — WHAT I COULD NOT CHECK, AND WHY

Stated as a section rather than buried, because implying a complete sweep
is the failure this file exists to prevent.

- **I did not build and did not run the suite.** Read-only; I never invoked `cargo`. Every `DONE` means "there is evidence at this path", never "it compiles" and never "it works". The last reported figure is 443 pass / 1 fail, and the one red test (`a_wound_javelin_flies_harder_than_a_flicked_one`) is **the uncommitted spear-charge work in `sim.rs`** — another lane mid-write, not a regression.
- **I read the uncommitted `sim.rs` diff and refused to bank it.** `TRV-0303`, `0304`, `0305` are `UNVERIFIED` on purpose. When that work commits, someone should re-derive them; three rows becoming `DONE` in one commit is the expected outcome and it is not my call to make early.
- **I did not read the uncommitted `frontend.rs` diff at all.** +20 lines, another lane, moving target.
- **Thirteen of the bow-and-spear spec's nineteen sections are invisible to me** (`TRV-0314`). So is its acceptance checklist beyond items §B and §D. I reconstructed six sections from `§owner` markers and two commit bodies. I have no way to see the rest and **no way to tell whether they were built, refused, or never read.** The same is true of FRONT END P7+ (`TRV-0297`).
- **I opened no PNGs this run.** Run 1 opened two of 165 and neither settled C1. There are now ~193 committed captures and the number that hold RMB with a projectile weapon is still **zero** — which is `TRV-0317` and is the finding, not a gap in my sweep.
- **Read whole:** `BRIEF_XI_agile_mech_motion_limbs_guns.md`, `ISSUED_VS_DELIVERED.md` (run 1), the five commit bodies since `889701a` in full, `FRIDAY_LOG.md` §FRIDAY33 BRIEF X, the uncommitted `sim.rs` diff in full, `agile_mech.rs` palette + luminance tests, `sim.rs` §6604-6690 (`AimPhase`), `main.rs` §2792-2970 (screen profiles + low-ready), `main.rs` §16975-17020, `config/settings.txt`.
- **Sampled by targeted grep + region reads:** `main.rs` (~29k lines, ~9 regions), `sim.rs` (~28k lines, ~7 regions), `briefs/*` (grep only, except BRIEF XI), `frontend.rs` (grep only).
- **Not read at all this run:** `THOR_LOG.md`, `TOTO_LOG.md`, `research/maps/MAP_METRICS.md`, `research/motion-architecture/DECISION.md` (I verified they exist and are committed and cited what BRIEF XI §0.2 quotes from the second; I did not audit either file's content), `handback/ACCOMPLISHMENTS.md`, `AUDIT.md`, `CHANGES.md`, the eleven per-topic `SOURCES.md`, everything in `jk_wall` / `jk_bevy` / `jk_client` / `jk_spike` / `jk_core`.
- **The 21 `UNVERIFIED` rows are the honest total of the above.** None carries a fabricated disposition. None was bucketed as a negative result because a check did not finish — that is the failure mode this project has hit twice (46 verify agents killed by a rate limit and filed as "disputed"; three agents' research discarded by a missing `await`).
- **Concurrency.** Two builders and a separate session are live. Line numbers here were true at `5473d06` with the working tree as described at the top. **Every row also names its symbol; re-anchor on the symbol, `git fetch` first.**

---

## §9 — THE REGISTER, BY SOURCE

| Block | Source | Rows | Open |
|---|---|---|---|
| A | `WHATS_MISSING.md` §0-SPEC15 (P1-P4) | 0001-0021 | **1** (`TRV-0010`) at P1/P2; 2 at P3; P4 untouched |
| B | §0-QUEUE Tiers 0-4 | 0022-0053 | 20 |
| C | §0-NOW | 0054-0065 | 8 |
| D | `BRIEF_VII_optimized.md` | 0066-0077 | 3 + **C1** |
| E | `BRIEF_VIII_master.md` | 0078-0100 | 7 + **C1, C2** |
| F | `BRIEF_VIII_B_addendum.md` | 0101-0115 | 6 + **C3, C4** |
| G | `BRIEF_IX` A/B/C | 0116-0141 | 17 |
| H | `PROMPT_MASTER_research_build.md` | 0142-0157 | 11, most superseded by OPERATION rule 13 |
| I | `PROMPT_brief_X_research.md` (superseded, indexed) | 0158-0166 | 0 live |
| J | `PROMPT_mech_rebuild.md` (superseded, indexed) | 0167-0173 | 2 |
| K | `PROMPT_RND_CYCLE.md` + `BACKLOG.md` | 0174-0187 | 8 |
| L | `PROMPT_motion_system_research.md` | 0188-0190 | 0 — closed 2026-08-10 |
| M | Chat asks recorded second-hand | 0191-0234 | 5 |
| N | Agent-origin | 0235-0255 | 12 |
| O | Images — uploaded, missing, generated | 0256-0266 | 3, all `BLOCKED` on the owner |
| P | FRONT END spec — chat only | 0267-0281 | 4 |
| Q | BRIEF X | 0282-0293 | **2** (was 3) |
| R | Contradictions and findings, run 1 | 0294-0300 | 3 contradictions + 4 |
| **S** | **BOW & SPEAR spec — chat only, 19 §§ + checklist** | **0301-0317** | **13** |
| **T** | **BRIEF XI** | **0318-0341** | **23** |
| **U** | **Chat asks and findings, run 2** | **0342-0344** | **2** |

---

*Written by TREVOR, run 2. I do not edit source, briefs, or another
agent's log. Where a document is wrong I record the disagreement here and
hand it over. Where I was wrong, §7 says so first.*
