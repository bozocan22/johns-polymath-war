# VERTICAL BANDS — the altitude ladder, rescued from Cliffhold

**Written 2026-08-12, immediately before `MapKind::Cliffhold` and
`MapKind::Battlefield` were deleted** per `BRIEF_XIII` §1.

## Why this file exists

Cliffhold was **the only map in this game with real occupied altitude
bands** — not a flat field with tall walls, but height layers that were
each a *place*, reachable on foot, with things to do on them.
Battlefield was the only map at 400 m scale with towers to 36 m.

`BRIEF_XIII` deletes both, and in the same breath asks Castle Gardens to
provide *"ground level, rooftops, towers, elevated bridges, platforms,
floating structures, courtyard balconies, underground/covered routes"*
(§4) at a size that holds 16v16 mech combat (§2).

**So the maps go and the knowledge must not.** These numbers were
authored, played and tested. Re-deriving them from scratch on the Gardens
rebuild would cost a cycle and would probably land somewhere worse.

This is not a design document. It is a **recovered parts bin**, with the
reasoning attached to each part.

---

## 1. THE LADDER — Cliffhold's bands, in metres

Taken from the constants at `sim.rs` (`CH_*`), each with the doc comment
that justified it. Read the right-hand column; the *reason* is the part
worth keeping, not the number.

| Band | m | What stood there, in the author's own words |
|---|---|---|
| Ground | 0 | the city floor |
| `CH_ROOF_LOW` | **5** | *"the city's low roof course and the aqueduct deck"* |
| `CH_BENCH_EAST` | **6** | *"the mesa's eastern apron, the gentle side"* |
| `CH_PLAZA_TOP` | **7** | *"the muster plaza at the map origin — the KOTH hill and the contested chassis pad both crown it, so it has to be a real, climbable place"* |
| `CH_SHELF` | **12** | *"the middle shelf, west and east alike"* |
| `CH_PLATEAU` | **18** | *"the cliff top, the castle courtyard floor, and the top of every approach: Cliffhold's headline band"* |
| `CH_RAMPART` | **24** | *"the curtain wall-walk, six metres above its own courtyard"* |
| `CH_KEEP_TOP` | **32** | *"the keep parapet — the highest surface a man can stand on unaided"* |

Playable half-extent was `CLIFFHOLD_HALF_M = 300.0` — a 600 × 600 m
field. Battlefield was 400 × 400 m with corner watchtowers to 36 m.

### What the ladder is actually made of

Three observations that survive the maps and are worth more than the
figures:

1. **The gaps are not uniform, and that is the design.** 5 → 6 → 7 is a
   cluster (three ways onto roughly the same level from different
   approaches); 7 → 12 → 18 is the real climb; 18 → 24 → 32 is the
   castle stack. A ladder of evenly spaced floors reads as a car park.
2. **Every band was reachable on foot.** The doc comment on the map is
   explicit: bands exist *"rather than a flat map with tall walls, so
   rooftops, benches and parapets are places rather than scenery"* — and
   *"every band is also reachable on foot, because the map has to play
   today."* Built for flight that had not arrived, playable without it.
3. **The two capture rings were deliberately at DIFFERENT altitudes** —
   `CH_CHECKPOINT_CITY` on the city floor and `CH_CHECKPOINT_KEEP` 18 m
   up in the castle courtyard, and pointedly *not* mirrored across the
   origin the way the older maps' objectives are. The comment states the
   principle outright: **"a map about height whose two objectives sit at
   the same altitude would be lying."**

That third one is the single most transferable idea in this file. If
Castle Gardens is to be a vertical map, its objectives must not all sit
on the floor.

---

## 2. THE CONSTRAINTS ANY NEW LADDER MUST OBEY

These come from `MAP_METRICS.md` and the movement code, and they bind
Gardens exactly as they bound Cliffhold.

- **`STEP_UP = 0.55 m`** — how tall a ledge the legs climb. Above it a
  soldier is stopped dead by the tread.
- **A mech's step is `STEP_UP * chassis_scale()`** (`sim.rs:4213`), so
  the same lip that stops infantry is walked over by a walker. **This is
  a design tool, not a bug**: it is the cheapest way in the game to make
  a route mech-only or infantry-only, and BRIEF XIII §6 asks for exactly
  that separation.
- **The real ledge ceiling is `apex + STEP_UP`, not `apex`** — a box
  stops blocking you and becomes your floor in the same tick, so the
  soldier ceiling is **2.26 m**, not 1.71 m. Every band boundary in
  Cliffhold was drawn with this in mind; a Gardens ledge drawn at the
  naive number will be 0.55 m wrong. Derivation is in `MAP_METRICS.md`
  §4.
- **Band separation must be ≥ 2.26 m** to be a real gameplay separation.
  Below that, two "bands" are one surface with a step in it. Applying
  this to Cliffhold's own ladder collapses three of its eight bands into
  one — which is a finding about Cliffhold, and a warning for Gardens.
- **`STAIR_RISE_M = 0.5`**, under `STEP_UP` by design, so stairs are
  walkable rather than climbable. Cliffhold laid eighteen flights.
- **`BOT_CLIMB_LANE_M = 7.0`** — the width the bots' climb lane assumes.
  Note this is an *assumption*, not a measurement; `MAP_METRICS.md`
  flags the bot-aperture figure as ASSUMED and names the experiment that
  would replace it.

---

## 3. WHAT DIES WITH THE MAPS, AND SHOULD BE REBUILT ON GARDENS

- **The only 18-flight stair network in the game.** Cliffhold's
  `sim.rs` test asserted eighteen flights and that the links were
  published; that test dies with the map.
- **The only end-to-end bot-routing proof on a vertical map.** The bots
  reached the top band from the bottom. Gardens will need its own.
- **The only sightline floor test** — Battlefield's, asserting ≥ 85 m.
  Gardens at 16v16 scale will want something similar, and
  `MAP_METRICS.md` §6 argues the *global* 40 m rule was never satisfiable
  and should be replaced by a local objective-pair rule plus a
  distributional one. **That is still an open owner decision.**

---

## 4. RECOMMENDED STARTING LADDER FOR CASTLE GARDENS

**DERIVED, not measured — a starting point for a builder to argue with,
not a spec.** Labelled per this project's rule that MEASURED, DERIVED and
ASSUMED are never blended.

BRIEF XIII gives a 200 × 200 m bailey inside a larger field. Cliffhold's
ladder was authored for a 600 × 600 m map, so the vertical range should
compress while the *structure* — cluster, climb, stack — is kept:

| Band | Suggested m | Role, from §4 of the brief |
|---|---|---|
| Ground | 0 | gardens, plazas, village streets |
| Low roofs | **5** | village rooftops, garden walls — the infantry shortcut layer |
| Courtyard balconies | **8** | overlooking the open areas, one climb from the street |
| Bailey wall-walk | **14** | the 200 m bailey's own rampart, ringing the fight |
| **Central high ground** | **20** | the contested objective of §4, with the brief's required *multiple approaches* |
| Tower / keep tops | **28** | sniper and long-sightline positions |
| Floating platforms | **34+** | §3's suspended architecture, bridge-linked |

Every gap above is ≥ 2.26 m, so every band is a real separation rather
than a step. The 0 → 5 → 8 cluster reproduces Cliffhold's
multiple-approaches-to-one-level idea; 8 → 14 → 20 is the climb; 20 → 28
→ 34 is the castle stack.

**And the rule that matters more than any of these numbers:** put at
least one objective off the ground floor. A vertical map whose fights all
resolve at y = 0 is a flat map with scenery.
