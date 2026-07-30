# GAME STATUS REPORT — John Kingdom Game

**Audit date:** 2026-07-25 · **Branch:** `claude/shieldwall-reforged-research-k2zm28` · **Scope:** every `.rs` file in `engine/` (12,752 lines across 6 crates), design docs, and the shieldwall_reforged research corpus. Analysis only — no code changed.

All paths below are relative to `projects/john_kingdom_game/`. Line numbers are from the audited revision.

---

## 1. GAME INVENTORY — what exists so far

The project is **two games on one deterministic 120 Hz core**:

- **The battle sim** (`jk_wall` + clients `jk_bevy`, `jk_client`, harness `jk_spike`) — the Shieldwall-inspired formation game. This is the project's stated "crown jewel."
- **The TDM arena** (`jk_tdm`) — a self-contained third-person/first-person shooter side-mode with its own sim (`jk_tdm/src/sim.rs`), sharing only `jk_core` (timestep + RNG).

### 1.1 Implemented systems

| System | Status | Where | How it works |
|---|---|---|---|
| Fixed timestep (120 Hz) | DONE | `engine/crates/jk_core/src/timestep.rs:8-51` | Accumulator with 8-step spiral-of-death clamp; renderer interpolation hook (`alpha()`). |
| Deterministic RNG | DONE | `engine/crates/jk_core/src/rng.rs` | Hand-rolled PCG32; separate seeded streams for spawn (`0x5EED`) and combat (`0xC0B47`) (`jk_wall/src/sim.rs:148,315`). |
| Calibrated constants (single source) | DONE | `engine/crates/jk_core/src/constants.rs` (226 lines) | Every physical number tagged `SOURCED`/`PROVISIONAL` per ADR-004. |
| Wall physics (push, crush, cohesion) | DONE | `engine/crates/jk_wall/src/sim.rs:360-1016` (`step()`) | Rapier capsules + enemy-only shield colliders; lateral PD slot-holding, forward velocity-servo that saturates into othismos push; brace degradation; crush injury; α is measured, never authored. |
| Squad commands (6 orders) | DONE | `engine/crates/jk_wall/src/command.rs:58-116` | Advance/Hold/Brace/Charge/Withdraw + timed Rotate drill; each order reweights speed/spacing/push/lateral/brace/power — physics levers, not stat toggles. |
| Rank rotation drill | DONE | `jk_wall/src/sim.rs:361-419, 500-530` | Per-file stamina comparison assigns roles; diagonal pass against a captured front-plane anchor; auto-reverts after 6 s (`command.rs:38`). |
| Two-pool stamina | DONE | `engine/crates/jk_wall/src/stamina.rs:27-43` | Aerobic ceiling free; excess drains 67.5 kJ anaerobic pool; sqrt output curve to a 0.35 floor. |
| Melee combat (energy vs armor) | DONE | `engine/crates/jk_wall/src/combat.rs:129-163`, `sim.rs:800-878` | Strike = kinetic energy; armor demands energy (Williams data); penetrated J accumulates to 45 J knock-down. Block is state-based probability. |
| Morale / fear / rout / rally | DONE | `jk_wall/src/sim.rs:895-1000`, constants `constants.rs:131-165` | Fear from witnessed falls, compression, outnumbering; commander aura ×4 recovery; per-man rout tolerance; rally at fear < 0.45. |
| Javelin volleys | DONE | `jk_wall/src/sim.rs:675-798, 1154-1219` | Ballistic integration, deterministic scatter, auto-volley at 14 m closure, 2 per man; player aimed throw. |
| Player-in-the-crush (third person) | DONE | `jk_wall/src/sim.rs:469-489`, `command.rs:119-135` | Velocity-servo body force-capped below crowd pressure; take-over-next-man on death. |
| Cohesion / breach detection | DONE | `engine/crates/jk_wall/src/cohesion.rs:32-62`, `sim.rs:1111-1137` | Euclidean gap → overlap ratio Ω; breach = enemy centroid past front plane with no covering defender. |
| Telemetry + α fitting | DONE | `engine/crates/jk_wall/src/metrics.rs` | Per-tick StepMetrics; `fit_alpha` grid-search least squares. |
| Headless spike + battle report | DONE | `engine/crates/jk_spike/src/main.rs` | α sweep depth 1–8, 40v40 run, CSV/PNG/GIF/report to `output/`. |
| Scale benchmark | DONE | `engine/crates/jk_spike/src/bin/bench.rs` | 80→1024 bodies; README-recorded: 80 @ 29×, 256 @ 8.7×, 500 @ 4.5×, 1024 @ 1.8× realtime, single thread. |
| Battle 3D client (Bevy) | DONE | `engine/crates/jk_bevy/src/main.rs` (997 lines) | Procedural rigged men (or GLB drop-ins), sim-driven animation, spring camera with speed-FOV, HUD, pause menu. |
| Battle fallback client (macroquad) | DONE | `engine/crates/jk_client/src/main.rs` (338 lines) | Zero-dep renderer, same sim API. |
| **TDM mode (whole feature set)** | DONE | `engine/crates/jk_tdm/src/sim.rs` (2,886 lines) + `main.rs` (4,581 lines) | 10-weapon loadout shooter: hitscan + projectiles, 4 hit zones, shield stance, dodge roll/breakfall, jump/gravity/step-up on AABB maps, 3 maps, TDM + KOTH, checkpoints, pickups, bots with 3 difficulties, full determinism test suite. |
| TDM client (Bevy) | DONE | `engine/crates/jk_tdm/src/main.rs` | First/third person, articulated chibi rigs with fingered hands, modeled weapons, viewmodel, minimap, scoreboard, settings, procedural WAV SFX. |

### 1.2 Stubbed / referenced but not implemented

| Feature | Status | Evidence |
|---|---|---|
| **Kingdom building** | MISSING | No code anywhere. Roadmap item 14 (`DESIGN_MAP.md:122`) marked 📐 designed-only. |
| **Economy** (income, costs, trade) | MISSING | Fully researched (`../shieldwall_reforged/research/03_economy.md` — charcoal/land/transport model with proposed formulas) but **zero lines of economy code**. |
| **Blacksmith / forge / metallurgy gameplay** | MISSING (socket exists) | `combat.rs:5-8` names the socket ("blades from the forge plug into `Weapon` in M4"); metallurgy constants live only in research (`07_CONSTANTS.md` System B). No forge code. |
| **Raids / campaign / army management** | MISSING | No code; `DESIGN_MAP.md:77` "hierarchy at campaign layer 🔜". |
| Terrain (battle sim) | MISSING | Flat plane only; M5 roadmap. |
| Arrows/archery (battle sim) | MISSING | Javelins only; "arrows (M5)" `DESIGN_MAP.md:101`. |
| Formation shapes beyond the wall | PARTIAL | Brace ≈ tight order; wedge/square etc. 🔜 (`DESIGN_MAP.md:79`). |
| Weapon colliders / hit locations (battle) | PARTIAL | Reach+arc query only (`sim.rs:824-843`); locations 🔜. |
| Shield durability & geometric block cone (battle) | PARTIAL | Block is probability, not geometry (`combat.rs:138-151`). |
| Slash/blunt damage split | PARTIAL | Point-attack thresholds only (`ArmorKind::e_required_j`, `combat.rs:27-33`). |
| Multiplayer | MISSING | Determinism groundwork exists (fixed tick, seeded RNG, bit-identical replays); no netcode. |
| LOD / Jolt physics swap | MISSING | ADR-002 planned trigger: ">300 bodies or client work". |
| glTF skeletal animation playback | PARTIAL | GLB scenes load as static drop-ins (`jk_bevy/src/main.rs:295-302`); no animation playback. |
| Save/load | MISSING | No serialization anywhere. |
| Casualty model from chest injury | STUB | `CHEST_INJURY_FORCE_N` (`constants.rs:33`) declared "used by the (future) casualty model" — **never referenced**. |
| `breach_risk()` sigmoid | STUB | `cohesion.rs:13-15` defined, never called outside its file. |

---

## 2. CHARACTER & UNIT SYSTEM

### 2.1 The battle sim: ONE archetype, procedurally varied

There are no unit *classes*. Every soldier is an `Agent` (`engine/crates/jk_wall/src/agent.rs:32-83`) whose identity is rolled at spawn (`jk_wall/src/sim.rs:154-282`) from the seeded PCG in deterministic order:

| Stat | Value / roll | Where set |
|---|---|---|
| Body mass | uniform **62–82 kg** | `sim.rs:187`, `constants.rs:13` |
| Gear mass | + armor mass + **4.5 kg** (shield 3 + spear 1.5) | `sim.rs:186-189` |
| Armor | rank 0: **35 % mail**; ranks 1+: **10 % mail**; then 50 % gambeson, remainder cloth | `sim.rs:171-185`, `constants.rs:107-109` |
| Weapon | **70 % spear / 20 % sword / 10 % axe** | `sim.rs:239-246` |
| Aerobic ceiling | 300 W × uniform(0.85, 1.15) → **255–345 W** | `sim.rs:235` |
| Anaerobic pool | 67,500 J × uniform(0.85, 1.15) → **57.4–77.6 kJ** | `sim.rs:236` |
| Strike period | weapon `period_mult` × uniform(**2.4–4.0 s**) | `sim.rs:265-266` |
| Initial strike cooldown | uniform(0.5–2.0 s) (staggers openings) | `sim.rs:262` |
| Javelins | **2** | `sim.rs:263`, `constants.rs:118` |
| Rout tolerance (nerve) | uniform(**0.65–1.0**) fear units | `sim.rs:268-269` |
| Crush tolerance | uniform(**8–15 s**) over 4.5 kN | `sim.rs:273-275` |
| Capsule | r 0.22 m, h 1.72 m; shield cuboid 0.9 × 1.0 × 0.2 m, massless, enemy-only collision | `sim.rs:201-233` |
| "HP" | none — **45 J of penetrated energy** downs a man (`wounds_j` accumulator) | `agent.rs:52`, `constants.rs:99` |

**Weapon table** (`engine/crates/jk_wall/src/combat.rs:68-103`):

| Weapon | eff. mass | strike v | reach | period mult | pool cost | full-effort energy |
|---|---|---|---|---|---|---|
| Spear | 0.6 kg | 7.5 m/s | **1.9 m** | 1.0 (→ mean 3.2 s) | 250 J | **16.9 J** |
| Sword | 0.9 kg | 10.5 m/s | 1.1 m | 0.6 (→ mean 1.92 s) | 180 J | **49.6 J** |
| Axe | 1.4 kg | 12.0 m/s | 1.3 m | 1.6 (→ mean 5.12 s) | 380 J | **100.8 J** |

**Armor table** (`combat.rs:27-41`): Cloth — 5 J to defeat, 0 kg; Gambeson — 30 J, 3 kg; Mail — 100 J, 11 kg.

**Stats never change over time** — no levels, no XP, no equipment upgrades. The only dynamic quantities are stamina pool, fear, wounds, and crush exposure. The forge (which would make weapon/armor numbers *earned*) is the missing system these structs were built to receive.

### 2.2 Battle-sim formulas (plain math)

- **Strike energy** (`combat.rs:107-110`): `E = ½ · m_eff · (v · clamp(effort, 0.3, 1.2))²` where `effort = stamina_output · (1 − 0.3·min(overload,1))` (`sim.rs:850-852`).
- **Stamina output** (`stamina.rs:40-43`): `out = 0.35 + 0.65·√(pool/pool_max)`.
- **Stamina drain** (`stamina.rs:27-36`): if `P > aerobic`: `pool −= (P − aerobic)·dt`; else `pool += 0.04 · spare_frac · (pool_max − pool) · dt`.
- **Per-tick metabolic load** (`sim.rs:596-604`): `P = 100 + extra_order_W + 0.9·push_N + 0.1·compression_N` (that last term — `COMPRESSION_POWER_W_PER_N`, `constants.rs:58` — is *why walls rotate*: a 3 kN front press costs ~300 W).
- **Block chance** (`combat.rs:138-151`): `b = 0.55 + 0.25·braced − 0.30·min(overload,2) − 0.20·(stamina<0.5)`, clamped [0.05, 0.95]; routing men block at flat 0.05.
- **Wound** (`combat.rs:155-162`): if `E > E_req(armor)` → `wounds += E − E_req`; at `wounds ≥ 45 J` the man is down (`sim.rs:866-872`).
- **Brace degradation** (`sim.rs:463-466`): `overload = max(0, (compression − 1600·brace_mult)/1600)`; `push_factor = clamp(1 − 0.3·overload, 0.2, 1.0)`.
- **Push authority** (`sim.rs:549-562`): follower (not front, not charging): **250 N flat lean**; otherwise `stamina_out · push_factor · order_push_mult · 1200 N`.
- **Crush down** (`sim.rs:880-894`): `compression > 4500 N` accumulates exposure (decays at half rate below); down when exposure > tolerance(8–15 s).
- **Fear** (`sim.rs:949-999`): witnessed ally down within 6 m: `+0.16·(1−d/6)`; commander falls within 12 m: `+0.30`; enemy down: `−0.06·(1−d/6)`; compression above brace limit: `+0.02/s`; outnumbering: `+0.05·excess/s`; recovery `0.015/s` (×4 within 7 m of standing commander, ×3 while routing). Rout when `fear > tolerance`; rally at `fear < 0.45`.
- **Javelin impact** (`sim.rs:752-757`): `E = ½ · 0.7 kg · |v|²` — at the 19 m/s launch cap this lands ~**126 J**, which **defeats mail** (see §3.4).
- **Cohesion** (`cohesion.rs:7-9`): `Ω = clamp((0.9 − gap)/0.9, 0, 1)` per adjacent front pair.

### 2.3 TDM roster (v6)

Fighter definition: `engine/crates/jk_tdm/src/sim.rs:816-859`. **100 HP** (`MAX_HEALTH`, line 39), zone multipliers **head ×4.0, arms/legs ×0.75** (lines 58-60), zone bands by hit height fraction: head > 0.82, arms > 0.66, torso > 0.35, else legs (`apply_hit`, lines 1792-1801). Movement: walk 4.8, sprint 6.6, crouch ×0.5, ADS ×0.62, scoped ×0.35, shield ×0.55, robot armor ×1.12, roll 8.6 m/s (lines 32-87). Bots differ from the player only via `BotParams` (aim σ, reaction, engage range, aggression — lines 751-772) and a deterministic loadout rotation (lines 1086-1094). Names are cosmetic (lines 1007-1012).

**Full gun table, exactly as in code** (`gun()`, `sim.rs:156-327`). `dmg` is base torso damage/bullet; zone multipliers apply on top; projectile weapons (bow, spear) use flat damage with **no zone multiplier** (`step_missiles`, line 1989: "projectile damage: flat"):

| Gun | class | fire s | mag | reserve | reload s | spread | spread_move | kick | dmg | pellets | projectile (v0, dmg) | zoom° |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Unarmed | — | 1.0 | 0 | 0 | 2.0 | 0 | 0 | 0 | 0 | 1 | — | 62 |
| Glock 17 | Sec | 0.16 | 17 | 68 | 1.3 | .010 | .016 | .0025 | 9.0 | 1 | — | 55 |
| Desert Eagle | Sec | 0.42 | 7 | 35 | 1.6 | .006 | .022 | .008 | 27.0 | 1 | — | 52 |
| MP5 | Pri | 0.08 | 30 | 150 | 1.8 | .011 | .015 | .003 | 10.0 | 1 | — | 52 |
| Remington 870 | Pri | 0.95 | 7 | 28 | 2.8 | .055 | .020 | .010 | 6.5 | **8** | — | 56 |
| AK-47 | Pri | 0.105 | 30 | 120 | 2.2 | .011 | .024 | .0055 | 13.5 | 1 | — | 48 |
| M4A1 | Pri | 0.09 | 30 | 120 | 2.0 | .008 | .018 | .004 | 12.5 | 1 | — | 48 |
| AWM | Spc | 1.6 | 5 | 20 | 3.0 | .0012 | .050 | .015 | 70.0 | 1 | — | 16 (scoped) |
| M249 | Pri | 0.075 | 100 | 200 | 4.5 | .016 | .032 | .0055 | 11.0 | 1 | — | 52 |
| War Bow | Spc | 0.95 | 1 | 24 | 0.9 | .004 | .018 | .002 | 34.0 | 1 | (38 m/s, 34) | 45 |
| War Spear | Spc | 1.3 | 1 | 5 | 1.1 | .006 | .015 | .003 | 55.0 | 1 | (17 m/s, 55) | 50 |

Other TDM combat math:
- **Spread** (`try_fire`, lines 1659-1666): `spread = base + (moving ? spread_move : 0) + bloom`, then ×0.32 ADS, ×0.55 crouch; bloom grows by `kick` per shot (×0.8 leaning), caps 0.05, decays 0.12/s (line 1227).
- **Shield** (lines 61-69, 1758-1782): front ±60° arc only; ×0.65 damage cut standing, ×0.95 crouched; nothing from sides/rear; no firing while up; drops on death/roll.
- **Robot armor** (lines 1815-1820): `absorbed = min(dmg, armor); armor −= absorbed; hp −= dmg − 0.7·absorbed` — i.e. 70 % soak until the 100-pt pool is gone.
- **Creation dynamics**: everyone spawns with a picked 3-slot loadout + shield; per-slot magazine memory (`switch_slot`, 1578-1595); respawn 3 s at base or owned checkpoint (1239-1280); spawn protection 1.2 s.

There is **no cost, training time, or upkeep anywhere in either game** — no economy exists to price anything (see §3.6).

---

## 3. BALANCE & MATH ANALYSIS

Numbers below were computed by script (`scratchpad/balance.py`) directly from the code's constants, and cross-checked against the sim's own tests.

### 3.1 TDM: cost vs effective power

There is no currency, so "cost" = the slot the gun occupies. TTK assumes all shots land (upper bound); STK = shots to kill.

| Gun | dmg/shot | STK body | STK head | TTK body s | TTK head s | burst DPS | mag dmg | sustained DPS* |
|---|---|---|---|---|---|---|---|---|
| Glock 17 | 9 | 12 | 3 | 1.76 | 0.32 | 56 | 153 | 38 |
| Desert Eagle | 27 | 4 | **1** | 1.26 | 0.00 | 64 | 189 | 42 |
| MP5 | 10 | 10 | 3 | 0.72 | 0.16 | 125 | 300 | 71 |
| Rem 870 (8 pellets, point-blank) | 52 | 2 | 1 | 0.95 | 0.00 | 55 | 364 | 39 |
| AK-47 | 13.5 | 8 | 2 | 0.73 | 0.10 | 129 | 405 | 76 |
| M4A1 | 12.5 | 8 | 2 | 0.63 | 0.09 | 139 | 375 | 80 |
| AWM | 70 | 2 | 1 | 1.60 | 0.00 | 44 | 350 | 32 |
| **M249** | 11 | 10 | 3 | 0.67 | 0.15 | **147** | **1100** | **92** |
| War Bow | 34 | 3 | 3 (no head mult) | 1.90 | 1.90 | 36 | 34 | 18 |
| War Spear | 55 | 2 | 2 (no head mult) | 1.30 | 1.30 | 42 | 55 | 23 |

\* sustained = mag damage / (mag·fire_period + reload).

**Findings, ranked:**

1. **M249 is the best primary on every axis that matters.** Highest burst DPS (147), highest sustained DPS (92), a 100-round mag, and its recoil (`kick 0.0055`) *equals* the AK's. Its only taxes are base spread (0.016) and a 4.5 s reload it rarely needs. There is **no per-gun movement-speed penalty** (speed depends only on stance/ADS/shield — `sim.rs:1347-1362`), so the support gun sprints like an SMG. It is strictly better than the MP5 and dominates mid-range.
2. **No damage falloff on any hitscan gun** (`try_fire` traces to `t_hit = 200 m`, line 1705). An MP5 or shotgun pellet at 150 m does full damage; the only range tax is spread. This flattens the intended niche structure (SMG close / rifle mid / sniper far) and makes the bow/spear objectively worse than rifles at their own long-range game.
3. **Projectiles get no headshot bonus** (flat damage, `sim.rs:1989`), while suffering travel time, drop, 1-round mags. War Bow: 3 hits ≈ 1.9 s TTK best case — the worst effective weapon in the game while being a "signature" pick. War Spear: 2 hits, only **5 spears total** (mag 1 + reserve 5, ~6 kills of theoretical max). They can't compete with the AWM in the same Special slot.
4. **Glock vs Deagle is no contest.** Same slot: Deagle has better DPS (64 vs 56), a 1-tap head, 4-STK body vs 12-STK. The Glock's only edges (mag size, lower move-spread) don't buy back an 8-shot STK gap. Glock is a dead pick.
5. **Crouched shield is near-immunity from the front**: M4 needs **160** body shots through a crouched shield (12.5 → 0.625 dmg), AWM needs 29. Flanking is the designed counter and roll disables it (tested, `sim.rs:2538-2551`) — defensible, but with no shield durability a corner-camper in a checkpoint ring is a stalemate the bots can't solve (bots only flank by accident of waypoints).
6. **Robot armor ≈ +75 % effective HP against rifles** (8 → 14 M4 STK) *plus* ×1.12 speed, on a 45 s respawn — and it spawns on the **same spot as the KOTH hill** (`sim.rs:1036-1039` vs `1160`): whoever holds the hill also gets the strongest item, a snowball loop in KOTH.
7. **Exhausted-overtime tie hands the win to Blue** — the player's team (`finish(Team::Blue) // blue by honor`, `sim.rs:1216`). Cosmetic now, but it biases every recorded stat toward Blue.
8. Doc rot: header comment `sim.rs:8-9` still says the v3 rule "5 body shots / 2 headshots"; v6 baseline is 8/2 (tested at `sim.rs:2381-2409`). Comment at `sim.rs:2458` says crouch height 1.05; the constant is 1.15 (`sim.rs:31`).

### 3.2 TDM difficulty scaling (sane)

Easy → Normal → Hard: aim σ .055/.030/.014, reaction .45/.28/.15 s, engage 22/35/50 m, aggression 0.6/1.0/1.4 (`sim.rs:751-772`). Monotonic, verified by a field test (`difficulty_scales_the_bots`, `sim.rs:2577-2606`). Note Hard bots at σ=0.014 with 0.15 s reaction out-aim the AWM's own base spread — Hard is *very* hard at range.

### 3.3 Battle sim: the weapon-vs-armor matrix has holes

Full-effort strike energy vs energy-to-defeat (strikes-to-down = ceil(45 J / penetration); "—" = cannot penetrate **ever**):

| | Cloth (5 J) | Gambeson (30 J) | Mail (100 J) |
|---|---|---|---|
| **Spear 16.9 J** | 11.9 J → **4 hits** | — (needs effort 1.33 > max 1.2) | — |
| **Sword 49.6 J** | 44.6 J → **2** | 19.6 J → **3** | — |
| **Axe 100.8 J** | 95.8 J → **1** | 70.8 J → **1** | 0.8 J → **57 hits** (≈ 5 min of unblocked swings — i.e. never) |

1. **The spear — 70 % of every army — cannot wound ~50 % of every army.** Gambeson is rolled on half the men (`sim.rs:176-185`), and 16.9 J < 30 J at any physically reachable effort (v-clamp 1.2 → max 24.3 J). Force `armor_a = armor_b = Gambeson` and melee produces *zero* wounds; all casualties are crush. The sourced band said thrusts are 10–30 J *into gaps* — the model charges the thrust against full armor every time (no gap-finding roll).
2. **Mail is melee-proof.** Nothing in the melee triangle meaningfully opens it (the axe's 0.8 J over threshold is noise). The `mail_side_outlasts_cloth_side` test (`tests/combat_battle.rs:39-53`) passes precisely *because* of this. Fine as the metallurgy thesis — but when the forge arrives, mail is a binary win-button unless E_req or weapon quality gets a continuous scale (research already has one: `E_req = 55·Q·t^1.6/cosθ`, `07_CONSTANTS.md`).
3. **The javelin is secretly the best anti-armor weapon in the game.** Impact energy = ½·0.7·|v|² with |v| ≈ 19 m/s at man-height → **~126 J > 100 J mail threshold**, penetrating ~26 J → **2 hits down a mailed man** — something no melee weapon can do. The asymmetry comes from modeling: thrusts use *effective point mass* (0.6 kg), javelins use *full projectile mass* (0.7 kg) at full velocity. Physically arguable, but it inverts the expected ranged-vs-armor relationship and nobody chose it on purpose (the volley test only checks cloth).
4. **Block probability dominates melee pacing.** With base 0.55 (0.8 braced), expected unblocked spear hits on cloth ≈ 4/(1−0.55) ≈ 9 attempts ≈ **29 s per kill** — matches the slow-bleed battles in `output/BATTLE_REPORT.md`. Intentional grind, but worth knowing the knob: `BLOCK_BASE` moves overall lethality more than any weapon stat.
5. **Stamina math is coherent and validated**: pool drains in 60–90 s of hard shove (`constants.rs` test, `stamina.rs` test), strike costs (250–380 J) are negligible against the 67.5 kJ pool (~270 spear strikes) — stamina pressure comes from *pushing and compression*, not swinging. That's a deliberate, defensible modeling choice; it does mean "exhausted swings" effectively never happens from swinging alone.
6. Command profiles (`command.rs:58-116`) are self-consistent and every claimed property is regression-tested (charge > advance impact ×1.2, brace yields least, hold recovers, rotation freshens the front — `tests/commands.rs`).

### 3.4 Morale numbers

Compression fear (+0.02/s) vs recovery (0.015/s) means a hard press alone *sensitizes but cannot break* a line — deliberate, documented at `constants.rs:141-145`, and the pulse-dynamics comment explains the 0.04 → 0.02 retune. Witness fear 0.16 vs rout band 0.65–1.0: ~4–6 nearby deaths in quick succession break the weakest men. Commander down = 0.30 within 12 m — nearly half the weakest man's tolerance. These ratios pass all four morale tests. One quirk: `CHEER_ENEMY_DOWN` can push fear negative within a tick before the end-of-tick clamp (`sim.rs:992`) — harmless but means cheering is slightly stronger than it looks.

### 3.5 Progression / upgrade curves

**There are none.** No blacksmith, tech, era, or level curves exist in code to audit. The only "curve"-like data is in research: bloomery yields, `E_req` quality law, charcoal economics (`07_CONSTANTS.md`, `research/03_economy.md`). Consequence: §5's redesign is a *baseline* table; the first real curve work is the forge vertical slice (§4).

### 3.6 Economy check

**No income, no costs, no pace to verify.** `research/03_economy.md` contains a complete, sourced model (charcoal = land + labor + radius; transport ratios; proposed formulas like `charcoal_output(site) = min(coppice_ha × yield × charcoal_frac, collier_labor_cap)`) — but not one line is implemented. Any statement about affordability would be fiction; flagged as the biggest gap between research investment and shipped code.

### 3.7 Unused / duplicated / contradictory stats

| Item | Kind | Where |
|---|---|---|
| `PUSH_FORCE_SUSTAINED_N` (400 N) | **Unused** — sim uses `PUSH_FORCE_BURST_N` and `FOLLOWER_LEAN_FORCE_N` only | `constants.rs:25` |
| `CHEST_INJURY_FORCE_N` (1.1 kN) | **Unused** — crush model uses `CRUSH_DOWN_FORCE_N` (4.5 kN) instead | `constants.rs:33` |
| `ALPHA_EXPECTED_BAND`, `OVERLAP_TARGET` | Test/validation only (documented as such) | `constants.rs:37,66` |
| `breach_risk()` | Dead code — real breaches are geometric | `cohesion.rs:13-15` |
| Bow/Spear damage duplicated in `damage` **and** `projectile.1` | Duplication — two fields must be kept in sync by hand | `sim.rs:307,322` |
| Three gravities: bodies 18 (`GRAVITY`, tdm `sim.rs:42`), missiles 9.81×(1.0 spear / 0.55 arrow) (`sim.rs:1935`), battle `GRAVITY_M_S2 = 9.81` | Inconsistent by design ("gamey gravity") but the arrow factor 0.55 is a magic number **duplicated** in `step_missiles` (1935) and `predict_arc` (1876) | |
| v3 TTK comment "5 body / 2 head" vs v6 rule 8/2 | Contradiction (doc rot) | tdm `sim.rs:8-9` |
| Crouch-height comment 1.05 vs constant 1.15 | Contradiction (doc rot) | tdm `sim.rs:2458` vs `31` |
| Axe described as having "the reach to break things open" | Misleading — axe reach 1.3 m < spear 1.9 m; it's the *energy* that breaks things | `jk_wall/sim.rs:238-241` |
| `Missile.id`, `HitEvent.at` | `#[allow(dead_code)]` — declared API, unused by client | tdm `sim.rs:919-921,942-944` |

### 3.8 Test suite

`cargo test --release` across the workspace: **all green** (see run log). Coverage is genuinely good: determinism (bit-identical replays in both sims), α-band emergence, cascade-vs-depth, all six commands' physical claims, morale (4 scenarios), TTK rules, shield arcs, checkpoints, maps validity, arc prediction ±0.6 m. What's *not* tested: any cross-weapon balance claim (e.g. nothing asserts the M249 isn't dominant), and nothing exercises gambeson vs spear (which would have caught §3.3.1).

---

## 4. OPTIMIZATION OPPORTUNITIES

### 4.1 Performance (ranked by impact at target scale — 1000+ bodies)

1. **Unbounded telemetry growth — a real leak in live clients.** `WallSim::step()` pushes a `StepMetrics` (containing 3+ heap `Vec`s) **every tick, forever** (`jk_wall/src/sim.rs:1013-1015`). `jk_bevy`/`jk_client` run indefinitely: at 120 Hz that's ~430 k allocations/hour and unbounded RAM. Fix: ring buffer or a `telemetry_enabled` flag for clients. *Effort: small.*
2. **O(n²) scans in the wall sim.** Melee targeting scans every agent per ready striker (`sim.rs:829-842`), and the 10 Hz outnumber refresh is a full n² pass (`sim.rs:909-941`). At 1024 bodies that's ~1 M distance checks per refresh plus worst-case n² in melee. A uniform spatial grid (cell ≈ 2 m) makes both O(n). This is the difference between 1.8× realtime and comfortably >5× at 1024. *Effort: medium.*
3. **Per-tick allocation churn.** Each `step()` allocates: `follow` vec, per-file sort vecs (`sim.rs:386-399`), `agent_snap` (717-724), morale `positions` (911-918), `newly_down`, plus `front_line()` building sorted vecs **6–8 times per tick** (twice for `enemy_plane` at 429-439, twice in `collect_metrics` 1077-1080, more in breach detection). Compute the front line once per tick into reusable buffers. *Effort: small-medium.*
4. **Rapier is single-threaded here, with 8 solver iterations** (`sim.rs:304-312`). Options in order of cheapness: enable rapier's `parallel` feature; profile 8 → 6 iterations against α-band tests; the planned Jolt swap (ADR-002 trigger ">300 bodies" is already tripped by the 1024 benchmark). *Effort: small / large.*
5. **TDM is fine at its 8v8 cap.** Linear cover scans per pellet/LOS (`ray_hit` over ~60 AABBs) and per-missile checks are negligible at this scale. Only if maps or counts grow: a BVH over `cover`. The client's `sync_fighters` does ~30 `get_mut` part lookups per fighter per frame (tdm `main.rs:2573-2848`) — fine at 16 rigs.
6. **AI think-rate**: battle-sim behavior runs at full 120 Hz per agent (only outnumbering is throttled). `DESIGN_MAP.md:80` already plans 10–20 Hz decision throttling — the follow/role logic (`sim.rs:386-419`) is the next candidate to move to the slow tier.

### 4.2 Code structure — single source of truth for stats

Current state is *better than most codebases* (ADR-004 works: `jk_core::constants` + one `gun()` table + `Weapon::spear()/sword()/axe()`), but three things undercut it:

- The TDM gun table is **code**, not data. Balancing means recompiling; the in-game manual already generates from the table, so the table is one `serde` derive away from being a `weapons.ron` asset with hot-reload. Include zone multipliers, movement multipliers, and (new) falloff curves in the same file.
- Duplicates listed in §3.7 (projectile damage pairs, the 0.55 arrow-gravity factor, three gravities) should each collapse to one named constant.
- The battle sim's kit distribution (35/10 % mail, 50 % gambeson, 70/20/10 weapons — `jk_wall/sim.rs:171-246`) is buried in the spawn closure; it's the army-composition knob the campaign layer will need, so promote it into `WallSimConfig`.

### 4.3 Gameplay — top 10 concrete changes

| # | Change | Why | Files | Effort |
|---|---|---|---|---|
| 1 | **Hitscan damage falloff** (e.g. ×1.0 to 15 m → ×0.6 at 50 m, per-gun near/far) | Restores SMG/rifle/sniper niches; makes bow/AWM the long-range picks; fixes shotgun sniping | tdm `sim.rs` `try_fire`/`apply_hit` (add distance → multiplier), gun table | Small |
| 2 | **Tax the M249**: move-spread ×2 while unbraced, or per-class move-speed mult (LMG 0.9) | It currently outclasses every primary (§3.1.1) | tdm `sim.rs:156-327, 1347-1362` | Small |
| 3 | **Projectile headshot bonus** (×1.5–2 via hit-height on the cylinder, like hitscan) + raise spear reserve 5 → 8 | Bow/spear are strictly-worse picks; they're the game's signature | tdm `sim.rs:1941-1951` (compute zone from `m.pos[1]`), `1989` | Small |
| 4 | **Give the Glock a role**: 0.16 → 0.11 s fire, or 9 → 11 dmg (STK 10) | Dead pick vs Deagle in the same slot | tdm `sim.rs:187-200` | Small |
| 5 | **Decouple robot armor from the KOTH hill** (in KOTH, spawn it on a flank pad) | Hill-holder also gets the best item — snowball (§3.1.6) | tdm `sim.rs:1034-1039` | Small |
| 6 | **Spear vs gambeson**: add a gap-finding roll (e.g. 20 % of thrusts test vs cloth) or raise spear energy band | 70 % of the army is harmless against 50 % of the army (§3.3.1) | `jk_wall/combat.rs` `resolve_strike`, `constants.rs:83-96` | Small-Medium |
| 7 | **Unify the javelin/thrust energy model** (effective mass for both, or E_req applies a projectile factor) | Javelins silently out-armor-pierce every melee weapon (§3.3.3) | `jk_wall/sim.rs:752-757`, `constants.rs:113-129` | Small |
| 8 | **Cap telemetry** (ring buffer, N=2 for clients / full for spike) | Unbounded RAM growth in live clients (§4.1.1) | `jk_wall/metrics.rs`, `sim.rs:1013-1015` | Small |
| 9 | **Externalize the gun table to `weapons.ron`** + hot-reload; regenerate the manual from it | Balancing without recompiles; single source of truth | tdm `sim.rs:156-327`, `main.rs` manual page | Medium |
| 10 | **The forge vertical slice** — bloomery → blade quality Q → `Weapon.strike_v`/`E_req` via the researched law `E_req = 55·Q·t^1.6` | It's the project's entire thesis ("Era 1: bloomery → one blade → one 40v40 wall") and every socket already exists (`combat.rs:5-8`) | new crate `jk_forge`; `jk_wall/combat.rs` | Large |

---

## 5. RECOMMENDED STAT REDESIGN (baseline to tune)

### 5.1 TDM guns

Principles: M4 stays the anchor (8 body / 2 head, owner's rule). Every gun gets exactly one identity stat it wins. New columns `falloff_start/end` (damage ×1.0 → ×0.6 linearly) and `move_mult` create the niches ranged stats can't.

| Gun | dmg | fire s | STK body (close) | falloff m | move_mult | other changes | identity |
|---|---|---|---|---|---|---|---|
| Glock 17 | **11** | **0.13** | 10 | 18–40 | 1.0 | — | fast, forgiving sidearm |
| Desert Eagle | 27 | 0.42 | 4 | 15–35 | 1.0 | kick 0.008 → **0.010** | the 1-tap head, punished spam |
| MP5 | 10 | 0.08 | 10 | **12–30** | **1.05** | — | close-range mobility king |
| Rem 870 | 6.5×8 | 0.95 | 2 | **8–20** | 1.0 | — | door-fight monster, useless past 20 m |
| AK-47 | 13.5 | 0.105 | 8 | 25–55 | 1.0 | — | hardest-hitting rifle, hardest to control |
| M4A1 | 12.5 | 0.09 | 8 | 25–55 | 1.0 | — | the baseline (unchanged) |
| AWM | 70 | 1.6 | 2 | **none** | 0.95 | — | only head is instant (unchanged) |
| M249 | 11 | 0.075 | 10 | 20–50 | **0.88** | spread_move .032 → **.045** | area denial, not a sprint gun |
| War Bow | 34 | 0.95 | 3 | none | 1.0 | **zone mult on arrows (head ×2 → 68 = 2-hit head)** | silent skill pick |
| War Spear | 55 | 1.3 | 2 | none | 1.0 | reserve 5 → **8**, head ×1.5 (82.5) | high-risk lob, now sustainable |

Shield: keep 65/95 % but add **durability 250 dmg** (breaks, regenerates 25/s after 5 s unraised) — ends infinite turtling without touching the flanking counter. Robot armor: 100 → **75** pool, keep 70 % soak (≈ +4 rifle hits instead of +6), speed bonus 1.12 → **1.06**.

### 5.2 Battle sim (Era-1 melee/armor)

Goal: every weapon threatens every armor *somewhere*, with the ordering preserved (axe > sword > spear vs armor; mail ≫ gambeson ≫ cloth). Uses only existing constants:

| Constant | Now | Proposed | Reasoning |
|---|---|---|---|
| `SPEAR_THRUST_V_MS` | 7.5 | **8.5** (E = 21.7 J) | Still inside the sourced 10–30 J band; spear now wounds gambeson on a committed thrust (pen 21.7−30 <0 — still no; see next row) |
| *new* `GAP_FINDING_P` | — | **0.25** | 25 % of penetrating checks resolve vs one armor tier lower (thrusts hunt seams — the historical mechanism the point-data implies). With it, spear vs gambeson downs in ~ceil(45/16.7)=3 gap hits; mail stays axe/javelin business |
| `E_REQ_MAIL_J` | 100 | 100 (keep) | Sourced (Williams); axe at 100.8 J stays a coin-flip splitter — correct feel |
| Axe `eff_mass` | 1.4 | **1.5** (E = 108 J) | Pen 8 J → mail downed in ~6 unblocked hits ≈ one sustained duel, not 57 |
| `JAVELIN_V0_MAX_MS` | 19 | **16** (E = 89.6 J) | Javelin no longer out-pierces mail; still shreds cloth/gambeson volleys |
| `WOUND_DOWN_J` | 45 | 45 (keep) | Keeps the validated 2-hit sword / 4-hit spear vs cloth pacing |
| `BLOCK_BASE` | 0.55 | **0.50** | Battles resolve ~10 % faster; front-rank mail still the survivability story |
| Kit fractions | hardcoded | move to `WallSimConfig` | Campaign layer's army-composition knob (§4.2) |

Sanity checks under the proposal (same formulas): spear vs cloth 3–4 hits (was 4), sword vs gambeson 3 (unchanged), axe vs mail ~6 unblocked (was 57), javelin vs mail stopped (was 2-hit kill), all `tests/` invariants except `mail_side_outlasts_cloth_side`'s exact ×3 margin should still hold — rerun the suite after applying.

---

## Suggested order of work — next 5 sessions

1. **Session 1 — hygiene + leaks (all small, all §4):** telemetry ring buffer; delete/implement `PUSH_FORCE_SUSTAINED_N`, `CHEST_INJURY_FORCE_N`, `breach_risk`; collapse duplicated constants (projectile damage pair, 0.55 arrow gravity); fix the three stale comments; overtime tie → draw.
2. **Session 2 — TDM balance pass:** damage falloff + per-gun move_mult columns, M249/Glock/projectile changes, armor-off-hill, shield durability (§5.1). Add a `balance.rs` test that asserts no primary strictly dominates another (DPS × mobility × capacity envelope).
3. **Session 3 — data-driven stats:** gun table → `weapons.ron` with hot reload; battle kit fractions → `WallSimConfig`; regenerate the in-game manual from the data file.
4. **Session 4 — battle-sim combat rebalance:** §5.2 constants + gap-finding roll + javelin/thrust energy unification; add the missing gambeson-vs-spear regression test; rerun spike + morale suites.
5. **Session 5 — scale + the thesis:** spatial grid for melee/outnumber scans (target: 1024 bodies ≥ 4× realtime single-thread), then start the **forge vertical slice** (`jk_forge`: bloomery yield → bar quality Q → `Weapon`/`ArmorKind` parameters via the researched penetration law) — the system every socket in `combat.rs` has been waiting for.

---
*Verification artifacts: balance math script at `scratchpad/balance.py` (session temp dir); full workspace test run log referenced in §3.8.*
