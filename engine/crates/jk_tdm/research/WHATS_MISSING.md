# What is missing / not built yet — 2026-08-07 (rev 5)

Compiled from `BACKLOG.md`, `THOR_LOG.md`'s ranked findings, and session
knowledge. Ordering inside each tier is Thor's ranking rule: an item
moves up when its blocker clears, not when it becomes interesting.

## 0. Needs the USER (nothing code-side can proceed)

Nothing. The three branding PNGs landed 2026-08-04 (key art, wordmark,
emblem) and are wired through the splash, menus, and seal footers.

## 0-SPEC15. THE MECH SYSTEM SPEC (owner, 2026-08-09) — SUPERSEDES THE QUEUE

A 15-section spec landed. It is the current priority; 0-QUEUE below is
still valid but ranks behind it. Owner's own priority order.

### P1 — ARCHITECTURE (do first, everything else sits on it)
- **Remove PYRO ARMOUR completely.** [S]+[C] Models, inventories,
  training, UI, menus, spawn logic, equipment logic, references, dead
  assets. Already known: no pad spawns it, yet TWO per-map relocation
  tables still place a Pyro pad, and `BIND_REGISTRY` still sells its
  flame ability. Grep the whole tree.
- **Training Mode: ONE fixed scenario.** [C] No settings menu, no rules
  config, no setup screen. Entering it starts the scenario. Hardcode the
  ruleset rather than exposing it.
- **Six mech variants registered**: player Agile/Big/Royal and
  opposition Agile/Big/Royal. [S]
- Inventory/equipment stability preserved.

### P2 — GAMEPLAY
- **Shield in inventory as `Shield [4]`**, an interactive slot showing
  quantity. [C]
- **Grenade on G reuses the BOW/SPEAR architecture**: inventory → equip
  → HOLD IN HAND → aim → release → throwable physics → decrement stock.
  A held item, not an instant projectile. Grenade-specific physics. [S]+[C]
  (Note: the G/LMB fuse work already landed; this is the HELD-IN-HAND
  and inventory-decrement half.)
- **Big Mech recoil: controlled for 1-2 s, then progressively chaotic.**
  Straight/heavy/absorbed first, then rising instability. Must read as
  intentional, not random. [S] numbers + [C] visual.
- **Royal tier ~10% larger and ~10% stronger** than Big. [S]
- **Royal ARROW LAUNCHER: minigun + 3 crossbows.** Compact minigun
  silhouette, rotating mechanism, three crossbow assemblies around a
  central weapon, bolt ammunition, mechanical loading. [C]+[S]

### P3 — VISUAL
- **Agile Mech major upgrade** — the largest visual item. Must read
  FASTER, LIGHTER, more advanced, more mechanically detailed, and
  clearly distinct from Big and Royal *in silhouette*. Plates, joints,
  legs, hydraulics, torso, shoulders, head/cockpit, weapon mounting,
  small components, surfaces, energy details. [C]
- **Rocket Launcher redesign** — chambers, mounting, reload mechanism,
  barrel detail, materials, VFX, firing animation, recoil. [C]
- **Royal Mech: its own body and silhouette, NOT a scaled Big Mech.** [C]
  ~~SUBTLE neon-blue energy accents~~ — **SUPERSEDED, owner 2026-08-10.**
  The Royal shipped with GOLD accents instead of the neon-blue this line
  asked for. Trevor raised it as Band-0 question Q4; the owner's answer
  was *"keep the royal mech"*. **The gold stays. Do not rework it, do not
  re-open it.** The body-and-silhouette half of this line is UNTOUCHED by
  that ruling and is still open — it is Task 4, and `main.rs:11414` still
  says "Same machine, same 53 plates".
- **Opposition mechs are NOT recolours.** Own armour design, body
  structure, silhouette, mechanical detail, weapon styling — while
  keeping the faction colour language: neon red, dark red, neon blue,
  dark blue. Neon stays recognisable inside darker materials. [C]
- **Opposition Royal**: unique body. **Palette set by owner 2026-08-10:
  RED, YELLOW, and NEON-BLUE details.** [C] This supersedes the earlier
  "neon-red/dark-red primary with dark-blue complements". Red remains the
  primary; yellow and neon-blue are detail accents, not coatings.
  **Must still not read as a recoloured player Royal** — and note that
  ruling now has teeth, because the player Royal is gold and this one
  carries yellow. The two must be told apart by BODY and SILHOUETTE, not
  by colour. If a capture shows them reading alike at 30 m in black, the
  fix is the chassis, not the palette.

### P4 — POLISH
Animations, VFX, weapon feedback, materials, lighting, UI consistency,
performance.

### CODE QUALITY (from the spec, and they match this project's rules)
Shared systems over duplicated mech logic; faction visuals kept as
DATA; training config isolated; no new settings; compile clean; fix
warnings caused by the change; test every new interaction.

### TRAPS THIS PROJECT ALREADY KNOWS — hand these to whoever builds
1. **A third chassis tier is a DATA question.** The heavy already paints
   from a three-tone table (`mech_body_tones`) and the medic has a
   four-level trim table. Royal and the opposition variants belong in
   those tables, not in forked spawn functions. The spec's own "do not
   duplicate mech logic" says the same thing.
2. **`armor_set` is an enum the sim branches on in many places.** Adding
   tiers means auditing every `== ArmorSet::RobotSuit` — that exact
   pattern already caused "the medic pilot failed every mech gate".
   Prefer `in_mech()` / `in_heavy_mech()` predicates.
3. **Silhouette beats paint.** A variant identifiable only by colour
   does not exist at range; that is why the Royal got a crown. Each new
   variant needs one silhouette element.
4. **The luminance rule is unguarded.** Ally/enemy separation rests on
   value, not hue, and the enemy's light blue is ALREADY brighter than
   the ally's brightest tone — only part coverage saves it. Neon on
   dark is safe; neon over large areas is not.
5. **Capture everything.** Five defects this week were invisible to the
   compiler and to 400 tests, and obvious in a screenshot.

## 0-QUEUE. SMALLEST TO BIGGEST (2026-08-09)

The owner asked for the work split by SIZE. Everything below is real and
still open. Sizes are honest estimates of one agent-session each unless
marked. Lane in brackets: [S] = sim.rs, [C] = main.rs/client, [D] = docs.

### TIER 0 — MINUTES. One line each, no design decisions.
1. `ROBOT_SPEED_MULT` (1.12) is dead and states the OPPOSITE of the live
   0.85 beside it. Delete it. [S]
2. `MECH_SHIELD_ARC_COS` is named, documented, and unread — the barrier's
   real arc test uses a bare `cos > 0.5` literal 40 lines away. Point one
   at the other. [S]
3. `TDM_TARGET_CHOICES` is unread; the menu hand-types `30`/`60`. [C]
4. `pod_aim_held` is written every tick and read nowhere. Delete or wire. [S]
5. `shot_handgun.wav` is generated and loaded by nothing. [D]
6. `SCOUT_SCALE`'s doc says "slimmer and SHORTER" for a chassis that is
   5% TALLER than a man. [S]
7. `FORGE_SLOTS` is unread; both slot UIs hardcode the list. [C]
8. Export a `pub const` for the `9000.0` spray scale — the client copies
   it today and guards the copy with a test. [S]

### TIER 1 — AN HOUR. One clear fix, one test.
9.  **Every explosion is silent.** The sim publishes `Boom`; no sound
    exists. Unblocker is `gen_sfx.py`, already in the repo. [C]
10. **The Field Manual quotes the DEFAULT score target, not the one you
    chose** — two screens on one pause menu disagree. [C]
11. **Hull-climbing `U` is missing from the "full bind list."** [C]
12. **The medic's mid-air jump and second flip are undocumented** —
    two movement options a pilot will never discover. Put them on the
    Controls screen, NOT the equip hint (that line already overflows). [C]
13. `gait_pose` bakes the INFANTRY crouch ratio (0.646) against the
    sim's `MECH_CROUCH_HEIGHT_FRAC` (0.72) — kneeling, the x2.0 visor
    weak point lands on rendered neck and shoulder. [C]
14. The barrier test is half vacuous: its alpha and span constants are
    copied INTO the test body, so mutating the real material or the real
    disc size leaves the suite green. [C]

### TIER 2 — A SESSION. Real work, one system each.
15. **Armour damage states are invisible.** Four stages ship and are
    tested; `armor_stage_of` has ZERO client readers, so Fresh, Scuffed
    and Cracked render identically. Only Severed shows, and only because
    it removes the piece. [C]
16. **Mech boarding: 8 tested sim stages, 7 render a `debug!` line no
    player sees.** The largest built-but-invisible system in the game.
    The strings already exist verbatim inside the debug calls. [C]
17. **Pyro armour is unobtainable on every map** while the controls
    screen still sells its flame ability — and TWO per-map relocation
    tables still place a Pyro pad that never spawns. [S]
18. **Delete Cliffhold.** Half-done and reverted. Salvage first: the
    +25% scale trap, the flight-joint bug, and the reachability-test
    shape are all reusable. [S] + [C], coordinate or the `match` breaks.
19. **Make capture scripts DATA, not code** — Thor's highest-leverage
    finding. Beats are compile-time constants in a 28k-line file, which
    is WHY every framing tweak costs a 6-minute rebuild. ~40 s after. [C]

### TIER 3 — MULTI-SESSION. Design decisions inside.
20. **The three core maps: +10% larger, real elevation, high ground,
    randomised structures.** The trap: "randomised" must be seeded at
    map-BUILD time from the match seed. Drawing from the gameplay RNG
    stream shifts every later number and breaks replay for every other
    system. Also fix the two lying centrepieces (Bailey's keep is
    argument-for-argument Dust Arena's tower; Gardens' gazebo is a solid
    block). [S] + art [C]
21. **Bot navigation, properly.** Waypoints are 2D with no height and no
    reachability check. Bots cannot choose to climb. [S]
22. **Soldier finger ANIMATION.** The hand poses from one `curl` that is
    still a spawn-time argument. [C]
23. **Weapon crafting station** — shares a front end with the Forge
    per-piece grid. Build both or neither. [C]
24. **Ragdoll + hit-reaction impulse.** The rig's mass/inertia data is
    complete, tested, and read by nothing. [S] + [C]
25. **Mech control scheme** — worth doing now that jump, crouch and the
    cockpit exist to be controlled. [C]

### TIER 4 — BLOCKED BY A NAMED THING, not by difficulty.
26. **Your uploaded gun assets: `jk_tdm` HAS NO glTF LOADER.** Only
    `jk_bevy` does. A GLB in `assets/characters/` changes nothing in the
    shipping game. Writing that loader is the actual task.
27. **Texture pipeline** — procedural textures exist now (Cliffhold rock,
    metal, wood); no IMPORTED image is used as a world texture, and the
    older maps' cover materials are still flat.
28. Networking (zero deps), traversal (blocked on map metrics), full
    character customisation.

### A DECISION FOR THE OWNER, not a task
29. **Mech first-person aim sits 1.10 m above the muzzle** — camera at
    the visor (2.72 m), weapon fires from `EYE_REL` (1.62 m). A hull
    turret genuinely IS a metre below the visor, so this may be correct
    and merely unstated. Changing it moves every mech engagement, hit
    test, cover line and tracer in the game.

## 0-NOW. THE LIST, REBUILT AFTER THE SIX-AGENT SESSION (2026-08-08)

Everything below is what is ACTUALLY left. Sections 0a and 0-SCOUT
underneath are kept as history; where they disagree with this, this
wins. Suite is 358 tests at commit `8482933`.

### A. DEBTS FROM THIS SESSION — do these first, they are known-broken

1. **The jump telegraph has no client reader.** During Compress and
   Recover the sim kneels the hitbox (`height()` drops 0.85 m, every hit
   band shrinks) while the rendered chassis stands upright. ~0.95 s of
   model/hitbox disagreement per jump, and the first-person camera DOES
   follow, so the pilot's view sinks for no visible reason. Key the rig
   on `chassis_kneeling()` + `mech_jump_compression_of()`, not raw
   `f.crouch`.
2. **`gait_pose` bakes the INFANTRY crouch ratio** (0.646) against the
   sim's `MECH_CROUCH_HEIGHT_FRAC` (0.72), which appears nowhere in
   main.rs. Kneeling, the x2.0 visor weak point lands on rendered neck
   and shoulder. Same class as the bug the crouch ban was holding shut.
3. **Armour damage states have no client half.** `armor_stage_of` and
   `armor_wear_of` are published and read by NOTHING; plates render at
   one appearance whatever their condition. The whole feature is
   invisible.
4. **The barrier test is half vacuous** — its alpha and span constants
   are copied into the test body, so mutating the real material or the
   real disc size leaves the suite green.
5. **Capture verification owed** for four fixes that landed unphotographed:
   the per-owner turret spinners, the world minigun spin, the kneeling
   crouch drop, and the grenade throw.
6. **`gatling_heat` carries TWO scales in one sim field** — 0..100 for
   the heavy's gatling, 0..1 for the medic's plasma. `main.rs:20167`
   prints the raw value under a `%`, correct for one and 100x wrong for
   the other if that branch is ever shared. Split it or document it.
7. **DECISION NEEDED, not a bug: mech first-person aim is 1.10 m off.**
   The camera sits at the visor (pos+2.723 m); every mech weapon fires
   from `muzzle_origin` (pos+1.62 m), because `EYE_REL.min(height-0.12)`
   only bites for a SHORT fighter. A hull turret genuinely IS a metre
   below the visor, so this may be correct and merely unstated. Changing
   it would move every mech engagement, hit test, cover line and tracer.
   The owner should choose.

### B. PROMISES THE GAME MAKES AND DOES NOT KEEP

8. **Mech boarding is 8 named, tested sim stages and the client renders
   SEVEN of them as a `debug!` line no player sees.** The largest
   built-but-invisible system in the game.
9. **`SCOUT_SCALE = 1.42` is read by nothing** — the Mechanical Medic
   renders man-sized despite a constant documented as the reason it
   "reads as a different silhouette from across a map".
10. **CASTLE BAILEY's keep is byte-identical to DUST ARENA's centre
    tower** (same helper, same two arguments), and **CASTLE GARDENS'
    "stone gazebo" is a solid 8x8x2.4 m block** you can neither enter
    nor shoot through. The rest of both maps is real.
11. **`visor_ready` promises a camera that does not exist** — it is a
    Bevy `Local`, so nothing outside its own system CAN read it.
12. **Audio placeholders**: plasma, repair beam, barrier and precision
    charge all play `shot_mp5`; every boarding stage plays `click`.
    Unblocker is `engine/assets/audio/gen_sfx.py`, not the owner.

### C. SPEC SECTIONS STILL OPEN

13. §16/§17 projectile origin audit — all five projectile types, both
    views, never checked together.
14. §19 HUD redesign — deliberately last; it must unify readouts that
    are still being added.
15. §4 soldier finger ANIMATION — the hand poses from one `curl`, which
    is still a spawn-time argument. Driving it from weapon, reload,
    melee and grip is the remaining half.
16. §24 crafting station — shares a front end with the Forge per-piece
    grid; build together or not at all.
17. §31 mech controls — only worth doing now that jump, crouch and the
    cockpit exist to be controlled.
18. §29 throwing consistency, §28 arc-attached distance readout.
19. §11 turret recoil: muzzle climb and the hull answering the shot (the
    camera-kick half is live and tested).
20. §32 a real first/third-person consistency pass. Three separate
    instances of that defect were found in ONE session.

### D. LARGE / BLOCKED

21. Ragdoll + hit-reaction impulse — the rig's mass/inertia data is
    complete, tested, and read by nothing.
22. Texture pipeline — every world surface is flat colour.
23. **§23 uploaded gun assets — the blocker is now NAMED and it is not
    the owner.** `jk_tdm`, the crate the game launches from, has NO
    glTF loader; only `jk_bevy` does. A GLB dropped in
    `engine/assets/characters/` changes nothing in the shipping game.
24. Networking, traversal, full character customization.

### DEAD CONSTANTS (cheap, do while passing)
`ROBOT_SPEED_MULT` (1.12, contradicts the live 0.85 beside it),
`MECH_SHIELD_ARC_COS`, `TDM_TARGET_CHOICES`, `FORGE_SLOTS`, and
`shot_handgun.wav` which is on disk and never loaded.

## 0-SCOUT. WHAT TWO READ-ONLY SCOUTS FOUND (2026-08-08)

**This file was stale AGAIN, for the second time.** A gap scout checked
section 0a against the code rather than against the commit messages and
found six items listed as open that had already shipped. That is the
recurring failure of this document: it is written by whoever last built
something, and they always know less than the code does. **Treat every
line here as a claim to re-check, never as truth.**

### Corrected — listed as open, actually DONE
- TIER 2 #6 mech jump and crouch, #5 rocket launcher detail, #7 green
  plasma flash — all built.
- TIER 1 #3 grenade pre-aim — the max-range indicator and the metre
  readout both exist in the status line. Only the ARC-ATTACHED distance
  is missing.
- TIER 1 #4 turret recoil — the camera-kick half is live and tested;
  muzzle climb and the hull answering the shot are what remain.
- TIER 3 #14 castle map — mis-stated in BOTH directions. Bailey and
  Gardens are NOT arena blockouts: they have their own extents,
  ramparts, cross-walls, drum towers, hedge rings, terraces. What IS
  arena blockout is exactly the centrepiece (see below).
- Per-piece armour GEOMETRY — the 24 plate groups exist and visibility
  follows the loadout. The backlog's "stripping a gauntlet changes
  nothing" is no longer true.

### NEW, ranked by how likely a player is to notice

1. **Mech boarding is 8 named stages and 7 of them render nothing.**
   The sim runs a tested 8-stage timer; the client answers seven with a
   `debug!` line the player never sees. This is the single biggest
   built-but-invisible system in the game. (BACKLOG #16.)
2. **`SCOUT_SCALE = 1.42` is declared and read by NOTHING** — the
   Mechanical Medic renders at scale 1.0, i.e. man-sized, despite a
   constant whose own doc says it should read as a different silhouette
   from across a map. Sim and client agree, so there is no hitbox bug;
   the machine is simply the size of a man.
3. **CASTLE BAILEY's "keep" is byte-identical to DUST ARENA's centre
   tower** — the same helper called with the same two arguments. And
   **CASTLE GARDENS' "stone gazebo" is a solid 8x8x2.4 m block** you
   can neither enter nor shoot through.
4. **`visor_ready` promises a camera that does not exist.** It is a
   Bevy `Local`, so nothing outside its own system CAN read it.
5. **`ROBOT_SPEED_MULT = 1.12` is dead and contradicts the live 0.85**
   sitting beside it — a trap for the next editor.
6. **`engine/assets/characters/` sharpens the asset blocker.** Its
   README specifies GLB characters and says `cargo run -p jk_bevy`.
   `jk_tdm` — the crate the game actually launches from — has no glTF
   loader at all. So §23 is not "the owner points at the assets": the
   shipping crate cannot load a mesh if pointed at one.
7. Dead constants duplicated as bare literals: `MECH_SHIELD_ARC_COS`,
   `TDM_TARGET_CHOICES`. Behaviour is correct; the names are unused.
8. `shot_handgun.wav` is on disk and never loaded.

### From the DEFECT scout — regressions introduced this session
1. **Every mech's hull turret spins from the PLAYER's trigger.** One
   rate computed from `fighters[player]` is written to every
   `MechTurretSpinner`, which now tags both the viewmodel and a
   per-fighter hull node. On foot, nothing spins at all. This is the
   §32 defect the fix was written to kill, relocated.
2. **The carried minigun's WORLD model never spins** — the spinner is
   only tagged when `with_hands` is true, which is the viewmodel flag.
3. **§21's jump telegraph has no client reader**, and during Compress
   and Recover the hitbox kneels while the rendered chassis stands
   upright — ~0.95 s of model/hitbox disagreement per jump.
4. **The renderer re-derives mech crouch depth from INFANTRY
   constants** (0.646 vs the sim's 0.72), so kneeling, the x2.0 visor
   weak point lands on rendered neck and shoulder.
5. **Third-person `crouch_drop` is double-counted for a kneeling mech.**
6. `Fighter::pod_aim_held` is written every tick and read nowhere.

Both scouts also confirmed what is NOT a gap, which is worth as much:
the light chassis refusing to crouch, the medic having no power core,
the armour-pip HUD excluding the medic, and 8v8/Extraction being
withdrawn from the menu are all DELIBERATE and documented at the site.

## 0a. THE ORDER OF WORK — everything left, ranked

Ranking rule, as always: an item moves up when its blocker clears, not
when it becomes interesting. Within a tier, cheapest-per-unit-of-felt-
difference first. Spec section numbers in brackets.

### TIER 1 — feel. Cheap, and the player notices immediately.

1. **First-person aiming [§7].** The one Priority-1 item still open.
   The spec says aiming is "too difficult"; the spear turned out to be a
   silent 2x halving, so look for the same class of thing here before
   tuning anything: crosshair-to-muzzle alignment, whether the aim ray
   and the drawn barrel agree, recoil recovery rate.
2. **Projectile origin audit [§16/§17].** Partly done historically (mech
   fire used to leave screen centre). Nothing has ever audited ALL of
   bullets / rockets / arrows / spears / grenades in both views at once.
   Cheap, and it is the shared root of several "feels wrong" reports.
3. **Grenade pre-aim [§28].** The arc preview and the 55 m/2.3 s curve
   both exist now; what is missing is the max-range indicator and the
   distance readout on the arc itself.
4. **Turret recoil feel [§11].** The numbers landed with the fire modes.
   This is the presentation half: camera kick, muzzle climb, and the
   hull answering the shot.

### TIER 2 — visible content, self-contained, one session each.

5. **Rocket launcher visual detail [§14].** The mount is a housing and a
   tube; the spec wants targeting electronics, a loading mechanism and
   data/power hardware. Same additive method as the turret module ring.
6. **Mech jump and crouch [§21].** SIM-side. Wants compression, launch,
   landing impact and recovery, plus a real crouch that lowers the hull.
   Note `set_crouch` currently REFUSES to crouch a mech by design - that
   rule has to be revisited deliberately, not deleted.
7. **Green plasma hit effect [§25].** Build the positive half only. The
   "x-ray" this section asks to remove has no identified target in the
   code (see 0b) - do not delete something on the strength of a guess.
8. **Spacecraft cockpit [§20].** First-person mech frame, instrument
   panels, screen glow, vibration. The `hands` capture's boom scale is
   the instrument that makes this checkable.
9. **HUD redesign [§19].** Consistent type, spacing, panels and
   hierarchy across everything above. Deliberately AFTER the readouts
   it has to unify - redesigning a HUD that is still growing elements
   means doing it twice.

### TIER 3 — large. Real multi-session work; do not rush.

10. **Soldier finger ANIMATION [§4, redirected].** The hand exists and
    poses from one `curl`. Driving that curl from the weapon, the
    reload, the melee swing and the grip is the remaining half.
11. **Armour damage states (Brief IX §C).** Per-piece HP so a worn plate
    goes Fresh → Scuffed → Cracked → Severed. Needs the hit path to
    resolve which PIECE it struck, not which zone. The biggest item
    outside the spec.
12. **Weapon crafting station [§24].** Category grid, attachments, stat
    comparison. Shares its front end with the Forge per-piece grid, so
    build them together or not at all.
13. **Mech control scheme [§31].** Only worth doing once jump, crouch
    and the cockpit exist to be controlled.
14. **Castle map.** Content/geometry. The intro already offers CASTLE
    BAILEY and CASTLE GARDENS, which today are arena-blockout layouts -
    the menu is writing a cheque the map does not cash.
15. **Ragdoll + hit-reaction impulse.** The 20-segment rig publishes
    mass, length and inertia that nothing reads yet.

### TIER 4 — blocked. Named unblocker, not difficulty.

16. **Uploaded gun assets [§23]** - no unused image or model assets are
    in the repo. Unblocker: the owner points at them.
17. **Texture pipeline** - every world surface is flat colour. Unblocks
    wear maps, decals, weapon material stacks [§12 of the backlog].
18. **Networking** - zero deps. The deterministic sim is the right
    foundation; nothing else exists.
19. **Traversal (climb/vault/mantle)** - blocked on map metrics.

### ONGOING, not a task
- **Audio placeholders**: plasma, repair beam, barrier deploy and the
  precision charge all borrow soldier sounds, marked at the call sites.
- **§5 emotion dynamics**: keep working, verify after each rig change.
- **§33/§34 polish and LOD**: continuous, not a milestone.

## 0b. THE 36-SECTION SPEC — status per section

The owner delivered a consolidated 36-section development spec. Several
of its premises turned out to be wrong about the current build, and
those corrections matter more than the ticks:

**Done and pushed**
- §6 bow orientation — it was not random. An earlier pass laid the bow
  HORIZONTAL on purpose to keep the upper limb out of the sight line.
  Standing it up exposed two instruments welded to that decision:
  `bow_string_half` could only aim in one plane, and the screen-
  intrusion sweep discarded `vm_carry`'s ROLL (the spear had been
  measured unrolled the whole time).
- §18 shield HUD — the shield was NOT missing; "SHIELD [4]" was already
  in the strip. Its STATE was missing. Only the mech barrier has the
  "current / max / recharging" the spec asks for; the soldier's plate
  is damage reduction with no pool, so it reports block % and arc
  instead of a fabricated 0/0.
- §10 rotating turret — never lost, only ever half-built: the spinner
  existed in the VIEWMODEL alone, so the pilot saw it turn and nobody
  else did. Plus the module ring (2 tracking, 2 cooling), all on the
  static housing.
- §32 (partial) — the turret was the first-person/third-person
  inconsistency the spec predicted.

**Could not be found**
- §25 "x-ray style" enemy hit effect. Searched translucency,
  depth-bias, render-layer and see-through material paths on fighters:
  the only translucent set in the codebase is the deliberate
  first-person guard plate. The pale blue discs near soldiers in
  captures look like spawn protection, not a hit effect. The GREEN
  PLASMA impact effect the section asks for is still worth building on
  its own merits; the removal half has no identified target.
- §23 uploaded gun assets — no unused image/model assets located in the
  repo. `engine/assets/` holds audio plus the three branding PNGs.

**Blocked on other work, not on difficulty**
- §30 firing-mode UI — needs the sim's fire-mode accessor to exist
  first. Wiring a display before then means inventing the state.

**Done since that audit** (see section 0a for what remains)
- §8/§9 spear, §12/§13 fire modes + recoil, §15 rocket arc,
  §26/§27 grenade, §30 fire-mode UI, §22 royal variant,
  §1 Agile Mech graphics pass, §2/§3 soldier hands + arm joints.
- Three of those were DEAD CODE rather than tuning: the spear's speed
  halving fired on every player throw while the preview drew the full
  arc; the grenade's fuse and wind-up shared one clock; the autocannon's
  kick was erased inside the tick it landed.

## 1. Small, one-session buildable (ranked, next up)

*(Audited 2026-08-07 against the code. Three items on this list had
already been built and were still sitting here as open — bot support
logic, per-piece armour geometry, and the second beam pool. A stale
"missing" list is worse than none: it sends the next session to rebuild
finished work. What follows is what is actually left.)*

1. **Armour damage STATES** (Brief IX §C). The 24 plates are
   equippable, visible, and a missing one exposes its segment — but a
   WORN one has no condition. The brief's four-stage table (Fresh →
   Scuffed → Cracked → Severed, with a plate detaching on the next hit)
   needs per-piece HP, which needs the hit path to resolve which *piece*
   it struck rather than which zone. A real extension of
   `apply_hit_dmg`, not a polish pass.
2. **HUD award toasts for a resource economy** — the toast system
   itself shipped (kills/assists/parries/streaks). §4.3's *resource*
   awards still have no economy in TDM/KOTH to award from, so this
   waits on a mode that has one rather than being faked.
3. **Audio for the new systems.** Plasma, the repair beam, the barrier
   deploying and the precision charge all use borrowed sounds from the
   soldier arsenal. They are placeholders and are marked as such at the
   call sites.

### Closed (the medic REDESIGN, and the shield put back)

| Item | How it closed |
|---|---|
| **A variant system nobody had ever looked at** | The four trims were captured as a LINEUP — four machines, one frame, one livery, one light, differing in nothing but plate — because reading the table only proves the code branches, not that the branches are worth having. It immediately paid for itself twice. The first attempt spaced them 3.1 m apart all on one side of the player and photographed one and a half machines: the third-person boom is locked behind the player, so the row has to STRADDLE him (now a named constant with the reason beside it). And the HEAVY trim's one unique plate, the gorget, sat at y=0.80 — inside a torso egg whose crown is 0.82. The heaviest trim's distinguishing feature was geometry nothing could ever see. It rings the neck now, in the gap between torso and head. |
| **The mech barrier had grown hardware nobody asked for** | A later pass bolted an outer frame, a six-node emitter ring and its feed conduits around the projector, on the argument that a field that size "needs a structure to be projected from". The owner asked for the original back: just the blue transparent hologram, at 60% bigger. All of that hardware is gone and the emitter is the three-petal folding module it always was. `BARRIER_SCALE` is 1.60 against the original 1.7 m field — 2.72 m. Note the arithmetic: the tree was already at 1.55×, so reading "60% bigger" against *that* would give 4.2 m, which the file's own test rejects as "a building, not a shield" (it brackets the barrier at 2.4–3.4 m). 2.72 sits mid-band. |
| **The barrier had never been photographed** | It shipped, was redesigned twice, and every claim about its appearance came from reading the code that spawned it — `shield_fp` captures the SOLDIER's tower shield, which is a different object. A `barrier` script now boards the heavy, raises the plate on the key a player actually presses, and orbits it: a flat disc seen edge-on is the one angle that proves nothing. |
| **The medic was still a military machine** | The owner brought reference art: a squat utility robot, rounded masses, one big camera lens, worn amber over near-black. The chassis was rebuilt to that language — ~90 exposed struts/pistons/louvres down to ~45 nameable masses. One LENS eye with the team accent as its iris, at visor height, so friend-or-foe is answered by the glance that meets its eye. Plantigrade thick legs and big feet replaced the digitigrade frame: it read fast but also skittish, and half its hardware (boom, calf thrusters, bracing stays) existed only to explain itself. The ally shell went AMBER — the team read never rested on the shell (it rests on luminance against a near-black foe, plus the emissive accent), so a service livery is affordable and truthful: it is equipment, not a soldier. |
| **Armour is now a trim level, not one fixed suit** | Four trims off one frame — STRIPPED / LIGHT / FIELD / HEAVY — on two deliberately crude dials, because a variant needing six numbers tuned is a variant nobody adds. `limb_scale` wraps thicker or thinner shell round the *same* black skeleton, so no trim can break the silhouette's proportions. `plates` is a COUNT, not a set: the optional pieces have a fixed order of value (knees → pauldrons → belly → gorget) and a trim wears the first N. Ordering it that way makes coverage monotonic in the trim, which is the one property a cheap test can pin and the one a set would let drift. Knees lead because a walker takes its hits from below; the gorget is last because the throat is the smallest target. Picked by slot index at spawn exactly as the helmet shapes are, so it cannot touch replay state. Mutation-proven: collapsing FIELD onto LIGHT's plate count fails the test and nothing else. |
| **Every mech wore the pilot's rifle** | `weapon_root` hangs off the SPINE, not an arm, so hiding the arms never hid it. Third-person only; the first-person half was fixed long ago. |
| **A medic pilot failed every mech gate** | `let in_mech = f.armor_set == ArmorSet::RobotSuit` — written before the second chassis existed, and never revisited. The stowed rifle stayed out, rolls played the soldier shoulder-roll under a rigid hull, and the legs aimed like hips. Replaced with the sim's own `f.in_mech()`. |
| **The medic wore the heavy's shin armour** | D.1 leg armour was gated on `is_mech`. Invisible while both liveries were grey; the amber repaint made it obvious in one frame. |
| **The pilot's hips showed through the chassis** | The chassis shares the soldier's thorax space and its pelvis was 0.30 m against a 0.345 m waist stripe, so the man's own body was visible as a pale band through the machine. Every block from the pelvis up now has a stated minimum width. |

### Closed 2026-08-07 (the medic pass — model, HUD, and three bugs)

| Item | How it closed |
|---|---|
| **The repair beam could not be selected at all** | `cmd.slot` was hardcoded `0 => Gatling, 1 => Rockets` — the HEAVY's mounts, applied to every chassis. In a medic, whose strip reads PLASMA BOW [1] / REPAIR BEAM [2], pressing 2 set ROCKETS and the validity guard reverted it the same tick, so the support half of the chassis was unreachable while the HUD advertised it by name and number. Split brain: the strip renders from `for_set` and this handler carried its own copy of the list. One source now, and the index comes from the same slice the HUD draws, so slot N *is* the mount labelled N. Found by a capture; every existing mount test passed throughout because they all assign `mech_weapon` directly and none press a key. |
| **The capture rig could not photograph a side** | A beat named `02-medic-side-on` had produced rear views for its whole life: `look` turns the PLAYER and the third-person boom is rigidly behind the player, so the subject rotates with the camera and no yaw yields a profile. `CapBeat.orbit` swings the boom around a stationary subject (and re-aims at the anchor, which the first attempt did not — it photographed the scenery beside the machine). Capture-only and inert in play. Also pinned: beat times must not run backwards, and every script's last beat must set `end`. |
| **The medic model read as a small heavy** | Four changes, each paid for by an angle the rig could not previously take. Head: a pale ball that read as a human skull in the soldiers' own white → a raked sensor pod, longer front-to-back than wide (the one proportion that separates a bird's head from a person's at any distance). Chest: one 46 cm plate sealing the front, so the exposed core showed only in profile → two halves with a chamfered slot. Shoulders: box caps → swept pauldrons with trailing fins. And a COUNTERWEIGHT BOOM behind the hips — the heavy is a vertical column with nothing behind it, so a boom separates the two machines as pure shape with all detail resolved away. |
| **Medic HUD had no support readout** | The target bracket frames the ally the beam would take, in green, from `Sim::repair_candidate` — the function the beam itself calls, made `pub` for that reason rather than letting the HUD run a lookalike search. Precision charge also renders at the crosshair now: both prior readouts sit in corners, and a pilot winding up a 95-damage shot is looking at centre screen. Colour snaps at full charge instead of fading, because the release point is a threshold. |
| **The beam could not appear in a capture** | It only takes an allied MECH under 85% hull, which a fresh match never contains. The boarding hook parks one hurt machine in reach — by hand, because a script that waits on combat to reach a state captures a different frame every run. |

### Closed 2026-08-07 (the art + second-chassis pass)

| Item | How it closed |
|---|---|
| **Heavy mech was a slab** | Hull worn at 85% as a uniform root scale (uniform because the hull is full of rotated cylinders and a non-uniform parent scale shears every one — and every hardpoint moves inboard for free). ~120 new parts buy the size back as MASS: armour layering with shadow gaps, pistons as rod-in-barrel, louvre runs, a five-layer reactor core with conduits leaving it, joint collars. Head went from a box with a slit to segmented armour + optic pods + sensor clusters + comms blades, with the ×2 visor slit untouched. Arms went from bare hardpoints to a pauldron→elbow→cradle chain. Legs got a knee cap, calf mass and an attitude thruster. |
| **Both hull mounts were placeholders** | The turret gained spinner-mounted clamps and muzzle collars, a STATIC drive housing (the drive does not spin, the thing it drives does — most of what makes a gatling read right), a heat-sink stack with a glowing seam, a belt box with gold feed links, and a partial cowl open below. The launcher gained a ribbed casing, mouth jaws, backblast vents, a feed arm mid-travel and a boxed seeker head. |
| **Mech barrier had no visual** | An arm-mounted folding emitter (three petals, 0.18 s deploy) projecting an 8.5%-alpha fill plus a bright hex LATTICE of real geometry. That split is the answer to a contradiction in the brief — "transparent to the pilot" and "visible to enemies" cannot both come from one translucent sheet. A test pins the gap between the two alphas. |
| **Maps were small** | Every map +25%, applied centrally: POSITIONS scale, extents and heights do not. Plus an infill pass (outer ring at three height bands, mid-field stepping stones, angled flank lanes) because a bigger map is a worse map empty. |
| **AGILE SUPPORT MECH** | A second `ArmorSet`, not a flag — a third of the hull, faster than a man on foot, plasma that never runs out but overheats (hard lockout), a repair beam that mends allied CHASSIS only, and a ×2.4 vulnerability to spears and arrows that gives those two weapons something to beat. Digitigrade ~60-part frame. Its own pads on the flanks. |
| **Repair beam was invisible** | The sim publishes `repair_target`; the client draws a segmented shaft plus travelling packets. Two layers because a glowing line says a connection exists and only moving packets say something is being CARRIED — and a beam aimed at a teammate that looks like a weapon makes people dodge their own medic. |

### Closed 2026-08-07

| Item | How it closed |
|---|---|
| **Intrusion budgets were audited, not measured** | `weapon_parts(kind)` lifted out of `spawn_weapon_model` — the geometry is data now, so `weapon_bounded_extent` measures the budgets instead of transcribing them. The minigun's barrel cluster became a `spin` flag on the part, which was the one thing welding the table to `Commands`. Measurement immediately disagreed with the audit: the M249's carry handle is 0.192, not the 0.085 the comment claimed. It also caught the BOW clipping the crosshair circle when the grenade coil stacked on the draw — fixed by making the poses exclusive and dropping the bow's carry to -0.22. |
| **Suppression had no HUD read** | The sim records the bearing of the last close round; the screen edge facing it lights pale gold. It reuses the DAMAGE flash's own strips rather than a new widget — suppression is the same information one step earlier, and a second directional element would teach the same idea twice. Red at 0.55 alpha, gold at 0.25, so a hit wins the strip on alpha alone with no priority rule. |
| **Squad retreat** | The last quarter of squad AI, and the only part needing MEMORY — every other squad behaviour is re-derived each tick. `fear` flows in from witnessed falls, suppression, being outnumbered and your own blood; bleeds out over time and faster once contact breaks; crosses a threshold with hysteresis. Modelled on `jk_wall`'s morale pass, minus its rolled `rout_tolerance` — that comes from CLASS instead, making nerve a fifth thing the four classes trade. A broken man holds fire, keeps facing the enemy, and reloads. The player is never routed. |
| **26-piece armour** (built as 24 — see below) | 24 plates, each mapped to a hit zone, each with the brief's weight. A bare segment takes ×1.25; total weight past the class ceiling costs 0.15 m/s per kg, which finally WIRES `armor_weight_movement_penalty` after it sat pure and unreachable. Its own Forge page, four rows by body region, with a live weight-against-ceiling readout. **The brief's own table sums to 24, not the 26 its title claims** — built to the table, recorded rather than reconciled by inventing two plates. |
| **20-segment mass-bearing rig** | The data half (§B.3 mass, §B.4 length, §B.5 inertia) plus the three missing segment groups: a real **lumbar** between pelvis and thorax (the twist is shared 38/62 now, not landed on one hinge), **clavicle bones** the arms actually hang from (the girdle spring existed but only nudged an IK target), and **toes** — so the sprint pushes off something instead of gliding. All five of §B.6's tests are live, including the mass-closure one that catches the brief's own trap: the clavicles are carved FROM the thorax, not added beside it. Spring stiffness is `I·ω²` off the mass model rather than hand-guessed. |

### Closed 2026-08-06

| Item | How it closed |
|---|---|
| **Helmets were a colour, not a shape** | Five helmet SHAPES (FIELD CAP, VISOR, CREST, HOOD, HORNS) as a data table of primitive pieces, orthogonal to the four tints — 20 heads. Bots derive theirs from slot INDEX so a firefight has five silhouettes and replays stay bit-identical. Index 0 frozen byte-for-byte to the old cap; saved profiles in the pre-helmet four-field format still load. Three invariants tested, each a real failure the geometry can have. |
| **Third-person bow anchor** | The restage, and it turned out to be three bugs rather than one: the draw hand rode 14 cm ABOVE a now-horizontal string, the string was ONE STATIC BOX that never moved at all, and the nocked arrow hung off the BOW hand so it tracked a hand instead of a bow. The nock is a function now and the halves, the arrow and the hand all derive from it. The viewmodel gained a nocked arrow it never had. Verified in `bow_draw_fp`. |
| **AI suppression** | Produced by GEOMETRY — every round banks suppression on every enemy it passes within 1.15 m of, measured against the segment the tracer actually drew. Bots pay in accuracy (a multiplier on their own sigma, never a floor, so a pinned Hard bot still out-shoots a free Easy one) and stop walking into the fire. The player pays in a viewmodel shake and nothing else: the round still leaves the camera ray where the crosshair is. |
| **AI bounding overwatch** | A second axis, not folded into `SquadRole` — anchor/flank is WHERE, this is WHEN. One half moves and holds fire, the other plants and shoots; they swap on a function of the sim clock and index rank, so no phase is stored and every replay test survives. A lone bot is `Free`, and so is a room fight. Three of its six tests cover what a determinism test cannot see: a deadlock reproduces perfectly. |
| **Per-weapon screen-intrusion profiles** | `spear_raised` and `bow_drawn` built, and the sweep taught to see them: it had been running ONE hard-coded root and `carry_offset` alone, so no weapon's own placement and no pose shift had ever been checked. `bow_drawn` deliberately supersedes the brief's vertical-bow wording — restated, not dropped. |

### Closed 2026-08-05

| Item | How it closed |
|---|---|
| Viewmodel drew over every menu | The vm camera renders after MainCam (order 1, no clear). New `vm_rendered` predicate gates on the same `hud_visible` the HUD uses. Proven by a capture that pauses from live first-person play — the old `menus` script could not reach the case. |
| Pilot held his stowed rifle in a mech | Two hull-mount viewmodels: a spinning gatling cluster and a launch tube. Punch rides the mount's own cycle via `shot_clock`; rifle reload/sight/scope poses gated out in a chassis. |
| RMB zoomed while pre-aiming rockets | `pod_aim_owns_rmb` gate in `input_and_step`. Also dropped an accidental `ADS_SPEED_MULT` slow that Y-targeting never paid. |
| Guard plate blinded its own holder | FP-only translucent material set; third-person and enemy shields stay opaque. |
| Shield bound to E | Now inventory slot 4, an essential beside the three guns. E is dead. |
| Mech HUD garbled / overlapping | Bottom-left is chassis vitals only; the mounts own the bottom-right. Fixed a real argument swap — POWER printed the turret belt and AMMO printed the energy core. |
| Turntable showed a placeholder mannequin | `spawn_soldier_body` extracted from `spawn_fighter_rigs`, so the card and the field share one geometry source. Verified a pure move: all 183 geometry constants byte-identical. |
| Mech read as a plain slab | Detail pass — visor brow/cheeks, spine fins, exhaust stacks, pauldron trim and bolts, waist pistons, gatling clamp rings and feed chute, drum ribs, recessed pod face, two-tone paint, stencils, knee hazard, toe teeth. Detach parts became group nodes so a damage stage sheds a whole cluster. |
| Mech hull-climbing | Full 7-item checklist: zone grabs on stripped plates, position parenting, asymmetric grip drain, involuntary detach, 1.6x climbing strike on both melee paths, 4 tests. |
| Mech inventory / rocket controls | Strip shows TURRET/ROCKETS only while piloting; RMB pre-aims, LMB fires through `try_fire_rocket`. |
| Killfeed WALLBANG modifier | Bullet penetration shipped (`PEN_WINDOW_M`, 0.5x damage), killfeed marks it `#`. |
| Melee parry + stagger | Parry window on both the axe arc-sweep and knife/thrust line paths; attacker takes `PARRY_STAGGER_S`. |

### Closed 2026-08-05 (rev 2)

| Item | How it closed |
|---|---|
| Grip HUD + climb prompt | Ten-segment GRIP bar in the vitals block while hanging; "U - GRAB THE HULL" / "U - LET GO", and a distinct line when the pool is too spent. The attach search moved out of the step into `climb_target` so the prompt asks the same question the verb answers. |
| Mech mount audio | `own_shot` read `fire_cd`, which never moves in a chassis - a piloted mount fired silent for its own pilot. Now on `shot_clock`, one sound per mount, and the "same weapon" guard understands a mount swap. |
| Iron sights on every gun | Two guns had NO rear sight at all (shotgun, M249); five declared the front-post centre as their sight line instead of the aperture centre. Then all of it was superseded: every firearm now carries a **1x red-dot optic with an illuminated red cross**, the AWM's cross living in its scope overlay. |
| M249 "grey wall" | Its sight line sat 2 mm above a flat feed cover, so aiming laid a 30 cm plate across the eye; it also shared the rifle carry offset despite a receiver 2.1x the AK's cross-section. Own carry, raised sights, arched carry handle. |
| Recoil correctness | The permanent camera-pitch channel ignored two rules the sim obeys - projectile weapons produce no punch (a drawn bow was walking the aim up the screen) and a scoped rifle's punch is scaled 25/78. The M249's kick equalled the AK's while cycling 40% faster. |
| Arrow + spear models | Both were one featureless box. Now a bodkin head, tapered shaft, nock and three spinning fletching vanes; a leaf blade, collar, haft and butt spike. |
| Mech fire came from the screen centre | The ray still leaves the eye (that is the hit test), but the STREAK is drawn from the muzzle actually on screen. Rockets ease off the tube onto their true position over 90 ms without touching physics. |
| The HUD was drawn OVER by the weapon | The interface rendered on MainCam (order 0) while the viewmodel camera is order 1 with no clear, so the gun composited on top - it ate the first characters of the mech ammo readout. The UI has its own Camera2d at order 2 now; nothing 3D can draw over it again. |
| **AI squad coordination** | Bots read teammates for the first time. Focus fire (a target a squadmate is on scores as closer, bounded so it cannot out-weigh a man at point blank), anchor/flank roles by index order, and spacing so a squad stops arriving as one clump. All rng-free so replays stay bit-identical - there is a test asserting a full 4v4 runs identically twice. *(Suppression and bounding overwatch, open at the time, both closed 2026-08-06 — see below.)* |
| **Melee depth v2** | Directional attacks: strafe picks the swing's LINE (left / right / overhead) at the wind, latched for the whole strike. A parry only meets a blade on the same line, so a knife fight is a read rather than a timing check. Both viewmodel and third-person silhouettes cock to the chosen line, because the defender has to be able to SEE what to answer. |
| **4-class system** | LINE / SKIRMISHER / WARDEN / MARKSMAN, each hooked to health, movement, spread and swap speed, with a per-class silhouette and a Forge picker. LINE is 1.0 across the board; a test proves the other three each trade. |

## 2. Large, multi-session architectural (do not rush — BACKLOG.md)

Ranked by value, blockers first:

- ~~**26-piece armour**~~ — **DONE 2026-08-07** as 24 plates (the
  brief's table, not its title). What remains of §C is the damage-STATE
  table and per-piece geometry; both are listed in tier 1 above.
- ~~**20-segment mass-bearing rig**~~ — **DONE 2026-08-07**. All 20
  segments exist as real transforms with published mass/length/inertia
  behind them. The remaining rig work is *consuming* the data further:
  ragdoll and hit-reaction impulse are the two §B.5 names as payoffs
  that nothing reads yet.
- **Castle map**: content work (geometry), not code. The intro's CASTLE
  BAILEY / CASTLE GARDENS entries select layouts of the existing arena
  blockout, not a real castle.
- **Forge editor UI**: the turntable and SAVE/LOAD/RANDOMIZE rows are
  built; the specced per-piece category grid is not (and is really the
  front end of the 26-piece armour item above).
- **Mech weapon-kit D.7**: 20 named swappable parts with part-by-part
  damage states. The silhouette, the core kit, and three damage stages
  are built; per-part swapping is not.
- **Traversal** (#7): climb/vault/mantle — blocked on map metrics.
  Hull climbing proved the attach/parent/stamina mechanic.
- **Full character customization** (§8.1): sliders and per-piece
  cosmetic variants. The HELMET half landed 2026-08-06 (5 shapes × 4
  tints, a real part library rather than a colour swap); body/face
  sliders and weapon cosmetics are still 0 of 0.

## 3. Blocked, with the named unblocker

| System | Blocker |
|---|---|
| Weapon material stack, wear maps, decals, image import | Zero texture pipeline beyond branding UI images — every world surface is flat colour. Unblocker: mesh/material texture loading. |
| Advanced rendering | Depends on the above. |
| **Networking** (rollback/prediction/lag comp) | Zero networking deps; local-only. The deterministic sim + bit-identical replay is the right foundation, but no netcode exists. Scoreboard deliberately has no Ping column for this reason. |
| Swimming / ropes / ladders / fluids | No water volumes, no ropes, no muscle layer. |
| Grenade extra materials (mud/sand/ice…) | No such surface exists in any map. |

## What is NOT missing (commonly assumed otherwise)

- Mech presentation §A–§C: **done** (brace, entry/exit, idle-life, visor
  view, gatling + autocannon, and now first-person mounts).
- Mech silhouette D.1–D.6: **done** (43-plate hull, leg armour, three
  detach stages, detail pass).
- Hull climbing: **done and tested**.
- Assists, K/A/D/DMG scoreboard, death→killer-cam→spectate: **done**.
- Paged intro flow with branding + real-rig turntable: **done**.
- Determinism/replay guarantee incl. bot mechs: **done and tested**.
- Capture coverage for first-person mounts, the guard plate, and the
  pause menu: **done** (`mech_fp`, `shield_fp`).
