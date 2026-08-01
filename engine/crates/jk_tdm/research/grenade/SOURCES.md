# grenade — SOURCES

| ID | Tier | Type | Title | Author / Studio | Year | URL | Accessed | Status | What it gave us |
|---|---|---|---|---|---|---|---|---|---|
| S-01 | P | paper | Analytical Ballistic Trajectories with Approximately Linear Drag | G.J.P. de Carpentier | 2014 | https://www.decarpentier.nl/ballistic-trajectories | 2026-07-31 | READ | Closed-form trajectories via linear-drag approximation; solves launch velocity for time-to-target, obstacle-clearance waypoint, arc height, launch/impact slope, exact speed, or minimum energy (what humans naturally throw). C++ snippets + open Unity demo. Valid 10-1000 m constant-gravity scales. |
| S-02 | S | blog | Projectile Prediction: Part 1 | Sam Reitich | — | https://sreitich.github.io/projectile-prediction-1/ | 2026-08-01 | READ | Named mechanisms: fake projectile, lerp-synchronization, fast-forwarding, partial fast-forwarding, resimulation, reconciliation. Numbers: 60 ms RTT => 30 ms server + 30 ms replication delay; 100 m/s projectile appears 3 m ahead at 60 ms ping; 200 ms ping => 10 m server / 20 m remote forward-prediction. Failure modes: visible backward jump on fake-to-real teleport; remote clients seeing a projectile after its impact; client/server hit mismatch. |

## NUMBERS

| ID | Value | Unit | What it measures | Conditions | Source |
|---|---|---|---|---|---|
| N-01 | 3 | m | fake-projectile lead over server truth | 100 m/s projectile, 60 ms RTT | S-02 |
| N-02 | 10 / 20 | m | server / remote forward-prediction distance | 200 ms ping | S-02 |
| N-03 | 10–1000 | m | validity range of linear-drag closed form | constant-gravity game scales | S-01 |

## Applied to this codebase (status, not aspiration)

- The de Carpentier warning — preview stepped by different code than the
  throw — is enforced here by construction (one `grenade_tick`) and by
  test (`a_thousand_identical_throws_land_bit_identically`, preview
  within 0.1 mm of flight).
- The closed-form solver itself is NOT adopted: this sim's fixed-timestep
  determinism (R11) is load-bearing; the solver remains the recorded
  option for throw-assist / bot grenade aim.
- S-02's netcode prediction mechanisms are NOT applicable yet - this
  build is local-only, no client/server split exists. Recorded so the
  names exist when netplay does.

## Added 2026-08-01 (Section F fetch batch)

| ID | Tier | Type | Title | Author / Studio | Year | URL | Accessed | Status | What it gave us |
|---|---|---|---|---|---|---|---|---|---|
| S-03 | P | canonical article | Fix Your Timestep! | Glenn Fiedler, Gaffer On Games | 2004/rev | https://gafferongames.com/post/fix_your_timestep/ | 2026-08-01 | READ | Mechanisms: fixed dt, variable dt, semi-fixed, "free the physics" (accumulator decoupling), interpolation with alpha = accumulator/dt. Numbers: example dt 1/60 s and 0.01 s; accumulator clamp 0.25 s. Failure modes: spiral of death; determinism broken by remainder-step float imprecision; temporal-aliasing stutter. Recommendation: accumulator + interpolation. DIRECTLY VALIDATES this codebase: jk_tdm's input loop is an accumulator clamped at 0.25 s with a step cap, and the R11 tests prove the determinism this pattern buys. |

Quota now: 3/12 counted (P: 2/3, V: 0/3).
