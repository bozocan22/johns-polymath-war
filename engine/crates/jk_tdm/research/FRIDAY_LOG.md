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

> **[ERRATUM — 2026-08-03, see "F1–F4" below. Read this before quoting the
> paragraph above.]** That claim is true and is about *one test*. It is
> not a claim about the suite, and it was read as one. Thor ran the
> pre-change **suite** under the same mutation: **144 passed, 1 failed** —
> `spear_followthrough_carries_past_the_release_then_settles` caught the
> bug by another route. So D6 took detection from **one test to three**
> and moved it onto the function's own contract. Real hardening; **not** a
> rescue from zero coverage. Annotated rather than rewritten, because this
> log is append-only. (The forearm claim two sections down is unaffected
> and stands: 145/145 green with index 5 moved 6 ms means the suite
> genuinely was blind there.)

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

---

## 2026-08-03 — F1–F4: Thor's two defects, plus the hazard I named wrong

Thor reproduced all eight of my mutation proofs with **zero
discrepancies**, then found two defects I missed and — more usefully —
showed that the risk I called my most fragile was near-zero while the one
I never named was the real one. Everything below is measured in a
standalone f64/f32 harness compiled **outside the crate**, then proved
in-tree by mutation.

**149 passed, 0 failed, 2 ignored.** (The suite is 151 tests now, not 150:
`c7d93b0` landed `mech_brace_is_gated_slowed_and_damped_by_its_own_constants`
in `sim.rs` while I was mid-run. Nothing of mine touches it.)

### F1 — GOLDEN's provenance was a comment, and a comment is not a guard

This is the one that mattered, and Thor named it correctly. The old test
carried seven literal output values under a comment saying they had been
computed outside the crate. **Nothing enforced that.** The cheapest move
available to the next maintainer who breaks the curve is to print the
code's own output, paste it over the literals, and watch the test go
green — at which point it has silently become a change-detector that pins
the bug. Same defect class as the D6 test it replaced.

Chose **(c) both**, but the two halves do different jobs and only one of
them is load-bearing. The curve is now a **triangle**:

1. `closed_form(t)` — the spec as an f64 expression containing **no crate
   item at all**: not `SPEAR_RELEASE_YAW`, not the chain tables, not the
   four local consts. Literally
   `(0.35 + 0.10*min(t/0.12,1)) * exp(-6*max(t-0.05,0))`.
2. `ANCHORS` — seven f64 values at 15 dp, computed from that expression
   outside the crate, asserted against `closed_form` at **1e-12**.
3. The real shipped f32 `spear_followthrough_yaw`, swept against
   `closed_form` on a 1 ms grid over 0..1.2 s (1201 points, up from 7),
   and checked against `ANCHORS` directly. Both at 1e-6.

**Why regeneration now fails loudly.** The f32 function and the f64 closed
form agree only to ~4e-8, so anchors pasted from this crate's own output
miss the 1e-12 gate by a factor of **41,697** — and the assertion message
says exactly that, by name. There is no table anywhere in the test that
can be regenerated from the code to silence a real failure: break the
function and (3) fails; paste the broken output into `ANCHORS` and (2)
fails too. The only remaining route is editing `closed_form`, which is
visibly editing the *specification* rather than refreshing a table. That
distinction is the entire fix — the old shape could not make it.

**PROVED BY MUTATION, three runs, and the third is the one that matters:**

| # | mutation | result |
|---|---|---|
| F1-a | drop `+ tip_onset` from the real function | **146 passed, 3 failed** |
| F1-b | …**and** regenerate `ANCHORS` from the broken code's f64 output | **146 passed, 3 failed** — still caught, at the 1e-12 gate, message fires |
| F1-c | **counterfactual**: `git checkout` the OLD test shape, drop `+ tip_onset`, regenerate the f32 `GOLDEN` literals from the broken output | **147 passed, 2 failed** — `spear_followthrough_matches_its_hand_computed_curve` came back **ok** |

F1-c is the proof that the hazard was real and not theoretical: under the
old structure the regeneration attack **works**, and the golden test
happily certifies a broken curve. Under the new one it does not.

### F2 — units, in the block written to fix a units-ish overstatement

`main.rs` divided a **drive-term** residual (1.788e-7, dimensionless,
0..1) by a tolerance applied to **yaw** (1e-6 rad) and called the quotient
"~5.6x". That is not a margin of anything. The sentence supplies its own
correct conversion one clause earlier and then does not use it. The same
file said **33x** for the same quantity six thousand lines away.

Like-for-like, measured:

- drive residual expressed as yaw: `1.788e-7 × OVERSHOOT_RAD` = **1.79e-8
  rad** → 56x, if that were the whole story;
- what the test actually sees — worst end-to-end divergence across the six
  table variants, on the 1 ms grid the test itself sweeps — is **2.98e-8
  rad** → **34x**. Bigger than the drive residual alone, because the
  divide, the add and the decay multiply each round on top of it.

**34x is now the single figure the file quotes**, in both places. I added
one thing Thor did not ask for, because it changes the advice: the margin
is **grid-dependent**. On a 1 µs grid the worst divergence is **5.96e-8
rad → 17x**. So the "do not tighten toward 1e-7" warning is stronger than
it looked — at 1e-7 that off-grid worst case leaves **1.7x**, and the test
becomes a coin flip the moment anyone refines the sweep. That is now
written down next to the tolerance.

Thor's 8,290 / 1.788139e-7 / t=0.09359 / exactly-zero-after-saturation all
reproduce to the digit.

### F3 — an assertion whose message claimed more scope than it had

The per-hop arm-compression check (D5's replacement) ended its failure
message "indices 5 and 6 are the interpolated ones", which reads as a
claim that it constrains them at the table's resolution. It does not.
Stepping one index at a time with the other seven fixed, monotonicity
survives across:

```
idx5 in [0.093 .. 0.098]   (shipped 0.094: -1 / +4 ms slack)
idx6 in [0.112 .. 0.117]   (shipped 0.114: -2 / +3 ms slack)
```

So it fires on idx5 down >=2 ms or up >=5 ms, idx6 down >=3 ms or up
>=4 ms — and on **neither** at 1 ms. Thor's measurement (idx6
0.114 -> 0.115 passes) reproduces exactly.

**I narrowed the message and left the assertion alone**, deliberately, and
this is the one place I chose against the more impressive-looking option.
The only way to broaden this check to 1 ms is to re-derive the geometric
ratio inside it — which is
`the_arm_onsets_reproduce_an_independently_solved_geometric_root`'s entire
job. A second copy would be redundancy wearing the costume of coverage:
the same error class, in a new place, added while fixing that error class.
What was false was the **claim**, so the claim is what changed. The
message now states the numeric windows and names the test that does cover
1 ms.

**PROVED BY MUTATION, at both edges of the claim:**

| # | mutation | result |
|---|---|---|
| F3-a | `CHAIN_ONSET_OFFSETS[6]` 0.114 → **0.115** (inside the stated window) | **148 passed, 1 failed** — only `geometric_root`; the arm-hop test passes, exactly as the new message says |
| F3-b | `CHAIN_ONSET_OFFSETS[6]` 0.114 → **0.119** (outside it) | **147 passed, 2 failed** — the arm-hop assertion fires, as the window predicts |

A message that states a bound is only worth having if the bound is right
at both ends, so both ends were tested.

### F4 — a stale number in my own doc

`main.rs` said the golden test's worst f32-vs-f64 gap was "5.5e-8, so 1e-6
leaves ~18x". Thor measured **5.96e-8 / 16.8x**; confirmed to the digit.
Corrected, and superseded — carrying the anchors at f64 instead of
7-significant-digit f32 recovers the rounding those literals threw away,
so the current margins are:

- real f32 fn vs `closed_form`, 1 ms sweep 0..1.2 s: worst **6.00e-8** at
  t=0.115 → **16.7x**
- real f32 fn vs `ANCHORS` at the seven points: worst **4.17e-8** at
  t=0.06 → **24x**

### Thor's verdict on my three volunteered risks: I agree on all three

Not conceding to be agreeable — I re-measured each one.

**(a) `exp()` bit-stability — agreed, overstated, and I ranked it worst
when it is nearly the best.** Thor's ulp framing is the right one and I
did not think of it: the test does not need bit-stability, it needs ~37
ulp, and real `expf` is within ~1. I checked the one thing that could have
changed the answer — my new dense sweep calls `exp` at 1201 points instead
of 7, so I re-derived the binding constraint by perturbing the decay
ULP-by-ULP until 1e-6 breaks: **36 ULP, at t=0.115**, versus Thor's 37 for
the old 7 rows. Widening the sweep did **not** widen the exposure.
(The naive sweep-wide minimum is 21 ULP at t=0.048 — but that point sits
before `HOLD_S`, where the argument is exactly 0 and `exp(0)==1.0` is
exact in every conforming libm. Excluding the zero-argument points is what
gives 36. Worth stating, because 21 is the number you get if you run the
measurement without thinking about it.)

**(b) the 1.58x geometric-root margin — agreed, and I misframed it.** My
numbers were right and my category was wrong. A margin is a risk only if
the quantity varies; that test is a 200-step bisection over IEEE-754
doubles with no transcendental and no libm call anywhere in it, so it is
bit-deterministic on every conforming target. The 1.58x is the distance of
the true 0.0943161 from the 0.0945 rounding boundary — a fact about the
anthropometric data, not a property of the test. And the tightness is a
**virtue**: F3-a above only fails because that tolerance is tight. Loosen
it and you delete the only 1 ms guard on index 6.

**(c) "7 numbers to retune" — Thor agreed with the engineering and named
the failure mode I missed.** Correct, and it is the most valuable thing in
his review. I flagged the *cost* of the golden table and never asked the
obvious next question: what stops someone paying that cost the cheap,
wrong way? F1 above is the answer.

**The pattern, stated plainly because it is the actual lesson:** all three
misses are the same shape. I audited the numbers I had written and not the
*process by which the next person will change them*. (a) and (b) are risks
to a value; (c) is a risk to a workflow, and that is where this was going
to erode. My evidence was clean and my risk ranking was inverted — I was
hardest on the parts that hold up best.

### Lesson

**A comment asserting provenance is not a guard; it is a note asking to be
ignored.** If a test carries numbers that came from outside the code, the
test must be able to *detect* numbers that came from inside it. Here the
detector was free: f32 output and f64 truth differ by ~4e-8, so a 1e-12
gate on an independently-derived closed form separates them by four orders
of magnitude. Ask of every pinned table: *what does the maintainer do when
this goes red, and does the cheapest thing they can do leave it honest?*

### What I could not do / am least sure about

- **`ANCHORS` is now cross-platform-sensitive in a way the old table was
  not**, and this is the thing I would look at first if CI ever goes red
  on a new target. The 1e-12 gate compares f64 `exp()` against 15-dp
  literals. Slack is ~20,000 ULP of f64 `exp` — enormous, and no
  conforming libm is anywhere near it — but it is a *tighter* coupling to
  libm than the old 1e-6 f32 comparison, and I verified it on exactly one
  toolchain (rustc 1.97.1, x86_64-pc-windows-msvc). Measured, not assumed:
  4 of the 7 anchors call `exp` at all; the other 3 are at or before
  `HOLD_S` and are exact. If it ever fires on a new platform the fix is to
  loosen 1e-12 toward 1e-10 — **not** to touch the anchors, which is
  precisely the move the gate exists to stop. Written here because the
  assertion message cannot say it.
- **F3 leaves indices 5 and 6 guarded at 1 ms by exactly one test.** That
  is a deliberate choice against redundancy, not an oversight, but it does
  mean `geometric_root` is a single point of failure for both interpolated
  indices. If it is ever weakened, they become unguarded.
- **I did not re-verify Thor's ulp figures for the four old golden rows
  individually** (39/37/146/9399). I reproduced the *binding* constraint
  under my new structure and it agrees; I did not check the other three.
- **`main.rs` only.** `sim.rs` untouched; `SOURCES.md` untouched (Toto's);
  `TOTO_LOG.md` had uncommitted changes from another agent throughout and
  I left it alone — every revert in this session was a targeted
  `git checkout -- engine/crates/jk_tdm/src/main.rs`, never a bare
  `git stash`.

---

## 2026-08-03 — Mech plan §C: the hull gatling + autocannon (`sim.rs` only)

**Baseline at session start: 149 passed, 0 failed, 2 ignored.**
**End: 160 passed, 0 failed, 2 ignored.** The delta is +6 from me
(`sim::tests` 91 → 97, verified by name) and +5 that appeared in
`main.rs`'s test modules from another agent working the same tree
concurrently. Every revert in this session was a targeted
`git checkout -- engine/crates/jk_tdm/src/sim.rs`; never a bare
`git stash`, and `main.rs` was never opened for writing.

### What shipped

- `MechWeapon { Gatling, Autocannon }` — a dedicated enum, **not** two
  new `GunKind` variants. `GunKind` is the spine of the *infantry* weapon
  pipeline (`gun()->GunSpec`, `ALL_WEAPONS`/`N_WEAPONS` and the
  `vm.weapons[N_WEAPONS]` viewmodel array, `weapon_slot`, `reload_pose`,
  the loadout screen's `PRIMARIES`/`GunClass` tables, the punch-slot map)
  and every one of those sites encodes *carryable, swappable,
  loadout-selectable*. The hull mounts are structural: always both
  present once the chassis seals, never picked up, never reloaded, never
  in a loadout, no hand pose. Adding them to `GunKind` would have meant
  answering ~8 exhaustive matches with lies.
- Four `Fighter` fields (`mech_weapon`, `gatling_heat`, `gatling_vent_t`,
  `autocannon_cd`), reset at the death/respawn site alongside
  `mech_brace`, at the `RobotArmor` pickup (a *fresh* chassis must not
  inherit a vent lockout the last one earned), and initialised in the
  constructor.
- Constants per plan. **No `AUTOCANNON_BRACED_KICK`** — the braced value
  is derived at the call site as
  `AUTOCANNON_UNBRACED_KICK * MECH_BRACE_RECOIL_DAMP`, consuming §A's
  damp exactly as §A said it was built to be consumed.
- `try_fire_gatling` / `try_fire_autocannon` as **siblings** of
  `try_fire`, not branches inside it. `try_fire`'s gate list is
  `armed()`/`gun`/`ammo`/`reload_t`/`switch_t`/`shield_up`/`knife_phase`/
  `flip_t`/`sprint_gate_t` — the state of a *pair of hands*. None of it
  describes a gun bolted to a chassis and triggered from a sealed
  cockpit.
- Input: in a chassis the trigger drives the hull mount, and 1/2 pick
  which one — the same key-repurposing §A did with crouch. The infantry
  slot path is now gated on `!in_mech()`; without that half, a number key
  kept switching the pilot's carried gun invisibly underneath the mech
  and burning a `SWITCH_S` he could neither see nor use.

### The extraction — behaviour-preserving, and it exposed a real defect

The plan was right that there was no helper: the per-pellet hitscan loop
was inline in `try_fire`. It is now
`hitscan_burst(i, o, aim, spread, damage, pellets)`, lifted verbatim,
with damage/pellets/spread as parameters instead of `GunSpec` reads —
which is precisely what lets a weapon that *has no `GunSpec`* use it. The
punch deflection came out the same way as `punched_aim`. **Zero changes
to any existing test; all 149 pre-existing tests stayed green.** RNG
order (two draws per pellet, x then y) is preserved exactly, which is
what the determinism/replay tests were checking.

Then the extraction paid for itself immediately. Writing the sixth test I
found that `apply_hit` **re-derived the damage from
`gun(self.fighters[i].gun).damage`** at the bottom of the chain. So my
first cut had both hull mounts dealing their correct damage to *zombies*
(which take the passed value) and the **pilot's carried rifle damage** to
*fighters*. The autocannon's 145 would simply never have reached a mech
hull. Fixed by splitting `apply_hit_dmg(..., base_dmg)` out with
`apply_hit` as a thin wrapper passing `gun(held).damage` — so all 20-odd
existing call sites keep their meaning and read unchanged.

That defect is the whole argument for the extraction, in miniature: the
duplicate path would have looked correct at the call site and been wrong
two functions down, and the zombie path would have hidden it for months.

### Mutation proofs — 6 tests, 6 mutations, each reverted

| # | Test | Mutation applied to `sim.rs` | Result |
|---|---|---|---|
| 1 | `gatling_heat_ramps_slower_than_minigun_in_absolute_terms` | `f.gatling_heat += GATLING_HEAT_PER_SHOT` became `+= MINIGUN_HEAT_PER_SHOT` | **FAILED. 157 passed; 1 failed** |
| 2 | `gatling_spread_widens_with_heat` | replaced the cold-to-hot lerp with a flat `GATLING_SPREAD_COLD` | **FAILED. 157 passed; 1 failed** |
| 3 | `autocannon_kick_is_damped_exactly_by_mech_brace_recoil_damp` | `let kick = if f.mech_brace {damp} else {full}` became `let kick = AUTOCANNON_UNBRACED_KICK` | **FAILED. 157 passed; 1 failed** |
| 4 | `autocannon_and_gatling_are_mutually_exclusive_by_mech_weapon` | deleted the `f.mech_weapon != MechWeapon::Gatling` clause from the gatling's gate | **FAILED. 157 passed; 1 failed** |
| 5 | `mech_weapons_refuse_to_fire_for_non_mech_fighters` | `!f.in_mech()` became `false` in **both** fire functions | **FAILED. 157 passed; 1 failed** |
| 6 | `hull_mounts_carry_their_own_damage_down_the_shared_hit_path` | `apply_hit_dmg(i, j, hit_y, end, damage)` became `apply_hit(i, j, hit_y, end)` — i.e. restore the original defect | **FAILED. 159 passed; 1 failed** |

Every mutation was reverted with
`git checkout -- engine/crates/jk_tdm/src/sim.rs` (1-5, against the
already-committed implementation) or an exact reverse edit (6), and the
suite was re-confirmed green afterwards.

Test 6 asserts the autocannon:gatling hull-damage **ratio**, not either
absolute number — the angle multiplier, armour floor and every other
shared stage cancel, so what is left under test is exactly "did the right
damage reach the shared resolver". It also fires each mount while the
pilot holds a *different* infantry gun, so a restored `gun(shooter.gun)`
re-read diverges by the held weapons rather than by the mounts.

### What I am least sure about

1. **The gatling heat decay is gated on `fire_cd <= 0.0`, which the plan
   did not ask for, and it is the judgement call I would most want
   reviewed.** The plan said "add the decay alongside the existing
   minigun `f.heat` decay". Taken literally as *unconditional* per-tick
   decay, `GATLING_HEAT_DECAY` (9.5/s) eats most of the ramp
   (0.9 per 0.07 s = 12.9/s), stretching time-to-forced-vent from ~7.8 s
   to ~30 s and destroying the "sustains about twice the minigun's 4 s"
   intent by a factor of four. So I gated it, mirroring how the minigun
   gates *its* decay on the trigger hold-timer: a barrel group under fire
   does not cool. But the minigun uses a dedicated `spin_cmd` hold timer,
   and I am leaning on `fire_cd` — which is *shared with the pilot's
   carried gun*. Two consequences I accept but flag: (a) a pilot who just
   fired his rifle briefly stops the hull mount cooling, and (b) the
   ~9.4 s real sustain depends on the tick granularity of `fire_cd`
   (4 ticks hot, 1 tick cool per 0.07 s cycle at 60 Hz), so changing
   `SIM_HZ` or `GATLING_FIRE_PERIOD` moves it. A dedicated
   `gatling_trigger_t` hold timer would be the clean fix; the plan fixed
   the field list at four, so I did not add a fifth without asking.
2. **Nothing tests the sustain duration end-to-end.** Test 1 pins the
   per-shot ramp behaviourally and the time-to-vent *relationship*
   arithmetically, but there is no test that steps the sim with the
   trigger held and measures when `gatling_vent_t` actually latches.
   That is exactly the number item 1 makes fragile, and it is the gap I
   would close first.
3. **I did not wire the BOT fire path.** §C.5 named the `cmd.fire` site
   and I did that one. Bot-piloted mechs (`sim.rs`, the bot `try_fire`
   call) still fire their carried infantry gun, so an AI mech does not
   use its hull mounts at all. Deliberate — changing it moves bot damage
   output and would ripple into the seeded determinism tests — but it
   means the feature is player-only right now, and someone should decide
   whether that is the intended shipping state.
4. **The mounts emit `gun_noise_m(GunKind::Minigun)` in Extraction
   mode.** Not in the plan. I added it because a silent mech is a real
   defect against the horde director, but it is the one place §C touches
   `GunKind` at all. It is a radius *lookup*, not an entry into the
   pipeline, and the alternative (a dedicated constant) was more surface
   than the plan authorised. Untested.
5. **The autocannon deliberately does not touch the spray table.** The
   deterministic per-weapon spray patterns are `GunKind`-indexed infantry
   state; a single-shot hull cannon has no pattern to walk. It adds one
   honest `punch_vel[0]` kick. If the design intended the autocannon to
   participate in spray recovery, that is not implemented.
