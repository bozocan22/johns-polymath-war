//! Melee combat — strikes resolved by energy transfer vs. protection
//! (brief §2.5). No damage numbers: a thrust carries kinetic energy, the
//! target's armor demands energy to defeat, and what passes through wounds.
//!
//! This is the socket the metallurgy plugs into: `Weapon` and `Armor` carry
//! the material properties (today from sourced constants; from the forge in
//! M4 — a badly tempered spearhead will bend and a slag-ridden blade snap
//! here, nowhere else).
//!
//! Determinism: all randomness draws from the sim's seeded PCG in fixed
//! agent order — same seed, same battle, same wounds.

use jk_core::constants as k;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArmorKind {
    /// Wool and luck.
    Cloth,
    /// Quilted textile.
    Gambeson,
    /// Riveted mail over padding — Era 1 wealth.
    Mail,
}

impl ArmorKind {
    /// Energy a point attack must spend to get through (research/02).
    pub fn e_required_j(self) -> f32 {
        match self {
            ArmorKind::Cloth => k::E_REQ_FLESH_J,
            ArmorKind::Gambeson => k::E_REQ_GAMBESON_J,
            ArmorKind::Mail => k::E_REQ_MAIL_J,
        }
    }

    pub fn mass_kg(self) -> f32 {
        match self {
            ArmorKind::Cloth => 0.0,
            ArmorKind::Gambeson => 3.0,
            ArmorKind::Mail => 11.0, // hauberk 9–12 kg + padding
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WeaponKind {
    Spear,
    Sword,
    Axe,
}

#[derive(Clone, Copy, Debug)]
pub struct Weapon {
    pub kind: WeaponKind,
    /// Effective mass at the point of percussion (kg).
    pub eff_mass_kg: f32,
    /// Strike velocity (m/s).
    pub strike_v_ms: f32,
    pub reach_m: f32,
    /// Multiplier on the wielder's strike tempo (lower = faster).
    pub period_mult: f32,
    /// Anaerobic cost of a committed strike (J).
    pub pool_cost_j: f32,
}

impl Weapon {
    /// The formation weapon: long reach, deliberate tempo, modest energy —
    /// strong in a wall, weak alone. SOURCED(brief §2.5): thrust 10–30 J.
    pub fn spear() -> Self {
        Weapon {
            kind: WeaponKind::Spear,
            eff_mass_kg: k::SPEAR_EFF_MASS_KG,
            strike_v_ms: k::SPEAR_THRUST_V_MS,
            reach_m: k::SPEAR_REACH_M,
            period_mult: 1.0,
            pool_cost_j: k::STRIKE_POOL_COST_J,
        }
    }

    /// Fast and versatile, short reach. SOURCED(brief §2.5): cut 15–80 J
    /// (eff. 0.9 kg at 10.5 m/s ≈ 50 J).
    pub fn sword() -> Self {
        Weapon {
            kind: WeaponKind::Sword,
            eff_mass_kg: 0.9,
            strike_v_ms: 10.5,
            reach_m: 1.1,
            period_mult: 0.6,
            pool_cost_j: 180.0,
        }
    }

    /// Huge blows, slow recovery. SOURCED(brief §2.5): axe 60–200 J
    /// (eff. 1.4 kg at 12 m/s ≈ 100 J — enough to open mail).
    pub fn axe() -> Self {
        Weapon {
            kind: WeaponKind::Axe,
            eff_mass_kg: 1.4,
            strike_v_ms: 12.0,
            reach_m: 1.3,
            period_mult: 1.6,
            pool_cost_j: 380.0,
        }
    }

    /// Kinetic energy of a committed strike, scaled by the striker's
    /// condition (a spent man thrusts slower; E ∝ v²).
    pub fn strike_energy_j(&self, effort: f32) -> f32 {
        let v = self.strike_v_ms * effort.clamp(0.3, 1.2);
        0.5 * self.eff_mass_kg * v * v
    }
}

/// Outcome of one resolved strike.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum StrikeOutcome {
    Blocked,
    /// Armor held; energy bruises but does not enter.
    Stopped { absorbed_j: f32 },
    /// Energy through armor, into the man.
    Wound { penetrated_j: f32 },
}

/// Resolve a strike against a defender's protection and state.
///
/// `block_roll` ∈ [0,1) comes from the sim's seeded RNG. `overload` is the
/// defender's compression overload (can't work a shield while being
/// crushed), `braced` whether their side is braced, `def_stamina` their
/// output fraction.
pub fn resolve_strike(
    energy_j: f32,
    armor: ArmorKind,
    block_roll: f32,
    overload: f32,
    braced: bool,
    def_stamina: f32,
    def_routing: bool,
) -> StrikeOutcome {
    let block = if def_routing {
        // His shield is on his back and his eyes are on the horizon.
        k::ROUT_BLOCK_CHANCE
    } else {
        let mut b = k::BLOCK_BASE;
        if braced {
            b += k::BLOCK_BRACE_BONUS;
        }
        b -= k::BLOCK_OVERLOAD_MALUS * overload.min(2.0);
        if def_stamina < 0.5 {
            b -= k::BLOCK_EXHAUSTED_MALUS;
        }
        b.clamp(0.05, 0.95)
    };
    if block_roll < block {
        return StrikeOutcome::Blocked;
    }
    let e_req = armor.e_required_j();
    if energy_j <= e_req {
        StrikeOutcome::Stopped { absorbed_j: energy_j }
    } else {
        StrikeOutcome::Wound {
            penetrated_j: energy_j - e_req,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spear_energy_in_research_band() {
        let e = Weapon::spear().strike_energy_j(1.0);
        assert!((10.0..=30.0).contains(&e), "spear thrust {e:.0} J outside 10–30 J");
    }

    #[test]
    fn mail_stops_what_flesh_does_not() {
        let e = Weapon::spear().strike_energy_j(1.0);
        // Unblocked thrust vs cloth wounds; vs mail it is stopped.
        let vs_cloth = resolve_strike(e, ArmorKind::Cloth, 0.99, 0.0, false, 1.0, false);
        let vs_mail = resolve_strike(e, ArmorKind::Mail, 0.99, 0.0, false, 1.0, false);
        assert!(matches!(vs_cloth, StrikeOutcome::Wound { .. }));
        assert!(matches!(vs_mail, StrikeOutcome::Stopped { .. }));
    }

    #[test]
    fn crushed_men_cannot_block() {
        // Same roll: a fresh braced man blocks it, a crushed exhausted one
        // takes it.
        let roll = 0.5;
        let fresh = resolve_strike(20.0, ArmorKind::Cloth, roll, 0.0, true, 1.0, false);
        let crushed = resolve_strike(20.0, ArmorKind::Cloth, roll, 1.5, false, 0.3, false);
        assert_eq!(fresh, StrikeOutcome::Blocked);
        assert!(matches!(crushed, StrikeOutcome::Wound { .. }));
    }
}
