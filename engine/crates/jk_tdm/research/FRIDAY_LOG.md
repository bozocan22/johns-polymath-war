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

---

## 2026-08-03 — Thor's §C review: the three surviving mutations, the heat gate rebuilt, two cross-talk bugs (`sim.rs` only)

**Baseline at session start: 162 passed, 0 failed, 2 ignored.**
**End: 165 passed, 0 failed, 2 ignored** (+3, all mine, all in
`sim::tests`). Every mutation in this session was reverted with a
targeted `git checkout -- engine/crates/jk_tdm/src/sim.rs`; never a bare
`git stash`. `main.rs` was never opened for writing.

### The headline: my §C shipped with an unfalsifiable core

Thor measured three mutations that passed the entire suite:

| Mutation | Before this session |
|---|---|
| ungate the gatling heat decay (sustain 9.08 s → 29.8 s) | **161/0, SURVIVED** |
| silence both hull mounts against the horde | **161/0, SURVIVED** |
| delete the ENTIRE forced-vent mechanism | **161/0, SURVIVED** |

`gatling_vent_t` was never set non-zero anywhere in the suite. The
gatling's whole identity — heat-limited sustain — could be deleted
without a single test noticing.

### F1 — the root cause was a self-referential test with a false comment

`gatling_heat_ramps_slower_than_minigun_in_absolute_terms` closed with:

```rust
let gat_s  = 100.0 / (GATLING_HEAT_PER_SHOT / GATLING_FIRE_PERIOD);
let mini_s = 100.0 / (MINIGUN_HEAT_PER_SHOT / gun(GunKind::Minigun).fire_period);
assert!(gat_s > mini_s * 1.5, "...");
```

I labelled that "TIME TO A FORCED VENT". It is not a time to anything —
it steps nothing, it cannot see the heat **decay**, it cannot see the
**vent latch**, and it cannot see that the fire period is quantised to
whole ticks. It evaluated to **7.78 s** against a real measured
**9.08 s**. Worse, it was the **only** mention of time-to-vent in the
suite, which is exactly why two of the three mutations above survived:
the number they broke was never read out of the sim.

That block is deleted (with a comment recording what stood there and why
it was worthless). In its place, stepping the sim:

`gatling_sustains_about_twice_the_minigun_before_a_forced_vent` holds
the trigger through `step()` and records the tick `gatling_vent_t`
latches, then takes the **same** measurement off the man-portable
minigun and asserts the *ratio* band the design sentence states
("about twice the minigun"), not a magic constant. It also asserts the
latch does something: the mount is locked out while venting, the vent
ends, and it ends with the pool at zero.

### F2 — my instinct on the heat gate was right; my stated mechanism was wrong

I wrote that gating decay on `fire_cd <= 0.0` "mirrors how the minigun
gates its decay on the trigger hold-timer." It does not, and Thor
measured the gap:

- `spin_cmd` is set **before every early return** in `try_fire`, so
  while the trigger is held it is *always* > 0 → the minigun's decay is
  suppressed **100%**.
- `fire_cd <= 0.0` is true for **exactly one tick per fire cycle**, so
  the gatling still shed `GATLING_HEAT_DECAY * DT` every cycle —
  0.0792 heat per shot, **88.9% suppression, not 100%**.

That residual is **linear in DT**, so my gate made the gatling's whole
identity tick-rate dependent: **11.18 s @ 60 Hz, 9.08 s @ 120 Hz,
8.22 s @ 240 Hz — a 36% swing.** The shipped 9.08 s did hit the design
intent (2.06× the minigun's 4.417 s), but by an accident that moves with
`SIM_HZ`.

Fixed as Thor recommended: a dedicated `gatling_trigger_t` hold timer,
same 70 ms and same shape as `MINIGUN_SPIN_HOLD_S`, set at the top of
`try_fire_gatling` **before** every early return. The decay now gates on
it, and the cycle clock moved off `fire_cd` to its own `gatling_cd`.

**Measured after the fix** (by temporarily setting `SIM_HZ` in
`jk_core/src/timestep.rs`, running only this test, and reverting with
`git checkout -- engine/crates/jk_core/src/timestep.rs`):

| SIM_HZ | gatling sustain | minigun sustain | ratio |
|---|---|---|---|
| 60 | 9.267 s | 4.433 s | **×2.090** |
| 120 | **8.333 s** | **4.417 s** | **×1.887** |
| 240 | 7.867 s | 4.133 s | **×1.903** |

**Correction to Thor's phrasing, which I want on the record: this is not
"tick-rate stable".** It removes the *decay* term's DT dependence, which
was the bug. What remains is the ceil-quantisation of the fire period
itself — `ceil(0.07/DT)*DT` is 0.0833 s at 60 Hz, 0.075 s at 120 Hz,
0.0708 s at 240 Hz — a **17.8% swing** in absolute sustain (down from
36%). The minigun carries the same quantisation (4.13–4.43 s), which is
precisely why the new test asserts a **ratio band** (1.5× to 3.0×): the
ratio moves only 1.887–2.090 across all three rates, and the test passes
at every one of them.

### F3 — two cross-talk bugs, the second of which I did not disclose

1. **`fire_cd`.** A gatling round set `f.fire_cd = 0.07` — the
   **pilot's carried-gun** clock. A pilot who dismounted mid-burst found
   the rifle in his hands throttled by a gun bolted to a chassis he had
   just climbed out of. `sim.rs:1399-1401` *already states this rule* at
   the declaration of `autocannon_cd` ("its own field, not `fire_cd`, so
   the two mounts cannot silently share a cooldown"). The gatling was
   the lone violator of a rule the file itself writes down. Fixed by
   giving it `gatling_cd`.
2. **`last_shot_at` — undisclosed by me, found by Thor.** Both mounts
   wrote it. Its **only** consumer is the carried gun's `spray_i` decay
   gate (`t - last_shot_at > gun(f.gun).fire_period * 1.1`). The mount
   refires every **0.075 s**, faster than *every* gun's threshold (the
   AK's is 0.1155 s), so **`spray_i` could not decay at all while the
   hull gatling fired**. The pilot dismounted into a fully-bloomed
   recoil pattern he had not fired one round to earn. Fixed by deleting
   the write from **both** mounts — a hull mount has no spray table to
   walk, which is the reason the autocannon already refuses to touch it.

Both are pinned by
`firing_the_hull_mounts_leaves_the_pilots_carried_gun_untouched`, and
each was mutation-proven **separately**.

### F4 — the horde noise is now falsifiable

`hull_mounts_are_heard_by_the_horde` places one zombie 4 m from the
muzzle and one 398 m away, fires each mount in Extraction, and asserts
the near one is alerted **and re-targeted onto the muzzle position**
while the far one is not. Both zombies sit off the fire axis, so the
only thing that can alert them is the noise, not a round.

### F5 — three corrections to my last entry

1. I wrote that "all 20-odd existing `apply_hit` call sites read
   unchanged." Accurate statement: they are **all inside `mod tests`**.
   After the `apply_hit_dmg` split, **`apply_hit` has zero production
   callers.** It survives as a test-facing wrapper. That is fine, but it
   is not what "20-odd call sites" implies.
2. I **under**-claimed the defect I found. `apply_hit` re-derived from
   the held gun in **two** places — the zone multiplier (~5699) and the
   armour floor (~5711) — and I described one. I fixed both.
3. `GATLING_FIRE_PERIOD = 0.07` does **not** mean 857 RPM. `gatling_cd`
   is decremented by `DT` and the gate opens at `<= 0`, so the real
   period is `ceil(0.07/DT)*DT` = **0.075 s = 800 RPM** at the 120 Hz
   floor (720 RPM at 60 Hz, 847 at 240 Hz). Now documented at the
   constant, before someone tunes it against a published RPM figure.

### Mutation proofs — every one applied, run, recorded, reverted

| # | Mutation | Test that caught it | Result |
|---|---|---|---|
| **d** | ungate the heat decay (`else if f.gatling_trigger_t <= 0.0` → `else`) | `gatling_sustains_...` | **FAILED. 164 passed; 1 failed** |
| **e** | silence both mounts vs the horde (both `emit_noise` calls removed) | `hull_mounts_are_heard_by_the_horde` | **FAILED. 164 passed; 1 failed** |
| **f** | delete the ENTIRE forced-vent mechanism (latch + countdown + lockout gate) | `gatling_sustains_...` | **FAILED. 164 passed; 1 failed** |
| **g** | restore cross-talk 1: the gatling writes the pilot's `fire_cd` again | `firing_the_hull_mounts_leaves_...` | **FAILED. 164 passed; 1 failed** |
| **h** | restore cross-talk 2: the gatling writes `last_shot_at` again | `firing_the_hull_mounts_leaves_...` | **FAILED. 164 passed; 1 failed** |
| **i** | the forced vent ends without clearing the heat pool | `gatling_sustains_...` | **FAILED. 164 passed; 1 failed** |

**d, e and f are Thor's three survivors.** All three now fail. Suite
re-confirmed at 165/0/2 after the final revert.

### What I am least sure about

1. **Moving the gatling off `fire_cd` has a client-side consequence I
   cannot see from inside my file scope, and it is the thing I would
   most want checked.** `main.rs` detects fresh shots by watching
   `fire_cd` **jump up** — at least five such sites (casing ejection
   ~3043-3057, scope flinch ~5818, muzzle/audio ~5947-5951, ~8801-8802,
   ~8853-8855) plus view-model kick and HUD "in combat" reads at
   ~6313/~6429/~8039/~8160. The hull gatling was accidentally feeding
   all of them. After this change it no longer does, so **the hull
   gatling may have lost its muzzle flash, casings and shot audio.** I
   judged the change correct anyway — the autocannon has *always* been
   invisible to those sites (it has used `autocannon_cd` since §C
   landed), so this makes the two mounts consistent rather than creating
   a new problem class, and `spec.fire_period - p.fire_cd` at ~8039 was
   already computing garbage for a pilot (it mixes the mount's 0.07 with
   the *carried* gun's period). But the hull mounts' FX now need to key
   off `gatling_cd`/`autocannon_cd`, and that is a `main.rs` edit I am
   not permitted to make. **Someone should dispatch it.** If mech
   shooting looks silent in game, this is why.
2. **The ratio band (1.5×–3.0×) is wide on purpose and the upper bound
   is currently redundant.** On today's numbers the 25 s loop cap is what
   catches the ungated-decay mutation (it runs ~40 s), not the `< 3.0×`
   assert. I kept the upper bound so that raising the cap later cannot
   silently disarm the test — but it has not itself been shown to be
   load-bearing, and I would rather say so than imply six independent
   guards.
3. **The 60/120/240 Hz measurements required editing
   `jk_core/src/timestep.rs`, which is outside the file scope I was
   given.** I treated it as a measurement instrument (same class as a
   mutation): change, run one test, revert immediately with a targeted
   `git checkout`, verify `SIM_HZ` is back to 120 and `git status` is
   clean. It was. But there was another agent in this tree and a broken
   `DT` would have broken their build for the ~30 s it took; if that was
   the wrong call, it was mine.
4. **The minigun comparison pre-spins the barrels** (`spin_t =
   MINIGUN_SPINUP_S`) so both sides measure *barrel* time to cook-off.
   Including the minigun's 0.4 s spin-up would push it to ~4.8 s and the
   ratio to ~1.74 — still inside the band, but it is a judgement call
   about what "sustain" means and I made it inside the test setup. It
   matches how Thor measured 4.417 s.
5. **`gatling_trigger_t` is set even while the mount is venting.** That
   mirrors `spin_cmd` exactly (which is also set before the vent check),
   and it is harmless today because the vent branch takes priority over
   the decay branch. But if anyone adds a second consumer of
   `gatling_trigger_t` meaning "is firing", it will read true through a
   3.5 s lockout during which nothing leaves the barrels.
6. **I did not wire the bot fire path, per the dispatch.** Bot mechs
   remain permanently disarmed after 150 rounds inside a full-hull
   chassis. That is real, it is Thor's finding, and it is now *safe* to
   fix — the `fire_cd` sharing that made it dangerous to land is gone.

---

## §D — the bot fire path reaches the hull mounts

**Suite: 166 passed / 0 failed / 2 ignored → 171 / 0 / 2.** Five new
tests. Ten mutations applied, run, recorded and reverted with a targeted
`git checkout -- engine/crates/jk_tdm/src/sim.rs`; the working tree was
asserted clean after every single revert.

This closes the item I deferred at the bottom of my last entry.

### D1 — what was actually wrong

Thor's measurement, over 2 s: `hits_dealt=4`, `ammo=26/30`, carried
`Ak47`, `gatling_heat=0`. A bot in a 1000-hull chassis was pulling the
trigger on the rifle its **pilot** happened to be carrying. It reloaded
every 30 rounds, and at 150 rounds — one mag plus a standard reserve —
it was **permanently disarmed inside a full-health chassis** for the
rest of the match. Bots do reach mechs: the pickup loop runs for every
fighter, not just the player.

`bot_act` now routes through `in_mech()` exactly as the player path
does, and the fire gate's `ammo > 0` becomes `ammo > 0 || in_mech` — a
gun bolted to a hull has no magazine to gate on.

### D2 — the mount-selection rule, and why it is this one

> **Autocannon** against another chassis, or past
> `MECH_BOT_AUTOCANNON_RANGE_M`. **Gatling** otherwise.

The range is **derived, not tuned**:
`MECH_BOT_AUTOCANNON_RANGE_M = BODY_RADIUS / GATLING_SPREAD_COLD`
≈ **18.9 m**. `GATLING_SPREAD_COLD` is the per-axis offset
`hitscan_burst` applies to the aim ray, so at range `d` a *cold* gatling
scatters over a half-width of `GATLING_SPREAD_COLD * d`; a man is
`BODY_RADIUS` wide to each side. Past that quotient even a cold mount is
putting most of its rounds beside the target — and every miss still
walks the barrels toward a 3.5 s forced vent. That is precisely where
one 145-damage round through a 3× tighter cone is the honest trade. I
picked a derivation over a round number for the reason §C already wrote
down at the `AUTOCANNON_BRACED_KICK` note: a second independently
tunable number describing one relationship drifts.

It lands inside **every** difficulty's engage range (Easy 22 / Normal 35
/ Hard 50), so all three tiers really do use both mounts rather than one
tier silently never reaching the switch. That was a check, not a
coincidence, and it is asserted in the test.

`hardened` carries **no range term on purpose**: 9 damage a round is not
an answer to 1000 hull at *any* distance, and the 145 exists precisely
to be that answer. To know it, `nearest_visible_threat` now returns
whether the body it picked is a live chassis — the caller receives a
bare position, and a position cannot be asked whether it has hull. No
zombie is ever hardened.

**The one place I went past "simple":** the switch has a
one-chassis-width hysteresis band (`MECH_BOT_MOUNT_HYSTERESIS_M =
MECH_RADIUS`, 0.58 m). A bare threshold over a dithering quantity is a
defect, not a rule. The two mounts keep **independent** cooldown clocks
(`gatling_cd` / `autocannon_cd` — §C made sure of that), so a bot
chattering across the line fires **both**, and "hold station at exactly
18.9 m" becomes the highest-DPS thing a bot mech can do. Four lines, and
mutation-proven (M6).

### D3 — bot bracing: the §A parity question, answered

**Yes — and only for the autocannon.** §A wired the `mech_brace`
movement tax onto both paths but left the flag unreachable from the bot
side; that comment is now deleted rather than left to rot a third time.

`MECH_BRACE_RECOIL_DAMP` has exactly **one** consumer in the file — the
autocannon's kick. Bracing to fire the gatling would therefore buy a bot
literally nothing while costing it 88% of its movement, so it does not.
Bracing for the autocannon buys the *next* shot its picture back, and it
is paid for in the same currency a human pays: a near-stationary chassis
that is trivial to hit. Grounded-gated exactly as the player's is.

Two failure modes I wired deliberately, both tested:

- the stance **drops when the target is lost** (a braced mech walks at
  12%; a bot that kept the plant after LOS broke would crawl the rest of
  the match);
- the write is **unconditional**, not inside `if in_mech`, so a pilot
  whose hull is blown out from under him mid-brace does not carry the
  chassis stance — and its 12% pace — onto foot. That is the §A defect
  class (a mech mechanic leaking onto infantry) coming back through a
  new door, and it is asserted from the bot path.

### D4 — a flaw the probe found in my own test rig

My first rig healed the victim every tick and called the result a
continuous engagement. It was not. One autocannon round is **145**
against a 100 HP man, so healing after the step still let the **kill**
register: `respawn_t` latched for `RESPAWN_S` (3 s), the corpse stopped
being a visible enemy, and the bot spent most of the window with nothing
to shoot at. The 30 m brace assertion was passing **only because that
seed's first round happened to miss** — a latent flake I would have
shipped. Worse: a kill can latch `round_over_t`, after which `step`
early-returns and, 7 s later, **replaces the sim wholesale** — and one
of my tests runs for 30 s straight over that trapdoor.

The rig now makes the victim genuinely immortal (`respawn_t` cleared,
score and round state rolled back every tick). The "target lost" case
steps **raw**, because that same rig would resurrect the very body the
case needs gone.

I found this by writing a throwaway probe, not by reasoning. It is the
second time in two entries that a measurement contradicted something I
believed about my own test.

### D5 — a mutation that survived the whole suite

Skipping the two `rng.range` aim draws on the mech branch — the obvious
"a hull mount doesn't need the bot's wobble" tidy-up — **passed all 170
tests untouched.** It is two defects at once:

1. *Visible:* every bot mech becomes a perfect shot at every difficulty.
2. *Invisible and worse:* those draws come off the sim's single seeded
   stream, so skipping them **re-orders that stream** for every scenario
   containing a bot mech. The replay guarantee is exactly "the same seed
   makes the same draws in the same order."

`a_chassis_does_not_make_a_bot_a_better_shot` closes it: 27 rounds
landed on Easy against 62 on Hard, asserted as a spread rather than an
absolute count so retuning `aim_sigma` cannot silently disarm it.

### Determinism — status, explicitly

**All 11 existing determinism / replay / golden tests are green**
(`deterministic_battle`, `abilities_replay_identically`,
`a_thirty_shot_ak_spray_is_bit_identical_on_replay`,
`a_thousand_identical_throws_land_bit_identically`,
`bow_draw_and_pierce_replay_identically`, `draw_power_golden_curve`,
`dropped_ammo_is_recoverable_and_deterministic`,
`minigun_heat_cycle_is_deterministic`,
`spray_replays_exactly_climbs_and_recovers`,
`throwables_are_deterministic`,
`zombies_spawn_chase_headshot_and_replay`).

The change touches no RNG. The two aim draws stay **unconditional and in
their original position**, before the branch, precisely so the seeded
sequence cannot shift — which is why the entire 166-test baseline passed
unchanged on the first run. A bot-mech engagement was separately probed
and replays bit-identically across two runs (`dealt`, `gatling_heat`,
`autocannon_cd`, `punch_vel`, `hits_dealt`, `mech_weapon`, `mech_brace`,
victim health and the next raw RNG draw, all compared by raw bits).

But see item 1 below before reading that as coverage.

### Mutation proofs — all ten applied, run, recorded, reverted

| # | Mutation | Caught by | Result |
|---|---|---|---|
| **M1** | bot always fires its CARRIED gun (the shipped behaviour) | D.1, D.2, D.3, D.5 | **FAILED. 167 passed; 4 failed** |
| **M2** | restore the `ammo > 0` fire gate for a chassis | D.2, D.3, D.5 | **FAILED. 168 passed; 3 failed** |
| **M3** | mount selection always picks the gatling | D.3, D.4 | **FAILED. 169 passed; 2 failed** |
| **M4** | bots never set `mech_brace` (the §A status quo) | D.4 | **FAILED. 170 passed; 1 failed** |
| **M5** | drop `hardened` — a range-only rule | D.3, D.4 | **FAILED. 169 passed; 2 failed** |
| **M6** | delete the hysteresis band (a bare threshold) | D.3 | **FAILED. 170 passed; 1 failed** |
| **M7** | brace for BOTH mounts, not just the autocannon | D.4 | **FAILED. 170 passed; 1 failed** |
| **M8** | the plant survives losing the target | D.4 | **FAILED. 170 passed; 1 failed** |
| **M9** | the bot pays no `mech_brace` movement tax | D.4 | **FAILED. 170 passed; 1 failed** |
| **M10** | skip the aim RNG draws for a mech (re-orders the stream) | D.5 | **FAILED. 170 passed; 1 failed** |

Every new test is individually proven: D.1 by M1; D.2 by M1–M2; D.3 by
M1–M3, M5–M6; D.4 by M3–M5, M7–M9; D.5 by M1–M2, M10.

### What I am least sure about

1. **No existing determinism test ever puts a bot in a mech, and I can
   prove it.** I probed `deterministic_battle` (seed 21, Arena, 5v5,
   40 s): **0 bot-mech ticks** out of ~4,800 ticks × 9 bots. Zero
   gatling heat, zero autocannon selections. That is *why* my change
   could not perturb it — and it means the suite's replay guarantee does
   **not** cover the path I just wrote. M10 surviving was the symptom.
   D.5 now catches the specific stream re-ordering I could think of, but
   **a full-match replay test with a bot mech in it does not exist and
   should.** I did not add one because a same-code A/B replay test
   cannot be honestly mutation-proven in a sim that is deterministic by
   construction — but someone should decide whether that objection is
   worth the gap. **This is the single thing I would most want checked.**
2. **The horde gets the same rule, and I am not certain it should.** In
   Extraction a zombie is never `hardened`, so a bot mech past 18.9 m
   fires the *autocannon* at the horde — one 145-damage round every
   1.35 s at a crowd, where the gatling is the obvious crowd answer. My
   defence is that past 18.9 m the gatling cannot hit anything anyway
   and zombies close fast, so the window is short. But the rule was
   designed against fighters and I applied it to the horde without a
   separate argument.
3. **The hysteresis is the one place I exceeded the brief's "do not
   overbuild".** I judged a bare threshold to be a genuine exploit
   (straddling the line fires both mounts) rather than a cosmetic
   nuisance, and it is four lines and mutation-proven. But it *is* scope
   I added on my own judgement, and 0.58 m is a band I chose because it
   is one chassis width, not because I measured the dither amplitude.
4. **`MECH_BOT_MOUNT_HYSTERESIS_M` is asymmetric.** The band applies
   only to the *down*-switch, so the rule still reads "autocannon past
   R". A bot that has never held the autocannon switches up at exactly
   R and back down at R − 0.58. That is deliberate and readable, but it
   means the switch point genuinely differs by 0.58 m depending on
   history — if anyone tunes against "the switch is at 18.9 m" they will
   be off by up to a chassis width in one direction.
5. **I did not touch the bot's carried-gun reload while piloting.** A
   bot in a mech with an empty carried mag still calls `try_reload`
   every tick, which can set `reload_t`, which sets `shield_up` inside
   16 m — a sealed chassis raising a riot shield. That behaviour
   predates me and is outside §D's scope, but it is now reachable in a
   state that matters more than it used to.
6. **`hits_dealt` is a shared counter and D.5 leans on it.** It is
   incremented by the hull mounts *and* by anything else the bot lands.
   In D.5 the carried gun is bone dry, so nothing else can move it — but
   the assertion would silently weaken if that setup ever changed.
7. **The §C client-side warning from my last entry still stands and is
   now more visible.** The hull mounts drive `gatling_cd`/`autocannon_cd`
   rather than `fire_cd`, so `main.rs`'s shot-detection sites do not see
   them. Bot mechs now fire constantly, which means that missing muzzle
   flash / casing / audio is about to be a lot more noticeable than it
   was when only the player could trigger it. It is still a `main.rs`
   edit I am not permitted to make.

---

## 2026-08-03 — BRIEF_VIII §4.6: the crosshair settings family

**Suite: 174 passed → 180 passed, 0 failed, 2 ignored.** (The task brief
said the baseline was 173; it was 174 when I measured it — a concurrent
`sim.rs` commit had landed one more.) Six new tests, plus the existing
settings round-trip test extended. File scope: `main.rs` only.

### The gap, precisely

Two problems, and the second is why this mattered more than it looked.
The crosshair was `Text::new("+")` — one glyph, one size, one colour,
nudged to `left: 49.6% / top: 48.6%` to fake being centred. And
`GameSettings` had five fields, none of them crosshair-related, so
§4.9's required test *"crosshair settings round-trip through the settings
file"* was not a failing test. It was an **unwriteable** one: there was
nothing to round-trip. A test that cannot be written is worse than a test
that fails, because nothing counts it.

### What I built

**Eleven settings fields**, all persisted through the existing
hand-rolled `key = value` file (no serde), all clamped on load:
`cross_size` (5), `cross_gap` (0, **negatives legal**),
`cross_thickness` (1), `cross_dot` (off), `cross_outline` (on) +
`cross_outline_px` (1), `cross_color_idx` + `cross_rgb` (green
50,250,50), `cross_alpha` (200), `cross_t_shape` (off), `cross_dynamic`
(off — the spec's **classic static**).

**The glyph is gone.** A zero-size root node at the exact screen centre
with ten children: four arms + a dot, each with a dark backing outline
behind it. Geometry comes from free functions — `crosshair_arm_rects` /
`crosshair_dot_rect` / `crosshair_outline_rect` / `crosshair_gap_px` —
extracted the way `view_recoil_offset`, `bow_sway_deg` and `splash_alpha`
were, so the drawing is testable without Bevy.

**The one design decision that carries the negative-gap requirement:**
each arm runs OUTWARD from `gap` to `gap + size`. `size` is the arm's
length, `gap` only *moves* it, and the two never fight. The obvious
alternative — measuring the arm from the centre and subtracting the gap —
inverts the rect the moment gap goes negative. Mutation M2 is exactly
that wrong implementation, and it dies.

**Colour ladder unified.** `hud_system`'s inline `match` on the glyph's
`TextColor` became `crosshair_feedback` + `crosshair_color`. Hiding (§5.2
scoped-while-unscoped) still beats everything, and the whole ladder is now
one function rather than one function plus a separate glyph swapper —
OPERATION.md rule 6.

**Kill pop preserved in meaning**: the cross still becomes an X for half a
second after your kill, but as a 45° rotation of the drawn geometry plus a
3 px outward kick, so it inherits the player's own size, thickness and
colour instead of swapping `+` for `X`.

**Nine settings rows**, because a persisted field with no control is a dead
control. Fifteen 50 px rows did not fit the game's own 720p default window,
so the page is now a wrapping two-column grid built from a single
`SETTINGS_ROWS` table.

### Mutation proofs — 20 applied, 20 killed

| # | mutation | test(s) killed |
|---|---|---|
| M1 | top arm ignores the gap | geometry, negative-gap |
| M2 | arms SHORTEN with the gap (inverts on negative) | geometry, negative-gap |
| M3 | thickness not centred on the arm axis | geometry |
| M4 | `cross_gap` written under a key nobody reads | file round-trip, settings round-trip, rows |
| M5 | `cross_thickness` not clamped on load | settings round-trip |
| M6 | `cross_color_idx` not clamped on load | settings round-trip |
| M7 | a kill outranks the no-scope hide | hiding |
| M8 | classic STATIC also blooms with the aim cone | t-shape/static |
| M9 | T-shape drops the BOTTOM arm | t-shape/static |
| M10 | CUSTOM colour reads the preset table | t-shape/static, file round-trip |
| M11 | `cycle_alpha` uses `>=` — clicking a preset is a no-op | rows |
| M12 | `cycle_i32` wraps to 0, not the range floor | rows |
| M13 | Idle ignores the settings alpha | hiding |
| M14 | Hidden drawn fully opaque (the no-scope leak) | hiding |
| M15 | default crosshair is DYNAMIC | file round-trip |
| M16 | default colour is not 50,250,50 | file round-trip |
| M17 | the centre dot is not centred | geometry |
| M18 | the outline SHRINKS the rect it backs | negative-gap |
| M19 | the static/dynamic row's label ignores its value | rows |
| M20 | the gap range floor clamped to 0 (negatives refused) | settings round-trip |

Per test: geometry ← M1,M2,M3,M17 · negative-gap ← M1,M2,M18 · settings
round-trip (extended) ← M4,M5,M6,M20 · file round-trip ← M4,M10,M15,M16 ·
t-shape/static/colour ← M8,M9,M10 · hiding ← M7,M13,M14 · rows ←
M4,M11,M12,M19.

Everything was committed **before** mutation testing (rule 7b) and each
mutation reverted with `git checkout -- engine/crates/jk_tdm/src/main.rs`,
never a bare stash.

### Live capture

`JK_CAPTURE=baseline` — **exit 0, 5 snaps, 0 panics.**
`JK_CAPTURE=menus` — **exit 0, 2 snaps, 0 panics.**

A unit test cannot see a crosshair, so I measured pixels. At defaults the
centre of `02-first-person-rest.png` holds 44 green pixels around
(50,250,50) in a plus with a dark outline. Then I wrote a **non-default**
`config/settings.txt` (size 10, gap **−4**, thickness 3, dot on, T-shape
on, custom red 240,60,55) and re-captured:

```
predicted  horizontal x in [-6,+6] logical -> [-8,+7] device @1.25 scale
           vertical   y in [-4,+6] logical -> [-5,+7]  (top arm DROPPED)
measured   x -8..+7,  y -5..+7
```

Exact. That is the settings→file→parse→geometry→screen path proven end to
end, including the two things a unit test can only assert about pure
functions: that T-shape really removes the top arm on screen, and that a
negative gap really makes the bottom arm cross above the centre.

The settings page was captured too: all fifteen rows, two columns, every
crosshair row showing its live value. The first capture caught a real
defect my own comment had asserted away — the 51-char mouse-swap row still
wrapped to two lines at 16 pt in a 470 px box. Fixed to 15 pt in 500 px and
re-captured. **Measured, not assumed** — the comment now says so.

Capture PNG churn reverted with `git checkout -- .../handback/`, and the
throwaway `config/settings.txt` deleted before committing.

### What I am least sure about

1. **The kill-pop X depends on Bevy UI honouring `Transform.rotation`, and
   I did not capture it.** `ui_layout_system` writes only `translation`, so
   rotation should survive and propagate to the children — Bevy's own
   `overflow_debug` example rotates UI nodes. But no capture script has a
   beat where the player gets a kill, so I have **no PNG of the X**. If
   rotation silently no-ops, the kill confirm degrades to the orange colour
   flash (which does work) and nothing warns you. **This is the single
   thing I would most want checked.**
2. **`crosshair_render` itself is untested.** Every pure function it calls
   is mutation-proven, but the system that wires them — the `shown`
   predicate combining hidden/T-shape/dot/outline, and the piece-index →
   rect mapping — is a Bevy system and I asserted nothing about it. A
   swapped arm index or an inverted `shown` term would pass the whole
   suite. The capture is the only thing standing behind it, and the capture
   exercises exactly two configurations.
3. **I changed the settings page layout, which was not asked for.** The
   task's four build items did not include settings rows. I added them
   because eleven persisted fields reachable only by hand-editing a text
   file is precisely the "dead control" this test file's own comment warns
   about — but it *is* scope I took on my own judgement, and it
   restructured a screen I was not sent to touch. The two-column grid was
   forced by the row count, not chosen.
4. **`CROSS_SPREAD_PX_PER_RAD` is a refactor with a risk I cannot see.**
   `stability_bracket` had a bare `2400.0`; I replaced it with the shared
   constant so a dynamic crosshair and the bracket bloom at one rate. The
   value is identical, so nothing moved — but **no test covers
   `stability_bracket`**, so if I had fat-fingered the number nothing would
   have caught it.
5. **The dynamic crosshair's px-per-radian is inherited, not derived.**
   2400 came from the stability bracket, where it was tuned against a
   bracket glyph, not against arm travel. A dynamic crosshair may bloom too
   far or not far enough; the spec defaults to static so it is not on the
   critical path, but the number is ASSUMED, not measured.
6. **The outline alpha is derived from the fill alpha (`0.75 ×`) and the
   spec says nothing about it.** A player who sets alpha 255 gets a
   near-opaque black backing. I judged a coupled outline better than a
   twelfth setting, but that is my call, not the brief's.
7. **The settings-file test writes to `std::env::temp_dir()`.** It is
   process-id-suffixed and removed afterwards, but it is the only test in
   this crate that touches the filesystem. Somewhere without a writable
   temp dir it fails for a reason that has nothing to do with crosshairs.
8. **The clamp RANGES are mine, not the brief's.** §4.6 gives defaults
   (size 5, gap 0, thickness 1, outline 1) but no bounds, so
   size 1..12 / gap −5..12 / thickness 1..5 / outline 0..3 are chosen to
   match the CS-lineage feel these values come from. They are the numbers
   the clamp tests assert, so changing them later means changing tests.

---

## FRIDAY22 — the hull turret kicks hard, and the bots never see it (d6e35d1)

**Task.** §owner: "increase the recoil" on the heavy mech's minigun turret.
The interesting constraint, handed to me up front: `MECH_RECOIL_CONTROL` is
bounded by BOT COMPETENCE, not by feel — at 0.45 `a_bot_mech_never_runs_dry…`
fails because `punched_aim` deflects a bot's ray by the full `RECOIL_SCALE`
with no mouse to pull down.

**What I found by measuring first.** The wall was real, but it was the wrong
wall. One constant governed two things the brief wants moving in opposite
directions: the PICTURE (`punch`, camera, viewmodel) and the ROUNDS
(`punched_aim`). Every degree of extra feel bought a degree of extra aim
error, and the aim error is what bots eat.

The owner's complaint also had an exact numeric form I did not expect. A
lone impulse under `PUNCH_DECAY_LIN_DEG * exp(PUNCH_DECAY_EXP * DT)` =
19.24 °/s is erased inside the tick it lands. The turret wrote 8.29 — 43% of
that. **One round of SINGLE moved the camera by exactly 0.0000 degrees.**
Not "a little". Zero. Measured through the real fire path before I touched
anything.

**The split.** `TURRET_FELT_FLOOR` 24.0 (midpoint of the window between that
floor and the M4's 28.8, which an existing test already caps the mount at)
and `TURRET_AIM_STABILISER` 0.25 in the new `mount_punched_aim`.

**The trap I nearly shipped.** I first wrote the stabiliser as the algebraic
reciprocal of the lift, and reasoned in the commit-in-progress that bots were
protected "by construction". They were not. Punch ANGLE is **superlinear** in
the impulse — `PUNCH_DECAY_LIN_DEG` shaves a *constant* off every tick, so it
is a smaller fraction of a bigger velocity. A 2.90x impulse gives a 3.98x
sustained angle (4.085 → 16.22° over 5 s of held AUTO). The reciprocal would
have let a bot's rounds walk **38% further** than before. Nothing that read
only constants could have caught it. I only caught it because I measured the
plateau instead of trusting the algebra, and I have written that into the
constant's doc comment so the next person does not re-derive the wrong thing.

**Evidence.** felt kick 8.29 → 24.00 °/s; one SINGLE round 0.000 → 0.101°;
AUTO plateau 4.09 → 16.22°; braced AUTO 0.00 → 3.24° (the brace damp finally
buys something that was not already zero); autocannon 133.63 → 133.63 with a
stabiliser of exactly 1.0 — bit identical, deliberately untouched. Bot damage
over 4 s at six ranges 6–18 m: 1667.8 → 1757.2 (+5.4%) on identical round
counts.

**Also folded in: five defects from the scout.** Barrier speed tax charged
twice (0.3025 of pace, player only, with the bot path on a different rule);
the medic paying for a barrier that does not exist; a bot medic healing and
firing plasma in one tick; a bot mech turtling for a magazine it is not
holding; and three dead values plus a constant that was never read.

Tests 386 → 404. Seven added, every one mutation-proven from a file copy.

### What I am least sure about

1. **The AUTO plateau is 4x what it was — 16.2° of punch, ~14.6° of camera
   climb on sustained fire.** That is by far the biggest recoil in the game
   (an AK settles at 3.18°). I believe it is right: the owner asked for more
   recoil, the brace takes it to 3.24° which is *below* today's unbraced, and
   every fire mode stays clearly ordered. But it is a large feel change and
   it is the one number a playtest could send back. `TURRET_FELT_FLOOR` is
   the single knob — and if it moves, `TURRET_AIM_STABILISER` must be
   RE-MEASURED, not scaled, for the superlinearity reason above.
2. **Braced fire is no longer perfectly accurate.** It used to deliver
   exactly 0° of aim error because the damped punch fell under the decay
   floor. It now delivers ~1.6°. Bots never brace with the gatling so this is
   a player-only change, and I think "bracing damps recoil" reading as
   "bracing deletes recoil" was the anomaly — but it is a nerf nobody asked
   for.
3. **Taps got ~28% steadier** (24.0 × 0.25 = 6.0 delivered against the old
   8.29). The stabiliser is calibrated at the plateau because that is where
   all of a bot's aim error lives; the tap regime falls out of that choice
   rather than being chosen.
4. **`the_guard_speed_rung_is_the_same_one_on_the_bot_path` cannot
   discriminate the heavy's rung today.** `SHIELD_SPEED_MULT` and
   `MECH_SHIELD_SPEED_MULT` are both 0.55, so swapping them in
   `shield_speed_mult` still passes. I left the assertion written by name and
   said so in the test; it starts discriminating the moment either constant
   is retuned, which is the only moment it matters.
5. **A bot-piloted chassis now never raises its barrier at all.** That is the
   honest consequence of fixing the reload gate, not a hidden one — barrier
   discipline for a bot mech is a new behaviour with its own "when is a wall
   better than a gun" rule, and I did not invent it.
6. **The player/bot asymmetry in `punched_aim_stabilised` is untouched.**
   It is the real root and it applies to every recoiling weapon in the game.
   Fixing it is a game-wide bot accuracy change and it does not belong in a
   commit titled "the turret kicks harder".

— **FRIDAY22**

---

## 2026-08-10 — SPEC15 P1/P2/§4, sim half (Pyro out, Royal in, the recoil envelope)

Four commits: `b11b7de`, `614bf03`, `107c8ef`, `879c95a`. `sim.rs` only.
Sim tests 218 → 227; whole crate 404 → 417 once Friday33's client half landed.

### 1. Pyro is gone, not retired
`ArmorSet::Pyro`, `PickupKind::PyroArmor`, `Fighter::fuel`, the five `FLAME_*`
constants, the 62-line flame ability arm, both per-map relocation tables that
hand-placed a pad no base list spawned, and its three passive immunities
(toxic, molotov `burn_t`, the blanket `fire` return in `apply_plain_damage`).
Nothing in the game is fire-immune now, which is the true state of a game with
no flame weapon in it.

### 2. Royal is a third `ArmorSet`, not a bool
Rejected `Fighter::royal: bool` (makes trap 2 impossible, but cannot carry a
DATA ROW — `armor_spec`, `mech_hull_max`, `MechWeapon::for_set` and
`PickupKind` are all keyed on `ArmorSet`) and rejected a `MechTier` enum
beside `armor_set` (same problem, plus it admits `ScoutMech + Royal`).
Trap 2 paid in the same commit: all 24 production `== ArmorSet::RobotSuit`
comparisons are now `in_heavy_mech()` / `is_heavy_chassis()`.
One constant, `ROYAL_MULT` 1.10, feeds `chassis_scale` (height/radius/step-up/
jump apex), `mech_hull_max`, `mech_shield_max` and a DERIVED `armor_spec` row.
It pays: `move_mult` is divided by the same constant, and a 10% bigger capsule
is a 10% easier target.

### 3. Controlled, then chaotic
`turret_chaos(i)` (0 through the controlled window, ramping to 1) plus
`turret_spray_entry(i)`, a fixed-seed pattern built the way `spray_entry`
builds a rifle's. Boundaries derived: the controlled window is the midpoint of
the owner's own "1-2 s"; full chaos is the midpoint between that and the
mount's own `turret_rounds_to_vent()`. **No new sim state** — pure functions of
`turret_burst_i`, and its own local `Pcg32`, so zero draws were added to,
removed from or reordered in the sim's stream.

### THE THINGS I AM LEAST SURE ABOUT

1. **A bot mech's sustained output at 10 m fell about a third** (492 → 331
   damage over 17 s on my own rig, which is cruder than the test's). The
   envelope is on the shared path, so a player pays it too and can compensate;
   a bot cannot. `a_bot_mech_never_runs_dry...` still passes with margin
   (81–130 damage in its final 3 s against a `> 0` gate). This is the intended
   direction — sustained fire is supposed to get worse — but a third is a real
   balance change, not a rounding error.
2. **That drop is insensitive to the pattern's AMPLITUDE, only to its
   presence**, which I do not fully understand. Three different width/yaw
   settings all measured 330.7 over 17 s. I chose the widest setting because
   the read is strongest and the measured cost did not move; if the mechanism
   turns out to be an artefact of my rig, the amplitude may be buyable back.
3. **`armor_spec`'s flats are unreachable for a piloted HEAVY chassis.**
   `apply_armor` takes the hull/angle branch and returns before them. True of
   the Big Mech's row too, and long-standing. The Royal's live durability is
   hull + barrier; the flats are kept scaled so the table stays consistent.
   Worth someone deciding on deliberately.
4. **Dropping the 0.55 mean-reversion carry from the turret pattern does not
   fail the suite.** At a step of 0.80 against a hard ±1 clamp the clamp is
   most of the containment. Genuine persistence (carry 0.97) IS caught. I said
   so in the comment rather than writing a guard I could not make fail.
5. **`mech_visor_eye_y(pos_y)`, the free function, still hard-codes
   `MECH_SCALE`** and so is wrong for a Royal. `Fighter::visor_eye_y()` is
   correct. Every client caller of the free function needs moving — that is
   Friday33's, and I did not reach across.
6. **The client's `PUNCH_DEG_S_PER_SPEC_KICK` bridge reads
   `turret_kick_per_round()`, which is unchanged**, so the client's recoil
   still models a single-axis mount. `mount_kick_axes` and `turret_chaos_of`
   are published for the visual half; nothing reads them yet.

— **FRIDAY22**


---

# FRIDAY33 - BRIEF X, THE AGILE MECH (2026-08-10)

`TRV-0011` / SPEC15 P3, "the largest visual item". Presentation only:
`agile_mech.rs` (new, 700 lines), `main.rs`, `mech_lineup.rs`. **`sim.rs`
untouched. `SCOUT_SCALE` untouched (owner question Q5 is still open).**

## What the machine is now

A new module, wired in with two lines. `spawn_scout_chassis` is deleted
from `main.rs` (354 lines); `agile_mech::spawn_agile_chassis` replaces it
at both call sites. ~150 parts, 3 meshes, 7 materials, 4 of them shared
with the heavy.

Five silhouette elements, none of them in the Big's vocabulary:
reverse-jointed legs (hip forward-high, knee forward-low, ankle swept
back, compact foot, heel spur), swept dorsal fins, a 0.245-wide wedge
helmet with a visor slit, shoulders at 0.40 half-width, a forward-canted
torso. §2's four material roles exist as four roles for the first time -
the old palette painted armour and mechanism the same colour.

## What the capture found that no test could

1. **A steel tripod standing beside the machine's knee.** The HEAVY's
   barrier emitter - housing, three petals, lit tips. §P1 SIX VARIANTS
   re-parented it from the Big's `armor_rig` (hidden unless the Big is
   worn) onto the soldier's THORAX (visible for everyone) and nothing
   replaced the hiding the old parent gave for free. It has been riding
   every scout AND every infantryman since. `mech_barrier_sync`'s own
   comment still reasons from the old parent. Gated on `in_heavy_mech()`
   now. **Not a Brief X defect - it predates this work.**
2. **The pilot's own bright torso showing through the machine's lower
   back**, 15 mm of it, between the rib box and the backpack.
3. **The pilot showing through the machine's CROTCH during the dodge
   roll.** The tumble is the only camera angle in this game that looks at
   a chassis from underneath, and the pelvis had no floor.
4. **Two gold tabs on the hips**: the pilot's waist stripe is 0.345 x
   0.27 and the pelvis was 0.385 x 0.265.

Every one of those was invisible to 438 tests and obvious in a PNG.

## What I am least sure about, and what I deferred

- **The opposition Agile keeps DARK BLUE as its primary armour** and
  carries the chassis's orange on its layered plates only. Two owner
  rulings from the same day pull opposite ways (Brief X §2 "orange
  becomes the Agile Mech's identity" vs "the light chassis takes the SAME
  two blues as the heavy so the enemy fields one army"). I honoured both
  by ROLE. If the owner wants the opposition Agile orange-primary,
  `scout_hull_foe` and `scout_plate_foe` are the whole change.
- **The double jump fires and gains altitude, but the Agile has no
  airborne POSE.** `try_mech_jump`'s compress/tuck is heavy-only, so the
  only visible evidence is the horizon dropping. §0's "still fires and is
  visible" is satisfied weakly. A client-side airborne tuck would be pure
  presentation and in my lane; it is new work, not preservation, so I
  flagged it rather than sneaking it in.
- **CLIMB is not an Agile Mech ability and Brief X §0's table is
  misleading about it.** `climb_target` requires `!p.in_mech()` and a
  DROPPED plate on the target: hull-climbing is a verb for a pilot ON
  FOOT against an enemy mech. There is no frame of an Agile Mech climbing
  to take, and I did not claim one.
- **No true black-matte silhouette.** The harness has no stencil or depth
  output and a luminance threshold failed (the exhibit stands against a
  dark wall). `agile_body/09-squint-derived.png` is post-processed from
  `mech_gallery/01`: native, then 30 m by downsampling 3.33x, then hue
  removed. It answers "does it separate by shape" honestly; it is not a
  matte, and its filename says derived.
- **`config/settings.txt` carries `fov_idx = 4` (100 deg) against
  `FOV_DEFAULT_IDX = 3`.** Every frame in this pass was taken through it.
  All my comparisons are same-settings, but none is what a clean checkout
  produces.
- **The portrait shows `MechTrim::Stripped`** - the player is slot 0 and
  `MechTrim::ALL[i % 4]` gives them the bare frame, so the pauldrons,
  knee cops, belly band and gorget are absent from `agile_body`. They are
  in `trims/01-trims-front.png`, which is the only capture that shows
  them.
- **The palette is DATA now.** Six colours moved out of `build_kit`'s
  inline literals into `agile_mech` consts, which is what makes SPEC15's
  trap 4 testable at all. `the_enemy_agile_never_out_luminates_the_ally`
  pins the hull separation at >= 8x in linear light (it measures 17x) and
  the strict form, every enemy surface below every ally surface. That is
  the guard Trevor's Task 5 asked for.

Nine tests, mutation-proven from a FILE COPY. Two of them failed for real
during the build: the pauldron lip overhung the published shoulder width
by 11 mm, and an assertion I wrote about the elbow was simply wrong (the
MOUNTS are the widest point of this machine and always were, because
`MOUNT_X` is pinned by the sim). The sole test had to be strengthened to
earn its place - it was recomputing its own expected value.

Captures, all from the release binary built at 21:51: `agile_body` 8
shots, `agile_moves` 12, `mech_gallery` 11, `trims` 2, `medic` 9. Exit 0
and zero panics on every run.

- **FRIDAY33**

---

## 2026-08-11 — BRIEF XII-A, the HUD consolidation pass (FRIDAY33)

`85e0667` and `d73a6d0`. Two files: `hud.rs`, `main.rs`. `sim.rs` untouched.

**Deleted, by symbol.** `weapon_strip`, `WeaponStripCell`,
`shield_readout`, `ShieldReadout` — functions, components, spawns and both
`add_systems` registrations. Systems in the `Playing` HUD-writer set went
from 10 to 8; nothing was added anywhere to replace them, because the
mech systems column was already a per-frame widget and the folded content
went into its existing `paint_systems` pass. `HullFill` and `HeatFill`
went with the two drawn bars in that column.

**The heat formatters at `main.rs:22858` and `:22992`-`:23010` were NOT
deleted, and that is a stated deferral.** They are not dead: they are
live branches of `hud_system` writing into `PanelInfoText` /
`PanelAmmoText`, which `suppress_legacy_hud` hides every frame but which
must keep existing or `hud_fade`'s `Local` snapshot query stops finding
its three entities — the scar the module header documents. Removing them
means rewriting a branch of the legacy `hud_system`, which is invisible
work with a real regression surface, so it is left for whoever shrinks
the suppression list at source.

**What the captures actually showed, twice.** Two defects were found by
looking at a frame and could not have been found by reading: the same
mount named `TURRET` in one corner and `AUTOCANNON` in the other, and the
boarding prompt striking through both big numerals. The instrument was
new — `hud_contrast` is the first script in this repo that points a
camera at a HUD against two different backdrops — and it paid for itself
on its first run. The pitch sign also runs opposite to the obvious guess:
POSITIVE pitches the view DOWN.

**Least sure about:** the `scrim()` alpha, 0.30. It is the one number
here chosen by eye rather than by constraint. It reads well over pale
sand and over the dark cockpit in the frames taken, but it is a taste
call and the owner may want it lower.
