# What is missing / not built yet — 2026-08-04

Compiled for the user from `BACKLOG.md`, `THOR_LOG.md`'s ranked findings,
and session knowledge. Ordering inside each tier is Thor's ranking rule:
an item moves up when its blocker clears, not when it becomes interesting.

## 0. Needs the USER (nothing code-side can proceed)

| What | Where it goes | What happens today |
|---|---|---|
| The 3 branding PNGs (key art, wordmark, emblem) | `engine/assets/branding/` as `key_art.png`, `wordmark.png`, `emblem.png` | All branding code ships and degrades gracefully — splash, menu art, emblem placements simply don't render until the files exist. See `engine/assets/branding/README.md` |

## 1. Small, one-session buildable (ranked, next up)

1. **§3.4 low-ready + ready-up** — muzzle obstruction dip (22°) near
   walls, with overshoot on ready-up. Pure presentation.
2. **§4.1 vitals bars + armor cluster** — HUD health presented as
   segmented bars with an armor pip cluster (numbers exist; the visual
   language doesn't).
3. **§4.3 minimap rotate/scale options** — settings entries; minimap
   itself works.
4. **§1.6 missing tests** — zero-instant-stop sweep, vertical-bob
   budget, lean-and-cut ordering. Mechanics exist; falsifiable coverage
   doesn't.
5. **§4.7 context progress bar** — the killer-cam/spectate flow shipped;
   the generic "channeling" progress bar it was specced beside didn't.
6. **Killfeed rich formatting** — team colors + border + modifier icons
   (headshot etc.). Assists already track and display.
7. **Remainder of THOR_LOG's 97 double-confirmed small gaps** — HUD
   toasts, crosshair settings, dead-field wirings
   (`ElasticMove.return_efficiency`, counter-movement→jump), numeric
   tunings. Picked off one at a time as work continues.

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
