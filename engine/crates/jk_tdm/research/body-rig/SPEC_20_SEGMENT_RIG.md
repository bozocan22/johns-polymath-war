Note: claude-sonnet-5[1m] (the safety classifier) was unavailable when reviewing this subagent's work. Please carefully verify the subagent's actions and output before acting on them.

# SPEC — 20-Segment Mass-Bearing Body Rig
## BRIEF_VIII_B §B, implemented against `engine/crates/jk_tdm/src/main.rs`

**Audit verified against source.** Every line number, constant, and coupling claim in the input audit reproduces in the working tree at `engine/crates/jk_tdm/src/main.rs` (11553 lines) and `sim.rs` (11461 lines). Three corrections/additions to the audit are flagged inline (§0.4).

---

## §0 — DECISIONS MADE BEFORE ANY CODE

### 0.1 Dataset: de Leva 1996 (male), NOT Winter Table 4.1 — deviating from the brief

BRIEF_VIII_B §B.3/B.5 quotes **Winter Table 4.1**. The spec uses **de Leva 1996 (adjusted Zatsiorsky-Seluyanov), male column** instead. Three reasons, all decisive:

1. **Winter physically cannot supply what the brief asks for.** The brief demands a three-part trunk whose *spring stiffness is derived from inertia* (§B.5). Winter Table 4.1's radius-of-gyration cells for **Thorax, Abdomen, Pelvis, Thorax+Abdomen, Abdomen+Pelvis and whole Trunk are all blank**. There is no trunk inertia in Winter. de Leva publishes all three axes for UPT/MPT/LPT.
2. **Winter is not independent of Dempster.** Every source code in Table 4.1 is "Dempster via Miller and Nelson", "Dempster via Plagenhoef", or "Calculated". Dempster's sample: 8 cadavers, ages 52–83, mean 59.6 kg. de Leva/Zatsiorsky: 100 living male young adults, mean age 24, gamma-ray scan.
3. **The character is an athletic fighter**, which is the population de Leva measured and the opposite of Dempster's.

Consequence: `BRIEF_VIII_B §B.3`'s numbers do not appear in this spec. The mass-closure gate the brief asks for still passes — de Leva-M sums to exactly 1.0000 (verified below to 1e-6).

### 0.2 The load-bearing architectural call: **the three trunk pivots are CO-LOCATED**

`pelvis`, `lumbar` and `thorax` all sit at the **same point** (the hip). `lumbar` and `thorax` spawn at local `(0,0,0)`; only `pelvis` carries a Y offset.

This is the single decision that defuses audit risks **#4, #5, #6, #7 and #9** at once:

- Every torso-local literal (`0.846` head, `(±0.26, 0.62, 0.02)` shoulders, `weapon_root (0.10, 0.50, 0.14)`, all **33 armor plates**, all 9 trunk shells) stays in an **unchanged frame**. Zero coordinates get rewritten.
- `gait_pose` / `head_base_y`'s closed form `hip − drop + HEAD_OVER_HIP·cos(pitch)` stays **exactly** valid (proof in §4, Step 2).
- The mech visor's world-Y fraction does not move, so `sim.rs:5333`'s `frac > 0.82` classifier keeps agreeing with rendered geometry.

The bones exist to carry **independent yaw**, which is 100% of what the brief requires. They do *not* model spine curvature. The de Leva CoM/length data drives **inertia → spring constants → motion timing and damping**, never bone placement. This is exactly what §B.5 asks for ("feed them into the spring solver so stiffness is derived from mass").

**Cost, stated honestly:** no real lumbar CoM travel, no spine arc. If a later brief wants a visible spine curve, the lumbar gains a length and `head_base_y` must become a composed-transform query — that is a separate, larger change and is out of scope here.

### 0.3 Rotation composition law (the invariant everything rests on)

```
pelvis.rotation  = Ry(y_pel)
lumbar.rotation  = Ry(y_lum)
thorax.rotation  = Ry(y_tho) · Rx(pitch) · Rz(roll)
```

Consecutive `Ry` commute and add, so:

```
pelvis · lumbar · thorax  ==  Ry(y_pel + y_lum + y_tho) · Rx(pitch) · Rz(roll)
```

which is **bit-for-bit the legacy single-`torso` rotation** whenever `y_pel + y_lum + y_tho == ` the old total yaw. Pitch and roll live **only on the thorax** — that is what keeps `head_base_y` valid (a pure `Ry` does not change a vector's Y).

**Everything in §3 and §4 is built to preserve `Σ yaw`.** That sum is the assertion.

### 0.4 Corrections to the audit

| Audit claim | Correction |
|---|---|
| "`CHAIN_PEAK_SCALE` drives the spear follow-through" | **`CHAIN_PEAK_SCALE` is 100% inert in production.** `spear_followthrough_yaw` computes `chain_segment_scale(TIP,…) / CHAIN_PEAK_SCALE[TIP]`, and `chain_segment_scale` returns `peak * clamp(…)` — the division cancels the peak exactly. Index 7 is the only index read in production. **The peak table can be corrected with provably zero visual risk.** |
| BRIEF §B.6 "proportion test: every segment length within ±5% of B.4" | **Do not ship this test.** The rig is deliberately stylised and would fail every row by 13–37% (§2.4). Ship the **intra-limb ratio** test instead, which passes at ±10%. |
| BRIEF §B.3 foot split "hindfoot 0.011 / toe 0.0035" | **Inconsistent with de Leva's measured whole-foot CoM.** That split implies a foot CoM at 0.481 of foot length; de Leva measures 0.4415. §2.3 derives 0.01147/0.00223 which is consistent by construction. |

---

## §1 — THE 20 SEGMENTS AND THE HIERARCHY

`root` (the `FighterVis` entity) is **not** a segment — it is the world-transform carrier. `weapon_root`, `hat_socket` and `armor_rig` are attachment nodes, not mass segments.

```
root  ── FighterVis, Transform = f.pos / f.yaw / MECH_SCALE   [NOT a segment]
└── 1  Pelvis          rig.pelvis
    ├── 2  Lumbar      rig.lumbar
    │   └── 3  Thorax  rig.torso            (field name RETAINED = thorax)
    │       ├── 4  HeadNeck        rig.head          (renamed from rig.neck)
    │       ├── 5  ClavicleL       rig.clavicle[0]
    │       │   └── 7  UpperArmL   rig.arm_l[0]
    │       │       └── 9  ForearmL   rig.arm_l[1]
    │       │           └── 11 HandL      rig.arm_l[2]
    │       ├── 6  ClavicleR       rig.clavicle[1]
    │       │   └── 8  UpperArmR   rig.arm_r[0]
    │       │       └── 10 ForearmR   rig.arm_r[1]
    │       │           └── 12 HandR      rig.arm_r[2]
    │       ├── weapon_root                       [NOT a segment]
    │       └── armor_rig                         [NOT a segment]
    ├── 13 ThighL     rig.leg_l[0]
    │   └── 15 ShankL rig.leg_l[1]
    │       └── 17 FootL   rig.leg_l[2]     (hindfoot)
    │           └── 19 ToeL   rig.leg_l[3]  (forefoot + toes)
    └── 14 ThighR     rig.leg_r[0]
        └── 16 ShankR rig.leg_r[1]
            └── 18 FootR   rig.leg_r[2]
                └── 20 ToeR   rig.leg_r[3]
```

```rust
/// §B.1 (BRIEF_VIII_B): the 20 MASS-BEARING segments. Fingers are a
/// sub-rig on Hand{L,R} and are deliberately not counted here.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum SegmentId {
    Pelvis, Lumbar, Thorax, HeadNeck,
    ClavicleL, ClavicleR,
    UpperArmL, UpperArmR,
    ForearmL,  ForearmR,
    HandL,     HandR,
    ThighL,    ThighR,
    ShankL,    ShankR,
    FootL,     FootR,
    ToeL,      ToeR,
}
impl SegmentId { const ALL: [SegmentId; 20] = [ /* order above */ ]; }
```

`FighterRig` field changes (all additive except the two noted):

```rust
struct FighterRig {
    // ... existing phase/prev_speed/accel_lean/sprint_t/carry_t/prev_yaw_vis/wr_lag_* ...

    /// §B.2 the three-part trunk. All three pivots are CO-LOCATED at the
    /// hip; they exist to carry independent axial YAW, not spine curvature.
    /// `torso` is RETAINED as the field name and now holds the THORAX
    /// entity, so `zone_overlay`, the armor_rig/weapon_root parents and
    /// every `parts.get_mut(rig.torso)` compile unchanged.
    pelvis: Entity,
    lumbar: Entity,
    torso: Entity,               // == thorax

    /// §B.1 shoulder girdle: [left, right]. Parents of arm_{l,r}[0].
    clavicle: [Entity; 2],

    /// per side: [thigh (hip), shin (knee), foot (ankle), toe (MTP)]
    leg_l: [Entity; 4],          // WAS [Entity; 3]
    leg_r: [Entity; 4],          // WAS [Entity; 3]

    /// the HEAD pivot. Renamed from `neck`, which was misleading: the
    /// field was assigned the `head` entity (old main.rs:4173) and the
    /// real neck is a static cylinder shell on the thorax.
    head: Entity,                // WAS `neck: Entity`

    arm_l: [Entity; 3], arm_r: [Entity; 3],
    weapon_root: Entity, weapons: [Entity; N_WEAPONS],
    shield: Entity, bow_arrow: Entity, armor_rig: Entity,
}
```

`leg_*: [Entity; 4]` costs one array-literal edit in `spawn_fighter_rigs` and one added `if let Ok(...) = parts.get_mut(leg[3])` in `sync_fighters`. **`zone_overlay` (main.rs:8380) iterates `rig.leg_l.iter()`, so it gains toe debug coverage for free.**

---

## §2 — SEGMENT PARAMETERS (the data table)

All rows are **de Leva 1996 Table 4, MALE column** unless marked **DERIVED**.

**Convention change you must not get wrong:** de Leva reports trunk CoM as a percent of segment length **from the CRANIAL endpoint**. Our trunk bones' origins are at their **caudal (proximal-to-the-ground)** ends and point +Y up. Trunk `com_frac` below is therefore `1 − deLeva%`. Limb rows are already proximal-first and pass through unchanged. This inversion is the single most likely silent bug in the whole spec.

### 2.1 The table

`rg_frac` is the **sagittal** radius of gyration about the segment CoM (de Leva's `r`; `I = (M·m)·(l·r)²`). `len_frac` is length as a fraction of stature (derived by dividing de Leva's mm by his stated male stature 174.1 cm — de Leva does **not** publish length/height ratios).

| # | SegmentId | mass_frac | com_frac (from OUR origin) | rg_frac (sag) | len_frac (H) | Source |
|---|---|---|---|---|---|---|
| 1 | `Pelvis` | 0.1117 | **0.3885** (= 1 − 0.6115) | 0.615 | 0.0837 | de Leva LPT, OMPHALION–MID-HIP |
| 2 | `Lumbar` | 0.1633 | **0.5498** (= 1 − 0.4502) | 0.482 | 0.1238 | de Leva MPT, XIPHION–OMPHALION |
| 3 | `Thorax` | **0.1496** | **0.6801** | 0.716 | 0.0980 | de Leva UPT − 2 clavicles ⚠ |
| 4 | `HeadNeck` | 0.0694 | **0.4998** | 0.303 | 0.1395 | de Leva Head, **VERTEX–CERVICALE** ⚠ |
| 5,6 | `Clavicle{L,R}` | **0.0050** | 0.712 | **0.289** | **0.109** | **DERIVED** — see 2.2 |
| 7,8 | `UpperArm{L,R}` | 0.0271 | 0.5772 | 0.285 | 0.1618 | de Leva, SJC–EJC |
| 9,10 | `Forearm{L,R}` | 0.0162 | 0.4574 | 0.276 | 0.1545 | de Leva, EJC–WJC |
| 11,12 | `Hand{L,R}` | 0.0061 | 0.7900 | 0.628 | 0.0495 | de Leva, WJC–MET3 ⚠ |
| 13,14 | `Thigh{L,R}` | 0.1416 | 0.4095 | 0.329 | 0.2425 | de Leva, HJC–KJC |
| 15,16 | `Shank{L,R}` | 0.0433 | 0.4459 | 0.255 | 0.2493 | de Leva, KJC–LAT MALLEOLUS |
| 17,18 | `Foot{L,R}` | **0.011467** | **0.500** | **0.289** | **0.1067** | **DERIVED** — see 2.3 |
| 19,20 | `Toe{L,R}` | **0.002233** | **0.500** | **0.289** | **0.0415** | **DERIVED** — see 2.3 |

**Mass closure, exact:**
```
0.1117 + 0.1633 + 0.1496 + 0.0694                    = 0.4940
      + 2(0.0050 + 0.0271 + 0.0162 + 0.0061)          = 0.5028  → 0.9968… (running)
full: 0.4940 + 0.0100 + 0.0542 + 0.0324 + 0.0122
    + 0.2832 + 0.0866 + 0.022934 + 0.004466          = 1.000000
```
Closes to **1e-6**, better than the brief's ±0.001 gate.

⚠ **Three definitional traps, all load-bearing:**
- **`HeadNeck` uses de Leva's ALTERNATIVE endpoint row (VERTEX–CERVICALE, L 242.9 mm, CoM 50.02%, r 30.3/31.5/26.1)**, not the primary VERTEX–MID-GONION row. The research states this explicitly: *"Use this variant if your neck joint sits at C7."* Our head pivot is the neck base. Mass `0.0694` still **excludes** neck soft tissue — de Leva leaves neck mass inside the trunk (he did not adjust masses at all). Winter's head+neck is 0.081; the 0.0116 difference is the neck and it lives in our `Thorax`, which is also where the neck cylinder shell hangs. Closure is unaffected. **Do not "fix" this by adding 0.0116 — it would double-count.**
- **`Hand` uses de Leva's SHORT hand (WJC → 3rd metacarpale, L 86.2 mm, CoM 0.79).** The rig's hand is a mitten ball ending at the knuckle, so this is correct. The alternative WJC→3rd-dactylion row (L 187.9, CoM 0.3624) is a **different segment**; mixing them is the documented common error.
- **`Foot` here is de Leva's heel→toe-tip definition, split.** Winter's foot (lateral malleolus → metatarsal-II head) **excludes the toes and uses a different axis**. The two are not interchangeable. Never take `com_frac` from one and `mass_frac` from the other.

### 2.2 GAP: the clavicle — no measured data exists in any source

**Stated plainly: there is no clavicle inertia data in de Leva, Winter, or Dempster.** de Leva contains no clavicle, scapula or shoulder girdle at all — the word does not appear in the paper. Winter has exactly one row, "Shoulder mass" (sternoclavicular joint → glenohumeral axis), and it is **crippled: mass_frac cell BLANK, all three radius-of-gyration cells em-dashes.** Only CoM 0.712/0.288 and density 1.04 are printed.

**Derivation, marked DERIVED:**

| Quantity | Value | How |
|---|---|---|
| `com_frac` 0.712 | **published** | Winter Table 4.1 "Shoulder mass", proximal = sternoclavicular = our bone origin. The one real clavicle number that exists anywhere. |
| `mass_frac` 0.0050 | **UNMEASURED** | Carried over from BRIEF §B.3's own figure. **Carved out of `Thorax` (UPT 0.1596 → 0.1496), not added**, so closure is exact by construction. |
| `com_frac` correction to Thorax | 0.7001 → **0.6801** | Removing 2×0.005 sitting at the top of the UPT (fraction 1.0) shifts the residual CoM down: `(0.1596·0.7001 − 0.010·1.0)/0.1496 = 0.6801`. Computed, not fudged. |
| `rg_frac` 0.289 | **GEOMETRIC MODEL** | Uniform slender rod about its own centroid: `1/√12 = 0.2887`. Longitudinal axis floored at **0.10** to keep the inertia tensor non-singular (a true rod has zero polar inertia). |
| `len_frac` 0.109 H | **DERIVED** | Drillis & Contini half-shoulder-width 0.129 H minus ≈0.02 H for the sternoclavicular offset from the midline. |

**Sensitivity bound (why this is acceptable):** `I_clavicle ≈ 0.005 · (0.2887 · 0.109 · 1.78)² · M ≈ 1.4e-4·M` — **three orders of magnitude below the thorax**. The only consumer is spring stiffness (§5), where a ±50% error moves a segment whose total travel is ±7.7 cm. It is bounded further by `CLAV_YAW_MAX` (§4, Step 4).

### 2.3 GAP: the toe/forefoot — derived from de Leva's measured foot CoM

**No source in the research treats the foot as anything but one rigid segment.**

**A subtraction route was tried and is invalid — record the negative result:** Winter's toeless foot is 0.0145 of body mass, de Leva's foot-with-toes is 0.0137. The *toeless* foot is *heavier*. That proves the difference is a dataset offset (Dempster's elderly cadavers vs Zatsiorsky's young adults), not anatomy. **Do not derive toe mass by differencing the two published foot definitions.**

**Derivation that works — solve the split from the measured whole-foot CoM.** Let `L` be foot length, `s = 0.72` the metatarsophalangeal break fraction from the heel (**ASSUMED — no source in the research payload gives it**), and each part uniform so its CoM is at its own midpoint (hindfoot 0.36 L, toe 0.86 L). de Leva measures the whole-foot CoM at **0.4415 L** from the heel.

```
m_h·0.36 + m_t·0.86 = m_F·0.4415          and   m_h + m_t = m_F
⇒ 0.36·m_F + 0.50·m_t = 0.4415·m_F
⇒ m_t = (0.4415 − 0.36)/0.50 · m_F = 0.163 · m_F
⇒ m_t = 0.163 × 0.0137 = 0.002233        m_h = 0.0137 − 0.002233 = 0.011467
```

**Validation — the split reconstructs de Leva's measured whole-foot radius of gyration.** Normalise `L = 1`, `m_F = 1`, `m_h = 0.837`, `m_t = 0.163`; each part a uniform slab so `r = 0.2887 × its own span`; parallel-axis about the whole-foot CoM at 0.4415:

```
I_h        = 0.837 × (0.2887 × 0.72)² = 0.837 × 0.0432075 = 0.0361647
I_t        = 0.163 × (0.2887 × 0.28)² = 0.163 × 0.0065345 = 0.0010651
m_h·d_h²   = 0.837 × (0.4415 − 0.36)²  = 0.837 × 0.0066423 = 0.0055596
m_t·d_t²   = 0.163 × (0.86 − 0.4415)²  = 0.163 × 0.1751423 = 0.0285482
                                                  Σ = 0.0713376
r_foot(model) = √0.0713376 = 0.2671     vs de Leva MEASURED 0.257  →  +3.9%
```

**A two-part uniform-slab model reproduces the measured whole-foot inertia to within 4%.** That is a real check, and it ships as an assertion (§5, Step 0). The brief's 0.0035 toe mass fails the same check — it would put the whole-foot CoM at 0.481 instead of the measured 0.4415.

**Not derivable, art-directed and labelled as such:** the MTP break at 0.72 L, and toe plantarflexion ROM (`TOE_OFF_RAD = 0.45 rad`). Neither exists in any source consulted.

### 2.4 Rig proportions vs the literature — **the brief's ±5% test must not ship**

The character is heavily stylised (mitten hands, 0.38 m-wide head ellipsoid, wide yoke). Rig lengths at `BODY_HEIGHT = 1.78`:

| Segment | Research (H-fraction → m) | Rig actual | Δ |
|---|---|---|---|
| Thigh | 0.2425 → 0.432 | 0.29 (`0.63 → −0.29`) | **−32.8%** |
| Shank | 0.2493 → 0.444 | 0.28 | **−36.9%** |
| Upper arm | 0.1618 → 0.288 | 0.21 (`ELBOW_Y`) | **−27.1%** |
| Forearm | 0.1545 → 0.275 | 0.19 (`WRIST_Y`) | **−30.9%** |
| Foot | 0.1482 → 0.264 | 0.22 (mesh z-span) | −16.6% |
| Biacromial | 0.259 H → 0.461 | 0.52 (`2×SHOULDER_X`) | **+12.7%** |
| Hip width | 0.191 H → 0.340 | 0.22 | −35.3% |
| Shoulder height | 0.818 H | 0.702 H | −14.2% |

**Every row fails ±5%.** Shipping BRIEF §B.6's proportion test as written means a red suite on the character that is already approved and on screen. **Ship this instead**, which passes:

```rust
/// §B.4: absolute limb lengths are ART-DIRECTED (the rig is stylised and
/// runs 27-37% short in every limb). What must NOT drift is the INTRA-LIMB
/// RATIO - that is what makes a body read as a body.
#[test]
fn intra_limb_length_ratios_match_the_literature() {
    // thigh:shank  rig 0.29/0.28 = 1.0357  vs de Leva-M 422.2/434.0 = 0.9728
    let leg = (0.29 / 0.28) / (0.2425 / 0.2493);          // = 1.065
    // upper:fore   rig 0.21/0.19 = 1.1053  vs de Leva-M 281.7/268.9 = 1.0476
    let arm = (0.21 / 0.19) / (0.1618 / 0.1545);          // = 1.055
    assert!((leg - 1.0).abs() < 0.10, "thigh:shank ratio drifted: {leg}");
    assert!((arm - 1.0).abs() < 0.10, "upper:fore ratio drifted: {arm}");
}
```

---

## §3 — KINETIC-CHAIN TIMING FROM MEASUREMENT

### 3.1 Current state and what is actually consumed

`CHAIN_ONSET_OFFSETS = [0.000, 0.020, 0.035, 0.055, 0.065, 0.090, 0.110, 0.125]` (main.rs:396) was authored by feel. Its **only two production paths** are:

1. `chain_lag_chase` (main.rs:422) uses `CHAIN_ONSET_OFFSETS[7]` as a first-order time constant → `rig.head` pitch.
2. `spear_followthrough_yaw` (main.rs:467) samples the tip from `CHAIN_ONSET_OFFSETS[7]` → `rig.torso` yaw.

`CHAIN_PEAK_SCALE` has **zero** production effect (§0.4).

`chain_peak_tick(i) = onset[i] + ramp_s` with a **shared** `ramp_s`. Therefore **only the differences between offsets are load-bearing**; the absolute zero is arbitrary and the shared ramp absorbs the onset-to-peak lag. That is what licenses mapping measured *peak* times onto *onset* offsets.

### 3.2 The measured anchors

**Campos, Brizuela & Ramón (2004), New Studies in Athletics 19:47-57, Table 3.** n = 7 elite male javelin finalists, World Championships Seville 1999, two SVHS cameras at 50 fps, DLT, quintic spline. **Verified against the primary PDF in two independent extraction modes.**

| Marker | Peak linear velocity, relative to release |
|---|---|
| Hip | −0.130 s (individuals 0.12–0.16, SD 0.01) |
| Shoulder | −0.090 s (every individual is 0.08 or 0.10) |
| Elbow | −0.060 s (individuals 0.05 or 0.06) |
| Release (tip) | 0.000 s |

**Mapping markers → segments (this matters).** A joint marker's linear velocity peaks when the segment **proximal** to it is at peak angular velocity — the marker is that segment's distal endpoint. So: hip marker ← `Pelvis`; shoulder marker ← `Clavicle` (the shoulder joint is the clavicle's distal end); elbow marker ← `UpperArm`.

### 3.3 The arithmetic — offsets from chain start

Chain start := the pelvis peak. `offset(x) = t(x) − t(hip)`:

```
offset(Pelvis)   = −0.130 − (−0.130) = 0.000   MEASURED
offset(Clavicle) = −0.090 − (−0.130) = 0.040   MEASURED
offset(UpperArm) = −0.060 − (−0.130) = 0.070   MEASURED
offset(Tip)      =  0.000 − (−0.130) = 0.130   MEASURED
```

Chain span **130 ms**. Measured inter-anchor gaps: **40 ms → 30 ms**, i.e. the chain compresses distally by a factor of **0.75**.

Four indices are unmeasured: `Lumbar(1)`, `Thorax(2)`, `Forearm(5)`, `Hand(6)`. Two different interpolation rules, each chosen for a stated physical reason.

**(a) Trunk window `Pelvis(0.000) → Clavicle(0.040)`, three hops — inertia-weighted.** In the trunk the segments are massive and the delay to each is dominated by the inertia that must be accelerated. Longitudinal inertia from the table above, `I ∝ m·(r_long · L)²`:

```
I_MPT (lumbar)  = 0.1633 × (0.468 × 0.2155)² = 0.001661015
I_UPT (thorax,  = 0.1496 × (0.659 × 0.1707)² = 0.001893083
       post-carve)
I_clav          = 0.0050 × (0.2887 × 0.1898)² = 0.0000150   ← ~0.4% of the budget
```

The clavicle hop lands at **0.17 ms** — physically right (it is nearly massless) but useless as an animation offset and it would make strict-ordering pass only on float noise. **Floor every hop at 5 ms**: above f32 noise, and below the finest sampling resolution in any source (handball 4 ms @ 250 Hz; this javelin study 20 ms @ 50 Hz), so it never claims precision the data does not have.

```
clavicle hop = 5 ms (floor)
remaining 35 ms split I_MPT : I_UPT = 0.001661015 : 0.001893083 = 0.46736 : 0.53264
  Pelvis → Lumbar  = 35 × 0.46736 = 16.36 ms
  Lumbar → Thorax  = 35 × 0.53264 = 18.64 ms
  Thorax → Clavicle=  5.00 ms
⇒ Lumbar = 0.0164   Thorax = 0.0350   Clavicle = 0.0400  ✓ hits the measured anchor
```

**(b) Arm window `UpperArm(0.070) → Tip(0.130)`, three hops — geometric compression.** Distal segments are not gated by their own mass (the hand is 1/7 of the forearm's inertia; inertia weighting would give it a 5 ms hop and the forearm 39 ms, which contradicts the measured trend). They are gated by the sequencing strategy, and the literature gives us that trend directly: 40 → 30 ms, ratio 0.75. Extrapolate geometrically with ratio `q` from the measured 30 ms base, solved to land **exactly** on the measured 130 ms release anchor:

```
30q + 30q² + 30q³ = 60   ⇒   q + q² + q³ = 2   ⇒   q = 0.8107
  UpperArm → Forearm = 30 × 0.8107   = 24.32 ms
  Forearm  → Hand    = 30 × 0.65723  = 19.72 ms
  Hand     → Tip     = 30 × 0.53282  = 15.99 ms      Σ = 60.03 ms
⇒ Forearm = 0.0943   Hand = 0.1140   Tip = 0.1300  ✓ hits the measured anchor
```

### 3.4 The replacement constants

```rust
/// §3 (BRIEF_VIII_B): the kinetic chain, DERIVED FROM MEASUREMENT.
/// Anchors 0/3/4/7 are Campos, Brizuela & Ramon (2004) NSA 19:47-57
/// Table 3 - n=7 elite male javelin finalists, Seville 1999 World
/// Championships, 50 fps. Indices 1,2 are inertia-weighted across the
/// de Leva trunk subsegments; 5,6 are a geometric compression seeded by
/// the measured 40->30 ms gap and solved to land exactly on the 130 ms
/// release anchor. Full arithmetic in the spec, §3.3.
///
/// ONLY THE DIFFERENCES ARE LOAD-BEARING: `chain_peak_tick` adds a
/// SHARED `ramp_s` to every entry, so the absolute zero is arbitrary and
/// the shared ramp absorbs the onset-to-peak lag.
const CHAIN_ONSET_OFFSETS: [f32; 8] =
    [0.000, 0.016, 0.035, 0.040, 0.070, 0.094, 0.114, 0.130];

/// The four MEASURED anchors, held separately so the test above compares
/// two independent tables rather than a constant against itself.
const JAVELIN_ANCHOR_S: [(usize, f32); 4] =
    [(0, 0.000), (3, 0.040), (4, 0.070), (7, 0.130)];

/// Peak angular-velocity multiplier per segment. Indices 0..=2 are
/// MEASURED: thorax/pelvis peak angular velocity ratio = 1.43, the mean
/// of four marker-based pitching datasets (Wada 2025 659/1025 = 1.556;
/// Ramsey & Crotin 2025 584.1/797.0 = 1.364 and 658.9/851.0 = 1.292;
/// Owens 2024 432/646 = 1.495). Lumbar = sqrt(1.43) = 1.196, one hop.
/// Indices 3..=7 are NOT derivable from any source consulted - their
/// per-hop gains are carried over UNCHANGED from the by-feel table and
/// rescaled onto the new thorax value. See §0.4: index 7 CANCELS in the
/// only production consumer, so this whole table is currently inert.
const CHAIN_PEAK_SCALE: [f32; 8] =
    [1.000, 1.196, 1.430, 1.554, 1.679, 1.990, 2.300, 2.611];

/// §B.1: the chain's 8 indices, named against real rig segments. Index 7
/// ("tip") is the WEAPON - the chain's output, not a body segment. This
/// is the mapping the pre-existing test at main.rs:11477 asserted by
/// string literal against bones that did not exist.
const CHAIN_SEGMENTS: [Option<SegmentId>; 8] = [
    Some(SegmentId::Pelvis),    Some(SegmentId::Lumbar),
    Some(SegmentId::Thorax),    Some(SegmentId::ClavicleR),
    Some(SegmentId::UpperArmR), Some(SegmentId::ForearmR),
    Some(SegmentId::HandR),     None,
];
```

### 3.5 Delta from the shipped table, and corroboration

| idx | segment | old | new | Δ |
|---|---|---|---|---|
| 0 | pelvis | 0.000 | 0.000 | 0 |
| 1 | lumbar | 0.020 | **0.016** | −4 ms |
| 2 | thorax | 0.035 | **0.035** | 0 |
| 3 | clavicle | 0.055 | **0.040** | **−15 ms** |
| 4 | upper_arm | 0.065 | **0.070** | +5 ms |
| 5 | forearm | 0.090 | **0.094** | +4 ms |
| 6 | hand | 0.110 | **0.114** | +4 ms |
| 7 | tip | 0.125 | **0.130** | +5 ms |

The by-feel author was within 15 ms everywhere and exact on the thorax. **This de-risks Step 1 substantially** — the two production consumers move by ≤5 ms (a 4% change in the head-lag time constant and in the follow-through sample point).

**Independent corroboration (Serrien & Baeyens 2018, *J Hum Kinet* 63:9-21 — random-effects meta-analysis, 14 articles, ~20 samples of experienced handball players, with 95% CIs):** measured pelvis→trunk peak-velocity gaps are **24 ms** (standing throw with run-up, I² = 0%), **38 ms** (jump throw), **61 ms** (penalty throw). Our derived pelvis→thorax span is **35 ms** — inside that band and, correctly, nearer the run-up value than the planted penalty throw, since a javelin delivery is a run-up throw.

**Explicitly NOT used:** the "≈17–39 ms baseball pelvis-to-torso delay" is a *derivation* (percent-of-cycle × 0.145 s), not a published number, and is superseded here by the directly-measured javelin anchors.

**Precision ceiling — do not exceed it.** The javelin source is 50 fps = 20 ms per frame. Its between-athlete spreads (hip 40 ms, shoulder 20 ms, elbow 10 ms) are 2, 1, and 0.5 frames. **Never build per-fighter timing variance, CV differences, or anything finer than ~20 ms resolution on top of this table.** The 5 ms floor is a monotonicity device, not a claim of 5 ms accuracy.

---

## §4 — MIGRATION PATH

Eight steps. Each compiles, ships green, and reverts with a single `git revert`. **No step touches `sim.rs`.**

Dependency graph: `0 → 1` independent · `2 → 3 → 3b` · `2 → 4` · `5` needs only `0` · `6` needs only `0` · `7` needs `5`.

---

### **Step 0 — the data layer, zero rig change**

Add `mod bsp` inside `main.rs` (the crate is a single-file binary plus `sim`; do not create a new crate).

```rust
/// §B.3/B.5 (BRIEF_VIII_B): body segment parameters as DATA.
/// de Leva, P. (1996) "Adjustments to Zatsiorsky-Seluyanov's segment
/// inertia parameters", J Biomech 29(9):1223-1230, Table 4, MALE column
/// (n=100 living young adults, gamma-ray scan, mean age 24, reference
/// body 73.0 kg / 174.1 cm).
///
/// NOT Winter Table 4.1: every source code in Winter is a Dempster
/// derivative (8 cadavers, ages 52-83, mean 59.6 kg) and Winter publishes
/// NO radius of gyration for ANY trunk subsegment - which is exactly what
/// a three-part trunk needs. See spec §0.1.
mod bsp {
    use super::SegmentId;

    #[derive(Clone, Copy)]
    pub struct SegmentDef {
        pub id: SegmentId,
        pub parent: Option<SegmentId>,
        /// fraction of TOTAL BODY MASS
        pub mass_frac: f32,
        /// CoM along this bone's own axis measured from THIS BONE'S
        /// ORIGIN (its proximal joint). Trunk rows are ALREADY inverted
        /// from de Leva's cranial-first convention - see spec §2.
        pub com_frac: f32,
        /// sagittal radius of gyration about the segment CoM, as a
        /// fraction of segment length (de Leva's r; I = (M*m)*(l*r)^2)
        pub rg_frac: f32,
        /// segment length as a fraction of STATURE
        pub len_frac: f32,
        /// false => this row is DERIVED, not measured. Clavicle and toe
        /// have NO published inertia data in de Leva, Winter or Dempster.
        pub measured: bool,
    }

    pub const SEGMENTS: [SegmentDef; 20] = [ /* §2.1 table */ ];

    impl SegmentDef {
        /// Moment of inertia about this segment's PROXIMAL joint, kg*m^2.
        /// Parallel-axis from de Leva's centroidal radius:
        ///   I_prox = m * L^2 * (rg^2 + com^2)
        pub fn inertia_proximal(&self, body_mass_kg: f32, stature_m: f32) -> f32 {
            let m = body_mass_kg * self.mass_frac;
            let l = stature_m * self.len_frac;
            m * l * l * (self.rg_frac * self.rg_frac + self.com_frac * self.com_frac)
        }
    }
}

/// Render-side only. MUST NOT be added to sim.rs - the sim has no mass
/// model and adding one would put a new field in the replay state.
const BODY_MASS_KG: f32 = 78.0;
```

**Nothing consumes it.** Ships behind zero behaviour change.

**Tests:**
| Test | Assertion |
|---|---|
| `bsp_masses_close_to_unity` | `SEGMENTS.iter().map(mass_frac).sum::<f64>()` within `1e-6` of `1.0` |
| `bsp_has_exactly_twenty_named_segments` | `SegmentId::ALL.len() == 20`; every `SEGMENTS[i].id == SegmentId::ALL[i]`; no duplicate ids |
| `bsp_hierarchy_is_a_tree_rooted_at_pelvis` | every non-`Pelvis` has `Some(parent)`; `Pelvis.parent == None`; walking parents from any node reaches `Pelvis` in ≤6 hops (proves acyclic + connected) |
| `derived_foot_split_reproduces_de_leva_whole_foot` | reassemble hindfoot+toe via parallel axis (§2.3): reconstructed `r_foot` within **5%** of de Leva's measured `0.257`, and reconstructed CoM within `1e-6` of `0.4415` |
| `derived_rows_are_flagged_as_derived` | `{id : !measured}` is **exactly** `{ClavicleL, ClavicleR, FootL, FootR, ToeL, ToeR}` — stops a future edit silently promoting a guess to a measurement |
| `intra_limb_length_ratios_match_the_literature` | §2.4 verbatim |

**Revert:** delete the module.

---

### **Step 1 — chain constants from measurement**

Swap `CHAIN_ONSET_OFFSETS` / `CHAIN_PEAK_SCALE`, add `JAVELIN_ANCHOR_S` and `CHAIN_SEGMENTS` (§3.4). Refactor `spear_followthrough_yaw` so the peak scale is a parameter, purely so the inertness test can run.

**COSMETIC.** Two consumers move ≤5 ms.

**Tests:**
| Test | Assertion |
|---|---|
| `chain_onsets_hit_the_measured_javelin_anchors` | for `(i, t)` in `JAVELIN_ANCHOR_S`: `(CHAIN_ONSET_OFFSETS[i] - t).abs() < 1e-4` |
| `chain_onsets_are_strictly_increasing_with_a_five_ms_floor` | `for i in 1..8: OFFSETS[i] - OFFSETS[i-1] >= 0.005` |
| `chain_gaps_compress_distally_between_the_measured_anchors` | `40 > 30 > 24.3 > 19.7 > 16.0` ms — **assert at anchor granularity, not per index**; per-index gaps (16,19,5,30,24,20,16) are correctly non-monotone because of the 5 ms clavicle floor |
| `peak_scale_pelvis_to_thorax_matches_measured_ratio` | `(PEAK[2]/PEAK[0] - 1.43).abs() < 0.05` |
| `peak_scale_is_inert_in_the_follow_through` | sweep `release_t ∈ 0..0.6`: `spear_followthrough_yaw_with_peak(t, 2.611) == spear_followthrough_yaw_with_peak(t, 5.222)` **bit-identically** — proves index 7 cancels |
| existing `kinetic_chain_peaks_fire_in_strict_proximal_to_distal_order` | **unchanged, must still pass** |
| existing `kinetic_chain_segment_is_silent_before_its_own_onset` | **unchanged, must still pass** |
| existing `the_head_trails_a_sprint_start_then_settles` | **unchanged, must still pass** (reads `OFFSETS[7]` symbolically) |
| existing `spear_followthrough_carries_past_the_release_then_settles` | **unchanged, must still pass** |

**Revert:** restore two array literals.

---

### **Step 2 — insert `pelvis` / `lumbar` / `thorax` as a PROVABLE NO-OP**

The structural step. It changes the hierarchy and changes **nothing on screen**.

**Spawn** (`spawn_fighter_rigs`, ~main.rs:3821-3898):
```rust
let pelvis = commands.spawn((Transform::from_xyz(0.0, 0.63, 0.0), Visibility::default()))
    .set_parent(root).id();
let lumbar = commands.spawn((Transform::IDENTITY, Visibility::default()))
    .set_parent(pelvis).id();
let torso  = commands.spawn((Transform::IDENTITY, Visibility::default()))
    .set_parent(lumbar).id();          // == THORAX
```
- Legs reparent `root → pelvis`; `thigh` spawn goes `(lx, 0.63, 0.0)` → **`(lx, 0.0, 0.0)`**.
- All 9 trunk shells, `head`, both `upper` arms, `weapon_root`, `armor_rig` reparent to `torso` — **their local Transforms are untouched** (they were already `torso`-local, and `torso` is now the thorax at the same origin).

**Sync** (`sync_fighters`): delete `leg[0].translation.y = hip_y` (main.rs:6201; leg[0].y is now a spawn constant `0.0`) and replace the `rig.torso` block (main.rs:6362-6381) with a call to one **pure** function:

```rust
/// §B.2: the three co-located trunk pivots. Extracted PURE so the test
/// can assert `pelvis * lumbar * thorax` equals the legacy single-`torso`
/// transform bit-for-bit, and so `head_base_y` keeps sampling the real
/// render path.
///
/// pitch and roll live ONLY on the thorax. That is what preserves
/// `head_base_y`'s closed form: pelvis and lumbar are pure Ry, and a Ry
/// does not change a vector's Y component.
fn trunk_locals(
    hip_y: f32, crouch_drop: f32, breath: f32, sway_x: f32,
    yaw_pelvis: f32, yaw_lumbar: f32, yaw_thorax: f32,
    pitch: f32, roll: f32,
) -> (Transform, Transform, Transform) {
    (
        Transform::from_xyz(0.0, hip_y, 0.0)
            .with_rotation(Quat::from_rotation_y(yaw_pelvis)),
        Transform::from_xyz(sway_x, breath - crouch_drop, 0.0)
            .with_rotation(Quat::from_rotation_y(yaw_lumbar)),
        Transform::IDENTITY.with_rotation(
            Quat::from_rotation_y(yaw_thorax)
                * Quat::from_rotation_x(pitch)
                * Quat::from_rotation_z(roll)),
    )
}
```

At Step 2, `yaw_pelvis = yaw_thorax = 0.0` and `yaw_lumbar` = the whole legacy expression `0.045·sin(phase)·amp + spear_yaw + torso_aim`.

**Why it is a no-op — the proof to put in the commit message:**
- **Translation.** Legs: `pelvis(hip_y) · leg[0](0)` → world Y `hip_y`, identical to today. Trunk: `pelvis(hip_y) · lumbar(sway_x, breath−drop) · thorax(0)` → `(sway_x, hip_y − drop + breath, 0)`, identical to today's `rig.torso` write.
- **Rotation.** `Ry(0)·Ry(a)·(Ry(0)·Rx(p)·Rz(r)) = Ry(a)·Rx(p)·Rz(r)` — exactly today's torso rotation.
- **`head_base_y` stays valid.** Head local `(0, 0.846, 0)`; `Rz(r)` → `(·, 0.846·cos r, ·)`; `Rx(p)` → `y = 0.846·cos r·cos p`; the two outer `Ry` leave Y alone. World head Y = `hip − drop + breath + 0.846·cos p·cos r` — **the same expression as today**, including the same pre-existing omission of the `cos r` term in `head_base_y`.
- **Mech visor unmoved** (audit #9 neutralised): the trunk pivots are co-located, so the visor's local `y = 0.885` is unchanged.
- **Death freeze (audit #14):** in the dead branch (main.rs:6007-6021, before `continue`), add `pelvis.rotation = Quat::IDENTITY` and `lumbar.rotation = Quat::IDENTITY`. **No new persistent `FighterRig` state is introduced** — the twist is derived per-frame from `spear_wind_t` / `knife_phase` / `release_t`, which the sim already resets on respawn. So the audit's respawn-mid-animation class of bug cannot occur here.

**Tests:**
| Test | Assertion |
|---|---|
| `trunk_pivots_compose_to_the_legacy_single_torso_transform` | over a sweep of `hip_y ∈ {0.54,0.63}`, `drop ∈ {0,0.12}`, `breath`, `sway_x`, `yaw ∈ ±1.2`, `pitch ∈ 0..0.185`, `roll ∈ ±0.2`: `(p.compute_matrix()*l.compute_matrix()*t.compute_matrix())` vs the legacy `Transform` matrix, **max element delta < 1e-6** |
| `legs_attach_at_the_hip_not_the_crouch_dropped_torso` | composed world Y of `leg[0]` `== hip_y` for `crouch ∈ {false,true}` — the 12 cm crouch drop must **not** reach the legs (audit #2) |
| `head_pivot_world_y_matches_head_base_y` | composed world Y of the head pivot vs `head_base_y(...)`, delta `< 1e-4`, over the same sweep the band test uses |
| `mech_visor_lower_edge_stays_in_the_head_band` | `(0.63 + 0.885 - 0.014) * MECH_SCALE / (BODY_HEIGHT * MECH_SCALE) >= HEAD_BAND_FRAC` → **0.8433 ≥ 0.82**, margin 9.4 cm at mech scale |
| existing `head_never_leaves_its_band_in_any_gait` | **unchanged, must pass** |
| existing `rig_joints_bridge_with_no_daylight` | **unchanged, must pass** |
| existing `roll_settle_still_dips_when_the_band_allows_it` | **unchanged, must pass** |

**Revert:** re-parent legs to `root`, restore the single `rig.torso` write. The added `pelvis`/`lumbar` fields can stay dead or go with it.

---

### **Step 3 — split the yaw (the feature the brief demands)**

```rust
/// §B.2: how the ACROMION-LINE separation `sep` distributes across the
/// trunk bones. The returned components ALWAYS sum to `sep` - that
/// invariant is what keeps the composed thorax world-yaw identical to the
/// pre-split single-torso yaw, so `rig_separation_tests` keeps measuring
/// the real render path.
///
/// TWIST_SPINE_FRAC 0.60 is MEASURED: a stereo-radiograph study reports
/// that shoulder-relative-to-TORSO motion contributes ~40% of the total
/// axial rotation of the shoulders relative to the pelvis (Bourgain et al.
/// 2022, Sports 10(6):91, section 3.4). The same review's cross-study
/// figures (torso-vs-pelvis ~30 deg vs shoulders-vs-pelvis ~60 deg) imply
/// 0.50; 0.60 is taken from the direct measurement of the mechanism, and
/// 0.50 is the documented lower bound if this needs tuning.
const TWIST_SPINE_FRAC: f32 = 0.60;

/// DERIVED: inverse-inertia compliance. For equal internal torque, the
/// segment with the smaller longitudinal inertia twists more.
///   I_MPT = 0.1633*(0.468*0.2155)^2 = 0.001661015  -> 1/I = 602.0
///   I_UPT = 0.1496*(0.659*0.1707)^2 = 0.001893083  -> 1/I = 528.2
///   lumbar share = 602.0 / (602.0 + 528.2) = 0.5326  ... using the
///   post-carve UPT; with de Leva's raw UPT it is 0.548. Either lands the
///   "mid trunk is the twist segment" read the brief asks for.
const TWIST_LUMBAR_SHARE: f32 = 0.548;

/// §1.3 "hips lead shoulders". SHIPS AT 0.0 - see Step 3b.
const PELVIS_LEAD_FRAC: f32 = 0.0;

/// Returns (pelvis, lumbar, thorax, clavicle) yaw. Both clavicles take
/// the SAME signed yaw: rotating both by q about a shared midline origin
/// rotates the acromion line by exactly q (Ry(q)*(1,0,0) = (cos q,0,-sin q)).
fn trunk_twist_split(sep: f32) -> (f32, f32, f32, f32) {
    let pel   = sep * PELVIS_LEAD_FRAC;
    let rest  = sep - pel;
    let spine = rest * TWIST_SPINE_FRAC;
    let lum   = spine * TWIST_LUMBAR_SHARE;
    (pel, lum, spine - lum, rest - spine)
}
```

**Step 3 ships with `TWIST_SPINE_FRAC = 1.0`** (no clavicles exist yet, so the clavicle share has nowhere to live); Step 4 re-splits to 0.60/0.40. **At every step the four components sum to `sep`.**

`sep` is `torso_coil_yaw(...)` unchanged — so `hip_shoulder_separation_reaches_35_to_45_degrees_at_windup` (main.rs:11369) keeps passing and, for the first time, keeps *meaning* something about the render path.

**Weapon-root compensation (do not skip):** `weapon_root` is a thorax child. Once the clavicle takes 40% of the coil (Step 4), the weapon would under-rotate by 40%. Fix without reparenting or touching the IK frame — one term on an already-fully-driven rotation:
```rust
// main.rs:6597 becomes:
t.rotation = Quat::from_rotation_y(clav_yaw) * wr_rot * Quat::from_rotation_x(-0.5 * fidget);
```
Composed weapon yaw = `lumbar + thorax + clav = sep`, exactly today's value. This lands in **Step 4**, with the clavicles.

**Tests:**
| Test | Assertion |
|---|---|
| `trunk_twist_split_sums_to_its_input` | for `sep ∈ −1.2..1.2` step 0.01: `|Σ(components) − sep| < 1e-6` |
| `acromion_separation_through_the_composed_bones_hits_35_to_45_deg` | build the three locals from `trunk_twist_split(torso_coil_yaw(Spear, wind_t, ..))`, compose to matrices, extract world yaw of thorax and of pelvis, peak of the difference **∈ [35°, 45°]** |
| `the_thorax_segment_alone_carries_about_half_the_acromion_value` | `thorax_only_sep / acromion_sep ∈ [0.50, 0.62]` — reproduces the golf review's published definitional ratio (~30° torso-method vs ~60° shoulder-method) |
| `pelvis_lead_zero_leaves_the_legs_exactly_where_they_were` | with `PELVIS_LEAD_FRAC = 0.0`, `trunk_twist_split(sep).0 == 0.0` for all `sep` |
| existing `hip_shoulder_separation_reaches_35_to_45_degrees_at_windup` | **unchanged, must pass** |
| `separation_is_genuinely_nonzero_not_a_fused_bone` | **STRENGTHEN**: compare the composed world yaws of two *different* entities rather than testing `torso_coil_yaw != 0.0` |
| existing `no_gun_no_twist` | **unchanged, must pass** |

**Revert:** `TWIST_SPINE_FRAC` / `TWIST_LUMBAR_SHARE` → put the whole `sep` on `yaw_lumbar`.

---

### **Step 3b — turn on `PELVIS_LEAD_FRAC` (one constant, own commit)**

`PELVIS_LEAD_FRAC: 0.0 → 0.15`. Because the legs are now `pelvis` children, this **twists the feet** — correct human motion (the rear foot pivots on a throw) but a visible change to leg animation, so it gets its own commit and its own capture. Assert: `|Σ| == sep` still holds, and `leg[0]` composed world yaw shifts by exactly `sep · 0.15`.

---

### **Step 4 — clavicles**

**Spawn:** clavicle at **thorax-local `(0.0, 0.62, 0.02)`** — the midline sternoclavicular point at shoulder height. `upper` becomes its child at **clavicle-local `(±SHOULDER_X, 0.0, 0.0)`** (was thorax-local `(±0.26, 0.62, 0.02)`, a duplicated literal). With clavicle rotation identity the composed shoulder position is **exactly** `(±0.26, 0.62, 0.02)` in thorax space — provable no-op at spawn.

**Sync — three edits, all in `sync_fighters`:**

1. Kill the duplicated shoulder literals (main.rs:6604-6605), audit risk #4:
```rust
// WAS: let sh_l = Vec3::new(-0.26, 0.62, 0.02);
let clav_o = Vec3::new(0.0, 0.62, 0.02);
let clav_q = Quat::from_rotation_y(clav_yaw.clamp(-CLAV_YAW_MAX, CLAV_YAW_MAX));
let sh_l = clav_o + clav_q * Vec3::new(-SHOULDER_X, 0.0, 0.0);
let sh_r = clav_o + clav_q * Vec3::new( SHOULDER_X, 0.0, 0.0);
```
2. The IK output is now expressed in *clavicle* space, not thorax space (main.rs:6745):
```rust
// solve_arm_ik still solves in THORAX space; re-express for the new parent
if let Ok((mut t, _)) = parts.get_mut(arm[0]) { t.rotation = clav_q.inverse() * sh; }
```
3. The weapon-root yaw compensation from Step 3.

`solve_arm_ik`'s `L1`/`L2 = 0.21` are untouched (they must stay `== ELBOW_Y`; `WRIST_Y = -0.19` is already 2 cm out of sync and is left alone — a pre-existing, documented approximation, not this spec's business).

```rust
/// Clavicle protraction/retraction limit. BOUNDED BY GEOMETRY, not taste:
/// the shoulder travels SHOULDER_X*sin(q) fore/aft, and the yoke shell's
/// Z half-depth is 0.12 - so q_max = asin((0.12 - 0.005)/0.26) = 0.458 rad
/// before daylight opens at the shoulder. 0.30 rad (17deg) keeps a wide
/// margin and is above the 0.292 rad that a full 0.73 rad coil demands.
const CLAV_YAW_MAX: f32 = 0.30;
```

**Tests:**
| Test | Assertion |
|---|---|
| `clavicle_identity_puts_the_shoulder_on_the_legacy_literal` | composed shoulder pos in thorax space `== Vec3::new(∓0.26, 0.62, 0.02)` within `1e-6` when `clav_q == IDENTITY` |
| `clavicle_yaw_rotates_the_acromion_line_by_exactly_its_own_yaw` | for `q ∈ ±CLAV_YAW_MAX`: yaw of `(sh_r − sh_l)` `== q` within `1e-6` |
| `the_shoulder_travels_when_the_clavicle_rotates` | `‖sh_r(q) − sh_r(0)‖ == 2·SHOULDER_X·sin(q/2)` within `1e-5` — proves the IK reads the clavicle, not a literal |
| `weapon_root_yaw_is_preserved_across_the_clavicle_split` | composed weapon-root world yaw with the compensation term `== lumbar + thorax + clav == sep`, within `1e-6` |
| `yoke_still_reaches_the_travelled_shoulder` | `0.24/2.0 >= SHOULDER_X * CLAV_YAW_MAX.sin() + 0.005` → `0.12 ≥ 0.0818` |
| `four_way_twist_split_still_sums_to_sep` | with `TWIST_SPINE_FRAC = 0.60`, `|Σ − sep| < 1e-6` |
| existing `rig_joints_bridge_with_no_daylight` | **unchanged, must pass** (`YOKE_HALF_W ≥ SHOULDER_X` still holds; `sh` X shrinks to `0.26·cos q`) |

**Revert:** reparent `upper` to `torso` with the original literal transform; restore the two `sh_*` literals; drop the `clav_yaw` term from `wr_rot`.

---

### **Step 5 — toes (independent of Steps 2–4)**

```rust
/// §B.1 the metatarsophalangeal break, in FOOT-LOCAL Z. The rig's foot
/// mesh spans z = -0.06 (heel) .. +0.16 (toe tip), length 0.22 m. The
/// break sits at 0.72 of foot length from the heel:
///   -0.06 + 0.72 * 0.22 = 0.0984
/// 0.72 is ASSUMED - no source consulted (de Leva, Winter, Dempster,
/// Drillis & Contini) gives an MTP location. It is the ONE free parameter
/// in the foot split; everything else follows from de Leva's measured
/// whole-foot CoM (0.4415 of foot length from the heel).
const TOE_BREAK_Z: f32 = 0.098;
/// MTP joint ball radius. Must fully bridge the foot's 0.09 m depth.
const TOE_R: f32 = 0.048;

/// §B.6 toe-off: MTP plantarflexion through the stance->swing transition.
/// Sign convention matches the leg chain (+X swings the segment back).
/// TOE_OFF_RAD is ART-DIRECTED - no toe ROM exists in any source consulted.
const TOE_OFF_RAD: f32 = 0.45;
const TOE_OFF_PHASE: f32 = 2.35;   // contact exit
fn toe_off(theta: f32, amp: f32) -> f32 {
    let w = (theta - TOE_OFF_PHASE).cos().max(0.0);
    TOE_OFF_RAD * w * w * amp      // cos^2 lobe: smooth, zero outside
}
```

**Spawn** — replace the single foot ball (old main.rs:3882-3889) with:
- hindfoot ball, foot-local `(0.0, -0.025, 0.024)`, scale `(0.14, 0.09, 0.168)` → spans `z ∈ [-0.06, 0.108]` (10 mm past the break)
- MTP joint ball, foot-local `(0.0, -0.025, TOE_BREAK_Z)`, scale `splat(TOE_R * 2.0)`
- `toe` pivot, foot-local `(0.0, -0.025, TOE_BREAK_Z)`
- toe ball, toe-local `(0.0, 0.0, 0.026)`, scale `(0.13, 0.075, 0.072)` → spans foot-local `[0.088, 0.16]` (10 mm before the break)

Union at identity = the old `[-0.06, 0.16]` span. Overlap 20 mm plus the MTP ball.

**Sync** — add after the `leg[2]` write (main.rs:6214):
```rust
if let Ok((mut t, _)) = parts.get_mut(leg[3]) {
    t.rotation = Quat::from_rotation_x(if rolling || airborne { 0.0 }
                                       else { toe_off(rig.phase + off, amp) });
}
```

**Deliberately additive.** The existing ankle-roll fudge in `foot` (main.rs:6196-6198) is left in place, so there is **zero regression on the ankle**. Netting the fake out of the ankle is a separate tuning commit; the brief's toe-off gate passes immediately either way.

**Tests:**
| Test | Assertion |
|---|---|
| `toe_rotates_through_its_range_at_contact_exit` (brief §B.6) | sweep `θ ∈ 0..TAU` at `amp = 1.0`: `max(toe_off) >= 0.9 * TOE_OFF_RAD`, `min == 0.0`, and `argmax` within `±0.25` rad of `TOE_OFF_PHASE` |
| `toe_is_silent_when_standing` | `toe_off(θ, 0.0) == 0.0` for all θ — a stationary fighter's toes do not twitch |
| `toe_geometry_bridges_the_metatarsal_break` | `TOE_R*2.0 >= 0.09 + 0.005`; hindfoot z-end `0.108 >= TOE_BREAK_Z + 0.005`; toe z-start `0.088 <= TOE_BREAK_Z - 0.005` |
| `toe_split_preserves_the_foot_silhouette` | hindfoot span ∪ toe span `== [-0.06, 0.16]` at identity |
| existing `head_never_leaves_its_band_in_any_gait` | **unchanged, must pass** — the toe is below the hip and cannot reach the head band; assert additionally that no toe constant appears in `gait_pose`'s signature |

**Revert:** one mesh block and one `parts.get_mut` block.

---

### **Step 6 — inertia-derived spring stiffness (BRIEF §B.5's real ask)**

**Honest scope statement first.** The research supplies **no joint stiffnesses**, so an absolute `k` cannot be derived from it. What the inertia data *does* supply rigorously is the **relative** stiffness across segments: for a shared target natural frequency `f`, `k = I·(2πf)²`. One art-directed frequency per joint class then replaces N independent hand-tuned `k`s, and the relative feel across segments becomes physically correct automatically. That is precisely the tuning loop §B.5 wants removed.

```rust
impl bsp::SegmentDef {
    /// Critically-damped rotational spring about the proximal joint for a
    /// target natural frequency. `f_hz` is ART-DIRECTED; the RATIOS between
    /// segments are DATA. Returns (k [N*m/rad], c [N*m*s/rad]).
    pub fn spring(&self, body_mass_kg: f32, stature_m: f32, f_hz: f32) -> (f32, f32) {
        let i = self.inertia_proximal(body_mass_kg, stature_m);
        let w = std::f32::consts::TAU * f_hz;
        (i * w * w, 2.0 * i * w)
    }
    /// First-order chase time constant for the same target.
    pub fn tau(&self, f_hz: f32) -> f32 { 1.0 / (std::f32::consts::TAU * f_hz) }
}
```

Worked values at `BODY_MASS_KG = 78.0`, `stature = BODY_HEIGHT = 1.78`:

| Segment | `I_proximal` (kg·m²) | `k` at `f = 4 Hz` |
|---|---|---|
| `Thorax` | 0.3462 | 218.7 |
| `HeadNeck` | 0.1140 | 72.0 |
| `Forearm` | 0.02727 | 17.2 |
| `Toe` | 3.170e-4 | 0.20 |

`I_thorax / I_toe = 1092`. **That three-order span is the entire point** — it is what hand-tuning never gets right, and it is now free.

**Convert exactly one consumer** so the mechanism is proved without a broad regression surface: the head lag. Its time constant becomes `CHAIN_ONSET_OFFSETS[TIP]` (which is itself measurement-derived after Step 1), and the inertia layer sets its damping relative to the other segments.

**Tests:**
| Test | Assertion |
|---|---|
| `derived_stiffness_is_proportional_to_segment_inertia` | for any two segments and any `f_hz`: `k_a/k_b == I_a/I_b` within `1e-5` — proves stiffness is inertia-driven, not hand-set |
| `derived_stiffness_spans_three_orders_from_thorax_to_toe` | `I(Thorax)/I(Toe) > 500` (actual 1092) |
| `the_head_lag_time_constant_still_matches_the_shipped_value` | derived τ within **15%** of `0.130` |
| existing `the_head_trails_a_sprint_start_then_settles` | **unchanged, must pass** |

**Revert:** revert the one consumer; the API can stay (it is inert without callers).

---

### **Step 7 — retire the duplicated spawn literals; harden the tests (audit #8)**

`rig_joints_bridge_with_no_daylight` (main.rs:10531-10572) currently asserts against **hand-transcribed copies** of spawn literals: `0.625 + 0.07`, `0.846`, `0.09 - 0.08`, `-0.145`, `-0.29`, `-0.14`, `-0.28`, `0.065`, `0.045`, `-0.02 + 0.05`. **The test is asserting against a transcription of the old spawn code, not against the spawn code.** Promote every one to a named const in the §1 connectivity block (`HIP_BALL_R`, `PELVIS_SHELL_*`, `THIGH_MESH_*`, `KNEE_Y`, `ANKLE_Y`, `KNEE_BALL_R`, `ANKLE_BALL_R`, `MITTEN_*`, `YOKE_TOP`, `HEAD_PIVOT_Y = HEAD_OVER_HIP`) and have both `spawn_fighter_rigs` and the test read the consts.

Also here: rename `FighterRig::neck` → `head` (mechanical; `zone_overlay` main.rs:8386 is the only other reader), and add:

```rust
/// Every named segment, for debug overlay coverage and for the segment-count
/// gate. 20 mass segments + weapon_root.
fn rig_debug_entities(rig: &FighterRig) -> Vec<Entity> { /* … */ }
```

**Tests:**
| Test | Assertion |
|---|---|
| `head_pivot_constant_has_exactly_one_definition` | `HEAD_PIVOT_Y == HEAD_OVER_HIP`, and the `0.846` literal at the sync site (old main.rs:6451) is replaced by the const |
| `zone_overlay_covers_every_named_segment` | `rig_debug_entities(&rig).len() == 21` (20 segments + `weapon_root`) |
| existing `rig_joints_bridge_with_no_daylight` | rewritten to read only consts; **must still pass with identical numeric outcomes** |

---

## §5 — SIM-AFFECTING vs COSMETIC

### 5.1 The bright line

**Every step in this spec is COSMETIC. Not one line of `sim.rs` is edited.**

The bit-identical replay guarantee holds **by construction**, not by testing: `sim.rs` contains no `Entity`, no `Transform`, no bone. Its entire body model is a capsule plus a Y-fraction classifier (`apply_hit`, sim.rs:5319-5341) and `muzzle_origin` (sim.rs:4760). Nothing in the rig feeds the sim, and no rig change can desync a replay.

### 5.2 The two contracts that keep it that way

| Contract | Where | How this spec honours it |
|---|---|---|
| Rendered head geometry ≥ `HEAD_BAND_FRAC` (0.82) × height | `sim.rs:5333` `frac > 0.82 → HitZone::Head`; guarded by `head_never_leaves_its_band_in_any_gait` | The trunk pivots are **co-located** and pitch/roll stay on the thorax, so `head_base_y`'s closed form remains exactly valid. Step 2 adds `head_pivot_world_y_matches_head_base_y`, which samples the **composed** transform rather than trusting the closed form. |
| Rendered mech visor stays in the same band | `sim.rs:6647` `zone == HitZone::Head`; test fixtures at `sim.rs:8614/9812/11431` use `BODY_HEIGHT * MECH_SCALE * 0.90` | The visor is a static `armor_rig` plate at thorax-local `y = 0.885`, and the thorax origin does not move. Step 2 adds an explicit assertion: lower slit edge at frac **0.8433 ≥ 0.82**. |

### 5.3 Hard rules for the implementer

1. **`sim.rs` is untouched at every step.** If a diff shows `sim.rs`, the step is wrong.
2. **`BODY_MASS_KG` lives in `main.rs`, never in `sim.rs`.** Adding a mass field to the sim would put new state in the replay.
3. **No new sim-read state.** All twist is derived per-frame from `f.spear_wind_t`, `f.knife_phase` and `rig`-local follow-through clocks — fields the sim already resets on respawn.
4. **Run the existing determinism suite unchanged at every step**, as a leak detector:
   - `a_thirty_shot_ak_spray_is_bit_identical_on_replay` (sim.rs:8152)
   - `spray_replays_exactly_climbs_and_recovers` (sim.rs:9464)
   - `minigun_heat_cycle_is_deterministic` (sim.rs:8917)
   - `zombies_spawn_chase_headshot_and_replay` (sim.rs:10034)
   - the per-map `assert_eq!(run_map(map), run_map(map))` gate (sim.rs:8603)

   If any of these changes behaviour, a sim-layer edit leaked in. Nothing in §4 should touch them.

### 5.4 If a future brief wants a SIM-affecting version

Two candidates, both explicitly **out of scope** and both requiring a replay-version bump:

- **Per-segment hitboxes** replacing the Y-fraction classifier. This deletes `apply_hit`'s `frac > 0.82` cascade and makes the sim read rig geometry — it is a full replay-format change and should never be bundled with cosmetic rig work.
- **Ragdoll driven by `bsp::SEGMENTS` masses.** Only sim-affecting if ragdoll state feeds back into gameplay. If ragdolls are render-only, this stays cosmetic and can consume the Step 0 data directly.

---

## §6 — WHAT THIS SPEC DOES NOT CLAIM

1. **Clavicle mass (0.005) is a guess.** No measured value exists in de Leva, Winter or Dempster. Carved from the thorax so closure is exact; bounded by `CLAV_YAW_MAX` and by being 1/1000th of the thorax's inertia.
2. **Clavicle radius of gyration (0.289) is a uniform-rod geometric model**, not a measurement. Winter's three radius cells are em-dashes.
3. **The MTP break at 0.72 of foot length is assumed.** Nothing else in the foot split is — the masses fall out of de Leva's measured whole-foot CoM and the model reconstructs his measured whole-foot radius to 3.9%.
4. **`TOE_OFF_RAD = 0.45` is art-directed.** No toe ROM in any source consulted.
5. **`CHAIN_PEAK_SCALE[3..=7]` is not derivable.** Only the pelvis→thorax ratio (1.43) is measured. Those five entries carry their by-feel per-hop gains forward, and index 7 cancels in the only production consumer anyway.
6. **The javelin timing rests on one 50 fps study of n = 7** (`New Studies in Athletics` — a coaching-federation publication, not peer-reviewed; its prose contains multiple errors, though its Table 3 is clean and internally consistent). The *ordering* is independently corroborated (Navarro et al. at 200 Hz; Serrien & Baeyens's pooled handball meta-analysis with CIs). **The 20 ms frame interval is the precision ceiling — do not build sub-frame variation on top of it.**
7. **The trunk carries no spine curvature.** Co-located pivots are a deliberate trade (§0.2): full axial separation, zero coordinate churn, `head_base_y` preserved.
8. **de Leva's female column is not used and should not be** without fresh work: n = 15, mean age 19, stature 173.5 cm (statistically indistinguishable from the male mean), and the LPT/thigh rows diverge from the male column far more than any other segment.

---

## FILES TOUCHED

| Path | What |
|---|---|
| `engine/crates/jk_tdm/src/main.rs` | **all of it** — `mod bsp`, `SegmentId`, `FighterRig`, `spawn_fighter_rigs`, `sync_fighters`, `zone_overlay`, the chain constants, `trunk_locals`, `trunk_twist_split`, `toe_off`, and all new tests |
| `engine/crates/jk_tdm/src/sim.rs` | **NONE. Zero lines, at every step.** |