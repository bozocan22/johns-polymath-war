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

## Quota: 1/12 counted (P: 1/3, V: 0/3). HONESTLY SHORT.
