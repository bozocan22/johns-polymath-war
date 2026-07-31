# Research source ledger

Rules this ledger enforces (from the brief):

- **R2 — no claim without a source row.** Every number that reaches a
  spec carries an inline `[S-nn]` tag pointing at a row here.
- **R3 — never invent a source.** A page that could not be reached is
  logged `UNREACHABLE` / `PAYWALLED` / `NO-TRANSCRIPT` and does **not**
  count toward quota. Writing a quote that was not read is the worst
  possible outcome — worse than missing quota entirely.
- **Extraction rule.** A source only *counts* if it yielded a number with
  units, a named mechanism, or a documented failure mode. Otherwise it is
  marked `SKIMMED` and does not count.

## Status: PARTIAL — quota NOT met

The brief calls for 16 counted sources per topic (>=4 primary, >=4 video
with timestamped quotes). **That quota is not met and this file does not
pretend otherwise.** What is below is what was actually retrieved.

Two honest constraints:

1. The brief document itself never arrived in the conversation — only a
   detailed *description* of it. So the exact search queries and source
   classes it specified were not available; the searches below are my
   reconstruction from that description.
2. No video transcripts were retrieved at all, so the video quota is at
   **0/4** for every topic. Timestamped quotes are therefore absent
   rather than fabricated.

---

## Topic 1 — First-person dynamics

| ID | Source | Type | Status | Extracted |
|---|---|---|---|---|
| S-01 | [Recoil — Call of Duty Wiki](https://callofduty.fandom.com/wiki/Recoil) | community-ref | COUNTED | Named two-channel split: **Visual Recoil** (weapon firing animation, cosmetic only, never affects true aim) vs **View Kick** (the view actually moving off the aim point). Documented failure mode: because visual recoil misaligns the sights without moving the bullet, players over-correct and *miss more*. |
| S-02 | [Recoil — Counter-Strike Wiki](https://counterstrike.fandom.com/wiki/Recoil) | community-ref | COUNTED | Aimpunch is small on view but much larger on shot placement — i.e. the two channels are deliberately scaled differently rather than being one value. |
| S-03 | [First-Person Weapons — s&box docs](https://sbox.game/dev/doc/assets/ready-to-use-assets/first-person-weapons) | engine-doc | SKIMMED | Asset/setup oriented; no numeric or mechanism detail extracted. Does not count. |
| S-04 | [Adding Recoil and Impact to the Weapon — gameidea](https://gameidea.org/2025/09/07/adding-recoil-and-impact-to-the-weapon-fps-series-part-4/) | tutorial | SKIMMED | Implementation walkthrough; no independent numbers extracted. Does not count. |

**Counted: 2/16. Primary: 0/4. Video: 0/4.**

### What this changes in `jk_tdm`: nothing — already correct

S-01/S-02 describe exactly the architecture this codebase already has,
which is worth recording as a *validation* rather than a gap:

- `sim.rs` owns the true deflection (`punch` / `punch_vel`, the CS-style
  integrate-then-decay model).
- `main.rs`'s camera shows `punch * RECOIL_SCALE * VIEW_RECOIL_TRACKING`
  — deliberately **45%** of the true deflection.
- The crosshair is screen-anchored and never moves, so impacts drifting
  off it is the skill expression.

That is S-01's two-channel split with the scaling S-02 describes. No
change made; the audit's finding for this subsystem was elsewhere (the
crosshair not reflecting minigun *heat*, since fixed).

---

## Topic 1b — Ballistics / grenade arcs  (sources supplied by the user)

The user did this research themselves and supplied it verified. Recorded
here with their own `SNIPPET-ONLY` discipline preserved — I have not
read the six snippet-only items either, so they stay uncounted rather
than being written up from a search result.

| ID | Source | Type | Status | Extracted |
|---|---|---|---|---|
| S-09 | [Analytical Ballistic Trajectories with Approximately Linear Drag](https://www.decarpentier.nl/ballistic-trajectories) (de Carpentier, Int. J. Computer Games Technology 2014) | **primary, peer-reviewed** | COUNTED | Approximates drag as linear to obtain a **closed-form** trajectory instead of stepping a sim. Solves launch velocity given: time-to-target, an intermediate waypoint (throwing *over* a battlement to a point behind it), arc height, impact angle, or **minimum energy** — the last being what humans naturally throw. Ships C++ snippets and an open-source Unity demo. |
| S-10 | [Level Design Book — blockout metrics](https://book.leveldesignbook.com/process/blockout/metrics) | reference | COUNTED | Cross-engine hard numbers. Unity player box 1.0 x 1.8 m, eye 1.5-1.7 m; Unreal 60 x 176 cm, eye 152 cm. Min hallway 2.0 m. Stairs 15 x 25 cm, 30-35 deg slope, landings every 12-16 steps. TF2 ranges: close <=256 u, medium <=1024 u, max safe drop 256 u. |

**SNIPPET-ONLY (do NOT count, not yet read):** [Weihs, GDC 2013 aim
assist](https://archive.org/details/GDC2013Weihs) (names magnetism /
centering / friction); [Brink's SMART traversal](https://gdcvault.com/play/1015930/Vault-Slide-Mantle-Building-Brink);
[Bournemouth procedural parkour thesis](https://nccastaff.bournemouth.ac.uk/jmacey/MastersProject/MSc24/02/ProceduralParkourandTraversalAnimationTechniques.pdf);
[Yoder, multiplayer level design](https://gdcvault.com/play/1025183/Level-Design-Workshop-The-Holy);
[Reitich on projectile prediction](https://sreitich.github.io/projectile-prediction-1/);
[Wagar on i-frames](https://critpoints.net/2017/07/25/how-iframes-augment-dodge-rolls/).

**Explicit negative result (worth as much as a positive one):** searches
for aim response curves and deadzone values return almost entirely
affiliate-SEO content quoting each other. No primary source was found,
so no numbers are carried forward for those.

### What S-09 changes here: the rule was already enforced, by accident

S-09's central warning is that most games step a physics sim to draw the
preview arc and then throw with *different* code, so the grenade does not
land on the shown line. This codebase already routes both through ONE
`grenade_tick`, and audit wave 2 independently caught the one weapon that
had drifted from that discipline — the bow's arc preview was flying at a
stale 52 m/s while the sim launched at 19-55 m/s. Fixed before this
source arrived; the source explains *why* it mattered.

Not adopted: the closed-form solver itself. This sim is a fixed-timestep
120 Hz integrator whose determinism is load-bearing (see R11 below), and
swapping in an analytical solution would change every existing throw's
trajectory and invalidate the golden-value tests. S-09's *solve-for-
launch* capability (arc height, impact angle, over-a-wall waypoints)
would be a genuine addition for a throw-assist or a bot's grenade aim —
recorded as a real opportunity, not silently skipped.

### R11 — determinism, now tested

The user's R11 requires that a seeded sim produce bit-identical throws.
Implemented as `a_thousand_identical_throws_land_bit_identically`: 1000
throws from one seed compared on RAW BITS (not float `==`, which would
let a NaN or -0.0/+0.0 pair pass), plus an assertion that
`predict_grenade`'s preview endpoint lands within 0.1 mm of the live
flight — the S-09 property, asserted rather than assumed.

## Topic 2 — Layered character creation

**Counted: 0/16.** No searches run. The layer design (L0 identity →
L1 physique → L2 kit → L3 cosmetic → L4 mode/role) and its genuinely
interesting question — the **commit boundary**, i.e. which layers freeze
at match start and what happens when a physique slider changes hitbox
volume — are recorded in the brief description but are **not yet
researched and not implemented**.

Blocking reality: this codebase has no class system and no per-piece
armour system (5 whole-body `ArmorSet` presets), so there is nothing for
a physique/hitbox commit rule to attach to yet. See
`handback/brief-ix/REPORT.md`.

---

## Topic 3 — Reload / weapon render / console

| ID | Source | Type | Status | Extracted |
|---|---|---|---|---|
| S-05 | [Mid-animation reload cancelling exploit — Killing Floor 2](https://steamcommunity.com/app/232090/discussions/1/154642447929189363/) | bug-report | COUNTED | Concrete failure mode: hold fire + tap bash **as the ammo counter updates**; the animation continues but no ammo is loaded. Confirms the exploit lives at the **ammo-commit frame**. |
| S-06 | [FPS Annoyances: Interrupting the Gun Reload Animation — ResetEra](https://www.resetera.com/threads/fps-annoyances-interrupting-the-gun-reload-animation.302852/) | discussion | COUNTED | Named mechanism: reload-cancel lets a weapon be swapped *after* ammo time has elapsed but *before* full reload time, without restarting on swap-back. **Titanfall / Apex pause the reload instead of cancelling it** — a documented alternative policy. |
| S-07 | [Reload — GTFO Wiki](https://gtfo.wiki.gg/wiki/Reload) | community-ref | SKIMMED | No transferable number extracted. Does not count. |
| S-08 | [Designing a Data-Driven Weapon System — gameidea](https://gameidea.org/2025/09/07/designing-a-data-driven-weapon-system-fps-series-part-2/) | tutorial | SKIMMED | Generic data/logic separation. Does not count. |

**Counted: 2/16. Primary: 0/4. Video: 0/4.**

### What this changes in `jk_tdm`: audited, no exploit found

Checked the real implementation against S-05/S-06:

- `try_reload` sets `reload_t = spec.reload_s` (sim.rs).
- The ammo commit is at the **END** of the timer: when `reload_t <= 0`,
  `ammo = mag.min(ammo + reserve)`, keeping chambered rounds.
- `switch_slot` sets `reload_t = 0.0` — a cancel that grants **no** ammo.

So the S-05 exploit shape (commit early, then cancel) **cannot occur
here**: there is no early commit to exploit. This codebase implements the
*cancel* policy, not the Titanfall/Apex *pause* policy (S-06) — that is a
design choice, not a bug, and is left alone.

The one thing worth flagging for a future pass: `switch_slot` discards
reload progress entirely, so swapping away at 95% of a reload and back
costs a full reload. That is consistent with the cancel policy and with
CS, so it is **not** being changed without a design decision.

---

## Not attempted

- **Gun material stack** (layered base/coating/wear/grime, channel
  packing, edge-wear masks, VRAM budget): this engine has **zero
  image-texture pipeline** — no `asset_server.load` of any image
  anywhere, confirmed by earlier audit. Every material is a procedural
  `StandardMaterial` colour. A material-stack spec would have nothing to
  attach to.
- **In-game console with player image import**: not built. Worth noting
  the brief's own instinct is right — a file-import surface needs its
  refusals specified *before* its success path (format whitelist, size
  ceiling enforced **before** decode, guarded decode, and path
  confinement so `../` cannot escape the import directory). Given there
  is no texture pipeline to import *into*, this is blocked upstream
  regardless.
