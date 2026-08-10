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

---

## 2026-08-08 — Dispatch: four-stage armour damage (Fresh → Scuffed → Cracked → Severed)

**Ledger written:** `armor-damage/SOURCES.md` (new, IDs AD-01…AD-09).
**Asked:** is a four-stage progressive-damage model defensible for 24
steel-analogue plates? How much protection survives per stage? Does hit
LOCATION compound? What differs between penetrating / blunt / edged?

**Answer: staged degradation is real and measurable — but the model is
keyed off the wrong variable. Damage in steel plate is POSITIONAL, not
cumulative.**

### Sources read end to end this pass

| ID | Source | Status |
|---|---|---|
| AD-01 | NIJ Standard-0101.06, NCJ 223054, 89 pp | **READ** — §2, §3 defs, §7.6, §7.8.5.1, Tables 4–7 verbatim; conditioning sections skimmed |
| AD-02 | Qiang et al. (2022) *Proc IMechE Part C*, DOI 10.1177/09544062221080139 | **READ**, all 40 pp of the NUAA-hosted author manuscript |
| AD-03 | Børvik, Langseth, Hopperstad & Malo (2002) *Int J Impact Eng* 27:19–35 Part I | **READ**, all 17 pp end to end |
| AD-04 | Göde et al. (2023) *Eng Sci Tech Int J* 38:101337, CC BY-NC-ND | **READ**, all 9 pp |
| AD-05 | Seidl, Lehmann & Grobert, ISB proceedings, DOI 10.52202/080042-0046 | **READ**, all 10 pp |
| AD-06 | Alan Williams, *The Knight and the Blast Furnace* (Brill 2003) | **UNREACHABLE** — commercial title, no OA copy anywhere |
| AD-07 | Forsom & Smith (2017) | **OFF-TOPIC** — arrowheads into cattle scapulae, no armour. Logged so it is not re-chased |
| AD-08/09 | ceramic damage-zone taxonomy; "one-third residual strength after two hits" | **SNIPPET-ONLY — NOT CARRIED.** AD-09 was the most tempting number in the dispatch and I did not read its paper |

**5 counted. No breadth quota pursued.**

### The finding that reframed the dispatch — and it was a NULL

**AD-04: eight 7.62×51 M61 AP rounds at 815–829 m/s into ONE 12.4 mm Armox
600T plate. All eight defeated. Verbatim: *"The multi hit shootings did not
influence the main ballistic resistance since the max depth of the craters
is very similar."*** Against **AD-02**, where two hits at the *identical*
spot drop capacity to 0.346 and three to 0.212 of pristine.

Those do not contradict — they are the two ends of one spatial curve, and
AD-02 measured where it turns over. Projectile D = 12.7 mm; offsets 0 D /
1.00 D / 1.97 D. At 1 D the *"damage caused by the second projectile
complemented that caused by the first… inferior ballistic resistance"*; at
1.97 D the *"damage zones… did not overlap"* and resistance was *"almost
not affected."* **Interaction radius ≈ 1 projectile diameter; independent
beyond ≈ 2.** AD-01 corroborates from the regulatory side — mandated
minimum shot-to-shot **51 mm** (≈ 6.7 calibres for 7.62 mm) and shot-to-edge
**51 mm / 76 mm** — and both AD-01 and AD-02 independently say hits near a
plate EDGE are weaker.

**So per-piece HP cannot express the thing the physics is about.** One
stored "last significant impact point + radius ≈ 2 projectile diameters"
per plate captures nearly all of it.

### The degradation curve is concave, not linear — arithmetic in the ledger

From AD-02's own FE triple (`V_L` 206.3, `V_2L` 121.3, `V_3L` ≈ 95 m/s),
energy ∝ v²: capacity **1.000 → 0.346 → 0.212**; marginal loss 0.654 then
0.134; **ratio 4.89**. The first co-located hit costs ~4.9× what the second
does. A linear four-step ramp is wrong in shape. (Ratio written out per the
standing rule from the last pass — no order-of-magnitude adjectives.)

### Five contradictions of the proposed design, recorded not averaged

1. **Linear degradation** — contradicted (4.89× front-loading).
2. **"Spall/pitting" as stage 2** — **unsupported for steel.** Absent from
   AD-02/03/04; in AD-04 it is the *projectile* that shatters. Spall zones
   are a ceramic phenomenon.
3. **Hit-count HP** — contradicted; position governs.
4. **All damage types alike** — contradicted with a number: **2.51×** energy
   difference blunt vs round-nose on the same plate (AD-03), and 40 %
   penetration-depth difference from tip geometry alone (AD-05).
5. **Hit order irrelevant** — contradicted. AD-02: Cone-Sphere is strictly
   worse for the plate than Sphere-Cone. Pointed weapons *prime* a plate.

Plus a conditional: **"Scuffed" only exists for medium steel.** AD-03's
Weldox 460 E dishes 0.62–4.10 mm; AD-04's Armox 600T *"cannot plastically
deform, but instead crack."* A hard plate goes Fresh → Cracked → Shattered
with no scuffed stage.

**"Cracked" is the best-evidenced stage in the model** — three independent
sources show non-perforating hits leaving through-thickness cracks (AD-03
test H2: a crack *"almost completely around the circumference of the
bulge"* on a plate that held).

### A contradiction between two READ sources, left unresolved

AD-02 and AD-03 order the nose shapes **oppositely** — AD-03 (h/D = 0.600):
blunt easiest, `v_bl` 184.5 vs ~291 m/s. AD-02 (h/D = 0.056): conical
easiest. AD-03's own Introduction already calls the prior literature
*"to some extent incompatible"* and says *"the problem is not yet fully
solved"*; AD-02 warns its own ordering *"may be various"* for other
thicknesses. Woodward's criterion (plugging favoured when `h < √3·D/2`)
explains AD-03 but **not** AD-02. **Only the direction both agree on is
safe: tip geometry matters, by a factor ~2.5 where measured cleanly.**

### Method notes for the next Toto

- **The DSpace REST API is now a proven route, twice over.** Unpaywall said
  AD-04 was `gold` OA but returned **zero `url_for_pdf`**; ScienceDirect
  403'd; the naive `/bitstreams/{uuid}/content` guess 404'd. What worked:
  `discover/search/objects?query=` → item uuid → `core/items/{uuid}/bundles`
  → `ORIGINAL` → `/bitstreams` → `_links.content.href`. Same shape as the
  4TU route in the body-rig pass. **Treat it as the standard move for any
  institutional repository.**
- **"Gold OA" ≠ fetchable.** Openness status and retrievability are
  different facts. Check the second one.
- **Verify a formula by reproducing a number the paper already printed.**
  AD-03's Woodward criterion came through OCR mangled as `√3 p D/2`;
  computing `√3 × 20/2 = 17.321` against its own text *"about 17 mm"*
  confirmed the reading before I used it. Two minutes.
- **Read the conclusions of the boring-looking experiment.** The pass's most
  valuable result was AD-04's null.
- **No OCR needed on this topic** — first time in this repo. All five PDFs
  had genuine text layers (2 000–6 500 chars/page).
- **Licence, logged per R6:** AD-04 is **CC BY-NC-ND 4.0**, read verbatim
  off its own front matter — same trap family as LaFAN1. Facts citable,
  artifact not redistributable. AD-03 was obtained from a third-party
  mirror (`aux.ciar.org`), is © Elsevier, and **the mirror must not be
  linked as if it were authorised** — I verified it as the published
  version by its PII line and pagination. Nothing this pass is proposed for
  shipping, so R6's shipping test is not triggered.
- **Tier-V is still 0.** Not touched.

### What I would need to read next to close the gaps I just left

1. **A repeated-rifle-fire study on a body-armour-scale steel plate.** The
   standing hole: AD-02 is 0.71 mm stainless, AD-03 is a 197 g slug, AD-04's
   headline is a null. **Nobody I found measures progressive degradation of a
   2–12 mm steel plate under repeated rifle rounds at controlled spacing.**
   That single study would replace most of this ledger's extrapolation.
2. **AD-02's reference 23** — the *experiment* behind `V_L = 205.1` and
   `V_2L = 127.2`. Those two numbers reach me only through AD-02's Table 2.
   Reading the originating paper would also likely give intermediate offsets
   between 1 D and 2 D, which is the ledger's sharpest missing resolution.
3. **Quasi-static punching of plate** (Johnson, Ghosh & Reid's survey, cited
   by AD-03). **The only route to the spear-thrust case** — everything I have
   is 146–300 m/s and a thrust is ~5–10 m/s. Do not extrapolate; read this.
4. **Anything at all on a CUTTING edge vs plate.** Zero sources found. The
   dispatch groups spears as "edged" and the entire edged category is
   currently unsourced — every pointed source here is a penetrator.
5. **Alan Williams (AD-06)** — only if someone acquires the physical book.
   The 80 J / 100 J / 120 J mail figures that circulate online came to me via
   a third-party summary only and are **NOT CARRIED**. Given this repo's
   history that is precisely the shape of number that gets fabricated; leave
   it out until someone reads the page.

---

## 2026-08-08 — Dispatch: vertical maps, flight, and what bots can cope with — TOTO33

First entry by **TOTO33**, the practitioner-tier researcher (talks,
postmortems, dev blogs). **Ledger written:** `vertical-maps/SOURCES.md`
(new, IDs VM-01…VM-18).

**Asked:** what changes about level design when players can fly; how
shipped games structure altitude bands; how a multi-level map stays
readable; how to join a castle-on-a-cliff to half a city to open ground;
and — urgently — what breaks for bots on vertical geometry, because a
builder is being asked that question right now.

### THE HEADLINE: tier-V is no longer 0

Every ledger in this repo has reported **tier-V = 0** since the project
began. This log said so three times, most recently "Not touched this pass
either." The recorded blocker was GDC Vault gating.

**The blocker was misidentified.** Two routes work, both trivial:

1. **`pip install youtube-transcript-api`.** GDC talks published on the
   official YouTube channel return full timestamped auto-captions.
   Three talks read this pass. Worked first try.
2. **Internet Archive carries OCR'd GDC slide decks** at
   `archive.org/download/<item>/<item>_djvu.txt` for Vault-gated talks.
   That is how VM-06 (Brewer, GDC 2015 AI Summit) was read.

**Both routes were already reachable from existing ledgers and nobody
tried them.** `maps/SOURCES.md` S-02 (Yoder, "Holy Grail of Multiplayer
Level Design") sat SNIPPET-ONLY and uncounted since 2026-08-01 — I read
the whole 27 minutes. `aiming/SOURCES.md` S-01 (Weihs, aim assist) already
records the URL `archive.org/details/GDC2013Weihs` **in its own source
row** and is still marked "NOT COUNTED until watched". That is a slide
deck one `curl` away.

**Standing correction for the next researcher: "watched" was the wrong
verb and it froze this tier for the whole project.** We cannot watch
anything. We can read transcripts and slides, and for these two talks the
text was always there. **5 of the 14 sources this pass are tier V.**

### Sources read end to end this pass

| ID | Source | Status |
|---|---|---|
| VM-01 | Jim Brown / Epic, "The Importance of Nothing", GDC 2014 | **READ-TRANSCRIPT** (auto-caption, full 51 min) |
| VM-02 | Andrew Yoder / Hi-Rez, "Holy Grail of Multiplayer Level Design" | **READ-TRANSCRIPT** (full 27 min) — upgrades `maps/` S-02 |
| VM-03 | Beinke-Schwartz, "Singleplayer vs Multiplayer LD", GDC 2017 | **READ-TRANSCRIPT (PARTIAL)** — scanned all, read 5 passages |
| VM-04 | Crystin Cox / ArenaNet, "Gliding in Central Tyria", 2016 | **READ-WRITEUP** (raw HTML fetched and read, not summarised) |
| VM-05 | Brewer / Digital Extremes, *Game AI Pro 3* ch.21 | **READ** (full 10 pp) |
| VM-06 | Brewer, GDC 2015 AI Summit slides via Internet Archive | **READ-SLIDES** (OCR, no narration) |
| VM-07 | Guerrilla, Killzone 3 MP bots, *Game AI Pro* ch.29 | **READ** (full 14 pp) |
| VM-08 | Kirst, voxel navmesh coverage, *Game AI Pro 2* ch.32 | **READ** (full 16 pp) |
| VM-09 | Butcher & Griesemer / Bungie, "Illusion of Intelligence", GDC 2002 | **READ-SLIDES** (27 slides) |
| VM-10…13 | The Level Design Book: Verticality, Composition, Wayfinding, Circulation | **READ** (`.md` sources pulled to disk) |
| VM-14 | Bennett Foddy, "zk map for stranger", 2021 | **READ-WRITEUP** |
| VM-15 | Gamasutra on Titanfall 2 action blocks | **SECOND-HAND**, and does not answer the question — logged so it is not re-chased |
| VM-16 | Mononen, "Automatic Annotations in Killzone 3" | **NO-TRANSCRIPT** — identified, not retrieved. Top unread target |
| VM-17/18 | ArenaNet on designing HoT *for* gliding; any Respawn/DICE primary | **NOT FOUND / PAYWALLED** — stated plainly, not substituted |

**14 counted. Tier V: 5.**

### The finding that outranked the literature — and it came from the source tree

The urgent question was "what can bots cope with". I read `sim.rs`
read-only before reading anyone's opinion, and the answer is not a
literature question:

- `Fighter` carries `pub waypoint: [f32; 2]` — **x and z. No height.**
- Waypoints are uniform-random samples inside a **square** of half-extent
  `self.half`, team-biased, then clamped — **never checked for
  reachability, collision, or line of sight** (`sim.rs` approx L11976-12016).
- Grep across `src/` for `navmesh|nav_mesh|pathfind|path_find|a_star|astar`:
  **exactly two hits, neither of them code** — one false positive from the
  test name `head_glance_is_a_glance_not_a_stare`, and one comment saying a
  morale fixture "had drifted into testing PATHFINDING". **There is no
  navmesh, no path graph and no pathfinder.**
  *(I first wrote "20 hits, all comments". That regex included `waypoint`,
  which matches the real bot field and several research `.md` files, so the
  count and the characterisation were both wrong. Re-run narrowed, hits
  re-read individually, corrected here and in the ledger. The corrected
  finding is stronger than the botched one.)*

**Bots walk in a straight line at a random 2D point.** On the flat arenas
that ship, that reads as roaming and is the right design. On a
castle-on-a-cliff map, sampled waypoints land inside the mountain and the
bot presses into the rock until the 15% re-roll fires — an *intermittent*
failure, which is the worst kind to diagnose. Bots have no notion of "up"
and can never choose to go to the castle.

**This is the only item in the whole pass that is a defect rather than a
risk**, and it is what I sent the map builder first.

### What the practitioner tier gave that papers could not

- **VM-01, Gears of War *Gridlock*: they changed the art and nothing
  else, and lost the map.** "We hadn't changed the physical play space the
  weapon load out or even the collision — but just that shift in tone and
  color meant a change in perception" [21:37]. Most-popular map to bottom
  of the time-played chart, rebuilt, then "back in the top three most
  played maps of Gears of War 3's lifetime" [22:17]. **Independently
  corroborated by VM-02 [24:20]**, different studio and decade: Hi-Rez's
  *Corey* tested well as grey-box and went lukewarm after the art pass.
- **VM-02, the flight rule, from a studio with flying characters:** "what
  you see is what you get… we have characters that can fly" [19:57].
  Flight abolishes the difference between scenery and playable space.
  Cheap at blockout, expensive later.
- **VM-04, what actually breaks when you add flight to grounded maps:**
  progression/skippable content first (got a bespoke No-Fly Zone
  mechanism), scripted interiors next ("just not designed to handle
  gliding" so flight was disabled wholesale), out-of-bounds geometry last
  (tolerated).
- **VM-07 + VM-09, twenty-four years apart, same answer:** give the AI a
  discrete set of *authored* positions. Guerrilla: "**We chose to manually
  place these, because they would require complex terrain reasoning to
  generate automatically.**" Bungie: firing points, "need a discrete
  answer to a continuous problem."
- **Tried and rejected** — the thing only this tier supplies: stacked
  navmesh layers and 3D waypoint graphs for flight (VM-05, with the
  boundary condition that *favours* us — layers "work well in confined
  spaces", the rejection is scoped to 2 km volumes); 3D Jump Point Search
  ("an order of magnitude slower than the tweaked heuristic A-star");
  Hi-Rez's *Sewer*, a map deliberately enclosed "so you can't really fly
  anywhere" — "on average players didn't like it so much" (VM-02 [23:42]).

### Two contradictions recorded, neither averaged away

1. **CONTRADICTION-1:** `maps/SOURCES.md` S-01 says stairs should run
   30-35 deg; VM-08's illustrative walkable-slope limit is `maxSlopeRad` =
   0.5 rad = **28.65 deg**. A ramp at the recommended human-comfort slope
   is steeper than that navmesh will walk. They do not truly conflict
   (voxel generators take stairs through `maxStepHeight` per riser, not
   the aggregate angle) but they are 1.05-1.22x apart and someone will
   eventually build a smooth 33 deg ramp and find bots refuse it.
2. **CONTRADICTION-2:** VM-01 states a "280 times increase in detail" from
   130 polygons to 60,000 triangles. **60,000 / 130 = 461.5, not 280.**
   His *other* comparison reproduces (30,000 / 130 = 230.8 vs "more than
   200 times"), so the 280 is the anomaly — either a polygons/triangles
   unit mismatch or a caption error. **NOT CARRIED.** Ceiling: "two to
   three orders of magnitude", no specific multiplier.

### Numbers I refused to carry

**VM-12's 20-row wayfinding table with "% certainty" values** (allegory
1% … hard barriers 98%). **The authors' own text calls them "a
non-scientific estimate".** It is the most quotable-looking table in this
ledger and it is vibes with a decimal point. Ordering kept, percentages
dropped. Same family as `aiming`'s fabrication and `armor-damage`'s AD-09.

Also **not answered by anybody: how far apart altitude bands should be, in
metres.** No practitioner source found gives a figure. The only defensible
ceiling is VM-01's, and it is about silhouettes rather than distance —
vertical extent is bounded above by the range at which a 1.78 m fighter
still reads as a figure against his background. **This project can measure
that**, using the `max_unobstructed_sightline` instrument `maps/SOURCES.md`
already records, pointed upward.

### Method notes for the next researcher

- **The two transcript routes above. Use them before declaring a talk
  unreachable.** "PAYWALLED" was true of the Vault and false of the talk.
- **Auto-captions drop negations.** VM-02 [20:13] reads "we want to make
  sure that as you're going over rooftops you're hitting invisible walls",
  which is the exact opposite of the speaker's meaning; context fixes it
  beyond doubt. **Carry the principle, never that sentence.** Flag the
  repair at the point of use, not in a footnote.
- **Divide the numbers before repeating the adjective.** That standing
  rule from the body-rig pass caught CONTRADICTION-2 on its first
  outing against a new source type. It keeps earning its place.
- **Read the source tree before the literature when the question is
  "what can our thing cope with".** The single most important finding this
  pass took one grep and was not in any paper.
- **`.md` versions of GitBook sites** are available by appending `.md` to
  any page URL, and `llms.txt` at the site root is a full index. That is
  how VM-10…13 were read as files rather than summaries.
- **A tool summary is still not a source.** WebFetch's pass at VM-04
  produced a decent summary with quotes; I fetched the raw HTML, stripped
  it and read it anyway. It happened to be accurate. That is not a reason
  to stop.

### What I would read next to close what I left open

1. **VM-16, Mononen, "Automatic Annotations in Killzone 3 and Beyond"** —
   the primary on deriving cover from geometry. It is a *blog* (slides on
   `digestingduck.blogspot.com`), so it should be reachable. Highest value.
2. **A Respawn or DICE primary on flight / vertical map design.** Sweep
   the GDC YouTube channel by title now that transcripts work, before
   concluding it does not exist. Titanfall, Battlefield, Anthem and Just
   Cause currently contribute **nothing** to this ledger.
3. **Recast/Detour off-mesh connections and jump links** — the documented
   hole in VM-08, which covers crouch/prone/swim and never mentions
   cliffs. **This is a code-reading job — hand it to TOTO22**, which is
   the entire reason we are separate roles.
4. **Bellard, "Environment Design as Spatial Cinematography", Rockstar,
   GDC 2019** — the credible opposing view to VM-11's "leading lines are
   brain poison", named by VM-11 itself. On YouTube, so transcriptable.
5. **Kevin Lynch, *The Image of the City* (1960)** — the districts/edges/
   nodes model is the whole theoretical basis of my Q4 answer and it is
   currently second-hand through VM-12. **Nobody here has read Lynch.**

— TOTO33

---

## 2026-08-10 â€” Dispatch: write `MAP_METRICS.md`. SYNTHESIS, not search.

**Deliverable written:** `research/maps/MAP_METRICS.md`. Closes
`TRV-0149`; its Â§9 closes `TRV-0051` and `TRV-0180` explicitly.
**Ledger updated:** `research/maps/SOURCES.md` (three corrections + a
deliverable line).

**The dispatch was explicit that this was NOT a research cycle** â€” Rule
13, build over research. One external source was touched, and only to
re-verify a ledger row. No new sources sought. Quota unchanged at 1/12
and deliberately not padded; that is the correct outcome for a synthesis
pass and it should not be read as a shortfall.

### What made it worth doing beyond restating the sources

The value was not in the literature. It was in **deriving the numbers
this game's own constants already imply and nobody had computed.** Three
findings that did not exist anywhere before this pass:

1. **The ledge ceiling is `apex + STEP_UP`, not `apex`.** `support_top`
   accepts any cover top with `c.max[1] <= y0 + step_up` (`sim.rs:1052`)
   and the horizontal push-out skips that same set (`sim.rs:9285`) â€” so a
   box stops blocking you and becomes your floor in the same tick, 0.55 m
   before you have cleared it. Soldier ceiling is **2.26 m**, not 1.71 m.
   Every ledge-band number in the project would have been 0.55 m wrong.
2. **A shipped defect, quantified.** The shared infill draws "shoulder"
   cover uniformly from 1.6â€“2.2 m and Battlefield clutter from 1.1â€“2.4 m.
   Both ranges **straddle a traversal threshold**, so ~21% of shoulder
   cover and ~11% of clutter differs in mountability from its identical-
   looking neighbour, drawn by the same line of code. Four-line fix,
   written up in `MAP_METRICS.md` Â§9.2.
3. **`max_unobstructed_sightline` is flat-map-only.** It samples at an
   ABSOLUTE `y = EYE_REL` (`sim.rs:9643`), so on any multi-band map every
   position above y=0 is "buried" and dropped. Cliffhold's number
   measures the ground band and nothing else. `terrain_top`
   (`sim.rs:1065`) already exists and is the one-line fix.

### The stale-number catch â€” and the method that caught it

`maps/SOURCES.md` recorded sightline baselines of Arena 80.2 / Bailey
93.4 / Gardens 92.0 / Battlefield 509.9 m. **The dispatch told me to
verify against source rather than trust the table, and it was right.**
Re-running the shipping test gave 102.9 / 120.2 / 115.0 / 637.4 /
Cliffhold 577.1. Two of the four ratios are exactly **1.250** and two are
1.283â€“1.287: the baselines predate the `MAP_SCALE = 1.25` map expansion.
**Standing lesson: a recorded measurement in a ledger has a date and a
build, and neither is usually written down.** Numbers taken from a
running system go stale in a way numbers taken from a paper do not.

`BODY_HEIGHT`, `EYE_REL` and `BODY_RADIUS` had **not** drifted.

### On S-01, and why I re-fetched a source the ledger already marked READ

My first extraction pass on the Level Design Book metrics page returned
every dimension table but **not** the 30â€“35Â° stair slope or the "landings
every 12â€“16 steps" figure â€” both of which `maps/SOURCES.md` attributes to
it. Given this repo's fabrication incident (`aiming/SOURCES.md`), that
gap had to be closed rather than assumed benign. A second targeted fetch
**reproduced both verbatim**, with the source's own derivation
`arctan(7/11) = 32 degrees`. I checked its arithmetic: 32.47Â°, and all
four of its engine step-slopes (33.69, 30.96, 33.69, 32.47) fall inside
its own 30â€“35Â° claim. **S-01 is clean and the ledger row was accurate.**

Recording the near-miss anyway: a first pass that silently omits a figure
looks identical to a first pass that never had it. The only defence is
asking the second question.

### A contradiction resolved by measurement rather than averaged

`vertical-maps/SOURCES.md` CONTRADICTION-1 â€” S-01's 30â€“35Â° stairs vs
VM-08's 28.65Â° illustrative navmesh slope limit. **Does not bite this
game**, on two independent grounds: our stairs run 18.4â€“22.6Â° (derived
from `STAIR_RISE_M = 0.5` over 1.2â€“1.5 m treads), and there is no voxel
navmesh here for the limit to apply to. Logged as resolved, not blended.

### The 40 m rule â€” I changed the ledger's conclusion

`SOURCES.md` said the IX-A rule "binds NEW maps, not retrofits". **That
is wrong and the reason is arithmetic.** It is a global maximum over all
pairs, so any single open line anywhere sets it. The validator's own
instrument check asserts an empty Arena reads its own ~117 m diagonal.
**A 40 m global max is unsatisfiable on any map above ~15 m half-extent**
and always was. Proposed retire-and-replace (a local objective-pair rule
+ a distributional median-in-25â€“35 m rule) is in `MAP_METRICS.md` Â§6.4
and is **labelled a DESIGN PROPOSAL by me, not a finding.** It needs an
owner decision.

Independent corroboration of the rule's *order* came free: transferring
TF2's ranges by ratio to the Source player height (256/72, 1024/72,
2048/72 Ã— 1.78 m) puts "medium" at 25.3 m and "sniper" at 50.6 m, with
40 between them. Transferring by ratio rather than guessing a
units-to-metres constant is the honest route and I recommend it as
standing practice.

### What would I need to read next to close the gaps I just left?

1. **Nothing, for band SPACING â€” because it does not exist.**
   `vertical-maps` Â§Q2 already searched and found no practitioner text
   giving a vertical spacing figure in metres. My 2.26 m is a
   *traversal* bound (when two bands stop being one surface), not a
   design bound (when two bands stop being interesting). **Reading more
   will not produce that number. A playtest will.** Stop looking.
2. **A bot-aperture experiment, not a source.** Â§3.2's 7.0 m floor is an
   analogy to `BOT_CLIMB_LANE_M`, and the real answer is an afternoon:
   wall with a gap, roam one team 60 s, count crossings, sweep 2â†’24 m.
   This is the one gap where the repo can generate its own primary data.
3. **VM-16 â€” Mononen, *Automatic Annotations in Killzone 3 and Beyond***
   (slides said to be on `digestingduck.blogspot.com`). Still
   `NO-TRANSCRIPT`, still the top unread target, and it is the primary on
   deriving cover and firing points *from geometry* â€” which is exactly
   what a map with no pathfinder would need. Unchanged from TOTO33's
   list two days ago.
4. **A capture, not a paper**, for the one derived claim I could not
   observe: a heavy mech standing on 3.4 m hard cover (2.586 + 0.935 =
   3.521 > 3.4). One screenshot settles it.

### Method note for the next Toto

**The two highest-value things I did were both re-checks, not searches.**
Running a test that already existed, and fetching a source a ledger
already marked READ. Both found errors. In a repo this far along, the
research is usually done and the ledger is usually the weak link.

— TOTO (maps synthesis pass, appended by the session that dispatched it)

---

## 2026-08-10 - Motion architecture: the DECISION, copied from motion-architecture/NOTES.md

(That dispatch was scoped to its own directory and could not write here. Copied across 2026-08-10 by the dispatching session, as the note in NOTES.md asked.)

# SESSION 2 â€” 2026-08-10 â€” the refusal above was overridden, and why that was right

**`DECISION.md` now exists.** Read it, not this file, for the decision.
This section is the working record: what was done, what was corrected,
and where the honest gaps are.

## The refusal was correct and its premise turned out to be wrong

Session 1 (above) refused to write the decision because axis 5 â€”
per-character CPU cost at crowd scale â€” had no evidence. That was the
right call **given what it had read.** It had read papers and crate
metadata. It had not read this repository.

Reading the source first changes the shape of the problem entirely:

1. **Families B and C are eliminated by the R5 licence/asset gate before
   axis 5 is ever consulted.** We hold zero hours of licence-cleared
   mocap and zero animation clips; LaFAN1 is CC BY-NC-ND 4.0. A
   per-character cost of 0 ms would not make an unavailable architecture
   available. **Axis 5 could only ever have decided between two
   available options, and there is only one.**
2. **The "crowd" in axis 5 is not animated.** `jk_wall` (the 250v250
   sim) has no rig, no pose layer, and no bevy dependency at all. The
   animated population is `jk_tdm`'s: 16 fighters (`per_team` clamped to
   8) + 40 zombies (`ZOMBIE_CAP`) = **56**. The number session 1 was
   blocked on was for a population that does not exist.
3. **The number is measurable here.** `jk_spike/src/bin/bench.rs` already
   walks a body-count ladder; `autoplay_report` already drives headless
   matches. `DECISION.md` Â§9 specifies BM-1 and BM-2 precisely enough to
   build. **Axis 5 stops being a research blocker and becomes a build
   task**, and the resulting figure is for this game on this hardware,
   which no talk could supply.

So: the gap session 1 named is still open as a *literature* question and
is marked as such in `DECISION.md` Â§2 (axis 5 declares **no winner**) and
Â§10.4. It is no longer load-bearing.

## The greps that did the work, recorded so nobody re-runs them blind

Against `engine/crates`, commit `e2866a9`:

- `AnimationPlayer|AnimationGraph|AnimationClip|AnimationNodeIndex|AnimationTransitions`
  â†’ **4 hits, none in code** (two `Cargo.toml`, two research `.md`).
  The `bevy_animation` cargo feature is enabled and entirely unused.
- Glob `engine/crates/jk_tdm/assets/**` â†’ **no files.** No clips, no glTF,
  no BVH. There is no motion content of any kind in this project.
- `foot_ik|ground_ik` in `jk_tdm/src` â†’ **zero hits.** Legs are open-loop
  gait sinusoids (`main.rs:17759-17812`). **This is the one genuinely
  missing core-scope item** and `DECISION.md` Â§7 Step 2 closes it.
- `solve_arm_ik` â†’ **8 call sites**, 6 on the fighter rig, 2 on the
  viewmodel, 1 in a test. Two-bone IK is already load-bearing here.
- Line counts, for whoever has to work in these files: `sim.rs` **27 737**
  lines, `main.rs` **29 261**. `DECISION.md` Â§7 Step 1 (extract the pure
  pose kernel into its own module) is partly motivated by that alone.

## The single sharpest finding, and it is not in any paper

`jk_tdm`'s sim classifies hits by **height fraction** (`HitZone`), and
the render is clamped to respect it â€” `gait_pose`'s own doc comment
(`main.rs:2300-2310`) records a real bug where a settle dip put the head
base at ~0.79 of height, "outside the 0.82 band the test claims to
guard, and classified as Arms by the sim while looking like a head."

**A pose retrieved from a motion database, or emitted by a network, does
not know about your hit bands.** It puts the head where the data put it.
Every frame of disagreement is a frame where the player shoots what he
sees and hits something else. Motion matching here would need a
constraint layer on top whose entire job is to undo the data. That is a
game-specific structural argument against family B that no amount of
reading SIGGRAPH would have produced, and it took one doc comment.

## Corrections to session 1

Both are written up in full in `SOURCES.md` Â§"Corrections to session 1":

1. **The `bevy_mod_inverse_kinematics` quick win is reversed.** Its
   licence and version facts stand (MIT/Apache-2.0, 0.8.0 â†’ `bevy ^0.15`,
   confirmed via the crates.io API). Its premise does not: session 1
   wrote that the body's "grip poses and reach are hand-posed", and they
   are not â€” `solve_arm_ik` is a full two-bone solver with a pole vector,
   plus an elbow clamp and sprung targets the crate lacks. Rejected on
   axis 9, duplicate capability, `DECISION.md` Â§5.4, with the condition
   that would reopen it (chains longer than two bones â€” for which the
   crate would not help either, being two-bone only).
2. **This file's tier-V blocker paragraph is stale.** TOTO33 solved it on
   2026-08-08 (`youtube-transcript-api`, and Internet Archive
   `_djvu.txt` for Vault-gated decks). **It is nevertheless still 0 today
   for an unrelated reason:** this session's shell has no `curl`, no
   `wget`, no Python, no `git`, no coreutils â€” every probe exits 127.
   Only `WebSearch`/`WebFetch` reach the network, and the Learned Motion
   Matching PDF is over the 10 MiB fetch limit with no local-download
   route. **Tier-V reachability varies by session. Probe the shell
   before writing it off, and record which way it went.**

## What I would need to read next to close the gaps I left

1. **Nothing, to make this decision.** That is the point of Â§3 of
   `DECISION.md`. The R5 gate is decisive and it is already closed by a
   licence fact and an empty `assets/` directory. **Do not re-open this
   as a literature question.**
2. **BM-1, our own benchmark** (`DECISION.md` Â§9.2). This is the
   highest-value next artefact on the topic and it is a *build* task, not
   a read task. It closes axis 5 with a number that is better than any
   citation because it describes this engine.
3. **The Learned Motion Matching PDF** â€” only if risk (1) in
   `DECISION.md` Â§8.3 fires, i.e. if someone licences commercial mocap
   and family B comes back on the table. Needs a session with a shell
   that can download a 10 MiB file. Until then it is the classic
   over-valued unread source this log warned about after the Hatze pass.
4. **`bevy_animation_graph`'s compatibility table, the 0.15 row
   specifically** â€” the one unresolved factual question about a real,
   permissively-licensed, genuinely useful crate. Its partial-ragdoll
   feature (some bones simulated, some kinematic) is the most valuable
   third-party capability found on this topic, and it becomes relevant
   the day this project acquires a rigged character with clips.
5. **A talk on motion-matching search cost, via TOTO33's transcript
   route, from a session that has Python.** Explicitly *not* on the
   critical path â€” it would be a cross-check on an order of magnitude for
   a family we have rejected. Listed last on purpose.

**Note on the log:** the canonical entry belongs in
`research/TOTO_LOG.md`, but this dispatch restricted writes to
`research/motion-architecture/`, so it is recorded here instead. Whoever
lifts that restriction should copy this section across.

â€” session 2
