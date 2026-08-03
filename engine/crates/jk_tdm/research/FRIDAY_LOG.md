# Friday's log — the implementation agent

Append-only. What was built, what broke, what was deferred and why.
Written for the next Friday, who will have no memory of today.

Standing rules live in `.claude/agents/friday.md`.

---

## Inherited context — the codebase Friday is walking into

**Scale:** `jk_tdm` is ~22,700 lines across `sim.rs` (deterministic sim)
and `main.rs` (Bevy client). 145 tests, all green.

**Determinism is enforced, not aspirational.** Fixed 120 Hz tick, seeded
PCG32, and tests that compare 1000 grenade throws raw-bit and replay a
30-shot spray bit-identically. Anything added to `sim.rs` becomes replay
state.

**Repeated defect classes already burned by this project — do not
re-create them:**

1. **Player/bot parity.** The mech turn rate and the movement
   acceleration model BOTH shipped bot-broken first: the human paid a
   commitment cost the bot did not. Prefer one shared function.
2. **"The wall stop."** Velocity was written raw from input, so releasing
   a key stopped you dead in one tick. Fixed by `approach_velocity`
   (sim.rs) — but the doctrine says hunt it *everywhere*, and landings
   and other state transitions have not all been swept.
3. **The confident narrator.** Doc comments that claim a consumer which
   does not exist, or imply a dependency that algebraically cancels.
   Both have been found here. Say what the code does, not what it was
   meant to do.
4. **Self-referential tests.** A test that rebuilds the expression it
   claims to verify cannot fail. Assert against an independent table or
   a hand-computed value.
5. **Stale test setups.** When a test breaks, ask whether the ASSERTION
   is wrong or the SETUP was written for a world that no longer exists.
   Fixing a stale setup is legitimate and must be stated; weakening a
   correct assertion is falsification.

**Conventions:** tunables go in hand-rolled `key = value` text files
(`config/camera_tuning.txt`, `config/settings.txt`) — deliberately no
serde/RON dependency. Capture scripts for live verification: `baseline`,
`traversal`, `map_lap`, `mech_scale`, `minigun_check`, `idle_life`,
`bow_draw`, `menus`.

**Kill the running binary before rebuilding** — it locks the exe.

---
