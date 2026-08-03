# Friday's log — the implementation agent

Append-only. What was built, what broke, what was deferred and why.
Written for the next Friday, who will have no memory of today.

Standing rules live in `.claude/agents/friday.md`.

---

## Inherited context — the codebase Friday is walking into

**Scale:** `jk_tdm` is ~22,700 lines across `sim.rs` (deterministic sim)
and `main.rs` (Bevy client). 145 tests, all green.

**Determinism is enforced, not aspirational.** Fixed 120 Hz tick, seeded
PCG32, and tests that compare 1000 grenade throws raw-bit and replay a
30-shot spray bit-identically. Anything added to `sim.rs` becomes replay
state.

**Repeated defect classes already burned by this project — do not
re-create them:**

1. **Player/bot parity.** The mech turn rate and the movement
   acceleration model BOTH shipped bot-broken first: the human paid a
   commitment cost the bot did not. Prefer one shared function.
2. **"The wall stop."** Velocity was written raw from input, so releasing
   a key stopped you dead in one tick. Fixed by `approach_velocity`
   (sim.rs) — but the doctrine says hunt it *everywhere*, and landings
   and other state transitions have not all been swept.
3. **The confident narrator.** Doc comments that claim a consumer which
   does not exist, or imply a dependency that algebraically cancels.
   Both have been found here. Say what the code does, not what it was
   meant to do.
4. **Self-referential tests.** A test that rebuilds the expression it
   claims to verify cannot fail. Assert against an independent table or
   a hand-computed value.
5. **Stale test setups.** When a test breaks, ask whether the ASSERTION
   is wrong or the SETUP was written for a world that no longer exists.
   Fixing a stale setup is legitimate and must be stated; weakening a
   correct assertion is falsification.

**Conventions:** tunables go in hand-rolled `key = value` text files
(`config/camera_tuning.txt`, `config/settings.txt`) — deliberately no
serde/RON dependency. Capture scripts for live verification: `baseline`,
`traversal`, `map_lap`, `mech_scale`, `minigun_check`, `idle_life`,
`bow_draw`, `menus`.

**Kill the running binary before rebuilding** — it locks the exe.

---

## 2026-08-03 — Thor's 7 defects in rig Step 1, fixed

Thor independently verified Step 1 and **upheld the central claim**: the
blast radius really is one 4% head-lag time constant, no consumer was
missed, `chain_peak_tick` really is test-only. Everything he found was
overstated certainty and two bad tests. No shipped constant moved.

**145 → 148 tests. 0 failed. 2 ignored.**

### The two that mattered

**D6 — a test guarding a lemma under the name of the theorem.**
`spear_followthrough_is_invariant_to_the_chain_tables` never called
`spear_followthrough_yaw`. It retyped that function's internal drive
expression and asserted the *copy* matched the algebra. **Proven, not
argued:** I checked out pre-change `main.rs`, deleted the `+ onset` from
the real function — the exact bug shipped once already (AUDIT.md, "bugs I
introduced this session" #1) — and the old test came back **ok**. Under
the rewrite it fails, along with two others.

Fix: did the refactor spec §Step 1 asked for and Step 1 skipped.
`chain_scale_from(onset, peak, elapsed, ramp)` and
`spear_followthrough_yaw_from(release_t, tip_onset, tip_peak)` are the
parameterised cores; the index-taking functions are thin wrappers. The
test now sweeps six substituted `(onset, peak)` rows through the **real**
function. Paired with a new golden-value test, because invariance alone is
vacuous — a function returning 0.0 is invariant to everything.

**D7 — nothing pinned the behaviour Step 1 changed.**
`the_head_trails_a_sprint_start_then_settles` derives its tick count from
`CHAIN_ONSET_OFFSETS[7]`, so it self-adjusts and passes for any value.
New `head_lag_chase_pins_the_measured_tip_onset` hand-computes from the
literal 0.130.

### The other five

- **D1** `chain_lag_chase`'s doc said the tip onset is 0.125 s, five lines
  under the constant that made it 0.130. Now it *refers* to the constant
  instead of restating it.
- **D2** "reduces **exactly**" was false in f32. Measured: worst drive
  residual **1.788e-7** (~14% of samples on a 10 µs grid), worst
  end-to-end divergence **2.98e-8 rad**. Exact in exact arithmetic; ~2e-7
  in f32; only 5.6× inside the test's 1e-6 tolerance, so the doc now says
  don't tighten it.
- **D3** spec §3.3 used `q = 0.8107`; the root of `q + q² + q³ = 2` is
  **0.81053571**. 0.8107 sums to 2.0007545 → arm hops 60.023 ms → tip at
  130.023, not 130. The spec also contradicted itself (claimed exactly 60,
  printed Σ = 60.03). **Shipped table unaffected** — both roots round to
  the same milliseconds, which is exactly why it went unnoticed.
- **D4** "the by-feel author was exact on the thorax" was circular
  corroboration. Index 2 = `0.040 − the 5 ms floor` for *any* inertia
  split. It measures a floor we chose. Struck from the spec and warned
  against in the table's own doc comment.
- **D5** the `gap_trunk`/`gap_arm` assertion read only indices 0/3/4, all
  pinned to 1e-6 twenty lines above — unfalsifiable — and its comment was
  backwards (0→3 is three hops at 13.3 ms, 3→4 is one at 30 ms; the chain
  **expands** there). Replaced with per-hop compression across the arm
  window, which is the only place an *interpolated* index can fail it.

### What the pre-change suite could not see

145/145 passed with the forearm (index 5) moved **6 ms**. Nothing
constrained the two interpolated arm indices at all. The new
`the_arm_onsets_reproduce_an_independently_solved_geometric_root` solves
D3's root by bisection *inside the test*, from `JAVELIN_ANCHOR_S` alone,
and catches a 1 ms move of either.

### Two things I got wrong on the way, both caught by running it

1. The spec's Step 1 table specifies the inertness test assert
   **bit-identical** equality. I wrote it, ran it, and it **failed** —
   `(t + onset) − onset` and `peak · x / peak` both round. The spec's
   assertion is false; corrected there rather than quietly loosened here.
2. Replacing D5's assertion, I first wrote a *trunk* per-hop check that
   read only indices 0/3/4 — the same unfalsifiable defect in a new
   costume. Deleted. The fact is now a comment, because an assertion that
   cannot fail is worse than a comment: it looks like coverage.

### Lesson (added to the anti-patterns above)

D6's shape is the dangerous one: a test named after the theorem that only
tests a lemma it retyped. The name is what makes it dangerous — it buys
the confidence of coverage it does not have. **A test for a function must
call that function.**
