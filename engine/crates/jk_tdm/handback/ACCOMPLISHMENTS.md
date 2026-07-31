# What was actually built and fixed — the full accomplishment brief

Everything below is committed, tested, and (where visual) photographed by
the repaired capture harness. Test count over this stretch: **49 → 127**.
Nothing on this list is aspirational; the honest not-done list is at the
bottom, because a brief that only lists wins is advertising.

---

## 1. Features built (new capability that didn't exist before)

### Combat & weapons
- **Bow draw/release system** — hold to draw (0.15s min, 0.7s full),
  power scales launch speed 19.25→55 m/s, forced letdown at 10s, arrows
  pierce up to 3 bodies at decaying damage (90 → 67.5 → 50.6). Later
  audits then wired what the first pass missed: real spread on release,
  auto-nock, and a preview that draws the actual power-scaled arc.
- **Spear throw** — committal 0.4s windup, angle-based stick-vs-bounce on
  impact (30° threshold), full zone damage (head ×2), carry cap of 2.
- **Grenade physics per surface** — stone bounces at 0.40, crates 0.50,
  vegetation *sticks* (0.05, instant rest). One shared integrator drives
  both the live flight and the aim preview, so they cannot disagree —
  now guaranteed by test to 0.1 mm.
- **Blast falloff curve** — flat 100% to 2 m, then linear breakpoints
  50%@6 m / 15%@12 m / 0%@20 m, replacing a hard 6 m cliff.
- **Minigun identity** — 0.4s spin-up, heat per shot, forced 3s vent at
  100 heat, manual vent on R, heat-widened spread (1.2°→3.5°) shown live
  on the crosshair, mass-tax on movement, idle barrel crawl vs vent
  freeze vs full-spin blur as three distinct visual states.

### Mech
- **1.7× scale walker** with olive-drab palette, forward stance lean,
  exposed knee/waist actuator geometry, hazard striping, and 33 hull
  plates.
- **Committal entry AND exit** — 1.6s seal-up, 1.2s power-down, firing
  blocked during both; being blown out mid-entry hands back a pilot who
  can fight immediately.
- **Damage-state plates** — 70/40/15% hull thresholds each fire exactly
  once; exposed frame takes ×1.25 after any plate drops.
- **Parity everywhere** — bot-piloted mechs obey the same crouch ban,
  turn-rate cap, and armour movement tax the player's does.

### Movement & animation
- **Turn-in-place** (§5.2, finished this pass) — the legs lag the aim at
  their own turn rate while the torso covers up to ±60°, so a quick flick
  reads as shoulders-first with the feet catching up. Mechs exempt: they
  turn as one piece, which is their commitment.
- **Living-motion layer** — per-fighter breathing (rate ramps with
  sprint), weight shift, grip fidget, head glances, damage flinch, ally-
  death head-snap, kill exhale, and a continuous burning shudder. All
  phase-seeded from entity id, never the sim RNG (replay-safe).
- **Kinetic-chain follow-through** — a spear throw's release carries the
  torso *past* the release angle then settles, driven by the same chain
  timing model the tests verify, with hand-off continuity from the windup
  (release cannot pop).
- **Hand craft** — two-bone arm IK hard-clamped to human elbow range,
  DIP-follows-PIP finger coupling, trigger-finger travel (0.06s out /
  0.10s back), weapon-mass lag per gun class.

### Camera
- **Third-person state machine** — hip 2.2 m boom, sprint eases to 2.5 m,
  aim pulls to 1.35 m, shoulder auto-mirror near walls, landing dip with
  a genuine delayed rebound that crosses neutral, all height-scaled so a
  3 m mech frames correctly.
- **Collision boom** — instant pull-in on contact, critically-damped
  k=90 spring push-out that governs *only* corner recovery (free-space
  eases keep their own documented timings — explicit occlusion state,
  because a distance test cannot tell the two apart).

### Modes & AI
- **AI teammates fight the horde** — Extraction bots now target zombies
  (per-kind heights, enemy fighters still win ties). The co-op mode has
  allies in it for the first time.
- **Explosives, fire, flame and repulsor all damage the horde** — every
  one previously looped players only. Now tested: a frag in a packed trio
  kills, a fire pool burns.
- **Extraction's no-respawn rule holds for every death** — grenade
  suicides no longer hand back a full-health rearm; one function now owns
  the rule at all five death sites.
- **Zombies gained a vertical reach check** — no more clawing players on
  crates above them — and their claws and the Bloater gas now go through
  the armour pipeline, so the mech hull, brace and shield all matter
  against the horde.

### Game prep & settings
- **Real settings** — mouse sensitivity (0.50–3.00×), FOV (62–110°,
  default 90), invert-Y, mouse-swap, minimap — applied live, labels
  derived from the same source as the input handler so they can't lie.
- **Loadout screen** — primary/secondary/special/melee/grenade preset/
  hat/outfit/map/difficulty/team size with a live spec readout.
- **The Forge** — 3 save/load slots over the cosmetic loadout, real file
  round-trip, confirmations now visible in the lobby where the Forge
  actually runs.

### Infrastructure
- **The capture harness** — scripted synthetic input driving the real
  client, one screenshot per beat (fixed from 157-per-beat), provably-
  clear staging positions, menu-state capture, typo-proof script names,
  keep-alive for weapon-feel scripts. This is the machine that makes
  "visible or it didn't happen" enforceable.
- **Config externalization slice** — `config/camera_tuning.txt` overrides
  7 camera-feel constants with proven no-op fallback on any malformed
  input.
- **R11 determinism guarantee** — 1000 seeded throws bit-identical, by
  test, compared on raw bits.

---

## 2. The audit programme (how 48+ bugs got found)

Four waves, ~78 agents. Every subsystem deep-read by its own agent, then
**every finding handed to a separate adversarial verifier told to refute
it**. 12 of 56 fully-verified claims were killed — including one where
the verifier confirmed the bug but corrected the finder's reasoning, and
only the accurate half got fixed.

| Wave | Scope | Confirmed / claimed |
|---|---|---|
| 1 | minigun, mech, spear, motion/camera | 12 / 16 |
| 2 | weapons, throwables, bots, abilities, physics | 18 / 20 |
| 3 | HUD, rendering, persistence, harness, modes | 14 / 20 |
| 4 | damage pipeline (audio/input/simcore lost to session limits) | 4 hand-verified / 7 |

Wave 4's verifiers hit the account's usage limit, so its four surviving
findings were **verified by hand against the code** rather than trusted:
Bloater gas bypassing armour (real — fixed through the pipeline), rockets
ignoring spawn protection (real — the one blast path missing the gate),
`death_respawn_t`'s doc overclaiming (real — all five sites now routed),
and blast hit-indicators pointing at the thrower's current position
instead of the explosion (real — fixed).

Three of the confirmed bugs were **my own, written earlier in the same
session** — the spear follow-through that did the opposite of its doc,
the camera spring that governed everything, and a turn-in-place first
draft whose own test caught it holding a permanent 60° twist. All three
are called out in their commits rather than quietly repaired.

### The five recurring bug shapes this codebase teaches
1. **The doc claims a consumer that doesn't exist** (five spring
   constants "wired" with one real call site; thrusters advertised two
   briefs after deletion; `torso_aim_offset` tested and never called).
2. **The client re-derives what the sim computes** and drifts (minigun
   crosshair, AWM bracket, bow preview, both mouse-mapping labels).
3. **A guard exists in one direction only** (throw-blocks-thrust but not
   thrust-blocks-throw; player mech crouch ban but not bots').
4. **A loop indexes a list that shrinks mid-loop** (axe sweep panic).
5. **State survives a transition it shouldn't** (cook timer across
   grenade switch, bot reaction time across respawn, intro UI into the
   match, mech timer onto the ejected pilot).

---

## 3. Verified research adopted

- **de Carpentier 2014** (closed-form ballistics): its "preview must use
  the throw's own solver" warning is enforced here by construction and by
  test. The solver itself deliberately **not** adopted — this sim's
  fixed-timestep determinism is load-bearing. Logged as a future
  throw-assist/bot-aim opportunity.
- **Level Design Book metrics** and the two-channel recoil model:
  recorded in `research/SOURCES.md`; recoil architecture validated as
  already-correct. Six further sources remain SNIPPET-ONLY and uncounted
  because nobody has actually read them yet — including me.

---

## 4. Still not done (the honest list)

- Castle map (IX-A), 26-piece armour/class system (IX-C), 20-part
  detachable mech, gatling/autocannon swap — all content builds, all
  sized in the open-work ledger artifact.
- Four of five named spring constants still unwired (need per-fighter
  spring state).
- `ElasticMove` / `counter_movement_bonus` remain spec fixtures — wiring
  them needs a sim-side pre-move velocity snapshot, a design decision.
- Wave 4's audio / input / sim-core audits never ran (session limits);
  those subsystems have had **zero** adversarial coverage.
- `briefs/PROMPT_MASTER_research_build.md` and its six sibling briefs are
  not on this machine; the research programme they define is only
  partially reconstructed from chat summaries.
