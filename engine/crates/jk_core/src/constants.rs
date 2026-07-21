//! Calibrated constants — single source of numeric truth (ADR-004).
//!
//! Provenance tags:
//! - `SOURCED(research/NN)` — traced to projects/shieldwall_reforged/research/
//!   and 07_CONSTANTS.md
//! - `PROVISIONAL(...)` — defensible estimate, no primary source yet
//!
//! No sim code may carry numeric literals for physical quantities; it imports
//! them from here.

// ------------------------------------------------------------------ bodies
/// SOURCED(research/01, brief §2.1): era/nutrition dependent band.
pub const BODY_MASS_KG: (f32, f32) = (62.0, 82.0);
/// SOURCED(brief §2.1): Era-1 kit — gambeson ~3 kg, mail 9–12 kg, shield+spear.
pub const GEAR_MASS_ERA1_KG: (f32, f32) = (6.0, 14.0);
/// PROVISIONAL(anthropometry): capsule radius/height for a standing man.
pub const BODY_CAPSULE_RADIUS_M: f32 = 0.22;
pub const BODY_CAPSULE_HEIGHT_M: f32 = 1.72;
/// SOURCED(research/01): round-shield diameter band, Viking-age finds ~0.8–1.0 m.
pub const SHIELD_WIDTH_M: f32 = 0.9;

// ------------------------------------------------------------------- push
/// SOURCED(research/01): sustained individual push, braced ~250–500 N;
/// rugby machine studies: individual sustained ~1–1.5 kN peak.
pub const PUSH_FORCE_SUSTAINED_N: f32 = 400.0;
pub const PUSH_FORCE_BURST_N: f32 = 1_200.0;
/// SOURCED(research/01): whole-column saturation — instrumented rugby packs
/// 4.5–8 kN and crowd-crush barrier loads in the same band. The chain cannot
/// deliver more than this to the front interface regardless of depth.
pub const FRONT_FORCE_CAP_N: f32 = 6_000.0;
/// SOURCED(research/01): >~1.1 kN sustained chest compression = injury risk
/// (crowd-crush literature); used by the (future) casualty model.
pub const CHEST_INJURY_FORCE_N: f32 = 1_100.0;
/// SOURCED(research/01 + prototype exp_pushchain_depth): emergent per-rank
/// attenuation lands ~0.78 with misalignment + stamina desync. The engine does
/// NOT apply this — it must EMERGE. Kept for validation bounds only.
pub const ALPHA_EXPECTED_BAND: (f32, f32) = (0.60, 0.90);

// ---------------------------------------------------------------- stamina
/// SOURCED(research/01, brief §2.4): aerobic ceiling of sustained output.
pub const AEROBIC_POWER_W: f32 = 300.0;
/// SOURCED(brief §2.4): anaerobic reservoir ≈ 60–90 s of hard shoving above
/// the aerobic ceiling. Pool sized so ~900 W excess drains it in ~75 s.
pub const ANAEROBIC_POOL_J: f32 = 67_500.0;
/// PROVISIONAL(physiology): recovery rate coefficient when below ceiling.
/// Anaerobic repayment (EPOC) has a half-time of ~30–60 s; 0.04 with ~50%
/// spare aerobic capacity gives ≈35 s — a short lull takes the edge off,
/// it does NOT reset a spent man. (0.35 refilled the pool in ~10 s, which
/// made rank rotation pointless — a breather healed everyone.)
pub const RECOVERY_RATE: f32 = 0.04;
/// PROVISIONAL: fraction of max push force available when the pool is empty.
pub const EXHAUSTED_PUSH_FRACTION: f32 = 0.35;
/// PROVISIONAL(crowd-crush physiology): metabolic cost of withstanding
/// sustained chest compression — impaired breathing plus isometric bracing.
/// 0.1 W/N puts a 3 kN front-rank press at ~300 W on top of fighting load:
/// the front drains in the design's 60–90 s window while the leaning rear
/// stays near its aerobic ceiling. THIS is why walls rotated ranks.
pub const COMPRESSION_POWER_W_PER_N: f32 = 0.1;
/// PROVISIONAL(posture): sustainable lean of a follower pressed into the
/// man ahead — supportive contact, not a max-effort shove. Only a CHARGE
/// order opens the whole column's push to full effort.
pub const FOLLOWER_LEAN_FORCE_N: f32 = 250.0;

// --------------------------------------------------------------- cohesion
/// SOURCED(brief §2.3): target shield-overlap ratio band.
pub const OVERLAP_TARGET: f32 = 0.4;
/// SOURCED(brief §2.3): below this the line is individuals, not a wall.
pub const OVERLAP_CRITICAL: f32 = 0.15;
/// PROVISIONAL: sigmoid temperature for breach risk.
pub const BREACH_TAU: f32 = 0.08;

// ---------------------------------------------------------------- commands
/// PROVISIONAL(biomechanics): sprint-in-kit charge speed. Loaded infantry
/// short-burst ~3–4 m/s; full sprint unloaded ~6+.
pub const CHARGE_SPEED_M_S: f32 = 3.2;
/// PROVISIONAL: player soldier's max self-propulsion speed (m/s) and the
/// fraction of push force under direct control.
pub const PLAYER_MOVE_SPEED_M_S: f32 = 2.2;

// ------------------------------------------------------------------ combat
/// SOURCED(brief §2.5, research/01): spear thrust — effective mass at the
/// point ~0.6 kg at 6–10 m/s → 10–30 J, small contact area.
pub const SPEAR_EFF_MASS_KG: f32 = 0.6;
pub const SPEAR_THRUST_V_MS: f32 = 7.5;
pub const SPEAR_REACH_M: f32 = 1.9;
/// PROVISIONAL(drill tempo): seconds between committed thrusts in a press
/// (probing strikes are more frequent but not wounding).
pub const STRIKE_PERIOD_S: (f32, f32) = (2.4, 4.0);
/// PROVISIONAL(physiology): anaerobic cost of a committed thrust.
pub const STRIKE_POOL_COST_J: f32 = 250.0;
/// Energy to defeat protection, point attack (research/02, Williams tests):
/// bare flesh/clothing ~5 J; textile armor ~30 J (jack/gambeson defeats up
/// to ~50 J vs cut, less vs point); riveted mail + padding ~100 J.
pub const E_REQ_FLESH_J: f32 = 5.0;
pub const E_REQ_GAMBESON_J: f32 = 30.0;
pub const E_REQ_MAIL_J: f32 = 100.0;
/// PROVISIONAL: penetrated energy that incapacitates outright, and the
/// accumulation threshold for lesser wounds.
pub const WOUND_DOWN_J: f32 = 45.0;
/// PROVISIONAL(research/05 block model, refined to geometry in M4):
/// base chance the shield/parry eats a strike, modified by state.
pub const BLOCK_BASE: f32 = 0.55;
pub const BLOCK_BRACE_BONUS: f32 = 0.25;
pub const BLOCK_OVERLOAD_MALUS: f32 = 0.30;
pub const BLOCK_EXHAUSTED_MALUS: f32 = 0.20;
/// Era-1 kit distribution: fraction with mail (front rank / rear ranks) —
/// "mail for the rich", and the rich stand in front. SOURCED(era framing).
pub const MAIL_FRACTION_FRONT: f32 = 0.35;
pub const MAIL_FRACTION_REAR: f32 = 0.10;

// --------------------------------------------------------------- javelins
/// SOURCED(brief §2.5 band): light throwing spear ~0.6–1.0 kg.
pub const JAVELIN_MASS_KG: f32 = 0.7;
/// PROVISIONAL: cast release speed cap (strong overhand throw 15–20 m/s).
pub const JAVELIN_V0_MAX_MS: f32 = 19.0;
/// High-arc launch angle — volleys must clear your own front ranks.
pub const JAVELIN_ARC_RAD: f32 = 0.9; // ~52°
pub const JAVELIN_COUNT: u8 = 2;
pub const JAVELIN_MIN_RANGE_M: f32 = 6.0;
pub const JAVELIN_MAX_RANGE_M: f32 = 28.0;
/// Dispersion radius at the aim point (deterministic scatter).
pub const JAVELIN_SCATTER_M: f32 = 1.8;
/// Both sides loose one volley on their own when the lines close to this.
pub const VOLLEY_AUTO_RANGE_M: f32 = 14.0;
/// Player's re-throw cooldown (s).
pub const THROW_COOLDOWN_S: f32 = 1.6;
pub const GRAVITY_M_S2: f32 = 9.81;
/// Stuck javelins litter the field this long before despawning (s).
pub const JAVELIN_STUCK_TTL_S: f32 = 20.0;

// ------------------------------------------------------------------ morale
// All PROVISIONAL(design) — morale is psychology, not physics; tuned so a
// steady line holds under pressure but a flank massacre breaks it.
/// Fear from witnessing a comrade go down, at zero distance (falls off
/// linearly to the witness radius).
pub const FEAR_WITNESS_DOWN: f32 = 0.16;
pub const FEAR_WITNESS_RADIUS_M: f32 = 6.0;
/// Seeing an ENEMY fall nearby heartens the line.
pub const CHEER_ENEMY_DOWN: f32 = 0.06;
/// Fear accumulation per second while being crushed past the brace limit.
/// Deliberately close to recovery: sustained press SENSITIZES a line
/// (fear creeps up) but cannot break it alone — witnessed deaths tip it.
/// (At 0.04 the whole front routed by t≈30 even in a bloodless grind —
/// an emergent "battle pulse", historically interesting but too total.)
pub const FEAR_COMPRESSION_PER_S: f32 = 0.02;
/// Fear per second while locally outnumbered (per unit of enemy excess).
pub const FEAR_OUTNUMBER_PER_S: f32 = 0.05;
pub const LOCAL_RADIUS_M: f32 = 4.0;
/// Baseline fear decay per second (nerve recovering).
pub const FEAR_RECOVERY_PER_S: f32 = 0.015;
/// Commander aura: within this radius of the standing player-commander,
/// recovery is multiplied and rout thresholds effectively harden.
pub const COMMANDER_AURA_M: f32 = 7.0;
pub const COMMANDER_AURA_RECOVERY_MULT: f32 = 4.0;
/// Broadcast fear when the commander himself goes down.
pub const COMMANDER_DOWN_FEAR: f32 = 0.30;
pub const COMMANDER_DOWN_RADIUS_M: f32 = 12.0;
/// Per-man rout threshold band (nerve varies) and the fear level a routing
/// man must decay to before he rallies back to the line.
pub const ROUT_TOLERANCE: (f32, f32) = (0.65, 1.0);
pub const RALLY_FEAR: f32 = 0.45;
/// Flight speed of a routing man (fear is fast).
pub const ROUT_SPEED_M_S: f32 = 2.8;
/// A routing man barely blocks — his shield is on his back.
pub const ROUT_BLOCK_CHANCE: f32 = 0.05;

// -------------------------------------------------------------- formation
/// PROVISIONAL(drill): lateral file spacing giving ~0.35 overlap at 0.9 m
/// shields: 0.9 * (1 - 0.35) ≈ 0.585 m — round to shoulder-comfortable 0.6 m.
pub const FILE_SPACING_M: f32 = 0.6;
/// PROVISIONAL(drill): front-to-back rank spacing, close order.
pub const RANK_SPACING_M: f32 = 0.75;

// ------------------------------------------------------------- controller
/// PROVISIONAL(tuning): PD gains for slot-holding (per kg of body mass).
pub const PD_KP: f32 = 40.0;
pub const PD_KD: f32 = 12.0;
/// PROVISIONAL(biomechanics): max horizontal ground-reaction force a braced
/// man can exert to hold station, per kg (≈ 0.9 g of own mass).
pub const MAX_FOOT_FORCE_PER_KG: f32 = 8.8;
/// PROVISIONAL(tuning): forward velocity-servo gain — high enough that the
/// drive SATURATES at the stamina-limited push when blocked (the saturation
/// is the othismos; the servo only shapes free marching).
pub const SERVO_GAIN: f32 = 40.0;
/// PROVISIONAL(drill): fraction of foot force budget spent holding the file
/// line laterally. Finite on purpose: columns under compression must be able
/// to buckle sideways — that lateral leakage is a real attenuation mechanism.
pub const LATERAL_HOLD_FRACTION: f32 = 0.35;

// ------------------------------------------------------------------ crush
/// SOURCED(research/01): compressed people lose push output — bracing and
/// pushing share the same postural budget. Overload = compression beyond
/// brace limit degrades push linearly to a floor.
/// Calibrated so the emergent depth curve lands in the α 0.60–0.90 research
/// band (8-deep must meaningfully beat 4-deep — historical drill practice).
pub const BRACE_DEGRADATION_SLOPE: f32 = 0.3;
/// SOURCED(research/01): compression a braced man tolerates before his push
/// output degrades — rugby front-rows sustain 1.5–2.5 kN individually while
/// still driving; degradation onset at the low end of that band.
pub const BRACE_LIMIT_N: f32 = 1_600.0;
pub const BRACE_PUSH_FLOOR: f32 = 0.2;
/// SOURCED(research/01): crowd-crush fatal band ~4.5–7 kN sustained; a man
/// goes down (incapacitated, not necessarily dead) at the band's low end
/// held for his tolerance window.
pub const CRUSH_DOWN_FORCE_N: f32 = 4_500.0;
/// PROVISIONAL: seconds of sustained over-threshold compression to go down.
pub const CRUSH_TOLERANCE_S: (f32, f32) = (8.0, 15.0);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn anaerobic_pool_drains_in_expected_window() {
        // Hard shove: burst power well above aerobic ceiling.
        let excess_w = 900.0;
        let t = ANAEROBIC_POOL_J / excess_w;
        assert!((60.0..=90.0).contains(&t), "drain time {t} s outside 60–90 s");
    }

    #[test]
    fn file_spacing_gives_target_overlap() {
        let overlap = (SHIELD_WIDTH_M - FILE_SPACING_M) / SHIELD_WIDTH_M;
        assert!((OVERLAP_TARGET - 0.1..=OVERLAP_TARGET + 0.1).contains(&overlap));
    }
}
