# BRIEF XII-A — HUD CONSOLIDATION PASS

**Issued by the owner, 2026-08-11**, after seeing the first `BRIEF_XII`
captures. This is a **cleaning and consolidation** pass, not a redesign.

> Keep the mech UI simple, clean, and optimized, with a visual feel
> inspired by **Call of Duty + Battlefield**, but adapted specifically for
> a futuristic mech game.

## Owner's goals, verbatim

- Make the player HUD feel **military, tactical, and professional**.
- Keep it minimal — only show information the player actually needs during
  combat.
- **Avoid duplicated information** or unnecessary UI elements.
- Prioritize readability during fast mech combat.
- Use **subtle** futuristic/mechanical styling rather than excessive sci-fi
  graphics.

### Mech HUD
- **Show HEAT only once. Remove the duplicated heat displays.**
- Keep the most useful heat indicator in a clear, easy-to-read position.
- **Fold the old weapon-strip UI into the new HUD** instead of having two
  separate systems.
- Make the weapon/ammo information compact and readable.
- **Ensure HUD elements remain visible against both dark and pale
  environments.**
- Use subtle transparency, outlines, or contrast rather than making the UI
  visually heavy.

### Visual direction
> Call of Duty's clean combat HUD + Battlefield's military presentation +
> futuristic mech instrumentation.

The player should immediately understand: health/armor · heat · current
weapon · ammo · important combat warnings · minimal targeting information.
**Everything else stays hidden unless needed.**

### Optimization
- Reuse existing UI components wherever possible.
- **Do not create multiple systems that display the same information.**
- Keep the implementation lightweight and modular.
- Avoid unnecessary animations, effects, or expensive UI rendering.
- Preserve existing mech functionality; this pass is cleaning,
  consolidating and optimizing the player-facing HUD.
- **Do a final visual pass** to make sure there are no duplicated heat
  indicators, overlapping elements, washed-out text, or legacy HUD pieces
  still visible.

---

## 0. THE EXACT DUPLICATES, LOCATED 2026-08-11

Verified in the source at `1713fe1`, and visible in
`handback/brief-vii/medic/06-plasma-firing.png`.

### HEAT appears THREE times on one mech frame

| # | Where | Source |
|---|---|---|
| 1 | Big numeral, bottom right (`40` + `%`) | `hud.rs` ammo corner |
| 2 | `HEAT 40%` row in the systems column | `hud.rs` `systems_lines` |
| 3 | `PLASMA BOW  40%` in the legacy strip | `main.rs:5444`, `weapon_strip` |

### HULL appears TWICE

| # | Where | Source |
|---|---|---|
| 1 | Big numeral, bottom left (`210`) | `hud.rs` vitals corner |
| 2 | `HULL 210` row in the systems column | `hud.rs` `systems_lines` |

### The legacy systems still alive in `main.rs`

- `weapon_strip` (`main.rs:5359`) + `WeaponStripCell` (`:5247`, spawned
  `:16711`) — the four slot tiles, top right. **This is the washed-out
  element**: it is semi-transparent and fades to α 0.45 after 4 s idle, so
  against a pale wall it is unreadable. It is also the third heat display.
- `shield_readout` (`main.rs:5268`) + `ShieldReadout` (`:5266`, spawned
  `:16730`) — BARRIER / GUARD line beneath the strip.
- The old `hud_system` heat formatters at `main.rs:22858`, `:22992`,
  `:22998`, `:23001`, `:23010`.

## 1. THE TARGET LAYOUT

**Two big numerals carry the primary state; nothing repeats them.**

```
  HUMAN                              MECH
  ┌──────────────────────┐           ┌──────────────────────┐
  │ urgent line (only    │           │ urgent line          │
  │ when it matters)     │           │  ⌐ mech framing ¬    │
  │                      │           │                      │
  │        (clear)       │           │        (clear)       │
  │                      │           │      MOUNTS + LOCK   │
  │ 100          30 /120 │           │ 210            40 %  │
  │ ▬▬▬▬▬▬       M4A1    │           │ ▬▬▬▬▬       PLASMA   │
  └──────────────────────┘           └──────────────────────┘
```

- **Bottom left = survivability.** Health for a human, HULL for a mech.
  One number. The systems column must not repeat it.
- **Bottom right = the firing resource.** Ammo for a human (big/small
  pair), **HEAT for a mech** — heat is what gates a mech's trigger, so it
  earns the ammo slot. Weapon name beneath, small. **This is the one heat
  indicator.**
- **The systems column becomes the folded weapon strip**, and carries only
  what is not already on screen: the mount list with the selected one
  marked, `LOCK` when tracking, and `BARRIER`/`GUARD` **only when a shield
  is actually up**. No HULL row. No HEAT row.
- **Human POV keeps no permanent weapon list.** CoD and Battlefield both
  show the current weapon and its ammo, not the whole loadout. The slot
  list is *"hidden unless needed"* — show it as a brief transient on weapon
  switch, or not at all.

## 2. CONTRAST — the washed-out failure

The legacy strip fails because it is low-alpha text with no backing over a
scene that can be pale sand or dark interior.

**Rule:** every permanent reading gets either a plate behind it or a dark
outline/shadow. Never bare low-alpha text. The owner's words: *"subtle
transparency, outlines, or contrast rather than making the UI visually
heavy."*

The check is not "does it look fine in one screenshot" — it is **the same
element photographed against both a pale wall and a dark interior.** Both
frames, or the claim is not made.

## 3. OPTIMIZATION

Deleting `weapon_strip` and `shield_readout` removes two per-frame systems
and their queries. Do not replace them with two new ones — fold their
content into the HUD's existing update pass. Net systems count should go
**down**, and the report must state the before/after number.

## PROOF

Rule 8: every claim here is visual.

- Mech frame with **exactly one** heat indicator visible — and say where
  the other two went, by symbol.
- Mech frame against a **pale** environment and against a **dark** one,
  same elements, both readable.
- Human frame showing no permanent weapon list and no legacy strip.
- The transition pair — before boarding, after boarding.
- **Open every PNG.** A capture not looked at is not evidence; this
  project has already shipped frames whose labels did not match their
  pixels.

Rule 12: any test kept or added must be mutation-proven. A HUD test that
reads the same constant the HUD reads cannot fail.
