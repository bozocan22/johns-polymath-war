# R&D Cycle 2 report — grenade surface friction

Per `briefs/PROMPT_RND_CYCLE.md` Section 4. Backlog item #3.

## 1. What was built

`surface_friction(kind: CoverKind) -> f32`, paired with the existing
`surface_restitution`, wired into `grenade_tick`'s bounce calculation
with the identical fallback rule restitution already had: a contact on
a known cover object uses that material's friction; a bare ground-plane
hit keeps the throw kind's own uniform friction, unchanged from before
this cycle. Values: Stone 0.30, Crate 0.45 (Hedge/Tree 0.60, defined
for completeness though practically unreachable — they stick on
contact before tangential friction would ever apply).

## 2. Evidence

[S-01] RoyMech tribology reference table, read directly (not trusted
from the search snippet — checked against the actual page content,
which included an honest "not found" for one queried pair rather than
inventing a number, a good sign the extraction is real). Real ranges:
metal-on-wood (dry, clean) 0.2–0.6; the closest metal-on-masonry proxy
found was concrete+rock at 0.3 sliding. The ranges overlap — the source
does not hand over a crisp single answer, and this is reported
honestly rather than oversold into more precision than the data
supports. Direction taken: worked masonry (this game's stone cover) is
smoother than a rough-grain wood crate, so stone sits at the low end of
its bracket and crate at the low-to-mid end of its (wider) bracket.

## 3. Tests

Command: `cargo test --release -p jk_tdm`
Before: 138 passed
After: **139 passed**, 0 failed, 2 ignored

New test `surface_friction_is_per_material_and_stone_skids_further_than_a_crate`
— fails on the pre-change code (`surface_friction` didn't exist). Three
parts: (1) the table values themselves and the stone<crate ordering,
(2) a **behavioral** proof, not just a table check — two grenades with
identical impact geometry, one bounced off stone, one off a crate;
runs real ticks (gravity included) until the bounce actually registers
rather than assuming a hand-picked tick count, then asserts stone's
post-bounce tangential speed is strictly greater, (3) a regression
check that ground-plane hits (no cover object) are byte-for-byte
unchanged — still use the throw kind's own uniform friction, with an
exact expected value (not a loose tolerance): the impact normal is
flat ([0,1,0]), so gravity only ever touches the normal velocity
component and the tangential vector at impact is provably [6.0,0,0]
regardless of fall duration, making the post-bounce speed an exact
prediction rather than an approximation.

## 4. Capture

None — this is a sim-layer physics coefficient change with no new
visible state (the grenade already bounced and rolled before this
cycle; the difference is HOW FAR it skids on different surfaces, which
existed as a mechanism already and is now materially differentiated).
Stated per R11 rather than claiming a capture proves something a
screenshot can't actually show (the difference is a distance-over-time
effect, not a single-frame visual).

## 5. Rejected alternatives

- **Full angular momentum / spin-affecting-bounce simulation** —
  rejected on cost/risk vs. value: this game already deliberately keeps
  grenade tumble COSMETIC/client-side only (a documented prior design
  choice), and adding real rotational dynamics to a fixed-timestep,
  replay-critical integrator is a much larger and riskier change for
  a bounce-feel improvement players are unlikely to perceive
  distinctly from the tangential-friction change already made. R10's
  "one system, researched properly" scope favored the smaller, certain
  win over a larger, uncertain one in the same cycle.
- **Adding mud/sand/snow/ice/water surface types** — rejected per R10's
  feasibility gate: none of these exist as a `CoverKind` or appear in
  any map today. Inventing physics coefficients for materials nothing
  in the game can currently be built from would be exactly the
  "researching for a renderer with no textures" category error this
  brief itself warns against.

## 6. Backlog delta

- Item #3: **done** (the two rejected extensions above are the honest
  remainder, not silently dropped — see #17 below).
- New item added: **#17, grenade rotational dynamics** (Low) — full
  spin/angular-momentum bounce physics, explicitly deferred with the
  reason above, not blocked on anything technical, just not judged
  worth the determinism risk yet.
- New item added: **#18, additional surface materials (mud/sand/snow/
  ice/water)** (Blocked) — unblocker: any map content actually using
  these `CoverKind`s. Researching coefficients for them now would go
  stale before any map could use them.

## 7. What was not done, and why

- The exact steel-on-masonry friction coefficient was not found as a
  single clean number — the source's own overlapping ranges are
  reported as-is rather than papered over with false precision.
- Rotational/spin physics and additional surface materials — see
  Rejected alternatives and Backlog delta above.

## Rotating codebase review (Section 5) — categories 5-8 this cycle

(Cycle 1 covered 1-4; continuing the rotation.)
5. *Determinism risks* — `surface_friction` is a pure function of an
   enum, called deterministically from the same `grenade_tick` path
   the R11 thousand-throw test already exercises; no new risk
   introduced. Re-ran the full suite including
   `a_thousand_identical_throws_land_bit_identically` and
   `preview_matches_throw_for_200_random_throws` — both still pass.
6. *Automated-test gaps* — this cycle's own test closes the gap that
   existed (friction was previously untested per-material because it
   wasn't per-material).
7. *Documentation claiming things that don't exist* — none found in
   the grenade module this pass.
8. *Error handling / panic surfaces* — `surface_friction` and
   `surface_restitution` are total functions over a closed enum
   (exhaustive match, no `_` arm, no `unwrap`); a future `CoverKind`
   variant would be a compile error here, not a silent gap.

No findings from this pass worth a separate backlog row.
