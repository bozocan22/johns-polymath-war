# ANTI_PATTERNS — named failures, with sources

Per the master brief §1.4: collect the failures practitioners have NAMES
for; they become grep targets in the test suite. Inherited names first,
then this session's additions. A name with no source row is folklore and
does not belong here.

## Inherited (from the briefs)

| Name | What it is | Guarded by |
|---|---|---|
| the mannequin spin | whole body snaps to a new facing as one rigid piece | `step_leg_yaw` turn-in-place + its convergence test |
| the wall stop | motion killed instantly on contact, no absorb | boom pull-in is instant BY DESIGN for the camera; body motion eases |
| the ice skater | feet slide under a body that isn't striding | distance-driven `rig.phase` gait |
| the switch flip | a "power move" with no load phase | `spear_windup_is_committal`, roll load phase, `a_counter_movement_dodge_launches_harder` |
| the floating gun | weapon detached from body dynamics | weapon-mass lag (`weapon_lag`), carry sockets, cheek weld |

## Added this stretch (each caught live in THIS codebase)

| Name | What it is | Where it was found | Guarded by |
|---|---|---|---|
| the loyal ghost | state that survives a transition it should die with | cook_t across grenade switch; bot los_time, player reload_t/switch_t/fire_cd/bloom across respawn; intro UI into the match; mech timer onto the ejected pilot | respawn-reset tests, `a_respawned_fighter_can_fire_immediately` |
| the confident narrator | a doc/comment claiming a consumer or mechanic that does not exist | thrusters advertised 2 briefs after deletion; five "wired" spring constants with one call site; `torso_aim_offset` tested and never called | the wave audits; docs now name their consumers |
| the split brain | client re-deriving what the sim computes, then drifting | minigun crosshair, AWM bracket, bow preview v0, BOTH mouse-map labels | shared single-source fns (`base_spread`, `aim_spread_of`, `mouse_map`) + label-matches-binding test |
| the one-way mirror | a guard existing in one direction but not its reflection | throw-blocks-thrust but not thrust-blocks-throw; player mech crouch ban but not bots' | mirrored guards + parity tests |
| the shrinking-list index | indexing a collection that swap_removes mid-loop | axe sweep vs the horde (sim panic) | id-lookup discipline (`blast_zombies`), packed-horde sweep test |
| the backward jump | fake/predicted object teleported to truth instead of lerped | (netcode; not yet applicable here) | recorded from S-02 (grenade/Reitich) for future netplay |
| the checkmate dodge | a dodge with neither i-frames nor positional escape → forced damage | (design guard, not a bug here) | roll ducks the head band + speed burst; Wagar (traversal S-01) |
