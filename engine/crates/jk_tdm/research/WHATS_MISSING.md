# What is missing / not built yet — 2026-08-04

Compiled for the user from `BACKLOG.md`, `THOR_LOG.md`'s ranked findings,
and session knowledge. Ordering inside each tier is Thor's ranking rule:
an item moves up when its blocker clears, not when it becomes interesting.

## 0. Needs the USER (nothing code-side can proceed)

| What | Where it goes | What happens today |
|---|---|---|
| The 3 branding PNGs (key art, wordmark, emblem) | `engine/assets/branding/` as `key_art.png`, `wordmark.png`, `emblem.png` | All branding code ships and degrades gracefully — splash, menu art, emblem placements simply don't render until the files exist. See `engine/assets/branding/README.md` |

## 1. Small, one-session buildable (ranked, next up)

**All seven of the items that stood here on 2026-08-04 are now BUILT** —
see "Closed 2026-08-04" below. What remains of this tier is the tail of
THOR_LOG's 97:

1. **HUD award toasts** — §4.3 specs resource-award toasts stacking above
   the resource counter, each fading after 2.5 s. No resource economy
   exists in TDM/KOTH to award from, so this waits on a mode that has
   one rather than being faked.
2. **Killfeed WALLBANG modifier** — needs bullet penetration to exist
   first; there is no wallbang mechanic to report. (THROUGH-SMOKE
   shipped 2026-08-04: the hitscan ray now tests smoke crossings and
   the killfeed marks them `~`.)
3. **Numeric tunings** — the remaining screen-intrusion profiles called
   out in THOR_LOG. Picked off one at a time as work continues.

### Closed 2026-08-04 (second pass)

- **ADS sight-alignment** — focus derives a per-gun shift from the
  shared carry table so the iron pair lands on the eye line.
- **Forge front door** — SAVE 1-3 / LOAD 1-3 / RANDOMIZE as real rows
  on the soldier page. The full editor (category grid, turntable,
  per-piece armour) is still the large open item below.
- **Bow overdraw** — holding past full draw charges +15% to 1.6 s.
- **Mech front rebalance** — hull 1000 -> 600, front x0.15 -> x0.525:
  ten AWM or ~fourteen spears now open a chassis from the FRONT.
- **Three mech pads per map**, scattered; up to three chassis walking.
- **Zombie Extraction withdrawn from menus** (sim + tests intact).

### Closed 2026-08-04

| Item | How it closed |
|---|---|
| §3.4 low-ready + ready-up | 22° up-and-in at 0.6 m; ready-up is a real ζ=0.7 spring (a lerp cannot overshoot at all), sub-stepped so it stays bounded at 10 fps. 3 tests. |
| §4.1 vitals bars + armor cluster | 10-segment health bar + 4-pip armour cluster, `hud_vitals_style` setting. The pips read the SET's flat protection on foot and the power core in a chassis — `Fighter::armor` is zero for four of the five sets, so a pool-driven cluster would have sat empty all match. |
| §4.3 minimap rotate/scale | Rotate-with-facing + 25–100% scale, both persisted and both on settings rows. |
| §1.6 tests | Zero-instant-stop sweep (6 stop states × 4 entry paces), vertical-bob budget, lean-and-cut fuzz over 16 angles. |
| §4.7 context progress bar | Generic — one resolver feeding mech entry, mech exit and the extraction hold. |
| Killfeed rich formatting | Rebuilt as real rows (a single `Text` structurally cannot carry two name colours or a border). Side colours, 2px #B50000 local-player border, headshot/noscope/blind glyphs. |
| `ElasticMove.return_efficiency` | Was declared and read nowhere, so "the mech should feel it" was a comment. Now normalised against human tendon: a human still gets Rule 2's exact ×1.35, a mech gets 0.209. |
| counter-movement → jump | §C.3 names the jump and only the dodge had it. A crouch-jump now launches 6% harder through the same shared `counter_movement_bonus`. |

Also landed the same day, from the owner's own notes rather than the
backlog: team colour became **relative to the viewer** (allies white/gold,
enemies red/orange — six call sites had hardcoded Blue/Red literals and
would have shown your own team in enemy colours on a Red spawn), right-click
focus reaches **every** weapon in both cameras (the pipeline was fully built
and fenced off at one line), and **Shift walks** CS:GO-style with sprint
moved to Alt. Movement noise gained a middle tier in the process — ordinary
running was already silent, so a walk key would otherwise have bought
nothing.

## 2. Large, multi-session architectural (do not rush — BACKLOG.md)

- **20-segment mass-bearing rig** (Steps 0, 2–7 remain): real
  pelvis→lumbar→thorax trunk, clavicles, toe segments, mass-fraction /
  CoM / radius-of-gyration data driving spring stiffness. Touches every
  posing system in main.rs.
- **Mech visual + weapon-kit rebuild D.1–D.7**: "walking weapons
  platform" silhouette (today: scaled humanoid), 20 named swappable
  parts, part-by-part damage states. The gatling+autocannon CORE kit is
  built (§C); the silhouette isn't.
- **Mech hull-climbing** (#2): design + 6-item build checklist done
  (`research/mech-climb/DESIGN.md`); the build itself not started.
- **Forge editor UI**: today Forge is 3 hotkey save/load slots to a text
  file — the specced category grid / turntable / randomize UI is absent.
- **26-piece armour + 4-class system**: no existing code to extend;
  currently 5 whole-body presets.
- **Castle map**: content work (geometry), not code. The intro's CASTLE
  BAILEY / CASTLE GARDENS entries select layouts of the existing arena
  blockout, not a real castle.
- **Melee depth** (#4): parry, deflection, directional attack, stagger.
- **AI squad coordination** (#5): flanking, suppression, bounding
  overwatch; `jk_wall` morale exists to hook into.
- **Traversal** (#7): climb/vault/mantle — blocked on map metrics.
- **Full character customization** (§8.1): sliders, cosmetic variants —
  currently 2 flat color fields.

## 3. Blocked, with the named unblocker

| System | Blocker |
|---|---|
| Weapon material stack, wear maps, decals, image import | Zero texture pipeline beyond branding UI images — every world surface is flat color. Unblocker: mesh/material texture loading. |
| Advanced rendering | Depends on the above. |
| **Networking** (rollback/prediction/lag comp) | Zero networking deps; local-only. The deterministic sim + bit-identical replay is the right foundation, but no netcode exists. Scoreboard deliberately has no Ping column for this reason. |
| Swimming / ropes / ladders / fluids | No water volumes, no ropes, no muscle layer. |
| Grenade extra materials (mud/sand/ice…) | No such surface exists in any map. |

## What is NOT missing (commonly assumed otherwise)

- Mech presentation plan §A–§C: **done** (brace stance, entry/exit
  presentation, idle-life, visor view, gatling+autocannon).
- Assists, K/A/D/DMG scoreboard, death→killer-cam→spectate: **done**.
- Paged intro flow with branding: **done** (this session).
- Determinism/replay guarantee incl. bot mechs: **done and tested**.
