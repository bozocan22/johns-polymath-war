# aiming — SOURCES

| ID | Tier | Type | Title | Author / Studio | Year | URL | Accessed | Status | What it gave us |
|---|---|---|---|---|---|---|---|---|---|
| S-01 | P/V | GDC talk | Techniques for Building Aim Assist in Console Shooters | Nick Weihs / Insomniac (Resistance 3) | 2013 | https://archive.org/details/GDC2013Weihs (Vault 1017942) | 2026-08-01 | SNIPPET-ONLY | Snippet names the three canonical assist systems — magnetism, centering, friction — plus camera acceleration and deadzoning. THE talk for this topic; NOT COUNTED until watched with timestamps. |

## Added 2026-08-01 (Sonnet 5 pass) — one real primary source, fully read

| ID | Tier | Type | Title | Author / Studio | Year | URL | Accessed | Status | What it gave us |
|---|---|---|---|---|---|---|---|---|---|
| S-02 | P | peer-reviewed paper (CHI 2014) | The Effectiveness (or Lack Thereof) of Aim-Assist Techniques in First-Person Shooter Games | Vicencio-Moriera, Mandryk, Gutwin (U. Saskatchewan), Bateman (UPEI) | 2014 | https://rodrigov.ca/wp-content/uploads/2016/03/aim-assist-cameraReadychi2014-v8-final.pdf | 2026-08-01 | **READ** (full PDF, all 10 pages, verified against the primary text — see fabrication note below) | Three studies (target range / full walkthrough / two single-factor variants) of five aim-assist techniques in a custom UDK FPS. Real parameters, real stats, real conclusion about WHY two techniques work and three don't. Full extraction below. |

### ⚠️ Methodology note: caught a fabricated summary before it entered the ledger

`WebFetch`'s first pass at this PDF (prompted for a detailed quantitative extraction)
returned a *plausible-sounding* summary: "~87% hit rate", "2.5 degrees visual angle
[for bullet magnetism]", "1.8× standard reticle diameter", "0.7 normalized units",
"20 participants". **None of these numbers appear anywhere in the actual paper.**
The tool had saved the source PDF locally; reading it directly (not re-summarizing
it) showed the real technique names, real formulas, and real statistics are
completely different from what the auto-summary reported. Per R3 this would have
been the single worst outcome of this research pass — a fabricated source treated
as `READ` — caught only because the raw file was verified against, not because the
summary looked wrong on its face. It looked *entirely plausible*. **Lesson for the
rest of this ledger: a WebFetch/tool summary is not itself a source; only the
primary text it was drawn from is, and it must be checked, not trusted, whenever
the extraction will carry a number into a spec.**

### The five techniques, exactly as implemented in the paper (UDK/UnrealScript, Levels 1–10)

| Technique | Mechanism | Formula / parameter | Fitts's-law category |
|---|---|---|---|
| **Target Lock** | Snaps player pitch/yaw toward nearest target's head on button press | Lock-on time: **0.5 s at Level 1** (~7°/tick) down to **0.15 s at Level 10** (~12°/tick), for a stationary target 90° away | amplitude reduction |
| **Bullet Magnetism** | Bends the fired-bullet vector toward the nearest target's body (or head, if crosshair already on-target) *after* the trigger is pulled, before hit-collision resolves | Activation range = **160 UDK units × Level** (1≤Level≤10) | post-hoc correction (not a classic Fitts category — the paper notes this) |
| **Area Cursor** | Widens the hit-test volume from a zero-extent trace to a rectangle; nearest-to-crosshair-center target inside it is hit | Radius = **10 px + (5 px × Level)** | width increase |
| **Sticky Targets** | Lowers control-to-display (CD) ratio while the crosshair is over a target — a "pseudohaptic" stickiness | Movement divided by **(Level + 4)** | width increase in motor space |
| **Target Gravity** | Crosshair is pulled toward a weighted average of visible targets' positions, only while the player moves the mouse toward that pull | `w_i = G / (|p0−pi|² + 1)`; warped position is the weight-averaged target position; only engages if input direction agrees with pull direction | amplitude reduction |

### The one finding that matters most for this codebase

**Bullet Magnetism and Area Cursor both worked well across nearly every condition.
Target Gravity and Sticky Targets performed poorly almost everywhere.** The paper's
own explanation (Discussion, p.9) is the load-bearing part:

> "It is interesting that both bullet magnetism and area cursor alter the targeting
> process only *after* the player has carried out their aiming action... It is
> possible that magnetism and area cursor worked well because they caused less
> conflict with the user's control actions."

Gravity and Sticky both move the crosshair **during** the player's own aiming
motion, fighting the player's hand in real time. Bullet Magnetism and Area Cursor
only act **after** the trigger is pulled, correcting the *outcome* without ever
touching the player's view. Target Lock was the single most effective technique on
raw numbers, but was rated most perceptible by a wide margin (Perception Rating,
Table 1) — players *feel* their view being taken over, and Table 1 / S2 shows this
is exactly the technique intrusive enough to break trust in competitive settings.

**This is an empirical, peer-reviewed, statistically-tested version of this
project's own R8 ("player intent wins... within one frame")** — the paper found
the same thing from the opposite direction: assist techniques that touch the
player's own control motion feel bad and perform worse than ones that correct only
after control has already been exercised. `jk_tdm` already implements exactly this
shape for recoil (§ existing S-01/S-02 above: true deflection lives in `punch`,
the *visible* camera only shows 45% of it, the crosshair never moves) — this paper
is direct evidence that the same principle should govern any future aim-assist
work: **correct the bullet, never the player's crosshair mid-motion.**

### Numbers worth carrying forward

| Value | What it measures | Conditions | Source |
|---|---|---|---|
| 0.5 s → 0.15 s | Target Lock lock-on time, level 1 → level 10 | stationary target, 90° away | S-02 |
| 160 UDK units × Level | Bullet Magnetism activation range | Level 1–10 | S-02 |
| 10 px + 5 px×Level | Area Cursor radius | Level 1–10, 1920×1080 @ 60 Hz reference display | S-02 |
| movement ÷ (Level+4) | Sticky Targets CD-ratio divisor | Level 1–10 | S-02 |
| n=12 / n=16 / n=15 | participants, Studies 1 / 2 / 3 | mixed novice–expert FPS players | S-02 |

### Explicit negative result — the paper does NOT give us

No frame-rate-independent numbers (this is a 2014 UDK study at a fixed 60 Hz
reference monitor), no console/gamepad-specific deadzone data (mouse-only study —
"Logitech G5 gaming mouse"), and no numbers for moving targets under player motion
(S1 targets were stationary; S2/S3 targets moved but the player's own motion
wasn't systematically varied). None of these gaps are papered over with an invented
number.

## Explicit negative result (carried from the master brief, re-confirmed)

Searches for aim response curves (linear / exponential / dynamic
reverse-S) and controller deadzone values return almost entirely
affiliate-SEO content quoting each other's numbers. No primary source
found. Every such page is tier X. No curve or deadzone number is carried
into this project until an engine doc, input-system source file, or a
talk provides one.

## Applied to this codebase

Mouse-only today: raw delta, no smoothing, no acceleration (asserted in
code comments and by the zoom-sensitivity monitor-distance match, which
now tracks the player's CHOSEN hip FOV). Aim assist is a controller
feature; nothing to wire until controller input exists. The three Weihs
mechanism names are recorded so the eventual implementation starts from
the canonical decomposition rather than folklore.

## Quota: 1/12 counted (P: 1/3, V: 0/3). Still needs the video pass (S-01, Weihs) —
GDC Vault access is the blocker, not effort; no transcript-only route was found.
