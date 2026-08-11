# BRIEF X — THE AGILE MECH

**Issued by the owner, 2026-08-10.** Verbatim below from §1 onward.
Ledger row `TRV-0011` (SPEC15 P3, *"the largest visual item"*), Task 2 in
`research/TREVOR_TASKS.md`.

Redesign and implement the **Agile Mech** as a cool, simple, highly
recognizable cartoon-style mech built from technical geometric shapes.
It should feel like a **lightweight, fast mechanical version of the
existing player character**, not a separate character system.

---

## 0. WHAT ALREADY EXISTS — verified 2026-08-10, do not re-derive

The owner's closing instruction was: *"don't forget it can climb, double
jump, and double q dodge."* **All three already exist, and all three are
SCOUT-EXCLUSIVE.** They are not work to add — they are behaviour this
redesign must not break, and the visual identity this brief asks for
should make them legible.

| Thing | Where | State |
|---|---|---|
| The Agile Mech itself | `ArmorSet::ScoutMech` (`sim.rs:4801`), labelled `"AGILE"` at `sim.rs:4907` | Exists |
| Scale | `SCOUT_SCALE` via `ArmorSet::ScoutMech` arm (`sim.rs:4893`) | Exists — 1.05, and **owner question Q5 is open on it** |
| Hull | `SCOUT_HULL` (`sim.rs:3583`) | Exists |
| Weapons | `&[MechWeapon::Plasma, MechWeapon::Repair]` (`sim.rs:5189, 5201`) | Exists |
| Spawn pad | `PickupKind::ScoutArmor` (`sim.rs:4109, 6906`) | Exists |
| **Dodge (Q)** | Bind `sim`-side `cmd.dodge` (`sim.rs:3793`, handled `sim.rs:8379`); key at `main.rs:5120`, `main.rs:17125` | Exists — duck-spin somersault, `ROLL_SPEED` 8.6 |
| **Air flip (Q in air + direction)** | Same key, `flip_*` fields (`sim.rs:7158-7162`) | Exists — *"Ground: dodge roll — Air + direction: FLIP (no firing)"* |
| **Second flip charge** | `scout_second_flip_used` (`sim.rs:7163`) | Exists — **scout-only** |
| **Double jump** | `scout_air_jump_used` (`sim.rs:3447`, `sim.rs:8477`) | Exists — a single discrete air impulse, **scout-only**. Its doc comment is explicit that it is NOT the deleted heavy thruster system, which stays deleted. |
| **Climb** | 190 references in `sim.rs` | Exists |

**So "double Q dodge" is: Q on the ground is a dodge roll; Q again in the
air is a flip; the scout is the only chassis with a second flip charge
and an air jump on top.** That triple is the Agile Mech's mechanical
identity, and this brief's job is to give it a *visual* identity that
matches it.

**This is therefore a ONE-LANE task.** Presentation only — `main.rs` and
client modules, `friday33`. No `sim.rs` change is required or wanted. If
the build appears to need one, stop and say so rather than reaching
across the line; `SCOUT_SCALE` in particular is an OPEN OWNER QUESTION
(Q5) and must not be changed as a side effect of this work.

---

## 1. CORE DESIGN CONCEPT

Constructed visually from simple technical shapes: boxes, rectangles,
cylinders, circles, rounded mechanical joints, layered armour plates,
simple mechanical connectors, small technical components.

Simple, but detailed enough to look intentional and polished:

> **Simple geometric construction + mechanical engineering details +
> cartoon proportions**

Not photorealistic, not unnecessarily complicated. The silhouette must
immediately communicate **FAST + LIGHT + AGILE + MECHANICAL**.

## 2. COLOR IDENTITY — orange + metallic blue

**Primary armour — orange.** Dominant. Burnt orange, industrial orange,
warm orange, slightly darker orange for secondary plates. Avoid extremely
bright flat orange; it should read as painted industrial armour.

**Secondary materials — metallic blue.** Dark metallic blue, steel blue,
blue-gray metal. On joints, inner armour, mechanical limbs, connectors,
weapon mounts, exposed machinery.

**Blue highlights.** A brighter metallic/electric blue, used *sparingly*:
energy components, small lights, reactor/core details, armour seams,
mechanical interfaces. Do not make the whole mech neon.

**Supporting materials.** Dark graphite, black, dark steel for small
mechanical components.

> **Industrial Orange Armour + Metallic Blue Machinery + Dark Graphite
> Structure + Subtle Bright Blue Technology**

This becomes the Agile Mech's recognizable visual identity.

## 3. BODY & SILHOUETTE

Visually different from Big and Royal. Compact torso, relatively narrow
profile, lightweight armour, smaller shoulder structure, flexible-looking
joints, mechanical legs designed for movement, compact head, lightweight
weapon mounts, small backpack/reactor system. Not bulky.

> **Speed rather than strength.**

## 4. HEAD DESIGN

A dedicated mechanical head/helmet. **NO HAT.** Do not place a hat or a
normal human head accessory on the Agile Mech. Build a simple mechanical
head from small geometric armour pieces, a mechanical faceplate, a small
visor, blue technological lighting, orange armour accents — in the same
cartoon-tech style as the rest.

## 5. ARMOUR DESIGN — modular pieces

**Torso.** Main orange chest plate, smaller layered orange armour,
metallic-blue inner structure, central mechanical/energy component.

**Shoulders.** Circular mechanical shoulder joints, compact orange
shoulder plates, metallic-blue connectors. Smaller than the Big Mech's.

**Arms.** Cylindrical joints, orange upper-arm armour, metallic-blue
mechanical sections, compact forearm armour.

**Legs.** Clearly built for agility: mechanical thigh sections, circular
knee joints, layered orange shin armour, metallic-blue internal
components, compact feet. They must look capable of running and changing
direction quickly.

**Back.** A compact technical backpack — small reactor, mechanical
housing, vents, blue energy detail, orange armour casing. Not oversized.

## 6. REUSE EXISTING PLAYER CHARACTER MECHANICS

Where technically possible, reuse the existing player character system.
**Do NOT build a new player controller for the Agile Mech.** Reuse
existing movement, physics, animations, controller, combat logic, aiming,
jumping, running/sprinting, turning, weapon interaction.

> **Existing Player Mechanics + Agile Mech Body/Armour/Visuals =
> Playable Agile Mech**

## 7. PLAYER BODY INTEGRATION

If the existing body and skeleton support it: keep the skeleton, keep the
animations, build the armour around the existing body, replace or cover
the appropriate body areas, ensure the armour follows the skeleton. Use
the player's existing motion to make the mech move naturally.

If proportions are insufficient, create a compatible mech body while
still reusing the controller, animation architecture, physics and combat
systems. Do not rebuild the character system unnecessarily.

## 8. ANIMATION & MOTION

Must work correctly with existing player movement. Test with: idle,
walking, running, sprinting, jumping, landing, turning, strafing, aiming,
attacking, weapon switching, throwing — **and, per §0, climbing, the
double jump, the ground dodge roll and the air flip.**

Avoid armour clipping, floating armour, detached components, broken
joints, animation distortion, weapons passing through armour.

## 9. CARTOON-TECHNICAL VISUAL STYLE

Not a realistic military robot. Clear shapes, slightly exaggerated
proportions, simple armour plates, visible mechanical joints, strong
silhouette, readable colors, clean geometry. Instantly recognizable at a
distance.

## 10. TECHNICAL DETAIL WITHOUT OVERCOMPLICATION

Detail through **layering**, not polygon count:

> **Large shape → armour plate → mechanical joint → small technical
> component → blue accent**

No hundreds of tiny details invisible in gameplay. Every detail must
serve function, silhouette, mechanical identity, or readability.

## 11. WEAPON INTEGRATION

Existing weapons must visually attach. Weapon mounts use the same orange
+ metallic blue + dark graphite language. Attachment points follow the
existing player weapon system. No separate weapon-control architecture
unless necessary.

## 12. PERFORMANCE

Low/medium-poly. Reuse materials and meshes. Avoid unnecessary unique
materials. Reasonable draw calls. Modular components, optimized repeated
mechanical parts.

> **Simple geometry with strong visual identity.** Not maximum polygon
> count.

## 13. DIFFERENTIATE FROM OTHER MECHS

- **Agile** — Fast / Light / Orange + Metallic Blue / Compact
- **Big** — Heavy / Large / Powerful
- **Royal** — Largest / Premium / Highly detailed / Powerful

The Agile Mech must clearly occupy the **speed-focused visual role**.

Note the live constraint from `WHATS_MISSING.md`: the player Royal ships
GOLD and the opposition Royal now carries YELLOW (owner ruling
2026-08-10). Orange sits between them on the wheel, so the squint test
matters more here than usual — the Agile Mech must separate from both
Royals **by silhouette**, not by hue alone.

## 14. FINAL DESIGN TARGET

> **The existing player character transformed into a lightweight
> cartoon-tech combat mech using modular industrial-orange armour,
> metallic-blue machinery, and subtle blue energy technology.**

Cool, simple, mechanical, fast, modern, recognizable, easy to animate,
efficient to render. Spend the effort on body and armour; reuse the
mechanics.

---

## FINAL ACCEPTANCE CHECKLIST

- [ ] Agile Mech has a simple cartoon-tech design.
- [ ] Model is primarily constructed from boxes, circles, cylinders, and armour plates.
- [ ] Agile silhouette is compact and lightweight.
- [ ] Agile Mech is clearly different from Big and Royal Mechs.
- [ ] Primary armour uses industrial/burnt orange tones.
- [ ] Mechanical components use metallic blue.
- [ ] Subtle brighter blue technology accents are included.
- [ ] Dark graphite/black supports the main materials.
- [ ] **No hat.**
- [ ] Dedicated mechanical head/helmet exists.
- [ ] Existing player movement is reused.
- [ ] Existing player animations are reused where possible.
- [ ] Existing player physics/controller is reused.
- [ ] Agile armour follows the existing skeleton correctly.
- [ ] Armour does not break during movement or combat.
- [ ] Existing weapon systems remain compatible.
- [ ] Technical detail comes from simple layered geometry.
- [ ] Model remains performance-conscious.
- [ ] Agile Mech clearly communicates speed and mobility.
- [ ] Orange + metallic blue becomes the Agile Mech's recognizable identity.

### Added by §0 — the mechanics that must survive

- [ ] **Climb** still works and reads correctly with the new armour.
- [ ] **Double jump** (`scout_air_jump_used`) still fires and is visible.
- [ ] **Ground dodge roll (Q)** does not clip the new armour.
- [ ] **Air flip (Q airborne + direction)**, including the scout's second
      flip charge, does not clip or detach anything.
- [ ] `sim.rs` is unchanged by this work.
- [ ] `SCOUT_SCALE` is unchanged (owner question Q5 is open).

**Per OPERATION rule 8, every checklist line above that is a visual claim
needs a screenshot from the build that was actually launched. A tick with
no capture behind it is a hope, not a claim.**
