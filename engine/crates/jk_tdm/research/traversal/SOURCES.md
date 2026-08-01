# traversal — SOURCES

| ID | Tier | Type | Title | Author / Studio | Year | URL | Accessed | Status | What it gave us |
|---|---|---|---|---|---|---|---|---|---|
| S-01 | S | analysis | How iFrames Augment Dodge Rolls | Celia Wagar, CritPoints | 2017 | https://critpoints.net/2017/07/25/how-iframes-augment-dodge-rolls/ | 2026-08-01 | READ | Mechanisms: overlap-dependent invincibility (i-frames only need to cover hitbox overlap) and post-invincibility positioning (the roll must END somewhere safe). Core claim: i-frames must NOT span the whole roll or direction choice stops mattering. Failure modes: zero-i-frame dodges create forced-damage "checkmate scenarios"; full-duration invincibility deletes directional decision-making. Named counter-example: Nioh — fast movement, very few i-frames. No frame counts given. |
| S-02 | P/V | GDC talk | Vault, Slide, Mantle: Building Brink's SMART System | Splash Damage | 2012 | https://gdcvault.com/play/1015930 | 2026-08-01 | SNIPPET-ONLY | (Vault access required.) Snippet: precomputes traversal opportunities by analysing world geometry offline — no designer-placed hint volumes, lower runtime cost. Directly relevant to castle geometry, but NOT COUNTED until actually watched. |
| S-03 | P | thesis | Procedural Parkour and Traversal Animation Techniques (MSc, Bournemouth NCCA) | J. Macey (supervised) | 2024 | https://nccastaff.bournemouth.ac.uk/jmacey/MastersProject/MSc24/02/ProceduralParkourandTraversalAnimationTechniques.pdf | 2026-08-01 | UNREACHABLE | PDF exceeds this tool's 10 MB fetch limit. Not counted. Snippet claims Control Rig + procedural IK + motion warping; unverified. |

## Applied to this codebase

- This game's dodge roll has NO i-frames at all - by design, the roll
  ducks the head OUT of the head band (roll_loads_bursts_eases_and_
  ducks_headshots), which is Wagar's "positional avoidance" model taken
  to its limit. Her checkmate-scenario warning is the test case to keep:
  the roll's speed burst (now +12% on a counter-movement launch) is the
  leniency mechanism instead of invincibility. No change required;
  recorded as a validated design position.

## Quota: 1/12 counted (P: 0/3, V: 0/3). HONESTLY SHORT.

## Added 2026-08-01 (Section F fetch batch)

| ID | Tier | Type | Title | Author / Studio | Year | URL | Accessed | Status | What it gave us |
|---|---|---|---|---|---|---|---|---|---|
| S-04 | P | engine docs | Motion Warping in Unreal Engine | Epic Games | current | https://dev.epicgames.com/documentation/en-us/unreal-engine/motion-warping-in-unreal-engine | 2026-08-01 | READ | Mechanisms: named Warp Targets; notify WINDOWS (start/end handles inside the montage define WHEN warping applies); root-motion modifiers (uniform Scale vs Skew Warp to land the animation end on the target). Numbers: Warp Rotation Time Multiplier (0.5 halves rotation completion time). Constraints: root motion required; target name must match. The alignment model for our mantle/vault: play the authored move, warp the root inside a declared window so the hands land on the ACTUAL ledge instead of snapping - the answer to "the wall stop". |

Quota now: 2/12 counted (P: 1/3, V: 0/3).
