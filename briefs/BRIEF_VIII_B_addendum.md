# BRIEF VIII-B — ADDENDUM & CORRECTIONS
## The 20-Segment Body · Achilles Elastic Motion · The Mech, Read From The Concept Art

**How to use:** paste this AFTER Brief VIII v2 in the same Claude Code session,
or paste both together. This file **patches** Brief VIII v2 — where the two
disagree, **this file wins**. Section E lists every edit it makes.

**Why this exists:** three problems in Brief VIII v2 that the concept art and
the biomechanics literature expose.

1. **The brief demands a motion the rig cannot physically perform.** §5 requires
   *hip-shoulder separation* (the throwing "X-factor"). A rig with one trunk
   segment **cannot** rotate hips and shoulders independently — the spec was
   impossible as written. Fixed in Section B.
2. **"Achilles motion" was treated as a metaphor.** It shouldn't be. The
   Achilles tendon is the actual mechanical reason human movement looks springy
   and explosive, and it produces implementable numbers. Section C.
3. **The mech spec does not describe the concept art.** The art shows a machine
   roughly **2.5× soldier height** in olive-drab military paint with a drum-fed
   autocannon and a gatling arm. The brief specifies **1.15×**, gunmetal gray,
   and a different weapon fit. Section A forces a decision; Section D rewrites
   the visual spec to match what is actually drawn.

---

## SECTION A — THE SCALE CONFLICT (decide before building the mech)

**Measured from the concept art:** with the soldier at 1.8m, the mech's hull top
sits at roughly **4.5m**, antenna tip near **5.2m**. The soldier's helmet reaches
only to the mech's **knee joint**. That is a ratio of **≈2.5×**, not the 1.7×
the brief assumed, and nowhere near the 1.15× it specifies.

At 1.15× (2.07m) the mech would stand barely a head taller than the soldier
beside it. That is a **fundamentally different machine** from the one drawn —
the art's entire read (walk *under* it, shoot *up* at it, take cover *behind its
legs*) disappears.

**Pick one, explicitly, and write the choice into `config/mech.ron`:**

| Option | Height | What you get | What it costs |
|---|---|---|---|
| **A1 — Honour the art** | **2.5× (4.5m)** | The concept art's actual presence. Soldiers shelter under it. Legs become cover. Reads as a vehicle, not a big man. | Needs its own nav width, doorway rules, and a separate cover system. Cannot use soldier corridors. |
| **A2 — The brief's compression** | **1.15× (2.07m)** | Shares all doors, nav, and cover with soldiers. Cheapest to integrate. | Looks nothing like the art. Do not ship the concept art as marketing for it. |
| **A3 — Split the difference (recommended)** | **1.7× (3.06m)** | Keeps the "look up at it" read and the leg-cover fantasy; still fits standard 3m+ doorways and most exterior nav. | Needs a widened nav radius but not a bespoke system. |

**Recommendation: A3 (1.7×).** It preserves what the art is actually selling —
that the machine towers — while staying inside one nav profile. If the mech is
meant to be a *map-scale threat* rather than a soldier upgrade, take A1 instead.

**Whichever is chosen, these scale WITH it (do not leave them at soldier values):**
footfall camera-shake radius (6m → ×scale), step-up height (0.4m → ×scale),
braced side-step distance (3m → ×scale), visor height, viewmodel offsets
(×scale, not ×1.15), and the interpenetration sweep's clearance margins.

---

## SECTION B — THE 20-SEGMENT HUMAN BODY ★ replaces any prior body/rig breakdown

Biomechanics has solved this. Standard research models the human as **14–16
rigid segments**, and every one has published mass, length, and inertia data.
Below is that standard model extended to **20 segments** — the extra four are
not decoration; each one exists because Brief VIII already demands a motion
that is impossible without it.

### B.1 The 20 segments

| # | Segment | Count | Why it exists |
|---|---|---|---|
| 1 | Head + neck | 1 | Head-look layer (§1.1) |
| 2 | **Upper trunk** (thorax/shoulders) | 1 | ← rotates with the shoulders |
| 3 | **Mid trunk** (lumbar/spine) | 1 | ← **the twist segment: this is what makes hip-shoulder separation possible** |
| 4 | **Lower trunk** (pelvis/hips) | 1 | ← rotates with the hips; drives the kinetic chain |
| 5–6 | **Clavicle / shoulder girdle** | 2 | The shoulder must *travel*, not just rotate — throwing and recoil both need it |
| 7–8 | Upper arm | 2 | Kinetic chain link |
| 9–10 | Forearm (+2 twist bones each, §2.1) | 2 | |
| 11–12 | Hand | 2 | Root of the finger rig (§2) |
| 13–14 | Thigh | 2 | |
| 15–16 | Shank | 2 | Achilles spring lives here (§C) |
| 17–18 | Foot (hindfoot) | 2 | |
| 19–20 | **Toe / forefoot** | 2 | ← the toe-off snap the sprint spec (§1.2) requires |

Fingers are **not** counted here — they are a sub-rig on segments 11–12,
specified separately in Brief VIII §2. Counting them would push this to 50+
and misses the point: these 20 are the **mass-bearing, physics-relevant**
segments.

### B.2 The critical fix: three trunk segments

> **Brief VIII §5.2 requires 35–45° of hip-shoulder separation at the throw's
> wind-up. With a single trunk segment this value is always 0° and the test can
> never pass.**

Splitting the trunk into **pelvis → lumbar → thorax** is the minimum that allows:
- **Hip-shoulder separation** in the throw and the thrust (§5)
- **Hips-lead-shoulders** in the plant-and-cut (§1.3)
- **Upper-body additive aim ±60° over turning legs** in third person (§6.2)
- **Torso coil** as anticipation on every power move (§1.3)

All four were already specced. None were buildable. Build the three-part trunk
first — it unblocks the rest of the brief.

### B.3 Segment mass fractions (of total body mass)

Standard published values. Put these in `config/limbs.ron`; they drive ragdoll
mass, inertia, follow-through weight, and hit-reaction impulse.

| Segment | Mass fraction (each) |
|---|---|
| Head + neck | 0.081 |
| Trunk (all three combined) | 0.497 — split thorax 0.216 / lumbar 0.139 / pelvis 0.142 |
| Upper arm | 0.028 |
| Forearm | 0.016 |
| Hand | 0.006 |
| Thigh | 0.100 |
| Shank | 0.0465 |
| Foot (hindfoot + toe) | 0.0145 — split hindfoot 0.011 / toe 0.0035 |
| Clavicle | ~0.005 (carve from thorax) |

Whole-body check: head 0.081 + trunk 0.497 + arms 2×0.050 + legs 2×0.161 = **1.000**.

### B.4 Segment lengths (fraction of total height H)

| Segment | Length |
|---|---|
| Upper arm | 0.186 H |
| Forearm | 0.146 H |
| Hand | 0.108 H |
| Thigh | 0.245 H |
| Shank | 0.246 H |
| Foot length | 0.152 H |
| Shoulder width | 0.259 H |
| Hip width | 0.191 H |
| Shoulder height | 0.818 H |

At H = 1.8m: upper arm 33cm, forearm 26cm, thigh 44cm, shank 44cm, foot 27cm.
**Use these to validate the existing rig** — proportions that drift from these
are the usual cause of a character reading as "off" without anyone being able
to say why.

### B.5 Inertial properties (for ragdoll, follow-through, and hit reactions)

Centre of mass, as a fraction of segment length **from the proximal joint**:
upper arm **0.436**, forearm **0.430**, hand **0.506**, thigh **0.433**,
shank **0.433**, foot **0.50**.

Radius of gyration about the segment CoM, as a fraction of segment length:
upper arm **0.322**, forearm **0.303**, hand **0.297**, thigh **0.323**,
shank **0.302**, foot **0.475**.

These are what make a heavy limb *feel* heavy: a segment's resistance to being
whipped is `m · (k·L)²`. Feed them into the §2.5 spring solver so the spring
stiffness per segment is **derived from mass, not hand-guessed** — that single
change removes most of the "why does this arm feel wrong" tuning loop.

### B.6 Tests
- **Segment-count test:** assert the rig exposes all 20 named segments.
- **Separation test:** drive the throw wind-up; assert
  `yaw(thorax) − yaw(pelvis)` reaches the 35–45° band (impossible before B.2 —
  this test *is* the proof the fix landed).
- **Proportion test:** every segment length within ±5% of its B.4 fraction.
- **Mass-closure test:** all segment masses sum to 1.000 ± 0.001.
- **Toe-off test:** in the sprint cycle, assert the toe segment rotates through
  its plantar-flexion range at contact-exit — no toe rotation means the run is
  still a glide.

---

## SECTION C — "ACHILLES MOTION" AS ACTUAL MECHANICS ★ upgrades Brief VIII §1

Brief VIII treated "Achilles" as an epithet. Take it literally instead: the
**Achilles tendon** is the mechanical reason human movement looks explosive, and
the literature gives numbers you can build with.

### C.1 What the research says

- Tendons **return 90–95% of stored elastic strain energy** on recoil — they are
  near-perfect springs, not dampers.
- Energy stored scales hard with intensity: about **1.3 J per step walking**,
  rising to roughly **38 J in a one-leg vertical jump** — a ~30× range across
  the same tissue.
- The mechanism is the **stretch-shortening cycle (SSC)**: pre-activation →
  the unit is stretched while already loaded (eccentric) → it shortens
  explosively (concentric). The tendon absorbs most of the length change while
  the muscle stays nearly isometric.

**Translated to animation:** explosive human motion is *not* muscle firing from
rest. It is **load, then release** — and the release is faster than the load
that produced it. Motion that skips the load phase looks robotic no matter how
fast the release is. That is precisely the "switch flip" anti-pattern, now with
a physical explanation.

### C.2 The Elastic Load Model (one shared utility, alongside §1.4's kinetic chain)

Every explosive action gains a `load` phase feeding a `release`:

```
ElasticMove {
  load_s,            // eccentric: the coil / counter-movement
  release_s,         // concentric: the whip
  stored_energy,     // 0..1, accumulates during load
  return_efficiency: 0.92,   // tendons give back ~90-95%
}
```

**Rules derived from the mechanics:**
1. **Release is 2–3× faster than load.** Wind-up 0.4s → release 0.15–0.20s.
   A release *slower* than its wind-up reads as a shove, never a strike.
   (Brief VIII's spear already fits: raise 0.4s, whip 0.08s.)
2. **Stored energy scales the output.** Release velocity =
   `base × (1 + stored_energy × 0.35)`. A fully loaded move is measurably
   stronger than a snap one — and it is *visible*, because the wind-up was longer.
3. **A counter-movement beats a dead start.** Any move preceded by motion in the
   *opposite* direction gets a bonus. This is exactly the running-throw bonus
   (§5.4) generalised: **the same rule should govern the jump, the dodge launch,
   and the melee thrust**, not just the throw.
4. **Landings recharge.** A landing that flows into the next move (a hop-to-throw,
   a land-into-sprint) carries stored energy forward — a small speed or damage
   bonus for *flowing*, zero for stopping first. This is the mechanical core of
   feeling like an athlete rather than a state machine.
5. **Never fully damp a landing.** Soften the touchdown over 2–3 frames with a
   small rebound (~8% of impact velocity returned upward) instead of clamping to
   zero. This is the same "the wall stop" fix, now with a number.

### C.3 Where it applies
Sprint start (coil → drive), jump (counter-movement dip → launch), dodge/roll
launch, spear thrust, spear throw, mech braced side-step (load ×2.2 duration,
`return_efficiency` 0.55 — steel springs are worse than tendons, and the mech
should feel it).

### C.4 Tests
- **Load-release ratio test:** for every `ElasticMove`, assert
  `release_s ≤ load_s / 2`.
- **SSC bonus test:** the same move from a dead stop vs from a counter-movement
  — assert the output difference matches `stored_energy × 0.35`.
- **Landing rebound test:** no landing reaches exactly zero vertical velocity in
  a single frame; a small rebound is present.
- **Capture:** side-by-side of a flat-footed action and the same action performed
  out of a flow. If the difference isn't visible in the clip, it isn't implemented (C8).

---

## SECTION D — THE MECH, READ FROM THE CONCEPT ART ★ replaces Brief VIII §7.2

The art is the spec. Below is what is actually drawn, part by part.

### D.1 Overall read
A **walking weapons platform** — closer to a light armoured vehicle on legs than
a humanoid robot. Hull cantilevered forward over reverse-joint legs, arms
mounted low on the hull sides, no head, no cockpit glass, no visible pilot
opening. It reads *utilitarian military hardware*, not hero mech: nothing is
decorative and every shape looks like it houses something.

**Stance:** hull pitched slightly nose-down, hips high and set back, knees
carrying the load forward — a machine leaning into its own weight. Reproduce
this in the idle pose; a level, upright hull loses the whole silhouette.

### D.2 Palette — **CORRECTION to Brief VIII §7.2**
The brief says *faceted gunmetal / dark gray*. The art is **olive drab / khaki /
field tan**: a warm desaturated military green-brown, with darker gray-green for
recessed mechanical areas and near-black for joints, barrels, and shadowed
underside. Metal shows only at wear points and mechanism.

```
hull_primary     #8A8770  (olive-khaki, matte, roughness 0.72)
hull_shadow      #5F5E52  (recessed panels)
mechanism_dark   #33352F  (joints, actuators, cabling)
barrel_metal     #2B2C2B  (gun metal, roughness 0.45)
wear_metal       #9A9384  (edge chips only — plate borders, foot cleats)
marking_white    #D8D4C6  (stencils: the "3" on the hull)
```
Emissive: **minimal**. The art has no glowing visor. Use one small sensor-lens
glint and per-tube status lights only — keep the machine reading as *equipment*.
If the game needs the team-colour visor from Brief VIII, restrict it to a thin
lens line, not a lightbar.

### D.3 Part inventory (mirrors the 20-segment body — these are also the Forge's swappable/detachable plates)

**Hull (5)**
1. Main hull box — flat angled top, chamfered front, stencil number panel
2. Front sensor plate — dark recessed rectangular panel, slightly inset
3. Upper rear pod pair — two horizontal cylinders (comms/cooling drums)
4. Antenna mast — thin whip, offset right, plus a short ball-tipped sensor stalk left
5. Dorsal hatch/vent plate

**Shoulders & arms (6)**
6. Left shoulder housing — boxy, side-mounted, hard mechanical seam
7. **Gatling arm** — 4–6 rotating barrels in a sleeve, muzzle ring at the tip
8. Gatling ammo feed housing beneath the barrel cluster
9. Right shoulder housing
10. **Autocannon arm** — long single barrel with a muzzle device, angled downward
11. **Drum magazine** — the large cylinder sitting on top of the autocannon; it
    is the most recognisable shape on the machine. Give it a visible seam,
    latch, and a slight rotation when firing.

**Hip & waist (3)**
12. Waist actuator block — the busiest mechanical area: pistons, linkages,
    exposed cabling between hull and legs
13. Hip yaw ring (allows the hull to rotate over the legs)
14. Hip armour skirts — small, do not fully hide the mechanism

**Legs, per side ×2 (6)**
15. Thigh plate — the large flat armour panel, the leg's dominant read
16. Knee mechanism — exposed pistons and linkage, deliberately *not* covered
17. Shin plate — smaller panel, angled
18. Ankle actuator cluster
19. Foot pad — wide, flat, splayed
20. **Foot cleats** — the tread blocks visible on the sole; these give the
    machine its grip-and-weight read. Do not smooth them out.

### D.4 Shapes and proportions (from the art)
- Hull footprint is **wider than tall** — a slab, not a tower.
- Thigh plate is the **single largest flat surface**; it is what reads at
  distance, so its silhouette and one strong panel line matter more than texture.
- The knee and waist mechanisms are **intentionally exposed** — this contrast
  between smooth armour panels and dense mechanical clutter is what makes the
  design read as real machinery. Covering them kills it.
- Feet are **wide and flat with a rear spur**, splayed for stability — not
  claws, not blades.
- The arms hang **low and forward** off the hull sides, well below hull top.
  Do not mount them at the shoulder-top like a humanoid.

### D.5 Weapon fit — **CORRECTION to Brief VIII §7.8**
The brief specifies a **4-tube missile pod** on the left shoulder plus one arm
hardpoint. The art shows **no missile pod** — it shows a **gatling on one arm
and a drum-fed autocannon on the other**.

Reconcile deliberately, and write the decision down:
- **D5-a (matches the art):** left arm gatling, right arm autocannon; the missile
  pod becomes an optional swappable hardpoint, absent by default.
- **D5-b (keeps the brief's kit):** add a dorsal missile pod on the hull top
  behind the sensor plate — it is the only place on this silhouette that can
  carry one without breaking the read.

**Recommendation: D5-a**, with the missile pod as a Forge-swappable variant.
The art's weapon fit is more distinctive, and the anti-infantry lock rules from
Brief VIII §7.8 carry over unchanged to whichever loadout ships.

**Autocannon (new, replaces the heavy rifle on this chassis):** drum-fed,
15-round drum, ~200 RPM burst-capable, 45 damage/round vs soldiers, visible drum
rotation per shot, heavy shell ejection, strong muzzle flash, and a hull-shove
recoil that visibly rocks the machine back on its hips — mass reads through
recoil more than through any texture.

### D.6 Damage states mapped to the actual parts (refines §7.7)
- **70% HP:** hip armour skirts + left thigh plate shear off → exposes the waist
  actuator block (already the most mechanical-looking area, so the exposure reads
  instantly).
- **40% HP:** shin plate + one rear hull drum drop; the antenna mast bends or
  snaps; sparks at the knee mechanism.
- **15% HP:** drum magazine casing chips and stops rotating smoothly, sensor
  plate cracks, one foot cleat row tears away producing a visible limp, smoke
  from the waist block.
- Exposed mechanism zones take **×1.25** after angle armour, as specced.

### D.7 Tests + captures
- **Palette audit:** assert no mech material uses the old gunmetal values; sample
  rendered pixels against the D.2 palette within tolerance.
- **Part-count test:** all 20 named parts exist as separate meshes (this is what
  makes both damage-detach and Forge customization work).
- **Silhouette test:** render the mech at the chosen scale beside a 1.8m soldier,
  from the art's ¾ angle, and place it next to the concept art in the handback.
  **This side-by-side is the completion criterion** — if the shapes don't match,
  the section isn't done.
- **Capture:** the side-by-side, plus front/side/rear/underside, plus a firing
  clip showing drum rotation and hull recoil shove.

---

## SECTION E — WHAT THIS PATCHES IN BRIEF VIII v2

| Brief VIII v2 | Change |
|---|---|
| §7.1 Scale 1.15× | **Blocked pending decision** — Section A. Recommend 1.7×. |
| §7.2 Gunmetal palette | **Replaced** by the olive-drab palette, D.2. |
| §7.2 Silhouette description | **Replaced** by the part-by-part read, D.3–D.4. |
| §7.8 Missile pod as core kit | **Replaced** — gatling + drum autocannon core; pod optional (D.5). |
| §7.7 Damage plates | **Refined** to the real parts (D.6). |
| §2 Rig (arms/hands) | **Extended** — now sits on the 20-segment body (B.1). |
| §5.2 Hip-shoulder separation | **Unblocked** — required the 3-part trunk (B.2). |
| §1.2 Sprint toe-off | **Unblocked** — required toe segments (B.1 #19–20). |
| §1.3 Anticipation | **Upgraded** to the Elastic Load Model with numbers (C.2). |
| §2.5 Spring stiffness | **Derived** from segment mass and radius of gyration, not guessed (B.5). |
| §1.4 Kinetic chain | Unchanged, but now runs on segments that can actually express it. |

**Build order:** B.2 (three-part trunk) → B.1 (remaining segments) → C.2
(elastic load) → Section A decision → D (mech visual rebuild). The trunk split
is first because four already-written specs are silently blocked on it.

---

*Sources: Achilles tendon elastic energy return and the stretch-shortening cycle
(tendons return ~90–95% of stored strain energy; ~1.3 J walking to ~38 J in a
one-leg jump); standard body-segment-parameter models (Dempster 1955, Winter
1990, de Leva 1996) for the 14–16 segment baseline, mass fractions, segment
lengths, CoM positions, and radii of gyration; the supplied concept art for all
of Section D. All numbers are starting tunables.*
