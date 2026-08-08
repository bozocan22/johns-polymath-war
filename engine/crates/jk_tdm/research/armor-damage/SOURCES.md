# armor-damage — SOURCES

Dispatch (2026-08-08): a builder is implementing four-stage armour damage
**Fresh → Scuffed → Cracked → Severed** across 24 steel-analogue plates on
human-scale soldiers, struck by rifle rounds, arrows, spears and fragments.
Four questions: (1) is a four-stage model defensible and do the stages map
onto real failure modes? (2) how much protective value survives at each
stage? (3) does impact LOCATION compound damage? (4) what differs between
penetrating / blunt / edged damage?

**Standing rule inherited from `aiming/SOURCES.md`: a tool's summary is not
the source.** Every number below was read out of a file on disk — a PDF text
layer extracted locally with `pypdf` and read with Read/Grep. Where a number
appears in a web-search summary but I did not see it in primary text, it is
recorded as NOT CARRIED and named as such.

---

## 1. Source ledger

| ID | Tier | Source | Route that worked | Status | What it gave |
|---|---|---|---|---|---|
| AD-01 | P | **NIJ Standard-0101.06**, *Ballistic Resistance of Body Armor*, NIJ/NCJ 223054, 2008, 89 pp | `ojp.gov/pdffiles1/nij/223054.pdf` direct | **READ** — §2, §3 (defs 3.15/3.33/3.34/3.42/3.43), §7.6.1–7.6.2, §7.8.5.1 and Tables 4/5/6/7 read verbatim from extracted text; conditioning/tumbling/submersion sections SKIMMED | The regulator's own pass/fail model; shot-to-shot and shot-to-edge minimum distances; BFS limit; how many hits a plate must survive by class |
| AD-02 | P | **Qiang, Zhang, Zhao, Ren, Ni, Zhao & Lu (2022)**, "Multiple ballistic impacts of thin metallic plates: numerical simulation", *Proc IMechE Part C: J Mech Eng Sci*, DOI 10.1177/09544062221080139 | Author accepted manuscript hosted by NUAA MLMS lab (`mlms.nuaa.edu.cn`) | **READ** — all 40 pages of extracted text | The only quantitative same-location repeat-hit degradation curve I found; impact-offset thresholds; nose-shape ordering **and impact-order asymmetry** |
| AD-03 | P | **Børvik, Langseth, Hopperstad & Malo (2002)**, "Perforation of 12 mm thick steel plates by 20 mm diameter projectiles with flat, hemispherical and conical noses. Part I: Experimental study", *Int J Impact Engineering* 27:19–35 | Third-party mirror `aux.ciar.org` (see licence note §8) | **READ** — all 17 pages of extracted text, end to end | The measured blunt/round/pointed comparison, with failure modes; Woodward's thickness criterion; a non-perforating hit that left a through-thickness crack |
| AD-04 | P | **Göde, Teoman, Çetin, Tonbul, Davut & Kuşhan (2023)**, "An experimental study on the ballistic performance of ultra-high hardness armor steel (Armox 600T) against 7.62 mm × 51 M61 AP projectile in the multi-hit condition", *Eng Sci Tech Int J* 38:101337, gold OA, CC BY-NC-ND 4.0 | İYTE DSpace REST API — `/server/api/core/items/{uuid}/bundles` → bitstream content URL (see §9) | **READ** — all 9 pages | The multi-hit experiment on real armour steel; the grade→failure-mode map; crater metrology; **the negative result that separated hits do not accumulate** |
| AD-05 | P | **Seidl, Lehmann & Grobert**, "An experimental investigation into the threat posed by arrows to body armour", Int. Symposium on Ballistics proceedings, DOI 10.52202/080042-0046 | `proceedings.com` open PDF | **READ** — all 10 pages | Arrows/bolts vs modern body armour, measured; tip-geometry effect at constant mass and velocity |
| AD-06 | — | **Alan Williams, *The Knight and the Blast Furnace*** (Brill, 2003) | Goodreads / AbeBooks / Amazon / Google Books *about* pages; ResearchGate "Request PDF" stub | **UNREACHABLE** — commercial Brill title, no open copy | **Nothing.** See §7 |
| AD-07 | — | Forsom & Smith (2017), *J Archaeol Sci: Reports* 11:274–286 | Bournemouth eprints, downloaded and opened | **OFF-TOPIC, not counted** — it fires medieval arrowheads into *cattle scapulae* to classify bone trauma. No armour. Recorded so the next Toto does not re-chase it |
| AD-08 | — | Ceramic multi-hit damage-zone taxonomy (pulverised conoid / radial+circumferential cracks / spall / adjacent-fragment impact) | web-search summary of a patent family only | **SNIPPET-ONLY — NOT CARRIED.** Plausible and probably true, but I did not read the patent, and it is *ceramic*, which fails differently from steel (§4.4) |
| AD-09 | — | "residual strength was only one-third after two bullet hits" (Materials 15(3):901, ceramic/UHMWPE) | web-search summary only | **SNIPPET-ONLY — NOT CARRIED.** This is the single most tempting number in the whole dispatch and I did not read the paper. Do not use it |

**Count of sources that count: 5.** No breadth quota was pursued. Three of
the five are directly load-bearing (AD-02, AD-03, AD-04); AD-01 supplies the
regulator's framing and AD-05 the pointed-weapon case.

---

## 2. Q1 — How does plate armour actually fail, and is four stages defensible?

### 2.1 The verdict up front

**A staged model is defensible, but the stage list as written is wrong in two
specific places, and the thing that drives progression is wrong.**

Real steel plate does accumulate damage — that is measured, not assumed. But
it accumulates **positionally**, not globally, and the progression is
dominated by *plastic deformation then cracking*, with **no evidence for a
"spall/pitting" stage in steel** in anything I read.

### 2.2 The failure modes that actually exist, from primary text

**AD-03 (Børvik), 12 mm Weldox 460 E, 20 mm / 197 g projectiles, HRC 53,
n = 24 gas-gun shots.** Verbatim mechanisms:

- **Blunt** → *"failure by plugging, and an almost circular plug is ejected
  from the target. This failure mode is dominated by shear banding."* The
  paper's cross-sections show *"the deformation localises in narrow shear
  bands. In these localised zones, very large strains, strain rates and
  temperatures appear, causing material damage… When the strain reaches a
  critical value, a crack starts to grow towards the rear side of the target,
  and a plug is finally formed."*
- **Hemispherical** → indentation, *"a very localised bulge and target
  thinning… regions of intense tensile strain, but no shear localisation"*,
  then necking, then *"a plug with reduced thickness and diameter is ejected."*
- **Conical** → *"No plug is seen in any of the tests for conical projectiles,
  but petals are formed on both sides of the cavity"*; *"petals are formed
  because of high circumferential tensile stresses in the bulge."*

**AD-04 (Göde), Armox 600T, 12.4 mm, 635 BHN avg (SD 20.5), n = 8 shots of
7.62×51 M61 AP at 815–829 m/s.** For an *ultra-hard* plate the mechanism is
different in kind:

> *"Because of the harder martensitic structure and extremely high deformation
> rates caused by the shootings, the material cannot plastically deform, but
> instead crack… Those cracks extend through the thickness below the
> indentation… secondary cracks propagating in lateral direction are also
> visible."*

and its own conclusion:

> *"The major fractographic tendency of Armox 600T related to the designated
> ballistic threat, is prior cracking through thickness and followed by
> secondary cracking towards lateral direction."*

AD-04's Fig. 2 gives a grade → mode map (re-created by its authors after
Crouch 1988), which I read as text in §1 of that paper:

| Armour steel grade | Hardness | Dominant mode |
|---|---|---|
| RHA (rolled homogeneous) | 241–388 BHN | plastic flow, **ductile hole formation** (Mode A) |
| HH (high hardness) | 477–534 BHN | **adiabatic shear bands → soft plugging** (Mode B); AD-04 notes performance *"could be reduced to some extent when plugging occurs… premature failure due to formation of ASB"* |
| UHH (ultra-high hardness) | ~600 BHN+ | **projectile is shattered**; armour shows *"brittle fracture mode which mostly consists of quick formation and accelerated propagation of micro-cracks"* |

### 2.3 Does the game's stage list map onto these? Stage by stage

| Game stage | Real analogue | Verdict |
|---|---|---|
| **Fresh** | undamaged | fine |
| **Scuffed** (surface deformation) | **SUPPORTED, MEASURED.** AD-03: permanent global dishing of a 12 mm plate, **0.62 – 4.10 mm** depending on nose and velocity, excluding the local bulge. AD-04: laser-scanned crater on a plate that *defeated* the round — **max depth 2.57 mm, surface area 225.48 mm², volume 82.87 mm³** at 15 µm scanner resolution | keep — but see the hardness caveat below |
| **Cracked** (through-crack) | **SUPPORTED, and by two independent sources on plates that did NOT fail.** AD-03 test H2 (278.9 m/s, below the 292.1 m/s limit, no perforation): *"a through-thickness-crack was found almost completely around the circumference of the bulge."* AD-02 reproduces the same thing: *"circumferential cracking after significant bulging when the clamped plate was impacted at just below V_L."* AD-04: through-thickness cracks under the indentation on all-defeated shots | keep — this is the best-evidenced stage in the whole model |
| **spall / pitting** (the implied stage-2 mechanism in the dispatch) | **NOT SUPPORTED for steel.** Zero mentions of spalling as a *target* failure mode in AD-02, AD-03 or AD-04. In AD-04 it is the **projectile** that shatters, not the plate. Spall/pulverisation as a damage zone is a **ceramic** phenomenon (AD-08 — and I did not read that one) | **drop it, or rename the stage after deformation, which is what actually happens** |
| **Severed** (plate detaches) | **PARTIALLY supported, and not in the sense the design means.** What detaches in the literature is a **plug** — a disc roughly the projectile's diameter punched out of the plate (AD-03: cavity diameters 19.3–23.8 mm for a 20 mm projectile; plug masses 15.1–36.0 g) — or **petals** bent open around a hole. **Nothing I read addresses a whole plate coming off a wearer.** That is a strap/mount failure, not a ballistic one, and it is unsourced | keep if you want it, but label it a *mounting* failure; the physics literature does not reach it |

### 2.4 The part that changes the design: hardness decides whether "Scuffed" exists

AD-03's Weldox 460 E (a structural steel) deforms plastically and dishes
measurably before it fails. AD-04's Armox 600T *"cannot plastically deform,
but instead crack."* Same threat class, opposite intermediate state.

**So "Fresh → Scuffed → Cracked" is a soft/medium-steel story.** A genuinely
hard plate goes **Fresh → Cracked → Shattered** and never looks scuffed. If
the 24 plates are meant to read as hardened armour, stage 2 is the wrong
picture; if they read as thick wrought plate, it is right. That is an art and
fiction decision the literature can inform but not settle.

### 2.5 The honest counterweight: the regulator treats armour as binary

AD-01 does not grade armour at all. Per shot there are exactly two outcomes,
and both are binary:

- **Perforation** (3.34): *"Any impact that creates a hole passing through the
  armor"* — evidenced by the projectile/fragment in the clay, a hole through
  armour and/or backing, or *"any portion of the bullet being visible from the
  wear face."*
- **Backface Signature (BFS)**: the depression left in the clay backing.
  Limit **44 mm (1.73 in)** for **every** armour type in Table 4 — IIA, II,
  IIIA, III and IV alike. Measured with a device of *"1 mm (0.04 in) or better
  accuracy"*; readings over **40 mm** must be verified by a second measurement.

There is no "degraded plate" grade anywhere in the standard. **A real plate is
scored pass/fail per hit and its history is not tracked.** The four-stage
model is therefore a *game* abstraction over a real but sub-threshold physical
process — which is a legitimate thing to build, as long as nobody claims the
standard supports it. It does not; it is silent on it.

---

## 3. Q2 — How much protective value survives at each stage?

### 3.1 The one quantitative degradation curve I found (AD-02)

Conditions, stated because they are part of the number: **fully-clamped 304
stainless steel disc, 100 mm free diameter, 0.71 mm thick**; rigid **spherical
projectile, 12.7 mm diameter, 8.3 g**; LS-DYNA FE with Johnson-Cook +
Cowper-Symonds, damage by equivalent plastic strain; **all repeat hits at the
identical location**; interval 1000 µs.

| Quantity | Experiment (AD-02's ref. 23, quoted in its Table 2) | AD-02's FE | Prior FE (its ref. 24) |
|---|---|---|---|
| `V_L` single-impact ballistic limit | **205.1 m/s** | 206.3 (+0.6 %) | 209.9 (+2.3 %) |
| `V_2L` equivelocity limit, 2 hits same spot | **127.2 m/s** | 121.3 (−4.6 %) | 133.6 (+5.0 %) |
| `V_3L` equivelocity limit, 3 hits same spot | *not measured* | **≈ 95 m/s** | — |

`V_3L` is **FE only — there is no experimental value for three hits.** Say so
before anyone builds on it.

### 3.2 The arithmetic, shown in full

Energy scales as `v²` at fixed projectile mass, so limit-velocity ratios
squared give energy-capacity ratios directly.

**Framing A — per-hit capacity remaining, using AD-02's own FE triple so all
three points come from one model:**

```
(V_2L / V_L)² = (121.3 / 206.3)² = 0.587979² = 0.345719
(V_3L / V_L)² = ( 95.0 / 206.3)² = 0.460495² = 0.212055

capacity after 0 / 1 / 2 prior co-located hits :  1.000  →  0.346  →  0.212
marginal loss, hit 1 :  1.000 − 0.346 = 0.654
marginal loss, hit 2 :  0.346 − 0.212 = 0.134
ratio of the two     :  0.654 / 0.134 = 4.89
```

**The first co-located hit costs ~4.9× as much protective capacity as the
second.** Using the *experimental* pair instead of the FE pair,
`(127.2/205.1)² = 0.620185² = 0.384630`, i.e. 0.385 rather than 0.346 — same
shape, 11 % apart in level.

**Framing B — total energy needed to defeat the plate over n co-located hits,
as a fraction of the single-hit perforation energy:**

```
n = 1 :  1 × 1.000000 = 1.000
n = 2 :  2 × 0.345719 = 0.691
n = 3 :  3 × 0.212055 = 0.636          (experiment, n=2: 2 × 0.384630 = 0.769)
```

Splitting the same total energy across repeated hits **on the same spot** is
*more* efficient at defeating the plate than delivering it in one blow.

Absolute energies at those limits (m = 8.3 g), for scale:
`0.5 × 0.0083 × 206.3² = 176.6 J` / `121.3² → 61.1 J` / `95.0² → 37.5 J`.

### 3.3 What shape this is, and what it is not

**It is concave and strongly front-loaded. It is not linear.** AD-02's authors
go further and expect it to flatten out entirely — invoking *"the
pseudo-shakedown phenomenon"* they write:

> *"it can be inferred that the equivelocity ballistic limit velocity might
> tend to a constant as the number of impacts is increased."*

Note "inferred" — **that is the authors' extrapolation, not their data.**
Labelled ASSUMED below, not MEASURED.

### 3.4 The caveat that governs everything in §3

**These numbers are for hits at the *identical location*.** At separated
locations the degradation is not merely smaller — AD-04 could not measure it
at all (§4.2). **So "how much survives at each stage" has no answer
independent of where the hits landed.** Any table of per-stage multipliers
that does not carry a spacing condition is over-claiming.

### 3.5 Labels

| Value | Label |
|---|---|
| `V_L = 205.1`, `V_2L = 127.2` m/s | **MEASURED** (experiment quoted in AD-02 Table 2; that experiment is AD-02's ref. 23, which I did **not** read directly — it reaches me through AD-02's table. Second-hand from a primary table, not from the originating paper) |
| `V_L = 206.3`, `V_2L = 121.3`, `V_3L ≈ 95` m/s | **MODELLED** (validated FE, <5 % vs experiment on the first two) |
| capacity 1.000 → 0.346 → 0.212; 4.89× front-loading | **DERIVED** (arithmetic above, from the FE triple) |
| flattening toward a constant beyond 3 hits | **ASSUMED** (authors' own inference) |
| any value for a *4th* hit | **DOES NOT EXIST.** The game has four stages; the literature stops at three |

---

## 4. Q3 — Does impact LOCATION matter? Yes. This is the headline.

### 4.1 The direct measurement of the interaction radius (AD-02)

Projectile diameter **D = 12.7 mm**. Second projectile fired at offsets
`d = 0, 12.7, 25 mm` = **0 D, 1.00 D, 1.97 D**. Verbatim:

> *"Increasing the V_I caused obvious shifting of the residual velocity curve
> toward the left if d = 12.7 mm, thus reduced ballistic resistance. However,
> if the offset was increased to d = 25 mm, the ballistic resistance was almost
> not affected… This was understandable, as in the former case, 12.7 mm was
> also the diameter of the spherical projectile, so that the damage caused by
> the second projectile complemented that caused by the first projectile, thus
> leading to severe damage and inferior ballistic resistance. In contrast, in
> the case of d = 25 mm, the damage zones of the two separate impacts did not
> overlap… so that the offset had little influence on ballistic resistance."*

And its conclusion (i): *"the larger the impact position offset of projectile
II, the higher the ballistic limit velocity."*

**Interaction radius ≈ 1 projectile diameter. At ≈ 2 diameters the damage
zones no longer overlap and hits are effectively independent.**

### 4.2 The independent corroboration, on real armour steel (AD-04)

Eight 7.62×51 M61 AP rounds at 815–829 m/s into one 200 × 200 mm, 12.4 mm
Armox 600T plate. All eight defeated ("Partial Penetration"). Verbatim:

> *"The multi hit shootings did not influence the main ballistic resistance
> since the max depth of the craters is very similar."*

AD-04 **does not state the shot spacing.** From plate area and shot count:
`40000 mm² / 8 = 5000 mm² per shot`, `√5000 = 70.7 mm ≈ 9.0 projectile
diameters` — **DERIVED, and it ASSUMES an even spread over the full plate,
which I could not verify** (the layout is only visible in the paper's Fig. 10b,
which is an image). Treat it as an order-of-magnitude bound: the shots were
many diameters apart, and at that separation eight hits produced no measurable
degradation.

### 4.3 The regulator's own thresholds (AD-01)

Verbatim from §7.6:

> *"For armor types subjected to a single threat and for the lighter weight
> threat round when two threats are specified, the minimum shot-to-edge
> distance shall not be greater than 51 mm (2.0 in). For the heavier threat
> round when two threats are specified, the minimum shot-to-edge distance shall
> not be greater than 76 mm (3.0 in)."*
>
> *"7.6.2 Minimum Shot-to-Shot Distance — The minimum shot-to-shot distance
> shall be 51 mm (2.0 in)."*

with the definitions (3.42, 3.43) that shot-to-edge is measured from bullet
centre to the nearest panel edge and shot-to-shot from bullet centre to the
centre of the nearest prior impact.

These are **regulatory thresholds, not measurements** — label them
**STANDARD-MANDATED**. But their existence is itself evidence: the standard
body declines to certify hits closer than 51 mm together, and declines to
count hits closer than 51–76 mm to an edge. 51 mm / 7.62 mm ≈ **6.7 projectile
diameters** — far more conservative than AD-02's ~2 D, which is exactly what
you would expect of a safety margin.

### 4.4 Edges are weak — a second, free positional effect

Two independent sources, same direction:

- AD-02, on comparing single impacts at different positions: *"It was shown
  that the target was easier to be perforated closer to its boundary."*
- AD-01 sets a **minimum shot-to-edge distance** at all, and a larger one
  (76 mm) for the heavier threat — the standard will not let you score a hit
  near the edge.

### 4.5 Verdict for the design

**Per-piece HP is not sufficient, and the fix is cheap.** The three sources
agree in direction and roughly in scale:

| Separation | Effect | Source |
|---|---|---|
| 0 D (same spot) | severe: capacity 1.000 → 0.346 → 0.212 | AD-02 (FE) |
| ≈ 1 D | *"severe damage and inferior ballistic resistance"* — damage zones complement | AD-02 (FE) |
| ≈ 2 D | *"almost not affected"* — zones do not overlap | AD-02 (FE) |
| ≈ 9 D (derived) | no measurable change over 8 hits | AD-04 (experiment) |
| ≥ 6.7 D | mandated as the minimum spacing at which a hit counts | AD-01 (standard) |

A single stored "most recent significant impact point + radius ≈ 2 projectile
diameters" per plate captures nearly all of this. **A plate's total hit count
is the wrong state variable; the clustering of its hits is the right one.**

---

## 5. Q4 — Penetrating vs blunt vs edged

### 5.1 Measured, on one plate, one projectile mass (AD-03)

12 mm Weldox 460 E; projectile mass **0.197 kg**, diameter **20 mm**, HRC 53
in all cases — only the **nose shape** differs. Ballistic limits are the
average of the highest non-perforating and lowest perforating shot.

| Nose | `v_bl` (m/s) | `E_bl = ½ m v²` (J) | Failure mode of the plate |
|---|---|---|---|
| **Blunt** | **184.5** | **3 352.96** | shear plugging, adiabatic shear bands, circular plug ejected; *"limited plastic deformation… outside the localised shear zone"* |
| **Hemispherical** | **292.1** | **8 404.26** | ductile hole enlargement, severe local thinning, cup-shaped plug torn out after necking |
| **Conical** | **290.6** | **8 318.16** | petalling on both sides, **no plug**, *"the plastic deformation in the vicinity of the penetrating projectile… is considerable"* |

```
E_blunt / E_hemi = 3352.96 / 8404.26 = 0.398960   →  hemispherical needs 2.5065× the energy
E_conical / E_hemi = 8318.16 / 8404.26 = 0.989756  →  pointed ≈ round, within 1 %
```

**On this plate a blunt impactor defeats the armour with 39.9 % of the energy
a round-nosed one needs.** That single factor of 2.51 is the strongest reason
in this ledger not to treat the three damage types alike.

### 5.2 Pointed weapons spread damage; blunt weapons concentrate it

AD-03, on permanent target deformation (`w_max`, global dishing, *excluding*
the local bulge):

> *"Conical projectiles are found to give the largest global target deformation
> at all velocities. At an impact velocity of 300 m/s, the maximum deformation
> in the target is approximately three times larger for hemispherical and
> conical projectiles than for blunt projectiles."*
>
> *"These measurements also indicate that a larger part of the target plate is
> activated for conical projectiles than for the other two nose shapes."*

Measured `w_max` values for **non-perforating** hits (these are the "survived
but damaged" cases, i.e. the game's Scuffed/Cracked evidence):

| Test | Nose | `v_i` (m/s) | `w_max` (mm) | outcome |
|---|---|---|---|---|
| B8 | blunt | 181.5 | 2.92 | no perforation |
| B16 | blunt | 184.8 | 2.59 | no perforation |
| C3 | conical | 206.9 | 2.73 | no perforation |
| C1 | conical | 248.7 | 4.10 | no perforation |
| H4 | hemispherical | 292.1 | 3.10 | taken as the ballistic limit |
| H2 | hemispherical | 278.9 | — | no perforation, but **through-thickness crack nearly all the way round the bulge** |

**Design translation: a spear/arrow hit should mark a *wider* area of the plate
as damaged; a blunt hit should mark a *narrower* area but move it further
toward failure.** That is directly measured, not inferred.

### 5.3 Arrows defeat armour at a fraction of the bullet energy it is rated for (AD-05)

Ravin R500 crossbow, 136 kg (300 lb) draw; bolts 26–28 g; **v = 147 ± 2 m/s**,
**E = 292 ± 8 J**, impulse 3.97 N·s; 10 m; backing 20 % ballistic gelatine.
n = 1 per configuration — these are existence proofs, not rates.

- Target config 1 = **SK2** (German class certified for **9 mm hard-core**):
  aramid ICW plate + soft armour + anti-BABT layer. Test D, perpendicular:
  **perforated**, and the gelatine block fully penetrated. Test E with the
  other tip: perforated the plate, not the gelatine.
- The paper's own comparison: the bolt has *"roughly half the kinetic energy of
  a 9 × 19 mm FMJ bullet"* yet *"their rigid tip allows them to penetrate the
  protective body armour."* Checking that against its own stated reference
  round: `0.5 × 0.008 × 415² = 688.9 J`; `292 / 688.9 = 0.424`. **DERIVED —
  the bolt perforates at 42 % of the energy of the bullet the armour is
  certified to stop.**
- **Tip geometry alone, at the same mass and velocity: `PEN I` gave *"40 % less
  depth of penetration than PEN II"*.** MEASURED, n = 1 pair.
- Plate damage signature after an arrow perforation: *"relatively small material
  bulges caused a permanent deflection… No pronounced delamination was
  observed."* **An arrow makes a small local hole, not a wrecked plate.**

The mechanism the authors give — *"a consistent and high cross-sectional load
on the arrow throughout penetration"* from a sharp tip that *"maintains its
shape and does not deform upon entry"* — is a **pressure and tip-rigidity**
story, not an energy story. **Caveat: the "plate" here is an aramid ICW plate,
not steel.** Do not carry the 42 % figure to a steel plate; carry the
*direction*.

### 5.4 Impact ORDER matters, and it is asymmetric (AD-02)

Equivelocity double-impact ballistic limit, ranked from most resistant to
least:

> *"the size of equivelocity ballistic limit velocity was reduced in the
> following order: Sphere-Sphere, Flat-Flat, Sphere-Cone, Cone-Cone,
> Cone-Sphere."*

with the explanation:

> *"when the target plate was hit first by the conical projectile, it already
> experienced severe deformation such that its capability to absorb the impact
> energy of the follow-up spherical projectile was weakened. Consequently, the
> equivelocity ballistic limit velocity of Cone-Sphere double impacts was lower
> than that of Sphere-Cone."*

**Spear-then-bullet is worse for the plate than bullet-then-spear.** A cheap,
sourced, and unusual mechanic: pointed weapons *prime* a plate for whatever
lands next.

### 5.5 The contradiction I am not going to resolve by averaging

AD-02 and AD-03 **order the nose shapes oppositely**, and I am logging that as
a contradiction rather than splitting it.

| | AD-03 (experiment) | AD-02 (FE) |
|---|---|---|
| plate | 12 mm Weldox 460 E | 0.71 mm 304 SS |
| projectile D | 20 mm | 12.7 mm |
| `h/D` | **0.600** | **0.0559** |
| easiest to defeat the plate | **blunt** (184.5 vs ~291 m/s) | **conical** (*"V_L decreased… from sphere to flat and then from flat to cone"*) |

AD-03's own Introduction says the field already knew this was unsettled —
Grabarek (blunter → higher limit), Corran, Ipson & Recht, Wingrove, Othe,
Wilkins are cited as mutually *"to some extent incompatible"*, and AD-03
concludes *"the problem is not yet fully solved."* AD-02 volunteers the same
caution: *"the current result is limited to thin 304 SS plates that are prone
to plastic bulging and membrane stretching. If the target plate is thicker or
made of other metallic sheets, the order of nose shapes may be various."*

There is one candidate reconciler, quoted by AD-03 from Woodward (1984):
plugging is favoured over ductile hole enlargement when the target thickness
is below `√3·D/2`.

```
√3 × 20 / 2 = 17.321 mm    — matches AD-03's own text, "about 17 mm"  ✓ formula read correctly
√3 × 7.62 / 2 = 6.599 mm   — the equivalent threshold for a 7.62 mm projectile
```

It works for AD-03 (12 mm < 17.3 mm → plugging → blunt wins) but **does not
explain AD-02** (0.71 mm ≪ 11.0 mm, yet conical won there, not blunt). So the
criterion is not sufficient, and AD-02's very thin plate is a third regime
(membrane stretching / petalling) rather than a point on the same curve.

**Safe statement: the blunt-vs-pointed ordering is thickness-dependent and
reverses; do not hard-code a universal rule. The direction all sources agree
on is that nose/tip geometry matters a lot — a factor of ~2.5 in energy in the
one case where it was measured cleanly.** The game has effectively one plate
thickness, so it needs one consistent ordering, chosen deliberately and
labelled a design choice rather than a physical fact.

---

## 6. PRECISION CEILING — read this before building anything finer

1. **The degradation curve supports roughly three regimes, not four steps.**
   AD-02 gives `n = 1, 2, 3` co-located hits. **There is no fourth point.** A
   four-stage model has one more stage than the literature has data for, and
   the fourth transition is unavoidably invented. Say so in the code comment.
2. **The offset threshold is sampled at exactly three points: 0 D, 1.00 D,
   1.97 D.** There is nothing between 1 D and 2 D. The only defensible
   statement is "strongly interacting at ≤ 1 D, negligible at ≥ 2 D". Do not
   build a smooth falloff curve and claim it is sourced; if you want one, label
   it ASSUMED.
3. **AD-02 is a simulation.** Validated to < 5 % against experiment on `V_L`
   and `V_2L` — but `V_3L ≈ 95 m/s`, the whole third stage, is **FE only,
   never measured.**
4. **AD-03's ballistic limits are bracket midpoints.** From Tables 1–3, each
   `v_bl` rests on essentially one bracketing pair of shots (blunt: 184.3
   perforated / 184.8 did not; hemispherical: 278.9 / 292.1). n = 24 shots
   total across three nose shapes. **Resolution is of order ±5 m/s, not
   better**, and the hemispherical limit is explicitly *"assumed"* by the
   authors from an anomalous test.
5. **AD-04's "multi-hit does not matter" is one plate, one thickness, one
   velocity band (815–829 m/s), n = 8.** It is a well-controlled negative
   result, not a general law, and its shot spacing is not reported.
6. **AD-05 is n = 1 per configuration.** Eight tests, eight different
   configurations, no repeats. Existence proofs only.
7. **Scale mismatch is the largest single uncertainty in this ledger.** The
   game's plates are human-worn armour ~2–12 mm. AD-02's target is 0.71 mm
   stainless — thinner than a coin. AD-03's is a 12 mm plate hit by a 197 g
   slug, which is a fragment-simulation problem, not a rifle-round problem.
   Only AD-04 is at genuinely body-armour scale with a genuine rifle threat,
   and AD-04's headline result is a *null*. **No source read here measures
   progressive degradation of a body-armour-scale steel plate under repeated
   rifle fire. That study, if it exists, I did not find.**

---

## 7. What I could NOT answer — plainly

- **Edged / cutting damage to plate: nothing. Zero sources.** Every pointed
  source here (AD-03 conical, AD-05 bolts) is a *penetrator*, not a blade. The
  dispatch groups "spears" under edged; the literature I reached only speaks to
  points. **If the design needs a distinct cutting-vs-piercing behaviour, it is
  currently unsourced.**
- **Hand-delivered spear thrust vs plate: nothing.** Everything above is
  146–300 m/s. A thrust spear is order 5–10 m/s — a quasi-static punching
  problem, a different branch of mechanics. AD-03 cites Johnson, Ghosh & Reid's
  quasi-static punching survey but I did not read it. **Do not extrapolate the
  ballistic numbers down two orders of magnitude in velocity.**
- **Explosive fragment vs plate: not read.** AD-03 motivates its work by
  fragment impact but tests single rigid projectiles at controlled velocity.
- **Historical / medieval plate armour: no primary numbers obtained.**
  **AD-06, Alan Williams' *The Knight and the Blast Furnace*, is UNREACHABLE** —
  a commercial Brill title, no open copy; Google Books offers an *about* page
  with no preview, ResearchGate a "Request PDF" stub. The energies that
  circulate online attributed to it (mail split at 80 J, penetrating the jack
  at 100 J, failing at 120 J) reached me **only through a third-party search
  summary and are NOT CARRIED into this ledger.** Given this repo's history,
  that is exactly the shape of a number that gets fabricated.
- **Residual protection percentages for real body armour after N hits: not
  obtained for steel.** The tempting "one-third residual strength after two
  hits" (AD-09) is from a ceramic/UHMWPE compression study I did not read.
  **NOT CARRIED.**
- **Whether "cracked" and "dented" differ in residual capacity at equal hit
  count:** no source separates them. AD-02 counts hits, not states.
- **Whole-plate detachment:** no source. It is a mount/strap question.
- **Anything about the 4th stage transition.** See §6.1.

---

## 8. What this CONTRADICTS in the proposed four-stage design

| # | Design assumption | Status | Evidence |
|---|---|---|---|
| 1 | **Linear four-step degradation** | **CONTRADICTED** | Capacity goes 1.000 → 0.346 → 0.212; the first hit costs **4.89×** what the second costs. Strongly concave, and the authors expect it to flatten further (AD-02) |
| 2 | **"Spall/pitting" as a stage** | **UNSUPPORTED for steel** | Absent from AD-02, AD-03, AD-04. In AD-04 it is the *projectile* that shatters. Spall zones are a ceramic phenomenon (AD-08, unread) |
| 3 | **Per-piece HP driven by hit count** | **CONTRADICTED** | Eight hits, zero measurable degradation, when separated (AD-04). Two hits, catastrophic degradation, when co-located (AD-02). **Position, not count** |
| 4 | **All three damage types treated the same for armour wear** | **CONTRADICTED, with a number** | 2.51× energy difference between blunt and round-nose on the same plate (AD-03); 40 % penetration-depth difference from tip geometry alone (AD-05) |
| 5 | **Hit order irrelevant** | **CONTRADICTED** | Cone-Sphere is strictly worse for the plate than Sphere-Cone (AD-02 §4.3) |
| 6 | **"A plate detaches after the last stage"** | **NOT SUPPORTED, and not contradicted either — simply unaddressed** | What detaches in the literature is a plug (~projectile diameter) or petals. Whole-plate detachment is a mounting failure |
| 7 | **A dented stage exists at all** | **CONDITIONAL on hardness** | True for medium steel (AD-03, 0.62–4.10 mm dishing); false for ultra-hard steel, which *"cannot plastically deform, but instead crack"* (AD-04) |
| 8 | Stage "Cracked" | **SUPPORTED — the best-evidenced stage** | Non-perforating hits leaving through-thickness cracks in AD-03 (test H2), AD-02 and AD-04 independently |

**The single most design-relevant finding:** damage accumulation in steel
plate is **positional, not cumulative**. Two hits a plate-width apart are two
independent events with no memory between them; two hits within one projectile
diameter compound so hard that the plate's remaining capacity falls to ~35 %
after the first. A per-piece HP bar cannot express that difference, and it is
the difference that the physics is actually about.

---

## 9. Licences (R6) — read verbatim, and the shipping test

**Nothing in this ledger is a dataset, model or repository, so nothing here is
proposed for shipping.** Recorded anyway so the next Toto does not re-check:

| Source | Licence, as read | Class | Note |
|---|---|---|---|
| AD-01 NIJ 0101.06 | US Government publication, NCJ 223054, National Institute of Justice / NIST | **PERMISSIVE (US Gov work)** | Freely redistributable |
| AD-04 Göde et al. | Front matter, verbatim: *"This is an open access article under the CC BY-NC-ND license (http://creativecommons.org/licenses/by-nc-nd/4.0/)"* | **NON-COMMERCIAL + NO-DERIVATIVES** | Citable and readable. **Do not redistribute the PDF or derivatives with the game.** Same trap family as LaFAN1 in `motion-architecture/SOURCES.md` — facts and citations are free, the artifact is not |
| AD-02 Qiang et al. | SAGE author accepted manuscript, carries a "For Peer Review" watermark; no open licence stated | **UNCLEAR** | Cite by DOI; do not redistribute the file |
| AD-03 Børvik et al. | © 2001 Elsevier Science Ltd. Obtained from a **third-party mirror** (`aux.ciar.org`), not an authorised repository | **PROPRIETARY** | Content verified as the published version by its PII line `S0734-743X(01)00034-3` and journal pagination 27:19–35. Cite by DOI; **the mirror is not an authorised copy and must not be linked as if it were** |
| AD-05 Seidl et al. | proceedings.com "open" PDF, DOI 10.52202/080042-0046; no explicit licence on the paper | **UNCLEAR** | Cite by DOI |

---

## 10. Method notes for the next Toto

- **The DSpace REST API is a live route to gold-OA papers that ScienceDirect
  403s.** Unpaywall said `is_oa: true, oa_status: gold` but returned **no
  `url_for_pdf` at all**, and both ScienceDirect (403) and the naive
  `/bitstreams/{uuid}/content` guess (404) failed. What worked:
  `…/server/api/discover/search/objects?query=…` → item uuid →
  `…/core/items/{uuid}/bundles` → `ORIGINAL` bundle → `…/bitstreams` →
  `_links.content.href`. **This is the same shape as the 4TU/Figshare route in
  the body-rig log and it should now be treated as the standard move for any
  institutional repository.**
- **"Gold OA" does not mean "fetchable".** Three separate tools reported this
  paper as open while none of them could hand me the file. Openness status and
  retrievability are different facts; check the second one.
- **Author-accepted manuscripts on lab websites are underrated.** AD-02 came
  from a Chinese university lab's own `_upload/article/files/` directory,
  surfaced by an ordinary web search. Publisher paywall, lab open door.
- **Per-page character counts remain the cheap scan-detector** (body-rig log).
  All five PDFs here had real text layers (2 000–6 500 chars/page); **no OCR
  was needed on this topic, which is the first time that has been true.**
- **Verify a formula by reproducing a number the paper already printed.**
  AD-03 quotes Woodward's criterion in mangled OCR (`√3 p D/2`) and then says
  it gives *"about 17 mm"*. Computing `√3 × 20/2 = 17.321` confirmed I had read
  the formula correctly before I used it. **Two minutes; it is the difference
  between a criterion and a guess.**
- **State comparisons as ratios** (standing rule from the body-rig log). Every
  comparison in §3 and §5 is written as a division with both operands visible.
- **The most valuable single result this pass was a NULL.** AD-04's *"the multi
  hit shootings did not influence the main ballistic resistance"* is the finding
  that reframed the whole dispatch, and it would have been easy to skip as an
  unexciting paper. Read the conclusions of the boring-looking experiment.
- **Tier-V is still 0** across this repo. Not touched.
