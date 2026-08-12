# BRIEF XIII — CASTLE GARDENS: THE GALACTIC MEDIEVAL SHOWCASE MAP

**Issued by the owner, 2026-08-12**, with one reference image. Verbatim
from §1 onward.

> The final map should feel like a **large, cinematic galactic castle
> civilization** with gardens, villages, floating architecture, massive
> fortifications, protected futuristic spawn factories, open mech
> battlefields, and strategically important high ground.

---

## 0. WHAT I VERIFIED BEFORE WRITING THIS — four facts that change the work

### 0.1 16v16 IS ALREADY SUPPORTED. This is not a new feature.

`MAX_PER_TEAM = 16` (`sim.rs:54`) and `TdmSim::new` clamps
`cfg.per_team` to `1..=MAX_PER_TEAM` (`sim.rs:7542`). **A 16v16 match can
be configured today.** The brief's *"optimize the entire layout around
16v16 mech combat"* is therefore a MAP and PERFORMANCE problem, not a
netcode or roster problem.

**The real 16v16 risk is frame cost, and there is a number for it.**
`motion-architecture/DECISION.md` §9 sets the pose budget at **29.76 µs
per character** (60 fps → 16.667 ms, 10% of frame ÷ 56 bodies) and names
56 as today's shipped ceiling (16 fighters + `ZOMBIE_CAP` 40). 32 players
sits inside that count — but **32 MECHS do not cost what 32 infantry
cost**, and nothing has measured it. Build `posebench.rs` (spec already
written) before committing to 16v16, or the map ships and stutters.

### 0.2 SPAWN PROTECTION DOES NOT EXIST AS THE BRIEF DESCRIBES IT.

What exists is `SPAWN_PROTECT_S = 1.2` (`sim.rs:185`) — a **1.2-second
invulnerability timer** set on respawn (`sim.rs:7859`, `:8498`).

What the brief asks for is a **protected PLACE**: a room you cannot be
shot into, cannot be entered by enemies or mechs, that projectiles cannot
cross, with a holographic doorway that passes your own team. **None of
that exists.** This is a real `sim.rs` feature — geometry, a team test on
the volume, projectile rejection at the boundary, and a fire gate — plus
its client half.

**Owner amendment, same message:** *"spawn feature for every map and game
mode, it should be a must — expect it from the first time we play 4v4."*
So the protected spawn is not a Castle Gardens feature. **It is a global
system that every map and every mode must have, present from the very
first match.** That makes it the largest single item in this brief and
the one with the widest blast radius. (Reading noted: "expect from the
first time" is read as *present from the first match onward*, consistent
with §First 4v4 step 1, "Spawn in the protected futuristic factory." If
you meant the intro match should be EXEMPT, say so — it inverts that
step.)

### 0.3 The intro match is hardcoded to the WRONG MAP, and it is one line.

`frontend.rs:338`, inside `intro_match_config()`:

```rust
map: sim::MapKind::Arena,
```

with the comment *"Arena: the widest, flattest, most legible of the three
PvP maps. A first match should be readable, not clever."* The brief now
says the first 4v4 must always be Castle Gardens. **One line, and the
comment above it must change too or it becomes the next stale document.**

### 0.4 Removing two maps is 34 references, not a rewrite.

`Battlefield`: 12 in `sim.rs`, 4 in `main.rs`, 1 in `map_look.rs`.
`Cliffhold`: 13 in `sim.rs`, 2 in `main.rs`, 2 in `map_look.rs`.
`MapKind::ALL` (`sim.rs:993`) is the five-entry table everything else
reads, so the enum and that array are the spine of the change.

**Note what is being deleted.** Cliffhold is the only map with real
occupied altitude bands (0 / 5-6 / 11-12 / 18 / 24-25 / 32 m) and it is
600×600 m; Battlefield is 400×400 m with towers to 36 m. Both were built
for exactly the verticality and scale this brief now asks Castle Gardens
to provide. **The lesson they encode should be carried into Gardens
before they are deleted**, and `research/maps/MAP_METRICS.md` §4 already
holds the ledge-band arithmetic (soldier ceiling 2.26 m, band separation
≥2.26 m for a real gameplay separation). Do not throw away the numbers
with the maps.

### 0.5 THE REFERENCE IMAGE — written down, because pictures get lost

**The image is NOT in the repository.** Drop it in
`engine/crates/jk_tdm/handback/reference/maps/` as
`castle-gardens-ref.png`. Until then this description IS the reference —
the mech concept art was lost exactly this way and still blocks four
criteria (`TRV-0260`).

An illustrated three-quarter isometric island map, painted, cool palette:

- **A single island** ringed by turquoise sea, with rocky cliff edges
  dropping to the water and small breaking waves.
- **A colossal castle complex dominating the upper-left half** — dozens
  of slender towers with **conical blue roofs**, domes, pale cream and
  grey stone walls, arched windows, banners. It reads as one enormous
  continuous structure, not a cluster of buildings.
- **A curving causeway/bridge system** in pale stone sweeping from the
  castle around a central bay, with balustrades and arches. Bridges are
  the connective tissue of the whole composition.
- **A glowing circular feature to the right** — a sunken ring of
  luminous cyan water inside a dark rock rim, clearly magical/energetic.
  This is the closest thing in the image to the "central elevated zone".
- **Formal gardens** — hedge geometry, small blue-domed pavilions and
  fountains, tree clusters in deep green, terraced onto the slope.
- **A separate smaller castle on its own island, upper right**, joined by
  bridge — the "floating/detached architecture" idea in embryo.
- **A working harbour at the lower left**: wooden docks, tall ships,
  crates, warm brown timber — the one warm, humble, lived-in corner
  against all the cool stone.
- Scattered rock outcrops and small islets in the surrounding water.

**What to take from it:** the *silhouette density* (many slender vertical
towers against open ground), the **blue-roof-and-pale-stone palette**,
bridges as primary circulation, one glowing focal feature, and the
contrast between grand architecture and a small humble quarter. **What
not to take:** it is an illustration at world scale, not a playspace —
its towers are decorative and its causeways are one figure wide. Every
one of them must be re-cut to mech width per §Mech Accessibility.

### 0.6 Lane and sequencing

This brief spans **both builder lanes**, so it is not one task:
`sim.rs` owns map geometry, spawn volumes and the fire gate; `main.rs`
and client modules own the look, the holographic barrier and the HUD
side. Split every section and say which half goes first. Run `balance`
before dispatching anything parallel.

---

## 1. CORE MAP RULES

- **Completely remove Battlefield and Cliffhold from the game.** They
  must not appear in any menu, map selector, customization screen,
  game-mode selection, rotation, matchmaking list, or other UI. Do not
  reference them as selectable or hidden/locked maps anywhere.
- The **first 4v4 match must always take place in Castle Gardens.**
- Design Castle Gardens as the primary showcase map, large enough to
  support **16v16 mech combat** without feeling cramped.

## 2. CASTLE GARDENS — OVERALL SCALE

- **Castle Bailey approximately 200 m × 200 m.**
- The playable map extends well beyond the central bailey, with room for
  large mechs, infantry, long-range engagements, close quarters,
  flanking, vertical movement and open-field battles.
- Keep a mixture of **huge open areas, enclosed spaces, elevated areas,
  narrow routes and vertical structures**.

## 3. GALACTIC MEDIEVAL VILLAGE

Keep the current visual direction and graphics quality, but push toward a
**galactic medieval village**. The central town should feel enormous and
dominated by a **massive castle complex**. Mix traditional castle
architecture with futuristic elements:

very tall buildings · large towers · massive castle walls · futuristic
structures integrated into medieval architecture · suspended/floating
sections of the castle · floating platforms and architecture in the air ·
bridges connecting elevated structures · futuristic lighting and subtle
holographic elements · dense village areas around the castle · large
courtyards and plazas · some areas ancient, others distinctly futuristic.

> It should feel like a **huge civilization built around a futuristic
> castle**, not a normal medieval village with sci-fi decorations added
> afterward.

## 4. BATTLEFIELD LAYOUT

**Open areas** — large gardens and courtyards, wide enough for multiple
mechs to manoeuvre at once, long sightlines but not completely exposed,
cover distributed throughout.

**Central high-ground zone** — a visually important elevated/fortified
zone with a tactical advantage and strong visibility, **multiple
approaches so it cannot become unbeatable**, large enough for mech
combat, naturally the major contested objective.

**Vertical combat** — ground level, rooftops, towers, elevated bridges,
platforms, floating structures, courtyard balconies, underground/covered
routes. **Make verticality useful without making the map confusing.**

## 5. SPAWN AREAS — GLOBAL, PER §0.2

Each team's spawn is a **small futuristic factory/shelter** integrated
naturally into the world.

**Spawn protection.** Inside your team's spawn you cannot shoot, cannot
be shot, cannot take damage; combat projectiles cannot enter; enemy
players and mechs cannot enter; mechs cannot enter the interior at all.
**Reliable and impossible to exploit.**

**Spawn design.** Visually appealing and slightly **cute/friendly**,
contrasting with the battlefield's intensity. Contains a loadout station,
ammunition refill, health recovery, equipment/customization access, and
simple intuitive interaction points. **Loadout changes must be extremely
quick** — no complicated menus.

**Spawn entrance.** A **transparent holographic doorway/barrier**: the
owning team passes through normally, it visually reads as protected, it
feels futuristic rather than like a conventional force field, and enemies
cannot enter.

**Spawn placement.** Not beside the fighting. The entrance should be
about **6–7 seconds of walking** from the main combat zone, so players
can orient, spawn camping is hard, and the spawn feels like a real base.
**Avoid direct sightlines from the combat area into the spawn entrance.**

*(Note: 6–7 s at `MOVE_SPEED` is a distance a builder can compute exactly
rather than eyeball — derive it and put the number in the map data.)*

## 6. MECH ACCESSIBILITY

Designed **for mechs from the beginning**, not adapted from an
infantry-sized map. Doors and corridors large enough; bridges support
mech movement; open spaces allow turning and manoeuvring; buildings give
meaningful cover; vertical areas account for mech height; **mechs cannot
become permanently trapped in architecture**; smaller infantry routes
exist alongside larger mech routes. Supports both 16v16 and 4v4.

## 7. FIRST 4v4 EXPERIENCE

Castle Gardens, automatically. The choreography:

1. Spawn in the protected futuristic factory.
2. Leave through the holographic entrance.
3. Move through the village/gardens.
4. Encounter the first open combat area.
5. Progress toward the central elevated zone.
6. Experience both vertical and open-field combat.
7. The castle visually dominates the entire experience.

> It must immediately communicate: **"This is a huge futuristic castle
> world designed for mech warfare."**

## 8. VISUAL DIRECTION

Use the reference image as guidance. **Do not completely redesign the
current art direction.** Preserve what works; increase the scale
dramatically; improve environmental storytelling; make the architecture
more impressive and the castle enormous; add futuristic/floating elements
carefully; maintain readability during combat; keep it beautiful without
filling every area with unnecessary detail.

---

## PROOF STANDARD

Rule 8: every claim here is visual, and this map cannot be signed off
from a diff.

- Castle Gardens from the air, showing the 200 m bailey and the extent
  beyond it.
- The seven beats of §7, in order, as a capture sequence.
- **A mech and an infantryman in the same frame** at a door, a bridge and
  a corridor — §6 is unprovable otherwise.
- The spawn: interior, the holographic doorway from both sides, and a
  projectile failing to cross it.
- **The 6-7 second walk, timed** — a capture with the clock visible at
  the entrance and at the combat edge, not an assertion.
- A 16v16 frame with a frame-time readout, per §0.1.

Rule 12: mutation-prove every test. A map test that reads the same
constant the map generator reads cannot fail.

Rule 9: "feels bad" is often dead code. Before tuning any existing
Gardens number, check it is read at all.
