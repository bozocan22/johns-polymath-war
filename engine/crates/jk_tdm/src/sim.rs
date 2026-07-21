//! TDM sim v3 — deterministic third-person-shooter arena.
//!
//! Same discipline as the battle sim: fixed 120 Hz tick, seeded PCG, every
//! shot, arrow, and death reproducible from seed + inputs.
//!
//! v3: you spawn UNARMED — weapons live on pads (handgun, assault rifle,
//! machine gun, sniper, bow, war spear). Real elevation: plateaus, stairs,
//! and a center tower (gravity + step-up on an AABB terrain). Global TTK
//! rule: 5 body shots / 2 headshots. Hit events carry the zone (legs/body/
//! shoulder/head) and the shot's path for both-way feedback; impacts carry
//! surface normals for bullet marks. Modes: Team Deathmatch and King of the
//! Hill, 5-minute matches with an 80 s sudden-death overtime. Rare pickups:
//! health, ammo, and the ROBOT ARMOR (100 armor, faster legs).
//!
//! v5: recoil halved across the arsenal; a FULL crouch; the duck-spin
//! DODGE ROLL (and the automatic parkour breakfall off hard landings);
//! three maps (dust arena, castle bailey, castle gardens) behind one
//! `MapKind`, each cover box tagged with what it's made of; and
//! `predict_arc` — the exact flight an arrow/spear will take, for the
//! client's aiming laser.

use jk_core::timestep::DT;
use jk_core::Pcg32;

pub const ARENA_HALF: f32 = 34.0;
pub const EYE_REL: f32 = 1.62;
pub const BODY_RADIUS: f32 = 0.34;
pub const BODY_HEIGHT: f32 = 1.78;
/// Full crouch. 1.15 keeps the hitbox honest against the chibi rig's
/// visible crouched head — what you can see, you can shoot.
pub const CROUCH_HEIGHT: f32 = 1.15;
pub const MOVE_SPEED: f32 = 4.8;
pub const SPRINT_SPEED: f32 = 6.6;
pub const CROUCH_SPEED_MULT: f32 = 0.5;
pub const CROUCH_SPREAD_MULT: f32 = 0.55;
pub const ADS_SPREAD_MULT: f32 = 0.32;
pub const ADS_SPEED_MULT: f32 = 0.62;
pub const SWITCH_S: f32 = 0.6;
pub const MAX_HEALTH: f32 = 100.0;
pub const RESPAWN_S: f32 = 3.0;
pub const SPAWN_PROTECT_S: f32 = 1.2;
pub const GRAVITY: f32 = 18.0; // gamey gravity: snappy falls
pub const STEP_UP: f32 = 0.55; // how tall a ledge your legs climb
pub const JUMP_SPEED: f32 = 7.4; // clears a 1.3 m crate (v²/2g ≈ 1.5 m)
// ---- dodge roll (v5): duck-spin dodge, and the parkour breakfall --------
pub const ROLL_S: f32 = 0.55; // how long the somersault lasts
pub const ROLL_SPEED: f32 = 8.6; // faster than a sprint — it's a dodge
pub const ROLL_CD_S: f32 = 0.9; // cooldown after a roll ends
pub const ROLL_HEIGHT: f32 = 0.95; // balled up: a small target
/// Landing harder than this (m/s downward) automatically turns the fall
/// into a breakfall roll — what parkour people do after a big drop.
/// A full flat jump lands at ~7.4 m/s, so ordinary hops stay on their feet.
pub const HARD_LANDING_VY: f32 = 9.5;
/// v6 damage model: four hitbox zones with multipliers over each gun's
/// base TORSO damage. The owner's tuned target — 2 headshots / 8 body
/// shots — is the BASELINE RIFLE (M4A1 at 12.5); other guns scale around
/// it. Limb hits are punished less than center mass.
pub const HEAD_MULT: f32 = 4.0;
pub const ARM_MULT: f32 = 0.75;
pub const LEG_MULT: f32 = 0.75;
// ---- the shield (v6): a core system --------------------------------------
/// Half-angle of the protected front arc (±60° from facing).
pub const SHIELD_ARC_COS: f32 = 0.5; // cos(60°)
/// Damage reduction with the shield up, standing.
pub const SHIELD_BLOCK_STAND: f32 = 0.65;
/// Crouched behind the shield: near-total protection from the front.
/// Flanking is THE counter — sides and rear ignore the shield entirely.
pub const SHIELD_BLOCK_CROUCH: f32 = 0.95;
pub const SHIELD_SPEED_MULT: f32 = 0.55;
/// AWM scoped-in crawl.
pub const SCOPED_SPEED_MULT: f32 = 0.35;
// ---- respawn checkpoints (v6) --------------------------------------------
pub const CHECKPOINT_RADIUS: f32 = 3.0;
pub const CHECKPOINT_CAP_S: f32 = 4.0;
/// Lean (Phantom-Forces peek): how far the eye shifts sideways at full
/// lean. Kept UNDER `BODY_RADIUS` so a full lean against a wall cannot
/// push the muzzle through the wall's collision face.
pub const LEAN_SHIFT: f32 = 0.30;
pub const LEAN_RECOIL_MULT: f32 = 0.8;
pub const MATCH_LEN_S: f32 = 300.0;
pub const OVERTIME_S: f32 = 80.0;
pub const TDM_TARGET: u32 = 30;
pub const KOTH_TARGET_S: f32 = 90.0;
pub const HILL_RADIUS: f32 = 4.5;
pub const PICKUP_RADIUS: f32 = 1.1;
pub const ROBOT_ARMOR_HP: f32 = 100.0;
pub const ROBOT_SPEED_MULT: f32 = 1.12;

// ---------------------------------------------------------------- weapons

/// v6: the roster is a Counter-Strike-style lineup under real-world names —
/// original stats, original art, nothing copied. Bow and spear stay: they
/// are this game's own signature.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GunKind {
    Fists,
    Glock,
    Deagle,
    Mp5,
    Shotgun, // Remington 870
    Ak47,
    M4,
    Awm,
    M249,
    Bow,
    Spear,
}

/// Weapon classes for the loadout screen.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GunClass {
    Secondary,
    Primary,
    Special,
}

pub const PRIMARIES: [GunKind; 5] = [
    GunKind::M4,
    GunKind::Ak47,
    GunKind::Mp5,
    GunKind::Shotgun,
    GunKind::M249,
];
pub const SECONDARIES: [GunKind; 2] = [GunKind::Glock, GunKind::Deagle];
pub const SPECIALS: [GunKind; 3] = [GunKind::Awm, GunKind::Bow, GunKind::Spear];

#[derive(Clone, Copy, Debug)]
pub struct GunSpec {
    pub name: &'static str,
    pub class: GunClass,
    pub fire_period: f32,
    pub mag: u32,
    pub reserve: u32,
    pub reload_s: f32,
    pub spread: f32,
    pub spread_move: f32,
    /// Added to bloom per shot; bloom decays and widens spread (sustained
    /// fire gets unstable). v5: all kick values halved on owner request —
    /// "make the recoil fifty percent less".
    pub kick: f32,
    /// Base TORSO damage per bullet/pellet. Zone multipliers apply on top
    /// (see `zone_mult`): the owner's v6 rule is 2 headshots / 8 body
    /// shots for the baseline rifle.
    pub damage: f32,
    /// Pellets per trigger pull (shotguns fire many, each rolls spread).
    pub pellets: u32,
    /// Some => projectile weapon (launch speed, damage override).
    pub projectile: Option<(f32, f32)>,
    /// ADS zoom FOV in degrees.
    pub zoom_deg: f32,
    /// True → ADS is a full-screen scope overlay (AWM), with heavy
    /// movement slowdown while scoped.
    pub scoped: bool,
}

pub fn gun(kind: GunKind) -> GunSpec {
    // shared defaults keep the table readable
    let base = GunSpec {
        name: "",
        class: GunClass::Primary,
        fire_period: 0.12,
        mag: 30,
        reserve: 120,
        reload_s: 2.0,
        spread: 0.009,
        spread_move: 0.020,
        kick: 0.004,
        damage: 12.5,
        pellets: 1,
        projectile: None,
        zoom_deg: 48.0,
        scoped: false,
    };
    match kind {
        GunKind::Fists => GunSpec {
            name: "Unarmed",
            fire_period: 1.0,
            mag: 0,
            reserve: 0,
            spread: 0.0,
            spread_move: 0.0,
            kick: 0.0,
            damage: 0.0,
            zoom_deg: 62.0,
            ..base
        },
        GunKind::Glock => GunSpec {
            name: "Glock 17",
            class: GunClass::Secondary,
            fire_period: 0.16,
            mag: 17,
            reserve: 68,
            reload_s: 1.3,
            spread: 0.010,
            spread_move: 0.016,
            kick: 0.0025,
            damage: 9.0, // 12 torso / 3 heads
            zoom_deg: 55.0,
            ..base
        },
        GunKind::Deagle => GunSpec {
            name: "Desert Eagle",
            class: GunClass::Secondary,
            fire_period: 0.42,
            mag: 7,
            reserve: 35,
            reload_s: 1.6,
            spread: 0.006,
            spread_move: 0.022,
            kick: 0.008,
            damage: 27.0, // 4 torso — and the iconic one-tap head
            zoom_deg: 52.0,
            ..base
        },
        GunKind::Mp5 => GunSpec {
            name: "MP5",
            fire_period: 0.08,
            mag: 30,
            reserve: 150,
            reload_s: 1.8,
            spread: 0.011,
            spread_move: 0.015, // runs well: an SMG's whole point
            kick: 0.003,
            damage: 10.0, // 10 torso / 3 heads, but pours them out
            zoom_deg: 52.0,
            ..base
        },
        GunKind::Shotgun => GunSpec {
            name: "Remington 870",
            fire_period: 0.95, // pump between shells
            mag: 7,
            reserve: 28,
            reload_s: 2.8,
            spread: 0.055, // the pellet cone
            spread_move: 0.020,
            kick: 0.010,
            damage: 6.5, // ×8 pellets = 52 torso point-blank
            pellets: 8,
            zoom_deg: 56.0,
            ..base
        },
        GunKind::Ak47 => GunSpec {
            name: "AK-47",
            fire_period: 0.105,
            mag: 30,
            reserve: 120,
            reload_s: 2.2,
            spread: 0.011,
            spread_move: 0.024,
            kick: 0.0055, // hits harder, walks harder
            damage: 13.5, // 8 torso / 2 heads with authority
            ..base
        },
        GunKind::M4 => GunSpec {
            name: "M4A1",
            fire_period: 0.09,
            mag: 30,
            reserve: 120,
            reload_s: 2.0,
            spread: 0.008,
            spread_move: 0.018,
            kick: 0.004,
            damage: 12.5, // THE baseline: 2 headshots / 8 body shots
            ..base
        },
        GunKind::Awm => GunSpec {
            name: "AWM",
            class: GunClass::Special,
            fire_period: 1.6, // the bolt: slow re-chamber
            mag: 5,
            reserve: 20,
            reload_s: 3.0,
            spread: 0.0012,
            spread_move: 0.050,
            kick: 0.015,
            // owner's table: ONLY the head is instant. 70 torso → 2 body
            // shots; limbs ×0.75 → 52.5 → still 2; head ×4 → oblivion.
            damage: 70.0,
            zoom_deg: 16.0,
            scoped: true,
            ..base
        },
        GunKind::M249 => GunSpec {
            name: "M249",
            fire_period: 0.075,
            mag: 100,
            reserve: 200,
            reload_s: 4.5,
            spread: 0.016,
            spread_move: 0.032,
            kick: 0.0055,
            damage: 11.0,
            zoom_deg: 52.0,
            ..base
        },
        GunKind::Bow => GunSpec {
            name: "War Bow",
            class: GunClass::Special,
            fire_period: 0.95,
            mag: 1,
            reserve: 24,
            reload_s: 0.9,
            spread: 0.004,
            spread_move: 0.018,
            kick: 0.002,
            damage: 34.0,
            projectile: Some((38.0, 34.0)),
            zoom_deg: 45.0,
            ..base
        },
        GunKind::Spear => GunSpec {
            name: "War Spear",
            class: GunClass::Special,
            fire_period: 1.3,
            mag: 1,
            reserve: 5,
            reload_s: 1.1,
            spread: 0.006,
            spread_move: 0.015,
            kick: 0.003,
            damage: 55.0,
            projectile: Some((17.0, 55.0)),
            zoom_deg: 50.0,
            ..base
        },
    }
}

// ------------------------------------------------------------------ world

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Team {
    Blue,
    Red,
}

#[derive(Clone, Copy, Debug)]
pub struct Aabb {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

impl Aabb {
    /// Slab-method ray hit; also returns the face normal.
    fn ray_hit(&self, o: [f32; 3], d: [f32; 3], t_max: f32) -> Option<(f32, [f32; 3])> {
        let mut t0 = 0.0_f32;
        let mut t1 = t_max;
        let mut axis = 0usize;
        let mut sign = 1.0_f32;
        for i in 0..3 {
            if d[i].abs() < 1e-8 {
                if o[i] < self.min[i] || o[i] > self.max[i] {
                    return None;
                }
            } else {
                let inv = 1.0 / d[i];
                let (mut a, mut b) = ((self.min[i] - o[i]) * inv, (self.max[i] - o[i]) * inv);
                let mut s = -1.0_f32;
                if a > b {
                    std::mem::swap(&mut a, &mut b);
                    s = 1.0;
                }
                if a > t0 {
                    t0 = a;
                    axis = i;
                    sign = s;
                }
                t1 = t1.min(b);
                if t0 > t1 {
                    return None;
                }
            }
        }
        let mut n = [0.0; 3];
        n[axis] = sign;
        Some((t0, n))
    }
}

// ------------------------------------------------------------------- maps

/// v5: three battlefields. Same arena bounds, same pickup lanes, different
/// architecture — the sim owns the layout so every map stays deterministic.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MapKind {
    /// The original dusty range: plateaus, stairs, center tower.
    Arena,
    /// A castle courtyard: central keep, corner drum towers, bailey walls.
    Bailey,
    /// Castle gardens: hedge lanes, ruined walls, trees, a stone gazebo.
    Gardens,
}

impl MapKind {
    pub fn name(self) -> &'static str {
        match self {
            MapKind::Arena => "DUST ARENA",
            MapKind::Bailey => "CASTLE BAILEY",
            MapKind::Gardens => "CASTLE GARDENS",
        }
    }
    pub const ALL: [MapKind; 3] = [MapKind::Arena, MapKind::Bailey, MapKind::Gardens];
}

/// What a cover block is MADE of — the client picks materials by this.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CoverKind {
    Crate,
    Stone,
    Hedge,
    Tree,
}

/// Everything a map defines: its architecture, its size, and where the
/// two respawn checkpoints sit.
struct MapLayout {
    cover: Vec<Aabb>,
    kind: Vec<CoverKind>,
    center_top: f32,
    half: f32,
    checkpoints: [[f32; 2]; 2],
}

/// Build a map's cover set. The KOTH hill and the robot armor crown the
/// center structure, whatever its height. Pickup lanes stay clear on
/// every map. v6: castle maps grew (§10) — more room, more cover, and
/// real vertical layering (ramparts, terraces) with stairs up.
fn build_map(map: MapKind, rng: &mut Pcg32) -> MapLayout {
    let mut cover: Vec<Aabb> = Vec::new();
    let mut kind: Vec<CoverKind> = Vec::new();
    let push = |c: &mut Vec<Aabb>, k: &mut Vec<CoverKind>, a: Aabb, ck: CoverKind| {
        c.push(a);
        k.push(ck);
    };
    // shared center structure: a climbable block with stairs on both z sides
    let center = |cover: &mut Vec<Aabb>, kind: &mut Vec<CoverKind>, h: f32, steps: usize| {
        cover.push(Aabb {
            min: [-4.0, 0.0, -4.0],
            max: [4.0, h, 4.0],
        });
        kind.push(CoverKind::Stone);
        for s in 0..steps {
            let sh = (h / steps as f32) * (steps - s) as f32;
            let zoff = 4.0 + s as f32;
            cover.push(Aabb {
                min: [-2.0, 0.0, zoff],
                max: [2.0, sh, zoff + 1.0],
            });
            kind.push(CoverKind::Stone);
            cover.push(Aabb {
                min: [-2.0, 0.0, -zoff - 1.0],
                max: [2.0, sh, -zoff],
            });
            kind.push(CoverKind::Stone);
        }
    };
    let top;
    let half;
    let checkpoints;
    match map {
        MapKind::Arena => {
            half = ARENA_HALF;
            checkpoints = [[24.5, 0.0], [-24.5, 0.0]]; // the plateau tops
            // side plateaus (mirrored): 2.2 m high with stair slabs
            for sx in [-1.0_f32, 1.0] {
                let px = sx * (ARENA_HALF - 9.0);
                push(&mut cover, &mut kind, Aabb {
                    min: [px - 6.0, 0.0, -7.0],
                    max: [px + 6.0, 2.2, 7.0],
                }, CoverKind::Stone);
                for s in 0..4 {
                    let h = 0.55 * (4 - s) as f32;
                    let zoff = 7.0 + s as f32 * 1.1;
                    push(&mut cover, &mut kind, Aabb {
                        min: [px - 3.0, 0.0, zoff],
                        max: [px + 3.0, h, zoff + 1.1],
                    }, CoverKind::Stone);
                    push(&mut cover, &mut kind, Aabb {
                        min: [px - 3.0, 0.0, -zoff - 1.1],
                        max: [px + 3.0, h, -zoff],
                    }, CoverKind::Stone);
                }
            }
            center(&mut cover, &mut kind, 3.0, 6);
            top = 3.0;
            // scattered crates (mirrored pairs) — but never over a pickup
            // lane: a tall crate under a pad snaps the pickup onto a top
            // nobody can reach for the whole match
            for i in 0..9 {
                let x = rng.range(-ARENA_HALF + 5.0, ARENA_HALF - 5.0);
                let z = rng.range(6.0, ARENA_HALF - 6.0);
                let w = rng.range(0.9, 2.2);
                let h = if i % 3 == 0 { 2.1 } else { 1.3 };
                let d = rng.range(0.9, 2.2);
                let blocks_lane = [(19.0_f32, 14.0_f32), (-19.0, 14.0)]
                    .iter()
                    .any(|&(lx, lz)| {
                        (x - lx).abs() < w + 1.6 && (z - lz).abs() < d + 1.6
                    });
                if blocks_lane {
                    continue;
                }
                for sz in [1.0_f32, -1.0] {
                    push(&mut cover, &mut kind, Aabb {
                        min: [x - w, 0.0, sz * z - d],
                        max: [x + w, h, sz * z + d],
                    }, CoverKind::Crate);
                }
            }
            // low-ground trench walls flanking mid
            for sx in [-1.0_f32, 1.0] {
                push(&mut cover, &mut kind, Aabb {
                    min: [sx * 12.0 - 0.6, 0.0, -10.0],
                    max: [sx * 12.0 + 0.6, 1.5, 10.0],
                }, CoverKind::Stone);
            }
        }
        MapKind::Bailey => {
            half = 40.0;
            checkpoints = [[20.0, 0.0], [-20.0, 0.0]];
            // the keep — taller than the arena tower, same stair grammar
            center(&mut cover, &mut kind, 3.0, 6);
            top = 3.0;
            // corner drum towers: solid, unclimbable, fight around them
            for sx in [-1.0_f32, 1.0] {
                for sz in [-1.0_f32, 1.0] {
                    push(&mut cover, &mut kind, Aabb {
                        min: [sx * 30.0 - 2.5, 0.0, sz * 30.0 - 2.5],
                        max: [sx * 30.0 + 2.5, 3.4, sz * 30.0 + 2.5],
                    }, CoverKind::Stone);
                }
            }
            // bailey cross-walls with a wide-open center gate
            for sz in [-1.0_f32, 1.0] {
                for (x0, x1) in [(-28.0_f32, -8.0), (8.0, 28.0)] {
                    push(&mut cover, &mut kind, Aabb {
                        min: [x0, 0.0, sz * 18.0 - 0.6],
                        max: [x1, 2.6, sz * 18.0 + 0.6],
                    }, CoverKind::Stone);
                }
            }
            // RAMPARTS (v6 vertical layering): climbable wall-walks along
            // both flanks with stairs at each end — high ground with two
            // ways up and full exposure while you hold it
            for sx in [-1.0_f32, 1.0] {
                let px = sx * (half - 5.0);
                push(&mut cover, &mut kind, Aabb {
                    min: [px - 2.0, 0.0, -12.0],
                    max: [px + 2.0, 2.6, 12.0],
                }, CoverKind::Stone);
                for s in 0..5 {
                    let h = 0.52 * (5 - s) as f32;
                    let zoff = 12.0 + s as f32 * 1.05;
                    push(&mut cover, &mut kind, Aabb {
                        min: [px - 1.5, 0.0, zoff],
                        max: [px + 1.5, h, zoff + 1.05],
                    }, CoverKind::Stone);
                    push(&mut cover, &mut kind, Aabb {
                        min: [px - 1.5, 0.0, -zoff - 1.05],
                        max: [px + 1.5, h, -zoff],
                    }, CoverKind::Stone);
                }
                // parapet lip on the rampart's inner edge: hard cover up top
                push(&mut cover, &mut kind, Aabb {
                    min: [px - sx * 2.0 - 0.2, 0.0, -12.0],
                    max: [px - sx * 2.0 + 0.2, 3.3, -8.0],
                }, CoverKind::Stone);
                push(&mut cover, &mut kind, Aabb {
                    min: [px - sx * 2.0 - 0.2, 0.0, 8.0],
                    max: [px - sx * 2.0 + 0.2, 3.3, 12.0],
                }, CoverKind::Stone);
            }
            // stables clutter: crates + low walls in the yard, mirrored
            for (x, z, w, h, d) in [
                (6.0_f32, 10.0_f32, 1.2_f32, 1.3_f32, 1.2_f32),
                (-17.0, 7.0, 1.5, 1.3, 1.0),
                (22.0, 5.5, 1.0, 2.1, 1.0),
                (13.0, 24.0, 1.4, 1.3, 1.1),
                (-6.0, 26.0, 1.1, 2.1, 1.1),
            ] {
                for sz in [1.0_f32, -1.0] {
                    push(&mut cover, &mut kind, Aabb {
                        min: [-x - w, 0.0, sz * z - d],
                        max: [-x + w, h, sz * z + d],
                    }, CoverKind::Crate);
                    push(&mut cover, &mut kind, Aabb {
                        min: [x - w, 0.0, -sz * z - d],
                        max: [x + w, h, -sz * z + d],
                    }, CoverKind::Crate);
                }
            }
            // low chapel ruins flanking the keep
            for sx in [-1.0_f32, 1.0] {
                push(&mut cover, &mut kind, Aabb {
                    min: [sx * 12.0 - 0.6, 0.0, -6.0],
                    max: [sx * 12.0 + 0.6, 1.5, 6.0],
                }, CoverKind::Stone);
            }
        }
        MapKind::Gardens => {
            half = 38.0;
            checkpoints = [[24.0, 0.0], [-24.0, 0.0]];
            // the stone gazebo: lower than the keep, easier to storm
            center(&mut cover, &mut kind, 2.4, 5);
            top = 2.4;
            // hedge lanes flanking mid (trimmed, waist-to-shoulder high)
            for sx in [-1.0_f32, 1.0] {
                push(&mut cover, &mut kind, Aabb {
                    min: [sx * 12.0 - 0.5, 0.0, -10.0],
                    max: [sx * 12.0 + 0.5, 1.5, 10.0],
                }, CoverKind::Hedge);
            }
            // cross hedges with center gaps — two rings now
            for sz in [-1.0_f32, 1.0] {
                for (x0, x1) in [(-20.0_f32, -6.0), (6.0, 20.0)] {
                    push(&mut cover, &mut kind, Aabb {
                        min: [x0, 0.0, sz * 20.0 - 0.5],
                        max: [x1, 1.5, sz * 20.0 + 0.5],
                    }, CoverKind::Hedge);
                }
                for (x0, x1) in [(-16.0_f32, -4.0), (4.0, 16.0)] {
                    push(&mut cover, &mut kind, Aabb {
                        min: [x0, 0.0, sz * 28.0 - 0.5],
                        max: [x1, 1.5, sz * 28.0 + 0.5],
                    }, CoverKind::Hedge);
                }
            }
            // garden TERRACES (v6 vertical layering): raised platforms on
            // the mid-line with steps on the center-facing side
            for sz in [-1.0_f32, 1.0] {
                push(&mut cover, &mut kind, Aabb {
                    min: [-4.0, 0.0, sz * 24.0 - 2.0],
                    max: [4.0, 1.2, sz * 24.0 + 2.0],
                }, CoverKind::Stone);
                // steps descend from the terrace edge (|z| = 22) toward
                // center: 1.2 → 0.8 → 0.4, each riser within STEP_UP
                for s in 0..3 {
                    let h = 0.4 * (3 - s) as f32;
                    let near = 22.0 - s as f32 * 0.9; // edge closest to terrace
                    let (z0, z1) = if sz > 0.0 {
                        (near - 0.9, near)
                    } else {
                        (-near, -(near - 0.9))
                    };
                    push(&mut cover, &mut kind, Aabb {
                        min: [-2.0, 0.0, z0],
                        max: [2.0, h, z1],
                    }, CoverKind::Stone);
                }
            }
            // ruined garden walls on the diagonals (vaultable)
            for s in [-1.0_f32, 1.0] {
                push(&mut cover, &mut kind, Aabb {
                    min: [s * 16.0 - 4.0, 0.0, s * 7.0 - 0.5],
                    max: [s * 16.0 + 4.0, 1.1, s * 7.0 + 0.5],
                }, CoverKind::Stone);
                push(&mut cover, &mut kind, Aabb {
                    min: [-s * 16.0 - 4.0, 0.0, s * 11.0 - 0.5],
                    max: [-s * 16.0 + 4.0, 1.1, s * 11.0 + 0.5],
                }, CoverKind::Stone);
                // fountain-basin ruins on the far flanks
                push(&mut cover, &mut kind, Aabb {
                    min: [s * 30.0 - 2.0, 0.0, -2.0],
                    max: [s * 30.0 + 2.0, 0.9, 2.0],
                }, CoverKind::Stone);
            }
            // old trees: trunk is the collider, the client crowns them
            for (x, z) in [
                (23.0_f32, 9.0_f32),
                (-23.0, -9.0),
                (-23.0, 9.0),
                (23.0, -9.0),
                (9.0, 31.0),
                (-9.0, -31.0),
                (17.0, -15.0),
                (-17.0, 15.0),
            ] {
                push(&mut cover, &mut kind, Aabb {
                    min: [x - 0.3, 0.0, z - 0.3],
                    max: [x + 0.3, 2.8, z + 0.3],
                }, CoverKind::Tree);
            }
        }
    }
    MapLayout {
        cover,
        kind,
        center_top: top,
        half,
        checkpoints,
    }
}

// --------------------------------------------------------------- fighters

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HitZone {
    Legs,
    Torso,
    Arms,
    Head,
}

impl HitZone {
    pub fn name(self) -> &'static str {
        match self {
            HitZone::Legs => "legs",
            HitZone::Torso => "torso",
            HitZone::Arms => "arms",
            HitZone::Head => "HEAD",
        }
    }
    pub fn mult(self) -> f32 {
        match self {
            HitZone::Head => HEAD_MULT,
            HitZone::Torso => 1.0,
            HitZone::Arms => ARM_MULT,
            HitZone::Legs => LEG_MULT,
        }
    }
}

// ------------------------------------------------------------- difficulty

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Difficulty {
    Easy,
    Normal,
    Hard,
}

impl Difficulty {
    pub fn name(self) -> &'static str {
        match self {
            Difficulty::Easy => "EASY",
            Difficulty::Normal => "NORMAL",
            Difficulty::Hard => "HARD",
        }
    }
    pub const ALL: [Difficulty; 3] = [Difficulty::Easy, Difficulty::Normal, Difficulty::Hard];
}

/// Bot brain tuning per difficulty: aim, reflexes, and aggression.
pub struct BotParams {
    pub aim_sigma: f32,
    pub reaction_s: f32,
    pub engage_range: f32,
    pub aggression: f32,
}

pub fn bot_params(d: Difficulty) -> BotParams {
    match d {
        Difficulty::Easy => BotParams {
            aim_sigma: 0.055,
            reaction_s: 0.45,
            engage_range: 22.0,
            aggression: 0.6,
        },
        Difficulty::Normal => BotParams {
            aim_sigma: 0.030,
            reaction_s: 0.28,
            engage_range: 35.0,
            aggression: 1.0,
        },
        Difficulty::Hard => BotParams {
            aim_sigma: 0.014,
            reaction_s: 0.15,
            engage_range: 50.0,
            aggression: 1.4,
        },
    }
}

// ------------------------------------------------------------ match config

/// Player-picked loadout: [primary, secondary, special]. The SHIELD is
/// always carried in its own dedicated slot — it cannot be dropped.
pub type Loadout = [GunKind; 3];

pub const DEFAULT_LOADOUT: Loadout = [GunKind::M4, GunKind::Glock, GunKind::Awm];

#[derive(Clone, Copy, Debug)]
pub struct MatchConfig {
    pub seed: u64,
    pub per_team: usize, // 5..=8 (owner's cap: 8v8)
    pub mode: Mode,
    pub map: MapKind,
    pub difficulty: Difficulty,
    pub loadout: Loadout,
}

impl Default for MatchConfig {
    fn default() -> Self {
        MatchConfig {
            seed: 0x7EA9,
            per_team: 5,
            mode: Mode::Tdm,
            map: MapKind::Arena,
            difficulty: Difficulty::Normal,
            loadout: DEFAULT_LOADOUT,
        }
    }
}

/// A capturable forward-spawn checkpoint. Stand in the ring uncontested
/// to charge it toward your team; once owned, your dead respawn there.
#[derive(Clone, Debug)]
pub struct Checkpoint {
    pub pos: [f32; 3],
    pub owner: Option<Team>,
    /// −CAP..+CAP: positive charges Blue, negative Red.
    pub charge: f32,
}

#[derive(Clone, Debug)]
pub struct Fighter {
    pub name: &'static str,
    pub team: Team,
    /// The ACTIVE weapon (mirror of `inventory[active]` for fast access).
    pub gun: GunKind,
    /// [primary, secondary, special] — the shield is its own stance below.
    pub inventory: Loadout,
    /// Per-slot (magazine, reserve) so switching keeps each gun's state.
    pub slot_ammo: [(u32, u32); 3],
    pub active: usize,
    /// Shield raised: front-arc protection, no firing, slow walk.
    pub shield_up: bool,
    /// −1..+1 sideways lean (peek): shifts the muzzle, trims recoil.
    pub lean: f32,
    pub crouch: bool,
    pub switch_t: f32,
    pub pos: [f32; 3], // feet
    pub vel: [f32; 2], // xz
    pub vy: f32,
    pub grounded: bool,
    pub yaw: f32,
    /// >0 → mid-somersault: fast, low, can't shoot. Set by the dodge key
    /// or automatically by a hard landing (parkour breakfall).
    pub roll_t: f32,
    pub roll_cd: f32,
    pub roll_dir: [f32; 2],
    pub health: f32,
    pub armor: f32, // robot armor absorbs first
    pub ammo: u32,
    pub reserve: u32,
    pub reload_t: f32,
    pub fire_cd: f32,
    pub bloom: f32,
    pub respawn_t: f32,
    pub protect_t: f32,
    pub kills: u32,
    pub deaths: u32,
    pub hits_dealt: u32,
    // bot brain
    pub waypoint: [f32; 2],
    pub strafe_phase: f32,
    pub los_time: f32,
    pub think_offset: u32,
}

impl Fighter {
    pub fn alive(&self) -> bool {
        self.respawn_t <= 0.0 && self.health > 0.0
    }
    pub fn height(&self) -> f32 {
        if self.roll_t > 0.0 {
            ROLL_HEIGHT
        } else if self.crouch {
            CROUCH_HEIGHT
        } else {
            BODY_HEIGHT
        }
    }
    pub fn armed(&self) -> bool {
        self.gun != GunKind::Fists
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct PlayerCmd {
    pub move_x: f32,
    pub move_z: f32,
    pub sprint: bool,
    pub yaw: f32,
    pub aim: [f32; 3],
    pub shoot: bool,
    pub reload: bool,
    pub ads: bool,
    pub crouch: bool,
    pub jump: bool,
    /// Duck-spin dodge: a fast forward somersault in the move direction.
    pub dodge: bool,
    /// Select an inventory slot (0 primary, 1 secondary, 2 special).
    pub slot: Option<u8>,
    /// Toggle the shield stance (E). Edge-triggered like jump/reload.
    pub shield: bool,
    /// −1..+1 lean input.
    pub lean: f32,
}

// ---------------------------------------------------------------- events

#[derive(Clone, Debug)]
pub struct KillEvent {
    pub killer: usize,
    pub victim: usize,
    pub headshot: bool,
}

#[derive(Clone, Debug)]
pub struct HitEvent {
    pub shooter: usize,
    pub victim: usize,
    pub zone: HitZone,
    pub damage: f32,
    /// True if the victim's raised shield ate most of this hit.
    pub shielded: bool,
    pub from: [f32; 3],
    /// impact point — part of the event API even when the client skips it
    #[allow(dead_code)]
    pub at: [f32; 3],
    pub fatal: bool,
}

#[derive(Clone, Debug)]
pub struct Impact {
    pub at: [f32; 3],
    pub normal: [f32; 3],
}

#[derive(Clone, Debug)]
pub struct Tracer {
    pub from: [f32; 3],
    pub to: [f32; 3],
    pub team: Team,
    pub ttl: f32,
}

/// Arrow / thrown spear in flight (or stuck).
#[derive(Clone, Debug)]
pub struct Missile {
    /// stable identity — part of the sim API even when the client pools by index
    #[allow(dead_code)]
    pub id: u32,
    pub pos: [f32; 3],
    pub vel: [f32; 3],
    pub team: Team,
    pub shooter: usize,
    pub damage: f32,
    pub is_spear: bool,
    pub stuck_t: Option<f32>,
}

// ---------------------------------------------------------------- pickups

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PickupKind {
    Health,
    Ammo,
    RobotArmor,
}

#[derive(Clone, Debug)]
pub struct Pickup {
    pub kind: PickupKind,
    pub pos: [f32; 3],
    pub respawn_t: f32, // >0 → waiting to respawn (hidden)
}

// ------------------------------------------------------------------ modes

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Tdm,
    Koth,
}

pub struct TdmSim {
    pub cfg: MatchConfig,
    pub mode: Mode,
    pub map: MapKind,
    /// Playable half-extent — bigger maps have more room (v6 §10).
    pub half: f32,
    pub fighters: Vec<Fighter>,
    pub cover: Vec<Aabb>,
    pub cover_kind: Vec<CoverKind>,
    pub checkpoints: Vec<Checkpoint>,
    pub pickups: Vec<Pickup>,
    pub missiles: Vec<Missile>,
    pub score: [f32; 2], // kills (TDM) or hill-seconds (KOTH)
    pub kill_feed: Vec<(KillEvent, f32)>,
    pub hits: Vec<(HitEvent, f32)>,
    pub impacts: Vec<(Impact, f32)>,
    pub tracers: Vec<Tracer>,
    pub t: f32,
    pub match_t: f32,
    pub overtime: bool,
    pub round_over_t: Option<f32>,
    pub winner: Option<Team>,
    pub hill: [f32; 3],
    pub player: usize,
    rng: Pcg32,
    tick: u64,
    next_missile_id: u32,
}

const BLUE_NAMES: [&str; 8] = [
    "You", "Brasidas", "Cleon", "Pelops", "Dion", "Nikias", "Phormio", "Lamachos",
];
const RED_NAMES: [&str; 8] = [
    "Xerxes", "Otanes", "Mardon", "Artax", "Hydarnes", "Datis", "Bessus", "Tigran",
];

/// A fresh (magazine, reserve) pair for every slot of a loadout.
fn fresh_ammo(inv: Loadout) -> [(u32, u32); 3] {
    [
        (gun(inv[0]).mag, gun(inv[0]).reserve),
        (gun(inv[1]).mag, gun(inv[1]).reserve),
        (gun(inv[2]).mag, gun(inv[2]).reserve),
    ]
}

impl TdmSim {
    pub fn new(cfg: MatchConfig) -> Self {
        let per_team = cfg.per_team.clamp(1, 8);
        let mut rng = Pcg32::new(cfg.seed, 0x7D7D);
        let layout = build_map(cfg.map, &mut rng);
        let (cover, cover_kind, center_top, half) =
            (layout.cover, layout.kind, layout.center_top, layout.half);

        // ---- pickups: health / ammo / the robot armor -------------------
        // (weapon pads are gone in v6 — you BRING your loadout; the map
        // now feeds you consumables and the armor)
        let mut pickups = Vec::new();
        pickups.push(Pickup {
            kind: PickupKind::RobotArmor,
            pos: [0.0, center_top, 0.0],
            respawn_t: 0.0,
        });
        for (kind, x, z) in [
            (PickupKind::Health, -19.0, 14.0),
            (PickupKind::Health, 19.0, -14.0),
            (PickupKind::Ammo, -19.0, -14.0),
            (PickupKind::Ammo, 19.0, 14.0),
        ] {
            pickups.push(Pickup {
                kind,
                pos: [x, 0.0, z],
                respawn_t: 0.0,
            });
        }
        // snap pickups AND checkpoints onto the terrain under them — a
        // lane that crosses a plateau must sit ON it, not inside it
        let support_at = |cover: &Vec<Aabb>, x: f32, z: f32| {
            let mut y = 0.0_f32;
            for c in cover {
                if x > c.min[0] && x < c.max[0] && z > c.min[2] && z < c.max[2] && c.max[1] > y {
                    y = c.max[1];
                }
            }
            y
        };
        for p in &mut pickups {
            p.pos[1] = p.pos[1].max(support_at(&cover, p.pos[0], p.pos[2]));
        }
        let checkpoints = layout
            .checkpoints
            .iter()
            .map(|&[x, z]| Checkpoint {
                pos: [x, support_at(&cover, x, z), z],
                owner: None,
                charge: 0.0,
            })
            .collect();

        let mut fighters = Vec::new();
        for team_i in 0..2 {
            let team = if team_i == 0 { Team::Blue } else { Team::Red };
            for k in 0..per_team {
                let (pos, yaw) = spawn_point(team, k, half);
                let is_player = team_i == 0 && k == 0;
                let idx = team_i * per_team + k;
                // v6: everyone spawns with a LOADOUT. The player brings
                // the one picked on the loadout screen; bots run varied
                // kits off a deterministic rotation.
                let inv: Loadout = if is_player {
                    cfg.loadout
                } else {
                    [
                        PRIMARIES[idx % PRIMARIES.len()],
                        SECONDARIES[idx % SECONDARIES.len()],
                        SPECIALS[idx % SPECIALS.len()],
                    ]
                };
                let g0 = inv[0];
                let name = if team == Team::Blue {
                    BLUE_NAMES[k % 8]
                } else {
                    RED_NAMES[k % 8]
                };
                fighters.push(Fighter {
                    name,
                    team,
                    gun: g0,
                    inventory: inv,
                    slot_ammo: fresh_ammo(inv),
                    active: 0,
                    shield_up: false,
                    lean: 0.0,
                    crouch: false,
                    switch_t: 0.0,
                    pos,
                    vel: [0.0, 0.0],
                    vy: 0.0,
                    grounded: true,
                    yaw,
                    roll_t: 0.0,
                    roll_cd: 0.0,
                    roll_dir: [0.0, 1.0],
                    health: MAX_HEALTH,
                    armor: 0.0,
                    ammo: gun(g0).mag,
                    reserve: gun(g0).reserve,
                    reload_t: 0.0,
                    fire_cd: 0.0,
                    bloom: 0.0,
                    respawn_t: 0.0,
                    protect_t: SPAWN_PROTECT_S,
                    kills: 0,
                    deaths: 0,
                    hits_dealt: 0,
                    waypoint: [rng.range(-12.0, 12.0), rng.range(-8.0, 8.0)],
                    strafe_phase: rng.range(0.0, 6.28),
                    los_time: 0.0,
                    think_offset: idx as u32 % 12,
                });
            }
        }
        TdmSim {
            cfg,
            mode: cfg.mode,
            map: cfg.map,
            half,
            fighters,
            cover,
            cover_kind,
            checkpoints,
            pickups,
            missiles: Vec::new(),
            score: [0.0, 0.0],
            kill_feed: Vec::new(),
            hits: Vec::new(),
            impacts: Vec::new(),
            tracers: Vec::new(),
            t: 0.0,
            match_t: MATCH_LEN_S,
            overtime: false,
            round_over_t: None,
            winner: None,
            hill: [0.0, center_top, 0.0], // the hill IS the center top
            player: 0,
            rng,
            tick: 0,
            next_missile_id: 0,
        }
    }

    pub fn team_idx(team: Team) -> usize {
        match team {
            Team::Blue => 0,
            Team::Red => 1,
        }
    }

    pub fn step(&mut self, cmd: PlayerCmd) {
        self.tick += 1;
        self.t += DT;
        for tr in &mut self.tracers {
            tr.ttl -= DT;
        }
        self.tracers.retain(|t| t.ttl > 0.0);
        for (_, ttl) in &mut self.kill_feed {
            *ttl -= DT;
        }
        self.kill_feed.retain(|(_, ttl)| *ttl > 0.0);
        for (_, ttl) in &mut self.hits {
            *ttl -= DT;
        }
        self.hits.retain(|(_, ttl)| *ttl > 0.0);
        for (_, ttl) in &mut self.impacts {
            *ttl -= DT;
        }
        self.impacts.retain(|(_, ttl)| *ttl > 0.0);

        if let Some(over) = self.round_over_t {
            if self.t - over > 7.0 {
                let seed = self.rng.next_u32() as u64;
                *self = TdmSim::new(MatchConfig {
                    seed: seed ^ 0x9E37,
                    ..self.cfg
                });
            }
            return;
        }

        // ---- match clock + overtime ------------------------------------
        self.match_t -= DT;
        if self.match_t <= 0.0 {
            let (b, r) = (self.score[0], self.score[1]);
            if (b - r).abs() > 0.01 {
                self.finish(if b > r { Team::Blue } else { Team::Red });
            } else if !self.overtime {
                self.overtime = true;
                self.match_t = OVERTIME_S; // sudden death: next point wins
            } else {
                self.finish(Team::Blue); // exhausted overtime: blue by honor
            }
        }

        // ---- timers, respawns ------------------------------------------
        for i in 0..self.fighters.len() {
            let f = &mut self.fighters[i];
            f.fire_cd = (f.fire_cd - DT).max(0.0);
            f.protect_t = (f.protect_t - DT).max(0.0);
            f.switch_t = (f.switch_t - DT).max(0.0);
            f.roll_cd = (f.roll_cd - DT).max(0.0);
            f.bloom = (f.bloom - 0.02 * DT * 6.0).max(0.0);
            if f.reload_t > 0.0 {
                f.reload_t -= DT;
                if f.reload_t <= 0.0 {
                    // top up from reserve, KEEPING what's still chambered —
                    // dumping the part-full mag used to strictly lose ammo
                    let spec = gun(f.gun);
                    let total = f.ammo + f.reserve;
                    f.ammo = spec.mag.min(total);
                    f.reserve = total - f.ammo;
                }
            }
            if f.respawn_t > 0.0 {
                f.respawn_t -= DT;
                if f.respawn_t <= 0.0 {
                    let slot = i % self.cfg.per_team.max(1);
                    let (mut pos, mut yaw) = spawn_point(f.team, slot, self.half);
                    // "check back": an owned checkpoint pulls the respawn
                    // forward — you rejoin the fight where your team holds
                    if let Some(cp) = self
                        .checkpoints
                        .iter()
                        .find(|c| c.owner == Some(f.team))
                    {
                        let ang = slot as f32 * 1.3;
                        pos = [
                            cp.pos[0] + ang.cos() * 1.2,
                            cp.pos[1],
                            cp.pos[2] + ang.sin() * 1.2,
                        ];
                        // face the arena center from the forward spawn
                        yaw = (-pos[0]).atan2(-pos[2]);
                    }
                    f.pos = pos;
                    f.yaw = yaw;
                    f.vy = 0.0;
                    f.grounded = true;
                    f.health = MAX_HEALTH;
                    f.armor = 0.0;
                    // the full loadout comes back with you
                    f.slot_ammo = fresh_ammo(f.inventory);
                    f.active = 0;
                    f.gun = f.inventory[0];
                    f.ammo = f.slot_ammo[0].0;
                    f.reserve = f.slot_ammo[0].1;
                    f.shield_up = false;
                    f.lean = 0.0;
                    f.protect_t = SPAWN_PROTECT_S;
                    f.vel = [0.0, 0.0];
                    f.crouch = false;
                    f.roll_t = 0.0;
                    f.roll_cd = 0.0;
                }
            }
        }

        // ---- pickups respawn + collection ------------------------------
        for pi in 0..self.pickups.len() {
            if self.pickups[pi].respawn_t > 0.0 {
                self.pickups[pi].respawn_t -= DT;
                continue;
            }
            let ppos = self.pickups[pi].pos;
            let kind = self.pickups[pi].kind;
            let mut taken = false;
            for i in 0..self.fighters.len() {
                let f = &self.fighters[i];
                if !f.alive() {
                    continue;
                }
                let dx = f.pos[0] - ppos[0];
                let dz = f.pos[2] - ppos[2];
                let dy = f.pos[1] - ppos[1];
                if dx * dx + dz * dz < PICKUP_RADIUS * PICKUP_RADIUS && dy.abs() < 1.4 {
                    match kind {
                        PickupKind::Health => {
                            let f = &mut self.fighters[i];
                            if f.health >= MAX_HEALTH {
                                continue;
                            }
                            f.health = (f.health + 50.0).min(MAX_HEALTH);
                        }
                        PickupKind::Ammo => {
                            let f = &mut self.fighters[i];
                            if !f.armed() {
                                continue;
                            }
                            f.reserve += gun(f.gun).mag * 2;
                        }
                        PickupKind::RobotArmor => {
                            let f = &mut self.fighters[i];
                            f.armor = ROBOT_ARMOR_HP;
                        }
                    }
                    taken = true;
                    break;
                }
            }
            if taken {
                self.pickups[pi].respawn_t = match kind {
                    PickupKind::Health | PickupKind::Ammo => 20.0,
                    PickupKind::RobotArmor => 45.0,
                };
            }
        }

        // ---- player -----------------------------------------------------
        let p = self.player;
        if self.fighters[p].alive() {
            self.fighters[p].crouch = cmd.crouch;
            self.fighters[p].lean = cmd.lean.clamp(-1.0, 1.0);
            // slot select (number keys) + shield toggle (E)
            if let Some(s) = cmd.slot {
                self.switch_slot(p, s as usize);
            }
            if cmd.shield {
                let f = &mut self.fighters[p];
                f.shield_up = !f.shield_up;
            }
            let scoped = cmd.ads && gun(self.fighters[p].gun).scoped;
            let mut speed = if cmd.sprint { SPRINT_SPEED } else { MOVE_SPEED };
            if cmd.crouch {
                speed *= CROUCH_SPEED_MULT;
            }
            // the raised shield owns the pace — ADS/scope mults don't
            // stack on top (you're not sighting anything behind a plate)
            if self.fighters[p].shield_up {
                speed *= SHIELD_SPEED_MULT;
            } else if scoped {
                speed *= SCOPED_SPEED_MULT; // AWM glass: a crawl
            } else if cmd.ads {
                speed *= ADS_SPEED_MULT;
            }
            if self.fighters[p].armor > 0.0 {
                speed *= ROBOT_SPEED_MULT;
            }
            let mag = (cmd.move_x * cmd.move_x + cmd.move_z * cmd.move_z)
                .sqrt()
                .max(1e-6);
            let cap = if mag > 1.0 { mag } else { 1.0 };
            self.fighters[p].vel = [cmd.move_x / cap * speed, cmd.move_z / cap * speed];
            self.fighters[p].yaw = cmd.yaw;
            // duck-spin dodge: somersault in the move direction (facing if
            // standing still); grounded only, gated by a short cooldown
            if cmd.dodge {
                let f = &mut self.fighters[p];
                if f.grounded && f.roll_t <= 0.0 && f.roll_cd <= 0.0 {
                    let m = (cmd.move_x * cmd.move_x + cmd.move_z * cmd.move_z).sqrt();
                    f.roll_dir = if m > 0.2 {
                        [cmd.move_x / m, cmd.move_z / m]
                    } else {
                        [f.yaw.sin(), f.yaw.cos()]
                    };
                    f.roll_t = ROLL_S;
                    f.roll_cd = ROLL_S + ROLL_CD_S;
                }
            }
            if cmd.jump && self.fighters[p].grounded && self.fighters[p].roll_t <= 0.0 {
                let f = &mut self.fighters[p];
                f.vy = JUMP_SPEED;
                f.pos[1] += 0.05; // clear the support clamp so the ascent integrates
                f.grounded = false;
            }
            if cmd.reload {
                self.try_reload(p);
            }
            if cmd.shoot {
                self.try_fire(p, cmd.aim, cmd.ads);
            }
        } else {
            self.fighters[p].vel = [0.0, 0.0];
        }

        // ---- bots -------------------------------------------------------
        for i in 0..self.fighters.len() {
            if i == p || !self.fighters[i].alive() {
                continue;
            }
            if (self.tick + self.fighters[i].think_offset as u64) % 12 == 0 {
                self.bot_think(i);
            }
            self.bot_act(i);
        }

        // ---- integrate: XZ + walls, then gravity/steps ------------------
        // (dead fighters integrate too — velocity is zeroed on death, so
        // this just lets a mid-air corpse fall instead of hanging)
        let half = self.half;
        for i in 0..self.fighters.len() {
            // mid-roll: the somersault owns the velocity (player and bots
            // alike — bots roll off hard landings too)
            {
                let f = &mut self.fighters[i];
                if f.roll_t > 0.0 {
                    f.roll_t -= DT;
                    if f.alive() {
                        f.vel = [f.roll_dir[0] * ROLL_SPEED, f.roll_dir[1] * ROLL_SPEED];
                    }
                }
            }
            let (nx, nz) = {
                let f = &self.fighters[i];
                (f.pos[0] + f.vel[0] * DT, f.pos[2] + f.vel[1] * DT)
            };
            let y = self.fighters[i].pos[1];
            let mut px = nx.clamp(-half + 0.5, half - 0.5);
            let mut pz = nz.clamp(-half + 0.5, half - 0.5);
            // walls you cannot step onto push you out; walkable tops don't
            for c in &self.cover {
                if c.max[1] <= y + STEP_UP {
                    continue; // climbable — handled by support
                }
                if y >= c.max[1] - 0.01 {
                    continue; // already above it
                }
                let cx = px.clamp(c.min[0], c.max[0]);
                let cz = pz.clamp(c.min[2], c.max[2]);
                let (dx, dz) = (px - cx, pz - cz);
                let d2 = dx * dx + dz * dz;
                if d2 < BODY_RADIUS * BODY_RADIUS {
                    let d = d2.sqrt().max(1e-4);
                    let push = BODY_RADIUS - d;
                    if d > 1e-3 {
                        px += dx / d * push;
                        pz += dz / d * push;
                    } else {
                        pz += push;
                    }
                }
            }
            // vertical: stand on the tallest reachable support, else fall
            let f = &mut self.fighters[i];
            f.pos[0] = px;
            f.pos[2] = pz;
            let support = {
                let y0 = f.pos[1];
                let mut s = 0.0_f32;
                for c in &self.cover {
                    if px > c.min[0] - BODY_RADIUS * 0.4
                        && px < c.max[0] + BODY_RADIUS * 0.4
                        && pz > c.min[2] - BODY_RADIUS * 0.4
                        && pz < c.max[2] + BODY_RADIUS * 0.4
                        && c.max[1] <= y0 + STEP_UP
                        && c.max[1] > s
                    {
                        s = c.max[1];
                    }
                }
                s
            };
            if f.pos[1] > support + 0.02 {
                f.vy -= GRAVITY * DT;
                f.pos[1] = (f.pos[1] + f.vy * DT).max(support);
                if f.pos[1] <= support {
                    // hard landing → automatic parkour breakfall: the fall
                    // rolls out along the ground instead of stopping dead
                    let impact = f.vy;
                    f.vy = 0.0;
                    f.grounded = true;
                    if impact < -HARD_LANDING_VY && f.alive() && f.roll_t <= 0.0 {
                        let sp = (f.vel[0] * f.vel[0] + f.vel[1] * f.vel[1]).sqrt();
                        f.roll_dir = if sp > 0.5 {
                            [f.vel[0] / sp, f.vel[1] / sp]
                        } else {
                            [f.yaw.sin(), f.yaw.cos()]
                        };
                        f.roll_t = ROLL_S;
                        f.roll_cd = ROLL_S + ROLL_CD_S;
                    }
                } else {
                    f.grounded = false;
                }
            } else {
                f.pos[1] = support;
                f.vy = 0.0;
                f.grounded = true;
            }
        }

        // ---- missiles (arrows / spears) --------------------------------
        self.step_missiles();

        // ---- respawn checkpoints ("check back") ------------------------
        // Stand in the ring uncontested to charge it toward your team;
        // contested rings freeze. An owned ring is a forward spawn.
        for cp in &mut self.checkpoints {
            let (mut blue, mut red) = (0u32, 0u32);
            for f in &self.fighters {
                if !f.alive() {
                    continue;
                }
                let dx = f.pos[0] - cp.pos[0];
                let dz = f.pos[2] - cp.pos[2];
                if dx * dx + dz * dz < CHECKPOINT_RADIUS * CHECKPOINT_RADIUS
                    && (f.pos[1] - cp.pos[1]).abs() < 2.0
                {
                    match f.team {
                        Team::Blue => blue += 1,
                        Team::Red => red += 1,
                    }
                }
            }
            if blue > 0 && red == 0 {
                cp.charge = (cp.charge + DT).min(CHECKPOINT_CAP_S);
                if cp.charge >= CHECKPOINT_CAP_S {
                    cp.owner = Some(Team::Blue);
                }
            } else if red > 0 && blue == 0 {
                cp.charge = (cp.charge - DT).max(-CHECKPOINT_CAP_S);
                if cp.charge <= -CHECKPOINT_CAP_S {
                    cp.owner = Some(Team::Red);
                }
            }
        }

        // ---- KOTH scoring ----------------------------------------------
        if self.mode == Mode::Koth {
            let mut on_hill = [0u32; 2];
            for f in &self.fighters {
                if !f.alive() {
                    continue;
                }
                let dx = f.pos[0] - self.hill[0];
                let dz = f.pos[2] - self.hill[2];
                if dx * dx + dz * dz < HILL_RADIUS * HILL_RADIUS
                    && (f.pos[1] - self.hill[1]).abs() < 1.5
                {
                    on_hill[Self::team_idx(f.team)] += 1;
                }
            }
            if on_hill[0] > 0 && on_hill[1] == 0 {
                self.score[0] += DT;
            } else if on_hill[1] > 0 && on_hill[0] == 0 {
                self.score[1] += DT;
            }
            if self.score[0] >= KOTH_TARGET_S {
                self.finish(Team::Blue);
            } else if self.score[1] >= KOTH_TARGET_S {
                self.finish(Team::Red);
            }
        }
    }

    fn finish(&mut self, winner: Team) {
        if self.round_over_t.is_none() {
            self.round_over_t = Some(self.t);
            self.winner = Some(winner);
        }
    }

    /// Swap to another loadout slot; each slot keeps its own magazine.
    fn switch_slot(&mut self, i: usize, s: usize) {
        if s >= 3 {
            return;
        }
        let f = &mut self.fighters[i];
        if s == f.active || f.inventory[s] == GunKind::Fists || !f.alive() {
            return;
        }
        f.slot_ammo[f.active] = (f.ammo, f.reserve);
        f.active = s;
        f.gun = f.inventory[s];
        let (a, r) = f.slot_ammo[s];
        f.ammo = a;
        f.reserve = r;
        f.reload_t = 0.0;
        f.switch_t = SWITCH_S;
        f.shield_up = false; // both hands on the new weapon
    }

    fn try_reload(&mut self, i: usize) {
        let f = &mut self.fighters[i];
        if !f.armed() {
            return;
        }
        let spec = gun(f.gun);
        if f.reload_t <= 0.0 && f.ammo < spec.mag && f.reserve > 0 {
            f.reload_t = spec.reload_s;
        }
    }

    /// Where a fighter's shots actually leave from: crouch lowers the
    /// eye, lean shifts it sideways. The client mirrors this for the
    /// crosshair ray and the arc preview so aim and muzzle agree.
    pub fn muzzle_origin(&self, i: usize) -> [f32; 3] {
        let f = &self.fighters[i];
        // screen-right under this game's yaw convention (matches the
        // playtest-verified A/D strafe mapping): lean +1 peeks RIGHT
        let right = [-f.yaw.cos(), 0.0, f.yaw.sin()];
        [
            f.pos[0] + right[0] * LEAN_SHIFT * f.lean,
            f.pos[1] + EYE_REL.min(f.height() - 0.12),
            f.pos[2] + right[2] * LEAN_SHIFT * f.lean,
        ]
    }

    fn spawn_missile(&mut self, o: [f32; 3], d: [f32; 3], v0: f32, dmg: f32, i: usize) {
        let id = self.next_missile_id;
        self.next_missile_id += 1;
        let is_spear = self.fighters[i].gun == GunKind::Spear;
        self.missiles.push(Missile {
            id,
            pos: o,
            vel: [d[0] * v0, d[1] * v0, d[2] * v0],
            team: self.fighters[i].team,
            shooter: i,
            damage: dmg,
            is_spear,
            stuck_t: None,
        });
    }

    fn try_fire(&mut self, i: usize, aim: [f32; 3], ads: bool) -> bool {
        {
            let f = &self.fighters[i];
            if !f.armed()
                || f.fire_cd > 0.0
                || f.reload_t > 0.0
                || f.switch_t > 0.0
                || f.ammo == 0
                || !f.alive()
                || f.roll_t > 0.0 // no shooting mid-somersault
                || f.shield_up // the shield takes both hands
            {
                return false;
            }
        }
        let spec = gun(self.fighters[i].gun);
        let moving = {
            let f = &self.fighters[i];
            (f.vel[0] * f.vel[0] + f.vel[1] * f.vel[1]).sqrt() > 0.5
        };
        let mut spread =
            spec.spread + if moving { spec.spread_move } else { 0.0 } + self.fighters[i].bloom;
        if ads {
            spread *= ADS_SPREAD_MULT;
        }
        if self.fighters[i].crouch {
            spread *= CROUCH_SPREAD_MULT;
        }
        // lean shifts the muzzle sideways off the body line and steadies
        // the shoulder a touch (recoil ×0.8 while leaning)
        let lean = self.fighters[i].lean;
        let o = self.muzzle_origin(i);
        {
            let f = &mut self.fighters[i];
            f.ammo -= 1;
            f.fire_cd = spec.fire_period;
            // opening fire drops your spawn protection — no shooting from
            // behind the untargetable shimmer
            f.protect_t = 0.0;
            let kick = if lean.abs() > 0.1 {
                spec.kick * LEAN_RECOIL_MULT
            } else {
                spec.kick
            };
            f.bloom = (f.bloom + kick).min(0.05);
            if f.ammo == 0 && spec.projectile.is_some() && f.reserve > 0 {
                // nock the next arrow / heft the next spear automatically
                f.reload_t = spec.reload_s;
            }
        }
        if let Some((v0, dmg)) = spec.projectile {
            let (ex, ey) = (
                self.rng.range(-spread, spread),
                self.rng.range(-spread, spread),
            );
            let d = perturb(normalize(aim), ex, ey);
            self.spawn_missile(o, d, v0, dmg, i);
            return true;
        }
        // ---- hitscan: one trace per pellet (shotguns fire a cone) ------
        for _pellet in 0..spec.pellets.max(1) {
            let (ex, ey) = (
                self.rng.range(-spread, spread),
                self.rng.range(-spread, spread),
            );
            let d = perturb(normalize(aim), ex, ey);
            let mut t_hit = 200.0_f32;
            let mut hit_normal = [0.0, 1.0, 0.0];
            for c in &self.cover {
                if let Some((t, n)) = c.ray_hit(o, d, t_hit) {
                    if t < t_hit {
                        t_hit = t;
                        hit_normal = n;
                    }
                }
            }
            let shooter_team = self.fighters[i].team;
            let mut victim: Option<(usize, f32, f32)> = None;
            for (j, g) in self.fighters.iter().enumerate() {
                if j == i || g.team == shooter_team || !g.alive() || g.protect_t > 0.0 {
                    continue;
                }
                if let Some((t, hit_y)) = ray_vs_cylinder(o, d, g.pos, BODY_RADIUS, g.height()) {
                    if t < t_hit && victim.map_or(true, |(_, bt, _)| t < bt) {
                        victim = Some((j, t, hit_y));
                    }
                }
            }
            let end_t = victim.map(|(_, t, _)| t).unwrap_or(t_hit);
            let end = [o[0] + d[0] * end_t, o[1] + d[1] * end_t, o[2] + d[2] * end_t];
            self.tracers.push(Tracer {
                from: o,
                to: end,
                team: shooter_team,
                ttl: 0.06,
            });
            match victim {
                Some((j, _, hit_y)) => {
                    self.apply_hit(i, j, hit_y, end);
                }
                None => {
                    if end_t < 199.0 {
                        self.impacts.push((
                            Impact {
                                at: end,
                                normal: hit_normal,
                            },
                            30.0, // bullet marks linger
                        ));
                    }
                }
            }
        }
        true
    }

    /// Damage reduction from a raised shield, if the attack comes through
    /// the front arc. Sides and rear ignore the shield ENTIRELY — flanking
    /// is the counter-play, by design.
    fn shield_block(&self, j: usize, attack_from: [f32; 3]) -> Option<f32> {
        let v = &self.fighters[j];
        // no plate discipline mid-somersault — a rolling shield blocks
        // nothing (otherwise the roll would be a 95%-immune dash)
        if !v.shield_up || v.roll_t > 0.0 {
            return None;
        }
        let dx = attack_from[0] - v.pos[0];
        let dz = attack_from[2] - v.pos[2];
        let len = (dx * dx + dz * dz).sqrt();
        if len < 1e-3 {
            return None;
        }
        let facing = [v.yaw.sin(), v.yaw.cos()];
        let dot = (facing[0] * dx + facing[1] * dz) / len;
        if dot > SHIELD_ARC_COS {
            Some(if v.crouch {
                SHIELD_BLOCK_CROUCH
            } else {
                SHIELD_BLOCK_STAND
            })
        } else {
            None
        }
    }

    fn apply_hit(&mut self, i: usize, j: usize, hit_y: f32, at: [f32; 3]) {
        // a body that already dropped this tick takes no further hits —
        // otherwise a shotgun's later pellets score the same kill twice
        if !self.fighters[j].alive() {
            return;
        }
        let base = self.fighters[j].pos[1];
        let h = self.fighters[j].height();
        let frac = ((hit_y - base) / h).clamp(0.0, 1.0);
        let zone = if frac > 0.82 {
            HitZone::Head
        } else if frac > 0.66 {
            HitZone::Arms
        } else if frac > 0.35 {
            HitZone::Torso
        } else {
            HitZone::Legs
        };
        // v6 model: per-gun torso damage × zone multiplier. The baseline
        // M4A1 lands the owner's tuned rule: 2 headshots / 8 body shots.
        let mut dmg = gun(self.fighters[i].gun).damage * zone.mult();
        let from = {
            let f = &self.fighters[i];
            [f.pos[0], f.pos[1] + EYE_REL, f.pos[2]]
        };
        // the shield eats front-arc damage BEFORE armor
        let mut shielded = false;
        if let Some(block) = self.shield_block(j, from) {
            dmg *= 1.0 - block;
            shielded = true;
        }
        let armor = self.fighters[j].armor;
        if armor > 0.0 {
            let absorbed = dmg.min(armor);
            self.fighters[j].armor -= absorbed;
            dmg -= absorbed * 0.7; // the robot shell soaks most of it
        }
        self.fighters[j].health -= dmg;
        self.fighters[i].hits_dealt += 1;
        let fatal = self.fighters[j].health <= 0.0;
        self.hits.push((
            HitEvent {
                shooter: i,
                victim: j,
                zone,
                damage: dmg,
                shielded,
                from,
                at,
                fatal,
            },
            2.2,
        ));
        if fatal {
            self.fighters[j].deaths += 1;
            self.fighters[j].respawn_t = RESPAWN_S;
            self.fighters[j].vel = [0.0, 0.0];
            self.fighters[j].shield_up = false; // the plate drops with you
            self.fighters[i].kills += 1;
            if self.mode == Mode::Tdm {
                let s = Self::team_idx(self.fighters[i].team);
                self.score[s] += 1.0;
                if self.overtime || self.score[s] >= TDM_TARGET as f32 {
                    self.finish(self.fighters[i].team);
                }
            }
            self.kill_feed.push((
                KillEvent {
                    killer: i,
                    victim: j,
                    headshot: zone == HitZone::Head,
                },
                5.0,
            ));
        }
    }

    /// Predict where a projectile launched from `o` along `d` at `v0` will
    /// fly and land — the SAME integration as `step_missiles`, so the
    /// preview arc is exactly the flight the arrow/spear will take.
    /// Returns (sampled points along the arc, impact point, impact normal).
    pub fn predict_arc(
        &self,
        o: [f32; 3],
        d: [f32; 3],
        v0: f32,
        is_spear: bool,
        max_s: f32,
    ) -> (Vec<[f32; 3]>, [f32; 3], [f32; 3]) {
        let d = normalize(d);
        let mut pos = o;
        let mut vel = [d[0] * v0, d[1] * v0, d[2] * v0];
        let g = 9.81 * if is_spear { 1.0 } else { 0.55 };
        let mut pts = Vec::new();
        let steps = (max_s / DT) as usize;
        for step in 0..steps {
            vel[1] -= g * DT;
            let old = pos;
            pos[0] += vel[0] * DT;
            pos[1] += vel[1] * DT;
            pos[2] += vel[2] * DT;
            let seg = [pos[0] - old[0], pos[1] - old[1], pos[2] - old[2]];
            let seg_len = (seg[0] * seg[0] + seg[1] * seg[1] + seg[2] * seg[2])
                .sqrt()
                .max(1e-5);
            let dn = [seg[0] / seg_len, seg[1] / seg_len, seg[2] / seg_len];
            let mut best: Option<(f32, [f32; 3])> = None;
            for c in &self.cover {
                if let Some((t, n)) = c.ray_hit(old, dn, seg_len) {
                    if t <= seg_len && best.map_or(true, |(bt, _)| t < bt) {
                        best = Some((t, n));
                    }
                }
            }
            if let Some((t, n)) = best {
                let at = [old[0] + dn[0] * t, old[1] + dn[1] * t, old[2] + dn[2] * t];
                return (pts, at, n);
            }
            if pos[1] <= 0.0 {
                let t = if seg[1].abs() > 1e-6 {
                    (0.0 - old[1]) / seg[1]
                } else {
                    1.0
                };
                let at = [
                    old[0] + seg[0] * t.clamp(0.0, 1.0),
                    0.0,
                    old[2] + seg[2] * t.clamp(0.0, 1.0),
                ];
                return (pts, at, [0.0, 1.0, 0.0]);
            }
            if step % 5 == 0 {
                pts.push(pos);
            }
        }
        (pts, pos, [0.0, 1.0, 0.0])
    }

    fn step_missiles(&mut self) {
        let t_now = self.t;
        let mut hits: Vec<(usize, usize, f32, [f32; 3], [f32; 3])> = Vec::new();
        let snap: Vec<(Team, [f32; 3], f32, bool)> = self
            .fighters
            .iter()
            .map(|f| (f.team, f.pos, f.height(), f.alive() && f.protect_t <= 0.0))
            .collect();
        let cover = &self.cover;
        for m in &mut self.missiles {
            if m.stuck_t.is_some() {
                continue;
            }
            m.vel[1] -= 9.81 * DT * if m.is_spear { 1.0 } else { 0.55 };
            let old = m.pos;
            m.pos[0] += m.vel[0] * DT;
            m.pos[1] += m.vel[1] * DT;
            m.pos[2] += m.vel[2] * DT;
            // body check
            for (j, &(team, pos, h, alive)) in snap.iter().enumerate() {
                if team == m.team || !alive || j == m.shooter {
                    continue;
                }
                let dx = m.pos[0] - pos[0];
                let dz = m.pos[2] - pos[2];
                if dx * dx + dz * dz < 0.20 && m.pos[1] > pos[1] && m.pos[1] < pos[1] + h {
                    hits.push((m.shooter, j, m.damage, m.pos, m.vel));
                    m.stuck_t = Some(t_now);
                    break;
                }
            }
            if m.stuck_t.is_some() {
                continue;
            }
            // world check
            let seg = [
                m.pos[0] - old[0],
                m.pos[1] - old[1],
                m.pos[2] - old[2],
            ];
            let seg_len = (seg[0] * seg[0] + seg[1] * seg[1] + seg[2] * seg[2])
                .sqrt()
                .max(1e-5);
            let dn = [seg[0] / seg_len, seg[1] / seg_len, seg[2] / seg_len];
            for c in cover {
                if let Some((t, _)) = c.ray_hit(old, dn, seg_len) {
                    if t <= seg_len {
                        m.pos = [old[0] + dn[0] * t, old[1] + dn[1] * t, old[2] + dn[2] * t];
                        m.stuck_t = Some(t_now);
                        break;
                    }
                }
            }
            if m.pos[1] <= 0.0 {
                m.pos[1] = 0.0;
                m.stuck_t = Some(t_now);
            }
        }
        self.missiles
            .retain(|m| m.stuck_t.map_or(true, |s| t_now - s < 15.0));
        for (i, j, dmg, at, vel) in hits {
            // a corpse from an earlier missile this same tick stays down —
            // no double deaths, no double score
            if !self.fighters[j].alive() {
                continue;
            }
            // projectile damage: flat, no zone bonus (mass does the work)
            let mut d = dmg;
            // arrows and spears respect the shield too — the attack comes
            // from BACK ALONG the flight path, not from the impact point
            // (the impact point sits ON the victim and has no direction)
            let from_dir = [at[0] - vel[0], at[1] - vel[1], at[2] - vel[2]];
            let mut shielded = false;
            if let Some(block) = self.shield_block(j, from_dir) {
                d *= 1.0 - block;
                shielded = true;
            }
            let armor = self.fighters[j].armor;
            if armor > 0.0 {
                let absorbed = d.min(armor);
                self.fighters[j].armor -= absorbed;
                d -= absorbed * 0.7;
            }
            self.fighters[j].health -= d;
            self.fighters[i].hits_dealt += 1;
            let fatal = self.fighters[j].health <= 0.0;
            let from = {
                let f = &self.fighters[i];
                [f.pos[0], f.pos[1] + EYE_REL, f.pos[2]]
            };
            self.hits.push((
                HitEvent {
                    shooter: i,
                    victim: j,
                    zone: HitZone::Torso,
                    damage: d,
                    shielded,
                    from,
                    at,
                    fatal,
                },
                2.2,
            ));
            if fatal {
                self.fighters[j].deaths += 1;
                self.fighters[j].respawn_t = RESPAWN_S;
                self.fighters[j].vel = [0.0, 0.0];
                self.fighters[j].shield_up = false;
                self.fighters[i].kills += 1;
                if self.mode == Mode::Tdm {
                    let s = Self::team_idx(self.fighters[i].team);
                    self.score[s] += 1.0;
                    if self.overtime || self.score[s] >= TDM_TARGET as f32 {
                        self.finish(self.fighters[i].team);
                    }
                }
                self.kill_feed.push((
                    KillEvent {
                        killer: i,
                        victim: j,
                        headshot: false,
                    },
                    5.0,
                ));
            }
        }
    }

    fn los_clear(&self, from: [f32; 3], to: [f32; 3]) -> bool {
        let d = [to[0] - from[0], to[1] - from[1], to[2] - from[2]];
        let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
        if len < 1e-3 {
            return true;
        }
        let dn = [d[0] / len, d[1] / len, d[2] / len];
        for c in &self.cover {
            if let Some((t, _)) = c.ray_hit(from, dn, len) {
                if t < len - 0.1 {
                    return false;
                }
            }
        }
        true
    }

    fn nearest_visible_enemy(&self, i: usize) -> Option<usize> {
        let f = &self.fighters[i];
        let eye = [f.pos[0], f.pos[1] + EYE_REL, f.pos[2]];
        let mut best: Option<(usize, f32)> = None;
        for (j, g) in self.fighters.iter().enumerate() {
            if g.team == f.team || !g.alive() || g.protect_t > 0.0 {
                continue;
            }
            // sight the target's actual chest — a crouched enemy is LOWER
            let tgt = [g.pos[0], g.pos[1] + g.height() * 0.6, g.pos[2]];
            let d2 = (tgt[0] - eye[0]).powi(2) + (tgt[2] - eye[2]).powi(2);
            if best.map_or(true, |(_, b)| d2 < b) && self.los_clear(eye, tgt) {
                best = Some((j, d2));
            }
        }
        best.map(|(j, _)| j)
    }

    fn bot_think(&mut self, i: usize) {
        let half = self.half;
        let f = &self.fighters[i];
        let team = f.team;
        let (wx, wz) = (f.waypoint[0] - f.pos[0], f.waypoint[1] - f.pos[2]);
        if wx * wx + wz * wz < 4.0 || self.rng.next_f32() < 0.15 {
            let roll = self.rng.next_f32();
            // KOTH bots fight for the hill; everyone likes an unowned or
            // enemy checkpoint; otherwise roam toward the enemy side
            let target = if self.mode == Mode::Koth && roll < 0.55 {
                [
                    self.hill[0] + self.rng.range(-3.0, 3.0),
                    self.hill[2] + self.rng.range(-3.0, 3.0),
                ]
            } else if roll < 0.30 && !self.checkpoints.is_empty() {
                let want = self
                    .checkpoints
                    .iter()
                    .find(|c| c.owner != Some(team))
                    .or(self.checkpoints.first())
                    .unwrap();
                [
                    want.pos[0] + self.rng.range(-2.0, 2.0),
                    want.pos[2] + self.rng.range(-2.0, 2.0),
                ]
            } else {
                let toward = match team {
                    Team::Blue => 1.0,
                    Team::Red => -1.0,
                };
                [
                    self.rng.range(-half + 4.0, half - 4.0),
                    self.rng.range(-half + 4.0, half - 4.0) * 0.5
                        + toward * self.rng.range(0.0, half - 8.0),
                ]
            };
            // keep waypoints INSIDE the walls — the biased roam sum can
            // land past the border, pinning bots against it
            self.fighters[i].waypoint = [
                target[0].clamp(-half + 3.0, half - 3.0),
                target[1].clamp(-half + 3.0, half - 3.0),
            ];
        }
    }

    fn bot_act(&mut self, i: usize) {
        // difficulty shapes the whole brain: aim, reflexes, range, push
        let bp = bot_params(self.cfg.difficulty);
        // a fully dry active slot (no mag, no reserve) means SWITCH, not
        // sulk — grab the first slot that still has ammo
        {
            let f = &self.fighters[i];
            if f.ammo == 0 && f.reserve == 0 {
                if let Some(s) = (0..3).find(|&s| {
                    s != f.active && (f.slot_ammo[s].0 > 0 || f.slot_ammo[s].1 > 0)
                }) {
                    self.switch_slot(i, s);
                }
            }
        }
        let enemy = self.nearest_visible_enemy(i);
        let (fpos, strafe_phase, waypoint, ammo, reloading) = {
            let f = &self.fighters[i];
            (f.pos, f.strafe_phase, f.waypoint, f.ammo, f.reload_t > 0.0)
        };
        let yaw;
        let mut vel;
        match enemy {
            Some(j) => {
                self.fighters[i].los_time += DT;
                // an empty mag reloads NOW, whatever the range — waiting
                // until the enemy closes is how bots died mid-clack
                if ammo == 0 {
                    self.try_reload(i);
                }
                let gpos = self.fighters[j].pos;
                let ghigh = self.fighters[j].height();
                let (dx, dz) = (gpos[0] - fpos[0], gpos[2] - fpos[2]);
                let dist = (dx * dx + dz * dz).sqrt().max(0.01);
                yaw = dx.atan2(dz);
                let phase = strafe_phase + self.t * 1.7;
                let strafe = phase.sin().signum() * 0.8;
                let (px, pz) = (-dz / dist, dx / dist);
                let closing = if dist > 15.0 {
                    0.8 * bp.aggression
                } else if dist < 6.0 {
                    -0.6 / bp.aggression.max(0.5)
                } else {
                    0.0
                };
                self.fighters[i].crouch = closing == 0.0 && dist > 9.0;
                // shield discipline: caught reloading in the open → turtle
                // behind the shield until the mag is back in
                self.fighters[i].shield_up = reloading && dist < 16.0;
                vel = [
                    (px * strafe + dx / dist * closing) * MOVE_SPEED * 0.8,
                    (pz * strafe + dz / dist * closing) * MOVE_SPEED * 0.8,
                ];
                if self.fighters[i].los_time > bp.reaction_s
                    && dist < bp.engage_range
                    && ammo > 0
                {
                    // fire from the REAL muzzle (crouch lowers it) at the
                    // target's REAL chest (crouched enemies are short) —
                    // the old fixed heights sailed over crouchers and
                    // discounted everything into the arms band
                    let eye = self.muzzle_origin(i);
                    let tgt = [gpos[0], gpos[1] + ghigh * 0.55, gpos[2]];
                    let aim = [tgt[0] - eye[0], tgt[1] - eye[1], tgt[2] - eye[2]];
                    let (e1, e2) = (
                        self.rng.range(-bp.aim_sigma, bp.aim_sigma),
                        self.rng.range(-bp.aim_sigma, bp.aim_sigma),
                    );
                    let aim = perturb(normalize(aim), e1, e2);
                    self.try_fire(i, aim, false);
                }
            }
            None => {
                self.fighters[i].los_time = 0.0;
                self.fighters[i].crouch = false;
                self.fighters[i].shield_up = false;
                if self.fighters[i].armed()
                    && ammo < gun(self.fighters[i].gun).mag / 3
                {
                    self.try_reload(i);
                }
                let (dx, dz) = (waypoint[0] - fpos[0], waypoint[1] - fpos[2]);
                let d = (dx * dx + dz * dz).sqrt().max(0.01);
                vel = [dx / d * MOVE_SPEED * 0.85, dz / d * MOVE_SPEED * 0.85];
                yaw = dx.atan2(dz);
                // probe at 0.75 m so waist-high garden ruins register too
                let eye = [fpos[0], fpos[1] + 0.75, fpos[2]];
                let ahead = [
                    fpos[0] + dx / d * 1.2,
                    fpos[1] + 0.75,
                    fpos[2] + dz / d * 1.2,
                ];
                if !self.los_clear(eye, ahead) {
                    vel = [-dz / d * MOVE_SPEED * 0.7, dx / d * MOVE_SPEED * 0.7];
                }
            }
        }
        let fm = &mut self.fighters[i];
        if fm.shield_up {
            vel = [vel[0] * SHIELD_SPEED_MULT, vel[1] * SHIELD_SPEED_MULT];
        }
        if fm.crouch {
            // bots pay the same crouch tax the player does
            vel = [vel[0] * CROUCH_SPEED_MULT, vel[1] * CROUCH_SPEED_MULT];
        }
        fm.vel = vel;
        fm.yaw = yaw;
    }
}

fn spawn_point(team: Team, slot: usize, half: f32) -> ([f32; 3], f32) {
    let z = match team {
        Team::Blue => -half + 2.5,
        Team::Red => half - 2.5,
    };
    let yaw = match team {
        Team::Blue => 0.0,
        Team::Red => std::f32::consts::PI,
    };
    // room for up to 8 per team (v6 cap)
    let x = -10.5 + (slot as f32) * 3.0;
    ([x, 0.0, z], yaw)
}

fn normalize(v: [f32; 3]) -> [f32; 3] {
    let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt().max(1e-6);
    [v[0] / l, v[1] / l, v[2] / l]
}

fn perturb(d: [f32; 3], ex: f32, ey: f32) -> [f32; 3] {
    let up = if d[1].abs() < 0.9 {
        [0.0, 1.0, 0.0]
    } else {
        [1.0, 0.0, 0.0]
    };
    let rx = [
        d[1] * up[2] - d[2] * up[1],
        d[2] * up[0] - d[0] * up[2],
        d[0] * up[1] - d[1] * up[0],
    ];
    let rx = normalize(rx);
    let ry = [
        d[1] * rx[2] - d[2] * rx[1],
        d[2] * rx[0] - d[0] * rx[2],
        d[0] * rx[1] - d[1] * rx[0],
    ];
    normalize([
        d[0] + rx[0] * ex + ry[0] * ey,
        d[1] + rx[1] * ex + ry[1] * ey,
        d[2] + rx[2] * ex + ry[2] * ey,
    ])
}

fn ray_vs_cylinder(
    o: [f32; 3],
    d: [f32; 3],
    base: [f32; 3],
    radius: f32,
    height: f32,
) -> Option<(f32, f32)> {
    let (ox, oz) = (o[0] - base[0], o[2] - base[2]);
    let (dx, dz) = (d[0], d[2]);
    let a = dx * dx + dz * dz;
    if a < 1e-8 {
        return None;
    }
    let b = 2.0 * (ox * dx + oz * dz);
    let c = ox * ox + oz * oz - radius * radius;
    // muzzle already inside the body: point-blank contact, hit at t = 0
    // (the entry-face root is negative there and used to whiff entirely)
    if c < 0.0 {
        if o[1] >= base[1] && o[1] <= base[1] + height {
            return Some((0.0, o[1]));
        }
        return None;
    }
    let disc = b * b - 4.0 * a * c;
    if disc < 0.0 {
        return None;
    }
    let t = (-b - disc.sqrt()) / (2.0 * a);
    if t < 0.0 {
        return None;
    }
    let y = o[1] + d[1] * t;
    if y < base[1] || y > base[1] + height {
        return None;
    }
    Some((t, y))
}

#[cfg(test)]
mod tests {
    use super::*;
    use jk_core::timestep::SIM_HZ;

    fn run(sim: &mut TdmSim, secs: usize, cmd: PlayerCmd) {
        for _ in 0..(secs * SIM_HZ as usize) {
            sim.step(cmd);
        }
    }

    fn cfg(seed: u64, per_team: usize, mode: Mode, map: MapKind) -> MatchConfig {
        MatchConfig {
            seed,
            per_team,
            mode,
            map,
            ..MatchConfig::default()
        }
    }

    /// A clean shooting range: 1v1, no cover, both pinned and unprotected.
    fn range(seed: u64) -> TdmSim {
        let mut s = TdmSim::new(cfg(seed, 1, Mode::Tdm, MapKind::Arena));
        s.cover.clear();
        s.cover_kind.clear();
        s.pickups.clear();
        s.checkpoints.clear();
        s.fighters[0].pos = [0.0, 0.0, -5.0];
        s.fighters[1].pos = [0.0, 0.0, 5.0];
        s.fighters[1].protect_t = 0.0;
        s
    }

    /// Fire `shots` M4-style aimed rounds at a held target; returns hits
    /// dealt and the victim's remaining health.
    fn shoot_at(s: &mut TdmSim, aim_y: f32, secs: usize) {
        // aim from the eye (1.62) to the requested height at 10 m
        let cmd = PlayerCmd {
            aim: [0.0, (aim_y - EYE_REL) / 10.0, 1.0],
            shoot: true,
            ..Default::default()
        };
        for _ in 0..(secs * SIM_HZ as usize) {
            s.step(cmd);
            // pin the bot: bot AI resets pos/vel/stance INSIDE step —
            // hold it STANDING so the fixed aim heights land where the
            // test expects
            s.fighters[1].pos = [0.0, 0.0, 5.0];
            s.fighters[1].vel = [0.0, 0.0];
            s.fighters[1].protect_t = 0.0;
            s.fighters[1].shield_up = false;
            s.fighters[1].crouch = false;
            if s.fighters[0].kills >= 1 {
                break;
            }
        }
    }

    #[test]
    fn ttk_rule_two_head_eight_torso_for_baseline_rifle() {
        // the owner's tuned target IS the baseline M4A1
        let m4 = gun(GunKind::M4);
        assert_eq!(
            (MAX_HEALTH / (m4.damage * HEAD_MULT)).ceil() as u32,
            2,
            "2 headshots"
        );
        assert_eq!((MAX_HEALTH / m4.damage).ceil() as u32, 8, "8 body shots");
        // torso: must take 6+ hits (spread can graze arms; never < 6)
        let mut s = range(3);
        assert_eq!(s.fighters[0].gun, GunKind::M4, "default loadout leads M4");
        shoot_at(&mut s, 1.0, 8);
        assert_eq!(s.fighters[0].kills, 1, "torso fire must kill");
        assert!(
            s.fighters[0].hits_dealt >= 6,
            "torso kill takes ~8: took {}",
            s.fighters[0].hits_dealt
        );
        // head: exactly 2
        let mut s = range(4);
        shoot_at(&mut s, 1.66, 8);
        assert_eq!(s.fighters[0].kills, 1, "head fire must kill");
        assert!(
            s.fighters[0].hits_dealt <= 3,
            "head kill takes ~2: took {}",
            s.fighters[0].hits_dealt
        );
    }

    #[test]
    fn awm_only_the_head_is_instant() {
        let awm = gun(GunKind::Awm);
        // the owner's table: head instant; torso, arms, legs = 2 shots
        assert!(awm.damage * HEAD_MULT >= MAX_HEALTH, "head = oblivion");
        assert!(awm.damage < MAX_HEALTH, "torso must NOT one-shot");
        assert_eq!((MAX_HEALTH / awm.damage).ceil() as u32, 2, "torso 2");
        assert_eq!(
            (MAX_HEALTH / (awm.damage * ARM_MULT)).ceil() as u32,
            2,
            "arms 2"
        );
        assert_eq!(
            (MAX_HEALTH / (awm.damage * LEG_MULT)).ceil() as u32,
            2,
            "legs 2"
        );
    }

    #[test]
    fn loadout_slots_switch_and_keep_ammo() {
        let mut s = TdmSim::new(cfg(5, 1, Mode::Tdm, MapKind::Arena));
        assert_eq!(s.fighters[0].inventory, DEFAULT_LOADOUT);
        assert_eq!(s.fighters[0].gun, GunKind::M4);
        // burn 3 rounds, switch to the pistol, switch back: mag remembered
        s.fighters[0].ammo -= 3;
        let left = s.fighters[0].ammo;
        s.step(PlayerCmd {
            slot: Some(1),
            ..Default::default()
        });
        // switching takes SWITCH_S; the gun itself flips immediately
        assert_eq!(s.fighters[0].gun, GunKind::Glock);
        assert_eq!(s.fighters[0].ammo, gun(GunKind::Glock).mag);
        s.fighters[0].switch_t = 0.0;
        s.step(PlayerCmd {
            slot: Some(0),
            ..Default::default()
        });
        assert_eq!(s.fighters[0].gun, GunKind::M4);
        assert_eq!(s.fighters[0].ammo, left, "slot keeps its magazine");
    }

    #[test]
    fn shield_blocks_front_ignores_rear() {
        let mut s = range(6);
        // victim: shield up, crouched, FACING the shooter (yaw π → −z).
        // Crouched height is 1.05, so torso shots land around y ≈ 0.55.
        s.fighters[1].shield_up = true;
        s.fighters[1].crouch = true;
        s.fighters[1].yaw = std::f32::consts::PI;
        let h0 = s.fighters[1].health;
        s.apply_hit(0, 1, 0.55, [0.0, 0.55, 5.0]);
        let front_dmg = h0 - s.fighters[1].health;
        assert!(
            front_dmg < gun(GunKind::M4).damage * 0.1,
            "crouched shield must be near-total from the front: took {front_dmg}"
        );
        // same shot with the victim facing AWAY: the shield does nothing
        s.fighters[1].yaw = 0.0;
        let h1 = s.fighters[1].health;
        s.apply_hit(0, 1, 0.55, [0.0, 0.55, 5.0]);
        let rear_dmg = h1 - s.fighters[1].health;
        assert!(
            (rear_dmg - gun(GunKind::M4).damage).abs() < 0.01,
            "rear shots ignore the shield entirely: took {rear_dmg}"
        );
        // standing (not crouched) front block is real but not near-total
        // (standing height 1.78 → torso is around y ≈ 1.0)
        s.fighters[1].yaw = std::f32::consts::PI;
        s.fighters[1].crouch = false;
        let h2 = s.fighters[1].health;
        s.apply_hit(0, 1, 1.0, [0.0, 1.0, 5.0]);
        let stand_dmg = h2 - s.fighters[1].health;
        assert!(
            stand_dmg > front_dmg && stand_dmg < rear_dmg,
            "standing block sits between: {front_dmg} < {stand_dmg} < {rear_dmg}"
        );
        // shield up = no shooting
        s.fighters[0].shield_up = true;
        let fired = s.try_fire(0, [0.0, 0.0, 1.0], false);
        assert!(!fired, "the shield takes both hands");
    }

    #[test]
    fn reload_keeps_chambered_rounds() {
        let mut s = TdmSim::new(cfg(8, 1, Mode::Tdm, MapKind::Arena));
        // 5 in the mag, 10 in reserve → a reload must yield 15 total,
        // not throw the 5 away
        s.fighters[0].ammo = 5;
        s.fighters[0].reserve = 10;
        s.try_reload(0);
        for _ in 0..(3 * SIM_HZ as usize) {
            s.step(PlayerCmd::default());
            s.fighters[1].pos = [30.0, 0.0, 30.0];
        }
        let f = &s.fighters[0];
        assert_eq!(f.ammo + f.reserve, 15, "reload must not destroy ammo");
        assert_eq!(f.ammo, 15.min(gun(f.gun).mag), "mag tops up first");
    }

    #[test]
    fn one_body_scores_one_kill() {
        // two lethal spears landing the same tick must count ONE death
        let mut s = range(9);
        s.fighters[1].health = 10.0;
        // spawned one integration step short of the body so BOTH arrive
        // on the same tick
        for id in [901, 902] {
            s.missiles.push(Missile {
                id,
                pos: [0.0, 1.0, 4.9],
                vel: [0.0, 0.0, 30.0],
                team: Team::Blue,
                shooter: 0,
                damage: 55.0,
                is_spear: true,
                stuck_t: None,
            });
        }
        s.step_missiles();
        assert_eq!(s.fighters[1].deaths, 1, "one body, one death");
        assert_eq!(s.fighters[0].kills, 1, "one body, one kill");
        assert!((s.score[0] - 1.0).abs() < 0.01, "one body, one point");
    }

    #[test]
    fn rolling_shield_blocks_nothing() {
        let mut s = range(10);
        s.fighters[1].shield_up = true;
        s.fighters[1].crouch = true;
        s.fighters[1].yaw = std::f32::consts::PI; // facing the shooter
        s.fighters[1].roll_t = ROLL_S * 0.5; // mid-somersault
        let h0 = s.fighters[1].health;
        s.apply_hit(0, 1, 0.5, [0.0, 0.5, 5.0]);
        let dmg = h0 - s.fighters[1].health;
        assert!(
            dmg > gun(GunKind::M4).damage * 0.5,
            "a tumbling shield must not block: took only {dmg}"
        );
    }

    #[test]
    fn checkpoints_capture_and_pull_respawns() {
        let mut s = TdmSim::new(cfg(7, 1, Mode::Tdm, MapKind::Bailey));
        let cp_pos = s.checkpoints[0].pos;
        // stand in the ring uncontested → the ring flips Blue
        for _ in 0..((CHECKPOINT_CAP_S as usize + 2) * SIM_HZ as usize) {
            s.step(PlayerCmd::default());
            s.fighters[0].pos = cp_pos;
            s.fighters[1].pos = [-30.0, 0.0, -30.0]; // keep the enemy away
        }
        assert_eq!(s.checkpoints[0].owner, Some(Team::Blue), "ring must flip");
        // die → respawn at the owned checkpoint, not at base
        s.fighters[0].health = 0.0;
        s.fighters[0].respawn_t = RESPAWN_S;
        for _ in 0..((RESPAWN_S as usize + 2) * SIM_HZ as usize) {
            s.step(PlayerCmd::default());
            s.fighters[1].pos = [-30.0, 0.0, -30.0];
        }
        let p = s.fighters[0].pos;
        let d = ((p[0] - cp_pos[0]).powi(2) + (p[2] - cp_pos[2]).powi(2)).sqrt();
        assert!(d < 4.0, "respawn must be AT the checkpoint: {d} m away");
    }

    #[test]
    fn difficulty_scales_the_bots() {
        let e = bot_params(Difficulty::Easy);
        let n = bot_params(Difficulty::Normal);
        let h = bot_params(Difficulty::Hard);
        assert!(e.aim_sigma > n.aim_sigma && n.aim_sigma > h.aim_sigma);
        assert!(e.reaction_s > n.reaction_s && n.reaction_s > h.reaction_s);
        assert!(e.engage_range < n.engage_range && n.engage_range < h.engage_range);
        // and it shows on the field: the same battle bleeds the player's
        // team harder on Hard than on Easy
        let deaths = |d: Difficulty| {
            let mut s = TdmSim::new(MatchConfig {
                seed: 42,
                per_team: 4,
                difficulty: d,
                ..MatchConfig::default()
            });
            for _ in 0..(30 * SIM_HZ as usize) {
                s.step(PlayerCmd::default());
            }
            s.fighters
                .iter()
                .filter(|f| f.team == Team::Blue)
                .map(|f| f.deaths)
                .sum::<u32>()
        };
        assert!(
            deaths(Difficulty::Hard) >= deaths(Difficulty::Easy),
            "hard bots must bite harder"
        );
    }

    #[test]
    fn elevation_supports_and_gravity() {
        let mut s = TdmSim::new(cfg(4, 5, Mode::Tdm, MapKind::Arena));
        // put the player on the tower top
        s.fighters[0].pos = [0.0, 3.0, 0.0];
        run(&mut s, 1, PlayerCmd::default());
        assert!(
            (s.fighters[0].pos[1] - 3.0).abs() < 0.05,
            "tower must support: y {}",
            s.fighters[0].pos[1]
        );
        // walk off the edge → fall to ground
        s.fighters[0].pos = [8.0, 3.0, 8.0]; // off-tower, in the air
        run(&mut s, 2, PlayerCmd::default());
        assert!(
            s.fighters[0].pos[1] < 0.1,
            "gravity must land him: y {}",
            s.fighters[0].pos[1]
        );
    }

    #[test]
    fn bow_arrows_fly_and_hit() {
        let mut s = TdmSim::new(cfg(15, 1, Mode::Tdm, MapKind::Arena));
        s.cover.clear();
        s.cover_kind.clear();
        s.pickups.clear();
        s.checkpoints.clear();
        s.fighters[0].gun = GunKind::Bow;
        s.fighters[0].ammo = 1;
        s.fighters[0].reserve = 10;
        s.fighters[0].pos = [0.0, 0.0, -8.0];
        s.fighters[1].pos = [0.0, 0.0, 8.0];
        // flat release: from a 1.62 m eye, drop over 16 m at 38 m/s is
        // ~0.5 m — the arrow arrives at chest height. (A 0.06 rad loft
        // overcorrects and sails over the head.)
        let cmd = PlayerCmd {
            aim: [0.0, 0.0, 1.0],
            shoot: true,
            ..Default::default()
        };
        // disarm the target so it can't kill the archer mid-test
        s.fighters[1].ammo = 0;
        s.fighters[1].reserve = 0;
        let mut hit = false;
        for _ in 0..(8 * SIM_HZ as usize) {
            s.step(cmd);
            // pin the target: bot AI re-sets vel INSIDE step, so resetting
            // vel after the step is not enough — hold the position itself
            s.fighters[1].pos = [0.0, 0.0, 8.0];
            s.fighters[1].protect_t = 0.0;
            if s.fighters[0].hits_dealt > 0 {
                hit = true;
                break;
            }
        }
        assert!(hit, "arrows must connect at 16 m");
    }

    #[test]
    fn player_jumps_and_lands() {
        let mut s = TdmSim::new(cfg(7, 5, Mode::Tdm, MapKind::Arena));
        let jump = PlayerCmd {
            jump: true,
            ..Default::default()
        };
        s.step(jump);
        let mut peak = 0.0_f32;
        for _ in 0..(2 * SIM_HZ as usize) {
            s.step(PlayerCmd::default());
            peak = peak.max(s.fighters[0].pos[1]);
        }
        assert!(peak > 0.8, "jump must rise: peak {peak}");
        assert!(
            s.fighters[0].pos[1] < 0.05 && s.fighters[0].grounded,
            "must land again: y {}",
            s.fighters[0].pos[1]
        );
        // holding jump mid-air must not double-jump
        s.step(jump);
        let mut s2 = TdmSim::new(cfg(7, 5, Mode::Tdm, MapKind::Arena));
        s2.step(jump);
        s2.step(jump); // second press mid-air
        assert!(s2.fighters[0].vy <= JUMP_SPEED, "no double jump boost");
    }

    #[test]
    fn koth_scores_the_hill() {
        let mut s = TdmSim::new(cfg(6, 1, Mode::Koth, MapKind::Arena));
        // blue alone on the hill
        s.fighters[0].pos = [s.hill[0], s.hill[1], s.hill[2]];
        s.fighters[1].pos = [20.0, 0.0, 20.0];
        run(&mut s, 5, PlayerCmd::default());
        assert!(
            s.score[0] > 2.0,
            "holding the hill must score: {:?}",
            s.score
        );
    }

    #[test]
    fn dodge_roll_dashes_low_and_cools_down() {
        let mut s = TdmSim::new(cfg(11, 5, Mode::Tdm, MapKind::Arena));
        let dodge = PlayerCmd {
            move_z: 1.0,
            dodge: true,
            ..Default::default()
        };
        s.step(dodge);
        assert!(s.fighters[0].roll_t > 0.0, "dodge must start a roll");
        // mid-roll: faster than sprint, balled up small, gun locked out
        let sp = {
            let f = &s.fighters[0];
            (f.vel[0] * f.vel[0] + f.vel[1] * f.vel[1]).sqrt()
        };
        assert!(sp > SPRINT_SPEED, "roll must dash faster than sprint: {sp}");
        assert!(s.fighters[0].height() < CROUCH_HEIGHT, "roll must be low");
        // roll ends, cooldown blocks an immediate second roll
        for _ in 0..((ROLL_S / DT) as usize + 2) {
            s.step(PlayerCmd::default());
        }
        assert!(s.fighters[0].roll_t <= 0.0, "roll must end");
        s.step(dodge);
        assert!(
            s.fighters[0].roll_t <= 0.0,
            "cooldown must block a chained roll"
        );
    }

    #[test]
    fn hard_landing_breakfalls_into_a_roll() {
        let mut s = TdmSim::new(cfg(12, 5, Mode::Tdm, MapKind::Arena));
        // hurl the player off a high ledge, running forward
        s.fighters[0].pos = [8.0, 6.0, 8.0];
        s.fighters[0].grounded = false;
        let run = PlayerCmd {
            move_z: 1.0,
            ..Default::default()
        };
        let mut rolled = false;
        for _ in 0..(3 * SIM_HZ as usize) {
            s.step(run);
            if s.fighters[0].roll_t > 0.0 {
                rolled = true;
                break;
            }
        }
        assert!(rolled, "a 6 m drop must land in a breakfall roll");
        // an ordinary flat jump must NOT roll
        let mut s2 = TdmSim::new(cfg(12, 5, Mode::Tdm, MapKind::Arena));
        s2.step(PlayerCmd {
            jump: true,
            ..Default::default()
        });
        for _ in 0..(2 * SIM_HZ as usize) {
            s2.step(PlayerCmd::default());
            assert!(
                s2.fighters[0].roll_t <= 0.0,
                "a flat jump must land on its feet"
            );
        }
    }

    #[test]
    fn arc_prediction_matches_real_flight() {
        let mut s = TdmSim::new(cfg(13, 1, Mode::Tdm, MapKind::Arena));
        let o = [0.0, EYE_REL, -20.0];
        let d = normalize([0.05, 0.25, 1.0]);
        let v0 = 17.0;
        let (pts, predicted, _n) = s.predict_arc(o, d, v0, true, 6.0);
        assert!(!pts.is_empty(), "arc must have preview points");
        // fly a REAL spear on the same launch and compare landings
        s.missiles.push(Missile {
            id: 900,
            pos: o,
            vel: [d[0] * v0, d[1] * v0, d[2] * v0],
            team: Team::Blue,
            shooter: 0,
            damage: 0.0,
            is_spear: true,
            stuck_t: None,
        });
        let mut landed = None;
        for _ in 0..(6 * SIM_HZ as usize) {
            s.step_missiles();
            if let Some(m) = s.missiles.iter().find(|m| m.id == 900) {
                if m.stuck_t.is_some() {
                    landed = Some(m.pos);
                    break;
                }
            } else {
                break;
            }
        }
        let landed = landed.expect("spear must land");
        let d2 = (landed[0] - predicted[0]).powi(2)
            + (landed[1] - predicted[1]).powi(2)
            + (landed[2] - predicted[2]).powi(2);
        assert!(
            d2.sqrt() < 0.6,
            "prediction must match flight: predicted {predicted:?}, landed {landed:?}"
        );
    }

    #[test]
    fn all_maps_are_valid_battlefields() {
        for map in MapKind::ALL {
            // v6 cap: validate at the full 8v8 too
            let s = TdmSim::new(cfg(31, 8, Mode::Koth, map));
            assert_eq!(s.cover.len(), s.cover_kind.len(), "{map:?}: kind per cover");
            assert_eq!(s.checkpoints.len(), 2, "{map:?}: two forward rings");
            for cp in &s.checkpoints {
                for c in &s.cover {
                    let inside = cp.pos[0] > c.min[0]
                        && cp.pos[0] < c.max[0]
                        && cp.pos[2] > c.min[2]
                        && cp.pos[2] < c.max[2]
                        && cp.pos[1] + 0.3 > c.min[1]
                        && cp.pos[1] + 0.3 < c.max[1];
                    assert!(!inside, "{map:?}: checkpoint {:?} buried in {c:?}", cp.pos);
                }
            }
            for c in &s.cover {
                for i in 0..3 {
                    assert!(c.min[i] < c.max[i], "{map:?}: degenerate box {c:?}");
                }
            }
            // spawn rows and pickups must not be buried inside cover
            for f in &s.fighters {
                for c in &s.cover {
                    let inside = f.pos[0] > c.min[0]
                        && f.pos[0] < c.max[0]
                        && f.pos[2] > c.min[2]
                        && f.pos[2] < c.max[2]
                        && f.pos[1] + 0.5 > c.min[1]
                        && f.pos[1] + 0.5 < c.max[1];
                    assert!(!inside, "{map:?}: spawn {:?} buried in {c:?}", f.pos);
                }
            }
            for p in &s.pickups {
                for c in &s.cover {
                    let inside = p.pos[0] > c.min[0]
                        && p.pos[0] < c.max[0]
                        && p.pos[2] > c.min[2]
                        && p.pos[2] < c.max[2]
                        && p.pos[1] + 0.3 > c.min[1]
                        && p.pos[1] + 0.3 < c.max[1];
                    assert!(!inside, "{map:?}: pickup {:?} buried in {c:?}", p.pos);
                }
            }
            // the hill must sit on the center top
            assert!(s.hill[1] > 1.0, "{map:?}: hill must be elevated");
            // same seed, same map → identical battle
            let run_map = |m| {
                let mut s = TdmSim::new(cfg(31, 5, Mode::Tdm, m));
                for _ in 0..(10 * SIM_HZ as usize) {
                    s.step(PlayerCmd::default());
                }
                s.fighters.iter().map(|f| f.deaths).collect::<Vec<_>>()
            };
            assert_eq!(run_map(map), run_map(map), "{map:?} must be deterministic");
        }
    }

    #[test]
    fn deterministic_battle() {
        let outcome = || {
            let mut s = TdmSim::new(cfg(21, 5, Mode::Tdm, MapKind::Arena));
            run(&mut s, 40, PlayerCmd::default());
            (
                s.score[0] as u32,
                s.score[1] as u32,
                s.fighters.iter().map(|f| f.deaths).collect::<Vec<_>>(),
            )
        };
        assert_eq!(outcome(), outcome(), "same seed must replay identically");
    }
}
