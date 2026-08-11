# BRIEF XI — AGILE MECH: FINAL MOTION, BODY, HAND, WEAPON & POLISH PASS

**Issued by the owner, 2026-08-10**, immediately after `BRIEF_X_agile_mech.md`
shipped its first build. Verbatim from §1 onward. Owner's framing:

> *"Finish the Agile Mech rather than continually redesigning unrelated
> systems, while making the shared limb/hand/weapon system explicit."*

**Do not redesign unrelated game systems.** Finish the Agile Mech.

---

## 0. LIVE CONSTRAINTS — read before section 1

### 0.1 §5 CLIMBING IS CONTESTED. DO NOT BUILD IT BLIND.

The owner asks to *"improve Agile Mech climbing animations"*, and §19 has a
`Climbing` checkbox. **There may be no such thing to improve.**

`BRIEF_X` §0 asserted the Agile Mech can climb. Friday33 then reported the
opposite: `climb_target` is gated `!m.in_mech()` **at the climber** and needs
a *dropped plate* on the target, making hull-climbing a verb for a pilot **on
foot** against an enemy mech's stripped hull — something you do *to* a mech,
not *in* one. A dedicated verification is settling this.

**Rule for this build:** do not touch §5 until the verdict is in writing. If
the Agile Mech cannot climb, §5 and the §19 `Climbing` box are **NOT
BUILDABLE AS WRITTEN** and become an owner decision — *"do you want mech
climbing built?"* — which is a new feature and a `sim.rs` change, not a
polish pass. Say so; do not quietly animate a verb that never fires, and do
not quietly skip it either.

### 0.2 The motion decision already specifies §1, and it is binding

`research/motion-architecture/DECISION.md` (2026-08-10) ruled: keep
procedural, sim-driven posing; add no animation crate; adopt nothing from the
motion-matching or neural families — because this project holds **zero
animation clips and zero licence-cleared mocap**, and the data behind that
literature is CC BY-NC-ND. Its three ordered additions are:

1. Extract the pure pose functions from `main.rs` into a benchmarkable module.
2. Replace `Fighter`'s ~25 concurrent phase timers with an explicit action
   state machine, **one verb at a time**.
3. **Generalise the existing two-bone IK solver so the legs can use it** —
   closing foot placement on uneven ground.

**Item 3 IS §1 of this brief.** Build it that way. `solve_arm_ik`
(`main.rs:2579`) is already a closed-form two-bone solver with pole vector,
elbow clamp and sprung targets — **generalise it, do not write a second one,
and do not add an IK crate** (evaluated and rejected in `DECISION.md`).

Item 1 also serves §18's reuse objective. Item 2 is sim-side and **out of
scope here.**

### 0.3 Lane rule

Presentation only — `main.rs`, `agile_mech.rs`, client modules, new modules.
**`sim.rs` is not yours.** If a section appears to need a sim change, STOP and
report it. `SCOUT_SCALE` remains frozen (open owner question Q5).

### 0.4 Carried forward from BRIEF_X, still open

- **Enemy Agile livery** — ships dark-blue primary with orange layered plates,
  because BRIEF_X's orange-primary and SPEC15's opposition red/blue faction
  language conflict. `scout_hull_foe` + `scout_plate_foe` is the whole change.
  Owner decision, unresolved. §16 says "do not change the established colour
  identity", which does **not** resolve it.
- **Double jump has no airborne pose.** `try_mech_jump`'s compress/tuck is
  heavy-only. The apex frame is indistinguishable from standing. This lands
  squarely in §3 and is the highest-value single fix in this brief: the
  Agile's signature mechanic is currently invisible.

---

## 1. AGILE MECH — FOOT & LEG MOTION

Spend significant attention on the **feet and legs**. Believable, responsive
foot placement during all major movement states: walking, running, sprinting,
jumping, landing, crouching, strafing, turning, climbing, aiming while
standing, aiming while moving, aiming while crouched.

The feet must not simply slide across the ground.

**Foot placement**, where technically possible: keep feet aligned with the
ground; adjust to terrain; prevent obvious foot clipping; prevent excessive
foot sliding; keep knees and ankles visually connected; respond naturally to
slopes and uneven surfaces.

> It should look like it is **actually stepping and supporting its weight**,
> not a humanoid animation pasted onto a robot.

## 2. WALKING & RUNNING

**Walking** — controlled steps, shorter stride, stable torso, minimal
vertical movement.

**Running** — longer stride, faster leg movement, more forward body
inclination, more dynamic arm movement.

**Sprinting** — strong forward lean, faster foot placement, larger stride,
more visible mechanical movement.

Keep it lighter and faster than the Big Mech.

## 3. JUMPING & LANDING

**Jump** — the legs visibly prepare: knees compress slightly, body lowers,
mechanical joints react, feet push away from the ground. In the air, legs
react naturally; avoid completely rigid limbs.

**Landing** — feet contact the ground, knees compress, body absorbs the
impact, mechanical joints react briefly. Readable but not exaggerated.

*(See §0.4 — the double jump currently has no airborne pose at all.)*

## 4. CROUCHING

The entire mechanical lower body participates. **Do not simply lower the
character vertically.** Knee bending, hip movement, ankle adjustment, lower
torso movement, foot repositioning. Feet remain planted naturally. It should
look mechanically capable of crouching rather than shrinking downward.

## 5. CLIMBING

**GATED — see §0.1 before starting.** As written: hands visibly interact with
the surface; feet move between footholds where possible; knees and elbows bend
naturally; body shifts toward the surface; mechanical joints follow. Avoid
stiff climbing. Prioritise believable **Hands → Surface** and
**Feet → Surface** contact.

## 6. AIMING + FOOT DYNAMICS

Aiming must not completely freeze the lower body. Upper body focuses toward
the crosshair; feet remain naturally planted; legs subtly adjust to direction;
turning involves the feet/body where appropriate.

Aiming while moving: controlled foot placement, no excessive leg animation,
stable. It should look like a disciplined combat unit.

## 7. PLAYER + MECH LIMB SYSTEM

A detailed design pass on player arms, mech arms, elbows, shoulders, wrists,
hands, fingers, mechanical joints — **but do not create completely separate
complicated systems for every character.** Two core reusable families:

- **DESIGN A — PLAYER LIMBS.** Human/player-oriented arm, hand, joint, finger.
- **DESIGN B — MECH LIMBS.** A mechanical version of the same underlying
  structure.

Both modular and reusable.

## 8. REUSABLE LIMB COMPONENTS

```text
Arm
 ├── Upper Arm
 ├── Elbow Joint
 ├── Forearm
 ├── Wrist
 └── Hand
      ├── Palm
      └── Fingers
```

The same component architecture supports different sizes and shapes. Variation
through scale, proportions, armour plates, joint shape, finger length, finger
thickness, small graphical details.

**Do NOT build a completely unique arm system for every mech.**

## 9. MECH ARM DESIGN

Lightweight, mechanical, flexible, precise, combat capable. Simple geometric
components — cylinders, boxes, rounded joints, armour plates. Elbow and wrist
joints visually clear. The arms should communicate precise weapon handling.

## 10. MECH HAND & FINGER DESIGN

Simple but functional: palm structure, thumb, multiple articulated fingers,
small mechanical joints. Not unnecessarily complex.

The requirement is that the hand convincingly interacts with **guns, bow,
spear, grenade and other equipment** — fingers follow the weapon grip rather
than clipping through it.

## 11. PLAYER HAND DESIGN

The same pass on existing player hands: palm proportions, finger positioning,
thumb placement, wrist connection, weapon grip. Simple enough to be reused
across weapons.

## 12. SHARED HAND/WEAPON INTERACTION

Reusable weapon-hand attachment logic supporting gun, bow, spear, grenade.
Each weapon may have its own grip position; the underlying hand/arm system
stays shared.

```text
Weapon → Grip Point → Hand IK / Animation → Arm → Shoulder
```

This should remove the need to hand-tune hand positions per weapon.

## 13. GUN GRAPHICS — DEDICATED VISUAL PASS

Guns are not placeholder geometry. Improve silhouette, barrel, receiver,
magazine, grip, trigger area, stock, mechanical components, attachments, small
surface details.

Keep the art direction: **stylized + technical + simple geometry + readable
shapes.** Do not make every gun unnecessarily complicated.

## 14. GUN + AGILE MECH INTEGRATION

The gun must sit correctly in the mechanical hand, align with the arm, follow
aiming, rotate naturally, avoid clipping through armour, and stay correctly
positioned in **both** first and third person. It should look like it is
**actually holding the weapon.**

## 15. FIRST-PERSON WEAPON VISUALS

Maintain the previous rules: primarily right side, never covering the
crosshair, controlled movement, minimal unnecessary sway, follows aim
direction. Applies to the Agile Mech's guns as well as bow and spear.

## 16. AGILE MECH COLOR & MATERIALS

Keep the established identity: industrial/burnt orange primary; metallic blue
mechanical components; subtle brighter blue technology accents; dark
graphite/black structure. **Do not change the established colour identity
during this pass.** *(Does not resolve §0.4's enemy-livery question.)*

## 17. FINAL AGILE MECH INTEGRATION

Finish everything else required. Verify: body, armour, head, arms, hands,
fingers, legs, feet, joints, movement, climbing, jumping, landing, crouching,
aiming, weapons, weapon grips, first-person presentation, third-person
presentation, animations, materials, VFX, collision, physics, performance, UI
integration, inventory integration.

**Do not leave obvious placeholder systems behind.**

## 18. IMPORTANT — REUSE SYSTEMS

The main technical objective is **reuse**.

> **One good reusable system** over **many separate systems that do the same
> thing.**

Reuse existing player movement, animation systems, skeleton/rig where
practical, weapon systems, inventory, aiming, physics. Create modular
variation through scale, proportions, armour, shapes, small graphical changes.
This should make future character/mech development significantly easier.

## 19. FINAL QUALITY CHECK

**Movement:** walking · running · sprinting · strafing · turning · crouching ·
jumping · landing · climbing *(gated, §0.1)*

**Combat:** standing aim · moving aim · crouched aim · gun handling · bow
handling · spear handling · grenade handling

**Character:** arms · elbows · wrists · hands · fingers · mechanical joints ·
feet · knees · ankles

**Presentation:** first person · third person · orange armour · metallic blue
machinery · blue technical accents · clear silhouette · no unnecessary visual
clutter

**Technical:** reusable components · no unnecessary duplicated systems · no
major animation clipping · no obvious foot sliding · no weapon-hand clipping ·
reasonable performance · existing gameplay remains functional

### FINAL OBJECTIVE

Finish the Agile Mech so it feels like a **complete playable character**, not
a visual prototype:

> Existing Player Mechanics + Reusable Limb/Hand System + Detailed Agile Mech
> Armour + Proper Foot/Leg Dynamics + Functional Weapon Grip + Polished Gun
> Graphics + First/Third Person Integration

— modular enough to reuse for future mechs and characters.

---

## PROOF STANDARD

OPERATION rule 8: **the capture is the instrument.** Every box in §19 that is
a visual claim needs a screenshot from the build that was actually launched.
A tick with no capture behind it is a hope, not a claim.

Rule 12: **mutation-prove every test.** A test that derives its expected value
from the code under test cannot fail. One was already caught doing exactly
that during the BRIEF_X build.

"Foot sliding" and "no clipping" are the two claims most likely to be ticked
without evidence, because both need motion to see. Capture **consecutive
frames**, not one pose.
