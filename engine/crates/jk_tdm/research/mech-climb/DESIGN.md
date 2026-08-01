# Infantry-vs-mech: hull climbing — design (Cycle 3)

Backlog #2. Per [S-02], grounds a stamina-gated grip mechanic rather
than a binary or unlimited climb.

## Purpose, and what the player feels

A soldier who closes distance to a mech today has no verb beyond
"stand closer and melee" — already fully damage-modeled (angle armor +
visor weak point apply uniformly to melee per `apply_armor_tagged`,
confirmed this cycle), but not distinctive. Climbing gives close-range
infantry a genuine alternative: mount the hull, strike an exposed zone
at a damage bonus, at real risk — both hands occupied (no ranged
weapon), a huge visible target for every other gun on the field, and a
grip that fatigues and can fail.

## Architecture

**Attach points reuse the existing plate-detach zones (§7.7), not a
new parallel system.** `mech_plates_dropped`'s bitmask (bit0=70%,
bit1=40%, bit2=15%) already tracks which armor has come off; a grip
point is only climbable once its covering plate is gone — climbing is
the PAYOFF for stripping a mech, not a free action from full health.
This ties a new mechanic directly into damage state that's already
SIM-authoritative, rather than inventing disconnected hull geometry.

```
ClimbState {
    mech_target: usize,      // fighter index of the mech
    attach_zone: PlateZone,  // which dropped-plate zone (matches the existing bitmask)
    grip_stamina: f32,       // 0..100, drains while attached
}
```

Added as `pub climbing: Option<ClimbState>` on `Fighter` — `None` for
everyone not currently gripping, same optional-state shape as
`bow_draw_t`/`knife_phase` elsewhere in this file.

**Grip stamina, per [S-02]:** drains at a fixed rate while attached,
recovers ONLY while not attached (matching the study's rest-only
recovery — grip-release-grip costs less than one continuous hold of
equal total duration, because the release ticks let the meter climb
back). Reaching 0 detaches involuntarily (falls). This is a real
gameplay cost, not a cosmetic timer — it directly bounds how long a
single climb attempt can run.

**While attached:** position is parented to the mech's transform plus
the attach point's local offset (moves WITH the mech, including
power-stride and pivot-turns — this is itself a soft counterplay: a
mech that strides or pivots hard is harder to stay attached to,
without needing a separate "shake" input). Only melee is usable — both
hands are gripping. A melee strike on the current attach zone gets a
bonus multiplier on top of the angle-armor result already computed
(the exposed-underframe ×1.25 from §7.7 already exists; climbing adds
a further multiplier for landing the strike AT the point stripped bare,
reachable only by being here).

**Detach conditions:** grip stamina hits 0 (falls, takes fall-adapted
landing per the existing no-instant-stop rule); the mech's hull hits 0
(rides the wreck down, same as any fall); the climbing soldier dies;
voluntary release (input).

## SIM/COSMETIC classification (R6/R7)

**SIM-safe, mandatory.** Attachment affects position, hit resolution,
and match outcome — it cannot be COSMETIC. Fixed-timestep, no new
randomness: grip drain is a deterministic rate, attach-point selection
is nearest-valid-zone (no RNG). This keeps the whole mechanic inside
the existing deterministic-replay guarantee without touching the
sim's seeded stream at all.

## Cost at crowd scale

Cheap by construction: mechs are singular/rare per encounter (not
hundreds of bodies), and the attach-point count per mech (bounded by
the 3 plate-zones × both sides, i.e. a handful) caps how many soldiers
can be attached to one mech simultaneously. This is not a
per-crowd-member cost the way locomotion or aim-assist would be.

## Interaction with existing systems

- **§7.7 plate-detach thresholds:** attach points ARE the dropped-plate
  zones — direct reuse, not a fork.
- **Power stride (this session, Section H):** striding moves the mech
  hard and caps its own turn rate; a rider parented to the hull rides
  through it, which is free, emergent counterplay — no special-case
  code needed beyond the position-parenting already described.
- **Angle armor / visor multiplier:** climbing strikes still go through
  `apply_armor_tagged` — the climb bonus stacks on top of, not instead
  of, the existing model.
- **20-segment body / grip pose library:** the actual grabbing-on pose
  is presentation layer, out of scope for this design (same split this
  session has kept throughout: sim mechanic first, render binding as
  separately-scoped follow-up work, per Cycle 1's mech-entry precedent).

## Rejected alternatives

- **Free climbing anywhere on the hull** (no fixed attach points) —
  rejected: unbounded per-frame grip-point geometry queries are more
  expensive and harder to keep replay-deterministic than a small fixed
  set; the existing plate-detach zones already discretize the hull for
  free.
- **Climbing as a pure damage buff** ("melee does more within Xm of a
  mech," no positional attachment) — rejected: delivers a number, not
  a verb. The backlog explicitly wants a distinctive interaction, and
  a buff dressed as a mechanic doesn't turn the mech into "an encounter
  rather than a health bar" the way physically riding it does.
- **Unlimited grip (no stamina)** — rejected directly on [S-02]'s
  finding: real grip fatigues under sustained isometric load. Removing
  that removes the risk/counterplay the backlog explicitly wants, and
  makes climbing strictly dominant over standing at range with nothing
  pulling the other way.

## Build-readiness checklist (what DEVELOP needs, next cycle)

1. `PlateZone` enum matching the existing bitmask's three zones (currently
   just bits, not named) — small refactor, needed before attach points
   can reference zones by name instead of bit index.
2. `ClimbState` struct + `Option<ClimbState>` field on `Fighter`, reset
   at every point the other optional combat states already reset
   (death/respawn — same discipline this session applied to
   `sprint_gate_t`/`stride_t` etc.).
3. Attach/detach input handling (context-interact when adjacent to an
   exposed zone, mirroring the mech's own board/exit interact pattern).
4. Position-parenting to the mech transform + zone offset.
5. Grip-stamina drain/recover tick + involuntary-detach-at-zero.
6. Melee-strike zone bonus wired into the existing `apply_armor_tagged`
   call site for knife/axe hits.
7. Tests: grip drains under hold and recovers under release (not
   symmetric — matching [S-02]'s cumulative-drain/rest-recovers shape);
   involuntary detach at zero stamina; position tracks the mech
   including through a power-stride burst; a climbing strike lands
   with the correct combined (angle armor × exposed-underframe ×
   climb-bonus) multiplier; attach fails on an un-stripped zone.

## What was not done this cycle, and why

DEVELOP is explicitly deferred. This cycle's DISCOVER+DESIGN work is a
complete, honest cycle output on its own — the brief's own Section 4
report format has a standing "what you did not do, and why" section
that anticipates exactly this outcome, and building six new
interaction points (attach input, position-parenting, stamina tick,
damage-site wiring, tests, and reset-site plumbing) without enough
remaining session budget to verify each one live would risk exactly
the kind of half-tested mechanic this session has repeatedly caught
and fixed in OTHERS' code this session (the wave audits, the
sprint-out gate, the power-stride heat gate) — better to hand the next
cycle a concrete, build-ready checklist than to ship something rushed
and unverified this late in an already very long session.
