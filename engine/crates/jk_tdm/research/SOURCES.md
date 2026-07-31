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
