# Spear / javelin throw — source ledger

Topic slug `spear-throw`. R4 depth floor: ≥1 tier-P source read end to
end. Recorded 2026-08-01.

**Provenance note (R3 honesty):** the extraction below was performed by
a parallel session working in the pre-migration repo path, and reported
to this session by the owner. It is recorded here because this repo is
now the canonical home and the research would otherwise be lost. A
verification agent was tasked in parallel with reaching the primary
paper independently — its verdict is appended at the bottom of this
file. Until that verdict says otherwise, treat the numbers as
**REPORTED-VERIFIED-ELSEWHERE**, not as independently re-read here.

| ID | Tier | Type | Title | Authors | Year | Venue | Status | What it gave us |
|---|---|---|---|---|---|---|---|---|
| S-01 | P | peer-reviewed 3D kinematic analysis | Some kinematic factors of the javelin throw | Campos, Brizuela, Ramón | 2004 | New Studies in Athletics 19(4), IAAF/World Athletics | READ (full PDF, 11 pages, read directly — WebFetch returned raw binary and correctly REFUSED to summarize rather than inventing content; the file was saved locally and read from disk) | 3D photogrammetric analysis of the **seven male finalists at the 1999 World Championships (Seville)**, throws 83.8–89.5 m, two synchronised cameras at 50 Hz. Per-athlete data, not averages. Full extraction below. |

## The measured kinetic chain — the headline result

Time from each segment's **peak velocity** to **release**:

| Segment | Mean | Range | CV |
|---|---|---|---|
| Hip | **0.13 s** | 0.12–0.16 | 11% |
| Shoulder | **0.09 s** | 0.08–0.10 | 16% |
| Elbow | **0.06 s** | 0.05–0.06 | 10% |

**Why this matters here:** `CHAIN_ONSET_OFFSETS` in `main.rs` was
authored by feel. These are measured. Converting peak-to-release times
into onsets from chain start (hip first, release at the end):

```
hip      onset 0.000   (peaks 0.130 before release)
shoulder onset 0.040   (0.130 - 0.090)
elbow    onset 0.070   (0.130 - 0.060)
release  at    0.130
```

The existing 8-entry curve spans 0.000–0.125 s, which is
**coincidentally almost exactly the measured 0.130 s total**. The
authored curve got the total duration nearly right and the internal
distribution wrong.

## Other extracted values

| # | Value | Number | Conditions |
|---|---|---|---|
| 1 | Elite release velocity | 28.1–29.7 m/s | 7 finalists, 83.8–89.5 m throws |
| 2 | Release angle optimum | 32–37° | — |
| 3 | Attack angle | within ±8°, ideally 0–2.5° | angle between velocity vector and javelin long axis |
| 4 | Energy concentration | **60% of javelin KE generated in the final 50 ms** | attributed in-paper to Morris & Bartlett 1995 |
| 5 | Hip peak vs front-foot plant | **−10 to −80 ms (hips peak BEFORE the plant)** | 6 of 7 throwers |
| 6 | Elbow at double support | flexes to 105–148° | then re-extends to 151–160° at release |
| 7 | Preparatory phase | 140–260 ms | — |
| 8 | Delivery phase | 100–140 ms | varies LEAST between athletes |
| 9 | Release velocity vs distance | r = .714, **p = .072** | n=7 — **NOT significant**, and the authors say so |

## What this validates, and what it contradicts

**Validates — our 26 m/s full-charge spear.** Elite javelin release is
28.1–29.7 m/s. A heavier, shorter war spear thrown by an armoured
soldier landing just below elite javelin is defensible. That number was
reached by feel and happens to sit in the right place.

**Validates — the elastic load model, strongly.** 60% of energy in the
final 50 ms, against a 240–400 ms total throw, means ~13–20% of the
duration carries 60% of the energy. BRIEF_VIII_B's "release must be
2–3× faster than load" is, if anything, **conservative**.

**Contradicts — `SPEAR_WINDUP_S = 0.40` as a single scalar.** The real
throw decomposes into preparation (140–260 ms, highly variable) and
delivery (100–140 ms, near-fixed by anatomy). Our code comment says
"plant, hips, whip" but the implementation is one countdown. Design
consequence: **charge level should stretch the preparation and never
the whip.** A rushed throw should look rushed in the wind-up, not in
the release.

**Contradicts — hips-follow-plant.** 6 of 7 throwers reach peak hip
velocity BEFORE the front foot plants. The unwind *precedes* the block;
it is not triggered by it. Any future plant-and-cut implementation that
fires hip rotation off the foot-plant event would be backwards.

## Open contradiction, deliberately left open

This paper reports **r = .714, p = .072** for release velocity vs
distance — high but not significant at n=7, and the authors flag it. A
search snippet claimed r = 0.90 from a different study. **Both are
recorded; neither is used to justify a tunable.** Only the shared
direction (velocity dominates, angle matters far less) is safe to
design against. This is exactly the R2/R3 discipline: an unresolved
disagreement between sources is recorded as unresolved.

## Findings NOT acted on yet (recorded, not silently dropped)

1. **No release-angle handling exists in `sim.rs`.** The throw uses raw
   aim direction, so the 32–37° optimum and the attack-angle window are
   never expressed. The good fix is a drag/attack-angle model that
   makes the optimum *emerge*, which is a design decision, not a patch —
   backlogged rather than bolted on.
2. **The ×1.15 running-throw bonus may belong to the PLANT, not the
   run.** The front leg's block is what converts horizontal momentum
   into javelin velocity; throwers who lost knee extension into release
   (one finalist at 137°) were measurably worse despite comparable
   everything else. Making the bonus conditional on a real plant would
   turn a flat multiplier into skill expression. Recorded as a design
   proposal, not applied.

## Independent verification

*(appended by the verification agent when it reports — see
`THOR_LOG.md` for the dispatch record)*
