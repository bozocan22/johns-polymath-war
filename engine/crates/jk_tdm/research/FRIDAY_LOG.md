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

---

## 2026-08-03 — D8–D11: the spec's own arithmetic, units, labels, and a new corroborating source
## (DOCUMENTATION ONLY — zero code, zero tests touched)

These four were dispatched mid-flight during the D1–D7 run and arrived
after I had already finished, so they never landed. Landed now. Files
edited: `research/body-rig/SPEC_20_SEGMENT_RIG.md` and this log. **Nothing
in `src/` was opened for writing** — Thor is reading `main.rs`/`sim.rs`
concurrently, and none of D8–D11 has a code counterpart anyway (verified:
`main.rs` has no `mod bsp`, no `SegmentId`, no `TOE_BREAK_Z` — only Step 1's
chain constants have shipped, so the spec's §2 tables and Steps 0/5 are
still pure plan text and cannot drift from code).

### D8 — a 10× arithmetic error, and the document contradicting itself

§2.2's sensitivity bound read
`I_clavicle ≈ 0.005 · (0.2887 · 0.109 · 1.78)² · M ≈ 1.4e-4·M`.

I recomputed it rather than trusting the dispatch:

```
0.2887 × 0.109 × 1.78 = 0.05601357
        squared        = 3.13752e-3
        × 0.005        = 1.5688e-5
```

**1.57e-5·M. The shipped figure was 10× too large.** The dispatch's
hand-derived 1.57e-5 is confirmed — I agree with it to every digit given.

The document had *already caught itself*: §3.3 independently computed the
same quantity as **1.50e-5**, and nobody reconciled the two. (The residual
1.57 vs 1.50 was D9 — same length, two statures.)

**Conclusion unaffected and strengthened, stated explicitly in the spec:**
the clavicle is *more* negligible than the claim it was supporting, so the
5 ms floor in §3.3(a) is better justified, not worse.

**But the order-of-magnitude phrasing was wrong too, and not in the
direction anyone expected.** Checked against §3.3's thorax:

```
I_UPT / I_clav = 1.976945e-3 / 1.5688e-5 = 126.0   → TWO orders, not three
```

"Three orders of magnitude below the thorax" was **never true** — under the
old (wrong) 1.4e-4 it would have been 1.3 orders; under the correct number
it is 2.1. Both spellings of that sentence were false. §2.2 now prints the
ratio instead of an adjective. Two further copies of the same bad claim
were hunted down: §6 item 1 said "1/1000th of the thorax's inertia" (it is
1/126), and **SOURCES.md's CONTRADICTION-3 says "four orders below the
thorax"**, which is also wrong. I did not edit SOURCES.md — it is Toto's
ledger — so that correction is recorded in the spec and flagged here.

### D9 — the same length in two unit systems, 2.2% apart

Clavicle length appeared as a bare `0.109` (a stature fraction, §2.1/§2.2)
and a bare `0.1898` (metres, §3.3). Same quantity. `0.109 × 1.741 = 0.1898`;
`0.109 × 1.78 = 0.1940`. So the document was silently assuming **two
different statures**: de Leva's reference **1.741 m** and the rig's
`BODY_HEIGHT` **1.78 m**.

I did not silently pick one. New **§2.0 STATURE CONVENTION** states both,
states each one's role, and picks:

- **1.741 m is a divisor and nothing else.** Its only job was converting
  de Leva's published mm into the dimensionless `len_frac` column. It is
  never multiplied back in.
- **`H = BODY_HEIGHT = 1.78 m` is THE evaluation stature**, because it is
  what ships: Step 6 calls `inertia_proximal(BODY_MASS_KG, BODY_HEIGHT)`
  and §2.4's proportion audit was already at 1.78. The fighter is 1.78 m
  tall.

§3.3's three inertias and Step 3's two `1/I` values were computed at
1.741 m and are restated at 1.78 m. **No shipped constant moves, and that
is provable rather than lucky:** every consumer of those numbers is a
*ratio*, `I ∝ H²` uniformly, so `H²` cancels. Verified by recomputing at
both statures — trunk split `0.46767 : 0.53233` both times,
`TWIST_LUMBAR_SHARE = 0.548` both times, hops 16.4 / 18.6 ms both times.

Rule added to the spec: **never write a bare length.** Write `0.109 H` or
`0.1940 m`.

**One apparent contradiction I resolved rather than "fixed".** §2.2 floors
the clavicle's *longitudinal* radius of gyration at 0.10, yet §3.3 uses
0.2887 for the clavicle in a block about longitudinal inertia. That looks
like a bug and is not: a clavicle's long axis runs **laterally**, so body-Y
twist is *transverse* to it and 0.2887 is the correct radius. The 0.10
floor applies to an axis the clavicle does not twist about here. Recorded
in §3.3 with the counterfactual (wrongly applying the floor gives
`I = 1.88e-6` and a 0.02 ms hop instead of 0.17 ms — **still floored to
5 ms, so no shipped constant depends on it**), because the next reader will
hit the same apparent contradiction.

### D10 — the clavicle row was labelled DERIVED. It is ASSUMED.

Toto's research (SOURCES.md BR-06/BR-07) established the negative result
properly: no measured clavicle inertia exists in de Leva, Winter,
Dempster, **or in the n=7 cadaver dissection dataset behind the paper
literally titled "Inertia and muscle contraction parameters for
musculoskeletal modelling of the shoulder mechanism"** — that dataset's
inertia tables have no clavicle row and no scapula row at all, and its own
methods say the inertias it does publish came from Clauser/Hinrichs
**regressions**. The Delft Shoulder Model's parameter file says
`REM scapular mass and rotational inertia roughly estimated!!` and gives
the scapula a literally isotropic tensor.

**Values kept, label changed.** DERIVED asserts falsifiability — change the
measurement and the number moves. There is no measurement to change. The
0.0050 stays because it sits inside DSEM's real bracket
[0.0021 bone-only, 0.0115 girdle per side], and our `Clavicle` *is* the
functional girdle since it parents `UpperArm`.

**The estimation method is named honestly rather than apologised for:**
`0.2887 = 1/√12` is the uniform-rod value, i.e. geometric-solid modelling
in the **Hanavan (1964) / Yeadon (1990)** lineage — a legitimate named
method, and the same family as Hatze (1980), the one whole-body model that
does segment the shoulder (paywalled, unread, the ledger's highest-value
gap). **The method was right; only the label overstated it.**

Structural consequence recorded: §2.1 now defines **three** label states
(MEASURED / DERIVED / ASSUMED) and says plainly that `SegmentDef.measured`
is a `bool` and **cannot express the distinction** — it spells DERIVED and
ASSUMED both as `false`. The `derived_rows_are_flagged_as_derived` set is
unchanged and still correct; the narrative label lives in the doc comment,
with a note to make it an enum if it ever needs to be machine-readable.

### D11 — the toe row is now corroborated by real measurement

Matsumoto et al. (2022), *Front Bioeng Biotechnol* 10:894731 (BR-04)
publish CT-derived foot segment masses: phalanx **14.4%** / forefoot 42.4%
/ hindfoot 43.2%. Our `Toe` == their phalanx.

Back-solving our own model's identity `m_t = 0.883 − s` (which is just the
CoM-closure relation) against their measured 0.144 puts the MTP break at
**s = 0.739**, versus our explicitly-unsourced **0.72**. Two routes sharing
no data, no subject, no modality and no decade — an n=100 gamma-ray
whole-foot CoM plus a geometric split, versus an n=1 CT segmentation —
agreeing to **0.019 of foot length: 4.2 mm on our 0.22 m rig foot, 5.0 mm
on de Leva's 0.2638 m foot.** Their phalanx `Iyy` gives `r/L = 0.260` vs
our assumed 0.2887, **−9.9%**.

**KEEP the shipped 0.163 / 0.72.** But I corrected the *reason* the
dispatch gave. The dispatch said keep 0.163 because "it reproduces de
Leva's measured CoM by construction". True — but **so would the switch**,
because `m_t = 0.883 − s` is an identity: change both together and the CoM
assertion still passes exactly. (It would even reconstruct de Leva's
measured whole-foot `r` slightly *better*: +3.4% vs our +3.9%.) I checked
this before writing it. The real reasons, now in the spec, are table
coherence (0.144 would put an n=1 CT ratio inside an otherwise
de Leva-consistent table) and immateriality (BR-04's own ±50% sensitivity
sweep found "virtually no effect" on joint moments).

Both cautions recorded in the spec:

- **Definitional trap (the fourth one).** Take BR-04's **masses only, never
  its CoMs.** Their axis is joint-centre-referenced — the hindfoot starts
  at the **ankle joint centre**, above and forward of the heel — and every
  `com_frac` in §2.1 is **heel-referenced**. Mixing them yields a number
  that looks right and is silently wrong.
- **Precision ceiling.** n = 1 (one 42-year-old male, 72 kg, 172 cm), CT
  volume at an **assumed** uniform 1.1 g/cm³ — volumetric, not gravimetric,
  no SD on the relative values. **Max 3 significant figures. Do NOT build
  per-fighter foot-mass variance on it** — the ±0.017 kg spreads are
  body-mass scaling across the cohort, not anatomical variation.

One new falsifiable assertion proposed for Step 0:
`the_foot_split_agrees_with_the_ct_measured_split` back-solves `s` from the
shipped toe mass and checks `|s − 0.739| < 0.03`. The shipped value gives
0.720, gap 0.019 — **0.011 of headroom upward**, so it is not comfortably
satisfied, which is the point. It is the only thing that would constrain
the toe mass at all, and its target comes from outside the crate.

Also fixed while in there: §0.4's audit table said the brief's 0.0035 toe
mass is inconsistent with de Leva's CoM. It is also **67% heavier than the
CT measurement** (24.1% vs 14.4% of foot mass) — a second, independent
falsification of the same number, now stated.

### Where I disagree with the dispatch, on the record

1. **"three orders of magnitude below the thorax" → the correct answer is
   TWO, not three.** The dispatch asked me to check whether the corrected
   number changes which order-of-magnitude claim is true. It does not —
   the claim was false before and after. The ratio is 126.
2. **SOURCES.md's "four orders below the thorax" (CONTRADICTION-3) is also
   wrong.** Not edited — Toto's file. Flagged in the spec and here.
3. **The stated reason to keep 0.163 was not quite right** (see D11
   above); the paired switch preserves the CoM assertion too. The spec now
   gives reasons that survive scrutiny.

Everything else in the dispatch reproduced exactly: 1.57e-5, 1.50e-5,
0.1898 vs 0.1940, s = 0.739, the 0.019 L / ≈5 mm agreement, r/L 0.260 vs
0.2887 within 10%, and the DSEM bracket containing 0.0050.

### What I could not do

- **Nothing was compiled or tested.** This dispatch is documentation-only
  and the file scope forbade touching `src/`. `cargo` was never invoked.
  The 148 tests from the D1–D7 run are assumed still green; I did not
  re-verify them and make no claim about them.
- **SOURCES.md's arithmetic error is unfixed**, by scope. It needs Toto or
  an explicit dispatch.
- **Hatze (1980) is still unread** (paywalled). It is the single source
  that could move the clavicle row off ASSUMED, and until someone reads it
  the row stays where it is.

### Lesson

D8 and D9 are one defect wearing two hats: a bare number written twice in
two unit systems. The document computed the same physical quantity in two
places, got answers that differed by 10×, printed both, and shipped. **A
quantity written without its unit is a quantity nobody can check** — and
the check that would have caught it (does §2.2 agree with §3.3?) was
available to every reader the whole time. Grep for the same physical
quantity before trusting either instance of it.
