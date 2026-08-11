# HUD REFERENCE — the two images for BRIEF XII

## OWNER: DROP THE TWO IMAGES IN THIS FOLDER

| Save as | Which one |
|---|---|
| `img1-mech-pov.png` | The **modern military shooter** (Battlefield) — grey sky, glass office building, rifle with iron sights, red *"ENEMY CONTROLS ALMOST ALL SECTORS"* banner |
| `img2-human-pov.png` | The **stylized cartoon shooter** (Team Fortress) — desert town, wooden CORNWELL / JENKIN COAL buildings, shotgun, red engineer portrait |

**Image 1 is the MECH POV reference. Image 2 is the HUMAN POV reference.**
That mapping comes from the owner's own brief and does not depend on the
order they were pasted in — they have been sent in both orders.

## WHY THIS FILE EXISTS

**This project has already destroyed one spec by losing an uploaded image.**
The mech concept art was uploaded, `BRIEF_VIII_B` §D was written from it
declaring *"the art is the spec"*, and four completion criteria across two
briefs are unsatisfiable to this day because the file was never saved.
`TRV-0260` is still open on it and it is the oldest live ask in the ledger.

So the images are described below in words, written while they were on
screen. **Until the PNGs land here, this description IS the reference.**
A builder may work from it. When the PNGs arrive, they supersede it.

---

## IMAGE 1 — MECH POV reference

Modern military shooter. First person, rifle low and right, iron sights on
the barrel. Overcast sky, a glass-and-concrete office block across a
concrete embankment.

**Layout — this is what matters, not the game:**

- **The centre of the screen is completely empty.** No frame, no panel, no
  cockpit border. A small dark reticle over the sights and nothing else.
  Every reading sits in a corner.
- **Top centre** — one urgent line, red caps, no panel behind it:
  *"ENEMY CONTROLS ALMOST ALL SECTORS"*. A state warning, one line only.
- **Upper-left, lower down** — a transient event strip on a dark red-edged
  bar with a red `!`: *"Enemy took Sector D // Friendly -10"*. The event
  **and its cost** on one line, not in two places.
- **Bottom left, one aligned cluster** — a small terrain minimap; two large
  numbers side by side above it (**864** health, **877** armour) each with
  a small icon; a squad roster of four rows, each with a name, a coloured
  class icon and a thin state bar; and a row of tiny equipment pips along
  the very bottom edge.
- **Bottom right** — the weapon cluster: **031** current ammo, very large
  and white; **124** reserve, small, to its right; beneath them a row of
  cyan equipment icons (sidearm, grenade, gadget) in a thin dark strip.
- **Bottom centre** — a compass strip with a heading readout, *"127 E"*.
- **Right edge** — two tiny circular cyan markers pointing at off-screen
  objectives.
- **In the world, NOT on the HUD** — objective markers sit on the building
  itself: a red hexagon icon, small lettered squares, a floating capture
  indicator. **Objectives live in world space; status lives at the edges.**

**Material language:** dark translucent panels with thin light strokes;
cyan/teal for systems and equipment; white for the big numbers; red
reserved strictly for threat and loss. Layered and technical, information
dense — and still, the middle of the screen is clear.

---

## IMAGE 2 — HUMAN POV reference

Stylized cartoon shooter. First person, shotgun bottom right, warm desert
town, wooden buildings reading CORNWELL and JENKIN COAL.

**Layout:**

- **Centre screen empty again.** No reticle frame, no panels, nothing.
- **Top left** — a small vertical list of four buildable states, each a
  flat row: tiny icon, label, status word (*Sentry / Not Built*,
  *Dispenser / Not Built*, *Entrance*, *Exit*). Thin dark backing, no
  border, no box.
- **Top centre** — a compact team/objective progress strip with small class
  icons and a timer.
- **Right, mid-height** — a small contract/objective card, quiet, easily
  ignored.
- **Bottom left** — a character portrait plate with **200** health beside a
  cross icon, in bold saturated red. Large type.
- **Bottom left, above it** — chat and event text in coloured type with
  **no panel behind it at all**, sitting directly on the world.
- **Bottom right** — **3** current ammo very large, **32** reserve smaller
  beside it, both red on a flat dark plate.
- **Bottom centre** — two tiny flat team indicators.

**Material language:** flat, unframed, saturated. Bold readable type doing
the work instead of chrome. Numbers are big and everything else is small.

---

## WHAT THE PAIR ACTUALLY INSTRUCTS

The two are not two art styles to copy. They are **one layout discipline at
two densities**:

| | Human POV (img 2) | Mech POV (img 1) |
|---|---|---|
| Framing | none — flat on the world | layered translucent panels |
| Density | minimum viable | richer, still corner-bound |
| **Centre screen** | **empty** | **empty** |
| Health | one big number, bottom left | big number + armour + bar |
| Ammo | big / small pair, bottom right | big / small pair, bottom right |
| Extra systems | none | targeting, heat, integrity, lock |
| Feeling | *"I am a person with a gun"* | *"I am operating a machine"* |

**The shared rules, which both references obey without exception:**

1. The centre of the screen is never touched.
2. Health and ammo are the two biggest glyphs, in opposite bottom corners.
3. Current and reserve ammo are a **big/small pair**, never equal weight.
4. Transient messages have little or no panel; permanent readings get one.
5. Objectives belong in the world, not in a list.
6. Colour carries meaning — one hue for threat, one for systems — rather
   than decorating.

**Adapt to this game's own art direction.** `frontend.rs` already holds a
committed language: `palette::GOLD` / `INK` / `INK_SOFT` / `NEON_BLUE` /
`NEON_RED`, the `CARTOON` border constants, and the
`T_TITLE`/`T_HEAD`/`T_BODY`/`T_SUB`/`T_MICRO` type ramp. §1 of the brief
says reuse the existing visual language — **that ramp and that palette are
the existing visual language.** Do not invent a second one.
