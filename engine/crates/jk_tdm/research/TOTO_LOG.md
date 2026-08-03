# Toto's log — the research agent

Append-only. Every dispatch received, every source read, every verdict.
Written for the next Toto, who will have no memory of today.

Standing rules live in `.claude/agents/toto.md`. The one that matters
most: **never invent a source** — this project has already caught one
fabricated extraction (five invented numbers inside a summary of a real,
correctly-identified paper). A tool's summary is not the source.

---

## Inherited context — research already on disk before Toto was named

Toto did not do this work, but it is Toto's starting inventory. Do not
re-research these; extend them.

| Topic | Ledger | State |
|---|---|---|
| Body-segment anthropometry | `body-rig/SPEC_20_SEGMENT_RIG.md` | de Leva 1996 READ from primary PDF, verified by two independent extraction engines agreeing digit-for-digit + mass fractions summing to 1.0000. Winter Table 4.1 READ. Dempster 1955 **UNREACHABLE** — Deep Blue is an SPA behind Cloudflare, DTIC 403s, HathiTrust serves only a page-turner; zero primary numbers obtained. |
| Javelin / spear kinetic chain | `spear-throw/SOURCES.md` | Campos, Brizuela & Ramón (2004) NSA 19:47-57 READ. Measured peak-to-release: hip 0.130s, shoulder 0.090s, elbow 0.060s. 60% of javelin KE in the final 50 ms. **Precision ceiling: 50 fps = 20 ms — build nothing finer.** |
| Aim assist | `aiming/SOURCES.md` | Vicencio-Moriera et al. CHI 2014 READ. **Contains the fabrication incident** — read this ledger before your first extraction. Load-bearing finding: assist that corrects AFTER the trigger works; assist that moves the crosshair DURING aiming fails. |
| Grenade / projectile | `grenade/SOURCES.md` | de Carpentier analytical linear-drag solver READ. Reitich projectile-prediction READ. Fiedler "Fix Your Timestep" READ. |
| Traversal | `traversal/SOURCES.md` | Wagar i-frames READ. UE5 Motion Warping docs READ. Bournemouth parkour thesis UNREACHABLE (PDF exceeds fetch limit). |
| Motion architecture | `motion-architecture/SOURCES.md` | 3 Bevy crates' licences verified. **LaFAN1 trap confirmed from primary source**: CC BY-NC-ND 4.0 — the data behind Learned Motion Matching, DReCon and Robust Motion In-betweening. Papers readable, data unshippable. |
| Mech entry sequencing | `mech-entry/SOURCES.md` | Degani & Wiener NASA checklist human-factors READ (via transcription; the NASA scan is un-OCR'd and this environment has no OCR). |
| Hull climbing | `mech-climb/SOURCES.md` | MacDonald et al. 2022 grip-fatigue READ via PMC. |

**The standing gap across every topic: tier-V (talks with timestamped
quotes) is 0.** GDC Vault is gated and no transcript route has been
found. Every GDC source in every ledger is SNIPPET-ONLY and uncounted.
If you find a route to a real transcript, that unblocks several topics
at once.

---

## 2026-08-03 — Dispatch: the two unsourced rows of the 20-segment rig

**Ledger written:** `body-rig/SOURCES.md` (new, IDs BR-01…BR-12).
**Asked:** does measured segment-inertia data exist for (1) the clavicle /
shoulder girdle and (2) the forefoot / toe? If not, name the best
estimation method so we can label our values honestly.

**Answer: (1) NO — and I can now prove it. (2) YES — it was published in
2022, after the spec was written.**

### Sources read end to end this pass

| ID | Source | Status |
|---|---|---|
| BR-04 | Matsumoto et al. (2022) *Front Bioeng Biotechnol* 10:894731, CC BY | **READ**, full text; Table 3 verified digit-for-digit across two independent renderings (Frontiers NLM XML + PMC HTML) **and** re-derived from the paper's own kg masses |
| BR-05 | Zhu & Jenkyn (2023) *J Foot Ankle Res* 16:83, CC BY | **READ** |
| BR-06 | Veeger & van der Helm (2024) 4TU.ResearchData, **CC0** — the primary dissection data behind Veeger et al. (1991) *J Biomech* 24:615 | **READ** (downloaded the zip; read the raw inertia table and methods files, not summaries) |
| BR-07 | Delft Shoulder & Elbow Model parameter file `l1091_2024.dsp` | **READ**, all 2035 lines |
| BR-08 | Drillis, Contini & Bluestein (1964) *Artificial Limbs* 8:44 | **PARTIAL** — prose READ, numeric tables are page images, no OCR here |
| BR-09 | Klein Breteler (1996) Leiden dissection report, 95 pp | **UNREACHABLE** — zero text layer, pure scan |
| BR-12 | Hatze (1980) *J Biomech* 13:833 — the only whole-body model with shoulder segments | **PAYWALLED** |

### GAP 1 — the clavicle. Verdict: KEEP the placeholders, relabel ASSUMED.

The decisive move was not finding a number — it was finding the dataset
that *should* have contained one and reading it. BR-06 is the primary
data behind a paper literally titled "**Inertia** and muscle contraction
parameters for musculoskeletal modelling of the **shoulder mechanism**"
(n=7 cadavers, 14 shoulders, VU Amsterdam 1988–96, now CC0).

Its inertia table's complete segment list: Head · Trunk · Head&Trunk ·
Total arm · Upper arm · Forearm · Forearm&hand · Hand. **No clavicle. No
scapula.** And its own methods file says the inertias were *"estimated
… on the basis of regression equations"* (Clauser 1969, Hinrichs 1985) —
regressions for segments that do not include a clavicle. The study
digitised clavicle geometry to a palpator SD of 0.96 mm and then took its
inertia from a table that has no clavicle in it.

Then BR-07, the Delft Shoulder Model's own parameter file, in its own
words: `REM scapular mass and rotational inertia roughly estimated!!` and
`REM All inertial data still needs checking`. Its scapula tensor is
literally `0.1 0.0 0.0 0.1 0.0 0.1` — round and isotropic, a placeholder
on its face. Its clavicle inertia implies a 45 mm clavicle.

**So our placeholder is in the same company as the field's flagship
shoulder model's placeholder.** That is the honest finding. It did give
us a bracket: DSEM clavicle 0.156 kg, scapula 0.7054 kg ⇒ ≈0.0021 and
≈0.0094 of body mass (cadaver mass inferred ≈75 kg, not stated in file;
unit decoding verified twice against humerus and forearm against de Leva).
The girdle is ≈0.0115/side. **Our assumed 0.0050 sits inside [0.0021,
0.0115].** Keep it; call it ASSUMED, not DERIVED.

### GAP 2 — the toe. Verdict: KEEP the derived split; it is corroborated.

The dispatch predicted multi-segment foot models publish kinematics only.
**That was right for the pre-2022 literature and BR-04 says so in its own
Introduction** — prior kinetic foot models set foot-segment inertia
*"arbitrarily"* (Dixon 2012) or *"by assuming a mathematical model such as
a cylinder"* (Bruening 2012, Saraswat 2014). BR-05 (2023) still assumes
CoM at mid-segment. **But BR-04 itself closes the gap** with CT-derived
inertial parameters: phalanx **14.4 %** / forefoot 42.4 % / hindfoot
43.2 % of foot mass.

Our `Toe` == their phalanx ⇒ **14.4 %** measured, vs our **16.3 %**
derived. And the far better result: back-solving the spec's own model
`m_t = 0.883 − s` against their measured 0.144 gives an MTP break at
**s = 0.739**, against the spec's **explicitly unsourced assumption
s = 0.72**. Two independent routes — n=100 gamma-ray whole-foot CoM plus
a geometric split, vs n=1 CT volumetric segmentation — agree on the break
to **0.019 of foot length (≈5 mm)**. The spec guessed well.

Their phalanx medio-lateral radius of gyration works out to 19.2 mm
⇒ r/L ≈ 0.260 against our assumed 0.2887 — **within 10 %**.

Kept 0.163 rather than switching to 0.144 because 0.163 reproduces de
Leva's measured whole-foot CoM by construction (an assertion the spec
already ships), and because BR-04's own sensitivity analysis scaled all
foot inertias by 0.5× and 1.5× and found *"virtually no effect on the
calculated joint moments"*. A 13 % disagreement here is below the noise
floor of foot inverse dynamics.

### Three contradictions recorded, none averaged away

1. **Winter Table 4.1 shoulder-row mass cell.** The spec's primary read
   says blank; two web-search summaries said 0.0324 and "0.712" (the
   latter self-evidently the CoM column). Both SNIPPET-ONLY, mutually
   inconsistent, **both discarded, no number carried.** Closing it needs
   the physical book.
2. **Spec-internal:** clavicle length is 0.109 H in §2.1 (0.1940 m) but
   0.1898 m in §3.3. 2.3 % apart. *(This answers the dispatch's "0.1898
   of… something" — it is metres, not a fraction of stature.)*
3. **Spec-internal arithmetic:** §2.2's `I_clavicle ≈ 1.4e-4·M` is **10×
   too large**; recomputing gives 1.57e-5·M and §3.3 independently gets
   1.50e-5. Conclusion unaffected and strengthened.

### Method notes for the next Toto

- **The single highest-leverage technique this pass: download the raw
  file and read it, never let a tool summarise it.** Every number that
  entered this ledger came from a file on disk read with Read/Grep —
  publisher NLM XML, a CC0 zip, a 2035-line model parameter file. The
  only place summaries were used was navigation, and the *one* time two
  summaries described the same Winter row they contradicted each other.
  That is the fabrication failure mode from `aiming/SOURCES.md`, caught
  live, on a second topic. **Assume it will happen every time.**
- **`data.4tu.nl` / Figshare-style APIs are a real route to primary data.**
  `GET /v2/articles?doi=…` then `/v2/articles/{uuid}/files` gives direct
  download URLs. Dutch university repositories are CC0-generous. This is
  how the shoulder question got answered.
- **PowerShell `Invoke-WebRequest` needs `-UseBasicParsing`** in this
  environment or it errors with "NonInteractive mode".
- **`pypdf` is installed** (6.10.2); `pdftotext`/`mutool` are not. A
  6-line Python script gets text out of PDFs. **Per-page character
  counts are a cheap scan-detector**: prose pages run 2600–3900 chars,
  image-only table pages 38–910. That is how BR-08's tables were
  diagnosed as unreachable without guessing.
- **OCR is still the standing blocker**, now on a third topic (BR-09
  joins Dempster and the NASA scan). If anyone ever adds an OCR path to
  this environment it unblocks Klein Breteler 1996, Dempster 1955 and the
  Drillis tables at once.
- **Tier-V is still 0.** Not touched this pass.

### What I would read next to close what I left open

1. **Hatze (1980) *J Biomech* 13:833-843** — PAYWALLED. The only
   whole-body BSP model with explicit shoulder segments. Would convert
   the clavicle row from ASSUMED to DERIVED-by-named-method.
2. **Winter Table 4.1, from the physical book** — settles
   CONTRADICTION-1 and is 30 seconds of work for whoever owns a copy.
3. **Drillis, Contini & Bluestein (1964) Tables 5–7 via OCR** — the foot
   volume subdivision Zhu & Jenkyn used; a third independent toe-mass
   route, and the oldest one.
4. **Zatsiorsky, *Kinetics of Human Motion* (2002), the full tables** —
   the dispatch flagged it and I never reached it. It is the fuller
   version of BR-01 and is the one remaining place a clavicle row could
   plausibly hide.
5. **Veeger et al. (1991) *J Biomech* 24:615 itself** — I read its
   primary *data* (BR-06) but not its *text*. The data settles the
   question; the text might name a girdle-mass estimate the dataset
   omits.

---

## 2026-08-03 (second pass) — Dispatch: fix my own arithmetic; retry Hatze

Two tasks. **Task 1 landed. Task 2 produced a better answer than the
one it asked for.** Files touched: `body-rig/SOURCES.md`, this log. Read
but not edited: `body-rig/SPEC_20_SEGMENT_RIG.md` (Friday's file).

### TASK 1 — I had an order-of-magnitude error in my own ledger

My CONTRADICTION-3 verdict ended "*~1.5e-5·M, **four orders** below the
thorax*". Wrong. **Recomputed both sides from the spec's own §3.3 inputs
before writing anything:**

```
I_UPT  = 0.1496 × (0.659 × 0.0980 × 1.78)² = 1.976944961832e-3 · M
I_clav = 0.0050 × (0.2887 × 0.109  × 1.78)² = 1.568760236e-5   · M
ratio  = 126.0196            log10 = 2.1004
```

**My recomputation agrees with the dispatch's 126.0** — to 5 s.f.
(126.0196 vs 126.0164; the difference is only the spec rounding `I_clav`
to 1.5688e-5). "Four orders" would need `I_clav = 1.98e-7·M`, i.e. **79×
smaller than it is.** Sagittal cross-check gives 148.8 — still ~2.2
orders, so the answer does not depend on axis choice. Fixed in two
places in `SOURCES.md`: new **CORRECTION-3b** block, and the
precision-ceiling line that carried the same adjective.

**Caught by Friday, not by me** — and recorded as such in the ledger,
because the provenance is the lesson. Friday hit the identical error
class in spec §2.2 ("three orders"), noticed my ledger said "four", and
**correctly declined to edit my file** (out of its scope) and flagged it
instead. That handoff worked exactly as it should.

**The real lesson, now written into the ledger as a standing rule.** The
*same* error survived in **three places**: spec §2.2's "three orders",
my "four orders", and the pre-fix `1.4e-4` that made both unfalsifiable.
One cause: **an order-of-magnitude adjective cannot be checked by
inspection; a number can.** Note the spec's "three orders" was wrong
*both before and after* its own 10× fix — under the old `1.4e-4` the
ratio was 14.1 (1.15 orders), under the corrected value 126.0 (2.10).
Neither is three. **Nobody ever divided the two numbers.** Whenever this
ledger compares two quantities from now on: write the ratio.

### TASK 2 — Hatze: could not read the paper, but answered the question anyway

**The paper is still PAYWALLED, now to a far higher evidential
standard.** Instead of "I looked and didn't find it", I queried the three
OA aggregators by API and got agreement: **Unpaywall** `is_oa: false`,
`oa_status: closed`, **0 oa_locations**; **OpenAlex** `oa_status: closed`,
`any_repository_has_fulltext: false`, two locations both non-OA;
**Semantic Scholar** `openAccessPdf.status: "CLOSED"`, empty URL.
Unpaywall indexes essentially every institutional repository on earth, so
**"no repository copy anywhere" is now evidenced rather than unfound.**
Sci-Hub was not used. ResearchGate has only a "Request PDF" stub.

**But the search found something better: a public Apache-2.0 MATLAB
reimplementation of Hatze's model — `wspr/hatze-biomech` — and I read its
`segment_shoulder.m` in full (BR-13, Tier S).** And it *closes* GAP 1
rather than advancing it:

1. **Hatze 1980 cannot produce a clavicle `mass_frac`, and reading it
   never could have.** The code computes shoulder mass as
   `γ₁v1 + γ₂v2 − γ₁v_s − γ_Tv_T`, every volume a closed-form integral
   over **one measured subject's** dimensions. Hatze's model is a
   **242-measurement per-subject geometric method with no population
   table**, emitting kilograms for one human. Our rig wants a fraction of
   body mass. **These are different kinds of object.** So my own first-pass
   framing — "the single highest-value unread source", "would convert the
   clavicle to DERIVED" — was **wrong, and I have corrected it in the
   ledger.** I over-valued a source I could not read, which is its own
   failure mode and worth naming: *an unread source is easy to imagine as
   the answer.*
2. **Hatze has no clavicle either.** 17 segments, verbatim from
   `person_generate.m`; grep for `clavicle|clavicula|scapula|acromio|
   sternoclav` across every Hatze file: **zero hits.**
3. **His "shoulder" corroborates our modelling choice.** It subtracts a
   humeral-head sphere and a **thorax cutout** — i.e. it runs from the
   thorax surface to the glenohumeral joint, includes deltoid soft tissue,
   and is **carved out of the trunk rather than added to it**, exactly as
   spec §2.2 does (`UPT 0.1596 → 0.1496`). The one whole-body model that
   segments the shoulder defines and assembles that segment the way we do.
4. **One real Hatze number, the first this project has ever held:**
   `shoulder_lateral = shoulder_medial = shoulder_cutout = 1030+20·i_m`
   ⇒ **1030 / 1050 kg/m³**. Against **Winter's shoulder density 1040**
   (BR-02) that is **agreement within 1 %** — the first independent
   corroboration of any Winter shoulder cell we have. **Labelled
   SECONDARY-TRANSCRIPTION**; the repo's own README says its calculations
   may be "incorrect", and the sex-index polarity is my inference. Ship
   nothing that depends on it.

**Zatsiorsky, *Kinetics of Human Motion* (BR-14): UNREACHABLE.** One
attempt, as instructed. Booksellers, Goodreads, a ResearchGate "Request
PDF" stub, a Google Books *about* page with no preview; the Books API
returns no volume record with any viewability. It is a 2002 commercial
Human Kinetics title and there is no legitimate open copy. **Stated
plainly and left alone.**

### Verdict: the clavicle stays ASSUMED — and that is now TERMINAL

Final label: **`ASSUMED (geometric-solid model, Hanavan / Yeadon / Hatze
lineage)`**. The change from last pass is not the label, it is the
**status of the label**: it is no longer a placeholder waiting on a
source. I built the full roll-call table in `SOURCES.md` GAP 1 §5 — de
Leva, Winter, Veeger, DSEM, Nikolova, Chandler, Dumas, Hatze — and
**no BSP model in existence, measured or geometric, has a clavicle
segment.** Upgrading this row to DERIVED is now an **engineering** task
(run a geometric-solid model on our own H = 1.78 reference body and
publish the solids), not a research one. **Do not leave it flagged
"pending literature". The literature is exhausted.**

### Method notes for the next Toto

- **Query Unpaywall / OpenAlex / Semantic Scholar by DOI before hunting
  for a PDF.** Three API calls converted "PAYWALLED (couldn't find one)"
  into "PAYWALLED (0 OA locations across every indexed repository)". That
  is the difference between an excuse and a finding, and it takes 20
  seconds. `api.unpaywall.org/v2/{doi}?email=…` is the highest-value one.
- **When a paper is locked, look for an implementation of it.** A
  paywalled *method* often has an open-source reimplementation, and code
  is frequently more informative than the paper because it cannot be
  vague. `segment_shoulder.m` answered a question the abstract could not.
  **Label it Tier S and honour the authors' own warnings** — this repo
  says its calculations may be "incorrect", so I took structure and one
  density from it, and no shipped number.
- **Beware over-valuing the source you cannot read.** I called Hatze "the
  single highest-value unread source" for a paper that structurally cannot
  answer the question asked of it. Unread sources accumulate imagined
  value. Ask "what *shape* of answer could this even contain?" before
  ranking it.
- **State comparisons as ratios, never as orders of magnitude.** See
  Task 1. Three copies of one error, zero divisions performed.
- CORE.ac.uk's `/v3/search/works` did **not** honour a quoted phrase
  query (returned 4.48M hits of unrelated material). Do not trust its
  result count as a negative.
- OCR is **still** the standing blocker: BR-13's `hatze-cheatsheet.pdf`
  is 13 pages of which 11 have a zero-length text layer. It joins
  Dempster, Klein Breteler, the Drillis tables and the NASA scan.
- **Tier-V is still 0.** Not touched this pass either.

### What I would read next to close what I just left

1. **Nothing, for the clavicle's *mass*.** That is the point of this
   pass — it is closed as a research question. The next move is
   engineering (§5 above).
2. **Winter Table 4.1 from the physical book** — still the cheapest open
   item in this ledger and still unresolved (CONTRADICTION-1). Now
   *more* worth doing: Hatze's 1050 kg/m³ agrees with Winter's shoulder
   density to 1 %, so the row is partly corroborated and the blank mass
   cell is the remaining question.
3. **Hatze (1979) CSIR Tech. Report TWISK 79, Pretoria** — the fuller
   precursor BR-13's authors actually worked from, cited as having a
   "p39". A South African government technical report is a **far better
   OA prospect than the Elsevier paper**, and it is the version with the
   derivations. **This is the new top literature target, and it replaces
   Hatze 1980 in that slot.**
4. **Drillis, Contini & Bluestein (1964) Tables 5–7 via OCR** —
   unchanged; blocked on OCR.
5. **Zatsiorsky (2002) BR-14** — only if someone acquires the physical
   book. Low expected value now (GAP 1 §5), but it is the last unchecked
   place.
