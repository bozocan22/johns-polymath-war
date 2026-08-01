# R&D Cycle 3 report — infantry-vs-mech hull climbing (design only)

Per `briefs/PROMPT_RND_CYCLE.md` Section 4. Backlog item #2.

## 1. What was built

Nothing shipped to `sim.rs` this cycle. What exists: a verified finding
that the CORE of "infantry vs. giant mech" — differentiated damage by
angle, a visor weak point, and progressive plate stripping — was
**already real and already applies uniformly to melee** (confirmed by
reading `apply_armor_tagged`'s call sites, not assumed), closing off a
false gap before researching it. What's genuinely missing is the
distinctive verb the backlog names: climbing. A complete DESIGN.md
(architecture, SIM classification, cost, rejected alternatives,
build-readiness checklist) was produced instead of a rushed build —
explained under §7 below.

## 2. Evidence

[S-02] MacDonald et al. 2022, read via PMC open access (full text
verified, direct quotes extracted, not a summary trusted at face
value). n=10 climbers, 30-min continuous top-rope sessions, grip
strength fell 22-23%, natural rest ~78s between routes, authors'
stated mechanism: sustained isometric contraction → reduced blood flow
→ swelling → fatigue. This grounds the design's stamina model:
cumulative drain under hold, recovery only when released — not a
binary grip toggle and not a single timer indifferent to how the
player actually climbs. [S-01] (the GDC Shadow of the Colossus
postmortem, the canonical design reference) stayed SNIPPET-ONLY —
Vault-gated, no transcript, same limitation hit on every GDC source
this session.

## 3. Tests

None this cycle — no code shipped. Six specific tests are named in
DESIGN.md's build-readiness checklist for the next cycle that
implements it, each tied to a concrete assertion (grip drain
asymmetry, involuntary detach, position tracking through a
power-stride burst, combined multiplier correctness, attach-gating on
stripped zones).

## 4. Capture

None — no visible feature exists yet.

## 5. Rejected alternatives

Three, each with the axis that killed it — full reasoning in
DESIGN.md: free climbing anywhere on the hull (determinism/cost:
unbounded per-frame geometry queries vs. reusing the already-discrete
plate-detach zones for free); climbing as a pure damage buff (doesn't
deliver a verb, just a number); unlimited grip / no stamina (directly
contradicted by [S-02] and removes the risk the backlog explicitly
wants).

## 6. Backlog delta

- Item #2: **design done, build queued** — see DESIGN.md's
  build-readiness checklist; promoted to the top of Critical for the
  next cycle since the design is now concrete and estimated at a
  bounded, known set of six changes.
- No new items discovered this cycle beyond what DESIGN.md already
  scopes.

## 7. What was not done, and why

DEVELOP was deliberately not attempted this cycle. This is the fourth
substantial cycle/section in one continuous session (mech entry
staging, grenade friction, and this design work, on top of the
sprint-out gate and power-stride mechanic earlier in the same
session) — implementing six new interaction points (attach input,
position-parenting through the mech's transform including mid-stride,
a new stamina resource with asymmetric drain/recover, damage-site
wiring into `apply_armor_tagged`, new reset-site plumbing across
death/respawn, and real tests for each) without enough remaining
budget to build AND verify each one live would risk shipping exactly
the kind of half-tested mechanic this session's own audit waves exist
to catch. A concrete, build-ready design with a numbered checklist is
a real and honest cycle output — Section 4 of the brief itself expects
a "what you did not do, and why" section for precisely this reason.

## Rotating codebase review (Section 5) — categories 9-10 this cycle

(Cycle 1: 1-4. Cycle 2: 5-8. Continuing the rotation.)

9. *Input validation / file-boundary safety* — N/A this cycle, no new
   file I/O or external input surface touched.
10. *Dead code and orphaned systems* — none found in the mech/armor
    modules touched while reading `apply_armor_tagged` and the
    plate-detach bitmask this cycle.

No findings from this pass worth a backlog row.
