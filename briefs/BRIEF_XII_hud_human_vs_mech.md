# BRIEF XII — PLAYER UI / MECH POV: HUD REFINEMENT

**Issued by the owner, 2026-08-11**, with two reference images. Verbatim
from §1 onward.

> **Do not just make the UI prettier. Make it feel like a finished
> commercial game.**

---

## 0. THE REFERENCE IMAGES — WRITTEN DOWN, BECAUSE PICTURES GET LOST

**Two images were uploaded with this brief and they are the primary visual
direction. They are NOT in this repository.**

This project has already lost one spec exactly this way: the mech concept
art was uploaded, `BRIEF_VIII_B` §D was written from it saying *"the art is
the spec"*, and four completion criteria across two briefs are now
unsatisfiable because nobody saved the file. `TRV-0260` is still open on it.

**Owner action:** drop both images into
`engine/crates/jk_tdm/handback/reference/hud/` as `img1-mech-pov.png` and
`img2-human-pov.png`. Until then, the descriptions below **are** the spec —
they were written from the images while they were on screen, and any builder
must treat this section as the reference rather than guessing.

### IMAGE 1 — the MECH POV reference (modern military shooter, Battlefield-like)

First-person, weapon low and right, sights visible. What matters is the
LAYOUT, not the game:

- **The centre of the screen is completely empty.** No panel, no frame, no
  clutter. Only a small, quiet reticle. Everything else is pushed to the
  edges. This is the single most important property of both references.
- **Top centre:** one urgent line in red caps on a dark strip — a state
  warning (*"ENEMY CONTROLS ALMOST ALL SECTORS"*). One line, transient.
- **Upper left, below it:** a second transient notification strip with a
  red `!` icon — an event plus its consequence in one line (*"Enemy took
  Sector D // Friendly -10"*). Event and cost together, not separately.
- **Bottom left:** the survivability cluster — two large numbers side by
  side (health and armour), a segmented horizontal bar beneath them, and a
  compact squad roster of four rows, each row an icon, a name and a thin
  state bar. Dense but aligned on a single left edge.
- **Bottom right:** the weapon cluster — a very large current-ammo number,
  a small reserve number beside it, and a row of small equipment icons
  beneath. Ammo is the biggest glyph on the screen after the health number.
- **Bottom centre:** a compass strip with a heading readout (*"127 E"*).
- **In the world, not on the HUD:** objective markers — small coloured
  diamonds and lettered squares floating on the building itself, plus tiny
  icons at the screen edges pointing to off-screen ones. **Objectives live
  in the world; status lives at the edges.**
- **Materially:** dark translucent panels, thin light strokes, cyan/white
  text, red reserved for threat. Layered, technical, information-dense —
  but every cluster is in a corner and the middle is clear.

### IMAGE 2 — the HUMAN POV reference (stylized cartoon shooter, TF2-like)

Same first-person framing, radically less UI:

- **Flat, not framed.** No translucent panels, no borders, no chrome. The
  numbers and icons sit directly on the world.
- **Bottom right:** health as one very large number on a simple coloured
  plate, and ammo as `current / reserve` in large type beside it. That is
  nearly the whole HUD.
- **Top left:** a small vertical list of buildable/equipment states, each a
  tiny icon, a label and a status word — flat rows, no box around them.
- **Bottom left:** transient text (chat/kill feed) with **no panel behind
  it at all**, coloured by team.
- **Top centre:** a compact objective/progress strip.
- **Materially:** bold saturated colour, thick readable type, cartoon-clean
  shapes, and again **an entirely empty centre screen.**

### What the pair actually instructs

The two images are not two art styles to copy. They are **one layout
discipline at two densities**:

| | Human POV (Image 2) | Mech POV (Image 1) |
|---|---|---|
| Framing | none — flat on the world | mechanical, layered panels |
| Density | minimum viable | richer, still corner-bound |
| Centre screen | empty | empty |
| Health/ammo | large, bottom corners | large, bottom corners |
| Extra systems | none | targeting, heat, integrity, lock |
| Feeling | *"I am a person with a gun"* | *"I am operating a machine"* |

**Adapt to this game's own art direction.** The frontend already has a
committed visual language — `palette::GOLD` / `INK` / `INK_SOFT` /
`NEON_BLUE` / `NEON_RED`, the `CARTOON` border constants, and the
`T_TITLE`/`T_HEAD`/`T_BODY`/`T_SUB`/`T_MICRO` type ramp in `frontend.rs`.
§1 says reuse the existing language — that ramp and that palette **are**
the existing language. Do not invent a second one.

---

## 0.1 LIVE CONSTRAINTS

- **Lane:** presentation only. The HUD lives in `main.rs` and client
  modules. **`sim.rs` is not yours** and is currently dirty with another
  lane's spear work. A HUD may never write sim state; a cosmetic system
  that touches simulation is a bug by this project's own rule.
- **Dispatch through `balance` first.** `main.rs` is the most contested
  file in the repo and this brief targets it directly. Prefer a NEW client
  module (`hud.rs`) over growing `main.rs` — new files do not collide, and
  §1's reuse goal is served better by one module than by more inline code.
- **This brief overlaps Trevor's Task 9, "the screens that lie to the
  player."** Six known HUD honesty defects are already logged: the Field
  Manual prints the default score rather than the match's, `U` is missing
  from the bind list, the medic jump/flip are undocumented, plus
  `TDM_TARGET_CHOICES`, `FORGE_SLOTS` and `gatling_heat`'s two scales.
  **§8's "remove redundant or outdated UI elements" is that task.** Do them
  in this pass; a HUD that is beautiful and wrong is worse than the one it
  replaced.
- **Existing UI to review first, per §1:** `frontend.rs` (title, learn,
  main menu, match complete — built this week, and the strongest visual
  work in the project), the `menu_ui` module and its `ZL_*` z-layers, the
  `palette` module, the in-match HUD in `main.rs`, and the Field Manual,
  Controls and Settings surfaces. **Read them before designing anything.**

---

## 1. REVIEW EXISTING WORK FIRST

Spend significant time reviewing the existing UI system and older UI work
already created for this game before making changes. Reuse the strongest
ideas, components, layouts and visual language from the previous versions
rather than rebuilding from scratch.

- Inspect the current player UI implementation.
- Review older UI versions, previous designs, existing assets/components.
- Identify elements from older versions that were visually stronger or more
  functional.
- Reuse and refine those where appropriate.
- **Do not unnecessarily introduce a completely new UI language.**

## 2. SIMPLIFY THE PLAYER UI

Make the HUD cleaner, less cluttered, instantly understandable, more
professional, more consistent across gameplay, and visually integrated with
the world.

Avoid excessive boxes, unnecessary text, oversized HUD elements and
distracting decorations.

> It should feel like a **real game HUD**, not a developer/debug interface.

## 3. USE THE UPLOADED REFERENCE IMAGES

Use them as visual direction and inspiration, adapted to our art style.
**Do not simply copy them.** Pay particular attention to: HUD placement,
information hierarchy, reticles, panels, mechanical interfaces, camera
framing, minimal visual indicators, and how information appears without
blocking gameplay.

*(The images are described in §0. Until they are in the repo, §0 is the
reference.)*

## 4. MECH POV — INSIDE THE MECH

**Image 1 is the primary reference.** When operating the mech, the
perspective must feel significantly different from normal play: futuristic,
mechanical, immersive, powerful, technological, and slightly richer than
the human HUD.

Use more visual/mechanical elements here: mechanical HUD framing, targeting
systems, weapon status, mech health/integrity, heat/energy indicators,
target lock, aim/reticle systems, minimal cockpit/mechanical overlays.

**Do not overcrowd the screen. Every element must have a gameplay purpose.**

> The player should immediately feel: *"I am operating a machine."*

## 5. OUTSIDE THE MECH — NORMAL PLAYER POV

**Image 2 is the primary reference.** Simpler, more lightweight, more
human/player-oriented, less mechanical, less visually dominant. Keep only
genuinely useful information.

The difference between **OUTSIDE → INSIDE** must be immediately
recognizable.

## 6. CLEAR VISUAL HIERARCHY

1. Immediate combat information
2. Health / survivability
3. Weapon / ammunition
4. Targeting information
5. Important objectives
6. Secondary information

**Do not give every piece of information equal visual weight.**

## 7. PRESERVE GAMEPLAY

The redesign must not interfere with movement, aiming, shooting, mech
interaction, enemy visibility or environmental awareness. The HUD supports
gameplay rather than competing with it.

## 8. IMPLEMENTATION

Before finishing, check: all existing player UI states; normal player POV;
mech POV; entering/exiting the mech; aiming and shooting; health/weapon
states; different resolutions and aspect ratios. Remove redundant or
outdated UI elements. Make spacing, sizing, typography, icons and alignment
consistent.

## OVERALL DIRECTION

Reuse the best parts of the old UI, combine them with the uploaded
references, simplify the current HUD, and create a strong visual distinction
between human POV and mech POV. The result should feel **clean, futuristic,
functional and immersive — with the mech interface being the more advanced
visual experience.**

---

## PROOF STANDARD

Rule 8: **the capture is the instrument.** Every claim here is visual, so
this brief is unprovable without screenshots from the launched build. At
minimum:

- Human POV, idle · aiming · firing · low health
- Mech POV, idle · aiming · firing · overheating · target locked
- **The transition**: the frame before boarding and the frame after. The
  "immediately recognizable" difference in §5 either survives that pair or
  it does not, and one image cannot show it.
- At least two aspect ratios, per §8. The frontend already has a
  `win_aspect` path — reuse it rather than assuming 16:9.

Rule 12: **mutation-prove every test.** A HUD test that reads the same
constant the HUD reads cannot fail.
