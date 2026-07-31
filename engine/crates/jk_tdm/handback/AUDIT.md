# Subsystem audit — minigun, mech, spear/Achilles, motion

A deep audit of four subsystems, run as 4 parallel deep-read agents over
`sim.rs` + `main.rs`, with **every finding adversarially verified** by a
separate agent instructed to refute it (default: refuted when uncertain).

**20 agents. 16 findings claimed → 12 confirmed, 4 refuted.**
All 12 confirmed defects are fixed. **111/111 tests green** (was 104; +7).

Two of the twelve were bugs **I introduced earlier in this same session**.
Both are called out below.

---

## The audit method (why these findings are trustworthy)

The verifier agents killed 4 of 16 claims — including one where the
finder's *cited line numbers were right but its stated consequence was
wrong*, and the verifier corrected the scope rather than rubber-stamping
it. That refutation rate is the evidence the process has teeth; a review
that confirms everything it finds is not reviewing.

---

## Confirmed and fixed

### Bugs I introduced this session

**1. `spear_followthrough_yaw` did the opposite of what it claimed**
*(main.rs — HIGH, false-doc-claim + correctness)*

I wired the kinetic chain into the spear follow-through earlier this
session and wrote a test that asserted it was *silent at release*. That
test was encoding a bug, not a spec.

The windup ends at **+0.28 rad**. The follow-through sampled the chain's
tip segment from zero — but the tip's own onset offset is 0.125 s, so
`chain_segment_scale` returned **0.0 for the first 0.125 s**: a hard snap
to neutral exactly when the motion should be most alive. It then ramped
to **−0.12 rad** — *back toward the coil*, the opposite direction from
the swing. The doc comment said "keeps unwinding… carries past."

Fixed: the follow-through now starts at the exact release yaw
(`SPEAR_RELEASE_YAW`, derived from the coil constants so it cannot drift),
samples the tip from *its own* onset, carries **past** the release angle,
then relaxes to neutral. Also added a `release_t < 0` sentinel so a
fighter who merely *holds* a spear isn't born mid-unwind at 20° of twist,
and reset it on death so respawns don't replay it.

**2. `boom_recover`'s k=90 spring governed ALL boom growth**
*(main.rs — HIGH, correctness)*

Also mine, from the same session. I wired `SPRING_K_CAMERA_BOOM` into
collision recovery — but `allowed` defaults to the *full free-space
length*, so the spring filtered **every** boom increase, not just recovery
from cover.

Measured consequences: the sprint boom-out reached 90% at **~0.55 s**
instead of its documented ~0.25 s; ADS release took **0.48 s** instead of
0.12 s; and plain vertical mouse-look became **asymmetric** — pitching
down lagged ~18 cm behind while pitching up snapped instantly.

Fixed via a new pure `boom_step`. The key insight the first attempt
missed: *"boom is shorter than the free-space target"* is equally true
while recovering from a wall **and** while an eased target is simply
growing — a distance comparison cannot tell them apart. It now tracks
occlusion as **explicit state**, so the spring governs only the
clear-the-corner pop it exists for. My own first fix attempt failed its
own new test, which is exactly what the test was for.

### Mech

**3. `mech_transition_t` disarmed the on-foot pilot** *(HIGH, correctness)*
The 1.6 s entry timer blocked firing unconditionally, but neither teardown
path cleared it. Dismount at t=0.1 s, or get blown out of the chassis
mid-boarding, and you were a soldier who **could not shoot for 1.5 s**
with nothing in the HUD explaining why — the eject case leaves you at 25 HP.
Fixed: the gate is scoped to actually being in a live chassis, and both
teardown paths clear the timer.

**4. `MECH_EXIT_S` was a dead constant** *(HIGH, unwired-feature)*
The comment said "boarding/leaving the mech is COMMITTED… real seconds to
seal up **or power down**." Boarding cost 1.6 s; leaving was a single-tick
state flip. `MECH_EXIT_S` had **zero consumers in the entire crate** — the
compiler's unused-constant warning was the symptom. Dismount is the mech's
escape hatch, so this was a real balance asymmetry.
Fixed: exit now sets the timer, defers teardown to the end of the window,
and blocks firing throughout.

**5. A crouching mech kept its full 3.03 m hitbox** *(HIGH, correctness)*
`height()` returns the chassis height unconditionally for a live mech, but
nothing stopped a mech from crouching — the renderer played the full
soldier squat. The ×2.0 visor weak point ended up **hittable in empty air
above the model and unreachable on it**.
Fixed: a mech no longer crouches.

### Spear

**6. One spear landed two attacks** *(HIGH, correctness)*
`try_fire` refused to start a throw during a live thrust, but nothing
guarded the reverse — holding melee inside a throw windup started a thrust
too. Fixed with the mirror guard, plus a regression test.

### Motion

**7. Roll-settle broke the head hit-band** *(HIGH, correctness)*
The 5.5 cm post-roll dip was applied **after** `gait_pose` returned, so
`head_base_y` — the function the band test samples — literally could not
see it. The rendered head base dropped to **0.79** of height while the
sim's Head band starts at 0.82: a head you can see but the sim classifies
as Arms. Worst-case margin was already only **2 mm**.
Fixed: the dip moved *inside* `gait_pose`, and is now clamped **by** the
band rather than checked against it afterwards. Band test extended to
sweep `settle`. Honest tradeoff: at a hard run lean the dip is clamped to
nearly nothing; a deeper absorb would need a compensating torso raise,
which is a pose change, not a constant — noted rather than guessed at. A
second test guards that the clamp didn't silently delete the feature.

**8. `land_rebound` could never lift the camera** *(MEDIUM, false-doc-claim)*
Task 3 rule 5 says landings never fully damp in one frame. The rebound ran
*simultaneously* with the dip, started ~23× smaller, and decayed faster —
so it could only ever *shrink* the dip, never cross neutral. The rule was
inert.
Fixed: `landing_offset` samples both curves from one clock, with the
rebound as a **delayed** counter-push that provably crosses zero.

**9. `grip_fidget` was dead code** *(MEDIUM, unwired-feature)*
Written, tested, and listed in the living-motion comment as live — but it
never reached a `Transform`. Wired into the weapon root, suppressed while
the fighter is doing anything committed.

### Minigun

**10. "idle crawl at rest" was impossible** *(MEDIUM, false-doc-claim)*
The doc named three states; the code produced two. `rate` was purely
proportional to `spin_t`, which the sim pins to exactly 0.0 at rest, and
an early return killed it — so a **resting** minigun looked identical to a
**vent-frozen** one, the two states the comment distinguishes.
Fixed: a real idle crawl constant.
*(The verifier corrected the finder here: the finder also claimed spin-up
was a "hard 0→blur pop", which was false — the ramp was fine. Only the
at-rest clause was wrong, and only that was changed.)*

**11. The crosshair never showed minigun heat** *(MEDIUM, inconsistency)*
`GunSpec.spread` holds only the **cold** value; `try_fire` overrides it
with the heat-widened cone. The stability bracket read the spec directly,
so across a full heat cycle the real cone nearly **tripled** (1.2°→3.5°)
while the bracket moved **zero pixels**. The widening cone *is* the
minigun's entire cost model.
Fixed: a shared `base_spread(kind, heat)` used by both sim and client, so
they cannot drift.

---

## Refuted (4)

Not reported as defects — the verifier read the code and the claims did
not survive. This is the process working as intended.

---

## Live verification

`JK_CAPTURE=minigun_check` — a new capture script. Confirmed on a real
launched build: minigun equips, fires, **HEAT climbs to 74%**, ammo drains
400→351, tracers render, hits land.

Two capture-harness bugs found and fixed along the way:
- The first-run tutorial card was silently covering the mech in *every*
  `mech_scale` shot, including ones already committed to the handback.
- A capture subject holding the trigger dies mid-script (firing clears
  spawn protection — correct game behavior, fatal to a weapon capture).
  Added a keep-alive for weapon-feel scripts. The first version only
  restored health and still showed `DOWN`, because `alive()` is
  `respawn_t <= 0 && health > 0` — the death was already latched.

---

## Still open

Everything in the standing deferral list is unchanged. Notably the four
remaining `SPRING_K_*` constants are still unwired (each needs new
per-fighter spring state), and the spear **viewmodel** still teleports at
release (30 cm + 31° in one frame) — that finding was confirmed but the
fix belongs with the first-person viewmodel work rather than bolted on
here.
