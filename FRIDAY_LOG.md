
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
