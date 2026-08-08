
## §21 — the heavy chassis kneels and jumps (sim half)

**FRIDAY22.** `engine/crates/jk_tdm/src/sim.rs` only. Tests 321 -> 328,
0 failed. 13 mutations applied and each one killed a named test.

CROUCH: the ban in `set_crouch` was papering over `height()` returning the
standing 3.03 m for a kneeling chassis, so the x2.0 visor band floated
above the model. Fixed at the cause — `height()` now follows the pose
(`chassis_kneeling`), which moves the collision capsule and the damage
bands together and leaves the visor at the same 0.90 of the machine.
Gated grounded-only; the light chassis still refuses (its `height()` has
no crouch term). Kneel and brace are ONE planted pose and are charged
`MECH_BRACE_SPEED_MULT` once, never twice.

JUMP: `Compress` (0.40 s readable wind-up) -> launch at
`mech_jump_speed()` = `JUMP_SPEED * sqrt(MECH_SCALE)` -> `Air` ->
`Recover` (0.55 s kneeling lockout). Paid out of `stride_heat`, at launch
only, so jump and power stride compete for one budget. Landing shocks
enemy infantry inside 4.5 m with linear falloff plus a flat stagger;
teammates and other chassis are spared.

Published for the client: `TdmSim::mech_jump_phase_of`,
`TdmSim::mech_jump_compression_of`, `MechJumpPhase::label`,
`Fighter::mech_jump_compression`, `Fighter::chassis_kneeling`,
`Fighter::visor_eye_y`, `Fighter::in_heavy_mech`.

Handed to FRIDAY33: `main.rs:17205` still calls the stance-blind
`mech_visor_eye_y(p.pos[1])` and should call `p.visor_eye_y()`;
`main.rs:17359` adds `MECH_BRACE_STANCE_DROP` on top of a hull the sim now
sinks itself, so the braced camera will double-drop.

## §C — armour damage states: Fresh / Scuffed / Cracked / Severed (sim half)

**FRIDAY22.** `engine/crates/jk_tdm/src/sim.rs` only — purely additive,
950 lines in, 0 removed. Tests 328 -> 338 mine (suite read 354 at the end
because FRIDAY33 was landing cockpit tests in the same run). 0 failed.
11 mutations applied; every one killed a named test, each reverted from a
file copy, never `git checkout`.

THE BLOCKER was that the hit path resolved a ZONE and per-piece HP needs a
PIECE. Rejected: (1) a seeded roll per hit — inserts an RNG call into the
hottest path, moves every later draw, breaks bit-identical replay for
every other system; (2) spreading wear over the whole zone — ten leg
plates then degrade in lockstep and fall off together, which is a
whole-zone armour bar in a per-piece costume, and the brief's own worked
example (one pauldron gone, an asymmetry readable at range) cannot happen.
TAKEN: ask the geometry the hit already carries. `frac` is already
computed to pick the zone and the impact point is already passed in to
place the damage number; between them they say how high, which side and
which face — which is exactly a plate. `ArmorPiece::struck` cuts each
zone's band into its anatomical layers (shoulder to hand, hip to boot) and
picks the mirrored half by side, or the cuirass by face. No state, no RNG,
no second geometry pass.

Every number is the brief's: 80-point pools, 70/40/15% thresholds, +5%
and +10% resistance. The one number it does not give is the WEAR RATE,
labelled ASSUMED at `wear_plate`: a plate takes what the fighter took, so
the baseline rifle scuffs it on the 2nd round, cracks on the 4th, severs
on the 6th and strips it on the 7th.

DETACH IS THE UNEQUIPPED PATH, not a new one:
`armor_pieces.set(p, false)`, the same bit the Forge switch clears, so
`zone_mult` exposes the segment with nothing to keep in step.
`a_shot_off_plate_is_indistinguishable_from_one_never_worn` is the test
that notices if that ever grows a second flag.

Player/bot is ONE rule — the wear is in `apply_hit_dmg`, which both paths
already share. Mutating it to fire only for `self.player` killed 5 tests.

Condition SURVIVES respawn, deliberately and against the house rule: §C
repairs plate in the Forge at 50 points a piece, between matches. Dying is
not a repair. Pinned by `plate_condition_survives_a_respawn` so it reads
as a decision, not as the recurring "state survived a transition" defect.

Published for the client: `TdmSim::armor_stage_of` (returns `Option` —
`None` is a bare mount, and a client must not draw clean steel on a naked
shoulder), `TdmSim::armor_wear_of`, `ArmorStage::label`,
`ArmorStage::tilts`, `ArmorStage::resist`, `ArmorPiece::struck`,
`HitZone::band`, `ArmorCondition::{hp,frac,stage,wear,repair}`.

Handed to FRIDAY33: nothing in `main.rs` reads any of the above yet — the
plates render at one appearance regardless of condition, and "piece tilts"
at Cracked is unbuilt. Deferred by me: plate wear fires only where plate
PROTECTION already fires (the zoned hitscan path), so grenades, melee,
claws and gas still neither wear plate nor are reduced by it — one gate,
`in_mech`, for both, so they cannot drift.

Found in my lane, NOT fixed, needs a decision: `sim.rs:8165` and
`sim.rs:11084` compute the muzzle as `pos[1] + EYE_REL.min(height()-0.12)`.
That `.min()` only ever bites for a SHORT fighter, so a 3.03 m chassis
fires from 1.62 m — knee height on the machine, 1.10 m under its own visor.
Changing it would move every mech engagement, so it is reported rather
than done.
