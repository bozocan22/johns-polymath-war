# GAME STATUS REPORT — John's Kingdom (Polymath Game)

**Report date:** 2026-08-01 · **Repo:** `bozocan22/johns-kingdom-polymath` (public) · **Scope:** every crate in `engine/`, every design brief, every research ledger. Verified against actual code — every claim below traces to a real file/function, not a memory of what was planned.

This project is **two complete games sharing one deterministic 120 Hz core** (`jk_core`): a large-scale medieval **battle simulator** (the "Shieldwall" wall-fighting game) and a **third/first-person arena shooter** (TDM/KOTH/Extraction). They share almost nothing except the fixed-timestep engine and seeded RNG — different sims, different clients, different genres, same underlying determinism discipline.

---

## PART 1 — THE ENGINE CORE (`jk_core`)

Both games run on the same tiny, shared foundation:
- **Fixed 120 Hz timestep** (`timestep.rs`) — an accumulator with an 8-step spiral-of-death clamp, plus a render-side `alpha()` interpolation hook so visuals stay smooth between fixed ticks.
- **Deterministic RNG** — a hand-rolled PCG32, never the OS's random source, never a UI-thread timer. Every dice-roll in both games traces to a seeded stream, which is what makes bit-identical replay possible at all.
- **Calibrated constants discipline** — physical numbers in the wall-sim are tagged `SOURCED` (traced to a real citation) or `PROVISIONAL` (best estimate, flagged as such), never silently invented.

---

## PART 2 — THE BATTLE SIMULATOR (`jk_wall` + clients `jk_bevy`/`jk_client`/`jk_spike`)

**Not touched this session — confirmed unchanged** (identical source line counts to the last full audit, 2026-07-25). Everything below is still current.

### What it is
A large-formation medieval combat sim: you command one soldier inside a shieldwall of dozens to hundreds, giving squad orders while the crowd itself is real physics — not animation, not a script. Every man is a Rapier capsule body with a shield collider; pressure, pushing, and crush injuries are *emergent* from soft-contact chains, not scripted "wall wins" logic.

### Character model — one archetype, procedurally varied
No classes, no levels. Every soldier is rolled at spawn from the seeded RNG:
- Body mass 62–82 kg + armor + 4.5 kg gear (shield 3 kg, spear 1.5 kg)
- Armor: front rank 35% mail, other ranks 10% mail, remainder gambeson/cloth
- Weapon: 70% spear / 20% sword / 10% axe
- Aerobic power ceiling 255–345 W, anaerobic reserve 57–78 kJ
- No hit points — **45 joules of penetrated energy** downs a man (a real physics quantity, not an HP bar)

### Weapons (energy-vs-armor, not damage-vs-HP)
| Weapon | effective mass | strike velocity | reach | full-effort energy |
|---|---|---|---|---|
| Spear | 0.6 kg | 7.5 m/s | 1.9 m | 16.9 J |
| Sword | 0.9 kg | 10.5 m/s | 1.1 m | 49.6 J |
| Axe | 1.4 kg | 12.0 m/s | 1.3 m | 100.8 J |

Armor absorbs energy before it wounds: cloth defeats at 5 J, gambeson at 30 J, mail at 100 J. A javelin lands ~126 J at full 19 m/s launch — enough to defeat mail, which is why volleys matter tactically.

### Systems that are real and working
- **6 squad orders** (Advance/Hold/Brace/Charge/Withdraw/Rotate) — each reweights actual physics levers (speed, spacing, push authority, brace stiffness), not stat multipliers
- **Two-pool stamina** (aerobic free, anaerobic drains and recovers on a curve) gates strike power and block chance
- **Morale/fear/rout/rally** — fear from witnessing allies fall, being flanked, being outnumbered, or losing your commander; routing men can't fight, they flee, and gaps cascade
- **Cohesion/breach detection** — real gap measurement between file-mates decides when a line is actually broken
- **Javelin volleys** — ballistic arcs, deterministic scatter, auto-volley at 14 m closure
- **Two render clients**: a full Bevy 3D client (procedural rigged soldiers, sim-driven animation, spring camera) and a zero-dependency macroquad fallback
- **A benchmark harness** (`jk_spike`) — verified scale: 80 bodies @ 29× realtime down to 1024 bodies @ 1.8×, single-threaded

### What's explicitly NOT built (honest, from the last full audit)
Kingdom building, economy, blacksmith/forge gameplay, raids/campaign, terrain (flat plane only), archery, additional formation shapes (wedge/square), weapon hit-location, shield geometry/durability, multiplayer netcode, save/load. All are named in the roadmap as future milestones, not silently dropped.

---

## PART 3 — THE ARENA SHOOTER (`jk_tdm`) — the focus of this session's work

**22,744 lines** across `sim.rs` (11,198) and `main.rs` (11,546) — roughly **triple** the size it was at the last full audit. Everything below is freshly verified against the current code.

### Core loop
Third-person-primary, first-person-when-aiming COD/CSGO-style arena shooter. 100 HP, four hit zones with real multipliers (head ×4.0, torso ×1.0, arms/legs ×0.75), zone selection by hit-height fraction on the target's body (not a hitbox mesh — a real geometric fraction check). Bots fill empty slots with three difficulty tiers, differing only in aim variance, reaction time, engagement range, and aggression — never in what rules apply to them.

### Game modes
- **TDM** — team deathmatch, first to a score target
- **KOTH** — king of the hill, a capturable zone
- **Extraction** — co-op PvE: insert, survive a zombie horde, extract with what you're still carrying; gear is lost on death, which is the whole tension of the mode

### Maps (4)
- **Dust Arena** — the original range: plateaus, stairs, a center tower
- **Castle Bailey** — central keep, corner drum towers, bailey walls
- **Castle Gardens** — hedge lanes, ruined walls, trees, a stone gazebo
- **Battlefield** (400×400 m) — keep, forge district, cathedral ruin, a river with bridges, corner watchtowers to 36 m; the vertical-scale map built for the mech

All four have been measured with a real sightline-validation tool (`max_unobstructed_sightline`) against the design brief's 40 m rule — none currently pass (worst-case sightlines run 80–510 m), a known, tracked gap.

### Weapon roster (11 guns + fists, exact current numbers)
| Gun | class | fire period | mag | damage | notes |
|---|---|---|---|---|---|
| Unarmed | — | 1.0s | — | 0 | — |
| Glock 17 | Secondary | 0.16s | 17 | 9.0 | — |
| Desert Eagle | Secondary | 0.42s | 7 | 27.0 | the one-tap-head pistol |
| MP5 | Primary | 0.08s | 30 | 10.0 | best move-accuracy, an SMG's whole point |
| Remington 870 | Primary | 0.95s | 7 | 6.5 ×8 pellets | ~52 dmg point-blank |
| AK-47 | Primary | 0.105s | 30 | 13.5 | hits harder, kicks harder |
| M4A1 | Primary | 0.09s | 30 | 12.5 | the baseline: 2 headshots or 8 body shots |
| AWM | Special | 1.455s | 5 | 115.0 | Valve's own AWP numbers, head always one-shots |
| M249 | Primary | 0.075s | 100 | 11.0 | — |
| War Bow | Special | 0.95s (draw-based) | 1 | 34 flat, no zone mult | 55 m/s full draw, full mechanic below |
| War Spear | Special | 1.3s | 1 | 85 flat, ×2 head/×0.75 legs | thrown, full mechanic below |
| M134 Minigun | Special (pickup-only) | 0.06s (1000 RPM) | 400 | 8.0/round | spin-up/heat/vent tradeoff, never in a loadout |

Recoil is a real **three-channel CS:GO-derived system**, not decoration: the bullet's true deflection, the camera's view (which only shows 45% of the true kick — the crosshair never moves), and the viewmodel's cosmetic rotation are three separate numbers. Deterministic per-weapon spray patterns are generated from a fixed seed at load, so every player learns the same, real, memorizable pattern, and replays reproduce every bullet bit-for-bit.

### The bow — a full draw-and-release mechanic, not a hitscan reskin
Hold to draw (0.7s to full power, power scales 35%→100% linearly with hold time), release before 0.15s = silent letdown, hold past 4s and **aim sway ramps from ±0.4° to ±1.2° over the next 4 seconds** (crouching halves it), forced letdown at 10s if you just stand there. Pierces up to 3 soldiers (90→68→45 damage per soldier). Arrows are recoverable pickups.

### The spear — a full throw mechanic with a kinetic-chain wind-up
Windup → plant → hip-drive → whip → release, sequenced through a shared proximal-to-distal timing curve also used for sprint-start head lag and dodge launches. **A throw released with ≥2 full strides of running momentum behind it launches 15% faster** — real reward for committing to a run-up rather than standing still. Spears that hit at ≥30° stick and become a retrievable pickup; shallower hits bounce and are lost.

### Grenades (4 types) — a real analytical trajectory solver
Frag, Flash, Smoke, Molotov. The throw arc preview uses the **exact same physics integrator** as the actual thrown grenade — never a second, divergent approximation, which is the single most common bug class in games with grenade previews. Bounces are **per-surface material-aware**: stone (0.40 restitution, 0.30 friction — skids far) versus a wooden crate (0.50 restitution, 0.45 friction — grips more) versus hedges/trees (stick on contact, zero bounce). Verified bit-identical across 1,000 seeded throws and confirmed the preview matches the actual flight across 200 random throws to sub-millimeter accuracy.

### Armor sets (5)
- **None** — no ability, baseline everything
- **Folk** — mail and plate; Shieldwall Brace (hold C): plant, shielded, slow, damage-resistant
- **Pyro** — heat plate, full fire immunity, Flame Projector (hold C)
- **Robot Suit (the mech)** — see below, its own major system
- **Recon** — light counterweight: fast, quiet, self-healing, no active ability

### The mech (Robot Suit) — the single most-developed system this session
A grounded walker, 1.7× soldier scale (deliberately compressed from concept art's 1.15×, to keep it sharing doors/cover/pathing with infantry). Flight was **deliberately deleted** — a documented decision, not a cut corner.
- **Committal entry (1.6s) and exit (1.2s)** — no cancel, staged into 8 named beats (cockpit open → climb-in → harness → power-up → servo sync → gyro calibration → weapon diagnostics → HUD boot), grounded in real aviation checklist-design research on why interruptible sequences fail
- **Power stride** — hold sprint, the hull winds up 0.35s, then commits to a 2.5-second burst at 110% speed, locks the missile pod, halves turn rate, costs a heat budget that needs a *full* cooldown before it can fire again (a bug caught by its own test before shipping)
- **Angle-based armor**: front 120° takes 15% damage, sides 30%, rear 90% takes full damage — the rear is the kill zone, by design. The visor is a weak point at ×2.0 *on top of* the angle multiplier
- **Plates physically detach at HP thresholds** (70%/40%/15%) — shoulder pod and knee plate shear off first, then side skirts, then the chest chips and the gait visibly limps; stripping a mech is something the attacker can *see happen*, not just a number dropping
- **Missile pod**: lock-on, but explicitly **cannot lock infantry** — dumb-fires straight at soldiers instead, a documented "anti-oppression" design rule the brief itself calls out as non-negotiable
- **A full, currently-unbuilt "climb onto the hull" mechanic has a complete design doc** (grip points reuse the plate-detach zones, stamina-gated grip grounded in real climbing-fatigue research) waiting for its build cycle

### HUD
Bottom-left vitals (health/armor with color thresholds), bottom-right ammo + loadout strip, top-left minimap (self as a white arrow, teammates as colored dots, **spotted enemies as red dots that ghost-fade to their last-known position** using the game's real line-of-sight query, not a cheat-vision always-on radar), top-center timer/score, top-right killfeed, a full crosshair, and state overlays (flash-blind, damage direction, context prompts). Large parts of this section are still being filled in — see the audit below.

### Character customization / "The Forge"
Currently minimal: a hat color and a tunic color, saved to 3 flat text-file slots via hotkeys. The brief specifies a much larger system (26-piece armor, per-piece weight/damage progression, a real visual editor with a category grid and turntable preview) — this is one of the largest confirmed-but-not-yet-built systems in the game.

### Determinism & testing discipline
**141 automated tests**, all passing. The sim is proven to replay **bit-identically** — same seed, same inputs, same result down to the raw floating-point bits, verified for 1,000 consecutive grenade throws and a full 30-shot weapon spray. A dedicated capture harness drives scripted input sequences against the actual launched build and saves real screenshots as evidence — nothing in this report about "it works" rests on an assumption; it rests on either a passing test or a real captured screenshot.

---

## PART 4 — THIS SESSION'S WORK (2026-08-01)

### Repository migration
Moved the game's home to **`bozocan22/johns-kingdom-polymath`** (public), with its full, real commit history preserved via a git subtree extraction — not a fresh start. This VS Code workspace is connected exclusively to that account.

### "Thor" — a persistent verification role
Set up a named, standing verification agent with its own durable log (`research/THOR_LOG.md`). Its first action was a 152-agent audit workflow that checked every concrete claim in all 9 design briefs against the actual running code — not "does this look done," but "read the code, find the exact line, prove it." Result: **143 real findings**, all logged with file/line evidence.

Along the way it caught a real process failure in itself: 46 of its own verification passes hit a session rate-limit mid-check, and the workflow's automatic bucketing would have silently filed those as "false alarm" instead of "never actually verified." Caught and corrected before trusting the result — the same class of error this session has now caught twice.

### Shipped this session (built, tested, pushed)
1. **Sprint-out fire gate** — sprinting and instantly firing is no longer free; a per-weapon-class beat has to pass first
2. **Empty-reload time cost** — running a gun dry costs 35% longer to reload than a tactical reload
3. **Power stride** (mech) — described above
4. **Mech entry staging** — the 8 named boarding beats, described above
5. **Per-surface grenade friction** — described above
6. **Spear running-throw bonus** — described above, and its test caught a real conflict with the sprint-out gate before it shipped
7. **Minimap enemy spotting** — described above
8. **Bow full-draw sway** — described above

### Honest remaining scope (not done, not hidden)
- **~95 more small, real, individually-scoped findings** from Thor's audit — mostly HUD completeness (crosshair settings, killfeed team-colors/border, scoreboard columns, death→spectate camera flow), numeric corrections, and test-coverage gaps for mechanics that already exist. All tracked in `BACKLOG.md`.
- **Several genuinely large, multi-session rebuilds**, deliberately *not* rushed:
  - A real 20-segment, mass-driven body rig (current rig has ~14 simple transforms; the brief wants a true pelvis/lumbar/thorax trunk with real mass/inertia data)
  - A full mech visual and weapon-kit rebuild ("walking weapons platform" silhouette, gatling+autocannon replacing the missile pod as the core kit)
  - A real Forge editor UI (currently 3 hotkey slots, no visual editor)
  - The 26-piece armor + 4-class character system, and the castle map's actual content build
  - The mech hull-climbing mechanic (design complete, build queued)

---

## APPENDIX — where to look for more

- `BACKLOG.md` — the full ranked list of every outstanding item, with its blocker named if it's blocked
- `THOR_LOG.md` — the complete, timestamped record of every verification action this session
- `research/*/SOURCES.md` — every external source actually read (not just cited) for every design decision, with honest READ/SNIPPET-ONLY/UNREACHABLE status per source
- `briefs/` — the original design documents this whole build traces back to
