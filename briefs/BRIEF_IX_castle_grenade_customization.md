# BRIEF IX — Castle Maps · Grenade Dynamics · Character Customization

Three specifications following the pattern set by Briefs VII–VIII-B: grounded in design
principle, implementable via data-driven config, with explicit test gates and capture
requirements.

Companion documents:
- `BRIEF_VIII_master.md` — motion doctrine, first-person, HUD, camera, mech, Forge
- `BRIEF_VIII_B_addendum.md` — 20-segment body, elastic load model, mech visual spec
- `PROMPT_brief_X_research.md` — the research-and-build prompt that extends these

---

# BRIEF IX-A — Castle Map Design Specification

## Operating contract

**Map non-negotiables:**

1. **Multiple distinct highgrounds** — minimum three elevation tiers with different
   strategic values (snipe tier, crossfire tier, objective tier). No flat symmetrical layouts.
2. **Vertical traversal is costly** — climbing, mantling, or parkour gains position but
   expends sprint energy or makes noise. Height is only free if you hold it.
3. **Sightlines branch, not cross** — two players cannot see each other across more than 40 m
   unobstructed. Long sightlines must have cover breaks.
4. **Corner fighting dominates edges** — 70% of the map perimeter has a height change or hard
   cover within 2 m. Exposed flat ground is a kill zone.
5. **Objectives anchor vertical space** — the primary objective sits at middle height, neither
   highest nor lowest. Teams fight UP or DOWN to reach it, not left-right.
6. **Environmental cover types layer** — walls, rubble, vegetation and architectural detail
   each provide different silhouette breaks. Solid geometry only; no invisible walls.
7. **Exit routes from high ground exist** — no dead-end peaks. A player on high ground has at
   least two departure routes (one climbed, one dropped or vaulted).

## Architecture: three elevation tiers

### Tier 1 — Ground floor (0.0 m reference)
- Open courtyards, rubble scatter, collapsed stonework
- Tactical purpose: entry points, objective staging, exposed vulnerability
- Cover: low walls (0.5–1.2 m), debris piles, foundation ruin
- Escape: narrow alleys with vertical exit stairs, crumbled sections offering climbing holds

### Tier 2 — Wall / tower walk (4.0–6.0 m above ground)
- Rampart sections, tower exteriors, archer's balconies, scaffolding
- Tactical purpose: crossfire position, route denial, suppression angle
- Cover: merlon gaps, tower corners, scaffold framework, hanging ivy breaking silhouettes
- Access: stairs (wide, slow), ramps (faster, exposed), climbing routes (hidden but costly)
- Constraint: three architectural interruptions (collapsed spans, closed gates) preventing
  straight-line runs along the full circuit

### Tier 3 — Tower summit (12.0–14.0 m above ground)
- Castle tower roofs, keep peaks, catapult platforms
- Tactical purpose: sniper position, team rally, high-ground fortress
- Cover: crenellations (1.2 m tall), tower cap (8 m diameter), central well (unsafe to cross)
- Access: single stair spine from Tier 2, alternative ladder route (slow, exposed, dangerous
  if suppressed)
- Risk: isolated. Descent is commitment; lingering invites grenades.

## Sightline design rules

### The 40 m unobstructed rule
No two player positions can see each other across more than 40 m of open space without cover.
This forces engagements into the 25–35 m band where weapon balance matters and movement skill
applies.

Implementation:
- At 40 m, place a cover break (rubble cluster 2–3 m wide, wall corner, building wing).
- Use architectural detail (balcony overhang, scaffold strut, hanging fabric) to fragment
  diagonal sightlines.
- Sight-blocking geometry must be within arm's reach — climbable or vaultable — not just
  visual noise.

### The crossfire anchor
One central objective or choke point is visible from Tier 2 LEFT and Tier 2 RIGHT
simultaneously, but only at angles more than 60° apart. Teams cannot see each other directly;
each sees only the contested zone. This creates the crossfire pit where ground-floor players
fight under fire from two directions.

## Environmental detail and visual complexity

### Three-material layering
- **Primary** — hewn stone, weathered blocks, mortar decay (warm tan `#8A7860` to cool shadow
  `#5A5847`)
- **Secondary** — metal: door hinges, rebar, weapon racks, chain, decorative coronet
  (gunmetal `#3A3A3A` to burnished `#5F5E52`)
- **Tertiary** — organic: moss, root cracks, ivy creep, scorch marks, blood stains, fabric
  flags (greens and grays woven into stone)

### Detail density by tier
- **Tier 1 (ground)** — high detail. Rubble per square metre, scattered shields and polearms,
  foundation cracks, root systems. Rewards close examination.
- **Tier 2 (wall)** — medium detail. Archer's notches, torch sconces, weathering, arranged
  garrison signals. Enough to break silhouettes; not maze-like.
- **Tier 3 (summit)** — low detail. Clean kill zone on roofs, central urn or monument,
  sky-facing surfaces uncluttered. Snipers value open sight.

### Vertical movement audio callouts
- Stair segments: stone worn smooth — footfalls soft and predictable
- Climbing routes: rough-hewn holds, moss grip — climbing sounds like stone shifting
- Vaults and mantles: iron railings — metallic ring on contact
- Drops: rubble crunch — unavoidable noise

## Objective placement and territory flow

### Primary: the Castle Heart (central courtyard, Tier 1.5 — 2 m above ground floor)
- Physical anchor: a stone well-house with four entryways (N/S/E/W)
- Capture zone: 6 m radius circle at the monument base
- Capture time: 12 s, requires holding together
- Tension: to attack, teams descend from Tier 2 into an exposed pit; to defend, teams hold
  Tier 2 firing downward — a natural defender advantage
- Twist: the well connects down to a Tier 0 sub-level — a hidden tunnel system offering a
  slower but invisible approach

### Secondary: the Gatehouse Signal (Tier 2 rampart wall, south face)
- Capture zone: 4 m diameter
- Capture time: 8 s (quick plant)
- Value: controls map visibility — if captured, fog-of-war shifts, denying minimap coverage of
  the western approach
- Risk: exposed position; the captor is silhouetted against sky

## Defensive asymmetry and spawn design

**Team A (defending)** spawns at the Tier 3 keep, holding high ground initially.
- Advantage: elevated sightlines, sniper positioning
- Disadvantage: attackers funnel through predictable ascent paths; overextending means a long
  drop

**Team B (attacking)** spawns at the Tier 0 perimeter, outside the castle proper.
- Advantage: multiple entry axes; may choose a quiet climb over open assault
- Disadvantage: must gain height under fire; the objective sits below defenders initially

**Inversion at 5:00 game time** — if Team B has not captured the objective, positions invert:
Team A must defend at Tier 1, Team B holds Tier 2. This forces late-game reshuffling and
prevents infinite stalemate.

## Grenade considerations in map geometry

Grenade dynamics (IX-B) constrain where key positions can sit:
- Merlon gaps are deliberately too narrow for grenades to pass — blocks grenade camping on
  summits
- The well mouth is deliberately wide enough for grenades to drop — defenders must expect
  ordnance from above
- Tower stairs have an alcove landing every 4 steps — grenades bunch at the bottom, usable as
  a barrier

Maps must account for grenade traversal when placing key positions.

---

# BRIEF IX-B — Grenade Dynamics Specification

## Operating contract

**Grenade non-negotiables:**

1. **Grenades open mechanically** — not instant detonation. Fuse time of 3–5 s from release to
   blast, and that fuse is visible and audible in-world.
2. **Opening sequences differ per type** — percussion detonates on impact (0.2 s delay), time
   grenades hiss for 4 s, gas grenades pop and spread over 2 s.
3. **Blast radius scales, falloff is smooth** — lethal radius 6 m, damage falling to 0 at 20 m.
   No hard edge cliffs.
4. **Grenades interact with geometry** — bounce predictably off stone (coefficient 0.40), stick
   to organic surfaces, roll in channels.
5. **Counter-play is spatial** — players can see grenades, hear the fuse, and move before
   detonation. No invisible arming, no instant blast.
6. **Environmental effects layer** — water reduces blast radius ~30%; enclosed spaces under 3 m
   wide add +20%; height changes alter air time.
7. **Visual and audio telegraph is mandatory** — a grenade must be readable on screen before
   detonation; the blast must be audible at 60 m.

## Grenade types and opening mechanics

### Type 1 — Percussion (impact-detonating)
- **Fuse mechanism:** striker pin plus spring-loaded hammer; impact above 2 m/s fires the cap
- **Opening sequence:** on impact the spring drives the hammer, the cap ignites the primary
  charge (0.15 s), the primary ignites the main charge (0.05 s). Total delay 0.2 s from impact.
- **Visual:** impact → bright spark and small puff → 0.1 s later, orange-yellow blast cloud
- **Audio:** CRACK on impact detonation, then blast wave audible past 20 m
- **Ballistics:** normal throw arc; loses roughly 10% effective range because it cannot roll
  past the target
- **Lethal radius:** 8 m

### Type 2 — Time (fuse-burning)
- **Fuse mechanism:** mechanical timer wound during arming; external fuse cord burns audibly
- **Opening sequence:** arming occurs on throw release — the fuse ignites then. It burns 4.0 s
  visibly (bright red coal at the top). At 4.0 s the internal detonator fires, 0.05 s to the
  main charge.
- **Visual:** immediately after throw a small red glow appears at the grenade top; at t=3.8 s
  it brightens; at t=4.0 s, flash and blast
- **Audio:** HISS on fuse ignition, audible from 30 m within 1.2 s of throw; at t=3.5 s the
  hiss intensifies into a crackle — the final warning; BOOM at t=4.0 s
- **Ballistics:** follows the throw arc to completion. Thrown into water at t=1.0 s it still
  detonates at t=4.0 s — the fuse is internal, not drowned.
- **Lethal radius:** 6 m — lower than percussion, but more predictable

### Type 3 — Gas (aerosol dissemination)
- **Fuse mechanism:** pressure-release cartridge; a mechanical fuze ignites the gas-generating
  charge at 2.0 s
- **Opening sequence:** t=0 to 2.0 s the grenade is an inert canister. At t=2.0 s the internal
  charge fires, rupturing the canister. Gas expands from the ruptured seams (t=2.1–2.5 s),
  reaching full effect radius at t=3.0 s.
- **Visual:** t=0–2.0 s the grenade falls and rolls as a solid object. At t=2.0 s a bright POP
  and visible seam rupture. t=2.1–3.0 s coloured aerosol streams out — yellow-green for
  choking, white-blue for smoke. By t=3.0 s a stable 8 m hemispherical cloud.
- **Audio:** HISS as pressure builds (t=1.0–2.0 s, audible 20 m). POP at rupture (t=2.0 s,
  audible 60 m). Then ambient hiss as gas spreads.
- **Ballistics:** the canister follows its arc until rupture at t=2.0 s, then gas drifts with
  wind (0.2 m/s wind effect, simulated)
- **Effect:** non-lethal. Choking gas deals 2 damage/s to unmasked players within 8 m. Smoke
  caps vision range at 3 m inside the cloud. The cloud persists 12 s after forming — 15 s of
  total effect.

## Blast physics and falloff

### Lethal falloff (percussion and time)

| Range | Damage | Effect on an 80 HP player |
|---|---|---|
| 0–2 m | 100% | Guaranteed kill |
| 2–6 m | 100%→50% | Linear falloff; 60 damage at 4 m (wounded) |
| 6–12 m | 50%→15% | 28 damage at 9 m (suppressed) |
| 12–20 m | 15%→0% | 12 damage at 15 m (tick) |
| 20 m+ | 0% | No effect |

### Gas radius
- Non-lethal damage zone: 0–8 m (2 DPS unmasked, 0 DPS masked)
- Vision-blocking zone: 0–8 m, vision range capped at 3 m
- Persistence: 12 s after full deployment — effective until t=15.0 s

## Environmental interaction

### Bounce and rolling

| Surface | Coefficient | Behaviour |
|---|---|---|
| Stone / concrete | 0.40 | Bounces at 40% of impact velocity, max bounce height 0.5 m. Enables bank shots around corners. |
| Wood / metal | 0.50 | Higher bounce; banks off railings to reach elevated targets. |
| Organic — cloth, flesh, sandbags | 0.05 | Sticks on contact. Does not bounce; detonates in place. |
| Water | 0.30 | Sinks at 0.3 m/s; blast radius reduced 30% as compression disrupts blast shaping. |

### Enclosed space amplification
- Rooms under 3 m wide: blast radius +20%, falloff steeper — reaches 0 at 18 m
- Rooms over 10 m wide: blast radius −10%, falloff gentler — reaches 0 at 24 m
- Open air: baseline falloff

### Height effects
- Thrown downward (Tier 2 → Tier 1): +0.15 s air time per 2 m of drop. Grenades fall slower
  than expected, giving defenders more reaction time.
- Thrown upward: −0.10 s air time per 2 m of climb. Grenades rise quickly, less reaction time.

## Counter-play

**Spotting** — grenades are visible in flight as solid objects with distinct colour and shape,
and audible via fuse cues. Players must move to avoid the blast. There are no invulnerability
frames.

**Suppression** — a thrower under active fire suffers +0.5 s ADS time before release (flinch).

**Environmental destruction** — grenades destroy weak scaffolding and hanging barriers,
opening new routes or closing choke points.

**Denial** — players can kick or melee a grenade away, moving it 3–5 m and resetting its bounce
trajectory. Works only within 1 m and only if the grenade is visible.

**Armour** — heavy-armour variants (see IX-C) take 20% less grenade damage from plating.

## Loadout and supply

**Grenades per loadout:** 2. One throw per grenade; no infinite mid-match supply.

| Class | Loadout |
|---|---|
| Assault | 1 percussion + 1 time |
| Scout | 2 time (rewards planned positioning) |
| Heavy | 1 gas + 1 percussion (area denial plus shock) |
| Support | 1 gas + 1 time (smoke cover for revives) |

**Resupply:** refreshes on objective capture or on team spawn.

---

# BRIEF IX-C — Character Customization Specification

## Operating contract

**Customization non-negotiables:**

1. **Loadout-based, not freeform** — players pick a preset class (Assault, Scout, Heavy,
   Support), then customize within that class by swapping armour, weapons and accessories. No
   mixing across classes.
2. **Stat customization is gated by weight** — more armour means slower movement. The trade-off
   is visible in the loadout preview before commit.
3. **Cosmetic customization is independent** — skins, colours, decals and paint do not affect
   stats. They must still preserve squint-test readability: the weapon stays identifiable at
   30 m in silhouette.
4. **Armour pieces are craftable via the Forge** — all 26 pieces can be independently swapped,
   painted, or damaged. Equipped pieces count toward the weight budget.
5. **Preset loadouts reduce friction** — 5 saved loadouts per class, quick-selected in the
   spawn lobby for instant re-equip on respawn.
6. **Visual identity persists** — team colour (shoulder stripe, helmet band, chest emblem) and
   one asymmetric anchor remain constant across every loadout. All loadouts of the same
   character must read as the same character at 30 m in black silhouette.

## Customization tiers

### Tier 1 — Class selection

| Class | Base movement | Default armour weight | Weight ceiling |
|---|---|---|---|
| Assault | 6.2 m/s | 22 kg | 25 kg |
| Scout | 6.8 m/s | 16 kg | 20 kg |
| Heavy | 5.0 m/s | 28 kg | 32 kg |
| Support | 5.8 m/s | 20 kg | 24 kg |

### Tier 2 — Armour customization

Piece weights, mapped to the 26-piece segment-mapped armour from Brief VIII-B:

| Piece | Weight | Count |
|---|---|---|
| helmet | 1.2 kg | ×1 |
| gorget | 0.8 kg | ×1 |
| cuirass_front | 4.0 kg | ×1 |
| cuirass_back | 3.0 kg | ×1 |
| fauld (segmented) | 2.5 kg | ×1 |
| pelvis_plate | 1.5 kg | ×1 |
| pauldron | 1.0 kg | ×2 |
| rerebrace | 1.2 kg | ×2 |
| vambrace | 0.9 kg | ×2 |
| gauntlet | 0.6 kg | ×2 |
| tasset | 1.5 kg | ×2 |
| cuisse | 2.0 kg | ×2 |
| poleyn | 0.8 kg | ×2 |
| greave | 1.5 kg | ×2 |
| sabaton | 0.7 kg | ×2 |

**Movement penalty:** for each 1 kg over the class ceiling, movement drops 0.15 m/s. At +4 kg
over, movement is penalized −0.60 m/s, offset by improved protection.

**Worked example.** An Assault player removes both gauntlets (−1.2 kg) and both rerebraces
(−2.4 kg): net −3.6 kg, dropping from 22 kg to 18.4 kg — under ceiling, no penalty. Payoff:
lighter gloves improve first-person hand tracking clarity and grenade throw arc, but the
unprotected hands now absorb hits that gauntlets would have taken.

### Tier 3 — Weapon selection

| Weapon | Weight |
|---|---|
| Rifle | 3.5 kg |
| Shotgun | 4.0 kg |
| Sniper | 4.2 kg |
| SMG | 2.8 kg |
| Pistol | 1.2 kg |
| Melee | 0.5 kg |

Weapon weight consumes the same budget as armour. An Assault with rifle plus shotgun secondary
(7.5 kg) leaves only 17.5 kg of its 25 kg ceiling for armour.

### Tier 4 — Cosmetic customization

- **Skin:** Base Human, Scarred Veteran, Cultist, Exo-Augmented
- **Palette:** primary (team colour, locked to shoulders and helmet stripe), secondary (accent
  on weapon and boot trim), tertiary (detail on fabric and linen underarmour)
- **Weapon paint:** polished, weathered, symbolic (clan rune or personal sigil), pristine
- **Decals:** team emblem (auto-placed on chest), personal sigil (left shoulder or shield),
  rank marker (vertical stripe on left arm)

## Armour damage states

| Threshold | HP | State | Visual | Mechanical |
|---|---|---|---|---|
| 100% | 80/80 | Fresh | Clean plate, no scratches | No bonus |
| 70% | 56/80 | Scuffed | Light surface scratches, edge dulling | +5% damage resistance |
| 40% | 32/80 | Cracked | Deep gouges, fracture lines, loose rivets | +10% resistance, piece tilts |
| 15% | 12/80 | Severed | Detached or hanging by strap | Detaches on next hit |

**Exposed segments:** when a piece detaches, its mapped body segment takes ×1.25 incoming
damage. Player max HP is unchanged — unarmoured segments are simply fragile. Pieces are
reattached in the Forge between matches.

## Forge integration

Between matches, in the spawn lobby:

1. **Loadout preview** — 3D turnaround showing every equipped piece, colour, paint and decal,
   at current damage state
2. **Piece repair** — 50 in-match points per piece; reverts to fresh visually and mechanically
3. **Repainting** — free, cosmetic only; takes effect on respawn
4. **Save/load** — 5 loadouts per class. Each save shows weight, movement penalty and damage
   state; loading re-equips everything instantly
5. **Class swap confirmation** — changing class discards unsaved customization, behind a
   warning prompt

## Preset loadouts

**Assault**
- *Aggressive* — full armour (25 kg, no penalty), rifle + pistol, polished, team colours prominent
- *Duelist* — reduced armour (20 kg, −0.75 m/s), rifle + sword, weathered, asymmetric left-shoulder shield
- *Fieldfare* — medium armour (22 kg, −0.45 m/s), shotgun + melee, pristine, religious sigil on chest

**Scout**
- *Vanguard* — light armour (18 kg, −0.30 m/s), sniper + pistol, polished, clean
- *Ghost* — ultra-light (16 kg, no penalty), SMG + pistol, weathered, minimal decals
- *Archer* — medium (19 kg, −0.15 m/s), sniper + melee, clan rune paint, asymmetric bow-case on back

**Heavy**
- *Tank* — full armour (32 kg, no penalty), cannon + melee, polished, overwhelming team colour
- *Reaper* — medium-heavy (28 kg, −0.60 m/s), shotgun + melee, weathered plus death rune
- *Sentinel* — full armour (32 kg, no penalty), rifle + shield secondary, pristine, largest profile

**Support**
- *Paladin* — full armour (24 kg, no penalty), rifle + revive beacon, polished plus religious sigil
- *Surgeon* — light armour (18 kg, −0.90 m/s), SMG + medkit, pristine, asymmetric red cross
- *Bombardier* — medium-heavy (26 kg, −0.90 m/s), shotgun + gas grenade resupply, symbolic

## Test gates

**Customization readability**
1. Load all 5 custom loadouts of one class
2. Render each turnaround (front/side/back)
3. Convert to solid black silhouette at 30 m
4. All silhouettes must read as the *same character* despite armour and weapon variation. If
   not, the asymmetric anchor is insufficient.

**Weight and movement**
1. Light loadout (16 kg, no penalty) → sprint speed equals class baseline
2. Heavy loadout (+4 kg over, −0.60 m/s) → sprint speed equals baseline minus 0.60
3. Acceleration 0–2.5 m/s: lighter loadout accelerates roughly 8% faster
4. Penalty must scale visibly and predictably with weight

**Damage state visibility**
1. Equip all 26 pieces, fresh
2. Apply progressive damage, photographing at 70%, 40% and 15% HP
3. Progression must be visually gradual, never sudden

**Armour coverage hierarchy**
1. Side-by-side silhouette: Scout *Ghost* (16 kg) versus Heavy *Tank* (32 kg)
2. Tank must read wider and more protected; Ghost must read lean and vulnerable

## Captures required

- Forge screen showing all 5 preset loadouts for one class
- Light versus heavy black silhouette side by side, both at 30 m
- Damage progression: four frames — fresh, scuffed, cracked, detached
- Same character with three different primary weapons, each readable at 30 m in silhouette
- Close-up of the asymmetric anchor, proving it is constant across all loadouts
