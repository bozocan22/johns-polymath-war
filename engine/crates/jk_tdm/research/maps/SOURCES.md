# maps — SOURCES

| ID | Tier | Type | Title | Author / Studio | Year | URL | Accessed | Status | What it gave us |
|---|---|---|---|---|---|---|---|---|---|
| S-01 | P | reference | Blockout metrics, The Level Design Book | Robert Yang et al. | — | https://book.leveldesignbook.com/process/blockout/metrics | 2026-07-31 | READ | Cross-engine hard numbers. Player box: Unity 1.0 x 1.8 m (eye 1.5-1.7 m); Unreal 60 x 176 cm (eye 152 cm); Quake/Source 32 x 72 in (eye 64 in). Min hallway: 2.0 m / 150 cm / 64 in. Door: 1.25 x 2.5 m / 110 x 220 cm. Stairs 15 x 25 cm, 30-35 deg, landings every 12-16 steps. TF2 combat ranges: close <=256 u, medium <=1024 u; max safe drop 256 u. |
| S-02 | P/V | GDC talk | The Holy Grail of Multiplayer Level Design | Andrew Yoder / Hi-Rez | — | https://gdcvault.com/play/1025183 | 2026-08-01 | SNIPPET-ONLY | (Vault.) Greybox maps pushed to a public test queue for data-driven iteration. NOT COUNTED until watched. |

## NUMBERS

| ID | Value | Unit | What it measures | Conditions | Source |
|---|---|---|---|---|---|
| N-01 | 1.0 x 1.8 | m | player collision box | Unity convention | S-01 |
| N-02 | 2.0 | m | minimum hallway width | Unity convention | S-01 |
| N-03 | 15 x 25 | cm | stair riser x tread | Unreal convention | S-01 |
| N-04 | 30-35 | deg | recommended stair slope | — | S-01 |
| N-05 | 12-16 | steps | landing interval | — | S-01 |

## Applied to this codebase — measured, not assumed

This game's soldier is 1.78 m (BODY_HEIGHT) with eye at EYE_REL — inside
S-01's cross-engine band, so the brief's castle metrics can be adopted
without rescaling.

The IX-A 40 m sightline rule now has a real measuring instrument
(`max_unobstructed_sightline`, raycast through the game's own los_clear)
and a recorded baseline: Arena 80.2 m, Bailey 93.4 m, Gardens 92.0 m,
Battlefield 509.9 m — every shipping map exceeds the rule, so it binds
NEW maps, not retrofits.

## ⚠ CORRECTIONS — 2026-08-10, by TOTO writing `MAP_METRICS.md`

Three corrections to the block above. All three were found by checking
against source rather than by finding a new source.

**1. The sightline baselines above are STALE — pre map-expansion.**
Re-run today (`cargo test -p jk_tdm --bins sightline -- --nocapture`):

| Map | old (above) | **measured 2026-08-10** | ratio |
|---|---|---|---|
| Arena | 80.2 | **102.9 m** | 1.283 |
| Bailey | 93.4 | **120.2 m** | 1.287 |
| Gardens | 92.0 | **115.0 m** | 1.250 |
| Battlefield | 509.9 | **637.4 m** | 1.250 |
| Cliffhold | — | **577.1 m** | — |

The drift is `MAP_SCALE = 1.25`; two maps moved by exactly that and two
by slightly more, having also gained infill cover. **Use the right-hand
column.** PRECISION CEILING: the instrument samples at `half/10`, so
these carry ±1 grid diagonal — ±6 m on Arena, **±35 m on Battlefield,
±42 m on Cliffhold**. Do not quote them to 0.1 m.

**2. "It binds NEW maps, not retrofits" is wrong.** The rule is a global
maximum over all pairs, so on any map with one open line anywhere it
degenerates to that line. The validator's own instrument check asserts an
empty Arena reads ~its own 117 m diagonal. **A 40 m global maximum is
unsatisfiable at this map scale and would have been before the expansion
too.** See `MAP_METRICS.md` §6.4 for the retire/replace verdict.

**3. `max_unobstructed_sightline` is flat-map-only.** It samples eye
points at an ABSOLUTE `y = EYE_REL` (`sim.rs:9643`), not at ground + eye,
so on Cliffhold every position on every band above 0 m is "buried" inside
a slab and dropped from the sample. **The 577.1 m above measures the 0 m
ground band and nothing else.** `terrain_top` (`sim.rs:1065`) already
exists and is the fix. See `MAP_METRICS.md` §6.2.

Verified unchanged today and NOT drifted: `BODY_HEIGHT` 1.78 m
(`sim.rs:62`), `EYE_REL` 1.62 m (`sim.rs:46`), `BODY_RADIUS` 0.34 m
(`sim.rs:47`).

**S-01 re-verified.** Re-fetched twice on 2026-08-10. The two figures
this ledger attributes to it that my first extraction did not return —
the 30–35° stair slope and landings every 12–16 steps — were reproduced
verbatim on a targeted second pass, including the source's own derivation
`arctan(7/11) = 32 degrees`. **S-01 is clean; the ledger row was
accurate.** Checked because "the ledger said so" is not a check.

## Deliverable

**`MAP_METRICS.md` — WRITTEN 2026-08-10.** Closes `TRV-0149`; §9 closes
`TRV-0051` (traversal) and `TRV-0180` (ledge bands) explicitly.
`BLOCKOUT_PROCESS.md`, the other half of `TRV-0149`, is still unwritten.

## Quota: 1/12 counted (P: 1/3, V: 0/3). HONESTLY SHORT — and deliberately
not padded on 2026-08-10, when the dispatch was synthesis, not search.
The ONE tier-P source here was re-read end to end rather than a second
being added. Note that `vertical-maps/SOURCES.md` (14 counted, 5 tier-V)
covers the adjacent ground and upgrades this ledger's S-02 from
SNIPPET-ONLY — read that before opening this one again.
