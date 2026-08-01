# R&D Cycle 1 report — mech entry sequence

Per `briefs/PROMPT_RND_CYCLE.md` Section 4.

## 1. What was built

A deterministic, SIM-layer staging function (`mech_enter_stage`,
`mech_enter_stage_for` in `sim.rs`) that names eight sequential stages
— CockpitOpen, ClimbIn, Harness, PowerUp, ServoSync, GyroCalibration,
WeaponDiagnostics, HudBoot — across the existing 1.6s committal entry
window (`MECH_ENTER_S`), pure functions of elapsed time so they are
replay-identical and cannot desync anything. This turns the entry
window from an opaque timer into eight named, individually-addressable
moments a render/audio/HUD layer can react to.

## 2. Evidence

[S-01b] (Degani & Wiener, NASA CR-177549, read via a hosted
transcription after the original NASA-hosted PDF turned out to be an
unOCR'd scan this environment has no tool to read) gave two real
findings that shaped the design: the Do-List vs Challenge-Response
checklist-philosophy distinction, and documented failure modes
(interruption, memory-guided skipping, short-cutting, distraction
cascade) that all stem from *interruptible* or *player-paced*
sequences. This is independent, real-world evidence for BRIEF_VIII
§7.6's existing "committed, no cancel" rule, and it argues specifically
for a Do-List-style automatic sequence (the system runs every stage on
a fixed timeline) over an interactive one — which is what got built.

## 3. Tests

Command: `cargo test --release -p jk_tdm`
Before: 137 passed (Section H's count)
After: **138 passed**, 0 failed, 2 ignored

New test `mech_entry_stages_are_monotonic_and_gapless` — fails on the
pre-change code (the function didn't exist). Covers: a fine-grained
sweep of the whole window visits all 8 stages exactly once each in
order (no skip, no repeat-after-advance); boundary clamping (negative
elapsed → first stage, elapsed far past the window → last stage, no
panic); a REAL entry through `step()`'s per-tick countdown confirms
CockpitOpen is active at tick 0 and HudBoot is active in the window's
last tick; and three "not entering" cases return `None` correctly —
never a mech, mid-EXIT (which reuses the same timer field for the
power-down countdown and must not report an entry stage), and after
the window closes.

## 4. Capture

**None. Stated plainly, per R11.** This cycle built the SIM-layer
staging function only — it is not wired to any render, audio, or HUD
system, so there is nothing yet for a player to see and nothing a
capture could honestly show beyond "the existing entry capture looks
unchanged," which would be misleading framing for what was actually
built. R1 ("done means it appears in a capture from the launched
build") is deliberately NOT claimed satisfied for this cycle's
deliverable — the stage function is real, tested, and correct, but not
yet "done" in R1's sense.

## 5. Rejected alternatives

- **Interactive challenge-response entry** (player confirms each
  stage) — rejected on [S-01b]'s own evidence: this pattern exists for
  two-person cross-verification under low time pressure, and this
  game has no second crew member; it would also reopen exactly the
  interruption/short-cut failure modes the source documents, which
  §7.6's "no cancel" rule already correctly avoids.
- **Sharing the render layer's existing entry animation timing as the
  source of truth** — rejected because that would make stage boundaries
  a presentation-layer fact instead of a sim-layer one, breaking
  replay-identical staging for no benefit; the sim already owns
  `mech_transition_t`, so it's the natural single source of truth.

## 6. Backlog delta

- Item #1 (mech entry sequence): **partially done** — sim-side staging
  built and tested; render/audio/HUD wiring is the concrete remaining
  work, added below as a new row rather than silently left implicit.
- New item added: **#16, mech entry stage presentation** (High) — wire
  `mech_enter_stage_for` to the viewmodel/third-person rig (visor
  flicker, servo audio per stage, camera transition on HudBoot) and
  capture it. Attaches directly to this cycle's output; blocked on
  nothing, just unstarted.
- Item #6 (mech operation feel) inches forward incidentally: Section H
  (same session, prior to this cycle) added the power-stride heat
  budget; still not extended to internal damage/eject as #6 envisions.

## 7. What was not done, and why

- No render/audio/HUD hookup (see Capture, above) — this cycle's scope
  was the sequencing logic itself; presentation is real, separate work
  queued as backlog #16.
- The original NASA PDF [S-01] could not be read directly — no
  PDF-rendering/OCR tool is available in this environment. Substituted
  a hosted transcription of the same document, flagged explicitly in
  the ledger rather than silently treated as equivalent.
- Backlog items #2 (infantry vs. mech) and #3 (grenade surfaces) were
  not started this cycle — #1 was chosen as explicitly ranked highest
  and directly continues this session's existing mech-dynamics work.

## Rotating codebase review (Section 5) — categories 1-4 this cycle

Quick pass only (budget for this cycle was mostly research+build):
1. *Duplicated logic* — none introduced; `mech_enter_stage` is the only
   place stage-timing math exists.
2. *CPU hot paths* — `mech_enter_stage_for` is O(1), called nowhere yet
   (unwired), zero cost today.
3. *Allocation/cache* — no allocations; stages are a `Copy` enum in an
   8-element `const` array.
4. *Scalability at crowd counts* — N/A, at most one fighter per team is
   ever mid-entry at a time; not a per-crowd-member cost.

No findings from this pass worth a backlog row.
