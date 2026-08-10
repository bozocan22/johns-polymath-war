# TREVOR TASKS — everything that needs to be worked on

**Generated 2026-08-10 from `TREVOR_LEDGER.md` run 1.** Rewritten in full
each run. Every task carries its `TRV-` rows, so the ledger is the "why"
and this file is the "do".

**266 asks indexed · 111 delivered · 104 open · 21 unverified.**

## How this is ranked

1. **The owner's own priority order first** — `WHATS_MISSING.md` §0-SPEC15
   P1 → P2 → P3 → P4, then §0-QUEUE Tier 0 → 4. I did not re-rank into my
   own taste. Where I would have ordered differently, I say so in one line
   underneath and leave the owner's order standing.
2. Then by what unblocks the most other rows.
3. Then by cost.

**Lanes.** `sim.rs` → **friday22**. `main.rs` + client modules
(`held_grenade.rs`, `mech_lineup.rs`, `mech_recoil.rs`, `cockpit.rs`,
`menu_ui.rs`, `branding.rs`, `map_look.rs`) → **friday33**. Those are the
only two. A third builder has nowhere to go.

**Warn every dispatch about the transient:** both builders run
`cargo test`, which compiles both files, so a suite run during the other
lane's write fails for no reason. Re-run before concluding a failure is
real.

---

# BAND 0 — BLOCKED ON YOU, AND ON NOBODY ELSE

Eight questions. Answering them unblocks 12 rows and stops two builders
guessing on your behalf. Nothing in this band is work — it is a decision.

### Q1 — The mech concept art is not in this repository. Can you re-supply it?
`TRV-0260` · blocks `TRV-0075`, `0100`, `0115`, `0172`

`BRIEF_VIII_B` §D opens with **"The art is the spec"** and §D.7 makes
"place it next to the concept art in the handback" the *stated completion
criterion* for the whole mech section. `PROMPT_mech_rebuild.md` Task 1
says it in plain words: *"reference that lives only in a chat log is lost
work."* It was lost. I checked `git log --diff-filter=D` across every
image type — no image of this kind was ever committed and later deleted.
It never arrived.

Until it does, three completion criteria in two briefs are unsatisfiable,
and every judgement about whether the machine matches the art is one
person's memory of a conversation.

**Drop it into `engine/crates/jk_tdm/handback/reference/`.**

### Q2 — Same for the medic reference art.
`TRV-0261`

The record quotes you: *"a squat utility robot, rounded masses, one big
camera lens, worn amber over near-black."* The chassis was rebuilt to it
and photographed (`medic/01..09.png`). The image itself is not here.

### Q3 — Should the mech's first-person aim leave the visor, or the hull turret?
`TRV-0053` · the plan itself files this as **"A DECISION FOR THE OWNER, not a task"**

The camera sits at the visor (2.7234 m). Every mech weapon fires from
`EYE_REL` (1.62 m). The gap is **1.1034 m** — your "1.10 m" confirmed
exactly, by independent arithmetic, by Thor. A hull turret genuinely *is*
a metre below the visor, so this may be correct and merely unstated.

Changing it moves every mech engagement, hit test, cover line and tracer
in the game. Nobody should decide that quietly for you.

### Q4 — ✅ ANSWERED BY OWNER, 2026-08-10. CLOSED.
`TRV-0013`

The question was: SPEC15 P3 asked for *"SUBTLE neon-blue energy accents
(channels, seams, reactor) — accent, not a coating"*, and the builder
instead shipped deep bronze-graphite with gold, writing the argument down
at `main.rs:14288-14307` rather than hiding it.

**Owner's ruling:** *"keep the royal mech, and keep the opposition royal
mech colour red yellow and neon blue details."*

1. **Player Royal: the GOLD STAYS.** No rework, no re-open. The builder's
   call is ratified. SPEC15's neon-blue line is superseded and marked so
   in `WHATS_MISSING.md`.
2. **Opposition Royal palette is now RED + YELLOW + NEON-BLUE details** —
   red primary, the other two as detail accents, not coatings. This
   replaces the old "neon-red/dark-red primary with dark-blue
   complements" in SPEC15 P3.
3. **The consequence Task 4 and Task 5 must now carry:** the player Royal
   is gold and the opposition Royal carries yellow, so *colour no longer
   separates them*. "Must not read as a recoloured player Royal" now
   rests entirely on body and silhouette. Apply the squint test — if the
   two read alike as black silhouettes at 30 m, the fix is the chassis,
   never the palette.

Assumption flagged, since it steers two tasks: I read *"keep the royal
mech"* as keeping the player Royal exactly as shipped. If you meant
instead that the player Royal should keep the neon-blue it was originally
specified, say so and Task 4 changes shape.

### Q5 — `SCOUT_SCALE = 1.05` makes the medic 1.87 m. Is that the intent?
`TRV-0059`

Thor's note: *"If the owner's intent was 'reads as a machine, not a man',
1.05 does not deliver it. Owner call, not a defect."* 1.05 × 1.78 m is a
big man, not a machine. The constant's own history says it was once 1.42.

### Q6 — The recoil envelope cost bot mechs about a third of their sustained output. Keep it?
`TRV-0239`, `TRV-0240`

Friday volunteered both: bot mech damage at 10 m fell 492 → 331 over 17 s,
and braced turret fire is no longer perfectly accurate (0° → ~1.6°). The
direction is what you asked for. The magnitude is a balance change nobody
signed off. **These want a playtest, not an argument.**

### Q7 — The scout's hitbox no longer shrinks when rolling or crouching.
`TRV-0248`

It is now the only fighter in the game whose height never changes in any
stance. A dodge that does not duck the head band is a different verb.

### Q8 — `armor_spec`'s flat values are unreachable for any piloted heavy chassis.
`TRV-0236`

`apply_armor` takes the hull/angle branch and returns before them. True of
the Big Mech's row too, and long-standing. Friday's words: *"Worth someone
deciding on deliberately."* Keep them scaled so the table stays
consistent, or delete them?

---

# BAND 1 — SPEC15 **P1 / P2**: the only unfinished architecture and gameplay rows

P1 is complete. P2 is one row short.

## TASK 1 — Build the Royal ARROW LAUNCHER
`TRV-0010` · **friday33** (+ friday22 for the numbers) · **1-2 sessions** · SPEC15 **P2**

**Your words:** *"Royal ARROW LAUNCHER: minigun + 3 crossbows. Compact
minigun silhouette, rotating mechanism, three crossbow assemblies around a
central weapon, bolt ammunition, mechanical loading."*

**Status: genuinely absent.** I grepped the whole `src/` tree for
`crossbow`, `arrow_launcher` and `ArrowLauncher` — zero hits in any weapon
path. Every "bolt" in the codebase is a rivet, a rifle bolt, or the
medic's plasma bolt. `MechWeapon` carries Gatling / Rockets / Autocannon /
Plasma / Repair and nothing else.

**This is the last open P1/P2 row, and the only mech weapon the Royal tier
has that the Big does not.** Right now a Royal is a Big Mech with more
hull, which is exactly what P3 says it must not be.

**What the thread already knows:**
- The Royal is a real `ArmorSet` with its own pad, hull pool and hitbox
  (`sim.rs:4802`, `:4838`). The weapon slot is the missing half.
- `MechWeapon::for_set` is the DATA row that decides which mounts a
  chassis has — add there, not in a forked spawn function. SPEC15's own
  trap 1 says the same thing.
- `a_number_key_selects_the_mount_the_strip_labels_with_it` exists
  specifically because a hardcoded `0 => Gatling, 1 => Rockets` once made
  the medic's repair beam unreachable. **The strip and the key handler
  must read one list.** That test will catch you if they drift.
- The turret already proves the pattern for "rotating mechanism you can
  see": `main.rs:10289` `§owner: IT HAS TO READ AS A MINIGUN`, and the
  defect it records (a "half-cowl, open below" built from cylinders facing
  down the barrel axis, i.e. **discs**, which capped the gun and hid all
  six barrels) is the trap to avoid on the crossbow assemblies.
- SPEC15 trap 3: **silhouette beats paint.** Three crossbows around a
  central minigun is a silhouette element. Use it.

**The one number nobody has** (`TRV-0010` in §E.2): rate of fire, bolt
velocity, and per-bolt damage against a mech hull. Nothing in the game
fires a bolt from a mount, so there is no neighbouring value to
interpolate from. If a `toto` dispatch is warranted anywhere, it is here —
and the dispatch must name that number, not the topic.

**Done when:** a Royal pilot can select it from the strip by number, it
fires bolts with their own physics, the barrels visibly rotate, and there
is a capture of it firing.

---

# BAND 2 — SPEC15 **P3**: the visual block, in the owner's own order

**This is the largest open block you named, and four of its five bullets
asked for GEOMETRY and received PAINT.** Take one per session. Do not
merge them.

Hand every one of these three things from SPEC15's own trap list:
- **Trap 3 — silhouette beats paint.** A variant identifiable only by
  colour does not exist at range. Each variant needs one silhouette
  element. That is why the Royal got a crown.
- **Trap 4 — the luminance rule is unguarded.** Ally/enemy separation
  rests on *value*, not hue, and the enemy's light blue is already
  brighter than the ally's brightest tone. Only part coverage saves it.
  Neon on dark is safe; neon over large areas is not.
- **Trap 5 — capture everything.** Five defects in one week were invisible
  to the compiler and to 400 tests, and obvious in a screenshot.

## TASK 2 — Agile Mech: the major visual upgrade
`TRV-0011` · **friday33** · **1-2 sessions** · SPEC15 **P3, and you called it "the largest visual item"**

**Your words:** *"Must read FASTER, LIGHTER, more advanced, more
mechanically detailed, and clearly distinct from Big and Royal in
silhouette. Plates, joints, legs, hydraulics, torso, shoulders,
head/cockpit, weapon mounting, small components, surfaces, energy
details."*

**Status: NOT STARTED under SPEC15.** No `§P3` marker touches the scout
chassis anywhere. The 2026-08-07 medic redesign (`72a93f3`) is a real,
good pass — but it answered a *different* ask: your reference art of a
squat utility robot. "Squat utility robot" and "faster, lighter, more
advanced" are not the same brief, and nobody wrote down that the second
one superseded the first.

**What the thread already knows:**
- The medic pass took ~90 exposed struts/pistons/louvres down to ~45
  nameable masses, gave it one LENS eye at visor height, and replaced the
  digitigrade frame with plantigrade thick legs — *because the digitigrade
  frame read fast but also skittish, and half its hardware existed only to
  explain itself*. **If you now want it to read FAST again, that decision
  is the one to revisit deliberately, not delete.**
- The ally shell went AMBER on the argument that the team read never rests
  on the shell. Adding "energy details" is exactly the case trap 4 warns
  about — accent, not coverage.
- `SCOUT_SCALE` is 1.05 and is now live in `height()`. See Q5.
- Captures already exist to compare against: `medic/01..09.png`.

**Done when:** the scout reads as a different machine from the Big and the
Royal *in black silhouette at 30 m*, and there is a capture proving it —
the three tiers in one frame, which `mech_lineup.rs` can already produce.

## TASK 3 — Rocket Launcher redesign
`TRV-0012` · **friday33** · **1 session** · SPEC15 **P3**

**Your words:** *"chambers, mounting, reload mechanism, barrel detail,
materials, VFX, firing animation, recoil."*

**Status: PARTIAL.** Shipped in the FLAGSHIP PASS (`aa14bcf`,
`main.rs:10988`) and the electronics pass (`5e09907`): ribbed casing,
mouth jaws, backblast vents, a feed arm mid-travel, a boxed seeker head,
targeting electronics. **Absent:** the reload mechanism as a visible
action, muzzle VFX, a firing animation, and the recoil answer.

**What the thread already knows:**
- `mech_recoil.rs` already exists and already reads the sim's own
  `punch`/`punch_vel` rather than a client table. The launcher's recoil
  answer belongs there, not in a new file. Read that module's header
  first — it exists specifically to stop a second, drifting recoil model.
- The turret got its detail pass by the same additive method (the module
  ring: 2 tracking, 2 cooling, on the static housing). Reuse it.

## TASK 4 — The Royal gets its own body
`TRV-0013` (body half), `TRV-0015` · **friday33** · **1-2 sessions** · SPEC15 **P3**

**Your words:** *"its own body and silhouette, NOT a scaled Big Mech."*

**Status: not started.** `main.rs:11414` says it plainly: *"§22 THE ROYAL
VARIANT. Same machine, same 53 plates, same [everything]."* The Royal is
the Big × 1.10.

**Blocked on Q4 for the accent colour — but not for the geometry.** Start
the geometry now.

**What the thread already knows:**
- The paint half is done and was worth doing: `a0f67ae` split the two
  Royals because *"ROYAL/ALLY and ROYAL/ENEMY differed only by a lamp"* —
  the spec's own "must not read as a recoloured player Royal", failed in
  the most literal way possible, caught in one frame.
- `spawn_armor_rig(commands, kit, ally, elite)` already takes `elite` as a
  parameter, so the fork point exists.
- `MECH_LEGS_ROYAL` (`main.rs:2225`) already marks where the Royal's own
  leg-armour pair starts — that is the pattern to extend, not replace.

## TASK 5 — Opposition mechs stop being recolours
`TRV-0014` · **friday33** · **1-2 sessions** · SPEC15 **P3**

**Your words:** *"Own armour design, body structure, silhouette,
mechanical detail, weapon styling — while keeping the faction colour
language."*

**Status: PARTIAL — the colour language shipped, the structure did not.**
The enemy machines are the ally machines with a different material table.

**What the thread already knows:**
- The colour language is already DATA (`branding.rs` `Side::steel()` /
  `Side::accent()`), so a structural change does not have to touch it.
- Thor has already measured the separation: ally 0.239 vs enemy 0.0178 =
  **13.4×** relative luminance; scout 0.380 vs 0.0158 = **24×**. You have
  headroom. Spend it on shape.
- **Do the guard test in the same session** (`TRV-0206`): Thor's finding
  is that the luminance rule *is now false as stated* and **nothing tests
  it**. One test pinning ally/enemy luminance separation costs almost
  nothing and stops a future repaint silently breaking friend-or-foe.

---

# BAND 3 — THE INVISIBLE SYSTEMS

Built, tested, and the player cannot see any of it. Highest felt value per
session in the whole file after Band 2.

## TASK 6 — Armour damage states get a client half
`TRV-0036`, `TRV-0137`, `TRV-0140` · **friday33** · **1 session** · 0-QUEUE Tier 2 #15

**Status, re-derived today: `main.rs` contains ZERO references to
`armor_stage_of`, `armor_wear_of` or `ArmorStage`.** Fresh, Scuffed and
Cracked render identically. Only Severed shows, and only because it
removes the piece.

**What the thread already knows — this is a fully-specified handoff:**
- The sim publishes everything you need: `TdmSim::armor_stage_of`
  (returns `Option` — **`None` is a bare mount, and a client must not draw
  clean steel on a naked shoulder**), `TdmSim::armor_wear_of`,
  `ArmorStage::label`, `ArmorStage::tilts`, `ArmorStage::resist`,
  `ArmorPiece::struck`, `HitZone::band`, `ArmorCondition::{hp,frac,stage,wear,repair}`.
- `ArmorStage::tilts` exists **and nothing draws it.** "Piece tilts at
  Cracked" is a brief requirement with a published accessor and no reader.
- Detach is already the *unequipped* path — `armor_pieces.set(p, false)`,
  the same bit the Forge switch clears — so the client needs no second
  visibility rule. The test
  `a_shot_off_plate_is_indistinguishable_from_one_never_worn` will notice
  if that ever grows a second flag.
- Brief IX-C's own table gives you the visual language: Fresh = clean
  plate; Scuffed = light surface scratches, edge dulling; Cracked = deep
  gouges, fracture lines, loose rivets; Severed = detached or hanging.
- The 24 plate groups already exist as separate geometry and visibility
  already follows the loadout (`TRV-0186`) — so you are changing
  materials, not building meshes.

**Done when:** the four-frame damage progression capture that Brief IX-C
asks for exists (`TRV-0141`), because right now that capture is
*impossible*, not merely missing.

## TASK 7 — Mech boarding: make seven of eight stages visible
`TRV-0037`, `TRV-0174` · **friday33** · **1 session** · 0-QUEUE Tier 2 #16

**Status: PARTIAL, and better than the plan says.** The client *does* read
`mech_enter_stage_for` now (`main.rs:19191`), fires one rising `click` per
beat, and drives `visor_ready`. But `main.rs:19243-19254` is eight
`debug!` arms and nothing else.

**What the thread already knows:**
- **The strings already exist verbatim inside the `debug!` calls**:
  "cockpit opens" / "pilot climbs in" / "harness closes" / "power-up, seam
  lights" / "servo sync" / "gyro calibration" / "weapon diagnostics - both
  hull mounts cycle" / "HUD boot - camera may cut to the visor". That is
  the spec, already written, by the person who built the timer.
- The system already fires **only on a stage CHANGE**, never every frame —
  the plan's "one at a time" rule, which is what keeps it reading as a
  machine waking up. Do not break that.
- `visor_ready_after` is pure and tested, and its doc explains the trap:
  `mech_enter_stage_for` returns `None` both when boarding has finished
  *and* when the fighter is not a mech at all, so the naive
  `matches!(stage, None | Some(HudBoot))` would snap the camera into a
  visor that does not exist.
- `visor_ready` is still a field of a `Local`, so nothing outside the
  system can read it (`TRV-0057`). If the camera cut is meant to be real,
  that flag has to leave the `Local`.
- Cycle 1's research is on disk and argues *why* the sequence must be
  committal: `research/mech-entry/CYCLE_1_REPORT.md`, grounded in aviation
  checklist design and why interruptible sequences fail.

## TASK 8 — Every explosion is silent
`TRV-0030` · **friday33** + `gen_sfx.py` · **1 hour** · 0-QUEUE Tier 1 #9

Re-derived today: `main.rs:14096-14116` is the complete `Sfx` load list —
20 wavs, no explosion, no boom. The sim publishes `Boom` and nothing
listens.

**Do the four placeholders in the same sitting**, because they are the
same file and the same hour:
- plasma, repair beam, barrier deploy and precision charge all play
  `shot_mp5`, marked as placeholders at the call site (`main.rs:21311`).
- every boarding stage plays `click` (see TASK 7).
- `shot_handgun.wav` is generated by `gen_sfx.py:73`, sits on disk, and is
  loaded by nothing (`TRV-0026`).

**There has never been an owner blocker on any of this.** The unblocker
was named as `gen_sfx.py` and `gen_sfx.py` has been in the repo the whole
time.

---

# BAND 4 — THE HONESTY FIXES

Six one-line defects where two screens in the same game disagree with each
other. Cheapest rows in the ledger. **Bundle them into one session.**

## TASK 9 — The screens that lie to the player
**friday33** · **1 session for all six**

| Row | The defect, re-derived today |
|---|---|
| `TRV-0031` | The Field Manual's MODES line prints the constant `TDM_TARGET` (`main.rs:24303`), not the target you chose. Every other number on that screen was correctly moved to live constants — this line was missed. |
| `TRV-0024` | `TDM_TARGET_CHOICES` (`sim.rs:430`) is declared and its only other mention in the crate is a doc comment calling it a known mistake. The menu hand-types 30/60. Fixing `TRV-0031` and this together is one change. |
| `TRV-0032` | `BIND_REGISTRY`'s only `U` row says "Dismount the mech". The in-world prompt says "U - GRAB THE HULL" (`main.rs:22802`). Hull climbing is not in the full bind list. |
| `TRV-0033` | The `Q` row names roll and flip. The medic's **second flip charge** (`sim.rs:3427`) and its **single mid-air jump** (`sim.rs:3436`) appear nowhere a player can find them. Put them on Controls, **not** the equip hint — that line already overflowed once and was cut to two facts for exactly that reason. |
| `TRV-0028` | `FORGE_SLOTS` (`main.rs:1372`) is declared and read by nothing; `forge_slot_path` and its callers all hardcode. |
| `TRV-0055` | `gatling_heat` carries two scales in one sim field. `main.rs:21763`/`:21769`/`:21772` print `×100` under a `%`; `:21781` prints the **raw** value under the same `%`. Both branches are live. Split the field or document it — the sim side is friday22's. |

**Why these matter more than their size:** every one of them is the game
telling the player something false. `ANTI_PATTERNS.md` has a name for the
class — **"the confident narrator"** — and it was earned here.

---

# BAND 5 — THE LEVERAGE TASK

## TASK 10 — Make capture scripts DATA, not code
`TRV-0040`, `TRV-0204`, `TRV-0251` · **friday33** · **1 session** · 0-QUEUE Tier 2 #19

**Thor ranked this his highest-leverage finding and he is right.**

`main.rs:5203` `struct CapBeat`, and every script is a compile-time const
array inside a 29,000-line file. A framing tweak therefore costs a full
release rebuild: **~6 minutes, versus ~40 seconds if the beats were data.**

Your own words, quoted in `THOR_LOG.md:2829`: *"several tasks needed 3+
iterations purely on camera framing."* Three iterations × 6 minutes = 18
minutes of pure rebuild to move a camera, and it recurs on every visual
task in Band 2.

**What the thread already knows — three properties of the rig, learned the
hard way, that must survive the change:**
- The boom anchors on the **HEAD**, so closing the distance magnifies the
  offset between anchor and subject.
- **Pitch orbits the CAMERA about the anchor** rather than tilting the
  view, so positive pitch photographs the top of a hat.
- `look` turns the PLAYER and the third-person boom is rigidly behind the
  player, so no yaw ever yields a profile. `CapBeat.orbit` swings the boom
  around a stationary subject **and re-aims at the anchor** — the first
  attempt did not re-aim and photographed the scenery beside the machine.
- Beat times must not run backwards; every script's last beat must set
  `end`. Both are pinned by tests.

**This task pays for Band 2, Band 3 and the four owed captures below. Do
it early.**

## TASK 11 — The four captures the ledger owes
`TRV-0008`, `TRV-0005`, `TRV-0056`, `TRV-0243` · **friday33** · **1 session, after TASK 10**

Rule 8: *a visual claim with no screenshot behind it is not a claim, it is
a hope.* Four shipped systems have none.

| Capture | Why |
|---|---|
| **The recoil envelope** | The single biggest feel change in the game — AUTO plateau 4.09° → 16.22° — and nobody has photographed the controlled window becoming chaotic. Beat it at 0.5 s / 1.5 s / 3 s / forced vent. |
| **The weapon strip as four slots** | `Shield [4]` shipped as an interactive slot with a border, a fill and a quantity. No frame shows it. |
| **Per-owner turret spinners + the world minigun spin** | Both claimed fixed, both listed as capture-verification owed since 2026-08-08, both still `UNVERIFIED` in this ledger for exactly that reason. |
| **The kill-pop X** | Friday's own words: *"no capture script has a beat where the player gets a kill, so I have no PNG of the X... **this is the single thing I would most want checked.**"* If Bevy UI silently no-ops `Transform.rotation` on a UI node, the kill confirm degrades to a colour flash and nothing warns you. |

---

# BAND 6 — TIER 3 AND BEYOND (real, open, not now)

Listed so nothing disappears. Do not start these while Bands 1-5 are open.

| Task | Rows | Lane | Note |
|---|---|---|---|
| Finish deleting Cliffhold | `TRV-0039` | **both — coordinate or the `match` breaks** | Client half went in `4152240`. `sim.rs` still holds **49** references including `build_cliffhold` and five reachability tests. Salvage first: the +25% scale trap, the flight-joint bug, and the reachability-test shape are all reusable — and the +25% trap in particular is written up in the root `FRIDAY_LOG.md` and is worth keeping. |
| The three core maps: +10%, real elevation, randomised structures | `TRV-0041`, `0042` | both | **The trap, and hand it to whoever builds:** "randomised" must be seeded at map-BUILD time from the match seed. Drawing from the gameplay RNG stream shifts every later number and breaks replay for every other system. Also fix the two lying centrepieces. |
| Bot navigation, properly | `TRV-0043` | friday22 | BOT ROUTING landed for Cliffhold with published up-links and a probe; `sim.rs:27649` states the flat maps were deliberately left alone. Whether `waypoint` is still a bare `[f32; 2]` is `UNVERIFIED` — re-derive before scoping. |
| Ragdoll + hit-reaction impulse | `TRV-0046`, `0106` | both | The rig's mass, length and inertia are complete and tested. `derived_spring_k` is the only consumer. §B.5 names both of these as the payoff for the column. |
| Soldier finger animation | `TRV-0044` | friday33 | The hand exists and poses from one `curl`. Driving it from weapon, reload, melee and grip is the remaining half. `afbe9d2` already built the instrument that can check it. |
| Weapon crafting station + the Forge per-piece grid | `TRV-0045`, `0076`, `0136`, `0138`, `0139` | friday33 | Shares a front end. **Build both or neither.** |
| Character creation L0-L4 | `TRV-0181`, `0146` | friday33 | **`BACKLOG.md` #9's stated blocker is known false.** It says "no class system and only 5 whole-body armour presets". Four classes shipped 2026-08-05; 24 per-piece plates shipped 2026-08-07. This row has been unblocked for five days and nobody noticed. |
| Mech control scheme | `TRV-0047` | friday33 | Worth doing now that jump, crouch and the cockpit exist to be controlled. |
| §16/§17 projectile origin audit | `TRV-0060` | friday33 | All five projectile types, both views, never checked together. Cheap, and it is the shared root of several "feels wrong" reports. |
| §32 first/third-person consistency pass | `TRV-0065` | friday33 | Three separate instances of that defect were found in ONE session. |
| §19 HUD redesign | `TRV-0061` | friday33 | **Deliberately last.** It must unify readouts still being added — TASK 6 and TASK 7 both add elements. Redesigning a growing HUD means doing it twice. |
| Injury / fatigue / dynamic CoG | `TRV-0183` | friday22 | **`BACKLOG.md` #11's blocker is known false** — the armour-weight formula was wired in the 24-plate pass. |
| Write the glTF loader for `jk_tdm` | `TRV-0048`, `0262` | friday33 | The blocker is **named and it is not the owner**: the shipping crate cannot load a mesh if pointed at one. Only `jk_bevy` can. This is the actual task behind "your uploaded gun assets". |
| Close or feed `motion-architecture` | `TRV-0188`, `0189`, `0190` | — | 5 of 14 core sources read; `DECISION.md`, the entire deliverable, never written. **Leaving it at 5/14 forever is the worst of the three options.** Either write the one-page decision from what was read, or close the thread explicitly. |
| Write `MAP_METRICS.md` | `TRV-0149` | — | Real cross-engine numbers are already extracted and READ in `research/maps/SOURCES.md`. Two rows (`TRV-0051` traversal, `TRV-0180` ledge bands) are blocked on a file that exists as research and not as a deliverable. |

---

# BAND 7 — BLOCKED. DO NOT START.

Each has a named unblocker that is not difficulty and not effort.

| Row | Blocked on |
|---|---|
| `TRV-0050` Networking | A networking dependency, and a decision to have one. The deterministic sim and bit-identical replay are the right foundation; nothing else exists. The scoreboard deliberately omits a Ping column for this reason. |
| `TRV-0051` Traversal | `MAP_METRICS.md` (see Band 6). |
| `TRV-0125` Grenades in water | No water volume exists in any map. |
| `TRV-0185` Mud / sand / snow / ice grenade surfaces | None of these exists as a `CoverKind` in any map. **Do not research the coefficients now** — they go stale before the blocker clears. |
| `TRV-0182` Destruction | rapier supports some of it; there is still no *design reason*, which is the honest blocker. |
| `TRV-0150` Powered armour | Research-only by your instruction, and Rule 13 retired the tier that would write it. Recording it so the want does not vanish: you said you want this **in the future**. |
| `TRV-0163` In-game console + image import | Blocked upstream on the texture pipeline. **The console CORE — cvar registry, autocomplete, history, scrollback — is blocked by nothing** and could ship alone if you want it. |

---

# LANE SHEETS

## friday33 — `main.rs` + client modules (the busy lane)

In order: **TASK 10** (capture scripts as data — pays for everything
below) → **TASK 1** (arrow launcher) → **TASK 6** (armour states visible)
→ **TASK 7** (boarding beats) → **TASK 8** (explosion + placeholders) →
**TASK 9** (the six honesty fixes) → **TASK 11** (the four owed captures)
→ **TASK 2** (Agile) → **TASK 3** (launcher) → **TASK 4** (Royal body) →
**TASK 5** (opposition body + the luminance guard test).

*Note where I would have re-ordered and did not: I would run TASK 10 and
TASK 8 before TASK 1, because they are cheap and they unblock the
evidence for everything else. Your order puts P2 first, so TASK 1 stands
at the top of the priority list. Both readings are in the file; pick one.*

## friday22 — `sim.rs`

`TRV-0026` (the audio side of the explosion, with `gen_sfx.py`) ·
`TRV-0055` (split the `gatling_heat` scale at the source) ·
`TRV-0235` (plate wear fires only on the zoned hitscan path — grenades,
melee, claws and gas neither wear plate nor are reduced by it; one gate,
`in_mech`, for both, so they cannot drift) ·
`TRV-0241` (the player/bot asymmetry in `punched_aim_stabilised` — Friday
calls it "the real root", and it applies to every recoiling weapon in the
game) ·
`TRV-0242` (a bot-piloted chassis now never raises its barrier at all) ·
`TRV-0043` (bot navigation) · `TRV-0039` (the Cliffhold sim half —
coordinate with friday33).

## thor — verify these, they are claimed and thin

`TRV-0008` (recoil envelope, no capture) · `TRV-0005` (slot strip, no
capture) · `TRV-0054` (does the rig key on `chassis_kneeling()` or raw
`f.crouch`?) · `TRV-0035` (**the barrier test copies its alpha and span
constants into the test body — if it is vacuous, `TRV-0207`'s evidence is
vacuous too**) · `TRV-0057` (`visor_ready` inside a `Local` — does the
camera cut actually happen?) · `TRV-0237` (did `fe07c19` close the
`mech_visor_eye_y` free-function bug?) · `TRV-0252`, `TRV-0253` (turret
spinners and world minigun spin) · `TRV-0250` (re-check the FP aim test
still cannot pass by coincidence now a third chassis exists) ·
`TRV-0134` (mutation-prove `armor_weight_movement_penalty` is wired).

## toto — only two rows qualify under Rule 13

Rule 13: `toto*` only when a specific unknown **NUMBER** blocks a build,
and it must be named in the dispatch.

- `TRV-0010` — the Royal arrow launcher's rate of fire, bolt velocity and
  per-bolt damage against a mech hull. Nothing in the game fires a bolt
  from a mount, so there is no neighbouring value to interpolate from.
- `TRV-0126` — "how enclosed is this point". The **method** is the unknown,
  not a coefficient. Ask for the method (ray fan? nearest-wall distance?
  cell occupancy?) and its cost at 120 Hz. Do not ask for a percentage.

Everything else that looks like a research need is a decision (Band 0) or
a build.

---

# THE THINGS NOBODY IS TRACKING

Not tasks. Facts about the record that will cost a session if nobody says
them out loud.

1. **`BACKLOG.md` has four entries that are known false**: melee depth
   ("Not started" — shipped `a99af96`), AI retreat ("the remainder" —
   shipped `e5431a4`), character creation's blocker, and the
   armour-weight formula's "unwired". Index it; never rank from it.
2. **`GAME_STATUS_REPORT.md` still lists Pyro** among the five armour
   sets, and names the 20-segment rig, the 26-piece armour and the 4-class
   system as unbuilt. All four are wrong now. It is dated 2026-08-01 and
   reads as current.
3. **`DESIGN_MAP.md` maps the wrong game.** `PROMPT_MASTER` §Read-first
   sends every new session to it as "what is actually built versus
   specified" — and every row in it is about `jk_wall`, the shieldwall
   battle sim, not `jk_tdm`. A session following the prompt's reading
   order gets a confident, detailed, irrelevant answer.
4. **There are two `FRIDAY_LOG.md` files.** The root one holds three
   entries and stops; `research/FRIDAY_LOG.md` holds the full history.
   They do not contradict each other — they are disjoint — but a reader
   who opens the root one gets 3 of ~20 entries and no signal that more
   exist. The root one contains the CLIFFHOLD entry and the +25% scale
   trap, which are not in the other.
5. **`handback/brief-ix/REPORT.md`** is an honest snapshot of 2026-07-30
   and a misleading document today: it says the class system, the 26-piece
   armour and the damage states do not exist "in any form". All three
   shipped.
6. **The `§owner` doc-comment convention is the best archival practice in
   this repo.** 44 chat asks in the ledger survive *only* because someone
   wrote `§owner` next to the code. Keep doing it. The one failure mode
   to watch: `TRV-0196` files *"put more effort in last 3 maps"* against a
   camera beat rather than against the maps, so the ask is preserved but
   pointed at the wrong thing.
