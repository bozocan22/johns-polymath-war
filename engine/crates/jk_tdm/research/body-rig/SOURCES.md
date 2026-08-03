# body-rig — SOURCES

Ledger for `SPEC_20_SEGMENT_RIG.md`. Standing rules: `.claude/agents/toto.md`.
Status vocabulary: **READ** (actual text read) / SKIMMED / SNIPPET-ONLY (search
result only — **does not count**) / UNREACHABLE / PAYWALLED.

Before adding anything here, read `../aiming/SOURCES.md` — it contains this
project's fabrication incident. **A tool's summary is not the source.**

---

## Ledger

| ID | Tier | Citation | URL | Accessed | Status | What it gave us |
|---|---|---|---|---|---|---|
| BR-01 | P | de Leva, P. (1996) "Adjustments to Zatsiorsky-Seluyanov's segment inertia parameters." *J Biomech* 29(9):1223-1230. Table 4, male column. n=100 living young adult males, mean age 23.8, gamma-ray scan, reference body 73.0 kg / 174.1 cm. | doi:10.1016/0021-9290(95)00178-6 | pre-2026-08-03 | **READ** (inherited; verified by two independent extraction engines agreeing digit-for-digit, mass fractions summing to 1.0000) | 18 of the 20 rows in spec §2.1. Whole-foot CoM **0.4415 of foot length from the heel** and whole-foot sagittal r **0.257** — the two anchors the toe derivation is solved against. |
| BR-02 | P | Winter, D.A. *Biomechanics and Motor Control of Human Movement*, Table 4.1. Every source code is a Dempster derivative. | (book) | pre-2026-08-03 | **READ** (inherited) | The single "Shoulder mass" row (sternoclavicular joint → glenohumeral axis): CoM **0.712 proximal / 0.288 distal**, density 1.04. Project's primary read records **mass cell blank, all three radius-of-gyration cells em-dashes**. See CONTRADICTION-1. |
| BR-03 | P | Dempster, W.T. (1955) WADC-TR-55-159. 8 cadavers, ages 52-83. | — | pre-2026-08-03 | **UNREACHABLE** (inherited; Deep Blue SPA behind Cloudflare, DTIC 403, HathiTrust page-turner only) | Nothing. Zero primary numbers ever obtained by this project. |
| **BR-04** | **P** | **Matsumoto, Y., Ogihara, N., Hanawa, H., Kokubun, T., Kanemura, N. (2022) "Novel Multi-Segment Foot Model Incorporating Plantar Aponeurosis for Detailed Kinematic and Kinetic Analyses of the Foot With Application to Gait Studies." *Front Bioeng Biotechnol* 10:894731. CC BY.** | https://www.frontiersin.org/journals/bioengineering-and-biotechnology/articles/10.3389/fbioe.2022.894731/full · publisher NLM XML: `.../894731/xml/nlm` · mirror: https://pmc.ncbi.nlm.nih.gov/articles/PMC9265906/ | 2026-08-03 | **READ** — full text end to end (Intro, Methods §2.1-2.7, Results §3.1-3.4, Discussion, Limitations, Conclusion). **Table 3 verified digit-for-digit across two independent renderings** (Frontiers NLM XML and PMC HTML), and internally re-derived from the kg masses (see below). | **THE source for GAP 2.** First published measured three-way foot inertia split. Full extraction below. |
| **BR-05** | **P** | **Zhu, Y. & Jenkyn, T. (2023) "Development of a clinically useful multi-segment kinetic foot model." *J Foot Ankle Res* 16:83. doi:10.1186/s13047-023-00686-0. CC BY. n=10 healthy adults.** | https://pmc.ncbi.nlm.nih.gov/articles/PMC10685473/ | 2026-08-03 | **READ** (methods + results; kinematics figures not needed) | Independent 2023 confirmation of the *method* our spec used: foot segment masses apportioned by **volume** from Drillis et al., **"The centers of mass were assumed to be halfway along the segment long axis"**, and **"The radii of gyrations of foot segments were determined according to De Leva."** Peer-reviewed precedent for exactly our `com_frac 0.500` + de Leva-anchored `rg` choice. Its "hallux" == Drillis's "five toes". |
| **BR-06** | **P** | **Veeger, D.H.E.J. & van der Helm, F.C.T. (2024) "Anatomical parameters for modeling of the human shoulder, anthropometric data collected in dissection studies" [dataset]. 4TU.ResearchData. **CC0**. The primary data behind Veeger et al. (1991) *J Biomech* 24(7):615-629 and van der Helm et al. (1992) *J Biomech* 25(2):129-144.** | https://data.4tu.nl/datasets/61a954ad-43ff-4736-996f-b2dc23683d8f · doi:10.4121/61a954ad-43ff-4736-996f-b2dc23683d8f.v1 | 2026-08-03 | **READ** — downloaded `VU.zip`, read `VUstudy_inertia.htm` (full inertial data table, all 7 specimens) and `VUstudy_intro.htm` (complete methods) as raw files, not summaries. | **THE decisive negative result for GAP 1.** Full extraction below. |
| **BR-07** | S | **Delft Shoulder & Elbow Model anatomical parameter file `l1091_2024.dsp`** (cadaver l1091r, Leiden; measured by Spoor, Klein Breteler & Minekus; file maintained by Chadwick / Veeger, last edit 2024-01-29). Shipped inside `Leiden.zip` of BR-06, CC0. | (as BR-06) | 2026-08-03 | **READ** (all 2035 lines; inertial block lines 1990-2033 read verbatim) | The only explicit clavicle/scapula segment masses + inertia tensors found anywhere. **Self-declared as rough.** Full extraction below. Tier S (a model artifact, not a measurement). |
| BR-08 | P | Drillis, R., Contini, R. & Bluestein, M. (1964) "Body Segment Parameters: A Survey of Measurement Techniques." *Artificial Limbs* 8(1):44-66. — the work BR-05 apportions foot volume from. | http://www.oandplibrary.org/al/pdf/1964_01_044.pdf | 2026-08-03 | **PARTIAL: body text READ, numeric tables UNREACHABLE.** The PDF's table/figure pages (6, 11, 12, 16, 17) carry 38-910 chars vs 2600-3900 on prose pages — the tables are **page images with no text layer**, and this environment has no OCR. | Confirmed the paper is a *survey*, confirmed Meeh (1884, n=10 living, 8M/2F, ages 12-56) as the volumetric-subdivision lineage. **Did NOT yield the "base of foot / middle foot / toes" volume fractions.** |
| BR-09 | P | Klein Breteler, M. (1996) internal dissection report, Leiden (the work behind Klein Breteler, Spoor & van der Helm (1999) *J Biomech* 32:1191-1197). Shipped inside `Leiden.zip` of BR-06. | (as BR-06) | 2026-08-03 | **UNREACHABLE.** 95-page PDF, **zero-length text layer on every page** — a pure scan. No OCR in this environment (same blocker as the NASA scan in `mech-entry/`). | Nothing. |
| BR-10 | S | Nikolova, G. & Toshev, Y. "Comparison of two approaches for calculation of the geometric characteristics..." *Acta Bioeng Biomech*. 16-segment geometric model. | https://actabio.pwr.edu.pl/d/.../1.pdf | 2026-08-03 | **READ** (6 pp) | Corroborates the estimation-method landscape: the named geometric-modelling lineage is **Hanavan (1964) / Jensen / Hatze (1980) / Yeadon (1990)**. Its own foot is a **single frustum of a cone**; it has **no shoulder segment**. |
| BR-11 | X | Web-search summaries of Winter Table 4.1's shoulder row; of Hatze (1980)'s segment list; of Dumas et al. (2007)'s segment list; of Chandler et al. (1975)'s 14 segments. | various | 2026-08-03 | **SNIPPET-ONLY — DOES NOT COUNT.** Two summaries of the *same* Winter row disagreed with each other (one said mass = 0.0324, the other said "mass = 0.712", which is self-evidently the CoM column). | Used **only** as negative-space navigation. **No number from BR-11 is carried into the spec.** |
| BR-12 | P | Hatze, H. (1980) "A mathematical model for the computational determination of parameter values of anthropomorphic segments." *J Biomech* 13:833-843. **17 segments — the only whole-body BSP model that contains explicit left/right shoulder segments.** | doi:10.1016/0021-9290(80)90171-2 | 2026-08-03 | **PAYWALLED** (Elsevier; no OA mirror, no institutional repository copy, no author page found). | Nothing directly. Named below as the best-available estimation method for GAP 1 precisely *because* it is the one model that segments the shoulder. **This is the single highest-value unread source in this ledger.** |

---

## GAP 2 — THE FOREFOOT / TOE: **measured data EXISTS.** (BR-04)

### The dispatch's prediction was right about the pre-2022 literature and wrong about 2022+

BR-04's own Introduction, verbatim, on the state of the art it was written to fix:

> "In previous studies, the mass and inertial tensor of each foot segment were
> determined **arbitrarily** (Dixon et al., 2012) or **calculated by assuming a
> mathematical model such as a cylinder** (Bruening et al., 2012; Saraswat et al.,
> 2014; Kevin et al., 2017); however, such an assumption is unreasonable as the
> radius and height of each foot segment are not uniform."

So: Bruening et al. (2012), Saraswat et al. (2014) and Dixon et al. (2012) — the
canonical kinetic multi-segment foot models — **do not contain measured toe
inertia.** BR-05 (2023) independently confirms the pattern from the other side:
it apportions mass by *volume* and *assumes* CoM at segment mid-length.

**But BR-04 itself closes the gap.** Its closing claim, verbatim:

> "Our study provided inertial parameters of the phalanx, forefoot, and hind foot
> segments based on the CT scan data of the foot... Thus, the present dataset
> should serve as a useful reference for inertial parameters of the kinetic
> multi-segment foot model."

### BR-04 Table 3 — verbatim, verified in two independent renderings

| Foot segment | Relative segment mass, % | Relative COM position, % | Ixx | Iyx | Iyy | Izx | Izy | Izz |
|---|---|---|---|---|---|---|---|---|
| Phalanx  | **14.4** | 43.6 | 2.55e-3 | −0.428e-3 | 1.43e-3 | −0.220e-3 | −0.0507e-3 | 3.38e-3 |
| Forefoot | **42.4** | 41.9 | 1.40e-3 | −0.000748e-3 | 1.73e-3 | −0.0510e-3 | −0.117e-3 | 2.20e-3 |
| Hindfoot | **43.2** | 55.4 | 1.54e-3 | 0.00986e-3 | 1.84e-3 | −0.195e-3 | −0.155e-3 | 1.48e-3 |

Inertia tensor unit is the paper's own: `I` normalised by segment mass^(5/3)
("arbitrary unit" — dimensionally `m²·kg^(−2/3)`, constant under geometric
scaling). Relative COM is **% of segment length from the segment's PROXIMAL
JOINT CENTRE**.

Mean absolute masses, n=10 (BR-04 §3.1): phalanx **0.131 ± 0.017 kg**, forefoot
**0.386 ± 0.051 kg**, hindfoot **0.393 ± 0.052 kg**.

**Internal consistency check (run by Toto, passes):** 0.131 + 0.386 + 0.393 =
0.910 kg. 0.910 / 62.8 kg mean body mass = **0.01449**, which reproduces BR-04
§2.5's stated scaling "the total mass of each individual's foot was estimated to
be **0.0145** times the body weight (Winter, 1990)" to 4 decimal places. And
0.131/0.910 = 14.40 %, 0.386/0.910 = 42.42 %, 0.393/0.910 = 43.19 % — the kg
masses and Table 3's percentages are mutually consistent. Two different parts of
the paper agree; the table is not a transcription artifact.

### Conditions attached to every number above (R4)

- **Relative masses / COMs / inertia tensors: n = 1.** BR-04 §2.6, verbatim:
  *"We estimated the relative mass of the segment and the inertia tensor of each
  segment from the CT data of a single male participant."* That participant
  (§2.1): **age 42, 72 kg, 172 cm**, CT, no foot deformity.
- **The 10 participants (age 23.9 ± 3.0 y, 171.8 ± 5.1 cm, 62.8 ± 8.2 kg, all
  male, no foot/lower-limb deformity) only supplied body mass for scaling.** The
  ± values on the kg masses are **body-mass spread, not anatomical spread in the
  split.**
- **Volumetric, not gravimetric.** §2.2: inertia computed in Autodesk Inventor
  Professional from the CT surface, *"assuming a homogeneous segment composition
  and a density of 1.1 g/cm³ (Winter, 1990)."* Uniform density is assumed, not
  measured.
- **Segmentation planes** (§2.2): ANKL–ANKM (ankle), TN–VMB (midtarsal),
  FMH–VMH (MTP). Hindfoot = calcaneus + talus + cuboid + navicular; forefoot =
  cuneiforms + metatarsals; phalanx = all five phalanges.

### Mapping onto our rig

Our `Foot` (ankle → MTP) == BR-04 **hindfoot + forefoot**. Our `Toe` (MTP → tip)
== BR-04 **phalanx**.

⇒ **toe = 14.4 % of whole-foot mass; foot = 85.6 %** (MEASURED, volumetric, n=1).

### The load-bearing result: BR-04 independently validates the spec's *assumed* MTP break

Spec §2.3 solves the split from de Leva's measured whole-foot CoM using an
**explicitly unsourced** metatarsophalangeal break fraction `s = 0.72` of foot
length from the heel ("*Not derivable, art-directed and labelled as such*").
That model, with each part uniform, reduces to a one-line relation:

```
m_h·(s/2) + m_t·((1+s)/2) = 0.4415 ,  m_h + m_t = 1
  ⇒  s/2 + m_t/2 = 0.4415
  ⇒  m_t = 2(0.4415) − s = 0.883 − s
```

- spec's assumption `s = 0.72`  ⇒ `m_t = 0.163`  (the shipping value ✓)
- BR-04's **measured** `m_t = 0.144` ⇒ **`s = 0.739`**

**Two fully independent routes — (a) an n=100 gamma-ray-scan whole-foot CoM plus
a geometric split assumption, and (b) an n=1 CT volumetric segmentation — agree
on the MTP break location to within 0.019 of foot length (≈5 mm on a 26 cm
foot), and on toe mass to within 13 % relative.** The spec's "art-directed"
label on `s = 0.72` can be upgraded to DERIVED-and-corroborated.

### Toe radius of gyration — a real cross-check on `rg_frac = 0.2887`

For a toe hinging at the MTP, the relevant axis is BR-04's **y** (the axis
through FMH–VMH, i.e. medio-lateral — the plantarflexion axis). `Iyy_rel` =
1.43e-3, phalanx mass 0.131 kg:

```
I_yy = 1.43e-3 × 0.131^(5/3) = 1.43e-3 × 0.033788 = 4.832e-5 kg·m²
r    = sqrt(4.832e-5 / 0.131) = 0.01920 m  =  19.2 mm
```

⚠ **Unit inference, labelled as such:** BR-04 calls the tensor unit "arbitrary".
The conversion above assumes the CAD package worked in **kg and m**. That is the
only dimensionally natural reading, but the paper does not state it. Treat
19.2 mm as *strongly indicated*, not *published*.

Against our rig's toe length (`len_frac 0.0415 × BODY_HEIGHT 1.78` = 73.9 mm):
`r/L = 0.260` vs the spec's uniform-slab **0.2887** → **−10 %**. Our assumed
value is right to within ten percent of a measured one.

### How much this can possibly matter — BR-04's own sensitivity analysis

§2.6 / §3.4: they scaled **all** foot segment masses and inertias by **0.5× and
1.5×** (chosen because *"the standard deviations of the mass and inertial
parameters were reportedly about a quarter of the respective mean values
(Zatsiorsky, 2002)"*). Result, verbatim: *"The change in the inertial parameters
had **virtually no effect** on the calculated joint moments."*

**A ±50 % error in our toe's mass and inertia is below the noise floor of foot
inverse dynamics.** The 13 % disagreement between 0.163 and 0.144 is
immaterial to anything the rig does.

### One definitional trap — add it to spec §2.1's existing three

**BR-04's "Relative COM position" column is NOT transferable to our foot rows.**
Its axis runs proximal-joint-centre → distal-joint-centre; the hindfoot's
proximal end is the **ankle joint centre (between the malleoli)**, which sits
*above and forward of* the heel. de Leva's 0.4415 and our `com_frac` are
**heel-referenced**. Hindfoot 55.4 % and forefoot 41.9 % cannot be combined into
a heel-referenced whole-foot CoM without segment lengths BR-04 does not publish.
**Take the MASSES from BR-04. Do not take the CoMs.**

---

## GAP 1 — THE CLAVICLE / SHOULDER GIRDLE: **no measured data exists.** (BR-06, BR-07)

### 1. No whole-body BSP dataset segments the clavicle

de Leva 1996 (BR-01) contains no clavicle, scapula or shoulder girdle — the word
does not appear. Winter Table 4.1 (BR-02) has one crippled row. Dempster
(BR-03) is unreachable. Chandler et al. 1975 segmented its 6 cadavers into 14
segments and Dumas et al. 2007 covers 10 (head+neck, thorax, abdomen, pelvis,
upper arm, forearm, hand, thigh, shank, foot) — **neither has a shoulder
segment** (BR-11, SNIPPET-ONLY, navigational only). BR-10, a 16-segment
geometric model, has none either. **The sole known exception among whole-body
models is Hatze 1980 (BR-12), which is PAYWALLED.**

### 2. The definitive shoulder-mechanism cadaver study did not measure clavicle inertia either

This is the finding that settles GAP 1. BR-06 is the primary dataset behind
Veeger et al. (1991) — a paper *titled* "**Inertia** and muscle contraction
parameters for musculoskeletal modelling of the **shoulder mechanism**". If a
measured clavicle inertia existed anywhere, it would be here.

**Population** (`VUstudy_inertia.htm`, read directly): n = 7 cadavers, **5 male
2 female**; age 70–90 (mean **80.00**, SD 7.02); mass 51.7–104.0 kg (mean
**76.13**, SD 16.43); stature 164.2–181.6 cm (mean **171.16**, SD 6.92). Both
shoulders of each = 14 shoulders dissected.

**The complete list of segments in that file's inertia tables:** Head · Trunk ·
Head&Trunk · Total arm · Upper arm · Forearm · Forearm&hand · Hand.
**There is no clavicle row. There is no scapula row.** The girdle is inside
"Trunk".

**And the inertias that *are* there are regressions, not measurements.**
`VUstudy_intro.htm` §Methods step 1, verbatim:

> "measurement of relevant body dimensions for the derivation of inertia
> parameters segment mass, segment volume, segment mass position and moments of
> inertia. Body dimensions were measured following the procedures and definitions
> as given by **Clauser et al (1969)** and used for the **estimation of segment
> properties on the basis of regression equations** published by those same
> authors and by **Hinrichs (1985)**."

The study digitised clavicle and scapula **geometry** exhaustively — every muscle
origin and insertion, every articular surface, palpator SD 0.96 mm per
coordinate — and then took its **inertia** from published regression equations
for segments that do not include the clavicle.

### 3. The field's flagship shoulder model uses a self-declared placeholder (BR-07)

`l1091_2024.dsp`, the Delft Shoulder & Elbow Model parameter file. Its inertial
block header, **verbatim**:

```
REM Gravitational forces, mass and rotational inertia
REM translational inertia in kg/100 !!
REM scapular mass and rotational inertia roughly estimated!!
REM rotational inertia in kg . m . cm !!
REM All inertial data still needs checking
```

The bodies (`MNODEDEF` 22–25) are thorax, **clavicle**, **scapula**, humerus.
The values:

```
REM Clavicula
XF 24 0.0 -1.56  0.0
XM 24 0.00156  0.00156  0.00156
XM  7  0.00064  -0.00067  -0.00095  0.00263  -0.00029  0.00243

REM Scapula
XF 25 0.0 -7.054 0.0
XM 25 0.007054  0.007054  0.007054
XM 10  0.1  0.0  0.0  0.1  0.0  0.1
```

**Unit decoding, verified twice against segments whose true inertia we know:**
mass = `XM` × 100 kg; rotational inertia = value × 0.01 kg·m².
- Humerus: m = 2.0519 kg, stated transverse 1.32 → 0.0132 kg·m². de Leva
  prediction `m(r·L)²` = 2.05 × (0.285 × 0.30)² = 0.0150. Agrees to 12 %.
- Forearm: m = 1.0928 kg, stated transverse 0.6117 → 0.006117 kg·m². de Leva
  prediction = 1.09 × (0.276 × 0.27)² = 0.00605. **Agrees to 1 %.**

| DSEM body | mass | centroidal inertia (kg·m²) |
|---|---|---|
| Clavicle | **0.156 kg** | Ixx 6.4e-6 (long axis) · Iyy 2.63e-5 · Izz 2.43e-5 |
| Scapula  | **0.7054 kg** | **1e-3 / 1e-3 / 1e-3 — exactly isotropic, exactly round** |

The scapula tensor is a placeholder on its face, and the file says so.

**The clavicle inertia fails its own physical sanity check.**
`r_transverse = sqrt(2.63e-5 / 0.156) = 13.0 mm`. A uniform rod has
`r = 0.2887 L`, so 13.0 mm implies a clavicle **45 mm long**. A real adult
clavicle is 140–160 mm. DSEM's clavicle inertia is ~9× too small for a
slender-rod clavicle. **Do not adopt it.**

**Body-mass normalisation (INFERRED by Toto — the file does not state cadaver
mass).** Total arm = 2.0519 + 1.0928 + 0.52466 = 3.669 kg. Divided by de Leva-M
total-arm 0.0494 → 74.3 kg; divided by BR-06's own cadaver mean 4.75 % → 77.2 kg.
Take **≈75 kg**, ±4 %:

- clavicle alone ≈ **0.0021** of body mass
- scapula alone ≈ **0.0094**
- **clavicle + scapula (the functional shoulder girdle) ≈ 0.0115 per side**

Our rig's `Clavicle` segment is functionally the whole girdle (it parents
`UpperArm`), so `0.0115` is its DSEM analogue and `0.0021` is the bone-only
lower bound. **The spec's assumed 0.0050 sits inside that bracket.**

---

## CONTRADICTIONS, recorded as contradictions (R7)

**CONTRADICTION-1 — Winter Table 4.1's shoulder-row mass cell.** Spec §2.2, from
this project's primary read, states the cell is **blank**. Two web-search
summaries encountered on 2026-08-03 disagreed — one asserted **0.0324**, the
other asserted the mass "is 0.712" (self-evidently the CoM column bleeding
across). **Both are SNIPPET-ONLY and mutually inconsistent; both are discarded.**
No number is carried. Not averaged, not resolved. **To close it: re-read Winter
Table 4.1 from the physical book.** Until then the spec's primary read stands and
the clavicle mass stays ASSUMED regardless — 0.0324 would be 6× the assumed
value and is not credible for a clavicle on its face.

**CONTRADICTION-2 — internal to the spec, found while auditing.** Clavicle length
is **0.109 H** in §2.1 (→ 0.109 × 1.78 = **0.1940 m**) but **0.1898 m** in §3.3's
inertia arithmetic (→ 0.1066 H). A 2.3 % inconsistency. Numerically immaterial,
but the two should be reconciled to one number. *(This resolves the dispatch's
open question "length 0.1898 of… something": 0.1898 is **metres**, not a
fraction of stature.)*

**CONTRADICTION-3 — arithmetic slip in the spec's own sensitivity bound.** §2.2
states `I_clavicle ≈ 0.005·(0.2887·0.109·1.78)²·M ≈ 1.4e-4·M`. Recomputing:
`(0.2887 × 0.109 × 1.78)² = 0.0031374`, × 0.005 = **1.57e-5·M** — and §3.3
independently gets **1.50e-5**. §2.2 is **10× too large**. The conclusion is
unaffected and in fact strengthened: the clavicle is *more* negligible than
claimed.

---

## VERDICTS FOR THE IMPLEMENTER

### CLAVICLE — **KEEP the placeholders. Relabel them ASSUMED, not DERIVED.**

No measured clavicle inertia exists in any source reachable from here, and the
one dataset that should have contained it (BR-06) demonstrably does not.

| Field | Value | Verdict | Label |
|---|---|---|---|
| `mass_frac` | **0.0050** — keep | Inside the DSEM bracket [0.0021 bone-only, 0.0115 girdle]. No measured value exists to replace it with. Changing it would force a re-carve of `Thorax` and a recompute of its CoM for no evidential gain. | **ASSUMED** (spec currently implies DERIVED — downgrade it) |
| `com_frac` | **0.712** — keep | The only published clavicle number in existence (BR-02). Itself a Dempster derivative, and subject to CONTRADICTION-1. | **PUBLISHED (secondary — Dempster-derived)** |
| `rg_frac` | **0.2887** — keep | Uniform slender rod, `1/√12`. Do **not** adopt DSEM's 13.0 mm — it fails its own sanity check and its file says "still needs checking". | **ASSUMED (geometric model)** |
| `len_frac` | **0.109 H** — keep, but fix CONTRADICTION-2 | Drillis & Contini half-biacromial minus sternoclavicular offset. | **DERIVED** |
| longitudinal `rg` floor 0.10 | keep | Non-singularity device. | **ASSUMED** |

**Best available estimation method, named:** **Hatze (1980), BR-12** — the only
whole-body BSP model containing explicit left/right shoulder segments, built from
subdivided geometric solids with per-solid densities. Failing access to it, the
standard fallback is **geometric-solid modelling in the Hanavan (1964) /
Yeadon (1990) lineage** — which is exactly what `rg_frac = 1/√12` already is. Our
method is the correct one; only the label is wrong.

**Precision ceiling:** there is none, because there is no measurement. Treat the
clavicle row as a **structural placeholder chosen for the right order of
magnitude**, bounded by DSEM's [0.0021, 0.0115]. Per CONTRADICTION-3 the
clavicle's inertia is ~1.5e-5·M, four orders below the thorax. **Never build a
feature whose feel depends on the clavicle's mass.**

### TOE / FOREFOOT — **KEEP the derived split. It is now independently corroborated.**

| Field | Shipping value | Measured alternative (BR-04) | Verdict |
|---|---|---|---|
| toe mass, fraction of foot | 0.163 | **0.144** | **KEEP 0.163.** It is anchored to the same n=100 dataset as every other row and reproduces de Leva's measured whole-foot CoM *exactly by construction* — which the spec already ships as an assertion. Switching would break that test for a 13 % change that BR-04's own sensitivity analysis shows is invisible. |
| `Foot.mass_frac` (of body mass) | 0.011467 | 0.856 × 0.0137 = **0.011727** | KEEP |
| `Toe.mass_frac` (of body mass) | 0.002233 | 0.144 × 0.0137 = **0.001973** | KEEP |
| MTP break `s` | 0.72 | implies **0.739** | **KEEP 0.72; upgrade its label** from "art-directed, no source" to **DERIVED, corroborated to within 0.019 L** |
| `Toe.rg_frac` | 0.2887 | implies **0.260** | KEEP (agrees within 10 %) |
| `Toe.com_frac` / `Foot.com_frac` | 0.500 | not transferable (different axis — see the trap above) | KEEP, **ASSUMED**; BR-05 is peer-reviewed precedent for the identical assumption |
| `TOE_OFF_RAD = 0.45` | — | no source | **ASSUMED**, unchanged — this is ROM, not inertia; nothing found |

**If the implementer prefers measured-over-derived**, the switch is
`0.163 → 0.144` in the two mass constants **plus** `s: 0.72 → 0.739` in the
assertion, so that `derived_foot_split_reproduces_de_leva_whole_foot` still
passes. Do not change one without the other — `m_t = 0.883 − s` is the invariant.

**Precision ceiling for BR-04:** relative masses and the inertia tensor come from
**a single CT scan of one 42-year-old male (72 kg, 172 cm)** at an **assumed
uniform density of 1.1 g/cm³**. It is a volumetric measurement, not a
gravimetric one, with **n = 1 and no published SD on the relative values**. Quote
at most 3 significant figures. **Do not build per-fighter foot-mass variance on
it** — the ±0.017/±0.051/±0.052 kg spreads are body-mass scaling, not anatomy.

### Labels to set on `SegmentDef.measured`

Spec Step 0's test `derived_rows_are_flagged_as_derived` asserts the not-measured
set is exactly `{ClavicleL, ClavicleR, FootL, FootR, ToeL, ToeR}`. **That set is
still correct and should not change.** But the *narrative* label differs by row
and belongs in the doc comment:

- `Clavicle{L,R}` → **ASSUMED** (no measurement exists anywhere; geometric model)
- `Foot{L,R}` / `Toe{L,R}` → **DERIVED from BR-01, corroborated by BR-04**

Consider a third state or a comment field, so a future reader can tell "solved
from a measurement" apart from "invented because nothing exists". They are not
the same kind of number and the spec currently spells both `false`.
