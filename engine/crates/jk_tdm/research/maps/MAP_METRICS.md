# MAP_METRICS — the authoritative metric table for `jk_tdm`

Written 2026-08-10 by **TOTO**, closing `TRV-0149`. This is the file
`TRV-0051` (traversal) and `TRV-0180` (ledge bands) were blocked on.
**Section 9 answers those two rows directly** — a builder who only needs
the unblock can read Section 9 and stop.

This is a **synthesis**, not a research cycle. The literature was already
read and lives in `maps/SOURCES.md` (S-01, tier P, READ) and
`vertical-maps/SOURCES.md` (VM-01…VM-14). What this file adds is the
conversion: every cross-engine number re-expressed against **this game's
body, this game's physics and this game's shipped maps**, with the
arithmetic shown so it can be re-checked, and with a verdict stated
wherever a convention and the shipping code disagree.

---

## 0. How to read this file

Every number carries a label. **They are never blended.**

| Label | Means |
|---|---|
| **MEASURED** | I read it in primary text: a Rust constant in `engine/crates/jk_tdm/src/`, or a line of test output I ran today, or a verbatim quote from a source I fetched. File and line given. |
| **DERIVED** | Arithmetic on MEASURED values. The full working is shown inline. If you cannot check the arithmetic from what is written here, that is a bug in this file. |
| **ASSUMED** | A judgement call by me. Nothing downstream may treat these as evidence. Every one is flagged in-line and listed again in §10. |

Two more conventions:

- **FINAL metres.** Every length in this document is in the sim's output
  space — after `MAP_SCALE`. See §2 for why that distinction is a trap.
- **Precision ceilings are stated where they exist.** Where a number
  comes from a sampled instrument, the sampling resolution is given and
  nothing finer may be claimed from it (§6).

### Verification record for this pass

Everything in §1–§8 marked MEASURED was checked against source **today**,
not taken from the `SOURCES.md` summary table. Two things had drifted:

1. **`max_unobstructed_sightline` baselines in `SOURCES.md` are stale by
   exactly the map expansion.** See §6.1. Corrected here and in
   `SOURCES.md`.
2. **`BODY_HEIGHT`, `EYE_REL` and `BODY_RADIUS` had NOT drifted** —
   1.78 m, 1.62 m and 0.34 m are current. `BODY_HEIGHT` carries a doc
   comment recording a reverted 5% shrink (`sim.rs:48-62`); the revert
   held.

The one external source was re-fetched and re-read today rather than
trusted from the ledger, because two of the figures the ledger attributes
to it (30–35° stair slope, landings every 12–16 steps) did not appear in
my first extraction pass. **A second targeted fetch reproduced both
verbatim, including the source's own derivation** (`arctan(7/11) = 32
degrees`). S-01 is confirmed clean. Recording this because this repo has
a fabrication in its history and "the ledger said so" is not a check.

---

## 1. THE BODY YOU ARE BUILDING FOR

Every metric in this file is ultimately a multiple of this table.

### 1.1 The soldier — MEASURED

| Quantity | Value | Source |
|---|---|---|
| `BODY_HEIGHT` | **1.78 m** | `sim.rs:62` |
| `BODY_RADIUS` | **0.34 m** → collision **diameter 0.68 m** | `sim.rs:47` |
| `EYE_REL` | **1.62 m** | `sim.rs:46` |
| `CROUCH_HEIGHT` | **1.15 m** | `sim.rs:65` |
| `ROLL_HEIGHT` | **0.95 m** | `sim.rs:232` |
| `STEP_UP` | **0.55 m** | `sim.rs:162` |
| `GRAVITY` | **18.0 m/s²** | `sim.rs:161` |
| `JUMP_SPEED` | **7.4 m/s** | `sim.rs:202` |
| `JUMP_COUNTER_BONUS` | **+6%** on launch speed, grounded crouch-jump only | `sim.rs:217`, applied `sim.rs:8500-8505` |
| `MOVE_SPEED` / `SPRINT_SPEED` | **4.8** / **6.6 m/s** | `sim.rs:66-67` |
| `WALK_SPEED_MULT` | **0.52** → walk = 2.496 m/s (DERIVED) | `sim.rs:71` |

The body is a **cylinder**, not a box: `ray_vs_cylinder` for hits
(`sim.rs:10383`), a radial push-out for collision (`sim.rs:9280-9304`).

### 1.2 The three piloted chassis — MEASURED constants, DERIVED sizes

`chassis_scale()` (`sim.rs:4889`) is the single multiplier;
`height()`, `radius()`, `step_up()` and `mech_jump_speed_of()` all read
it, so a chassis is one number.

| Chassis | scale | height | radius / width | `step_up()` |
|---|---|---|---|---|
| Soldier | 1.00 | 1.78 m | 0.34 / **0.68 m** | 0.550 m |
| `ScoutMech` | 1.05 | 1.869 m | 0.34 / **0.68 m** ⚠ | 0.550 m ⚠ |
| `RobotSuit` (heavy) | 1.70 | **3.026 m** | 0.578 / **1.156 m** | 0.935 m |
| `RoyalMech` | 1.87 | **3.329 m** | 0.636 / **1.271 m** | 1.029 m |

*DERIVED:* height = 1.78 × scale; radius = 0.34 × scale;
step_up = 0.55 × scale. Royal scale = `MECH_SCALE 1.7 × ROYAL_MULT 1.10
= 1.87` (`sim.rs:5871`, `sim.rs:4849`).

⚠ **MEASURED asymmetry, and it is a real one.** `radius()`
(`sim.rs:3736`) and `step_up()` (`sim.rs:3686`) are both gated on
`in_heavy_mech()`. The Scout is 1.05× **tall** and plain soldier **wide**
with a plain soldier **step-up**. The code says so deliberately
(`sim.rs:3733-3735`: "widening it here would be a nav and hit-test change
nobody asked for"). **For map purposes the Scout is a soldier with a
taller head and one extra jump.** Do not size an aperture for it as if it
were a mech.

### 1.3 Against the cross-engine band — S-01, verified verbatim today

Source: *Blockout metrics*, The Level Design Book (`maps/SOURCES.md`
S-01, tier P, **READ** — re-fetched 2026-08-10, quotes below are the
page's own strings). Imperial converted at 0.0254 m/in; conversions are
DERIVED.

| Engine | Player box (w × h) | Eye | eye / height |
|---|---|---|---|
| Unity | 1.0 × 1.8 m (or 1.0 × 2.0) | 1.5–1.7 m | 0.833–0.944 |
| Unreal | 0.60 × 1.76 m | 1.52 m | 0.864 |
| Quake / Source | 32 × 72 in = 0.813 × 1.829 m | 64 in = 1.626 m | 0.889 |
| US real-life (source's own reference row) | 20 × 69 in = 0.508 × 1.753 m | 66 in = 1.676 m | 0.957 |
| **`jk_tdm`** | **0.68 × 1.78 m** | **1.62 m** | **0.910** |

**VERDICT — the cross-engine convention transfers without rescaling.**
This game's soldier is inside the band on all three independent axes:
width 0.68 sits between Unreal's 0.60 and Quake's 0.813; height 1.78
between Unreal's 1.76 and Quake's 1.829; eye 1.62 within a centimetre of
Quake's 1.626, and the eye/height ratio 0.910 sits mid-band. **This is
the load-bearing result of the whole file** — it is why every derived
architecture metric below is legitimate rather than a coincidence of
units. `SOURCES.md` asserted this; here is the arithmetic behind it.

### 1.4 One drift to know about: "eye height" is not one number

MEASURED, `sim.rs` / `main.rs`. `EYE_REL` is consumed in **two forms**:

- **Bare** — `f.pos[1] + EYE_REL` → always 1.62 m above the feet
  regardless of stance (`sim.rs:11374, 11808, 13148, 13515`).
- **Stance-clamped** — `EYE_REL.min(f.height() - 0.12)` → 1.62 m
  standing, **1.03 m crouched**, **0.83 m rolling**
  (`sim.rs:9555`, `main.rs:20492`, `main.rs:21168`).

And one site clamps with `- 0.1` instead of `- 0.12` (`sim.rs:12640`),
giving 1.05 m crouched. **A 2 cm inconsistency, low severity, reported
because it is real.** The 0.59 m standing-vs-crouched split is not an
inconsistency — the bare form is used where stance is not in play — but a
builder measuring a sightline must know which form the instrument used.
**`max_unobstructed_sightline` uses the bare form at a fixed absolute
y = 1.62** (`sim.rs:9643`), which is the root of the limitation in §6.2.

---

## 2. MAP EXTENTS, AND THE `MAP_SCALE` AUTHORING TRAP

### 2.1 The five shipped maps — MEASURED + DERIVED

`MAP_SCALE = 1.25` (`sim.rs:35`) is applied to `half` at `sim.rs:1535`.

| Map | authored `half` | **final half** | **playable square** | source |
|---|---|---|---|---|
| Arena | `ARENA_HALF` 34.0 | **42.5 m** | 84 × 84 m | `sim.rs:45, 1139` |
| Bailey | 40.0 | **50.0 m** | 99 × 99 m | `sim.rs:1196` |
| Gardens | 38.0 | **47.5 m** | 94 × 94 m | `sim.rs:1278` |
| Battlefield | 200.0 | **250.0 m** | 499 × 499 m | `sim.rs:1365` |
| Cliffhold | `CLIFFHOLD_HALF_M/1.25` | **300.0 m** | 599 × 599 m | `sim.rs:1661, 1499` |

*DERIVED:* final half = authored × 1.25. Playable square = `2·half − 1`,
because bodies are clamped to `[−half+0.5, half−0.5]` (`sim.rs:9281`).
Cliffhold pre-divides so it lands on exactly 300.

**Spawn rows sit at z = ±(half − 2.5)**, and `SPAWN_CLEAR_M = 9.0`
(`sim.rs:43`) is the band kept clear of furniture around them
(`sim.rs:1572-1579`). MEASURED. **Nothing may be authored inside 9 m of
either z edge** or the expansion pass will bury a spawning fighter.

### 2.2 The trap — how to author coordinates

MEASURED, `sim.rs:1509-1525` and `sim.rs:1712-1720`.

The expansion **moves box centres and leaves extents alone**, on purpose:
the gaps grow, the cover does not. **Height never scales at all** —
"waist-high is waist-high" is a gameplay contract wired into crouch, the
hit bands and the step-up.

The corollary is a bug class, and it is stated in the code because
Cliffhold hit it: **two slabs that abut before the expansion end up a
quarter of their centre distance APART after it.** On furniture that is
a wider gap. On a mountain assembled from overlapping slabs it is a crack
you fall eighteen metres down.

> **RULE (MEASURED, already shipped): author a vertical map in FINAL
> metres and divide by `MAP_SCALE` on the way into the box list.** This
> is what `build_cliffhold`'s `slab` helper does (`sim.rs:1765`), and the
> round trip is the identity on centres, so an authored centre IS its
> final sim coordinate. Designing in the output space removes the bug
> class rather than requiring vigilance against it.

---

## 3. HORIZONTAL METRICS — corridors, apertures, cover

### 3.1 Minimum corridor width

S-01's minimum-hall figures, and the ratio each engine holds against its
own player width:

| Engine | min hall | player width | ratio |
|---|---|---|---|
| Unity | 2.00 m | 1.00 m | 2.000 |
| Unreal | 1.50 m | 0.60 m | 2.500 |
| Quake / Source | 64 in = 1.626 m | 32 in = 0.813 m | 2.000 |
| US real-life | 48 in = 1.219 m | 20 in = 0.508 m | 2.400 |

**The width ratio is the stable transfer, and I checked that it is.**
Ratio spread 2.000–2.500 (1.25× across four engines). The same figures
against player *height* spread 0.696–1.111 (1.60×) — nearly half again as
noisy. So transfer on width. DERIVED.

**`jk_tdm` minimum corridor, DERIVED:** 0.68 m × 2.000–2.500 =
**1.36 – 1.70 m** for a soldier.
Cross-check by the height ratio: 1.78 × 0.696–1.111 = 1.24–1.98 m. The
two methods intersect at **1.36–1.70 m**, so the figure is robust under
either transfer.

**ASSUMED (mine): use 1.70 m, the top of the band.** A corridor in a TDM
map is a place people fight, not merely walk, and the top of the band
costs nothing. Flagged as a judgement call, not evidence.

**Per chassis, DERIVED** (same 2.000–2.500 ratio on the chassis width):

| Body | width | min corridor |
|---|---|---|
| Soldier / Scout | 0.68 m | 1.36 – 1.70 m |
| Heavy `RobotSuit` | 1.156 m | 2.31 – 2.89 m |
| `RoyalMech` | 1.271 m | 2.54 – 3.18 m |

**Not currently binding on any shipped map** — nothing in `jk_tdm` has a
corridor that narrow. Recorded for the day interiors exist.

### 3.2 Doorways — where the convention LOSES, and why

S-01 door dimensions, and the ratio to the engine's own player width:

| Engine | door (w × h) | door w / player w |
|---|---|---|
| Unity | 1.25 × 2.5 m | 1.250 |
| Unreal | 1.10 × 2.20 m | 1.833 |
| Quake / Source | 56 × 112 in = 1.422 × 2.845 m | 1.750 |
| US real-life | 36 × 80 in = 0.914 × 2.032 m | 1.800 |

**DERIVED body-fit door widths:** soldier 0.68 × 1.250–1.833 =
**0.85 – 1.25 m**; heavy 1.156 × … = **1.45 – 2.12 m**; Royal 1.271 × … =
**1.59 – 2.33 m**.

**Now the two reasons that table is nearly useless here.**

**(a) Door HEIGHT cannot be expressed by this engine at all.** MEASURED,
`sim.rs:1741-1748`: cover is an `Aabb` with a top and no underside; the
support rule stands a body on the tallest reachable top and pushes it out
of everything else. **There are no lintels, arches, tunnels, overhangs or
bridge spans anywhere in `jk_tdm`, and there cannot be.** A "door" in
this game is a **gap between two wall slabs, unbounded above**. The
entire door-height column above is inapplicable. Saying so is more useful
than transcribing it.

**(b) The shipped map ignores the width column by an order of
magnitude.** MEASURED from `build_cliffhold`:

| Aperture | width | source |
|---|---|---|
| Castle gate (south curtain wall gap) | **24.0 m** | `sim.rs:1908-1909` (walls end at −18 and 6) |
| Keep doorway | **14.0 m** | `CH_KEEP_DOOR_X = [−27, −13]`, `sim.rs:1688` |
| North postern | **12.0 m** | `sim.rs:1918-1919` (walls end at −6 and 6) |

That is 9.4× to 15× the Royal's derived body-fit minimum. The code's own
justification for the keep door is body fit — "fourteen metres, which
admits a 1.16 m-wide chassis with room to spare" (`sim.rs:1943`) — which
does not explain the other 12.8 m.

**VERDICT: for `jk_tdm`, aperture width is set by BOT NAVIGATION, not by
body width. The cross-engine door convention is the fit floor, not the
design target.** The reason is MEASURED and severe (§7): there is no
pathfinder, bots walk a straight line at a 2-D waypoint, and `bot_act`
fires **one** 1.2 m whisker and veers hard perpendicular when it is
blocked. A gap sized to a body is a gap bots enter by luck.

**DERIVED-BY-ANALOGY / effectively ASSUMED — a bot-usable aperture
floor of 7.0 m.** The best number the codebase offers is
`BOT_CLIMB_LANE_M = 7.0` (`sim.rs:201`), which is its own statement of
how far a bot drifts sideways off a line it is following, "because a bot
on a flight is being shoved sideways by `squad_spacing` the whole way
up". Cliffhold's shipped 12 / 14 / 24 m clear it by 1.7× to 3.4×.
**This is an analogy, not a measurement.** Nobody has ever measured bot
throughput through an aperture of any width. That experiment is named in
§10 and it is an afternoon's work.

### 3.3 Cover heights — the three tiers that already ship

MEASURED, the shared infill (`sim.rs:1591-1595, 1613-1619, 1633-1637`),
which runs for **every** map, plus the Battlefield clutter loop
(`sim.rs:1488-1490`) and its trees (`sim.rs:1462`).

| Tier | height range | comment in code |
|---|---|---|
| Outer ring, low | 0.9 – 1.3 m | "vaultable" |
| Outer ring, mid | 1.6 – 2.2 m | "shoulder - shoot over crouched" |
| Outer ring, high | 2.6 – 3.4 m | "hard cover" |
| Mid-field stepping stones | 0.85 – 1.25 m | — |
| Flank lanes | 1.1 – 1.7 m | long low walls |
| Battlefield clutter | 1.1 – 2.4 m | — |
| Trees | 3.4 m (fixed) | — |
| `BOT_TERRAIN_M` | **3.5 m** | the line above which a bot's router calls it terrain, not furniture (`sim.rs:191`) |

The tiers are keyed to the stance heights: 0.9–1.3 m is above
`CROUCH_HEIGHT` 1.15 at its top and below `EYE_REL` 1.62 throughout, so
it hides a crouched body and not a standing one; 1.6–2.2 m brackets
`EYE_REL` 1.62 and `BODY_HEIGHT` 1.78; 2.6–3.4 m is over every body but
the two mechs. That relationship is the contract §2.2 says height must
never be scaled by.

**"Vaultable" is a misnomer and it has been for the whole project.**
There is no vault verb (§4.4). Those blocks are *jumpable*. `vault` and
`mantle` occur 7 times across `src/` (6 in `sim.rs`, 1 in `main.rs`) and
**every one is a comment. There is no call site.**

---

## 4. VERTICAL METRICS — the ledge bands

This section is the substance of `TRV-0180`. Everything here is DERIVED
from MEASURED constants, and the derivations are complete.

### 4.1 The vertical rule the engine actually runs

MEASURED, two lines that together decide every vertical question:

- **Support** (`sim.rs:1045-1059`, called at `sim.rs:9317`):
  `support_top` returns the tallest cover top under (x,z) satisfying
  `c.max[1] <= y0 + step_up`. Your feet snap to it.
- **Push-out** (`sim.rs:9285-9287`): a box is skipped — it does **not**
  block you horizontally — when `c.max[1] <= y + step_up()`.

The two use the same threshold, which produces the governing fact:

> **You do not have to clear a ledge. You have to get your feet within
> `step_up` of its top.** At that instant the box stops blocking you
> horizontally *and* becomes your support, in the same tick.

**Therefore the maximum standable ledge height is `apex + step_up`, not
`apex`.** This is the single most important derived number in the file
and it is 0.55 m larger than the intuitive answer.

The same function is shared by the bot route planner deliberately
(`sim.rs:1039-1044`), so a planner cannot disagree with a body about
where the ground is.

### 4.2 Jump apexes — DERIVED

Ballistic apex `h = v² / 2g`, with `g = GRAVITY = 18.0 m/s²`.

| Body | launch v | working | apex |
|---|---|---|---|
| Soldier, flat | 7.4 | 7.4² / 36 = 54.76 / 36 | **1.521 m** |
| Soldier, crouch-jump | 7.4 × 1.06 = 7.844 | 7.844² / 36 = 61.528 / 36 | **1.709 m** |
| `RobotSuit` | 7.4 × √1.7 = 9.648 | 93.092 / 36 | **2.586 m** |
| `RoyalMech` | 7.4 × √1.87 = 10.119 | 102.401 / 36 | **2.844 m** |
| `ScoutMech` | 7.4 (soldier path) | 54.76 / 36 | **1.521 m** |

The mech launch speed is `JUMP_SPEED × chassis_scale().sqrt()`
(`sim.rs:5983`) — the square root, so each chassis clears the same
obstacle *measured in its own body heights*. The code's own comment
predicts "~2.59 m" for the heavy; **my derivation reproduces it at
2.586 m.** MEASURED comment, DERIVED value, and they agree.

The crouch-jump bonus is grounded-only and gated on `f.crouch`
(`sim.rs:8500`). The **mech does not get it** — `f.vy` is set straight
from `mech_jump_speed_of` (`sim.rs:7601`) with no bonus term.
The **Scout uses the soldier path** — the mech jump branch is gated on
`in_heavy_mech()` (`sim.rs:7586`) — plus one air jump
(`scout_air_jump_used`, `sim.rs:8508`), which does not earn the bonus
either ("there are no coiled legs to release mid-air").

### 4.3 ⭐ THE LEDGE BANDS

**DERIVED: ceiling = apex + `step_up`.**

| Body | walk-on (free) | plain jump | crouch-jump | absolute foot ceiling |
|---|---|---|---|---|
| **Soldier** | ≤ **0.55 m** | ≤ 1.521 + 0.55 = **2.071 m** | ≤ 1.709 + 0.55 = **2.259 m** | **2.26 m** |
| `ScoutMech` | ≤ 0.55 m | ≤ **2.071 m** | — (no bonus) | **2.07 m**, or 3.59 m theoretical† |
| `RobotSuit` | ≤ **0.935 m** | ≤ 2.586 + 0.935 = **3.521 m** | — | **3.52 m** |
| `RoyalMech` | ≤ **1.029 m** | ≤ 2.844 + 1.029 = **3.873 m** | — | **3.87 m** |

† DERIVED-THEORETICAL only: an air jump fired exactly at apex re-sets
`vy` to the full `JUMP_SPEED`, giving 2 × 1.521 + 0.55 = 3.592 m. It
requires frame-perfect timing and I have not tested it. **Do not author
geometry to it.**

**The four numbers a level builder needs, in one line:**

```
0.55 m   the invisible step   — author nothing finer; it is not an obstacle
2.07 m   plain-jump ceiling   — a soldier gets up here with one press
2.26 m   ABSOLUTE FOOT CEILING — above this, no verb in the game reaches
3.52 m   heavy-mech ceiling   — 3.87 m for a Royal
```

**⭐ THE DESIGN TOOL THIS HANDS YOU: the band 2.26 m → 3.52 m is
soldier-proof and mech-passable.** A 3 m wall is a wall to infantry and a
step to a machine. That window is the whole physical argument for the
mech being big, and it is 1.26 m wide. Nothing in any brief names it.

### 4.4 The verbs that exist, and the one that does not

MEASURED by grep across `engine/crates/jk_tdm/src/`:

| Verb | Exists? | Reach |
|---|---|---|
| Auto step-up | **yes** | 0.55 m (chassis-scaled) |
| Jump / crouch-jump | **yes** | §4.2 |
| Stair flight | **yes** | unbounded, 0.5 m per tread (§5) |
| Free descent | **yes**, unlimited | §4.5 |
| Mech hull climb | **yes**, but not terrain | `CLIMB_ATTACH_RANGE_M = 2.9` (`sim.rs:6214`) attaches a soldier to a **mech**, not to geometry |
| **Vault** | **NO** | — |
| **Mantle** | **NO** | — |
| **Ledge climb** | **NO** | — |
| Ladders / ropes | **NO**, and impossible without undersides | — |

`vault` / `mantle` appear 7 times in `src/`; **every one is a comment or
a test string. There are zero call sites.**

### 4.5 Drops are free and unlimited — MEASURED

**`jk_tdm` has no fall damage.** Stated three separate times in the
source (`sim.rs:1021`, `sim.rs:13556`, `sim.rs:27015`) and there is no
`FALL_*` constant anywhere. The only vertical limit in the other
direction is `SOFT_CEILING_M = 120.0` (`sim.rs:998`), which pushes
flyers back down.

**Design consequence, DERIVED:** every ledge in this game is a
**one-way route downward**. Height is a gate on ascent only. A builder
who wants a band to be defensible must gate the *ways up*; the way down
is always open and always free, from any height.

**Contrast with S-01, and it is a clean non-transfer:** the source gives
TF2's "Maximum drop height without fall damage: 256" units. That figure
governs nothing here. Recorded so nobody imports it.

### 4.6 Band separation — the rule the code asserts, and the stronger one it needs

**MEASURED, and it is already a passing test** (`sim.rs:27283-27286`):
consecutive occupied bands must differ by more than `STEP_UP`, "else they
are one surface". Cliffhold's eight bands and their gaps:

```
  0 → 5 → 6 → 7 → 12 → 18 → 24 → 32     (m)
    5    1    1    5    6    6    8      gaps, all > 0.55 ✓
```

**DERIVED: that test is too weak by a factor of four.** A 1 m gap passes
it and is inside the 2.07 m plain-jump ceiling, so the 5 / 6 / 7 m trio
(city roofs, mesa apron, muster plaza) is freely inter-traversable in
both directions and reads as **one** region for gameplay purposes. Three
of Cliffhold's eight "bands" are one band wearing three names.

> **PROPOSED RULE (DERIVED, mine): for two altitude bands to be a real
> gameplay separation — one-way down, gated on the way up — they must
> differ by more than 2.26 m (the soldier's absolute foot ceiling), or
> more than 3.87 m if the separation must also hold against a Royal.**

Applied to Cliffhold: 0 / 12 / 18 / 24 / 32 are genuine bands (gaps 12,
6, 6, 8). 5 / 6 / 7 collapse into one. **Five real bands, not eight.**
That is still comfortably inside VM-10's "three floor planes for any
given area" heuristic, because they are not stacked in one place — the
plaza, the city and the castle are different areas (`vertical-maps`
§Q2 makes the per-area point explicitly).

---

## 5. STAIRS AND RAMPS

### 5.1 What this game ships — MEASURED

`STAIR_RISE_M = 0.5` (`sim.rs:994`) — **every riser on every flight in
the game**. Tread depth is per-flight. All eleven flights, read off the
tables at `sim.rs:1865-1883, 1891-1892, 1932, 1956, 1960`:

| Flight | tread | treads | rise | slope (DERIVED) | width |
|---|---|---|---|---|---|
| The Breach | 1.2 m | 36 | 0 → 18 m | **22.62°** | 8 m |
| The North Road | 1.4 m | 36 | 0 → 18 m | **19.65°** | 16 m |
| The Great Stair | 1.4 m | 12 | 0 → 6 m | 19.65° | 12 m |
| The Bench Stair | 1.4 m | 12 | 6 → 12 m | 19.65° | 10 m |
| The Shoulder Stair | 1.5 m | 12 | 12 → 18 m | **18.43°** | 12 m |
| The West Ramp | 1.5 m | 24 | 0 → 12 m | 18.43° | 12 m |
| The Quarry Steps | 1.5 m | 12 | 12 → 18 m | 18.43° | 12 m |
| Plaza stairs (×2) | 1.2 m | 14 | 0 → 7 m | 22.62° | 10 m |
| The Mural Stair | 1.3 m | 12 | 18 → 24 m | **21.04°** | 7 m |
| Keep flight 1 | 1.3 m | 14 | 18 → 25 m | 21.04° | 5 m |
| Keep flight 2 | 1.3 m | 14 | 25 → 32 m | 21.04° | 5 m |

*DERIVED:* rise = `0.5 × n`; slope = `arctan(0.5 / tread)`.
`arctan(0.5/1.2) = 22.620°`, `arctan(0.5/1.3) = 21.038°`,
`arctan(0.5/1.4) = 19.654°`, `arctan(0.5/1.5) = 18.435°`.
Every `base + 0.5n` lands exactly on its named band constant — I checked
all eleven.

**Why 0.5 m — MEASURED reason, and one gap in it.** The code
(`sim.rs:977-994`) gives two ceilings the riser must clear, and says the
lower binds: `STEP_UP` 0.55 (above it, infantry are stopped dead and the
route becomes mech-only) and `BOT_PROBE_Y` 0.75 (above it, the tread
reads as a wall to `bot_act`'s single whisker and bots slide off the
flight sideways). Both are **asserted as relationships in a test**
(`sim.rs:27304-27311`), not as a pinned value.

**What the code does NOT say is why 0.5 rather than 0.15**, which clears
both ceilings just as well. **INFERRED / ASSUMED (mine): box count.**
Each tread is one `Aabb`. An 18 m climb is 36 boxes at 0.5 m and 120 at
0.15 m; across nine flights that is 324 vs 1080 boxes added to a cover
set that is linearly scanned in the push-out and support loops. Flagged
as my inference. Do not attribute it to the code.

### 5.2 Against S-01 — the convention LOSES on the step, SPLITS on landings

Verbatim from S-01, re-fetched today:

> "Modern stairs should follow a 30-35 degree slope. `arctan(7/11) = 32
> degrees`"
>
> "Modern flights of stairs have landings / platforms every 12-16 steps.
> Long or tall stairs will feel industrial, monumental, or otherwise
> non-domestic."

Its step dimensions: Unity 0.10 × 0.15 m; Unreal 0.15 × 0.25 m;
Quake 8 × 12 in = 0.203 × 0.305 m; real-life 7 × 11 in = 0.178 × 0.279 m.
**Internally consistent** — I checked all four slopes: 33.69°, 30.96°,
33.69°, 32.47°, every one inside its own 30–35° claim, and its stated
`arctan(7/11) = 32` reproduces at 32.47°.

| | S-01 convention | `jk_tdm` shipped | ratio |
|---|---|---|---|
| Riser | 0.10 – 0.203 m | **0.50 m** | 2.46× – 5.0× |
| Tread | 0.15 – 0.305 m | **1.2 – 1.5 m** | 3.9× – 10.0× |
| Slope | 30 – 35° | **18.4 – 22.6°** | 8–17° shallower |

**VERDICT: `jk_tdm` wins, and it is not close.** Three reasons, in order
of force:

1. The shipped riser is **asserted by a passing test** against two
   independently-motivated engine ceilings. The convention is a
   readability heuristic about how a stair looks to a human.
2. `STEP_UP` is 0.55 m — **2.7× to 5.5× a single real riser.** The
   movement system physically cannot see a 0.15 m step. Any stair detail
   finer than 0.55 m is decoration that the body walks straight through.
   *This is the general rule: `jk_tdm` cannot express architectural
   detail below 0.55 m vertically.*
3. Bots need risers under `BOT_PROBE_Y` = 0.75 m, and there is no
   pathfinder to compensate.

**Landings — SPLIT verdict.** Nine of eleven flights are 12 or 14 treads,
inside the convention's 12–16 band. The keep stair even turns back on
itself at a half-landing after 14 treads (`sim.rs:1958`). **The two long
flights violate it by 2.25×** — 36 treads, no landing, 43.2 m and 50.4 m
of run. **Keep the violation.** The source's own stated cost of a long
flight is that it "will feel industrial, monumental, or otherwise
non-domestic", and a 43 m siege ramp up an 18 m cliff face *is*
monumental — the cost is the intent.

**But add landings anyway, for a reason S-01 does not name.** ASSUMED
(mine): a landing is the only flat ground on a flight, and this game has
no fall damage, so a landing is the only place on that ramp where a body
can stop, turn, and take cover. The code's own description of the Breach
is "fully exposed from both rims" (`sim.rs:1866-1867`). A 43 m run with
no beat on it is a shooting gallery — which is exactly the failure mode
`vertical-maps` §Q1 records for players on predictable arcs.

### 5.3 A recorded contradiction, resolved by measurement

`vertical-maps/SOURCES.md` logs **CONTRADICTION-1**: S-01's 30–35° stair
slope is steeper than VM-08's illustrative navmesh walkable-slope limit
of 0.5 rad = 28.65°, so "a staircase built to the recommended
human-comfort slope is steeper than the example navmesh will walk up".

**It does not bite this game, on two independent grounds, and neither is
an average.** (1) `jk_tdm` stairs run **18.4–22.6°**, below both figures
with 6° of margin against the tighter one. (2) There is no voxel navmesh
in `jk_tdm` for the limit to apply to. **If one is ever built, our
existing stairs pass it.** Recorded as resolved-by-measurement.

---

## 6. SIGHTLINES, AND THE VERDICT ON THE IX-A 40 m RULE

### 6.1 The baselines are re-measured, and the old ones were stale

**MEASURED today.** I ran the shipping validator rather than quoting the
ledger:

```
cargo test -p jk_tdm --bins sightline -- --nocapture
→ sim::tests::sightline_validator_measures_real_lines_and_reports_every_map ... ok
```

| Map | `SOURCES.md` (pre-2026-08-10) | **measured 2026-08-10** | ratio |
|---|---|---|---|
| Arena | 80.2 m | **102.9 m** | 1.283 |
| Bailey | 93.4 m | **120.2 m** | 1.287 |
| Gardens | 92.0 m | **115.0 m** | 1.250 |
| Battlefield | 509.9 m | **637.4 m** | 1.250 |
| Cliffhold | *(not recorded)* | **577.1 m** | — |

**The drift is `MAP_SCALE`.** Two maps moved by exactly 1.250 and two by
1.283–1.287 (Arena and Bailey also gained infill cover). The recorded
baselines were taken **before the +25% map expansion** and are wrong by
that factor. `SOURCES.md` is corrected. **DERIVED, and the explanation is
complete: 102.9/80.2 = 1.283, 120.2/93.4 = 1.287, 115.0/92.0 = 1.250,
637.4/509.9 = 1.250.**

**⚠ PRECISION CEILING — the numbers above must not be quoted to 0.1 m.**
The instrument samples a grid at `half / 10` (`sim.rs:19453`) and returns
the distance between two **grid points**:

| Map | grid step | quantisation |
|---|---|---|
| Arena | 4.25 m | ±1 grid diagonal ≈ 6.0 m |
| Bailey | 5.0 m | ≈ 7.1 m |
| Gardens | 4.75 m | ≈ 6.7 m |
| Battlefield | **25.0 m** | ≈ **35.4 m** |
| Cliffhold | **30.0 m** | ≈ **42.4 m** |

The true worst line can exceed the reported figure by up to one grid
diagonal. **"Battlefield 637.4 m" honestly means "Battlefield, roughly
640 m, ±35".** Nothing finer may be built on it.

### 6.2 ⚠ The instrument is flat-map-only. This is a defect, not a caveat.

**MEASURED, `sim.rs:9635-9672`.** `max_unobstructed_sightline` samples
eye points at `[x, EYE_REL, z]` — **an absolute y of 1.62 m**, not 1.62 m
above the local ground — and excludes any point "buried" inside a cover
volume.

On a multi-band map every consequence follows: a standing position on
Cliffhold's 18 m plateau is sampled at y = 1.62, which is *inside* the
mountain slab (0 → 18 m), so it is marked buried and **dropped from the
sample entirely**. The same is true of every band above 0.

> **The 577.1 m reported for Cliffhold measures the 0 m ground band and
> nothing else.** The plateau, the shelves, the rampart and the keep top
> — the entire point of that map — contribute zero samples. The
> instrument silently ignores every surface above y = 0.

This also explains why the old baseline table listed four maps and not
five: on a vertical map the number does not mean what its name says.

**This is the highest-value fix in this document.** Sampling at
`terrain_top(cover, x, z) + EYE_REL` instead of at `EYE_REL` would make
the validator work on vertical geometry, and `terrain_top` **already
exists** for exactly this reason (`sim.rs:1065`: "how high is the ground
here"). It is a one-line change to a function that already imports it.
I am not writing it — Toto does not write game code — but it is named,
scoped and located.

### 6.3 TF2's combat bands, transferred by ratio

S-01 gives TF2's ranges in Hammer units. Rather than guess a
unit-to-metre conversion — a place where a wrong constant would silently
poison every downstream number — I transfer by **ratio to the Source
player height (72), which the same page supplies.** DERIVED:

| S-01 band | units | ÷ 72 | × 1.78 m |
|---|---|---|---|
| "Close range weapons" | 256 | 3.556 | **6.33 m** |
| "Medium range weapons" | 1024 | 14.222 | **25.31 m** |
| "Rocket spam and snipers" | 2048 | 28.444 | **50.63 m** |
| "Maximum drop height without fall damage" | 256 | 3.556 | 6.33 m — **MOOT**, §4.5 |

**An independent corroboration of the 40 m rule's order of magnitude.**
The castle brief wants engagements in the **25–35 m** band. TF2's medium
ceiling lands at **25.3 m** and its sniper band opens at **50.6 m**. The
40 m rule sits between them. Two unrelated games, one derived by ratio
from the other, agree on where "medium" ends and "sniper" begins.
**This validates the ORDER of the rule, not the digit 40.** Do not read
it as proof that 40 is correct.

### 6.4 ⭐ VERDICT ON THE 40 m RULE: RETIRE the global form, REPLACE it with two

The rule as written (`sim.rs:9623-9625`): *"no two standing positions may
see each other across more than 40 m of open ground."*

**Every shipping map exceeds it — by 2.6× to 16×** (§6.1). The
`SOURCES.md` conclusion was "it binds NEW maps, not retrofits." **That
conclusion is wrong, and the reason is arithmetic, not history.**

The rule is a **global maximum over all pairs**. On any map with a single
open line anywhere — a corner, a spawn approach, a diagonal across dead
ground — the maximum is that line, no matter how well the middle is
broken up. The validator proves this itself: its own instrument check
asserts that an **empty** Arena reads ~its own diagonal
(`sim.rs:19431-19435`). A 42.5 m-half map has a 117 m diagonal.
**A 40 m global maximum is unsatisfiable on any map larger than about
15 m half-extent.** It is not that our maps fail the rule; the rule
cannot be passed by a map of this size, and could not have been passed by
the maps as they were before the expansion either.

**RETIRE the global form.** Keep the *intent*, which is real and is
corroborated in §6.3, and express it in two forms that a builder can
actually satisfy and a test can actually check:

> **RULE 3a — LOCAL, and it binds.** No unobstructed eye-level sightline
> longer than **40 m** may connect two *objective-relevant* standing
> positions: the two checkpoints, the KOTH ring, the spawn exits, and the
> named routes between them. Same instrument, restricted to that point
> set instead of a uniform grid. Cheap to implement, and it is what the
> rule was always trying to say.
>
> **RULE 3b — DISTRIBUTIONAL, and it is the design target.** The
> **median** pairwise-visible distance over the sampled standable set
> should sit in the **25–35 m** band. That is what "engagements belong in
> 25–35 m" means: a statement about where fights happen, not about the
> single longest line in the level. The instrument already computes every
> pair; it currently throws all of them away but the maximum.

**What a builder does when a new map violates the 40 m global: nothing.**
That is now an expected reading and not a defect. Break a long line only
when it connects two positions that both matter — and prefer breaking it
with a **mass that reads as a landmark** rather than a blocker, per
`vertical-maps` VM-01: figure/ground failure costs you both navigation
*and* target acquisition from one property.

**Labels:** the measurements are MEASURED; "40 m is unsatisfiable at this
map scale" is DERIVED; **rules 3a and 3b are a DESIGN PROPOSAL by me,
not a finding.** They need an owner decision, not a citation.

---

## 7. WHAT THE BOTS REQUIRE FROM A MAP

Every number MEASURED. These are hard authoring constraints, not advice —
violate one and bots stop using that part of the map.

| Constant | Value | What it constrains | Source |
|---|---|---|---|
| `BOT_PROBE_Y` | **0.75 m** | Height of the single obstacle whisker. **Any tread or step above this reads as a WALL** and the bot veers hard perpendicular. | `sim.rs:167` |
| whisker length | **1.2 m** | Look-ahead of that whisker. | `sim.rs:167` doc |
| `BOT_TERRAIN_M` | **3.5 m** | Above this a blocker is TERRAIN and gets routed around; below it, FURNITURE, left to the whisker. Set immediately above the tallest hard cover (3.4 m) and below every Cliffhold building (5 m+). | `sim.rs:191` |
| `BOT_ROUTE_PROBE_M` | **44.0 m** | How far ahead the ground is checked. **Local on purpose** — a bot re-plans several times a second. | `sim.rs:173` |
| `BOT_ROUTE_STEP_M` | **0.75 m** | Sample spacing of that check. Must be short enough that a flight never reads as one un-takeable step. | `sim.rs:180` |
| `BOT_ROUTE_MIN_M` | **2.6 m** | Closest a routed waypoint is ever planted. Below this a cornered bot re-rolls every tick. | `sim.rs:196` |
| `BOT_CLIMB_LANE_M` | **7.0 m** | Lane width on a flight, for "am I already on this one". Generous, because `squad_spacing` shoves bots sideways all the way up. | `sim.rs:201` |

**The two derived authoring rules:**

1. **DERIVED: a riser must be < 0.55 m (`STEP_UP`) AND < 0.75 m
   (`BOT_PROBE_Y`); the lower binds, so < 0.55 m.** A riser between 0.55
   and 0.75 m is a **mech-only route** — the mech's `step_up()` is 0.935
   and it walks up while infantry is stopped dead. That may be exactly
   what you want; know that you are doing it.
2. **DERIVED: aim long flights down a line a bot already walks.** Two of
   Cliffhold's flights are deliberately on x = 0, the same line the
   capture ring sits on and both spawn rows straddle, "so a bot walking
   straight at the objective walks onto a flight rather than into a wall"
   (`sim.rs:1849-1855`). With no pathfinder, a flight bots never cross is
   a flight only the player uses — the parity split this codebase keeps
   re-shipping.

**And the standing defect, MEASURED and unchanged since `vertical-maps`
recorded it:** `Fighter::waypoint` is `[f32; 2]` — x and z, **no
height**. Sampled uniformly from a square, never checked for
reachability. **A bot cannot choose to go up. Ever.** `Climb` links
(`sim.rs:1009-1034`) are published by every flight at build time so the
link list cannot go stale against the geometry, and BOT ROUTING consumes
them on Cliffhold. **`sim.rs:27649` states the flat maps were
deliberately left alone.** Verify the current state before scoping any
work on this; `TRV-0043` marks it `UNVERIFIED`.

---

## 8. HARD ENGINE CONSTRAINTS — the things no metric can get around

All MEASURED.

1. **Cover is `Aabb` only — axis-aligned boxes.** No slopes, no curves,
   no rotation. A ramp is a staircase; a round tower is a square one.
2. **A box has a top and no underside** (`sim.rs:1741-1748`). **No
   overhangs, arches, tunnels, bridge spans, ceilings or lintels are
   possible.** A deck at 18 m is a solid mass from the ground up. Every
   "building" is a solid mass or a **ring of walls around an open court**
   — Cliffhold's keep is the second kind, on purpose, because the
   Bailey's keep and the Gardens' gazebo are both solid blocks nobody can
   enter, and one of them is argument-for-argument a copy of the Arena's
   centre tower.
3. **No fall damage** (§4.5). Descent is free from any height.
4. **The playable area is a square clamp**, `[−half+0.5, half−0.5]` on
   both axes (`sim.rs:9281`). Non-square playable regions are not
   expressible; a bot's waypoint clamp is to that same square.
5. **Vertical detail below 0.55 m is invisible to the movement system**
   (§5.2). It is art, and it must never carry gameplay meaning.
6. **Height never scales with `MAP_SCALE`** (§2.2). Cover height is a
   contract with crouch, the hit bands and the step-up.
7. **A flight's top tread must land INSIDE the slab it climbs onto**, not
   flush against its face (`sim.rs:1857-1864`). A flight that merely
   touches its taller neighbour leaves a one-tread band of open ground
   where the support rule drops you the whole way to zero — after which
   the thing you were climbing onto is an unclimbable wall in your face.
   **This is invisible by inspection**; the
   `every_cliffhold_band_is_reachable_on_foot` walker found it on the
   North Road. Any new vertical map needs that walker pointed at it.

### 8.1 A note on Cliffhold's status

Most of the shipped vertical numbers in this document come from
`build_cliffhold`. **Its client half was deleted** (`main.rs:16759`: "the
map that has just been deleted"), and `TRV-0039` asks for the sim half to
follow. The sim half is intact and its tests pass — I ran one today.

`TREVOR_TASKS.md` Band 6 says of that row: **"Salvage first."** This
document is that salvage for the metrics. **The numbers in §4, §5 and §7
are independent of whether Cliffhold survives** — they are properties of
`STEP_UP`, `GRAVITY`, `JUMP_SPEED`, `STAIR_RISE_M` and the bot constants,
all of which live in `sim.rs` and serve every map. What dies with
Cliffhold is only the worked example.

---

## 9. ⭐ THE TWO BLOCKED ROWS — answered directly

Read this section alone if you are here to unblock.

### 9.1 `TRV-0051` — Traversal

**What it needed from this file**, per `BACKLOG.md` #7 ("Traversal:
climb, vault, mantle, ledge bands — *already scoped in the master prompt;
must match map metrics*") and `TREVOR_LEDGER.md:687`: the height envelope
a new traversal verb must cover, and the geometry it can key off.

**The answer.**

**(1) There are two different traversal projects here and they must not
be confused.**

- **PROJECT A — FEEL. Reach unchanged, 0.55 m → 2.26 m.** Today, mounting
  a ledge is `f.pos[1] = (f.pos[1] + f.vy*DT).max(support)`
  (`sim.rs:9325`) — an instant snap. **That single line is "the wall
  stop" at its source**, and it is the anti-pattern this project's own
  doctrine names. Replacing it with an animated mantle changes no reach
  and no map. **This is the one with a READ source behind it**:
  `traversal/SOURCES.md` **S-04** (Epic, *Motion Warping*, tier P, READ)
  gives the exact mechanism — play the authored move, declare a notify
  window, warp the root inside it so the hands land on the **actual**
  ledge instead of snapping. **Recommend doing this one.**

- **PROJECT B — REACH. Getting a soldier above 2.26 m.** This changes
  every map in the game. All "hard cover" ships at **2.6–3.4 m** (§3.3),
  which today is exactly the soldier-proof / mech-passable window (§4.3).
  Give infantry a 3.4 m mantle and every piece of hard cover on every map
  becomes standable, and the 1.26 m window that is the whole physical
  argument for the mech being big **closes**. **Do not do this without an
  owner decision on that trade.** It is a balance change wearing an
  animation costume.

**(2) The exact envelope, if you build a verb.** From §4.3, per body:

```
soldier   free ≤ 0.55   jump ≤ 2.07   crouch-jump ≤ 2.26   ← ceiling today
scout     free ≤ 0.55   jump ≤ 2.07                        ← +1 air jump
heavy     free ≤ 0.935  jump ≤ 3.52
royal     free ≤ 1.029  jump ≤ 3.87
```

A mantle that reaches **2.26 → 3.4 m** covers every piece of hard cover
and every tree. A mantle that reaches **2.26 → 3.87 m** additionally
gives infantry parity with a Royal. Pick deliberately.

**(3) What the verb can key off, and it is cheaper than you think.**
Cover is `Aabb` only, tops only, **no undersides (§8, item 2)**,
therefore:

- **Clearance above a ledge is always infinite.** There is nothing to
  check. Half of a normal mantle test does not exist here.
- The detector is: *for the fighter's facing, find the nearest `Aabb`
  whose `max[1]` lies in `[y + step_up(), y + reach]` and whose face the
  body is currently being pushed out of.* The push-out loop
  (`sim.rs:9284-9305`) **already iterates exactly that set and already
  knows which box rejected you.** The information is on the stack.
- **No navmesh is required, and none exists.** Ladders and ropes are
  impossible by the same rule that makes clearance free.

**(4) The map-side rule the verb must respect.** Whatever reach you pick
becomes the new "absolute foot ceiling" in §4.3, and §4.6's band
separation must move with it. If a soldier mantles 3.4 m, then a
gameplay-real altitude band must be **more than 3.4 m** clear of the one
below it, and Cliffhold's 18 → 24 m rampart (6 m) still holds while its
5 / 6 / 7 m trio is even more thoroughly one surface.

**Is `TRV-0051` unblocked? YES.** The envelope, the geometry contract,
the detector sketch, the single line to replace, and the balance trap are
all here. What is *not* here is a decision between Project A and Project
B — that is an owner call, not a missing metric.

### 9.2 `TRV-0180` — Ledge bands

**What it needed**, per `BACKLOG.md:22` and `TREVOR_LEDGER.md:878`
("Hull climbing proved the attach/parent/stamina mechanic but is
mech-specific"): the actual height bands, for terrain rather than for a
mech hull.

**The answer is §4.3 and §4.6.** Restated as the four thresholds:

```
0.55 m  — the invisible step. Nothing finer is an obstacle. Author no
          gameplay meaning below this line.
2.07 m  — plain-jump ceiling. One press.
2.26 m  — ABSOLUTE FOOT CEILING. No verb in the game goes higher.
3.52 m  — heavy-mech ceiling (3.87 m Royal). Between 2.26 and 3.52 is
          the soldier-proof / mech-passable window.
```

**Derivation, in full, so it can be re-checked:** the ceiling is
`apex + step_up`, **not** `apex`, because `support_top` accepts any top
with `c.max[1] <= y0 + step_up` (`sim.rs:1052`) and the push-out skips
that same set (`sim.rs:9285`) — so a box stops blocking you and becomes
your floor in the same tick, 0.55 m before you have cleared it.
Apex = `v²/2g`; `v = 7.4`, `g = 18.0`; flat `54.76/36 = 1.521`;
crouch-jump `(7.4×1.06)²/36 = 61.528/36 = 1.709`; add 0.55.

**And here is the concrete thing to fix first, which nobody has
noticed.** Cross the shipped cover tiers (§3.3) against those thresholds:

| Shipped tier | range | soldier verdict |
|---|---|---|
| stepping stones | 0.85 – 1.25 m | all plain-jump |
| "vaultable" | 0.9 – 1.3 m | all plain-jump — **and there is no vault; the label has been wrong for the whole project** |
| flank lanes | 1.1 – 1.7 m | all plain-jump |
| "shoulder" | 1.6 – 2.2 m | 1.6–2.07 plain jump (**78.5%** of the range); 2.07–2.2 **crouch-jump only** (21.5%) |
| Battlefield clutter | 1.1 – 2.4 m | 1.1–2.07 plain (74.7%); 2.07–2.26 crouch-jump (14.5%); **2.26–2.4 unmountable (10.8%)** |
| "hard cover" | 2.6 – 3.4 m | **none of it mountable on foot**; all of it mountable by the heavy mech (ceiling 3.52) |
| trees | 3.4 m | not on foot; heavy mech yes, by 0.12 m |

*DERIVED fractions:* shoulder `(2.071−1.6)/0.6 = 0.785`; clutter
`(2.071−1.1)/1.3 = 0.747`, `(2.259−2.071)/1.3 = 0.145`,
`(2.4−2.259)/1.3 = 0.108`.

> **⭐ THE DEFECT: roughly a fifth of "shoulder" cover and a tenth of
> Battlefield clutter is drawn from a uniform random range that straddles
> a traversal threshold — and the blocks look identical.** Two adjacent
> crates, one 2.05 m and one 2.15 m, are drawn by the same line of code
> and one of them you can get on top of. The generator does not know the
> difference and the player cannot see it. **That is the single most
> actionable output of this file** and it is a four-line fix: snap each
> tier's range to one side of the thresholds. Suggested (ASSUMED, mine):
> low `0.9–1.3` stays; shoulder becomes `1.6–2.0` (all plain-jump, no
> straddle); hard cover becomes `2.4–3.4` (none foot-mountable, no
> straddle); Battlefield clutter splits into two draws instead of one.

**Also fix `BOT_TERRAIN_M`'s near-coincidence, or at least document it.**
`BOT_TERRAIN_M` is 3.5 m and the heavy mech's foot ceiling is 3.521 m.
They differ by 21 mm and there is **no causal link between them** — one
is a bot routing threshold set just above the tallest cover, the other is
`7.4²·1.7/36 + 0.55`. A future retune of `MECH_SCALE` moves one and not
the other. **Coincidence, flagged as a coincidence.** Do not let anyone
derive one from the other.

**Is `TRV-0180` unblocked? YES**, and it comes with a shipped defect
located, quantified and costed.

---

## 10. WHAT THIS FILE DOES NOT ANSWER

Stated as plainly as the answers, per the standing rule.

1. **Altitude band SPACING in metres has no source behind it.**
   `vertical-maps/SOURCES.md` §Q2 searched for one and found none: *"How
   far apart, in metres, should altitude bands be? Not answered by any
   source in this ledger."* §4.6's 2.26 m figure is a **traversal**
   bound — it says when two bands stop being one surface. It says nothing
   about when two bands stop being *interesting*. **Do not let anyone
   cite §4.6 as design guidance on spacing.**
2. **Bot throughput through an aperture has never been measured.** §3.2's
   7.0 m floor is an analogy to `BOT_CLIMB_LANE_M`, not a measurement.
   The experiment: build a wall with a gap, roam one team at it for 60 s,
   count crossings, sweep the gap 2 → 24 m. One afternoon. Until then
   Cliffhold's 12–24 m is precedent, not evidence.
3. **Nothing here is playtested.** Every "should" in this file is derived
   from constants or transferred from another game. `vertical-maps`
   VM-01 and VM-02 both record shipped maps that tested well as greybox
   and failed after the art pass, with the geometry untouched. **Metrics
   do not survive contact with an art pass**, and this file cannot tell
   you they will.
4. **Interior architecture is untested and unshipped.** §3.1's corridor
   figures bind nothing today because no map has a corridor. They are a
   prediction.
5. **The 2.6–3.4 m "hard cover" tier being fully mech-mountable is
   derived, not observed.** I did not watch a mech stand on a crate. The
   arithmetic says it can (2.586 + 0.935 = 3.521 > 3.4). One capture
   would settle it.
6. **`vertical-maps` VM-16** (Mononen, *Automatic Annotations in
   Killzone 3*) remains **NO-TRANSCRIPT** and is still the top unread
   target for anything about deriving cover and firing points from
   geometry. This file does not touch that question.

---

## 11. Provenance

**Primary sources read for this pass:**

| What | Status | Route |
|---|---|---|
| `engine/crates/jk_tdm/src/sim.rs`, `main.rs`, `mech_lineup.rs` | **READ** (targeted, ~40 regions, every cited line opened) | local |
| `sightline_validator_measures_real_lines_and_reports_every_map` | **RUN 2026-08-10**, output in §6.1 | `cargo test -p jk_tdm --bins sightline -- --nocapture` |
| S-01, *Blockout metrics*, The Level Design Book | **READ** — re-fetched twice today, second pass targeted at the two figures the ledger claimed and the first pass had not returned. Both reproduced verbatim. | `book.leveldesignbook.com/process/blockout/metrics` |
| `vertical-maps/SOURCES.md` (VM-01…VM-18) | **READ** in full, as this repo's own prior ledger | local |
| `traversal/SOURCES.md`, `maps/SOURCES.md`, `BACKLOG.md`, `TREVOR_LEDGER.md`, `TREVOR_TASKS.md`, `WHATS_MISSING.md` | **READ** (relevant sections) | local |

**No new external sources were sought.** Rule 13, build over research:
this was a synthesis dispatch and the deliverable shipped the same day.

**Nothing in this file is cited from a tool summary.** Every code line
number was opened and read. The one web source was fetched twice and its
own internal arithmetic checked (`arctan(7/11) = 32°` reproduces at
32.47°; all four of its engine step-slopes fall inside its own stated
30–35° band). This matters because `research/aiming/SOURCES.md` records
a fabrication incident in this repo: five numbers and a technique name
that appear nowhere in a real, correctly-identified paper, produced by a
fetch summary and caught only when someone read the PDF.
