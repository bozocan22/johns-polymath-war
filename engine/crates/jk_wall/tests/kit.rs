//! Army composition: the campaign layer's kit knob.
//!
//! Two things need guarding. That the DEFAULTS still describe the Era-1
//! army (otherwise this refactor silently rebalanced every battle), and
//! that an OVERRIDE actually reaches the spawner (otherwise the knob is
//! decorative and every test here passes for the wrong reason).

use jk_wall::combat::WeaponKind;
use jk_wall::{ArmorKind, KitDistribution, Side, WallSim, WallSimConfig};

fn army(kit: KitDistribution, seed: u64) -> WallSim {
    WallSim::new(WallSimConfig {
        files: 16,
        ranks_a: 8,
        ranks_b: 8,
        seed,
        kit,
        ..Default::default()
    })
}

/// These five numbers WERE literals in the spawn closure. Pinning them
/// means a future edit reads as what it is — a change to who is carrying
/// what, not a harmless tidy-up — because this test goes red.
#[test]
fn the_defaults_still_describe_the_era1_army() {
    let k = KitDistribution::default();
    assert_eq!(k.mail_front, 0.35, "front-rank mail");
    assert_eq!(k.mail_rear, 0.10, "rear-rank mail");
    assert_eq!(k.gambeson_span, 0.5, "gambeson band above mail");
    assert_eq!(k.spear_max, 0.7, "spear share");
    assert_eq!(k.sword_max, 0.9, "spear+sword cumulative edge");
}

/// The wiring proof. If `cfg.kit` were ignored and the spawner still read
/// its old literals, this army would come out ~70% spears instead of all
/// axes, and this fails.
#[test]
fn an_override_actually_reaches_the_spawner() {
    let all_axes = KitDistribution {
        spear_max: 0.0,
        sword_max: 0.0,
        ..Default::default()
    };
    let sim = army(all_axes, 7);
    assert!(!sim.agents.is_empty());
    assert!(
        sim.agents.iter().all(|a| a.weapon.kind == WeaponKind::Axe),
        "every man should carry an axe; got {} non-axes",
        sim.agents
            .iter()
            .filter(|a| a.weapon.kind != WeaponKind::Axe)
            .count()
    );

    // and the armour half of the knob, independently
    let no_mail_all_gambeson = KitDistribution {
        mail_front: 0.0,
        mail_rear: 0.0,
        gambeson_span: 1.0,
        ..Default::default()
    };
    let sim = army(no_mail_all_gambeson, 7);
    assert!(
        sim.agents.iter().all(|a| a.armor == ArmorKind::Gambeson),
        "every man should be in gambeson"
    );
}

/// The historical mix is a distribution, not a guarantee, so this asserts
/// the band rather than an exact count — but a wiring error moves the
/// result far outside it.
#[test]
fn the_default_mix_lands_in_the_historical_band() {
    let sim = army(KitDistribution::default(), 0xC0FFEE);
    let n = sim.agents.len() as f32;
    assert!(n >= 200.0, "need a big enough sample, got {n}");

    let frac = |k: WeaponKind| {
        sim.agents.iter().filter(|a| a.weapon.kind == k).count() as f32 / n
    };
    let (spear, sword, axe) = (
        frac(WeaponKind::Spear),
        frac(WeaponKind::Sword),
        frac(WeaponKind::Axe),
    );
    assert!((0.62..0.78).contains(&spear), "spear share {spear}");
    assert!((0.14..0.26).contains(&sword), "sword share {sword}");
    assert!((0.05..0.16).contains(&axe), "axe share {axe}");

    // Mail is rank-dependent: the front rank is markedly better equipped.
    let mail_in = |rank: usize| {
        let men: Vec<_> = sim.agents.iter().filter(|a| a.rank == rank).collect();
        men.iter().filter(|a| a.armor == ArmorKind::Mail).count() as f32 / men.len() as f32
    };
    assert!(
        mail_in(0) > mail_in(3),
        "front rank should out-armour the rear: {} vs {}",
        mail_in(0),
        mail_in(3)
    );
}

/// Composition is drawn from the seeded stream, so it has to be a pure
/// function of the seed — a campaign that re-creates the same army must
/// get the same men.
#[test]
fn the_same_seed_equips_the_same_army() {
    let a = army(KitDistribution::default(), 4242);
    let b = army(KitDistribution::default(), 4242);
    assert_eq!(a.agents.len(), b.agents.len());
    for (x, y) in a.agents.iter().zip(b.agents.iter()) {
        assert_eq!(x.armor, y.armor);
        assert_eq!(x.weapon.kind, y.weapon.kind);
        assert_eq!(x.side, y.side);
    }
    // and a different seed does NOT produce the same army
    let c = army(KitDistribution::default(), 4243);
    let differs = a
        .agents
        .iter()
        .zip(c.agents.iter())
        .any(|(x, y)| x.armor != y.armor || x.weapon.kind != y.weapon.kind);
    assert!(differs, "a different seed must roll a different army");
}

/// Forcing a side's armour must still win over the distribution.
#[test]
fn forced_armour_still_overrides_the_kit() {
    let sim = WallSim::new(WallSimConfig {
        seed: 11,
        armor_a: Some(ArmorKind::Mail),
        kit: KitDistribution {
            mail_front: 0.0,
            mail_rear: 0.0,
            ..Default::default()
        },
        ..Default::default()
    });
    assert!(
        sim.agents
            .iter()
            .filter(|a| a.side == Side::A)
            .all(|a| a.armor == ArmorKind::Mail),
        "armor_a must beat a zero-mail kit"
    );
}
