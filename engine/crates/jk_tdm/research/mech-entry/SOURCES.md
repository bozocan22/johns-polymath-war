# Mech entry sequence — source ledger

R&D Cycle 1, per `briefs/PROMPT_RND_CYCLE.md`. R4 depth floor: ≥1
tier-P source read end to end. This topic has no game-specific
peer-reviewed literature (staged-activation UX isn't a research field on
its own), so the anchor source is the closest real discipline that
studies staged, high-stakes, must-not-skip sequences under time
pressure: aviation checklist human-factors research — directly
applicable, since the brief's own stage list (cockpit open → climb-in →
harness → power-up → servo sync → gyro calibration → weapon
diagnostics → HUD boot) is structurally a startup checklist.

| ID | Tier | Type | Title | Author | Year | URL | Accessed | Status | What it gave us |
|---|---|---|---|---|---|---|---|---|---|
| S-01 | P | NASA contractor report | Human Factors of Flight-Deck Checklists: The Normal Checklist (NASA CR-177549) | Asaf Degani, Earl L. Wiener | 1990 | https://ntrs.nasa.gov/api/citations/19910017830/downloads/19910017830.pdf | 2026-08-01 | **UNREACHABLE (original scan)** — the NASA-hosted PDF is an unOCR'd microform scan; WebFetch returned raw binary with no extractable text layer, and no PDF-rendering tool (poppler/pdftoppm) is available in this environment to OCR it locally. **READ instead via a hosted transcription** (see S-01b) — flagged explicitly rather than silently substituted. |
| S-01b | P | hosted transcription of S-01 | same report, Yumpu-hosted viewer | same authors | 1990 | https://www.yumpu.com/en/document/view/39407192/human-factors-of-flight-deck-checklists-the-normal-skybrary | 2026-08-01 | READ (verified: contains a direct quote and specific, checkable incident references — Northwest Flight 255, Monan's ASRS analysis — not generic-summary language) | Two checklist philosophies: **Challenge-Response** ("Pilot A calls the checklist item from the printed list; pilot-B and pilot-A together verify... pilot-B calls the verified status") and **Do-List / Call-Do-Response** (one pilot calls items, another executes sequentially, no prior verification). Failure modes: **interruption** breaks reliance on environmental cues the pilot needs to resume correctly (Northwest 255); **memory-guided deviation** — pilots run the list from memory and only touch the card, skipping the actual read step; **short-cutting** — chunking multiple items together, which defeats the redundancy the list exists to provide; **distraction cascade** — the checklist itself competes for attention when workload peaks elsewhere (Monan/ASRS). Core design principle: build for realistic operation under load, not an idealized attentive pilot. |

## The one finding that drove the design decision below

**Interruptible, player-paced sequences are exactly the failure mode
this literature documents** (short-cutting, memory-guided skipping,
distraction cascade). BRIEF_VIII §7.6 already specifies mech entry as
**committed, no cancel** — this source is independent, real-world
evidence for why that's the right call, not just flavor text: a
sequence a player can interrupt or rush invites the same failures a
pilot checklist that can be short-cut invites. It also argues AGAINST
modeling entry as an interactive challenge-response (press a button per
stage) and FOR a **Do-List-style automatic sequence** — the system
executes every stage on a fixed timeline, no player action required or
possible mid-sequence, matching the "do-list" pattern the source
describes for time-critical procedures (engine start, secure
procedures) rather than the slower challenge-response pattern meant
for two-crew cross-verification this game has no equivalent of.

Quota: 1/1 tier-P read for this cycle (R4 depth floor - no breadth
quota applies). Cycle judged sufficient per R4: one source directly
answered the design question this cycle needed answered.
