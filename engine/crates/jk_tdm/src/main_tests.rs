//! THE TESTS FOR `main.rs`, WHICH IS WHY THIS FILE IS ONLY TESTS.
//!
//! Every module here was lifted verbatim out of the bottom of `main.rs`
//! (5,188 lines of it) on 2026-08-12. Nothing else changed: the only
//! edit the move required was `use super::*` -> `use crate::*`, because
//! these modules used to be children of the crate root and are now
//! grandchildren. The code under test did NOT move - it is still in
//! `main.rs`, and a crate-root item stays visible to a descendant module
//! whether it is `pub` or not, so no visibility had to be widened.
//!
//! Why it is worth a file of its own: `main.rs` is the most contended
//! file in the repo, and 16% of it was test bulk that no other lane ever
//! needs to read or merge.
//!
//! What this does NOT do, so that nobody re-derives it the hard way: it
//! does not reduce rustc's memory. One crate is one rustc process with
//! one arena that lives until the process exits, so moving code between
//! FILES of the same crate changes nothing the compiler measures. Only
//! moving it to a different CRATE would. See `engine/.cargo/config.toml`
//! for the measurements.

#[cfg(test)]
mod band_tests {
    use crate::*;

    /// §1.4 (Brief VI) - the no-bounce gates, measured on the render
    /// path's OWN `carry_offset` (not a copy of its math).
    #[test]
    fn vm_never_bounces() {
        // standing still: ZERO positional motion at every bob phase
        for th in 0..100 {
            let o = carry_offset(0.0, th as f32 * 0.37, true, 0.0, 0.0, 0.0, 1.0);
            assert_eq!(o, Vec3::ZERO, "standing = frozen bob: {o:?}");
        }
        // ...and anywhere below the dead-zone
        let o = carry_offset(VM_BOB_DEADZONE * 0.9, 1.3, true, 0.0, 0.0, 0.0, 1.0);
        assert_eq!(o, Vec3::ZERO, "sub-deadzone speed must not bob");
        // bounce meter: the whole fire-kick envelope at a standstill -
        // no lateral or vertical translation, rear slide ≤ 2 cm
        for ph in 0..=20 {
            let kick = ph as f32 / 20.0;
            let o = carry_offset(0.0, 0.0, true, kick, 0.0, 0.0, 1.0);
            assert!(
                o.x == 0.0 && o.y == 0.0,
                "firing adds ZERO lateral/vertical translation: {o:?}"
            );
            assert!(
                (0.0..=0.02).contains(&o.z),
                "back-slide stays ≤ 2 cm, rearward only: {}",
                o.z
            );
        }
        // after the envelope: exactly rest (≤ 2 mm demanded, 0 delivered)
        let rest = carry_offset(0.0, 0.0, true, 0.0, 0.0, 0.0, 1.0);
        assert!(rest.length() <= 0.002, "rest within 2 mm after the spray");
        // the kick return window is the ≤ 120 ms contract
        assert!(VM_KICK_RETURN_S <= 0.12 + 1e-6);
        // run-lower: full sprint pulls the weapon DOWN, never up
        for th in 0..50 {
            let o = carry_offset(1.0, th as f32 * 0.41, true, 0.0, 1.0, 0.0, 1.0);
            assert!(o.y < 0.0, "sprint must lower the weapon: {o:?}");
        }
        // airborne: bob is exactly ÷ 5
        let g = carry_offset(0.8, 1.1, true, 0.0, 0.0, 0.0, 1.0);
        let a = carry_offset(0.8, 1.1, false, 0.0, 0.0, 0.0, 1.0);
        assert!(
            (a.x / g.x - VM_AIR_BOB).abs() < 1e-5
                && (a.y / g.y - VM_AIR_BOB).abs() < 1e-5,
            "airborne bob must be exactly the CS:GO ÷5"
        );
    }

    /// §1.4a screen-intrusion, Brief VII §3.3/§4.3: EVERY weapon, at ITS
    /// OWN carry, with ITS OWN measured geometry, under ITS OWN profile,
    /// across every pose it can be held in indefinitely.
    ///
    /// This replaces a sweep that could see almost none of that. The old
    /// one ran a single hard-coded root, `(0.11, -0.13, 0.32)` - the
    /// GENERIC carry - against a pair of DECLARED envelope boxes. So the
    /// pistol's, the M249's, the spear's and the bow's own placements
    /// were never checked; every pose shift was unbounded (full draw
    /// pulls the bow 7.5 cm toward the midline and nothing looked); and
    /// the envelope it did check was a transcription that had drifted
    /// 2.3x off the real geometry. It passed the whole time.
    ///
    /// Nothing it covered is lost: stance, bob phase, fire kick, sprint
    /// and air are all still swept, now per weapon.
    ///
    /// SUSTAINED poses only, and that is a real distinction rather than a
    /// convenience. A drawn bow, a cooked grenade, a sprint and a
    /// suppression shake are states you can sit in for as long as you
    /// like, so an intrusion there is permanent. A reload, an inspect and
    /// a melee swing are committed actions that run to completion on
    /// their own clocks - a swing that crosses the whole frame is the
    /// READ the defender is being given, and clamping it would be
    /// removing the tell. `transient_poses_cannot_be_held_open` keeps
    /// that from becoming a loophole.
    #[test]
    fn every_weapon_holds_its_own_screen_profile() {
        let r_at = |z: f32| 0.24 * z * (VM_FOV_DEG.to_radians() * 0.5).tan();
        for kind in ALL_WEAPONS {
            let prof = screen_profile(kind);
            // MEASURED off the model, not transcribed from it. The
            // aggregate answers the MIDLINE question (how far left does
            // anything reach); the per-part corners answer the CIRCLE
            // one, because that needs a point the weapon really occupies.
            let (bl, bu) = weapon_bounded_extent(kind);
            let corners = weapon_bounded_corners(kind);
            let carry = vm_carry(kind).pos;
            // the root, in the same frame the sweep above uses: vm_carry
            // stores forward as NEGATIVE z, the camera frame as positive.
            //
            // ABS on z, because a carry may now be BEHIND the eye (the
            // couched javelin is): what the reticle-circle radius needs
            // is the DISTANCE the weapon is drawn at, and a signed
            // negative would hand `r_at` a negative radius that every
            // corner trivially clears.
            let base = Vec3::new(carry.x, carry.y, carry.z.abs());
            for sf in [0.0_f32, 0.3, 0.6, 1.0] {
                for th in 0..80 {
                    for grounded in [true, false] {
                        for (kick, sp) in
                            [(0.0_f32, 0.0_f32), (1.0, 0.0), (0.0, 1.0), (1.0, 1.0)]
                        {
                            for pull in [0.0_f32, 1.0] {
                                for cook in [0.0_f32, 1.0] {
                                    // worst case: every sustained shift at
                                    // its most intrusive, all at once.
                                    // They cannot all peak together in
                                    // play, which is the point - the
                                    // bound has to hold even then.
                                    //
                                    // The draw shift is the BOW's -
                                    // applying it to a pistol is how the
                                    // first run of this test "found" a
                                    // midline crossing on the Glock that
                                    // no player could ever see.
                                    // the draw shift is the BOW's -
                                    // applying it to a pistol is how the
                                    // first run of this test "found" a
                                    // midline crossing on the Glock that
                                    // no player could ever see - and it
                                    // yields to the coil exactly as the
                                    // render path does
                                    let draw = if kind == GunKind::Bow {
                                        pull * (1.0 - cook)
                                    } else {
                                        0.0
                                    };
                                    let pose = base
                                        + carry_offset(
                                            sf,
                                            th as f32 * 0.173,
                                            grounded,
                                            kick,
                                            sp,
                                            0.04,
                                            vm_steady(kind),
                                        )
                                        + VM_BOW_DRAW_SHIFT * draw
                                        + VM_GRENADE_SHIFT * cook
                                        - VM_SUPPRESS_SHAKE;
                                    let sway =
                                        pose.z.abs() * VM_SWAY_CAP_DEG.to_radians().tan();
                                    let r = r_at(pose.z.abs());
                                    match prof {
                                        // §owner The bow is held CENTRED and
                                        // is symmetric about its own riser,
                                        // so the MIDLINE question does not
                                        // apply to it - a shape that reaches
                                        // equally both ways cannot answer a
                                        // test written for a gun carried on
                                        // the right.
                                        //
                                        // What stood in for it was a vertical
                                        // proxy: "the whole bow must sit
                                        // BELOW the crosshair circle". That
                                        // was free while the bow lay
                                        // horizontal, and it forbids the very
                                        // thing an upright bow IS. Restated
                                        // rather than dropped, because the
                                        // requirement was never "below the
                                        // crosshair" - it was "not ON it".
                                        // The bow answers the same per-corner
                                        // circle test as every other weapon
                                        // now, and a limb may pass above the
                                        // circle so long as it stays out of
                                        // it.
                                        ScreenProfile::BowDrawn => {
                                            for (pl, pu) in &corners {
                                                let dx = pose.x - pl - sway;
                                                let dy = pose.y + pu;
                                                let d = (dx * dx + dy * dy).sqrt();
                                                assert!(
                                                    d > r,
                                                    "{kind:?}: a bow corner sits ON the \
                                                     crosshair - d {d:.3} <= r {r:.3} \
                                                     (sf {sf} th {th} pull {pull} \
                                                     cook {cook})"
                                                );
                                            }
                                        }
                                        _ => {
                                            // midline: the widest thing on
                                            // the weapon, wherever it is
                                            assert!(
                                                pose.x - bl - sway > 0.0,
                                                "{kind:?} ({prof:?}): the bounded part \
                                                 crosses the midline at {:.3} \
                                                 (sf {sf} th {th} cook {cook})",
                                                pose.x - bl - sway
                                            );
                                            // circle: every REAL corner
                                            for (pl, pu) in &corners {
                                                let dx = pose.x - pl - sway;
                                                let dy = pose.y + pu;
                                                let d = (dx * dx + dy * dy).sqrt();
                                                assert!(
                                                    d > r,
                                                    "{kind:?} ({prof:?}): a part corner \
                                                     is inside the centre circle - \
                                                     d {d:.3} <= r {r:.3} \
                                                     (sf {sf} th {th} kick {kick} sp {sp})"
                                                );
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// What the arsenal's extremes actually ARE, pinned - so a model that
    /// grows past them names itself instead of drifting.
    ///
    /// This is the test that retired the audited budgets, and it retired
    /// them by disagreeing with them. `VM_RECEIVER_LEFT`/`VM_MAST_UP`
    /// carried the note "current widest receiver = minigun cluster (0.069
    /// left); tallest mast = AWM scope (0.085 up)". Measured:
    ///
    ///   widest   MINIGUN  0.065 up-left   (the note was close, and
    ///                                      conservative, which is fine)
    ///   tallest  M249     0.192 up        (the note said 0.085, and
    ///                                      named the wrong weapon)
    ///
    /// The M249 is 2.3x the claimed ceiling, and the reason is ordinary:
    /// its arched carry handle was raised clear of the sight line in a
    /// later change, and the comment describing the arsenal was not part
    /// of that change. Nothing could catch it, because the geometry could
    /// not be read - which is the whole argument for `weapon_parts`.
    ///
    /// It is NOT a screen intrusion. The M249 carries lower and further
    /// out than the generic placement (`vm_carry`), and
    /// `every_weapon_holds_its_own_screen_profile` clears it at its own
    /// carry. The defect was the claim, not the gun.
    #[test]
    fn the_arsenals_extremes_are_what_we_think_they_are() {
        let mut widest = (GunKind::Fists, 0.0_f32);
        let mut tallest = (GunKind::Fists, 0.0_f32);
        for kind in ALL_WEAPONS {
            if screen_profile(kind) != ScreenProfile::Strict {
                continue; // the polearm and the bow have their own rules
            }
            let (l, u) = weapon_bounded_extent(kind);
            if l > widest.1 {
                widest = (kind, l);
            }
            if u > tallest.1 {
                tallest = (kind, u);
            }
        }
        assert_eq!(
            widest.0,
            GunKind::Minigun,
            "the widest gun is the minigun's barrel cluster, not {:?} at {:.3}",
            widest.0,
            widest.1
        );
        // 0.085 as of the §owner GUN PASS - the motor ribs stand proud of
        // the barrel cluster, which is 2 cm wider than the cluster alone
        // was. Deliberate, and re-read against the sweep: the minigun's
        // own carry clears the midline with room, so the wider motor
        // costs nothing on screen.
        assert!(
            (widest.1 - 0.085).abs() < 0.002,
            "the minigun now reaches {:.4} left, not 0.085 - if that is \
             deliberate, move this number and re-read the sweep",
            widest.1
        );
        assert_eq!(
            tallest.0,
            GunKind::M249,
            "the tallest gun is the M249's carry handle, not {:?} at {:.3}",
            tallest.0,
            tallest.1
        );
        assert!(
            (tallest.1 - 0.192).abs() < 0.002,
            "the M249 now reaches {:.4} up, not 0.192",
            tallest.1
        );
    }

    /// The profiles exempt what they say they exempt, and nothing else.
    ///
    /// The spear's is the one worth pinning: bounding its 1.85 m SHAFT
    /// would be bounding the raised javelin's whole silhouette, which is
    /// the thing the profile exists to permit.
    #[test]
    fn each_profile_exempts_exactly_what_it_claims() {
        let spear = weapon_parts(GunKind::Spear);
        let bounded: Vec<&WPart> = spear
            .iter()
            .filter(|w| profile_bounds_part(ScreenProfile::SpearRaised, w))
            .collect();
        assert!(!bounded.is_empty(), "the grip cannot be empty");
        assert!(
            bounded.len() < spear.len(),
            "SpearRaised must exempt SOMETHING, or it is just Strict"
        );
        // the shaft - the longest part - must be among the exempt
        let longest = spear
            .iter()
            .max_by(|a, b| a.half().z.partial_cmp(&b.half().z).unwrap())
            .unwrap();
        assert!(
            !profile_bounds_part(ScreenProfile::SpearRaised, longest),
            "the shaft is bounded - that is the javelin's silhouette, not an \
             intrusion"
        );
        // and every bounded part really is at the hand
        for w in bounded {
            assert!(
                w.pos.z.abs() <= GRIP_WINDOW_M,
                "a part at z {} is not the grip",
                w.pos.z
            );
        }
        // a gun exempts nothing
        for w in &weapon_parts(GunKind::Ak47) {
            assert!(profile_bounds_part(ScreenProfile::Strict, w));
        }
    }

    /// The lift out of `spawn_weapon_model` was a pure MOVE.
    ///
    /// Every weapon still has parts, the minigun still has exactly the
    /// nine that spin, and no other weapon has any - if the `spin` flag
    /// had leaked onto a shared helper this is where it would show.
    #[test]
    fn the_part_tables_survived_being_lifted_out() {
        for kind in ALL_WEAPONS {
            let parts = weapon_parts(kind);
            assert!(!parts.is_empty(), "{kind:?} has no geometry at all");
            for w in &parts {
                assert!(w.size.min_element() > 0.0, "{kind:?}: a zero-sized part");
                assert!(w.pos.is_finite() && w.size.is_finite(), "{kind:?}: NaN part");
            }
            let spinning = parts.iter().filter(|w| w.spin).count();
            let expect = if kind == GunKind::Minigun { 9 } else { 0 };
            assert_eq!(
                spinning, expect,
                "{kind:?} has {spinning} spinning parts, expected {expect} - \
                 the barrel cluster is a spine, two caps and six barrels"
            );
        }
        // `Fists` is the one weapon that legitimately has no model, and
        // it is not in ALL_WEAPONS - assert that stays true, since the
        // loop above would fail loudly if it were ever added
        assert!(!ALL_WEAPONS.contains(&GunKind::Fists));
    }

    // ---- §owner BOW & SPEAR ------------------------------------------

    /// §2/§6: *"weapon on the right, crosshair never obstructed, nothing
    /// crosses the centre."*
    ///
    /// Fails on the code this replaced: the bow was carried at x -0.16,
    /// which is the wrong side of the screen outright.
    #[test]
    fn the_bow_and_the_spear_are_carried_on_the_right() {
        for kind in [GunKind::Bow, GunKind::Spear] {
            let c = vm_carry(kind);
            assert!(
                c.pos.x > 0.10,
                "{kind:?} is carried at x {:.3} - the owner's rule is the \
                 RIGHT of frame",
                c.pos.x
            );
        }
        // and the bow's cant is a ROLL, not the pitch it used to be: a
        // pitch cannot move a limb sideways, which is the entire job.
        assert!(
            vm_carry(GunKind::Bow).roll.abs() > 0.2,
            "the bow is not canted - the upper limb has nothing taking it \
             out of the centre"
        );
    }

    /// §2/§6 THE SACRED CENTRE, projected properly.
    ///
    /// Every part corner of the bow and the spear, at every SUSTAINED
    /// pose either can be held in, must land outside the 12%-of-screen-
    /// height circle at the crosshair - measured at the depth the part
    /// is actually drawn at, which is the whole point (see
    /// `VmCarry::screen_point`).
    ///
    /// Mutation-proved against the shipped code twice over: the old
    /// spear carry `(0.15, -0.10, -0.28)` puts its own point 0.16 half
    /// -heights from centre, well inside the 0.24 circle, and the old
    /// bow carry puts the riser on the left-hand side of it.
    #[test]
    fn the_bow_and_the_spear_leave_the_centre_of_the_screen_empty() {
        for kind in [GunKind::Bow, GunKind::Spear] {
            let c = vm_carry(kind);
            let parts = weapon_parts(kind);
            for pull in [0.0_f32, 0.5, 1.0] {
                for ads in [0.0_f32, 1.0] {
                    for sprint in [0.0_f32, 1.0] {
                        // every sustained shift the render path adds,
                        // at the same signs `fp_viewmodel` uses
                        // READ from production. Re-typing these three
                        // numbers here is what let the shift change
                        // without either guard noticing.
                        let pre = preaim_shift(kind);
                        let draw = if kind == GunKind::Bow { pull } else { 0.0 };
                        let shift = pre * ads
                            + VM_BOW_DRAW_SHIFT * draw
                            + carry_offset(
                                sprint,
                                pull * 6.28,
                                true,
                                0.0,
                                sprint,
                                0.0,
                                vm_steady(kind),
                            )
                            - VM_SUPPRESS_SHAKE;
                        for w in &parts {
                            let h = w.half();
                            for sx in [-1.0_f32, 1.0] {
                                for sy in [-1.0_f32, 1.0] {
                                    for sz in [-1.0_f32, 1.0] {
                                        let local =
                                            w.pos + Vec3::new(sx * h.x, sy * h.y, sz * h.z);
                                        let Some(s) = c.screen_point(shift, local) else {
                                            continue; // behind the eye, not drawn
                                        };
                                        assert!(
                                            s.length() > VM_CENTRE_CLEAR,
                                            "{kind:?}: a corner lands {:.3} half-heights \
                                             from the crosshair (limit {VM_CENTRE_CLEAR}) \
                                             at pull {pull} ads {ads} sprint {sprint}",
                                            s.length()
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// §6: *"during pre-aim it comes slightly closer and further RIGHT,
    /// never toward the centre."*
    ///
    /// Fails on the shipped code, which sent BOTH weapons to the
    /// midline on right-click: the no-iron-sights branch of `ads_shift`
    /// applies `-tr.x`, which cancels the carry exactly.
    #[test]
    fn pre_aim_moves_the_bow_and_the_spear_outward() {
        for kind in [GunKind::Bow, GunKind::Spear] {
            let c = vm_carry(kind);
            // the spear's own head is the part §6 is written about
            let nose = Vec3::new(0.0, 0.0, if kind == GunKind::Spear { 1.545 } else { 0.0 });
            let hip = c.screen_point(Vec3::ZERO, nose).unwrap();
            // the PRODUCTION shift, not a copy of it - `ads_shift`
            // calls this same function
            let pre = preaim_shift(kind);
            let aimed = c.screen_point(pre, nose).unwrap();
            assert!(
                aimed.x > hip.x,
                "{kind:?}: pre-aim moved it from screen x {:.3} to {:.3} - \
                 that is toward the centre, not away from it",
                hip.x,
                aimed.x
            );
        }
    }

    /// §8 REDUCED SWAY: the bow and the spear, and only those two.
    #[test]
    fn the_bow_and_spear_carry_steadier_than_the_rifles() {
        for kind in ALL_WEAPONS {
            let s = vm_steady(kind);
            let want_calm = matches!(kind, GunKind::Bow | GunKind::Spear);
            assert_eq!(
                s < 1.0,
                want_calm,
                "{kind:?}: steadiness {s} - only the bow and the spear are \
                 damped, the guns keep the motion they have"
            );
        }
        // and it really reaches the bob, at full amplitude
        let calm = carry_offset(1.0, 1.2, true, 0.0, 0.0, 0.0, vm_steady(GunKind::Bow));
        let loud = carry_offset(1.0, 1.2, true, 0.0, 0.0, 0.0, vm_steady(GunKind::Ak47));
        assert!(
            calm.x.abs() < loud.x.abs() * 0.6 && calm.y.abs() < loud.y.abs() * 0.6,
            "the damped bob is not damped: {calm:?} vs {loud:?}"
        );
        // ...and it damps the BOB ONLY. Sprint-lower and the landing dip
        // are reads the player is owed at full size on every weapon.
        let a = carry_offset(0.0, 0.0, true, 0.0, 1.0, 0.05, 0.45);
        let b = carry_offset(0.0, 0.0, true, 0.0, 1.0, 0.05, 1.0);
        assert_eq!(a, b, "steadiness must not touch the sprint lower or the dip");
    }

    /// §5: ONE SPEAR, EVERYWHERE - the held javelin and the thrown one
    /// are built from the same table, so they cannot be different
    /// objects. Fails on the shipped code, where they were two hand
    /// -written tables with different part counts and palettes.
    #[test]
    fn the_thrown_spear_is_the_spear_that_was_held() {
        let held = weapon_parts(GunKind::Spear);
        let profile = spear_profile();
        assert_eq!(
            held.len(),
            profile.len(),
            "the held spear no longer comes from `spear_profile`"
        );
        for (w, p) in held.iter().zip(profile.iter()) {
            assert_eq!(w.cyl, p.cyl);
            assert_eq!(w.tone, p.tone);
            assert!((w.pos.z - p.z).abs() < 1e-6, "z drifted at {}", p.z);
        }
        // the design, checked rather than described: a dark leaf blade,
        // a red collar, and a warm-wood shaft
        assert!(profile.iter().any(|p| p.tone == Tone::Collar), "no collar");
        assert!(
            profile.iter().filter(|p| p.tone == Tone::Blade).count() >= 4,
            "the leaf blade needs its taper"
        );
        assert!(
            profile.iter().filter(|p| p.tone == Tone::Wood).count() >= 3,
            "the shaft is the warm wood, in the runs the grip window needs"
        );
        // and the midrib is genuinely PRONOUNCED - thicker through than
        // the blade flats it rides on
        let rib = profile
            .iter()
            .filter(|p| p.tone == Tone::Mid)
            .map(|p| p.t)
            .fold(0.0_f32, f32::max);
        let flat = profile
            .iter()
            .filter(|p| p.tone == Tone::Blade && !p.cyl)
            .map(|p| p.t)
            .fold(0.0_f32, f32::max);
        assert!(rib > flat * 1.5, "midrib {rib} vs blade {flat} - not pronounced");
    }

    /// §1: the bow is WOOD, LEATHER and CORD, not the firearm greys.
    #[test]
    fn the_bow_is_built_out_of_bow_materials() {
        let parts = weapon_parts(GunKind::Bow);
        let wood = parts.iter().filter(|w| w.tone == Tone::Wood).count();
        assert!(wood >= 8, "only {wood} wooden parts - riser and both limbs?");
        // AT THE CENTRE, and a WRAP rather than a stripe. `any(leather)`
        // was the first version of this line and it could not fail: the
        // arrow rest is leather too, so painting the grip grey again
        // still left one leather part on the bow.
        assert!(
            parts.iter().any(|w| {
                w.tone == Tone::Leather && w.pos.y.abs() < 0.02 && w.size.y > 0.05
            }),
            "no slim leather grip wrapped round the centre of the riser"
        );
        // "not ornate": no emissive anything on a weapon with no sights
        assert!(
            !parts.iter().any(|w| w.tone == Tone::Reticle),
            "the bow is carrying an emissive pip"
        );
        // the limbs SWEEP - a stepped limb has every segment square on
        assert!(
            parts.iter().filter(|w| w.tilt.abs() > 0.05).count() >= 8,
            "the limbs are not swept"
        );
    }

    /// §7: the wind is one fraction, and the plant starts where it ends.
    ///
    /// Fails on the shipped pose, whose plant began 77 degrees NOSE-UP
    /// with the hand at (0.16, 0.72, 0.02) - nowhere a wind could hand
    /// over from, because there was no wind pose at all.
    #[test]
    fn the_javelin_plant_begins_where_the_wind_ends() {
        let end = javelin_wind_pose(1.0);
        let start = javelin_plant_pose(0.0);
        assert!(
            (end.hand - start.hand).length() < 1e-5,
            "the hand jumps {:?} -> {:?} at the release",
            end.hand,
            start.hand
        );
        assert!((end.pitch - start.pitch).abs() < 1e-5);
        // ...and it is not continuous by being BOTH WRONG. Continuity
        // alone is satisfied by any plant that derives from the wind,
        // including one that derives the old pose - so pin the two
        // absolutes the retired plant violated: it began at hand y 0.72
        // (below the shoulder line, which is 0.62) with the shaft 1.35
        // rad NOSE-UP.
        assert!(
            start.hand.y > 0.85,
            "the throwing hand starts the plant at y {:.2} - that is not \
             above the shoulder",
            start.hand.y
        );
        assert!(
            start.pitch > 0.0,
            "the shaft starts the plant nose-UP ({:.2} rad) - a javelin is \
             cocked pointing forward and down",
            start.pitch
        );
    }

    /// §7: it must READ as a wind-up - arm high, point angled DOWN,
    /// off arm forward, hips sunk, stance wide. All monotone in the
    /// sim's own fraction so there is no frame that undoes itself.
    #[test]
    fn the_javelin_wind_reads_as_a_wind() {
        let rest = javelin_wind_pose(0.0);
        let full = javelin_wind_pose(1.0);
        assert!(full.hand.y > rest.hand.y + 0.15, "the arm never got raised");
        assert!(full.hand.z < rest.hand.z - 0.20, "the hand never went back");
        assert!(full.pitch > 0.0, "the point must angle DOWN over the shoulder");
        assert!(
            full.off_hand.z > rest.off_hand.z + 0.15,
            "the balance arm never reached forward"
        );
        assert!(full.sink > 0.0 && full.stagger > 0.2, "no braced stance");
        let mut prev = javelin_wind_pose(0.0);
        for i in 1..=20 {
            let p = javelin_wind_pose(i as f32 / 20.0);
            assert!(p.hand.y >= prev.hand.y, "the raise reverses at {i}");
            assert!(p.stagger >= prev.stagger, "the stance narrows at {i}");
            prev = p;
        }
        // the WHIP throws the hand forward past where it started
        let released = javelin_plant_pose(1.0);
        assert!(
            released.hand.z > rest.hand.z + 0.4,
            "the plant never drives through: z {} vs {}",
            released.hand.z,
            rest.hand.z
        );
    }

    /// §7: the hips coil through the CHARGE now, not only through the
    /// 0.4 s plant. Fails on the shipped `torso_coil_yaw`, which had no
    /// wind parameter and returned the follow-through's 0.0 for every
    /// second of a seven-second hold.
    #[test]
    fn the_hips_coil_while_the_javelin_charges() {
        let held = torso_coil_yaw(GunKind::Spear, None, 0.0, false, -1.0, 1.0);
        assert!(
            held.abs().to_degrees() > 30.0,
            "a fully wound thrower's hips are square: {:.1} deg",
            held.abs().to_degrees()
        );
        // and it hands over to the plant's own curve continuously
        // the FIRST tick of the plant is plant-frac 0.0 - which is
        // exactly the value a bare f32 could not tell from "no plant",
        // and the reason the parameter is an Option.
        let plant_start = torso_coil_yaw(GunKind::Spear, Some(0.0), 0.0, false, 0.0, 0.0);
        assert!(
            (held - plant_start).abs() < 1e-5,
            "coil jumps {held} -> {plant_start} at the release"
        );
        // a rifleman never twists, whatever is held
        assert_eq!(torso_coil_yaw(GunKind::M4, None, 0.0, false, -1.0, 1.0), 0.0);
    }

    /// The directional read: the strip that lights is the one facing the
    /// fire, on all four bearings.
    ///
    /// This is the whole value of the suppression HUD - without a correct
    /// bearing it is a screen flash that tells a player to panic in an
    /// unspecified direction. The right/left convention is the easy one
    /// to get backwards, and getting it backwards would send someone
    /// turning INTO the gun, so both are pinned explicitly.
    #[test]
    fn the_lit_edge_faces_the_fire_on_every_bearing() {
        let me = [0.0_f32, 0.0, 0.0];
        // camera looking down +Z (yaw 0)
        assert_eq!(edge_toward([0.0, 0.0, 10.0], me, 0.0), Some(0), "ahead");
        assert_eq!(edge_toward([0.0, 0.0, -10.0], me, 0.0), Some(2), "behind");
        assert_eq!(edge_toward([-10.0, 0.0, 0.0], me, 0.0), Some(1), "screen-right");
        assert_eq!(edge_toward([10.0, 0.0, 0.0], me, 0.0), Some(3), "screen-left");
        // turning the camera turns the read with it: face the man who was
        // behind you and he is now ahead
        assert_eq!(edge_toward([0.0, 0.0, -10.0], me, PI), Some(0));
        // a shooter standing exactly on you has no bearing to report
        assert_eq!(edge_toward(me, me, 0.0), None);
        // and height never decides a compass bearing
        assert_eq!(
            edge_toward([0.0, 40.0, 10.0], me, 0.0),
            Some(0),
            "a mech firing down from a hull is still AHEAD"
        );
    }

    /// A hit always outranks being shot at, and it does so by
    /// construction rather than by a priority rule.
    #[test]
    fn being_hit_outshouts_being_shot_at() {
        assert!(
            SUPPRESS_EDGE_ALPHA * 2.0 <= 0.55 + 1e-6,
            "suppression's ceiling ({SUPPRESS_EDGE_ALPHA}) must stay far \
             enough under the damage flash's 0.55 that a hit wins the strip \
             on alpha alone - the moment they overlap, the two messages stop \
             being distinguishable and the shared widget stops being honest"
        );
        assert!(SUPPRESS_EDGE_ALPHA > 0.1, "and it still has to be visible");
    }

    /// The exemption above is only honest if the exempt poses are bounded
    /// in BOTH senses: they displace the weapon by a finite amount, and
    /// they run on clocks that finish.
    ///
    /// The first draft of this asserted something stronger and wrong -
    /// that a reload pose is the identity at r = 0 and r = 1, so it begins
    /// and ends exactly at the carry. The SHOTGUN disproves it: its
    /// shell-by-shell feed holds a constant 6 cm dip for the whole reload
    /// because the gun is held low at the loading gate throughout, so it
    /// steps to the dip the instant the reload starts. That is the pose
    /// the weapon is supposed to have, not a defect, and a test that
    /// demanded otherwise would have been demanding a worse animation.
    #[test]
    fn transient_poses_are_bounded_in_travel_and_in_time() {
        // No reload displaces the weapon further than this at any point
        // in its run. Generous - it is a cap on runaway, not a style
        // note - but finite, which is the whole claim.
        const RELOAD_TRAVEL_CAP_M: f32 = 0.30;
        for kind in ALL_WEAPONS {
            for step in 0..=100 {
                let r = step as f32 / 100.0;
                let (t, _) = reload_pose(kind, r);
                assert!(
                    t.length() < RELOAD_TRAVEL_CAP_M,
                    "{kind:?} at r {r}: the reload throws the weapon {:.3} m - \
                     an exempt pose still has to come back",
                    t.length()
                );
                assert!(t.is_finite(), "{kind:?} at r {r}: non-finite pose {t:?}");
            }
        }
        // and the melee windows are finite and positive - a swing that
        // never ended would hold the frame open forever, which is exactly
        // the loophole the exemption must not have
        for w in [
            KNIFE_QUICK_WIND_S,
            KNIFE_QUICK_ACTIVE_S,
            KNIFE_QUICK_RECOVER_S,
            AXE_QUICK_WIND_S,
            AXE_QUICK_ACTIVE_S,
            AXE_QUICK_RECOVER_S,
        ] {
            assert!(w > 0.0 && w < 2.0, "a melee window of {w}s is not a swing");
        }
    }

    /// §3.9 (Brief VI): the four-corner layout holds at 1920×1080,
    /// 2560×1440, and 1280×720 - every cluster inside the 5% safe area,
    /// anchored to its own quadrant, no two clusters colliding.
    #[test]
    fn hud_layout_holds_at_three_resolutions() {
        for (w, h) in [(1920.0_f32, 1080.0_f32), (2560.0, 1440.0), (1280.0, 720.0)] {
            let mut pts: Vec<(&str, f32, f32)> = Vec::new();
            for (name, anchor, off) in HUD_ANCHORS {
                let x = (anchor[0] + off[0]) * w;
                let y = (anchor[1] + off[1]) * h;
                // 5% safe area on both axes
                assert!(
                    x >= w * HUD_SAFE_FRAC - 1.0 && x <= w * (1.0 - HUD_SAFE_FRAC) + 1.0,
                    "{name} x={x} outside safe area at {w}x{h}"
                );
                assert!(
                    y >= h * HUD_SAFE_FRAC - 1.0 && y <= h * (1.0 - HUD_SAFE_FRAC) + 1.0,
                    "{name} y={y} outside safe area at {w}x{h}"
                );
                // the offset must pull INTO the screen from its anchor
                if anchor[0] == 0.0 {
                    assert!(off[0] > 0.0, "{name} must hang rightward");
                }
                if anchor[0] == 1.0 {
                    assert!(off[0] < 0.0, "{name} must hang leftward");
                }
                if anchor[1] == 1.0 {
                    assert!(off[1] < 0.0, "{name} must hang upward");
                }
                pts.push((name, x, y));
            }
            // no two clusters within 12% of screen width of each other
            for i in 0..pts.len() {
                for j in (i + 1)..pts.len() {
                    let dx = pts[i].1 - pts[j].1;
                    let dy = pts[i].2 - pts[j].2;
                    let d = (dx * dx + dy * dy).sqrt();
                    assert!(
                        d > w * 0.12,
                        "{} and {} collide at {w}x{h}: {d:.0}px apart",
                        pts[i].0,
                        pts[j].0
                    );
                }
            }
        }
    }

    /// No equip hint may span the frame.
    ///
    /// `equip_hint`'s own comment records the rule and the incident:
    /// the medic's ran to 120 characters, "off the right edge of the
    /// frame and over the vitals panel", and every other entry "sits
    /// near 55 - and that is not a coincidence, it is the width that
    /// fits". The rule was written down and then not enforced, so the
    /// Big and Royal hints shipped at 78 and crossed screen centre -
    /// the one region BRIEF_XII and both reference HUDs keep clear.
    ///
    /// Asserted over EVERY variant rather than over the two strings that
    /// were wrong, so a new chassis cannot reintroduce it.
    ///
    /// `ArmorSet` has no `ALL` constant and adding one belongs to the sim
    /// lane, so the list is spelled out here and guarded: the `match`
    /// below is exhaustive, so adding a variant to `ArmorSet` fails to
    /// compile until it is added here too. That is the same protection an
    /// `ALL` would give, without reaching across the line to write it.
    #[test]
    fn no_equip_hint_spans_the_frame() {
        const HINT_MAX: usize = 60;
        let every = [
            ArmorSet::None,
            ArmorSet::Folk,
            ArmorSet::RobotSuit,
            ArmorSet::Recon,
            ArmorSet::ScoutMech,
            ArmorSet::RoyalMech,
        ];
        for set in every {
            // exhaustive, so a new variant breaks the build here
            let _: () = match set {
                ArmorSet::None
                | ArmorSet::Folk
                | ArmorSet::RobotSuit
                | ArmorSet::Recon
                | ArmorSet::ScoutMech
                | ArmorSet::RoyalMech => (),
            };
            let h = equip_hint(set);
            assert!(
                h.len() <= HINT_MAX,
                "{set:?}'s hint is {} chars, which spans the frame: {h:?}",
                h.len()
            );
        }
    }

    /// The Forge row is built FROM `FORGE_SLOTS`, not beside it.
    ///
    /// The constant was declared and read by nothing while the menu
    /// spelled out SAVE 1/2/3 next to it, so raising it would have moved
    /// the number the code believes in and left the menu unchanged.
    /// **`len() == FORGE_SLOTS` would be vacuous** — both sides read the
    /// same constant, so it passes at any value and proves nothing. Rule
    /// 12. What is asserted instead are properties that can actually be
    /// false: the labels are 1-based and contiguous (they were typed
    /// `SAVE 1..3` by hand while `forge_slot_path` is indexed from the
    /// button's number), and **every slot the menu advertises gets its
    /// own file**. A menu offering a slot that collides with another
    /// slot's save is the failure this guards.
    #[test]
    fn every_advertised_forge_slot_has_its_own_file() {
        let saves = forge_slot_labels("SAVE");
        assert!(!saves.is_empty(), "a Forge with no slots is not a Forge");
        let mut paths = std::collections::HashSet::new();
        for (i, label) in saves.iter().enumerate() {
            let slot = i + 1;
            assert_eq!(
                label,
                &format!("SAVE {slot}"),
                "labels must be 1-based and contiguous - a player counts \
                 from one, and `ForgeUiButton::Save` passes this number \
                 straight to `forge_slot_path`"
            );
            assert!(
                paths.insert(forge_slot_path(slot)),
                "slot {slot} shares a save file with an earlier slot"
            );
        }
        // SAVE and LOAD must address the same slots or one writes where
        // the other cannot read.
        let loads = forge_slot_labels("LOAD");
        assert_eq!(loads.len(), saves.len());
        for (s, l) in saves.iter().zip(loads.iter()) {
            assert_eq!(
                s.trim_start_matches("SAVE"),
                l.trim_start_matches("LOAD"),
                "the SAVE and LOAD rows address different slots"
            );
        }
    }

    /// §3.9: the semantic thresholds flip at EXACTLY the specified
    /// values, and the killfeed glyph mapping renders from a scripted
    /// event stream.
    #[test]
    fn hud_thresholds_and_glyphs() {
        // vitals: white above 25, red at ≤25, pulsing (alpha < 1) at ≤20
        assert_eq!(vitals_color(25.1, 0.0), Color::srgb(0.95, 0.96, 0.98));
        assert_eq!(vitals_color(25.0, 0.0), Color::srgb(1.0, 0.18, 0.15));
        let pulsing = vitals_color(20.0, 0.4);
        assert!(pulsing.alpha() < 1.0, "≤20 HP must PULSE");
        // ammo: red at exactly ≤25% of the magazine
        assert!(!ammo_is_low(8, 30)); // 26.7%
        assert!(ammo_is_low(7, 30)); // 23.3%
        assert!(ammo_is_low(5, 20)); // exactly 25%
        assert!(!ammo_is_low(0, 0)); // fists: never "low"
        // §4.5 killfeed glyphs from a scripted stream. Each modifier has
        // its own mark, they COMPOSE (a blind noscope headshot earns all
        // three), and a plain kill earns none - an empty string, not a
        // pad, so the row does not reserve space for a badge it lacks.
        let stream = [
            ((false, false, false, false, false), ""),
            ((true, false, false, false, false), "*"),
            ((false, true, false, false, false), "o"),
            ((false, false, true, false, false), "?"),
            ((false, false, false, true, false), "~"),
            ((false, false, false, false, true), "#"),
            ((true, true, false, false, false), "*o"),
            ((true, true, true, true, true), "*o?~#"),
        ];
        for ((hs, ns, bl, sm, wb), want) in stream {
            assert_eq!(
                feed_glyphs(hs, ns, bl, sm, wb),
                want,
                "glyphs for headshot={hs} noscope={ns} blind={bl} smoke={sm} wallbang={wb}"
            );
        }
    }

    /// §owner: every firearm carries a 1x red-dot optic, and the
    /// reticle sits at EXACTLY the height focus aligns to the eye.
    ///
    /// This is the test the M249 needed and did not have. It shipped
    /// with a sight line 2 mm above a flat feed cover and no rear
    /// aperture at all, so aiming laid a 30 cm plate across the view -
    /// a defect no unit test could see, because nothing tied the
    /// declared number to the geometry that number is about.
    ///
    /// `push_red_dot(y, _)` builds its cross at `y`; `sight_line_y`
    /// says where the eye goes. If those two ever disagree the optic
    /// becomes decoration and the player aims with a cross that is not
    /// on the target.
    #[test]
    fn every_firearm_carries_an_aligned_optic() {
        // (kind, the y passed to push_red_dot in spawn_weapon_model)
        let optics = [
            (sim::GunKind::Glock, 0.1075_f32),
            (sim::GunKind::Deagle, 0.1300),
            (sim::GunKind::Mp5, 0.1160),
            (sim::GunKind::Shotgun, 0.0950),
            (sim::GunKind::Ak47, 0.1060),
            (sim::GunKind::M4, 0.1120),
            (sim::GunKind::M249, 0.1265),
            (sim::GunKind::Minigun, 0.1120),
        ];
        for (kind, reticle_y) in optics {
            let declared = sight_line_y(kind).unwrap_or_else(|| {
                panic!("{kind:?} carries an optic but declares no sight line")
            });
            assert!(
                (declared - reticle_y).abs() < 1e-6,
                "{kind:?}: the reticle sits at {reticle_y} but focus aligns                  {declared} to the eye - the cross would not be on the                  crosshair"
            );
        }
        // The AWM is the deliberate exception: `vm_hidden_while_scoped`
        // deletes its viewmodel while zoomed, so a modelled optic could
        // never be seen. Its illuminated cross lives in the scope
        // overlay instead. Any sight line here would be a lie.
        assert!(
            sight_line_y(sim::GunKind::Awm).is_none(),
            "the scoped rifle must not declare a viewmodel sight line"
        );
        // Fists, bow and spear have no sights to align.
        for kind in [sim::GunKind::Fists, sim::GunKind::Bow, sim::GunKind::Spear] {
            assert!(sight_line_y(kind).is_none(), "{kind:?} has no sights");
        }
    }

    /// The optic must be a WINDOW, not a block: the frame bars sit
    /// outside the clear aperture, and the cross arms stop short of the
    /// frame. A reticle that touched the frame would read as painted-on
    /// rather than projected, and a frame that overlapped the window
    /// would be the grey wall all over again.
    #[test]
    fn the_optic_window_stays_clear() {
        let mut parts = Vec::new();
        push_red_dot(&mut parts, 0.10, 0.0, 0.06);
        let reticles: Vec<&WPart> =
            parts.iter().filter(|p| p.tone == Tone::Reticle).collect();
        assert_eq!(reticles.len(), 1, "the reticle is ONE dot");
        let dot = reticles[0];
        assert!(
            (dot.pos.y - 0.10).abs() < 1e-6 && dot.pos.x.abs() < 1e-6,
            "the dot must be centred on the sight line"
        );
        // and it must stay inside the glass even at FULL recoil drift -
        // a dot that slid behind the housing would read as a rendering
        // bug rather than as recoil
        let farthest = dot.size.y * 0.5 + RETICLE_DRIFT_M;
        assert!(
            farthest < OPTIC_HALF,
            "the dot leaves the window at full drift: {farthest} vs the              {OPTIC_HALF} aperture"
        );
        // Every piece of the TUBE clears the aperture.
        //
        // Measured RADIALLY, which is the change the octagonal tube
        // forced and is also the more honest question. The old form
        // asked "is this bar clear in x OR clear in y", which is the
        // right test for a square frame and the wrong one for a ring:
        // the four corner rounds sit on the diagonal, clear of the
        // aperture by construction, yet clear in NEITHER axis alone.
        //
        // Each primitive is measured as the primitive it is. A box
        // contributes the nearest point of its footprint; a cylinder
        // contributes its axis distance minus its radius, because a
        // corner round's AABB is meaningfully fatter than the round
        // itself and bounding it by that box would reject geometry
        // that never enters the window.
        let mut saw_round = false;
        for b in parts.iter().filter(|p| p.tone == Tone::Black) {
            let (cx, cy) = (b.pos.x, b.pos.y - 0.10);
            let near = if b.cyl {
                saw_round = true;
                // a Z-aligned round: radius is half its X extent
                (cx * cx + cy * cy).sqrt() - b.size.x * 0.5
            } else {
                let dx = (cx.abs() - b.size.x * 0.5).max(0.0);
                let dy = (cy.abs() - b.size.y * 0.5).max(0.0);
                (dx * dx + dy * dy).sqrt()
            };
            assert!(
                near >= OPTIC_HALF - 1e-6,
                "the tube intrudes into the clear window: {near} vs the {OPTIC_HALF} aperture"
            );
        }
        // and the tube really is a ROUND one - four straight walls with
        // the diagonals filled. Without this the check above is happy
        // with the flat square frame it replaced.
        assert!(saw_round, "the optic body must be a tube, not a flat frame");
        // the objective LENS: present, translucent, and filling the
        // window rather than blocking it.
        let glass: Vec<&WPart> = parts.iter().filter(|p| p.tone == Tone::Glass).collect();
        assert_eq!(glass.len(), 1, "one objective lens");
        assert!(glass[0].cyl, "the lens is a disc");
        assert!(
            glass[0].size.x * 0.5 <= OPTIC_HALF + 1e-6,
            "the lens is wider than the aperture it sits in"
        );
        // the lens sits FORWARD of the dot - a reflex projects its
        // reticle onto the glass from behind, never in front of it.
        assert!(glass[0].pos.z > dot.pos.z, "the lens must be forward of the dot");
    }

    /// §1.4 Rule-2 gate: scoped + zoomed = the viewmodel is not rendered.
    #[test]
    /// §owner TEXTURE PIPELINE: the generators are pure functions of a
    /// seed, they TILE seamlessly, and they only ever darken.
    ///
    /// All three properties are load-bearing. Purity keeps every machine
    /// (and every capture) identical. Seamlessness is what makes a 128px
    /// tile usable across a 60 m map - a visible seam is worse than no
    /// texture. And "never brighter than white" is the one that already
    /// bit once: the first encoding centred the multiplier on mid-grey,
    /// which silently halved the brightness of every surface it touched.
    #[test]
    fn generated_textures_tile_and_only_darken() {
        // pure: same input, same output, every time
        for (x, y, seed) in [(0u32, 0u32, 7u32), (63, 17, 11), (127, 127, 29)] {
            assert_eq!(tex_hash(x, y, seed), tex_hash(x, y, seed));
            assert_eq!(tex_fbm(x, y, seed), tex_fbm(x, y, seed));
        }
        // seamless: the noise wraps, so the left edge matches where the
        // right edge would continue to
        for y in (0..TEX_SIZE).step_by(7) {
            let left = tex_noise(0, y, 8, 5);
            let wrapped = tex_noise(TEX_SIZE, y, 8, 5);
            assert!(
                (left - wrapped).abs() < 1e-6,
                "the tile must wrap at x: {left} vs {wrapped} at y={y}"
            );
        }
        for x in (0..TEX_SIZE).step_by(7) {
            let top = tex_noise(x, 0, 8, 5);
            let wrapped = tex_noise(x, TEX_SIZE, 8, 5);
            assert!(
                (top - wrapped).abs() < 1e-6,
                "the tile must wrap at y: {top} vs {wrapped} at x={x}"
            );
        }
        // in range: fbm never leaves 0..1, so no generator can be pushed
        // past white by its own noise term
        for x in (0..TEX_SIZE).step_by(11) {
            for y in (0..TEX_SIZE).step_by(11) {
                let v = tex_fbm(x, y, 3);
                assert!((0.0..=1.0).contains(&v), "fbm out of range: {v}");
            }
        }
    }

    /// §2.5: the Vec3 spring is the SAME math as the 2D one, not a
    /// second solver - it must critically damp on every axis, reach its
    /// target, and never overshoot.
    #[test]
    fn the_three_axis_spring_settles_without_overshoot() {
        let target = Vec3::new(0.4, -0.2, 0.7);
        let mut x = Vec3::ZERO;
        let mut v = Vec3::ZERO;
        let mut max_over = 0.0_f32;
        for _ in 0..400 {
            let (nx, nv) = damped_spring3(x, v, target, SPRING_K_HAND_FOLLOW, 1.0 / 120.0);
            x = nx;
            v = nv;
            // critical damping never crosses the target
            for a in 0..3 {
                let over = (x[a] - target[a]) * target[a].signum();
                max_over = max_over.max(over);
            }
        }
        assert!(
            max_over < 1e-3,
            "a critically damped spring must not overshoot, got {max_over}"
        );
        assert!(
            (x - target).length() < 1e-3,
            "the spring must actually arrive: {x:?} vs {target:?}"
        );
        // a stiffer spring must arrive sooner - this is what makes the
        // named k values mean something rather than being decoration
        let settle = |k: f32| -> usize {
            let (mut x, mut v) = (Vec3::ZERO, Vec3::ZERO);
            for i in 0..2000 {
                let (nx, nv) = damped_spring3(x, v, target, k, 1.0 / 120.0);
                x = nx;
                v = nv;
                if (x - target).length() < 0.01 {
                    return i;
                }
            }
            usize::MAX
        };
        assert!(
            settle(SPRING_K_FINGER_SETTLE) < settle(SPRING_K_HAND_FOLLOW),
            "fingers (k=220) must settle faster than the hand (k=120)"
        );
        assert!(
            settle(SPRING_K_HAND_FOLLOW) < settle(SPRING_K_ELBOW_POLE),
            "the hand (k=120) must arrive before the elbow pole (k=60) - \
             that lag IS the secondary motion"
        );
        assert!(
            settle(SPRING_K_ELBOW_POLE) < settle(SPRING_K_SHOULDER),
            "the clavicle (k=45) must be the slowest link in the chain"
        );
    }

    #[test]
    fn vm_hides_while_scoped() {
        assert!(vm_hidden_while_scoped(true, true));
        assert!(!vm_hidden_while_scoped(true, false));
        assert!(!vm_hidden_while_scoped(false, true));
        assert!(!vm_hidden_while_scoped(false, false));
    }

    /// The vm camera draws AFTER the UI camera, so a rendered gun would
    /// composite over every menu plate. The gate must close in every
    /// non-Playing state and open again in Playing.
    #[test]
    fn vm_hides_while_menu_open() {
        for s in [
            GameState::Intro,
            GameState::Paused,
            GameState::Settings,
            GameState::Manual,
            GameState::Controls,
        ] {
            assert!(!vm_rendered(&s, 0.0, true, 0.0), "hidden in {s:?}");
        }
        assert!(vm_rendered(&GameState::Playing, 0.0, true, 0.0));
        // the pre-existing gates still hold in Playing
        assert!(!vm_rendered(&GameState::Playing, 1.0, true, 0.0), "third person");
        assert!(!vm_rendered(&GameState::Playing, 0.0, false, 0.0), "dead");
        assert!(!vm_rendered(&GameState::Playing, 0.0, true, 0.5), "mid-roll");
    }

    /// §5 (Brief IV): the interpenetration sweep - for every weapon in
    /// every firearm stance, the rear point (grip + stock) must land
    /// OUTSIDE the chest ellipse. Same static-guarantee machinery as the
    /// §1.3 gap test: the offsets are constants, so this holds per-frame.
    #[test]
    fn weapon_stock_clears_the_chest_in_every_stance() {
        // chest ellipse half-extents at weapon height (x, z)
        let (cx, cz) = (0.20_f32, 0.15_f32);
        for gun_k in ALL_WEAPONS {
            if matches!(gun_k, GunKind::Bow | GunKind::Spear) {
                continue; // carried on other mounts, cleared vertically
            }
            let rear = weapon_rear_extent(gun_k);
            if rear < 0.25 {
                continue; // no stock - a pistol grip NEAR the chest is
                          // a grip, not a clip
            }
            for z_root in [WR_Z_HIP, WR_Z_ADS] {
                let rz = z_root - rear;
                let inside = (WR_X / cx).powi(2) + (rz / cz).powi(2) < 1.0 - 0.05;
                assert!(
                    !inside,
                    "{gun_k:?}: stock at (x {WR_X}, z {rz:.3}) pierces the chest"
                );
            }
        }
    }

    /// §1.3 (Brief IV): connectivity - every parent–child pair overlaps
    /// its joint geometry by ≥5 mm. Bone lengths are rotation-invariant,
    /// so these static assertions hold at every phase of every clip:
    /// the overlay finds gaps, this test keeps them fixed.
    #[test]
    fn rig_joints_bridge_with_no_daylight() {
        let min = 0.005_f32;
        // NECK: sunk into the yoke below, past the head pivot above,
        // and still inside the head across the full ±48° pitch
        let yoke_top = 0.625 + 0.07;
        assert!(yoke_top - NECK_BOT >= 0.02, "neck must sink into the yoke");
        assert!(NECK_TOP - 0.846 >= 0.015, "neck must pierce the head base");
        assert!(
            NECK_TOP - 0.846 >= NECK_R * 0.75,
            "neck crossing survives full head pitch"
        );
        // YOKE reaches past the shoulder pivots
        assert!(YOKE_HALF_W - SHOULDER_X >= min, "yoke must reach the shoulders");
        // ELBOW: the upper shell reaches through the ball's span, and the
        // forearm starts inside it
        let upper_end = UPPER_CENTER - UPPER_HALF;
        assert!(
            upper_end <= ELBOW_Y + ELBOW_R - min,
            "upper shell reaches the elbow ball: end {upper_end}"
        );
        let fore_start = FORE_CENTER + FORE_HALF;
        assert!(
            fore_start >= -ELBOW_R + min,
            "forearm starts inside the elbow ball: start {fore_start}"
        );
        // WRIST: forearm reaches the wrist ball; the mitten overlaps it
        let fore_end = FORE_CENTER - FORE_HALF;
        assert!(
            fore_end <= WRIST_Y + WRIST_R - min,
            "forearm reaches the wrist ball: end {fore_end}"
        );
        let mitten_top = -0.02 + 0.05;
        assert!(mitten_top >= -WRIST_R + min, "mitten overlaps the wrist ball");
        // LEGS (spawn literals): hip ball ↔ pelvis, thigh ↔ knee ball,
        // shin ↔ ankle ball
        let pelvis_bottom = 0.09 - 0.08;
        assert!(0.055 - pelvis_bottom >= min, "hip ball meets the pelvis");
        let thigh_end = -0.145 - (0.15 / 2.0 + 0.072);
        assert!(thigh_end <= -0.29 + 0.065 - min, "thigh reaches the knee");
        let shin_end = -0.14 - (0.15 / 2.0 + 0.060);
        assert!(shin_end <= -0.28 + 0.045 - min, "shin reaches the ankle");
    }

    /// §0.2 (Brief II): the head's minimum world-Y across the FULL
    /// animation phase must stay at or above the 0.82 hit-band line for
    /// every grounded gait - idle, walk, run, sprint, strafe, backpedal,
    /// crouch-walk. A gait that dips the head below the line silently
    /// converts headshots into arm hits; this test makes that a failure,
    /// not a mystery. It samples the SAME pure pose function the renderer
    /// uses, so it cannot drift from what's on screen.
    #[test]
    fn head_never_leaves_its_band_in_any_gait() {
        for crouch in [false, true] {
            let band = HEAD_BAND_FRAC * if crouch { CROUCH_HEIGHT } else { BODY_HEIGHT };
            for amp in [0.0_f32, 0.2, 0.36, 0.6] {
                for lean in [-0.07_f32, 0.0, 0.07] {
                    // the post-roll weight-absorb dip is part of the pose
                    // and must be swept too - it moves the WHOLE rig down,
                    // and was previously applied outside gait_pose where
                    // this test could not see it at all
                    for settle in [0.0_f32, 0.25, 0.5, 0.75, 1.0] {
                        for k in 0..=128 {
                            let th = k as f32 / 128.0 * std::f32::consts::TAU;
                            let y = head_base_y(crouch, th, amp, lean, settle);
                            assert!(
                                y >= band - 1e-3,
                                "crouch={crouch} amp={amp} lean={lean} settle={settle} \
                                 theta={th:.2}: head base {y:.4} below band {band:.4}"
                            );
                        }
                    }
                }
            }
        }
    }

    /// The settle dip must still DO something when there is headroom for
    /// it - a clamp that silently zeroed the whole effect would pass the
    /// band test above while deleting the feature.
    #[test]
    fn roll_settle_still_dips_when_the_band_allows_it() {
        let standing = gait_pose(false, 0.0, 0.0, 0.0, 0.0).0;
        let settled = gait_pose(false, 0.0, 0.0, 0.0, 1.0).0;
        assert!(
            settled < standing,
            "a standing fighter has band headroom, so the settle must visibly dip: \
             {standing} -> {settled}"
        );
    }
}

/// §1.4 (Brief VII) - the living-motion layer's completion gate. The
/// pure functions tested here are the SAME ones `sync_fighters` calls,
/// not copies - a passing statue test is a guarantee about the real
/// render path, not a parallel model of it.
#[cfg(test)]
mod living_motion_tests {
    use crate::*;

    #[test]
    fn id_period_stays_in_range() {
        for id in 0..64u32 {
            let p = id_period(id, 6.0, 12.0);
            assert!((6.0..=12.0).contains(&p), "id {id}: period {p} out of [6,12]");
        }
    }

    #[test]
    fn breath_never_negative_and_bounded() {
        for i in 0..200 {
            let tnow = i as f32 * 0.15;
            for heat in [0.0_f32, 0.5, 1.0] {
                let b = breath_offset(tnow, 1.7, heat);
                assert!(b >= 0.0, "breath must never sink below rest: {b}");
                assert!(b <= 0.006, "breath amplitude budget exceeded: {b}");
            }
        }
    }

    #[test]
    fn breath_rate_ramps_with_heat() {
        // the same phase point, sampled while calm vs. just after a
        // sprint, must not be in lockstep - the rate genuinely changed
        let calm: Vec<f32> = (0..40).map(|i| breath_offset(i as f32 * 0.05, 0.0, 0.0)).collect();
        let hot: Vec<f32> = (0..40).map(|i| breath_offset(i as f32 * 0.05, 0.0, 1.0)).collect();
        assert_ne!(calm, hot, "breathing heat must change the rate, not just amplitude");
    }

    #[test]
    fn weight_shift_is_silenced_by_full_gait_amplitude() {
        for i in 0..50 {
            let tnow = i as f32 * 0.3;
            assert_eq!(
                weight_shift(tnow, 0.9, 8.0, 1.0),
                0.0,
                "full-speed gait must silence the idle weight shift"
            );
        }
    }

    #[test]
    fn weight_shift_moves_while_idle() {
        // over one full 6-12s period, idle weight shift must be nonzero
        // somewhere - a fighter standing still still weight-shifts
        let period = id_period(3, 6.0, 12.0);
        let any_nonzero = (0..200)
            .map(|i| i as f32 / 200.0 * period)
            .any(|t| weight_shift(t, 0.5, period, 0.0).abs() > 1e-4);
        assert!(any_nonzero, "an idle fighter must weight-shift at some point in its period");
    }

    #[test]
    fn grip_fidget_is_a_brief_blip_not_a_constant_twitch() {
        let period = 10.0;
        let samples: Vec<f32> = (0..2000)
            .map(|i| grip_fidget(i as f32 / 2000.0 * period * 3.0, 0.0, period))
            .collect();
        let nonzero = samples.iter().filter(|v| v.abs() > 1e-6).count();
        // the blip window (0.35s) out of a 10s period is ~3.5% duty
        let frac = nonzero as f32 / samples.len() as f32;
        assert!(frac < 0.15, "grip fidget must be a brief blip, not sustained: duty {frac:.3}");
        assert!(frac > 0.0, "grip fidget must actually fire at least once");
    }

    #[test]
    fn head_glance_never_exceeds_25_degrees() {
        for i in 0..500 {
            let t = i as f32 * 0.03;
            let g = head_glance(t, 0.6, 5.0 /* way past the clamp */);
            assert!(g.abs() <= 0.436 + 1e-4, "head glance exceeded +/-25deg: {g}");
        }
    }

    #[test]
    fn head_glance_is_a_glance_not_a_stare() {
        // a target held fixed for a full 20s should NOT keep the head
        // turned the whole time - the brief's "every ~4s" cadence
        let samples: Vec<f32> = (0..2000).map(|i| head_glance(i as f32 / 100.0, 0.0, 0.4)).collect();
        let looking = samples.iter().filter(|v| v.abs() > 0.05).count();
        let frac = looking as f32 / samples.len() as f32;
        assert!(frac < 0.5, "head must return to neutral between glances: on {frac:.2} of the time");
        assert!(frac > 0.05, "the glance must actually happen sometimes: {frac:.3}");
    }

    /// THE statue test (§1.4): 30 simulated seconds of a stationary,
    /// out-of-combat fighter. At no point may every live layer of the
    /// idle stack go quiet for more than 2 continuous seconds - some
    /// combination of breathing/weight-shift/grip-fidget/head-glance
    /// must always keep the body in motion.
    #[test]
    fn statue_test_idle_layer_never_holds_still_for_2s() {
        for id in [0u32, 1, 7, 13] {
            let ph = id as f32 * 2.399;
            let wperiod = id_period(id, 6.0, 12.0);
            let gperiod = id_period(id.wrapping_add(41), 8.0, 15.0);
            let dt = 0.05;
            let steps = (30.0 / dt) as usize;
            let mut still_for = 0.0_f32;
            let mut max_still = 0.0_f32;
            let mut prev = 0.0_f32;
            for i in 0..steps {
                let t = i as f32 * dt;
                let pose = breath_offset(t, ph, 0.0)
                    + weight_shift(t, ph, wperiod, 0.0)
                    + grip_fidget(t, ph, gperiod)
                    + head_glance(t, ph, 0.3);
                let delta = (pose - prev).abs();
                prev = pose;
                if delta < 1e-5 {
                    still_for += dt;
                    max_still = max_still.max(still_for);
                } else {
                    still_for = 0.0;
                }
            }
            assert!(
                max_still <= 2.0,
                "id {id}: idle layer held perfectly still for {max_still:.2}s (> 2s budget)"
            );
        }
    }
}
/// §2.7 (Brief VII v2) - the hand/arm craft pass's completion gate.
#[cfg(test)]
mod hand_craft_tests {
    use crate::*;

    /// §owner HAND GRAPHICS PASS. Four claims about the finger row, one
    /// test, both hands. Every one of them FAILS on the pre-pass tables
    /// (one z for all four knuckles, one width for all four fingers,
    /// lengths shared in pairs, no fan) - which is the point: they are
    /// the difference between a hand and a garden rake, and nothing
    /// could state them while the numbers lived inside a spawn loop.
    #[test]
    fn the_finger_row_is_a_hand_and_not_a_rake() {
        for (name, row) in [("viewmodel", VM_FINGERS), ("world", WORLD_FINGERS)] {
            let len = |i: usize| row[i].2;
            let z = |i: usize| row[i].1;
            let w = |i: usize| row[i].3;
            let splay = |i: usize| row[i].4;
            // 1. FOUR DIFFERENT LENGTHS, middle the longest, little the
            //    shortest. Equal-length fingers are the rake.
            assert!(
                len(1) > len(0) && len(1) > len(2) && len(2) > len(3),
                "{name}: middle is not the longest finger: {:?}",
                (len(0), len(1), len(2), len(3))
            );
            for a in 0..4 {
                for b in (a + 1)..4 {
                    assert!(
                        (len(a) - len(b)).abs() > 1e-4,
                        "{name}: fingers {a} and {b} are the same length"
                    );
                }
            }
            // 2. THE KNUCKLE ARC: the index and middle knuckles stand
            //    forward of the little one by a readable margin.
            assert!(
                z(1) > z(3) + 0.008,
                "{name}: the knuckle row is flat ({:?})",
                (z(0), z(1), z(2), z(3))
            );
            // 3. THE TAPER ACROSS THE HAND: the little finger is
            //    slighter than the index.
            assert!(w(3) < w(0) - 1e-4, "{name}: every finger is the same gauge");
            // 4. THE FAN opens outward - the two ends of the row lean
            //    opposite ways, and no finger is left un-fanned by
            //    accident in the middle of a fanned row.
            assert!(
                splay(0) > 0.05 && splay(3) < -0.05,
                "{name}: the fingers are parallel planks: {:?}",
                (splay(0), splay(1), splay(2), splay(3))
            );
            for i in 0..3 {
                assert!(
                    splay(i) > splay(i + 1),
                    "{name}: the fan reverses between {i} and {}",
                    i + 1
                );
            }
        }
    }

    /// Joint-limit fuzz: 10,000 seeded random IK targets - the solved
    /// elbow flex must never exceed the biomechanical clamp, and never
    /// produce NaN (a degenerate target is a real risk this close to a
    /// two-bone solver's reach limit).
    #[test]
    fn elbow_flex_fuzz_never_exceeds_clamp_or_nans() {
        // a plain seeded LCG - this test doesn't need the sim's Pcg32
        // (main.rs has no reason to import it), just deterministic fuzz
        let mut seed = 0xE1B0_1234u32;
        let mut next01 = || {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            (seed >> 8) as f32 / (u32::MAX >> 8) as f32
        };
        let mut range = |lo: f32, hi: f32| lo + (hi - lo) * next01();
        for _ in 0..10_000 {
            let s = Vec3::new(range(-0.3, 0.3), range(1.3, 1.7), range(-0.1, 0.1));
            let t = Vec3::new(range(-0.5, 0.5), range(0.9, 1.8), range(-0.5, 0.5));
            let pole = Vec3::new(range(-1.0, 1.0), range(-1.0, 0.0), range(-1.0, 1.0));
            let (rot, flex) = solve_arm_ik(s, t, pole);
            assert!(!flex.is_nan(), "flex went NaN: s={s:?} t={t:?} pole={pole:?}");
            assert!(
                flex.to_degrees() >= ELBOW_FLEX_MIN_DEG - 0.01
                    && flex.to_degrees() <= ELBOW_FLEX_MAX_DEG + 0.01,
                "flex {:.1}deg outside clamp: s={s:?} t={t:?}",
                flex.to_degrees()
            );
            assert!(rot.is_finite(), "rotation went non-finite: s={s:?} t={t:?}");
        }
    }

    /// §2.2 coupling: the DIP-equivalent joint must track the driving
    /// joint at exactly the 0.7x ratio across the full curl range, and
    /// must never exceed it (a real fingertip cannot out-curl its own
    /// knuckle).
    #[test]
    fn dip_coupling_tracks_driving_joint_at_seven_tenths() {
        for i in 0..=20 {
            let curl = i as f32 / 20.0 * 2.0;
            let driving = -1.15 * curl;
            let dip = dip_from_driving_joint(driving);
            assert!((dip - driving * 0.7).abs() < 1e-5, "coupling ratio drifted at curl={curl}");
            assert!(
                dip.abs() <= driving.abs() + 1e-5,
                "DIP ({dip}) out-curled its driving joint ({driving}) at curl={curl}"
            );
        }
    }

    /// §2.5: the shared spring is critically damped - released from an
    /// offset with zero velocity, it must never overshoot the target
    /// (that's the entire definition of critical, vs. under- or over-
    /// damped), and it must actually converge.
    #[test]
    fn damped_spring_is_critical_never_overshoots() {
        for k in [45.0_f32, 60.0, 90.0, 120.0, 220.0] {
            let mut x = Vec2::new(1.0, 0.0);
            let mut v = Vec2::ZERO;
            let target = Vec2::ZERO;
            let dt = 1.0 / 240.0;
            let mut max_abs = 0.0_f32;
            for _ in 0..600 {
                let (nx, nv) = damped_spring(x, v, target, k, dt);
                x = nx;
                v = nv;
                max_abs = max_abs.max(x.x.abs());
            }
            assert!(max_abs <= 1.0 + 1e-3, "k={k}: spring overshot its 1.0 release point");
            assert!(x.length() < 0.01, "k={k}: spring failed to converge: {x:?}");
        }
    }

    #[test]
    fn damped_spring_agrees_at_60_and_240_fps() {
        // closed-form: the SAME wall-clock trajectory regardless of step
        // size - this is the entire reason it's closed-form, not Euler.
        let run = |dt: f32, steps: usize| {
            let mut x = Vec2::new(1.0, 0.0);
            let mut v = Vec2::ZERO;
            for _ in 0..steps {
                let (nx, nv) = damped_spring(x, v, Vec2::ZERO, 90.0, dt);
                x = nx;
                v = nv;
            }
            x
        };
        let at_60 = run(1.0 / 60.0, 30); // 0.5s of settle
        let at_240 = run(1.0 / 240.0, 120); // same 0.5s, 4x the steps
        assert!(
            (at_60 - at_240).length() < 0.01,
            "60fps ({at_60:?}) and 240fps ({at_240:?}) disagree"
        );
    }

    /// §2.4: the trigger finger is a brief travel-and-return, not a
    /// sustained press - it must be at rest well before the NEXT shot on
    /// every gun's fire cycle, even the fastest.
    #[test]
    fn trigger_finger_timing_matches_the_06_10_spec() {
        assert_eq!(trigger_finger_press(0.0), 0.0, "at rest the instant a shot resolves... ");
        assert!(trigger_finger_press(0.03) > 0.0 && trigger_finger_press(0.03) < 1.0);
        assert!(
            (trigger_finger_press(TRIGGER_OUT_S) - 1.0).abs() < 1e-4,
            "full travel exactly at 0.06s"
        );
        assert!(
            (trigger_finger_press(TRIGGER_OUT_S + TRIGGER_BACK_S)).abs() < 1e-4,
            "fully returned exactly at 0.06+0.10s"
        );
        assert_eq!(trigger_finger_press(10.0), 0.0, "long idle: at rest");
    }
}

/// §5.3 (Brief VII v2) - the camera rig's completion gate.
#[cfg(test)]
mod camera_v2_tests {
    use crate::*;

    /// §C regression guard: client FX must follow the weapon that
    /// ACTUALLY fired.
    ///
    /// Every shot effect in this file — casings, muzzle flash, shot
    /// audio, camera kick — detects a fresh shot by a cooldown jumping
    /// UP. That worked while every weapon shared `fire_cd`. When the
    /// hull mounts got their own clocks (correctly: sharing `fire_cd`
    /// was throttling the pilot's carried gun on dismount) they silently
    /// stopped feeding those sites, and a firing hull gatling went
    /// flashless and silent.
    ///
    /// This pins the fix so a future clock change cannot un-wire it
    /// again without a red test.
    /// §7 FIRST-PERSON AIM IS EXACT, and stays exact.
    ///
    /// The owner reported first-person aiming as "too difficult", so the
    /// mechanism was audited end to end before any number was touched.
    /// Five hypotheses, all cleared:
    ///
    ///   1. a parallax correction wrongly applied in first person
    ///   2. the movement penalty measuring the wrong axes
    ///   3. no feedback for the live cone
    ///   4. the ADS spread benefit not reaching first person
    ///   5. the ADS settle threshold being slower than a real trigger pull
    ///
    /// The one that MATTERS is (1), because it is the only one that could
    /// make the crosshair lie about where the bullet goes, and it is the
    /// one with no visible symptom until you miss. It is clear because
    /// `muzzle_origin` returns the EYE: the two-stage aim casts from the
    /// camera to find a point, then aims at that point from the muzzle,
    /// and when muzzle == camera the second stage is the identity.
    ///
    /// That is a load-bearing coincidence, not a guarantee. The day
    /// someone gives `muzzle_origin` a barrel offset - an entirely
    /// reasonable thing to want - first-person aim silently acquires a
    /// convergence error that grows as targets get closer, and no
    /// existing test would notice. This is the test that notices.
    #[test]
    fn a_first_person_shot_goes_exactly_where_the_crosshair_points() {
        let mut s = sim::TdmSim::new(sim::MatchConfig {
            seed: 0x51AB,
            per_team: 1,
            ..Default::default()
        });
        let p = s.player;
        s.fighters[p].lean = 0.0;
        // THE CAMERA POSITION IS COMPUTED INDEPENDENTLY, and that is the
        // whole test. The first version placed it at
        // `muzzle_origin(p)` - the very function under test - so a
        // mutation that moved the muzzle moved the camera with it and
        // the assertion could never fail. It was proven vacuous by
        // exactly the mutation it exists to catch.
        //
        // The camera in first person sits at the EYE, which is a fact
        // about the rig and not about the muzzle, so it is derived from
        // the fighter directly. Now the two can disagree, which is the
        // only way this test can say anything.
        let f = &s.fighters[p];
        let eye = Vec3::new(
            f.pos[0],
            f.pos[1] + sim::EYE_REL.min(f.height() - 0.12),
            f.pos[2],
        );

        // several look directions, including steeply up and down, since
        // a convergence error is a function of the aim angle
        for (yaw, pitch) in [
            (0.0_f32, 0.0_f32),
            (1.2, 0.0),
            (-2.4, 0.35),
            (0.6, -0.55),
            (3.0, 0.20),
        ] {
            let fwd = Vec3::new(
                yaw.sin() * pitch.cos(),
                -pitch.sin(),
                yaw.cos() * pitch.cos(),
            )
            .normalize();
            let cam = Transform::from_translation(eye).looking_to(fwd, Vec3::Y);
            let (dir, _) = crosshair_aim_dir(&s, &cam);
            // the shot leaves along the camera ray, to within float noise
            let err = dir.angle_between(fwd).to_degrees();
            assert!(
                err < 0.05,
                "first-person aim is off by {err:.4} deg at yaw {yaw} pitch {pitch} - \
                 the crosshair and the bullet disagree"
            );
        }

        // and it stays exact with something CLOSE in front, which is the
        // case a convergence error is worst in: aiming at a wall 2 m away
        // is where a muzzle offset would throw the shot furthest.
        let fwd = Vec3::Z;
        let cam = Transform::from_translation(eye).looking_to(fwd, Vec3::Y);
        let (near_dir, _) = crosshair_aim_dir(&s, &cam);
        assert!(
            near_dir.angle_between(fwd).to_degrees() < 0.05,
            "aim drifts when the aim point is near - that is a muzzle offset"
        );
    }

    #[test]
    fn shot_clock_follows_the_weapon_that_actually_fired() {
        // ..Default::default() rather than listing every field: this
        // struct has grown twice already, and a test that only cares
        // about two fields should not break when a third is added.
        let mut s = sim::TdmSim::new(sim::MatchConfig {
            seed: 0xFEED,
            per_team: 1,
            ..Default::default()
        });

        // On foot, the carried gun's clock is the shot clock.
        {
            let f = &mut s.fighters[0];
            f.armor_set = sim::ArmorSet::None;
            f.fire_cd = 0.11;
            f.gatling_cd = 0.0;
            f.autocannon_cd = 0.0;
        }
        assert_eq!(
            shot_clock(&s.fighters[0]),
            0.11,
            "an infantry fighter's shots run on fire_cd"
        );

        // In a mech the HULL MOUNT's clock is the shot clock - and
        // crucially it must NOT read fire_cd, or the FX fire on the
        // pilot's carried gun instead of the weapon that shot.
        {
            let f = &mut s.fighters[0];
            f.armor_set = sim::ArmorSet::RobotSuit;
            f.hull = sim::MECH_HULL;
            f.fire_cd = 0.11; // the carried rifle, NOT firing
            f.mech_weapon = sim::MechWeapon::Gatling;
            f.gatling_cd = 0.07;
            f.autocannon_cd = 0.0;
        }
        assert_eq!(
            shot_clock(&s.fighters[0]),
            0.07,
            "a piloted gatling's shots must run on gatling_cd, not the \
             carried gun's fire_cd"
        );

        // ...and switching mounts follows the selection.
        s.fighters[0].mech_weapon = sim::MechWeapon::Autocannon;
        s.fighters[0].autocannon_cd = 1.35;
        assert_eq!(
            shot_clock(&s.fighters[0]),
            1.35,
            "selecting the autocannon must move the shot clock with it"
        );
        // §C.7: the pod rides its relaunch cooldown
        s.fighters[0].mech_weapon = sim::MechWeapon::Rockets;
        s.fighters[0].pod_cd = 0.9;
        assert_eq!(
            shot_clock(&s.fighters[0]),
            0.9,
            "the ROCKETS mount's shot clock is pod_cd"
        );
        s.fighters[0].pod_cd = 0.0;
        s.fighters[0].mech_weapon = sim::MechWeapon::Autocannon;

        // The regression this guards: with the mount idle, the shot
        // clock must be ZERO even though the carried gun is hot. If this
        // ever reads 0.11 again, every mech FX site is firing off the
        // pilot's rifle.
        s.fighters[0].autocannon_cd = 0.0;
        assert_eq!(
            shot_clock(&s.fighters[0]),
            0.0,
            "an idle hull mount must read idle - a hot carried gun must \
             not make the mech look like it is shooting"
        );
    }

    /// §4.7: the death camera's phase logic. Pure, and shared by the
    /// camera and the HUD - this is the function that decides whether
    /// you are watching your killer or following them, so its edges are
    /// exactly the cases that would put a live player in a spectate cam
    /// or point a corpse camera at nobody.
    #[test]
    fn death_phase_knows_when_to_watch_and_when_to_follow() {
        let mut s = sim::TdmSim::new(sim::MatchConfig {
            seed: 0xDEAD,
            per_team: 1,
            ..Default::default()
        });

        // alive: no death camera, full stop
        assert_eq!(death_phase(&s, 0), None, "a live fighter has no death cam");

        // freshly killed by the enemy: killer-cam first
        s.fighters[0].health = 0.0;
        s.fighters[0].respawn_t = sim::RESPAWN_S;
        s.fighters[0].last_hit_by = Some((1, 0.0));
        assert_eq!(
            death_phase(&s, 0),
            Some((1, false)),
            "a fresh death looks AT the killer before following them"
        );

        // past the killer-cam window: spectate
        s.fighters[0].respawn_t = sim::RESPAWN_S - KILLER_CAM_S - 0.1;
        assert_eq!(
            death_phase(&s, 0),
            Some((1, true)),
            "after the killer-cam beat the camera follows the killer"
        );

        // suicide: nobody worth watching - ordinary corpse view
        s.fighters[0].last_hit_by = Some((0, 0.0));
        assert_eq!(
            death_phase(&s, 0),
            None,
            "a cooked frag has no killer-cam - you did this to yourself"
        );

        // no recorded attacker at all (fell, or state cleared)
        s.fighters[0].last_hit_by = None;
        assert_eq!(death_phase(&s, 0), None, "no killer, no spectate target");
    }

    /// §B.1: `visor_ready` must distinguish "fully entered" from "never
    /// boarded" — the two cases `mech_enter_stage_for` collapses into
    /// the SAME `None`. The naive `matches!(stage, None | Some(HudBoot))`
    /// the plan warns about would put an infantryman inside a visor,
    /// because a plain foot soldier also reports `None`.
    #[test]
    fn visor_ready_tells_fully_entered_apart_from_never_boarded() {
        use sim::MechEnterStage::*;

        // A fighter who never boarded: None forever, never ready.
        let mut ready = false;
        for _ in 0..5 {
            ready = visor_ready_after(None, None, ready);
        }
        assert!(
            !ready,
            "a fighter who never boarded reports None and must NEVER be visor-ready \
             - this is the exact case the naive `matches!(None | HudBoot)` gets wrong"
        );

        // A full boarding sequence, stage by stage. The camera must stay
        // OUTSIDE until the very last stage.
        let mut ready = false;
        let mut prev = None;
        for (i, s) in sim::MECH_ENTER_STAGES.iter().enumerate() {
            ready = visor_ready_after(prev, Some(*s), ready);
            prev = Some(*s);
            if *s == HudBoot {
                assert!(ready, "HudBoot must make the visor available");
            } else {
                assert!(
                    !ready,
                    "stage {}/8 ({s:?}) made the camera cut to the visor early - \
                     boarding should still be shown from outside",
                    i + 1
                );
            }
        }

        // Once earned, it survives the transition ending (stage -> None,
        // i.e. boarding complete and the timer hit zero).
        let held = visor_ready_after(Some(HudBoot), None, true);
        assert!(held, "the visor must not switch off the instant boarding completes");

        // A FRESH boarding clears it again — re-entering starts outside.
        let recleared = visor_ready_after(None, Some(CockpitOpen), true);
        assert!(
            !recleared,
            "a new boarding must put the camera back outside, not leave it in the \
             visor of the mech being climbed into"
        );
    }

    /// §B.2: the idle-life terms must actually keep the hull alive while
    /// standing still, must stay SMALL enough to read as machinery
    /// rather than a wobble, and must not secretly be the stride cycle
    /// wearing a different name.
    #[test]
    fn mech_idle_life_never_goes_perfectly_inert() {
        // 1. Never dead. Sample a long window and require real motion in
        //    every term - the whole point is that a stopped mech is not
        //    a statue. (mech_bob is zero at a standstill by design; these
        //    are what fill that silence.)
        let mut tremor_x_max = 0.0_f32;
        let mut tremor_z_max = 0.0_f32;
        let mut breath_max = 0.0_f32;
        let mut t = 0.0_f32;
        while t < 30.0 {
            let (tx, tz) = mech_servo_tremor(t);
            tremor_x_max = tremor_x_max.max(tx.abs());
            tremor_z_max = tremor_z_max.max(tz.abs());
            breath_max = breath_max.max(mech_hull_breath(t).abs());
            t += 0.01;
        }
        assert!(tremor_x_max > 0.002, "pitch tremor never moves: {tremor_x_max}");
        assert!(tremor_z_max > 0.0015, "roll tremor never moves: {tremor_z_max}");
        assert!(breath_max > 0.007, "the hull never breathes: {breath_max}");

        // 2. Small enough to read as machinery. A tremor a player can
        //    consciously SEE is a camera bug, not idle life.
        assert!(
            tremor_x_max < 0.005 && tremor_z_max < 0.005,
            "tremor is visible as motion ({tremor_x_max}, {tremor_z_max} rad) - \
             it should sit at the edge of perception"
        );
        assert!(breath_max < 0.02, "hull breath {breath_max} m reads as a bounce");

        // 3. NOT the stride cycle in disguise. mech_bob runs at 0.9 Hz;
        //    if a tremor frequency were a small multiple of that it would
        //    beat against the walk and read as bob, not tremor. Checked
        //    as a real ratio rather than trusting the literals.
        for hz in [3.1 / std::f32::consts::TAU, 2.3 / std::f32::consts::TAU] {
            let ratio = hz / 0.9;
            let nearest_harmonic = ratio.round();
            assert!(
                (ratio - nearest_harmonic).abs() > 0.1,
                "a tremor at {hz:.3} Hz is {ratio:.2}x the 0.9 Hz stride - too \
                 close to a harmonic, it will read as the walk cycle"
            );
        }

        // 4. The two tremor axes must not move as one. Identical phase
        //    would read as a single diagonal rock rather than machinery.
        let (ax, az) = mech_servo_tremor(0.0);
        assert!(
            (ax - az).abs() > 1e-4 || ax == 0.0,
            "both tremor axes start identical - they will look like one axis"
        );
    }

    #[test]
    fn torso_aim_limit_matches_60_degrees() {
        assert_eq!(torso_aim_offset(30.0), 30.0, "within the clamp, passes through");
        assert_eq!(torso_aim_offset(-30.0), -30.0);
        assert_eq!(torso_aim_offset(90.0), 60.0, "clamped at +60deg");
        assert_eq!(torso_aim_offset(-90.0), -60.0, "clamped at -60deg");
        assert_eq!(torso_aim_offset(60.0), 60.0, "exactly at the boundary passes through");
    }

    /// `capture_input_driver` advances a single cursor while the NEXT
    /// beat's `t` has passed, so a script whose times run backwards
    /// silently fires the out-of-order beat in the same frame as its
    /// predecessor - two beats collapse into one instant and the shot
    /// they were separating is taken at the wrong moment.
    ///
    /// I got this wrong pushing the medic's orbit beats in ahead of its
    /// firing beats, and the failure is quiet: the script still runs,
    /// still writes every PNG, and only the CONTENT is wrong. Cheap to
    /// pin for every script at once, so pin it for every script at once.
    /// §26: G, hold, release must throw. It did not - for anybody, ever
    /// - and the whole suite stayed green through it, because every
    /// throwable test in `sim.rs` hands `PlayerCmd::throw_hold` in
    /// directly and no test at all touched the code that decides it.
    ///
    /// This drives `throw_input` frame by frame with the real button
    /// edges a mouse produces, and asserts the thing a player asserts:
    /// the wind ran, and then a grenade left the hand.
    #[test]
    fn equip_hold_release_actually_throws() {
        let mut s = ThrowState { ready: false, wind: false };
        // G: the grenade is in hand and NOT yet winding
        s.ready = true;
        let (h, ns) = throw_input(s, false, false, false, false);
        s = ns;
        assert!(!h, "an equipped, untouched grenade must not be cooking");

        // press
        let (h, ns) = throw_input(s, true, true, false, false);
        s = ns;
        assert!(h, "the press frame must start the wind - this is the bug");
        // hold
        let mut held = 0;
        for _ in 0..6 {
            let (h, ns) = throw_input(s, true, false, false, false);
            s = ns;
            assert!(h, "the wind must keep running while the trigger is down");
            held += 1;
        }
        assert_eq!(held, 6);
        // release: throw_hold falls, which is the sim's throw edge, and
        // the hand comes up empty
        let (h, ns) = throw_input(s, false, false, true, false);
        s = ns;
        assert!(!h, "releasing must drop throw_hold - that edge IS the throw");
        assert!(!s.ready, "the hand empties on the release");
        assert!(!s.wind, "and the wind latch clears with it");

        // and nothing keeps cooking afterwards
        for _ in 0..4 {
            let (h, ns) = throw_input(s, false, false, false, false);
            s = ns;
            assert!(!h, "an empty hand must not keep winding");
        }
    }

    /// The property the deleted line was protecting: equipping while the
    /// trigger is ALREADY down must not hand you a half-cooked grenade.
    #[test]
    fn equipping_under_a_held_trigger_does_not_start_the_wind() {
        // trigger already down (firing), then G
        let s = ThrowState { ready: true, wind: false };
        let (h, s) = throw_input(s, true, false, false, false);
        assert!(!h, "no fresh press, so no wind");
        assert!(!s.wind);
        // releasing that same held trigger must not throw either
        let (h, s) = throw_input(s, false, false, true, false);
        assert!(!h);
        assert!(s.ready, "the grenade is still in hand - nothing was thrown");
    }

    /// H and Mouse4 are the legacy hold-to-cook and must keep working
    /// with no grenade "readied" at all - three capture scripts use them.
    #[test]
    fn the_legacy_hold_key_still_cooks() {
        let s = ThrowState::default();
        let (h, s) = throw_input(s, false, false, false, true);
        assert!(h, "H alone must cook");
        let (h, _) = throw_input(s, false, false, false, false);
        assert!(!h, "and releasing it must drop the hold");
    }

    #[test]
    fn every_capture_script_runs_forwards_in_time() {
        // Iterates CAPTURE_SCRIPTS rather than a transcribed list. The
        // transcription was already three scripts out of date, which
        // means the newest tables - exactly the ones most likely to have
        // a beat in the wrong order - were the ones going unchecked.
        //
        // `menus`, `frontend` and the splash are the deliberate
        // exemptions: they are wall-clock driven UI walks with no beat
        // table at all, and `init_capture_mode` still accepts them, so an
        // empty script here is correct rather than a typo.
        for name in CAPTURE_SCRIPTS {
            if matches!(name, "menus" | "frontend") || name == branding::CAPTURE_SPLASH_SCRIPT {
                continue;
            }
            let script = capture_script(name);
            assert!(!script.is_empty(), "{name}: script resolved to nothing");
            for w in script.windows(2) {
                assert!(
                    w[1].t >= w[0].t,
                    "{name}: beat at t={} follows t={} - times must not go backwards",
                    w[1].t,
                    w[0].t,
                );
            }
            assert!(
                script.last().map(|b| b.end).unwrap_or(false),
                "{name}: last beat must set `end`, or the capture never exits",
            );
        }
    }

    #[test]
    fn camera_rig_offsets_match_brief_vii_v2_table() {
        assert_eq!(TP_BOOM, 2.2, "hip boom 2.2m");
        assert_eq!(TP_UP, 0.12, "hip up 0.12m");
        assert_eq!(TP_RIGHT, 0.45, "hip right 0.45m");
        assert_eq!(TP_BOOM_SPRINT, 2.5, "sprint boom eases to 2.5m");
        assert_eq!(TP_BOOM_AIM, 1.35, "aim boom 1.35m");
        assert_eq!(TP_RIGHT_AIM, 0.55, "aim right 0.55m");
    }

    /// §2.5's own claim ("this is the ONE spring primitive behind every
    /// secondary-motion element... camera boom k=90") was false when
    /// checked: `damped_spring` had exactly one real call site (the
    /// viewmodel sway, using its own k=196, not any of the five named
    /// constants) plus a test. This is the boom-recovery fix that makes
    /// the claim true for at least this one consumer.
    /// §owner MINIGUN: the streak has to leave the BARRELS.
    ///
    /// `fp_muzzle_local` used to carry the turret's geometry as a
    /// hand-copied `-0.52 - 0.66 * 0.62` behind a comment, and the model
    /// has since been restaged twice. This ties the two together, so the
    /// next person who lengthens a barrel cannot leave the muzzle flash
    /// buried inside the housing.
    ///
    /// Fails on the pre-change code: the old literal put the origin at
    /// -0.929, and the barrels now reach -1.200.
    #[test]
    fn the_first_person_streak_leaves_the_turrets_barrel_tips() {
        let mut s = sim::TdmSim::new(sim::MatchConfig {
            seed: 0xB0A7,
            per_team: 1,
            ..Default::default()
        });
        {
            let f = &mut s.fighters[0];
            f.armor_set = sim::ArmorSet::RobotSuit;
            f.hull = sim::MECH_HULL;
            f.mech_weapon = sim::MechWeapon::Gatling;
        }
        let m = fp_muzzle_local(&s.fighters[0]);
        assert!(
            (m.x - TURRET_VM_CARRY.x).abs() < 1e-6 && (m.y - TURRET_VM_CARRY.y).abs() < 1e-6,
            "the streak must leave the mount's own axis, not the eye: {m:?}"
        );
        let want = TURRET_VM_CARRY.z - TURRET_VM_MUZZLE_Z * TURRET_VM_SCALE;
        assert!(
            (m.z - want).abs() < 1e-6,
            "the streak starts at z {} but the barrels end at {want}",
            m.z
        );
        // ...and it must be IN FRONT of every piece of machinery, or it
        // is a flash inside a box. The breech face is the line.
        let breech = TURRET_VM_CARRY.z - TURRET_VM_BREECH_Z * TURRET_VM_SCALE;
        assert!(
            m.z < breech - 0.3,
            "the muzzle at {} is only {:.3} m in front of the breech face - \
             the barrels are not proud of the housing",
            m.z,
            breech - m.z
        );
        // the autocannon rides this same viewmodel and must agree
        s.fighters[0].mech_weapon = sim::MechWeapon::Autocannon;
        assert_eq!(fp_muzzle_local(&s.fighters[0]), m);
    }

    /// §owner VISUAL RECOIL: the camera's mech kick must be the SIM's
    /// kick, converted — not a table in this file.
    ///
    /// Pins `PUNCH_DEG_S_PER_SPEC_KICK` against `spray_entry`, which is
    /// the function in sim.rs that owns the scale — deliberately NOT
    /// against `turret_kick_per_round`'s derivation, which is being
    /// retuned right now and whose factor list is nobody's business here.
    /// `spray_entry` jitters each round by ±12%, so the bound is that
    /// window and nothing tighter; a changed scale factor blows through
    /// it immediately.
    #[test]
    fn the_mount_kick_conversion_still_agrees_with_the_sim() {
        for k in [sim::GunKind::M4, sim::GunKind::Ak47, sim::GunKind::Deagle] {
            let spec_kick = gun(k).kick;
            for i in 0..8 {
                let ratio = sim::spray_entry(k, i).1 / spec_kick;
                assert!(
                    ratio > PUNCH_DEG_S_PER_SPEC_KICK * 0.85
                        && ratio < PUNCH_DEG_S_PER_SPEC_KICK * 1.15,
                    "sim.rs scales a spec kick to punch by {ratio} for {k:?}[{i}], \
                     but PUNCH_DEG_S_PER_SPEC_KICK says {PUNCH_DEG_S_PER_SPEC_KICK} - \
                     the copy is stale and the mech camera is lying about recoil"
                );
            }
        }
        // and the camera kick is the sim's own per-round punch in
        // GunSpec units: a gatling round under a rifle round, an
        // autocannon shell far over it, both because the sim says so and
        // not because a table here does.
        let gat = mech_mount_camera_kick(sim::MechWeapon::Gatling);
        let can = mech_mount_camera_kick(sim::MechWeapon::Autocannon);
        assert!(gat > 0.0, "a turret round must kick the camera at all");
        assert!(
            (gat - sim::turret_kick_per_round() / PUNCH_DEG_S_PER_SPEC_KICK).abs() < 1e-9,
            "the gatling's camera kick is not the sim's published number"
        );
        // A shell is an EVENT and a turret round is a tick, so the two
        // must not be in the same class - but the margin is deliberately
        // loose (2x, not the 16x their damages differ by), because the
        // sim is free to hand the turret extra FELT kick on top of what
        // its rounds carry, and it currently does. What must never
        // happen is the two converging, or swapping.
        assert!(
            can > gat * 2.0,
            "a {}-damage shell ({can}) kicks like a {}-damage round ({gat})",
            sim::AUTOCANNON_DAMAGE,
            sim::GATLING_DAMAGE
        );
        assert_eq!(mech_mount_camera_kick(sim::MechWeapon::Repair), 0.0);
    }

    /// §owner THE MECH JUMP HAS A LOAD POSE.
    ///
    /// `chassis_kneeling()` is true from the first tick of the coil, so
    /// the pose and the camera both snapped. This is the ramp, and it
    /// fails on the pre-change code for the simple reason that no ramp
    /// existed - `sync_fighters` read `f.crouch`, which a jumping chassis
    /// never sets, so the machine did not bend at all.
    #[test]
    fn the_chassis_bends_its_knees_over_the_sims_own_compression_window() {
        use sim::MechJumpPhase as P;
        // not kneeling: nothing, whatever the phase says
        assert_eq!(chassis_kneel_blend(false, P::Compress, 1.0), 0.0);
        assert_eq!(chassis_kneel_blend(false, P::None, 0.0), 0.0);
        // the coil RAMPS, and monotonically
        let mut prev = -1.0_f32;
        for k in 0..=10 {
            let c = k as f32 / 10.0;
            let b = chassis_kneel_blend(true, P::Compress, c);
            assert!(b > prev, "the load pose is not monotonic at compression {c}");
            prev = b;
        }
        assert_eq!(chassis_kneel_blend(true, P::Compress, 0.0), 0.0, "the first tick of a jump must not be a full squat");
        assert_eq!(chassis_kneel_blend(true, P::Compress, 1.0), 1.0, "launch must happen from a fully folded machine");
        // a held crouch and a landing absorb are NOT ramped - both are
        // already at the pose the moment they begin
        assert_eq!(chassis_kneel_blend(true, P::None, 0.0), 1.0);
        assert_eq!(chassis_kneel_blend(true, P::Recover, 0.0), 1.0);
        // and the camera's lift is the exact complement, so the eye is
        // where the sim says it is at the instant of launch
        assert_eq!(1.0 - chassis_kneel_blend(true, P::Compress, 1.0), 0.0);
    }

    #[test]
    fn boom_recover_converges_without_overshoot() {
        let (mut b, mut v) = (1.0_f32, 0.0_f32);
        let allowed = 2.2_f32;
        for _ in 0..300 {
            let (nb, nv) = boom_recover(b, v, allowed, 1.0 / 120.0);
            assert!(
                nb <= allowed + 1e-3,
                "critically damped: must never overshoot the target ({nb} > {allowed})"
            );
            b = nb;
            v = nv;
        }
        assert!((b - allowed).abs() < 1e-3, "must converge to the allowed distance, got {b}");
    }

    #[test]
    fn boom_recover_moves_meaningfully_within_100ms_at_k90() {
        let (b, _) = boom_recover(1.0, 0.0, 2.2, 0.1);
        assert!(
            b > 1.2 && b < 2.2,
            "k=90 should move well past the start but not fully settle in 100ms, got {b}"
        );
    }

    /// Task 3 rule 5: the landing must actually REBOUND - the camera has
    /// to cross above neutral, not merely decay toward it more slowly.
    /// The previous shape (dip and rebound decaying independently, the
    /// rebound both smaller and faster) made this provably impossible.
    #[test]
    fn landing_rebound_actually_lifts_the_camera_past_neutral() {
        // a real 10 m/s impact, using the same scalings camera_system uses
        let dip_amp = ((10.0_f32 - 3.0) * 0.016).min(0.15);
        let reb_amp = landing_rebound_vy(-10.0) * 0.05;

        assert!(
            landing_offset(dip_amp, reb_amp, 0.0) > 0.0,
            "the frame of impact must be a DIP (positive = camera pushed down)"
        );

        let mut min_offset = f32::INFINITY;
        let mut t_min = 0.0_f32;
        for i in 0..600 {
            let t = i as f32 * 0.002;
            let o = landing_offset(dip_amp, reb_amp, t);
            if o < min_offset {
                min_offset = o;
                t_min = t;
            }
        }
        assert!(
            min_offset < -1e-4,
            "the rebound must carry the camera ABOVE neutral, best was {min_offset} at {t_min}s"
        );
        assert!(
            t_min > 0.05,
            "the rebound is a delayed counter-push, not simultaneous with the dip (peaked at {t_min}s)"
        );
        assert!(
            landing_offset(dip_amp, reb_amp, 1.0).abs() < 1e-3,
            "and it must settle back to neutral"
        );
    }

    /// The k=90 spring must govern ONLY collision recovery. Applying it
    /// to every boom increase also filtered the sprint ease, the ADS
    /// blend, and plain mouse-look - two filters in series, heavier one
    /// wins. These pin the three cases apart.
    #[test]
    fn boom_step_tracks_free_space_directly_and_springs_only_out_of_occlusion() {
        let dt = 1.0 / 120.0;

        // free space, no cover hit, target grows: must track it EXACTLY.
        // This is the case the old distance-only heuristic got wrong - it
        // looked identical to a collision recovery and got sprung.
        let (b, v, occ) = boom_step(2.20, 0.0, false, 2.50, 2.50, false, dt);
        assert!((b - 2.50).abs() < 1e-6, "free-space growth must track directly, got {b}");
        assert_eq!(v, 0.0, "no spring velocity should accumulate in free space");
        assert!(!occ, "nothing was hit, so nothing is occluded");

        // contact: pull in immediately
        let (b, v, occ) = boom_step(2.50, 0.0, false, 1.10, 2.50, true, dt);
        assert!((b - 1.10).abs() < 1e-6, "pull-in must be instant, got {b}");
        assert_eq!(v, 0.0);
        assert!(occ, "a ray hit means occluded");

        // cleared the corner (no hit this frame, but we WERE occluded):
        // spring back out rather than popping
        let (b, _, occ) = boom_step(1.10, 0.0, true, 2.50, 2.50, false, dt);
        assert!(
            b > 1.10 && b < 2.50,
            "recovery must be sprung, not instant: got {b}"
        );
        assert!(occ, "still recovering, so still flagged occluded");

        // ...and once recovered, the flag clears and it tracks again
        let (b, _, occ) = boom_step(2.50, 0.0, true, 2.50, 2.50, false, dt);
        assert!((b - 2.50).abs() < 1e-6);
        assert!(!occ, "fully recovered: back to free-space tracking");
    }

    #[test]
    fn boom_step_sprint_ease_is_not_re_filtered_by_the_spring() {
        // Simulate the documented sprint boom-out (2.2 -> 2.5 on the
        // 0.12s first-order lag) in FREE SPACE and confirm the boom
        // arrives on the ease's own schedule. Before the fix the spring
        // stretched this to roughly double.
        let dt = 1.0 / 120.0;
        let (mut eased, mut boom, mut vel) = (2.2_f32, 2.2_f32, 0.0_f32);
        let mut t = 0.0_f32;
        let mut t_90 = None;
        while t < 1.0 {
            eased += (2.5 - eased) * (dt / 0.12_f32).min(1.0);
            // no cover anywhere: allowed == free_len, hit == false
            let (nb, nv, _) = boom_step(boom, vel, false, eased, eased, false, dt);
            boom = nb;
            vel = nv;
            t += dt;
            if t_90.is_none() && boom >= 2.2 + 0.9 * (2.5 - 2.2) {
                t_90 = Some(t);
            }
        }
        let t90 = t_90.expect("boom must reach 90% of the sprint ease within 1s");
        assert!(
            t90 < 0.35,
            "sprint boom-out should follow the 0.12s ease (~0.25s to 90%), took {t90}s"
        );
    }
}

/// The look-pitch clamp: recoil and the mouse write the same state, so
/// they must agree about how far it may go.
#[cfg(test)]
mod recoil_pitch_tests {
    use crate::*;

    /// The bug this module exists for.
    ///
    /// Recoil clamps the ACCUMULATED pitch, not its own delta. While its
    /// limit was (-0.7, 0.8) and the mouse's was ±1.53, a player holding
    /// a steep aim had it yanked back to the recoil limit the instant
    /// they fired — up to 0.73 rad in one frame, from one bullet, with no
    /// input. Firing must never move the aim further than the kick.
    #[test]
    fn firing_at_a_steep_aim_does_not_teleport_the_view() {
        let kick = gun(sim::GunKind::Ak47).kick;
        for &pitch in &[
            -LOOK_PITCH_LIMIT,
            -1.2,
            -0.71, // just outside the OLD lower clamp
            0.0,
            0.81, // just outside the OLD upper clamp
            1.2,
            LOOK_PITCH_LIMIT,
        ] {
            let after = recoil_kicked_pitch(pitch, kick, 0.0, 1.0);
            let moved = (after - pitch).abs();
            let most = kick * 6.0 + 1e-6;
            assert!(
                moved <= most,
                "pitch {pitch} moved {moved} rad on one shot; the kick is only {most}"
            );
        }
    }

    /// ...and the guard above must not be satisfied by doing nothing.
    #[test]
    fn the_kick_still_kicks() {
        let kick = gun(sim::GunKind::Ak47).kick;
        let after = recoil_kicked_pitch(0.0, kick, 0.0, 1.0);
        assert!(after < 0.0, "recoil must raise the muzzle (lower pitch)");
        assert!(
            (after.abs() - kick * 6.0).abs() < 1e-6,
            "one shot should move exactly kick*6, got {after}"
        );
    }

    /// Recoil may push the aim TO the ceiling, never through it.
    #[test]
    fn sustained_fire_stops_at_the_same_limit_the_mouse_obeys() {
        let kick = gun(sim::GunKind::M249).kick;
        let mut pitch = 0.0;
        for _ in 0..500 {
            pitch = recoil_kicked_pitch(pitch, kick, 0.05, 1.0);
        }
        assert!(
            pitch >= -LOOK_PITCH_LIMIT,
            "ran past the look limit: {pitch}"
        );
        assert!(
            (pitch + LOOK_PITCH_LIMIT).abs() < 1e-3,
            "500 rounds should pin the aim at the ceiling, got {pitch}"
        );
    }

    /// A brace is a discount on the kick, in both directions of the
    /// ladder — the sim damps a braced mech's punch, so the view must
    /// damp too or the stance does nothing the player can feel.
    #[test]
    fn bracing_reduces_the_kick_by_the_sims_own_factor() {
        let kick = 0.018; // the autocannon's camera kick
        let unbraced = recoil_kicked_pitch(0.0, kick, 0.0, 1.0).abs();
        let braced =
            recoil_kicked_pitch(0.0, kick, 0.0, sim::MECH_BRACE_RECOIL_DAMP).abs();
        assert!(braced < unbraced, "bracing must help: {braced} vs {unbraced}");
        assert!(
            (braced - unbraced * sim::MECH_BRACE_RECOIL_DAMP).abs() < 1e-6,
            "the view should scale by the sim's own damp factor"
        );
        // and leaning on foot is the milder discount
        let leaned = recoil_kicked_pitch(0.0, kick, 0.0, sim::LEAN_RECOIL_MULT).abs();
        assert!(
            leaned > braced && leaned < unbraced,
            "lean sits between braced and unbraced: {braced} < {leaned} < {unbraced}"
        );
    }

    /// Bloom widens the kick, so a hot gun climbs faster than a cold one.
    #[test]
    fn a_hot_gun_climbs_faster_than_a_cold_one() {
        let kick = gun(sim::GunKind::Ak47).kick;
        let cold = recoil_kicked_pitch(0.0, kick, 0.0, 1.0).abs();
        let hot = recoil_kicked_pitch(0.0, kick, 0.05, 1.0).abs();
        assert!(hot > cold, "bloom should add climb: {hot} vs {cold}");
    }
}

/// The bowstring must report the draw the SIM is running, not a guess
/// assembled from the ADS toggle.
#[cfg(test)]
mod bow_draw_visual_tests {
    use crate::*;

    const PERIOD: f32 = 0.95; // gun(Bow).fire_period

    /// The split brain this replaces: the pull was `1.0` while aiming and
    /// `0.25` otherwise, so the whole 0.15s..0.7s curve the sim runs was
    /// invisible - two positions standing in for a continuous draw.
    #[test]
    fn the_players_pull_tracks_the_sims_clock() {
        let at = |t: f32| bow_draw_visual(t, 0.0, PERIOD, true);
        assert_eq!(at(0.0), 0.0, "an untouched bow is slack");
        assert!(at(sim::BOW_DRAW_FULL_S) >= 1.0 - 1e-6, "0.7s is full draw");
        assert_eq!(at(5.0), 1.0, "holding past full stays full, never past it");

        // strictly increasing across the whole draw - a continuous pull,
        // which is exactly what the two-position version could not show
        let mut prev = -1.0;
        for i in 0..=70 {
            let v = at(i as f32 * 0.01);
            assert!(v >= prev, "pull went backwards at t={}", i as f32 * 0.01);
            prev = v;
        }
        // and it is genuinely partway at the halfway mark, not snapped
        let mid = at(sim::BOW_DRAW_FULL_S * 0.5);
        assert!(
            (0.4..0.6).contains(&mid),
            "half a draw should look half drawn, got {mid}"
        );
    }

    /// Bots never run `step_bow_draw`, so their clock is pinned at 0.
    /// Reading it directly would leave every bot bow permanently slack -
    /// a regression from the fixed 0.6 this replaced.
    #[test]
    fn a_bot_bow_still_draws_even_though_its_clock_never_runs() {
        let bot = |cd: f32| bow_draw_visual(0.0, cd, PERIOD, true.eq(&false));
        assert_eq!(bot(PERIOD), 0.0, "just loosed: string forward");
        assert!(bot(PERIOD * 0.5) > 0.4, "mid-cadence: drawing");
        assert_eq!(bot(0.0), 1.0, "about to loose: fully drawn");

        // the naive version of this fix - reading bow_draw_t for everyone
        let naive = bow_draw_visual(0.0, PERIOD * 0.5, PERIOD, true);
        assert_eq!(naive, 0.0, "which is why bots must not use that path");
    }

    /// A cadence-derived pull is meaningless without a period.
    #[test]
    fn a_zero_period_cannot_divide_by_zero() {
        assert_eq!(bow_draw_visual(0.0, 0.0, 0.0, false), 0.0);
    }

    /// Whatever the input, the string is a 0..1 quantity - the renderer
    /// multiplies anchor offsets by it.
    #[test]
    fn the_pull_is_always_a_unit_fraction() {
        for &(t, cd, player) in &[
            (-1.0, -1.0, true),
            (99.0, 99.0, true),
            (-1.0, -1.0, false),
            (99.0, 99.0, false),
        ] {
            let v = bow_draw_visual(t, cd, PERIOD, player);
            assert!((0.0..=1.0).contains(&v), "out of range: {v}");
        }
    }
}

/// The bow's string, arrow and draw hand, which must agree by
/// construction because for a long time they did not.
#[cfg(test)]
mod bow_string_tests {
    use crate::*;

    /// Sample draws across the full range, ends included.
    const DRAWS: [f32; 6] = [0.0, 0.2, 0.45, 0.7, 0.9, 1.0];

    /// §owner ANCHORS: the published hand points are where the bow
    /// actually is, and the live ones actually live.
    ///
    /// Mutation-proved by construction rather than by transcription:
    /// nothing here re-types a coordinate, it asserts the RELATIONS a
    /// hand system depends on. Weld `BowNock` to a constant and the
    /// travel assert fails; drop `BOW_HAND_OFF` from `BowDrawHand` and
    /// the "behind the string" assert fails.
    #[test]
    fn the_published_anchors_track_the_draw() {
        use weapon_anchors::AnchorKind as A;
        // the grip does NOT move with the draw - it is the bow hand
        let g0 = weapon_anchor_local(A::BowGrip, 0.0).translation;
        let g1 = weapon_anchor_local(A::BowGrip, 1.0).translation;
        assert_eq!(g0, g1, "the bow hand cannot travel with the string");
        // the nock does, by the full pull, monotonically, and BACKWARD
        let mut prev = weapon_anchor_local(A::BowNock, 0.0).translation.z;
        for d in DRAWS.iter().skip(1) {
            let z = weapon_anchor_local(A::BowNock, *d).translation.z;
            assert!(z <= prev, "the nock anchor moved FORWARD at draw {d}");
            prev = z;
        }
        let travel = weapon_anchor_local(A::BowNock, 0.0).translation.z - prev;
        assert!(
            (travel - BOW_DRAW_PULL).abs() < 1e-6,
            "the nock anchor travelled {travel} over a full draw, the string \
             travels {BOW_DRAW_PULL}"
        );
        // and the anchor is ON the string, at every draw
        for d in DRAWS {
            assert_eq!(
                weapon_anchor_local(A::BowNock, d).translation,
                bow_nock_local(d),
                "the anchor left the string at draw {d}"
            );
        }
        // the DRAW HAND hooks it from behind - never occupying it
        for d in DRAWS {
            let nock = weapon_anchor_local(A::BowNock, d).translation;
            let hand = weapon_anchor_local(A::BowDrawHand, d).translation;
            assert!(
                hand.z < nock.z,
                "the draw hand is in FRONT of the string at draw {d}"
            );
            assert!(
                (hand - nock).length() > 0.02,
                "the draw hand is sitting inside the string at draw {d}"
            );
        }
        // the spear's grip is on the shaft's centre run, not out on the
        // blade or off the butt
        let s = weapon_anchor_local(A::SpearGrip, 0.0).translation;
        let prof = spear_profile();
        let shaft = &prof[1];
        assert!(
            (s.z - shaft.z).abs() <= shaft.len * 0.5,
            "the spear grip at z {:.3} is not inside the hand swell",
            s.z
        );
    }

    /// The two halves meet AT the nock, and their far ends stay pinned to
    /// the limb tips.
    ///
    /// This is the whole geometric claim of a bowstring and it is the one
    /// the single tip-to-tip box could not make: it had no vertex at the
    /// nock, so there was nothing to pull and nothing to check.
    #[test]
    fn both_string_halves_run_from_a_limb_tip_to_the_nock() {
        for d in DRAWS {
            let nock = bow_nock_local(d);
            for side in [-1.0_f32, 1.0] {
                let t = bow_string_half(side, d);
                // reconstruct the segment from the transform alone - if
                // the rotation and the length disagree this fails
                let half = t.rotation * Vec3::X * (t.scale.x * 0.5);
                let (a, b) = (t.translation - half, t.translation + half);
                let tip = Vec3::new(0.0, side * BOW_TIP_Y, BOW_TIP_Z);
                // whichever end is nearer the tip must BE the tip, and the
                // other must be the nock
                let (near_tip, near_nock) =
                    if (a - tip).length() < (b - tip).length() { (a, b) } else { (b, a) };
                assert!(
                    (near_tip - tip).length() < 1e-4,
                    "side {side} draw {d}: string leaves {near_tip:?}, tip is {tip:?}"
                );
                assert!(
                    (near_nock - nock).length() < 1e-4,
                    "side {side} draw {d}: string ends {near_nock:?}, nock is {nock:?}"
                );
            }
        }
    }

    /// Drawing makes the string LONGER, monotonically. A V is two sides of
    /// a triangle and both grow with the pull; a version that shortened
    /// would mean the halves were being rotated without being re-measured.
    #[test]
    fn the_string_lengthens_as_it_is_drawn() {
        let len = |d: f32| bow_string_half(1.0, d).scale.x;
        let rest = len(0.0);
        assert!(rest > BOW_TIP_Y, "a slack string still spans the limb");
        let mut prev = rest;
        for d in [0.2_f32, 0.45, 0.7, 0.9, 1.0] {
            let l = len(d);
            assert!(l > prev, "draw {d}: {l} did not exceed {prev}");
            prev = l;
        }
        // and the nock really travels the full pull
        assert!(
            (bow_nock_local(1.0).z - (BOW_STRING_Z - BOW_DRAW_PULL)).abs() < 1e-6,
            "full draw must be exactly BOW_DRAW_PULL back"
        );
    }

    /// The arrow's NOCK sits on the string at every draw.
    ///
    /// The failure this pins is the one that shipped: the arrow hung off
    /// the bow HAND at a fixed offset, so it tracked a hand rather than a
    /// bow and never moved with the draw at all. Here the tail is computed
    /// from the arrow transform the same way the renderer will, so an
    /// arrow that floats off the cord fails.
    #[test]
    fn the_nocked_arrow_keeps_its_tail_on_the_string() {
        for d in DRAWS {
            let t = bow_nocked_arrow(d);
            let tail_z = t.translation.z + ARROW_NOCK_Z * t.scale.z;
            let nock = bow_nock_local(d);
            assert!(
                (tail_z - nock.z).abs() < 1e-5,
                "draw {d}: tail at z {tail_z}, string at z {nock:?}"
            );
            // and it points DOWNRANGE - +Z, never reversed
            assert!(
                t.translation.z > tail_z,
                "draw {d}: the arrow is facing backwards"
            );
        }
    }

    /// The shaft clears the riser it runs beside.
    ///
    /// Held horizontal there is no "on top of the riser" for an arrow to
    /// rest on, which is exactly what the old vertical-bow shelf assumed.
    /// The riser is 0.052 wide, so its face is at x 0.026.
    #[test]
    fn the_arrow_runs_clear_of_the_riser() {
        const RISER_HALF_W: f32 = 0.026;
        // the shaft is 0.020 across in the arrow's unit envelope, so it
        // scales with the nocked length like every other part of it
        let shaft_r = 0.020 * 0.5 * BOW_ARROW_LEN;
        assert!(
            BOW_ARROW_X - shaft_r > RISER_HALF_W,
            "the shaft at x {BOW_ARROW_X} (r {shaft_r}) cuts through a riser \
             half-width {RISER_HALF_W}"
        );
        // and the ADS arrow rest must be UNDER it, not above: the rest is
        // a side bracket at y -0.016 with a 0.018 box, top face -0.007
        assert!(
            -0.007 < 0.0 && -0.016 + 0.018 * 0.5 > -shaft_r - 0.004,
            "the rest has to actually meet the shaft it supports"
        );
    }

    /// The draw HAND is on the string - the anchor restage itself.
    ///
    /// What this replaces was a y lift of 0.14·draw, which was correct for
    /// a VERTICAL bow (whose string spans Y, so any height is still on the
    /// cord) and wrong the moment the limbs turned sideways: the hand rode
    /// a palm's width above a horizontal string, pulling nothing. The
    /// hand's offset from the nock must therefore be CONSTANT - it cannot
    /// grow with the draw, because fingers do not drift off a string they
    /// are holding.
    #[test]
    fn the_draw_hand_holds_the_string_at_every_draw() {
        for d in DRAWS {
            let hand = bow_nock_local(d) + BOW_HAND_OFF;
            let off = hand - bow_nock_local(d);
            assert!(
                (off - BOW_HAND_OFF).length() < 1e-6,
                "draw {d}: the hand drifted to {off:?}"
            );
            // within a hand's reach of the cord, in the cord's own plane
            assert!(off.length() < 0.06, "draw {d}: {off:?} is not a grip");
            assert!(
                off.y.abs() < 1e-6,
                "draw {d}: the hand left the string's plane by {}",
                off.y
            );
        }
    }
}

/// §B.6 (Brief VIII-B): the 20-segment body's own completion gate.
///
/// The data half. Three of the brief's five tests need no new bone -
/// segment count, mass closure, and proportions - which is exactly why
/// the table lands before the rig surgery does.
#[cfg(test)]
mod segment_tests {
    use crate::*;

    /// §B.6 segment-count test: all 20 named segments, each once.
    #[test]
    fn the_body_exposes_all_twenty_segments() {
        assert_eq!(SEGMENTS.len(), N_SEGMENTS);
        assert_eq!(N_SEGMENTS, 20);
        for s in SEGMENTS {
            assert_eq!(
                SEGMENTS.iter().filter(|q| **q == s).count(),
                1,
                "{s:?} is listed twice"
            );
        }
        // and the composition is the brief's: one head, three trunk, and
        // eight mirrored pairs
        let singles = SEGMENTS
            .iter()
            .filter(|s| {
                matches!(
                    s,
                    Segment::HeadNeck | Segment::Thorax | Segment::Lumbar | Segment::Pelvis
                )
            })
            .count();
        assert_eq!(singles, 4, "head plus a THREE-part trunk");
        assert_eq!((N_SEGMENTS - singles) % 2, 0, "everything else is a pair");
        assert_eq!((N_SEGMENTS - singles) / 2, 8, "eight mirrored pairs");
    }

    /// §B.6 mass-closure test: the whole body sums to 1.000 ± 0.001.
    ///
    /// This is the test that catches the trap in §B.3's own wording. The
    /// clavicle is "~0.005 (carve from thorax)" - CARVE, not add. Two
    /// clavicles bolted on beside a full-weight thorax put the body at
    /// 1.010, a 1% error that would never show up as anything but a
    /// vaguely wrong-feeling ragdoll.
    #[test]
    fn the_segment_masses_close_to_one_whole_body() {
        let total: f32 = SEGMENTS.into_iter().map(|s| segment_data(s).mass_frac).sum();
        assert!(
            (total - 1.0).abs() < 0.001,
            "body mass sums to {total}, not 1.000 - the clavicles are carved \
             FROM the thorax, not added beside it"
        );
        // the brief's own arithmetic, checked limb group by limb group
        let group = |f: fn(Segment) -> bool| -> f32 {
            SEGMENTS
                .into_iter()
                .filter(|s| f(*s))
                .map(|s| segment_data(s).mass_frac)
                .sum()
        };
        let trunk = group(|s| {
            matches!(
                s,
                Segment::Thorax
                    | Segment::Lumbar
                    | Segment::Pelvis
                    | Segment::ClavicleL
                    | Segment::ClavicleR
            )
        });
        assert!((trunk - 0.497).abs() < 0.001, "trunk is {trunk}, brief says 0.497");
        let one_arm = segment_data(Segment::UpperArmL).mass_frac
            + segment_data(Segment::ForearmL).mass_frac
            + segment_data(Segment::HandL).mass_frac;
        assert!((one_arm - 0.050).abs() < 0.001, "an arm is {one_arm}, brief says 0.050");
        let one_leg = segment_data(Segment::ThighL).mass_frac
            + segment_data(Segment::ShankL).mass_frac
            + segment_data(Segment::FootL).mass_frac
            + segment_data(Segment::ToeL).mass_frac;
        assert!((one_leg - 0.161).abs() < 0.001, "a leg is {one_leg}, brief says 0.161");
        // and no segment is weightless - a zero-mass segment would get a
        // zero-stiffness spring and hang limp
        for s in SEGMENTS {
            assert!(segment_data(s).mass_frac > 0.0, "{s:?} has no mass");
        }
    }

    /// A mirrored pair is the SAME segment on two sides - same mass,
    /// same length, same inertia.
    #[test]
    fn a_left_segment_weighs_what_its_right_twin_does() {
        for (l, r) in [
            (Segment::ClavicleL, Segment::ClavicleR),
            (Segment::UpperArmL, Segment::UpperArmR),
            (Segment::ForearmL, Segment::ForearmR),
            (Segment::HandL, Segment::HandR),
            (Segment::ThighL, Segment::ThighR),
            (Segment::ShankL, Segment::ShankR),
            (Segment::FootL, Segment::FootR),
            (Segment::ToeL, Segment::ToeR),
        ] {
            let (a, b) = (segment_data(l), segment_data(r));
            assert_eq!(a.name, b.name, "{l:?} and {r:?} are the same segment");
            assert_eq!(a.mass_frac, b.mass_frac);
            assert_eq!(a.len_frac, b.len_frac);
            assert_eq!(segment_inertia(l), segment_inertia(r));
        }
    }

    /// §B.6 proportion test: the published lengths, at the brief's own
    /// worked height.
    ///
    /// "At H = 1.8m: upper arm 33cm, forearm 26cm, thigh 44cm, shank
    /// 44cm, foot 27cm." Those five numbers are the brief checking its
    /// own table, so they are the right thing to check ours against - a
    /// fraction transcribed one digit wrong survives inspection and fails
    /// here.
    #[test]
    fn the_published_lengths_land_where_the_brief_says_they_do() {
        let cm = |s: Segment| segment_data(s).len_frac * RIG_HEIGHT_M * 100.0;
        for (s, want) in [
            (Segment::UpperArmL, 33.0_f32),
            (Segment::ForearmL, 26.0),
            (Segment::ThighL, 44.0),
            (Segment::ShankL, 44.0),
        ] {
            let got = cm(s);
            assert!(
                (got - want).abs() < 1.0,
                "{s:?} is {got:.1} cm at H={RIG_HEIGHT_M}, brief says {want}"
            );
        }
        // the foot is split hindfoot/toe, so the brief's 27 cm is the SUM
        let foot = cm(Segment::FootL) + cm(Segment::ToeL);
        assert!(
            (foot - 27.0).abs() < 1.0,
            "hindfoot + toe is {foot:.1} cm, brief says 27"
        );
        // §B.4's other three proportions are stated, not derived
        assert!((SHOULDER_WIDTH_FRAC * RIG_HEIGHT_M - 0.466).abs() < 0.01);
        assert!(SHOULDER_WIDTH_FRAC > HIP_WIDTH_FRAC, "shoulders are wider than hips");
        assert!(
            SHOULDER_HEIGHT_FRAC < 1.0,
            "the shoulders are below the top of the head"
        );
    }

    /// §owner AGILE SUPPORT MECH: the repair beam has to read as
    /// TRANSFER, not as a laser.
    ///
    /// A static glowing line says a connection exists. Packets moving
    /// along it say something is being carried, and only the second
    /// reads as repair - which matters, because a beam pointed at a
    /// teammate that looks like a weapon is a beam that will make people
    /// dodge their own medic.
    #[test]
    /// §owner The armour trims must actually differ, and must differ in
    /// ONE direction.
    ///
    /// The failure this catches is not a crash, it is a lineup where two
    /// trims render identically - which looks exactly like a working
    /// variant system to anyone reading the code, and like a bug to
    /// anyone looking at the screen. Both dials are therefore checked
    /// for strict monotonicity across `ALL`, which also fixes the
    /// order: heavier trims can never wear LESS.
    #[test]
    fn every_armour_trim_is_visibly_a_different_machine() {
        let all = MechTrim::ALL;
        for w in all.windows(2) {
            let (a, b) = (w[0], w[1]);
            assert!(
                b.limb_scale() > a.limb_scale(),
                "{b:?} does not wrap more shell than {a:?} - two trims                  that render the same are not two trims"
            );
            assert!(
                b.plates() > a.plates(),
                "{b:?} wears no more plates than {a:?}; coverage has to                  grow with the trim or the ordering means nothing"
            );
        }
        // the count is an index into TrimPiece's four - one past the end
        // would silently drop the last plate rather than fail
        assert_eq!(
            all[all.len() - 1].plates(),
            4,
            "the heaviest trim must wear every optional plate"
        );
        assert_eq!(all[0].plates(), 0, "Stripped is the bare frame");
        // and the bare frame still has to be a machine, not a wireframe
        assert!(
            all[0].limb_scale() > 0.5,
            "a trim that thin is a skeleton, not a stripped chassis"
        );
    }

    #[test]
    fn the_repair_beam_reads_as_energy_going_somewhere() {
        assert!(
            REPAIR_PACKETS >= 3,
            "one or two packets read as a flicker, not a flow"
        );
        assert!(
            REPAIR_SEGMENTS > REPAIR_PACKETS,
            "the shaft must be finer than the packets travelling it, or \
             the beam looks like a string of beads"
        );
        // enough segments to bend toward a moving target without the
        // seams showing
        assert!(REPAIR_SEGMENTS >= 8, "{REPAIR_SEGMENTS} segments will crease");
        // and the beam must actually out-range a fight - a support verb
        // that requires standing next to the man being shot at is one
        // nobody survives using
        assert!(
            sim::REPAIR_RANGE_M > 15.0,
            "a {:.0} m beam is a melee ability",
            sim::REPAIR_RANGE_M
        );
        // it must mend slower than a rifle removes: support extends a
        // fight, it does not decide one
        assert!(
            sim::REPAIR_PER_S < 100.0,
            "{} hull/s outheals a squad and makes the heavy immortal",
            sim::REPAIR_PER_S
        );
    }

    /// §owner: the weapon strip shows the mounts a chassis ACTUALLY has.
    ///
    /// It showed "TURRET / ROCKETS" inside a Mechanical Medic - two
    /// mounts it does not carry, both displaying a round count it never
    /// uses, both in the empty-magazine danger colour.
    /// `MechWeapon::for_set` had existed since the chassis landed and
    /// nothing asked it the question, which is the most ordinary way a
    /// feature ends up half-built: the data was right and no caller
    /// looked at it.
    #[test]
    fn the_weapon_strip_lists_the_chassis_you_are_actually_in() {
        let heavy = sim::MechWeapon::for_set(sim::ArmorSet::RobotSuit);
        let medic = sim::MechWeapon::for_set(sim::ArmorSet::ScoutMech);
        // both chassis fill the same two strip slots, so the UI needs no
        // per-chassis layout - only per-chassis CONTENT
        assert_eq!(heavy.len(), 2);
        assert_eq!(medic.len(), 2);
        // and they share nothing, so a strip built from the wrong list
        // is wrong in every cell rather than subtly off in one
        assert!(medic.iter().all(|w| !heavy.contains(w)));
        // the mounts that count rounds and the ones that count heat are
        // disjoint - which is why the corner readout branches on the
        // mount and not on the chassis
        for w in medic {
            assert!(
                matches!(w, sim::MechWeapon::Plasma | sim::MechWeapon::Repair),
                "{w:?} has a magazine and the medic has none"
            );
        }
    }

    /// §owner MECH BARRIER: the two halves of "transparent to me, a wall
    /// of light to you" are actually opposed, and the numbers have to
    /// keep them that way.
    ///
    /// A single translucent sheet cannot satisfy both readings, which is
    /// why the field is a near-invisible FILL plus a bright LATTICE. If
    /// the fill ever creeps up toward the lattice's opacity the pilot
    /// starts fighting through frosted glass, and if the lattice ever
    /// dims toward the fill the enemy stops seeing a shield at all.
    #[test]
    fn the_barrier_is_a_window_to_the_pilot_and_a_wall_to_the_enemy() {
        // the fill has to be nearly clear - this is the number the
        // pilot's visibility depends on
        const FILL_A: f32 = 0.085;
        const EDGE_A: f32 = 0.60;
        assert!(FILL_A < 0.12, "a pilot cannot fight through {FILL_A} alpha");
        assert!(
            EDGE_A > FILL_A * 5.0,
            "the lattice must dominate the fill by a wide margin, or the \
             barrier reads as a smudge from both sides"
        );
        // §owner SHIELD PASS: it has to actually COVER the machine
        // holding it. The mech is ~3 m and cannot crouch, so a barrier
        // sized for a torso leaves the legs and pauldrons in the open -
        // and "get lower behind it" is not available to a chassis.
        let span = 1.70 * BARRIER_SCALE;
        assert!(
            span > 2.4,
            "a {span:.2} m field does not cover a 3 m mech that cannot duck"
        );
        // but not so wide it becomes a wall. Flanking is the counter;
        // out-ranging it must not have to be.
        assert!(span < 3.4, "a {span:.2} m field is a building, not a shield");

        // and the deploy has to be combat-fast. A barrier you have to
        // raise half a second early is one you die behind.
        assert!(
            BARRIER_DEPLOY_S <= 0.25,
            "{BARRIER_DEPLOY_S}s is too slow to be a reaction"
        );
        assert!(BARRIER_DEPLOY_S > 0.0, "an instant deploy has no read at all");
        // the petals must actually open far enough to frame a field
        assert!(
            (30.0..90.0).contains(&BARRIER_PETAL_DEG),
            "{BARRIER_PETAL_DEG} deg does not read as a fold"
        );
    }

    /// §B.6 toe-off test: "assert the toe segment rotates through its
    /// plantar-flexion range at contact-exit - no toe rotation means the
    /// run is still a glide."
    ///
    /// It WAS a glide. There was nothing forward of the ankle, so the
    /// foot left the ground as a flat plate and the whole sprint pushed
    /// off nothing. This is the test that proves segments #19-20 landed.
    #[test]
    fn the_sprint_actually_pushes_off_its_toes() {
        // sweep a full cycle at sprint amplitude
        let mut peak = 0.0_f32;
        let mut peak_at = 0.0_f32;
        let n = 720;
        for i in 0..n {
            let ph = i as f32 / n as f32 * std::f32::consts::TAU;
            let a = toe_off_angle(ph, 1.0);
            assert!(a >= 0.0, "a toe pushes, it never pulls: {a} at {ph}");
            assert!(a <= TOE_OFF_MAX + 1e-6, "hyperextended to {a}");
            if a > peak {
                peak = a;
                peak_at = ph;
            }
        }
        assert!(
            (peak - TOE_OFF_MAX).abs() < 1e-3,
            "the toe must reach its full range somewhere in the cycle, got \
             {:.1} of {:.1} degrees",
            peak.to_degrees(),
            TOE_OFF_MAX.to_degrees()
        );
        // ...and reach it at CONTACT EXIT - the back of the stance, a
        // quarter-cycle after the leg's rearmost point, not at mid-swing
        assert!(
            (peak_at - PI).abs() < 0.2,
            "toe-off peaks at phase {peak_at:.2}, expected the back of \
             stance near {PI:.2}"
        );
        // a STANDING fighter has flat feet. A toe that stayed cocked at a
        // standstill would be the no-bounce contract broken in a new place
        for i in 0..64 {
            let ph = i as f32 * 0.31;
            assert_eq!(toe_off_angle(ph, 0.0), 0.0, "a standing toe must be flat");
        }
        // and a WALK gets a roll where a sprint gets a snap
        let walk = (0..n)
            .map(|i| toe_off_angle(i as f32 / n as f32 * std::f32::consts::TAU, 0.25))
            .fold(0.0_f32, f32::max);
        assert!(
            walk < peak * 0.5,
            "a walk must not toe off like a sprint: {walk} vs {peak}"
        );
    }

    /// §B.2: inserting the lumbar must not MOVE anything.
    ///
    /// This test exists because its absence cost a broken build. The
    /// first trunk split left `WAIST_Y` on the thorax while its new
    /// parent, the lumbar, also carried it - so the upper body sat a full
    /// waist above the legs and the soldier came apart in mid-air. Every
    /// rig test in this file passed: they measure ANGLES (separation, the
    /// kinetic chain, the trunk twist above) or the head BAND, which is
    /// derived from `gait_pose` rather than read back off the transform
    /// hierarchy. Nothing was watching where the torso actually WAS.
    ///
    /// The claim is composition: lumbar + thorax must land exactly where
    /// the single trunk segment used to.
    #[test]
    fn thorax_height_is_conserved_across_the_trunk_split() {
        for hip_y in [0.50_f32, 0.63, 0.71] {
            for crouch in [0.0_f32, 0.12] {
                for breath in [-0.004_f32, 0.0, 0.004] {
                    // what the SINGLE trunk segment used to be set to, in
                    // root space - the expression this replaced, verbatim
                    let before = hip_y - crouch + breath;
                    // and what the two segments now compose to
                    let after = WAIST_Y + thorax_local_y(hip_y, crouch, breath);
                    assert!(
                        (after - before).abs() < 1e-6,
                        "the trunk moved: {before} -> {after} \
                         (hip {hip_y} crouch {crouch} breath {breath})"
                    );
                }
            }
        }
        // and the waist really is where the legs are hung from, or the
        // subtraction is against the wrong number
        assert!(
            (WAIST_Y - 0.63).abs() < 1e-6,
            "WAIST_Y must match the height the thighs spawn at"
        );
        // a standing fighter's thorax sits AT the waist - local zero
        assert!(
            thorax_local_y(WAIST_Y, 0.0, 0.0).abs() < 1e-6,
            "an unposed thorax must sit exactly on its own parent"
        );
    }

    /// §B.2: the trunk twist is SHARED between lumbar and thorax, and the
    /// two still sum to exactly what one joint used to carry.
    ///
    /// The sum is the load-bearing half. Hip-shoulder separation and the
    /// §6.2 ±60° additive-aim contract are both stated against the TOTAL
    /// trunk yaw, so splitting it must not change that total - only where
    /// along the spine it happens.
    #[test]
    fn the_trunk_twist_is_shared_but_conserved() {
        assert!(
            LUMBAR_TWIST_SHARE > 0.0,
            "a lumbar with no share is the hinge this replaced"
        );
        assert!(
            LUMBAR_TWIST_SHARE < 0.5,
            "the thoracic spine out-rotates the lumbar - an even split \
             reads as a body hinged at the belt"
        );
        for yaw in [-1.2_f32, -0.4, 0.0, 0.3, 0.9] {
            let lumbar = yaw * LUMBAR_TWIST_SHARE;
            let thorax = yaw * (1.0 - LUMBAR_TWIST_SHARE);
            assert!(
                (lumbar + thorax - yaw).abs() < 1e-6,
                "the split changed the total at {yaw}"
            );
            // and they twist the SAME way - opposed shares would be a
            // counter-rotation nobody asked for
            if yaw != 0.0 {
                assert_eq!(lumbar.signum(), thorax.signum());
            }
        }
    }

    /// §B.5: stiffness comes OUT of the mass model, and it orders the
    /// segments the way physical intuition does.
    ///
    /// The claim being made is not that any particular number is right -
    /// it is that the numbers are no longer independent. A thigh is
    /// heavier and longer than a forearm, so at one shared frequency it
    /// must come back stiffer, and nobody has to decide that by feel.
    #[test]
    fn spring_stiffness_is_derived_from_mass_not_guessed() {
        const W: f32 = 14.0; // the ω the viewmodel sway already runs at
        let k = |s: Segment| derived_spring_k(s, W);
        assert!(
            k(Segment::ThighL) > k(Segment::ForearmL),
            "a thigh must be stiffer to drive than a forearm: {} vs {}",
            k(Segment::ThighL),
            k(Segment::ForearmL)
        );
        assert!(
            k(Segment::UpperArmL) > k(Segment::HandL),
            "an upper arm must be stiffer than a hand"
        );
        assert!(
            k(Segment::ShankL) > k(Segment::ToeL),
            "a shank must be stiffer than a toe"
        );
        // it scales as ω², the standard relation - so the ONE knob left
        // is a frequency, which is a thing a person can reason about
        let a = derived_spring_k(Segment::ThighL, W);
        let b = derived_spring_k(Segment::ThighL, W * 2.0);
        assert!((b / a - 4.0).abs() < 1e-3, "k must go as omega squared");
        // and every segment with a real length gets a real stiffness -
        // a zero would be a limb that never comes back
        for s in SEGMENTS {
            if segment_data(s).len_frac > 0.0 {
                assert!(k(s) > 0.0, "{s:?} has no stiffness");
            }
        }
    }
}

/// R4 - config externalization's completion gate (camera-tuning slice).
#[cfg(test)]
mod capture_path_tests {
    use crate::*;

    /// A capture that writes its frames somewhere nobody looks is worse
    /// than one that fails: it reports success. This is the regression
    /// guard for exactly that - two frames of the menus capture were
    /// silently written outside the tracked tree because the path was
    /// relative to the working directory.
    #[test]
    fn capture_frames_land_in_the_tracked_tree_not_the_working_directory() {
        let dir = capture_dir("menus");
        assert!(
            std::path::Path::new(&dir).is_absolute(),
            "capture dir must be absolute so the launch directory cannot move it, got {dir:?}"
        );
        assert!(
            dir.ends_with("/handback/brief-vii/menus"),
            "must land in the handback tree, got {dir:?}"
        );
        assert!(
            dir.contains("jk_tdm"),
            "must be anchored inside this crate, got {dir:?}"
        );
        assert!(
            !dir.contains('\\'),
            "separators must be normalised - a mixed path breaks the ends_with checks \
             callers and tests do, got {dir:?}"
        );
        // the crate root it anchors to must actually be the crate root
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        assert!(
            root.join("Cargo.toml").exists(),
            "CARGO_MANIFEST_DIR must point at this crate's root"
        );
        // and every script name gets its own directory, never a shared one
        assert_ne!(capture_dir("menus"), capture_dir("baseline"));
    }
}

#[cfg(test)]
mod lowready_tests {
    use crate::*;

    /// Drive the spring the way the frame loop does: 60 fps, fixed step.
    fn run(target: f32, frames: usize, from: (f32, f32)) -> Vec<f32> {
        let (mut x, mut v) = from;
        (0..frames)
            .map(|_| {
                ready_up_step(&mut x, &mut v, target, 1.0 / 60.0);
                x
            })
            .collect()
    }

    /// §3.4: "returns over 0.15s with ONE SMALL OVERSHOOT (ζ ≈ 0.7)".
    ///
    /// The overshoot is the whole point of the spec line - a lerp would
    /// satisfy "returns over 0.15s" and silently drop the character of
    /// the motion. So assert the overshoot EXISTS, that it is small, and
    /// that there is only one.
    #[test]
    fn ready_up_overshoots_once_and_settles_on_the_brief_s_clock() {
        // hold at full low-ready, then clear the wall
        let xs = run(0.0, 60, (1.0, 0.0));

        let min = xs.iter().cloned().fold(f32::INFINITY, f32::min);
        assert!(
            min < -0.001,
            "ζ=0.7 must overshoot PAST zero - got a minimum of {min}, \
             which is a lerp wearing a spring's name"
        );
        assert!(
            min > -0.15,
            "the overshoot must be SMALL (brief: 'one small overshoot'), got {min}"
        );

        // Exactly one overshoot the eye can SEE. A ζ=0.7 step response
        // rings analytically at 4.6%, then 0.2%, then 0.01% - counting
        // raw zero-crossings would count that decay tail as a wobble and
        // fail a spring that is behaving exactly as specified. So count
        // excursions that clear 1% of the travel, which is the threshold
        // below which nothing is visible on a 22 deg rotation (0.2 deg).
        let visible = {
            let mut n = 0;
            let mut below = false;
            for &x in &xs {
                if x < -0.01 && !below {
                    n += 1;
                    below = true;
                } else if x >= 0.0 {
                    below = false;
                }
            }
            n
        };
        assert_eq!(visible, 1, "one VISIBLE overshoot, not a wobble: {visible}");

        // settled inside the 2% band by the spec's 0.15 s (9 frames at
        // 60 fps), which is what READY_UP_OMEGA was derived to deliver
        let at_spec = xs[(0.15 * 60.0) as usize];
        assert!(
            at_spec.abs() < 0.02,
            "must be within the 2% band at 0.15s, got {at_spec}"
        );
    }

    /// The dip must actually ARRIVE when a wall is close - a spring that
    /// never reaches its target is a slow bug, not a stance.
    #[test]
    fn low_ready_reaches_full_dip_and_the_angles_match_the_brief() {
        let xs = run(1.0, 60, (0.0, 0.0));
        let settled = *xs.last().unwrap();
        assert!(
            (settled - 1.0).abs() < 0.02,
            "the dip must arrive at full, got {settled}"
        );
        // the brief's numbers, not a re-typed approximation of them
        assert!(
            (LOWREADY_PITCH.to_degrees() - 22.0).abs() < 1e-3,
            "§3.4 specifies 22 degrees"
        );
        assert!(
            (LOWREADY_RANGE_M - 0.6).abs() < 1e-6,
            "§3.4 specifies 0.6 m"
        );
        assert!(
            LOWREADY_YAW > 0.0 && LOWREADY_PITCH > 0.0,
            "'up-and-in' is two rotations, not one"
        );
    }

    /// The spring must not explode on a slow frame. An unstable spring
    /// here throws the weapon off screen rather than degrading, so the
    /// sub-stepping is load-bearing and gets its own proof.
    #[test]
    fn the_spring_stays_bounded_at_terrible_framerates() {
        for fps in [10.0_f32, 15.0, 20.0, 30.0, 144.0] {
            let (mut x, mut v) = (1.0_f32, 0.0_f32);
            for _ in 0..200 {
                ready_up_step(&mut x, &mut v, 0.0, 1.0 / fps);
                assert!(
                    x.is_finite() && x.abs() < 2.0,
                    "spring diverged at {fps} fps: x={x}"
                );
            }
            assert!(
                x.abs() < 0.02,
                "must still settle at {fps} fps, ended at {x}"
            );
        }
    }
}

#[cfg(test)]
mod config_tuning_tests {
    use crate::*;

    #[test]
    fn empty_or_missing_text_yields_exactly_the_compiled_in_defaults() {
        let t = parse_camera_tuning("");
        let d = CameraTuning::default();
        assert_eq!(t.tp_boom, d.tp_boom);
        assert_eq!(t.tp_up, d.tp_up);
        assert_eq!(t.tp_right, d.tp_right);
        assert_eq!(t.tp_boom_sprint, d.tp_boom_sprint);
        assert_eq!(t.tp_sprint_lag_s, d.tp_sprint_lag_s);
        assert_eq!(t.tp_boom_aim, d.tp_boom_aim);
        assert_eq!(t.tp_right_aim, d.tp_right_aim);
    }

    #[test]
    fn a_real_edit_overrides_exactly_that_one_key() {
        let t = parse_camera_tuning("tp_boom = 9.5\n");
        assert_eq!(t.tp_boom, 9.5, "the edited key must take effect");
        assert_eq!(t.tp_up, CameraTuning::default().tp_up, "untouched keys keep their default");
    }

    #[test]
    fn comments_blank_lines_and_whitespace_are_all_ignored() {
        let t = parse_camera_tuning(
            "\n  # a comment\n   tp_right   =   1.25   \n\n# tp_up = 99.0 (commented out)\n",
        );
        assert_eq!(t.tp_right, 1.25);
        assert_eq!(t.tp_up, CameraTuning::default().tp_up, "a commented-out line must not apply");
    }

    #[test]
    fn garbage_lines_and_unknown_keys_never_panic_and_never_apply() {
        let t = parse_camera_tuning("not a valid line at all\ntp_boom = not_a_number\nfake_key = 5.0\n");
        assert_eq!(t, CameraTuning::default(), "nothing here should have parsed");
    }
}

/// §7.4 (Brief VII v2) - the Forge's completion gate.
#[cfg(test)]
mod forge_tests {
    use crate::*;

    #[test]
    fn profile_line_round_trips_every_field() {
        for p in [
            ForgeProfile { hat: 0, tunic: 0, melee_axe: false, grenade_preset: 0, helmet: 0, armor: 0 },
            ForgeProfile { hat: 3, tunic: 2, melee_axe: true, grenade_preset: 3, helmet: 4, armor: 0x00FF_FFFF },
            ForgeProfile { hat: 1, tunic: 3, melee_axe: false, grenade_preset: 2, helmet: 2, armor: 0x0A5A },
        ] {
            let line = p.to_line();
            let back = ForgeProfile::from_line(&line).expect("must parse what it wrote");
            assert_eq!(p, back, "round-trip must be exact: {line}");
        }
    }

    #[test]
    fn from_line_rejects_garbage() {
        assert!(ForgeProfile::from_line("").is_none());
        assert!(ForgeProfile::from_line("not,a,valid,profile").is_none());
        assert!(ForgeProfile::from_line("1,2,3").is_none(), "too few fields");
    }

    /// §8.1: slot files written before the helmet field existed must
    /// still load. This is the actual upgrade path - anyone who used the
    /// Forge before this build has four-field files sitting on disk, and
    /// the failure mode if `from_line` were strict is that their saved
    /// profiles vanish on first launch without a word.
    #[test]
    fn a_pre_helmet_save_file_still_loads_as_the_field_cap() {
        let old = ForgeProfile::from_line("3,2,1,3")
            .expect("the four-field format that shipped must still parse");
        assert_eq!(old.hat, 3);
        assert_eq!(old.tunic, 2);
        assert!(old.melee_axe);
        assert_eq!(old.grenade_preset, 3);
        assert_eq!(
            old.helmet, 0,
            "a file with no helmet must read as the FIELD CAP - the shape              it was actually wearing when it was written"
        );
        // and a malformed FIFTH field is still an error, not a silent 0:
        // absent and wrong are different, and only absent is forgiven
        assert!(ForgeProfile::from_line("3,2,1,3,x").is_none());
    }

    /// §8.1: index 0 must still be the exact brim-crown-band the body had
    /// before the library existed. Anyone whose saved profile predates the
    /// helmet field loads as 0 (see above), so if these values drifted,
    /// those profiles would quietly change shape.
    #[test]
    fn helmet_zero_is_the_frozen_field_cap() {
        let (name, pieces) = HELMET_CHOICES[0];
        assert_eq!(name, "FIELD CAP");
        assert_eq!(pieces.len(), 5, "peak, band, crown, dome, badge");
        // peak, band, crown, dome, badge - the §owner HAT GRAPHICS shapes
        assert_eq!(pieces[0].pos, (0.0, 1.018, -0.09));
        assert_eq!(pieces[0].scale, (0.52, 0.030, 0.58));
        assert_eq!(pieces[1].pos, (0.0, 1.045, 0.0));
        assert_eq!(pieces[1].scale, (0.385, 0.045, 0.385));
        assert_eq!(pieces[2].pos, (0.0, 1.115, 0.0));
        assert_eq!(pieces[2].scale, (0.355, 0.16, 0.355));
        // the antenna is shared, and its two pieces are equally pinned
        assert_eq!(HELMET_ANTENNA[0].pos, (0.14, 1.20, 0.0));
        assert_eq!(HELMET_ANTENNA[1].pos, (0.14, 1.285, 0.0));
        assert_eq!(HELMET_ANTENNA[1].scale, (0.038, 0.045, 0.038));
    }

    /// §owner HAT GRAPHICS: the two defects the old FIELD CAP had, stated
    /// as properties rather than as a list of numbers - so they cannot
    /// come back under different values.
    ///
    /// MUTATION-CHECKED: both assertions fail on the pre-change geometry.
    /// The old brim was `(0.72, 0.028, 0.72)` - perfectly circular, so the
    /// peak test fails on it; and 0.72 against a 0.44 head is 1.64x, so the
    /// width test fails on it too.
    #[test]
    fn the_field_cap_wears_a_peak_and_not_a_sombrero() {
        let (_, pieces) = HELMET_CHOICES[0];
        let peak = pieces[0];
        // 1. A PEAK, not a brim: longer front-to-back than side-to-side.
        assert!(
            peak.scale.2 > peak.scale.0 * 1.05,
            "the field cap's peak is {:.2} wide by {:.2} deep - a circular \
             disc on a head is a sombrero, not a cap",
            peak.scale.0,
            peak.scale.2
        );
        // 2. It must not dwarf the skull. The widest head shell in the
        // library is VISOR's dome; a cap wider than about 1.25x that reads
        // as headwear the soldier is standing under.
        let head = HELM_VISOR[0].scale.0;
        assert!(
            peak.scale.0 <= head * 1.25,
            "the cap is {:.2} across against a {:.2} head ({:.2}x)",
            peak.scale.0,
            head,
            peak.scale.0 / head
        );
    }

    /// §owner HAT GRAPHICS: the antenna's lit tip must TOUCH its mast.
    ///
    /// It did not. The mast topped out at y 1.285 and the tip's box ran
    /// 1.2825..1.3175, which is a 2.5mm overlap on a 15mm-thin stalk -
    /// close enough to pass a naive "do the numbers meet" reading and far
    /// enough that the render showed a bright block floating free beside
    /// the head. Asserting a real overlap, not a touch, is the difference.
    ///
    /// MUTATION-CHECKED: fails on the old (0.13, 1.30) tip over the old
    /// (0.13, 1.22, 0.13-tall) mast.
    #[test]
    fn the_antenna_tip_sits_on_its_mast() {
        let (mast, tip) = (HELMET_ANTENNA[0], HELMET_ANTENNA[1]);
        let mast_top = mast.pos.1 + mast.scale.1 * 0.5;
        let tip_bot = tip.pos.1 - tip.scale.1 * 0.5;
        let overlap = mast_top - tip_bot;
        assert!(
            overlap >= 0.01,
            "the antenna tip overlaps its mast by only {overlap:.4} - it \
             reads as a lamp hanging in mid air"
        );
        // and they must share an axis, or the tip is beside the mast
        assert_eq!((mast.pos.0, mast.pos.2), (tip.pos.0, tip.pos.2));
    }

    /// §8.1: the array type on `ForgePreview::helmets` is `N_HELMETS`, not
    /// `HELMET_CHOICES.len()` (rustc crashes on the latter here - see the
    /// const's doc). That decoupling is exactly the kind that rots, so it
    /// is pinned.
    #[test]
    fn helmet_library_is_the_declared_size() {
        assert_eq!(HELMET_CHOICES.len(), N_HELMETS);
    }

    /// §8.1: every piece of every helmet sits inside the socket envelope.
    ///
    /// This is what makes the library safe to extend. A new entry gets
    /// checked for the three things hand-placed geometry actually gets
    /// wrong - sunk into the head, floating above it, or poking out wide
    /// enough to show through cover - without anyone opening the game.
    ///
    /// A leaning piece is measured by its ROTATED extent, not its resting
    /// one: a tilted box reaches further than its scale suggests, and
    /// checking the unrotated box would pass geometry that visibly clips.
    #[test]
    fn helmet_pieces_stay_in_the_socket_envelope() {
        for (name, pieces) in HELMET_CHOICES {
            assert!(!pieces.is_empty(), "{name} has no geometry");
            for (i, p) in pieces.iter().chain(HELMET_ANTENNA.iter()).enumerate() {
                let (hx, hy, hz) = (p.scale.0 * 0.5, p.scale.1 * 0.5, p.scale.2 * 0.5);
                // rotated half-extents: exact for a box, conservative for
                // the cylinder and sphere, which are inscribed in one
                let (cp, sp) = (p.pitch.cos().abs(), p.pitch.sin().abs());
                let (cr, sr) = (p.roll.cos().abs(), p.roll.sin().abs());
                let ry = hy * cp * cr + hz * sp + hx * sr;
                let rx = hx * cr + hy * sr;
                let rz = hz * cp + hy * sp;
                let (lo, hi) = (p.pos.1 - ry, p.pos.1 + ry);
                assert!(
                    lo >= HELMET_Y_MIN,
                    "{name} piece {i} sinks into the head shell: {lo} < {HELMET_Y_MIN}"
                );
                assert!(
                    hi <= HELMET_Y_MAX,
                    "{name} piece {i} floats above the fighter: {hi} > {HELMET_Y_MAX}"
                );
                let reach = (p.pos.0.abs() + rx).max(p.pos.2.abs() + rz);
                assert!(
                    reach <= HELMET_XZ_MAX,
                    "{name} piece {i} reaches {reach} wide, past {HELMET_XZ_MAX} -                      it would show through cover the player thinks hides them"
                );
            }
        }
    }

    /// §8.1: the five must actually look different. A library whose
    /// entries share a silhouette is four wasted menu rows - and the
    /// stated reason for the feature was that TINT washes out at range
    /// and SHAPE does not.
    ///
    /// Measured as a SAMPLED OUTLINE - how wide the helmet is at each of
    /// sixteen heights, which is what a distant player's eye integrates -
    /// rather than as a bounding box. The difference is not academic: the
    /// first version of this test compared bounding boxes and called VISOR
    /// and CREST identical, when one is a brow-and-cheeks helm and the
    /// other is a bare dome under a tall blade. They occupy a similar box
    /// while looking nothing alike, and a test that measures the box would
    /// have had me distort real geometry to satisfy a bad proxy.
    ///
    /// The shared antenna is excluded: every helmet has it, so it adds the
    /// same value to every profile and can only mask a real difference.
    #[test]
    fn the_five_helmets_have_distinct_silhouettes() {
        const BANDS: usize = 16;
        let outline = |pieces: &[HelmetPiece]| {
            let mut w = [0.0_f32; BANDS];
            for (b, slot) in w.iter_mut().enumerate() {
                let y = HELMET_Y_MIN
                    + (HELMET_Y_MAX - HELMET_Y_MIN) * (b as f32 + 0.5) / BANDS as f32;
                for p in pieces {
                    let hy = p.scale.1 * 0.5;
                    if (p.pos.1 - hy..=p.pos.1 + hy).contains(&y) {
                        *slot = slot.max(p.pos.0.abs() + p.scale.0 * 0.5);
                    }
                }
            }
            w
        };
        let all: Vec<_> = HELMET_CHOICES.iter().map(|(n, p)| (*n, outline(p))).collect();
        for (i, (na, a)) in all.iter().enumerate() {
            for (nb, b) in &all[i + 1..] {
                let d: f32 = a.iter().zip(b.iter()).map(|(x, y)| (x - y).abs()).sum();
                assert!(
                    d > 0.30,
                    "{na} and {nb} have near-identical outlines ({d}) - at range                      a player could not tell them apart"
                );
            }
        }
    }

    #[test]
    fn save_then_load_round_trips_through_the_real_filesystem() {
        let slot = 99; // a slot no real save will ever use
        let p = ForgeProfile { hat: 2, tunic: 1, melee_axe: true, grenade_preset: 1, helmet: 3, armor: 0x00F0_F0F0 };
        forge_save(slot, &p).expect("save must succeed");
        let back = forge_load(slot).expect("load must find what was saved");
        assert_eq!(p, back);
        let _ = std::fs::remove_file(forge_slot_path(slot)); // clean up after itself
    }

    /// Task 3.3 sprint-start: the head arrives at a new lean LAST, one
    /// tip-onset behind the pelvis, and the transient vanishes at steady
    /// state - a held sprint must look exactly as before.
    #[test]
    fn the_head_trails_a_sprint_start_then_settles() {
        let dt = 1.0 / 120.0;
        let lean = 0.07_f32; // a hard sprint start's full lean
        let mut lag = 0.0_f32;
        // one tip-onset in, the head must still be visibly behind
        let onset_ticks = (CHAIN_ONSET_OFFSETS[7] / dt) as usize;
        for _ in 0..onset_ticks {
            lag = chain_lag_chase(lag, lean, dt);
        }
        let behind = lean - lag;
        assert!(
            behind > lean * 0.2,
            "one onset in, the head should still trail: {behind} of {lean}"
        );
        // ...and well before half a second, it has fully arrived
        for _ in 0..(0.5 / dt) as usize {
            lag = chain_lag_chase(lag, lean, dt);
        }
        assert!(
            (lean - lag).abs() < lean * 0.02,
            "at steady state the transient must be gone: {}",
            lean - lag
        );
        // the ripple is strictly monotic toward the target - no wobble
        let mut lag2 = 0.0_f32;
        let mut prev_gap = lean;
        for _ in 0..200 {
            lag2 = chain_lag_chase(lag2, lean, dt);
            let gap = lean - lag2;
            assert!(gap <= prev_gap + 1e-7, "the chase must never overshoot");
            prev_gap = gap;
        }
    }

    /// D7 (Thor, 2026-08-03): `the_head_trails_a_sprint_start_then_settles`
    /// above derives its tick count FROM `CHAIN_ONSET_OFFSETS[7]`, so it
    /// self-adjusts to whatever that constant says and passes for any
    /// value of it. Nothing in the suite pinned the one behaviour BRIEF
    /// VIII_B Step 1 actually changed: the head-lag time constant moved
    /// 0.125 -> 0.130 s.
    ///
    /// This pins it. Every number below is HAND-COMPUTED from the literal
    /// 0.130 - never read from the table - so if index 7 moves, this test
    /// fails and the change has to be deliberate.
    ///
    ///   alpha        = dt / 0.130 = (1/120) / 0.130 = 0.064102564
    ///   first tick   = lean * alpha
    ///   gap after n  = lean * (1 - alpha)^n
    ///
    /// FALSIFIABILITY: set `CHAIN_ONSET_OFFSETS[7]` back to 0.125 and the
    /// first tick becomes 0.00466667 (want 0.00448718, 1.8e-4 off = 180x
    /// the tolerance) and the 15-tick gap fraction becomes 0.35526440
    /// (want 0.37018930, 1.5e-2 off = 15000x). Delete the `.min(1.0)` and
    /// the clamp case below overshoots to 0.1077 rad against a 0.07 target.
    /// Measured f32-vs-f64 drift on the gap fractions is <= 8e-8, so the
    /// 1e-6 tolerance has >= 12x headroom; it is a regression pin on an
    /// exact arithmetic identity, NOT a claim of sub-millisecond accuracy
    /// (the source data is 50 fps - see the table's precision-ceiling note).
    #[test]
    fn head_lag_chase_pins_the_measured_tip_onset() {
        let dt = 1.0 / 120.0;
        let lean = 0.07_f32;

        // one tick from rest closes exactly alpha of the gap
        let first = chain_lag_chase(0.0, lean, dt);
        assert!(
            (first - 0.004_487_179_5).abs() < 1e-6,
            "one tick at 120 Hz must close dt/0.130 of the lean: want 0.0044871795, got {first}"
        );

        // and the gap decays geometrically at (1 - alpha) per tick
        let mut lag = 0.0_f32;
        for n in 1..=30 {
            lag = chain_lag_chase(lag, lean, dt);
            let gap_frac = (lean - lag) / lean;
            match n {
                // 15 ticks = 0.125 s, ~one time constant: (1-alpha)^15
                15 => assert!(
                    (gap_frac - 0.370_189_30).abs() < 1e-6,
                    "after 15 ticks the head must still hold 37.019% of the gap \
                     ((1 - (1/120)/0.130)^15); got {gap_frac}"
                ),
                // two time constants: the same number squared
                30 => assert!(
                    (gap_frac - 0.137_040_12).abs() < 1e-6,
                    "after 30 ticks: want 0.13704012, got {gap_frac}"
                ),
                _ => {}
            }
        }

        // a frame longer than the whole time constant must ARRIVE, not
        // overshoot - this is what `.min(1.0)` is for
        let big = chain_lag_chase(0.0, lean, 0.2);
        assert!(
            (big - lean).abs() < 1e-6,
            "a dt past the time constant must land ON the target, not sail through it: got {big}"
        );
    }

    /// §5.2: the turn-in-place. Brief VII v2 shipped `torso_aim_offset`
    /// built and tested with ZERO production call sites - the clamp
    /// existed but nothing ever separated the legs from the aim, so
    /// there was nothing to clamp. `step_leg_yaw` is the missing half.
    #[test]
    fn the_legs_lag_the_aim_and_the_torso_covers_the_difference() {
        let dt = 1.0 / 60.0;
        // first frame SNAPS - a fresh fighter must not spin up from 0
        let (leg, off) = step_leg_yaw(f32::NAN, 2.0, dt);
        assert_eq!(leg, 2.0, "uninitialised legs snap to the aim");
        assert_eq!(off, 0.0, "and need no torso compensation");

        // A small flick is covered by the TORSO immediately, while the
        // legs only creep. (This originally asserted the legs must not
        // move AT ALL within the clamp - which is what left a permanent
        // 60deg twist standing forever. The convergence check below is
        // what caught it, so the assertion was the thing that was wrong.)
        let small = 30.0_f32.to_radians();
        let (leg, off) = step_leg_yaw(0.0, small, dt);
        assert!(
            leg < small * 0.5,
            "one tick must not swing the legs most of the way, got {} deg",
            leg.to_degrees()
        );
        assert!(
            (off + leg - small).abs() < 1e-4,
            "torso + legs must together land exactly on the aim: {} + {} vs {}",
            off.to_degrees(),
            leg.to_degrees(),
            small.to_degrees()
        );

        // a big flick EXCEEDS the clamp, so the legs start catching up
        let big = 140.0_f32.to_radians();
        let (leg, off) = step_leg_yaw(0.0, big, dt);
        assert!(leg > 0.0, "past the clamp the legs must turn, got {leg}");
        assert!(
            off.to_degrees() <= TORSO_AIM_LIMIT_DEG + 1e-3,
            "the torso can never exceed its clamp, got {}",
            off.to_degrees()
        );

        // and they converge: hold the aim and the legs arrive, with the
        // torso returning to neutral
        let (mut leg, mut off) = (0.0_f32, 0.0_f32);
        for _ in 0..600 {
            let (l, o) = step_leg_yaw(leg, big, dt);
            leg = l;
            off = o;
        }
        assert!(
            (wrap_pi(big - leg)).abs() < 1e-3,
            "the legs must finish facing the aim, off by {} deg",
            wrap_pi(big - leg).to_degrees()
        );
        assert!(off.abs() < 1e-3, "and the torso unwinds to neutral");
    }

    /// Turning 350 deg left is really 10 deg right - without wrapping,
    /// crossing the yaw seam sends the legs the long way around.
    #[test]
    fn leg_turn_takes_the_short_way_around_the_seam() {
        let dt = 1.0 / 60.0;
        let from = 3.0_f32; // just under +PI
        let to = -3.0_f32; // just over -PI: 0.28 rad away the SHORT way
        let (leg, _) = step_leg_yaw(from, to, dt);
        // moving the short way means yaw INCREASES past PI (wrapping),
        // never a long sweep back down through zero
        let moved = wrap_pi(leg - from);
        assert!(
            moved > 0.0,
            "must cross the seam forwards, not sweep the long way: moved {moved}"
        );
        assert!(
            moved.abs() < 0.2,
            "and must not overshoot the 0.28 rad gap in one 60fps tick"
        );
    }

    /// The mouse mapping must come from ONE place. The settings label
    /// and the manual each derived it independently and BOTH had it
    /// backwards - on the very control that changes it.
    #[test]
    fn the_mouse_map_label_matches_the_actual_binding() {
        for swap in [false, true] {
            let (aim_btn, fire_btn) = mouse_map(swap);
            let (aim_name, fire_name) = mouse_map_names(swap);
            let name_of = |b: MouseButton| match b {
                MouseButton::Left => "LEFT CLICK",
                MouseButton::Right => "RIGHT CLICK",
                _ => "OTHER",
            };
            assert_eq!(
                name_of(aim_btn), aim_name,
                "swap={swap}: the aim NAME must match the aim BUTTON"
            );
            assert_eq!(
                name_of(fire_btn), fire_name,
                "swap={swap}: the fire NAME must match the fire BUTTON"
            );
            assert_ne!(aim_btn, fire_btn, "aim and fire cannot be the same button");
        }
        // the default is the conventional LEFT-fires mapping
        assert_eq!(mouse_map(false).1, MouseButton::Left, "default: LEFT fires");
        // and the settings row says so
        let s = GameSettings::default();
        let label = settings_label_text(SettingsButtonKind::SwapMouse, &s);
        assert!(
            label.contains("LEFT CLICK fire"),
            "the default label must advertise LEFT as fire, got {label:?}"
        );
    }

    /// Settings persistence: set -> serialize -> parse -> identical, and
    /// a hostile/stale file can never index out of bounds or crash. The
    /// audit table named "not persisted" as an honest gap; this is the
    /// gap closing WITH its round-trip proof, not just an fs::write.
    #[test]
    fn settings_round_trip_and_hostile_files_are_safe() {
        // round-trip every non-default value
        let mut s = GameSettings::default();
        s.swap_mouse = true;
        s.minimap = false;
        s.sens_idx = SENS_CHOICES.len() - 1;
        s.fov_idx = 0;
        s.invert_y = true;
        // §4.6 (Brief VIII): the crosshair family rides the same file.
        // Every field is set AWAY from its default, so a field the
        // serializer forgot cannot pass by accidentally matching.
        s.cross_size = 9;
        s.cross_gap = -3; // negative is legal (§4.6) and must survive
        s.cross_thickness = 4;
        s.cross_dot = true;
        s.cross_outline = false;
        s.cross_outline_px = 3;
        s.cross_color_idx = CROSS_COLOR_CUSTOM_IDX;
        s.cross_rgb = (17, 200, 240);
        s.cross_alpha = 137;
        s.cross_t_shape = true;
        s.cross_dynamic = true;
        // §4.1/§4.3: the HUD readout options ride the same file, same
        // rule - every one set AWAY from its default.
        s.hud_vitals_style = 1;
        s.minimap_rotate = false;
        s.minimap_scale = 45;
        let back = parse_settings(&settings_to_text(&s));
        assert_eq!(back.swap_mouse, s.swap_mouse);
        assert_eq!(back.minimap, s.minimap);
        assert_eq!(back.sens_idx, s.sens_idx);
        assert_eq!(back.fov_idx, s.fov_idx);
        assert_eq!(back.invert_y, s.invert_y);
        assert_eq!(back.cross_size, s.cross_size, "crosshair size must persist");
        assert_eq!(back.cross_gap, s.cross_gap, "a NEGATIVE gap must persist as-is");
        assert_eq!(back.cross_thickness, s.cross_thickness);
        assert_eq!(back.cross_dot, s.cross_dot);
        assert_eq!(back.cross_outline, s.cross_outline);
        assert_eq!(back.cross_outline_px, s.cross_outline_px);
        assert_eq!(back.cross_color_idx, s.cross_color_idx);
        assert_eq!(back.cross_rgb, s.cross_rgb, "custom RGB must persist per channel");
        assert_eq!(back.cross_alpha, s.cross_alpha);
        assert_eq!(back.cross_t_shape, s.cross_t_shape);
        assert_eq!(back.cross_dynamic, s.cross_dynamic);
        assert_eq!(back.hud_vitals_style, s.hud_vitals_style, "§4.1 vitals style must persist");
        assert_eq!(back.minimap_rotate, s.minimap_rotate, "§4.3 rotate must persist");
        assert_eq!(back.minimap_scale, s.minimap_scale, "§4.3 scale must persist");

        // §4.1/§4.3 hostile: the same clamp rule as everything above.
        let evil_hud = "hud_vitals_style = 88\nminimap_scale = 9999\n";
        let h = parse_settings(evil_hud);
        assert!(h.hud_vitals_style <= 1, "vitals style clamps to a real mode");
        assert_eq!(
            h.minimap_scale, MINIMAP_SCALE_RANGE.1,
            "an oversize minimap clamps to the brief's 1.0 ceiling"
        );
        let tiny = parse_settings("minimap_scale = -400\n");
        assert_eq!(
            tiny.minimap_scale, MINIMAP_SCALE_RANGE.0,
            "a negative minimap clamps to the brief's 0.25 floor, never to nothing"
        );

        // hostile: out-of-range indices clamp instead of panicking later
        let evil = "sens_idx = 999\nfov_idx = -5\nswap_mouse = 7\n";
        let p = parse_settings(evil);
        assert_eq!(p.sens_idx, SENS_CHOICES.len() - 1, "oversize index clamps to last");
        assert_eq!(p.fov_idx, 0, "negative index clamps to first");
        assert!(p.swap_mouse, "any nonzero reads as true");
        // and the clamped values actually index safely
        let _ = p.sens_mult();
        let _ = p.fov_deg();

        // §4.6 hostile: every crosshair number clamps into a DRAWABLE
        // range. A zero-size or negative-thickness crosshair is not a
        // preference, it is an invisible or inverted one.
        let evil_cross = "cross_size = 9999\ncross_gap = -9999\ncross_thickness = 0\n\
                          cross_outline_px = 77\ncross_color_idx = 4000\n\
                          cross_r = 900\ncross_g = -12\ncross_alpha = 4096\n";
        let c = parse_settings(evil_cross);
        assert_eq!(c.cross_size, CROSS_SIZE_RANGE.1, "oversize size clamps to max");
        assert_eq!(c.cross_gap, CROSS_GAP_RANGE.0, "gap clamps to its NEGATIVE floor");
        assert_eq!(c.cross_thickness, CROSS_THICK_RANGE.0, "thickness never below 1");
        assert_eq!(c.cross_outline_px, CROSS_OUTLINE_RANGE.1);
        assert_eq!(
            c.cross_color_idx,
            CROSS_COLOR_CHOICES.len() - 1,
            "an out-of-range colour index must not index off the preset table"
        );
        assert_eq!(c.cross_rgb.0, 255, "channel clamps to 255");
        assert_eq!(c.cross_rgb.1, 0, "a negative channel clamps to 0");
        assert_eq!(c.cross_alpha, 255);
        // the clamped values actually produce drawable geometry
        let _ = crosshair_rgb(&c);
        for r in crosshair_arm_rects(c.cross_size as f32, c.cross_gap as f32, c.cross_thickness as f32) {
            assert!(r.w > 0.0 && r.h > 0.0, "clamped settings must still draw: {r:?}");
        }

        // garbage lines are ignored, defaults survive
        let junk = "!!!\nsens_idx = banana\n= 3\nfov_idx\ncross_size = wide\n\
                    cross_gap = \ncross_alpha = 3.5\n";
        let j = parse_settings(junk);
        assert_eq!(j.sens_idx, GameSettings::default().sens_idx);
        assert_eq!(j.fov_idx, GameSettings::default().fov_idx);
        assert_eq!(j.cross_size, CROSS_SIZE_DEFAULT, "non-numeric size keeps the default");
        assert_eq!(j.cross_gap, CROSS_GAP_DEFAULT);
        assert_eq!(j.cross_alpha, CROSS_ALPHA_DEFAULT, "'3.5' is not an integer");
    }

    /// **§4.9, the test that could not previously be written.**
    /// "Crosshair settings round-trip through the settings file" - and
    /// this one goes through an actual FILE, not just the two pure
    /// functions, so a serializer that emits something `parse_settings`
    /// reads but a disk write mangles (line endings, the comment line,
    /// a trailing newline) is caught too.
    #[test]
    fn crosshair_settings_round_trip_through_the_settings_file() {
        let mut s = GameSettings::default();
        s.cross_size = CROSS_SIZE_RANGE.1;
        s.cross_gap = CROSS_GAP_RANGE.0; // the negative extreme
        s.cross_thickness = CROSS_THICK_RANGE.1;
        s.cross_dot = true;
        s.cross_outline = true;
        s.cross_outline_px = 2;
        s.cross_color_idx = CROSS_COLOR_CUSTOM_IDX;
        s.cross_rgb = (1, 2, 254);
        s.cross_alpha = 200;
        s.cross_t_shape = true;
        s.cross_dynamic = true;

        let path = std::env::temp_dir().join(format!(
            "jk_tdm_crosshair_settings_{}.txt",
            std::process::id()
        ));
        std::fs::write(&path, settings_to_text(&s)).expect("write the settings file");
        let text = std::fs::read_to_string(&path).expect("read it back");
        let _ = std::fs::remove_file(&path);

        let back = parse_settings(&text);
        assert_eq!(back.cross_size, s.cross_size);
        assert_eq!(back.cross_gap, s.cross_gap);
        assert_eq!(back.cross_thickness, s.cross_thickness);
        assert_eq!(back.cross_dot, s.cross_dot);
        assert_eq!(back.cross_outline, s.cross_outline);
        assert_eq!(back.cross_outline_px, s.cross_outline_px);
        assert_eq!(back.cross_color_idx, s.cross_color_idx);
        assert_eq!(back.cross_rgb, s.cross_rgb);
        assert_eq!(back.cross_alpha, s.cross_alpha);
        assert_eq!(back.cross_t_shape, s.cross_t_shape);
        assert_eq!(back.cross_dynamic, s.cross_dynamic);
        // and the colour the renderer would use is the one we saved
        assert_eq!(crosshair_rgb(&back), (1, 2, 254));

        // a DEFAULT settings file round-trips to the spec's own defaults:
        // size 5, gap 0, thickness 1, dot off, outline on 1, green
        // 50/250/50, alpha 200, no T-shape, classic STATIC.
        let d = parse_settings(&settings_to_text(&GameSettings::default()));
        assert_eq!(d.cross_size, 5);
        assert_eq!(d.cross_gap, 0);
        assert_eq!(d.cross_thickness, 1);
        assert!(!d.cross_dot, "the dot is OFF by default");
        assert!(d.cross_outline && d.cross_outline_px == 1);
        assert_eq!(crosshair_rgb(&d), (50, 250, 50), "spec default is green 50,250,50");
        assert_eq!(d.cross_alpha, 200);
        assert!(!d.cross_t_shape);
        assert!(!d.cross_dynamic, "the default is classic STATIC");
    }

    /// The drawn geometry must actually MOVE with the settings - the
    /// whole point of replacing a `+` glyph. Every assertion here is
    /// derived by hand from what "size / gap / thickness" mean, not read
    /// back out of the function.
    #[test]
    fn crosshair_geometry_responds_to_size_gap_and_thickness() {
        let a = crosshair_arm_rects(5.0, 2.0, 1.0);
        let top = a[CROSS_ARM_TOP];
        let right = a[CROSS_ARM_RIGHT];
        let bottom = a[CROSS_ARM_BOTTOM];
        let left = a[CROSS_ARM_LEFT];

        // each arm is `size` long along its own axis and `thickness` across
        assert_eq!((top.w, top.h), (1.0, 5.0));
        assert_eq!((bottom.w, bottom.h), (1.0, 5.0));
        assert_eq!((right.w, right.h), (5.0, 1.0));
        assert_eq!((left.w, left.h), (5.0, 1.0));
        // inner edges sit exactly `gap` from the centre, all four ways
        assert_eq!(right.left, 2.0, "right arm starts at the gap");
        assert_eq!(bottom.top, 2.0, "bottom arm starts at the gap");
        assert_eq!(left.left + left.w, -2.0, "left arm ends at -gap");
        assert_eq!(top.top + top.h, -2.0, "top arm ends at -gap");
        // and each arm is centred on its own axis
        assert_eq!(top.left, -0.5);
        assert_eq!(right.top, -0.5);

        // SIZE lengthens the arms without moving their inner edges
        let bigger = crosshair_arm_rects(9.0, 2.0, 1.0);
        assert_eq!(bigger[CROSS_ARM_RIGHT].w, 9.0, "size is the arm length");
        assert_eq!(
            bigger[CROSS_ARM_RIGHT].left, right.left,
            "size must not move the inner edge - that is the gap's job"
        );
        assert!(
            bigger[CROSS_ARM_TOP].top < top.top,
            "a longer top arm must reach FURTHER up"
        );

        // GAP moves the inner edges outward without changing the length
        let wider = crosshair_arm_rects(5.0, 6.0, 1.0);
        assert_eq!(wider[CROSS_ARM_RIGHT].left, 6.0);
        assert_eq!(wider[CROSS_ARM_RIGHT].w, right.w, "gap must not resize an arm");
        assert!(
            wider[CROSS_ARM_LEFT].left < left.left,
            "a bigger gap pushes the left arm further left"
        );

        // THICKNESS widens the cross-axis and keeps the arm centred
        let fat = crosshair_arm_rects(5.0, 2.0, 4.0);
        assert_eq!(fat[CROSS_ARM_TOP].w, 4.0, "thickness is the arm's width");
        assert_eq!(fat[CROSS_ARM_TOP].h, 5.0, "thickness must not change the length");
        assert_eq!(fat[CROSS_ARM_TOP].left, -2.0, "still centred on the vertical axis");
        assert_eq!(fat[CROSS_ARM_RIGHT].top, -2.0, "still centred on the horizontal axis");
        // the dot follows the thickness, centred
        let dot = crosshair_dot_rect(4.0);
        assert_eq!((dot.w, dot.h), (4.0, 4.0));
        assert_eq!((dot.left, dot.top), (-2.0, -2.0));

        // opposite arms are exact mirrors of each other
        assert_eq!(top.left, bottom.left);
        assert_eq!(top.top + top.h, -(bottom.top), "top/bottom mirror through 0");
        assert_eq!(left.top, right.top);
        assert_eq!(left.left + left.w, -(right.left), "left/right mirror through 0");
    }

    /// §4.6 says gap may go NEGATIVE. That is the case most likely to
    /// produce a zero-size or inverted rect, so it gets its own test: at
    /// every legal gap, all four arms stay positively-sized and full
    /// length, and a negative gap really does cross the centre.
    #[test]
    fn crosshair_negative_gap_is_legal_and_never_inverts_a_rect() {
        for gap in CROSS_GAP_RANGE.0..=CROSS_GAP_RANGE.1 {
            for size in CROSS_SIZE_RANGE.0..=CROSS_SIZE_RANGE.1 {
                for thick in CROSS_THICK_RANGE.0..=CROSS_THICK_RANGE.1 {
                    let arms =
                        crosshair_arm_rects(size as f32, gap as f32, thick as f32);
                    for (i, r) in arms.iter().enumerate() {
                        assert!(
                            r.w > 0.0 && r.h > 0.0,
                            "arm {i} collapsed at size={size} gap={gap} thick={thick}: {r:?}"
                        );
                        // vertical arms run along h, horizontal along w -
                        // asserted per-axis, not by max/min, because at
                        // size 1 / thickness 5 an arm is legitimately
                        // wider than it is long
                        let vertical = i == CROSS_ARM_TOP || i == CROSS_ARM_BOTTOM;
                        let (along, across) = if vertical { (r.h, r.w) } else { (r.w, r.h) };
                        assert_eq!(
                            along, size as f32,
                            "arm {i} lost length at size={size} gap={gap} thick={thick}"
                        );
                        assert_eq!(
                            across, thick as f32,
                            "arm {i} lost thickness at size={size} gap={gap} thick={thick}"
                        );
                    }
                }
            }
        }
        // a negative gap must genuinely cross the centre, not clamp to 0
        let crossed = crosshair_arm_rects(5.0, -3.0, 1.0);
        assert!(
            crossed[CROSS_ARM_TOP].top + crossed[CROSS_ARM_TOP].h > 0.0,
            "at gap -3 the top arm's lower edge must sit BELOW the centre"
        );
        assert!(
            crossed[CROSS_ARM_RIGHT].left < 0.0,
            "at gap -3 the right arm must start LEFT of the centre"
        );
        // and the outline only ever grows the rect it backs
        let base = crossed[CROSS_ARM_RIGHT];
        let o = crosshair_outline_rect(base, 2.0);
        assert_eq!((o.w, o.h), (base.w + 4.0, base.h + 4.0));
        assert_eq!((o.left, o.top), (base.left - 2.0, base.top - 2.0));
        let none = crosshair_outline_rect(base, -5.0);
        assert_eq!(none, base, "a negative outline width can never SHRINK the fill");
    }

    /// The three remaining §4.6 switches: T-shape drops exactly one arm,
    /// static ignores the aim cone while dynamic blooms with it, and the
    /// colour comes from the preset table or the custom triple.
    #[test]
    fn crosshair_t_shape_static_dynamic_and_colour_presets() {
        // T-shape hides the TOP arm and nothing else
        for arm in 0..4 {
            assert!(crosshair_arm_shown(arm, false), "arm {arm} shows without T-shape");
        }
        assert!(!crosshair_arm_shown(CROSS_ARM_TOP, true), "T-shape drops the top arm");
        for arm in [CROSS_ARM_RIGHT, CROSS_ARM_BOTTOM, CROSS_ARM_LEFT] {
            assert!(crosshair_arm_shown(arm, true), "T-shape must keep arm {arm}");
        }

        // classic STATIC ignores spread entirely
        assert_eq!(crosshair_gap_px(2, 0.0, false), 2.0);
        assert_eq!(
            crosshair_gap_px(2, 0.05, false),
            2.0,
            "a static crosshair must not move with the aim cone"
        );
        // DYNAMIC blooms with it, monotonically, from the same base
        assert_eq!(crosshair_gap_px(2, 0.0, true), 2.0, "no spread, no bloom");
        let a = crosshair_gap_px(2, 0.01, true);
        let b = crosshair_gap_px(2, 0.05, true);
        assert!(a > 2.0 && b > a, "more spread must open the gap further: {a} then {b}");
        // a negative base gap still blooms outward from where it started
        assert!(crosshair_gap_px(-4, 0.02, true) > -4.0);

        // colour: presets come from the table, CUSTOM from the settings
        let mut s = GameSettings::default();
        s.cross_color_idx = 0;
        assert_eq!(crosshair_rgb(&s), CROSS_COLOR_CHOICES[0].1);
        s.cross_color_idx = 3;
        assert_eq!(crosshair_rgb(&s), CROSS_COLOR_CHOICES[3].1);
        s.cross_color_idx = CROSS_COLOR_CUSTOM_IDX;
        s.cross_rgb = (11, 22, 33);
        assert_eq!(crosshair_rgb(&s), (11, 22, 33), "CUSTOM reads the stored triple");
        // every preset is distinct, or a cycle click would look dead
        for i in 0..CROSS_COLOR_CUSTOM_IDX {
            for j in (i + 1)..CROSS_COLOR_CUSTOM_IDX {
                assert_ne!(
                    CROSS_COLOR_CHOICES[i].1, CROSS_COLOR_CHOICES[j].1,
                    "presets {i} and {j} are the same colour"
                );
            }
        }
    }

    /// The feedback ladder that used to be an inline `match` on the
    /// glyph's `TextColor`. Hiding must beat everything: a scoped weapon
    /// firing from the hip must not leak an aim point through a
    /// hitmarker, which is exactly what a re-ordered ladder would do.
    #[test]
    fn crosshair_hiding_beats_every_other_feedback_state() {
        // §5.2: scoped + unscoped = nothing drawn, whatever else happened
        for kill in [false, true] {
            for hit in [None, Some(false), Some(true)] {
                for blocked in [false, true] {
                    assert_eq!(
                        crosshair_feedback(true, kill, hit, blocked),
                        CrossFeedback::Hidden,
                        "noscope must hide through kill={kill} hit={hit:?} blocked={blocked}"
                    );
                }
            }
        }
        // and Hidden really is invisible, at ANY settings alpha
        for alpha in [0u8, 137, 255] {
            let c = crosshair_color(CrossFeedback::Hidden, (50, 250, 50), alpha)
                .to_srgba();
            assert_eq!(c.alpha, 0.0, "a hidden crosshair must be fully transparent");
        }

        // the rest of the ladder, in order
        assert_eq!(
            crosshair_feedback(false, true, Some(true), true),
            CrossFeedback::Kill,
            "a kill outranks a headshot marker"
        );
        assert_eq!(
            crosshair_feedback(false, false, Some(true), true),
            CrossFeedback::Headshot
        );
        assert_eq!(
            crosshair_feedback(false, false, Some(false), true),
            CrossFeedback::Hit,
            "a body hit outranks the blocked warning"
        );
        assert_eq!(
            crosshair_feedback(false, false, None, true),
            CrossFeedback::Blocked
        );
        assert_eq!(crosshair_feedback(false, false, None, false), CrossFeedback::Idle);

        // Idle is the ONLY state painted in the player's own colour
        let idle = crosshair_color(CrossFeedback::Idle, (50, 250, 50), 200).to_srgba();
        assert!((idle.red - 50.0 / 255.0).abs() < 1e-6);
        assert!((idle.green - 250.0 / 255.0).abs() < 1e-6);
        assert!((idle.blue - 50.0 / 255.0).abs() < 1e-6);
        assert!(
            (idle.alpha - 200.0 / 255.0).abs() < 1e-6,
            "idle alpha is the settings alpha, got {}",
            idle.alpha
        );
        // a feedback flash keeps its own signal colour - turning the
        // crosshair alpha down must not be able to mute a hitmarker
        let quiet_hit = crosshair_color(CrossFeedback::Hit, (50, 250, 50), 10).to_srgba();
        assert!(
            quiet_hit.alpha > 0.9,
            "a hitmarker must stay legible at alpha 10, got {}",
            quiet_hit.alpha
        );
        assert!(quiet_hit.red > quiet_hit.green, "the hit flash is red, not the fill green");
    }

    /// The settings rows are a real control surface: every row renders a
    /// live label, and every crosshair row's click actually changes the
    /// value it advertises. A persisted field with a no-op row is a dead
    /// control wearing a live one's clothes.
    #[test]
    fn every_crosshair_row_cycles_its_own_value_and_wraps_in_range() {
        // numeric cycles step up and wrap to the FLOOR, not to zero
        assert_eq!(cycle_i32(2, (-5, 12)), 3);
        assert_eq!(cycle_i32(12, (-5, 12)), -5, "wraps to the negative floor");
        assert_eq!(cycle_i32(-5, (-5, 12)), -4, "steps up out of the floor");
        // clicking from ANY start lands inside the range, always
        for range in [CROSS_SIZE_RANGE, CROSS_GAP_RANGE, CROSS_THICK_RANGE] {
            let mut v = range.0;
            for _ in 0..(range.1 - range.0 + 3) {
                v = cycle_i32(v, range);
                assert!(v >= range.0 && v <= range.1, "cycled out of range: {v} in {range:?}");
            }
        }
        // and the cycle visits every value before repeating
        let mut seen = std::collections::BTreeSet::new();
        let mut v = CROSS_GAP_RANGE.0;
        for _ in 0..(CROSS_GAP_RANGE.1 - CROSS_GAP_RANGE.0 + 1) {
            seen.insert(v);
            v = cycle_i32(v, CROSS_GAP_RANGE);
        }
        assert_eq!(
            seen.len() as i32,
            CROSS_GAP_RANGE.1 - CROSS_GAP_RANGE.0 + 1,
            "every gap value must be reachable by clicking"
        );

        // alpha cycles by VALUE, so it recovers from a hand-edited file
        assert_eq!(cycle_alpha(200), 230);
        assert_eq!(cycle_alpha(255), CROSS_ALPHA_CHOICES[0], "wraps at the top");
        assert_eq!(cycle_alpha(137), 160, "an off-preset value steps to the next preset");
        assert_eq!(cycle_alpha(0), CROSS_ALPHA_CHOICES[0]);

        // every row on the page has a live label, and no two rows share
        // one (a duplicated label means two rows edit the same thing)
        let s = GameSettings::default();
        let mut labels = std::collections::BTreeSet::new();
        for (_, kind, _) in SETTINGS_ROWS {
            let l = settings_label_text(kind, &s);
            assert!(!l.is_empty(), "a settings row rendered an empty label");
            assert!(labels.insert(l.clone()), "two settings rows render {l:?}");
        }
        assert_eq!(labels.len(), SETTINGS_ROWS.len());

        // and every CROSSHAIR row's label moves when its value moves
        let mut s = GameSettings::default();
        let before = settings_label_text(SettingsButtonKind::CrossSize, &s);
        s.cross_size = cycle_i32(s.cross_size, CROSS_SIZE_RANGE);
        assert_ne!(before, settings_label_text(SettingsButtonKind::CrossSize, &s));
        let before = settings_label_text(SettingsButtonKind::CrossGap, &s);
        s.cross_gap = cycle_i32(s.cross_gap, CROSS_GAP_RANGE);
        assert_ne!(before, settings_label_text(SettingsButtonKind::CrossGap, &s));
        let before = settings_label_text(SettingsButtonKind::CrossThickness, &s);
        s.cross_thickness = cycle_i32(s.cross_thickness, CROSS_THICK_RANGE);
        assert_ne!(
            before,
            settings_label_text(SettingsButtonKind::CrossThickness, &s)
        );
        let before = settings_label_text(SettingsButtonKind::CrossDot, &s);
        s.cross_dot = !s.cross_dot;
        assert_ne!(before, settings_label_text(SettingsButtonKind::CrossDot, &s));
        let before = settings_label_text(SettingsButtonKind::CrossColor, &s);
        s.cross_color_idx = (s.cross_color_idx + 1) % CROSS_COLOR_CHOICES.len();
        assert_ne!(before, settings_label_text(SettingsButtonKind::CrossColor, &s));
        let before = settings_label_text(SettingsButtonKind::CrossAlpha, &s);
        s.cross_alpha = cycle_alpha(s.cross_alpha);
        assert_ne!(before, settings_label_text(SettingsButtonKind::CrossAlpha, &s));
        let before = settings_label_text(SettingsButtonKind::CrossTShape, &s);
        s.cross_t_shape = !s.cross_t_shape;
        assert_ne!(before, settings_label_text(SettingsButtonKind::CrossTShape, &s));
        let before = settings_label_text(SettingsButtonKind::CrossDynamic, &s);
        s.cross_dynamic = !s.cross_dynamic;
        assert_ne!(
            before,
            settings_label_text(SettingsButtonKind::CrossDynamic, &s)
        );
        // every value reached by clicking still survives the file
        let back = parse_settings(&settings_to_text(&s));
        assert_eq!(back.cross_size, s.cross_size);
        assert_eq!(back.cross_gap, s.cross_gap);
        assert_eq!(back.cross_alpha, s.cross_alpha);
        assert_eq!(back.cross_color_idx, s.cross_color_idx);
    }

    /// Settings must be real: every choice list has to be non-empty,
    /// ordered, and indexable by the default, or the settings screen
    /// panics or silently shows the wrong row.
    #[test]
    fn settings_choice_lists_are_valid_and_defaults_are_in_range() {
        assert!(SENS_DEFAULT_IDX < SENS_CHOICES.len());
        assert!(FOV_DEFAULT_IDX < FOV_CHOICES.len());
        let s = GameSettings::default();
        assert!((s.sens_mult() - 1.0).abs() < 1e-6, "default sensitivity is 1.00x");
        assert_eq!(s.fov_deg(), 90.0, "default FOV is the recommended 90");
        // strictly increasing, so cycling reads as a real scale
        for w in SENS_CHOICES.windows(2) {
            assert!(w[1].1 > w[0].1, "sensitivity choices must ascend");
        }
        for w in FOV_CHOICES.windows(2) {
            assert!(w[1].1 > w[0].1, "FOV choices must ascend");
        }
        // and every index is reachable by cycling, landing back at the start
        let mut idx = 0usize;
        for _ in 0..SENS_CHOICES.len() {
            idx = (idx + 1) % SENS_CHOICES.len();
        }
        assert_eq!(idx, 0, "cycling must wrap exactly");
    }

    /// A settings value that cannot be read back is a dead control.
    #[test]
    fn every_settings_row_renders_a_distinct_live_label() {
        let mut s = GameSettings::default();
        let kinds = [
            SettingsButtonKind::Sens,
            SettingsButtonKind::Fov,
            SettingsButtonKind::InvertY,
            SettingsButtonKind::SwapMouse,
            SettingsButtonKind::Minimap,
        ];
        for k in kinds {
            assert!(!settings_label_text(k, &s).is_empty());
        }
        // and each row's label actually CHANGES when its value does
        let before = settings_label_text(SettingsButtonKind::Sens, &s);
        s.sens_idx = (s.sens_idx + 1) % SENS_CHOICES.len();
        assert_ne!(before, settings_label_text(SettingsButtonKind::Sens, &s));
        let before = settings_label_text(SettingsButtonKind::Fov, &s);
        s.fov_idx = (s.fov_idx + 1) % FOV_CHOICES.len();
        assert_ne!(before, settings_label_text(SettingsButtonKind::Fov, &s));
        let before = settings_label_text(SettingsButtonKind::InvertY, &s);
        s.invert_y = !s.invert_y;
        assert_ne!(before, settings_label_text(SettingsButtonKind::InvertY, &s));
    }

    /// §owner FRONT END P6: category navigation means EVERY setting is
    /// still reachable, and every tab still has something behind it.
    ///
    /// The failure this catches is the one a category system always has:
    /// a group whose category is never offered as a tab, so seventeen
    /// rows become fourteen and nobody notices because the screen looks
    /// tidier. Mutation-proven: deleting any entry from
    /// `SETTINGS_CATEGORIES`, or repointing any `SettingsGroup::category`
    /// arm at a category not in that table, fails here.
    #[test]
    fn every_setting_is_still_reachable_behind_some_tab() {
        let tabs: Vec<SettingsCategory> =
            SETTINGS_CATEGORIES.iter().map(|(c, _, _)| *c).collect();
        assert!(!tabs.is_empty());
        for (_, _, group) in SETTINGS_ROWS {
            assert!(
                tabs.contains(&group.category()),
                "{} is filed under a category no tab offers",
                group.title(),
            );
        }
        // ...and no tab is a door onto an empty room.
        for (cat, name, blurb) in SETTINGS_CATEGORIES {
            let n = SETTINGS_ROWS
                .iter()
                .filter(|(_, _, g)| g.category() == cat)
                .count();
            assert!(n > 0, "the {name} tab has no settings behind it");
            assert!(!blurb.is_empty(), "{name} has no line saying what is inside");
            assert!(name.is_ascii() && blurb.is_ascii(), "{name}: tofu");
        }
        // The default tab must exist, or the screen opens on nothing.
        assert!(SettingsCat::default().0 < SETTINGS_CATEGORIES.len());
    }

    /// §C tier 2: a slot file written before the harness field existed
    /// still loads, and loads as PLATE rather than as a naked man.
    ///
    /// The same upgrade path the helmet field needed, and the same
    /// failure if it were strict - except worse: a harness defaulting to
    /// 0 would silently strip every saved profile, and the player would
    /// discover it by dying faster than they used to.
    #[test]
    fn a_pre_armor_save_file_still_loads_as_plate() {
        let five = "3,2,1,3,4"; // the helmet-era format
        let p = ForgeProfile::from_line(five).expect("a five-field file must still load");
        assert_eq!(p.helmet, 4);
        assert_eq!(
            p.armor,
            sim::default_harness(sim::Class::Line).0,
            "an absent harness must read as the class default, not as nothing"
        );
        assert!(
            sim::ArmorLoadout(p.armor).weight_kg() > 0.0,
            "loading an old profile must not undress the player"
        );
        // and the four-field original still works too
        let four = ForgeProfile::from_line("1,1,0,2").expect("the original format");
        assert_eq!(four.helmet, 0);
        assert_eq!(four.armor, sim::default_harness(sim::Class::Line).0);
        // a PRESENT but garbage harness is still an error - otherwise a
        // corrupt file would be silently accepted as an old one
        assert!(ForgeProfile::from_line("1,1,0,2,0,zzz").is_none());
    }

    /// Every plate in the library reaches the grid exactly once.
    ///
    /// The grid is built by filtering `ArmorPiece::ALL` per zone, so a
    /// piece whose `zone()` fell outside the four rows would vanish from
    /// the UI while still counting toward weight - equippable in a save
    /// file, invisible in the Forge, and impossible to take off.
    #[test]
    fn every_plate_appears_in_exactly_one_forge_row() {
        for p in sim::ArmorPiece::ALL {
            let n: usize = ARMOUR_ROWS
                .iter()
                .map(|(_, row)| row.iter().filter(|q| **q == p).count())
                .sum();
            assert_eq!(n, 1, "{} appears in {n} grid rows, not 1", p.name());
        }
        let shown: usize = ARMOUR_ROWS.iter().map(|(_, r)| r.len()).sum();
        assert_eq!(
            shown,
            sim::ARMOR_PIECES,
            "the grid must show the WHOLE harness - a plate missing from \
             every row is one that still counts toward weight, is \
             equippable from a save file, and cannot be taken off"
        );
        // no two plates share a pill label - they are toggles, and a
        // duplicate label is a toggle nobody can identify
        for p in sim::ArmorPiece::ALL {
            let same = sim::ArmorPiece::ALL
                .iter()
                .filter(|q| q.short_name() == p.short_name())
                .count();
            assert_eq!(same, 1, "{} is not a unique pill label", p.short_name());
        }
    }

    /// §C tier 2: every plate in the harness has GEOMETRY, and the
    /// arrays that carry it are indexed by the same position the
    /// equipped bitmask uses.
    ///
    /// The failure this guards is quiet and total: `armor_plates` is a
    /// bare array indexed by `ArmorPiece::ALL`'s position, and if the
    /// two ever disagree by one the player equips a greave and a
    /// pauldron appears. Nothing would crash and nothing would look
    /// obviously wrong until someone counted.
    #[test]
    fn every_plate_has_a_model_slot_at_its_own_index() {
        // the array the rig carries is exactly the size of the harness
        assert_eq!(
            sim::ARMOR_PIECES,
            sim::ArmorPiece::ALL.len(),
            "the plate array and the piece list must be the same length"
        );
        // and the index a piece occupies in ALL is the index the bitmask
        // uses - the one relationship the whole mapping rests on
        for (k, p) in sim::ArmorPiece::ALL.into_iter().enumerate() {
            let mut only = sim::ArmorLoadout::EMPTY;
            only.set(p, true);
            assert_eq!(
                only.0,
                1u32 << k,
                "{} sits at index {k} in ALL but its bit is elsewhere",
                p.name()
            );
        }
    }

    /// The grid stays LEGIBLE: even rows, and labels short enough not to
    /// wrap at the pill widths this layout produces.
    ///
    /// The first version used the four damage ZONES as its rows, which
    /// put ten pills in LEGS against two in HEAD - every label in the
    /// long rows wrapped to two lines and the page read as a wall. The
    /// grouping is anatomical now, and this is what stops it drifting
    /// back.
    #[test]
    fn the_armour_grid_rows_stay_even_and_short() {
        const MAX_PILLS: usize = 6;
        const MAX_LABEL: usize = 11;
        for (name, row) in ARMOUR_ROWS {
            assert!(!row.is_empty(), "{name} is an empty row");
            assert!(
                row.len() <= MAX_PILLS,
                "{name} has {} pills - past {MAX_PILLS} the labels wrap",
                row.len()
            );
            for p in row {
                assert!(
                    p.short_name().len() <= MAX_LABEL,
                    "{} is {} chars; the pill fits about {MAX_LABEL}",
                    p.short_name(),
                    p.short_name().len()
                );
            }
        }
        // and the rows must leave the turntable card its space
        assert!(
            ARMOURY_ROW_W_PCT < 100.0,
            "full-width rows run under the soldier preview"
        );
    }

    #[test]
    fn apply_to_selected_only_touches_the_forges_own_fields() {
        let mut sel = Selected::default();
        sel.map = MapKind::Bailey; // untouched by the Forge - must survive
        let p =
            ForgeProfile { hat: 3, tunic: 0, melee_axe: true, grenade_preset: 3, helmet: 1, armor: 0x00AB_CDEF };
        p.apply_to(&mut sel);
        assert_eq!(sel.hat, 3);
        assert_eq!(sel.tunic, 0);
        assert!(sel.melee_axe);
        assert_eq!(sel.grenade_preset, 3);
        assert_eq!(sel.map, MapKind::Bailey, "the Forge must not touch match config");
    }
}

/// §2 (MISSION doc rig audit) - the separation test. The render root
/// (parent of legs) carries exactly `f.yaw`; torso is the root's CHILD
/// with `torso_coil_yaw(..)` as its OWN additional local Y-rotation - so
/// this function's return value literally IS "thorax yaw minus pelvis
/// yaw" (composition of a child's local rotation onto its parent), not
/// an estimate of it. This test is the direct rebuttal-or-confirmation
/// of the document's premise that a single trunk bone forces this to 0.
#[cfg(test)]
mod rig_separation_tests {
    use crate::*;

    #[test]
    fn hip_shoulder_separation_reaches_35_to_45_degrees_at_windup() {
        // sweep the windup progress (spear_wind_t counting DOWN from
        // SPEAR_WINDUP_S to 0) and track the peak |separation|
        let mut peak_deg = 0.0_f32;
        for i in 0..=100 {
            // the plant fraction runs 0 -> 1 (the sim's direction of
            // travel now, not the countdown the client used to invert)
            let plant = i as f32 / 100.0;
            let sep = torso_coil_yaw(GunKind::Spear, Some(plant), 0.0, false, 0.0, 0.0);
            peak_deg = peak_deg.max(sep.abs().to_degrees());
        }
        assert!(
            (35.0..=45.0).contains(&peak_deg),
            "separation peak {peak_deg:.1}deg outside the 35-45deg target"
        );
    }

    #[test]
    fn separation_is_genuinely_nonzero_not_a_fused_bone() {
        // the document's exact failure mode to rule out: if root and
        // torso were the SAME rotation (a fused single trunk bone), this
        // would be identically 0.0 at every sample - it is not.
        let sep = torso_coil_yaw(GunKind::Spear, Some(0.5), 0.0, false, 0.0, 0.0);
        assert_ne!(sep, 0.0, "torso and root must NOT share one fused rotation");
    }

    #[test]
    fn no_gun_no_twist() {
        // separation is a THROW-specific coil, not a permanent offset -
        // resting state must be neutral. The last parameter is the
        // follow-through clock; NEGATIVE is "nothing thrown yet", which
        // is the actual resting state this test means. (It used to pass
        // 0.0 and still read as rest only because the old curve was
        // silent at t=0 - which was itself the bug: a real release
        // starts at the release yaw, not at neutral.)
        assert_eq!(torso_coil_yaw(GunKind::M4, None, 0.0, false, -1.0, 0.0), 0.0);
        assert_eq!(torso_coil_yaw(GunKind::Spear, None, 0.0, false, -1.0, 0.0), 0.0);
        // a gun that is not a spear never twists, whatever the clock says
        assert_eq!(torso_coil_yaw(GunKind::M4, None, 0.0, false, 0.0, 0.0), 0.0);
        // and long after a throw, the follow-through has fully settled
        assert!(torso_coil_yaw(GunKind::Spear, None, 0.0, false, 3.0, 0.0).abs() < 1e-3);
    }
}

/// Task 3 (MISSION doc) - the elastic load model's completion gate.
#[cfg(test)]
mod elastic_load_tests {
    use crate::*;

    #[test]
    fn load_release_ratio_matches_spec_examples() {
        // the spec's own worked example: 0.4s wind-up -> 0.15-0.20s release
        let spear_throw = ElasticMove {
            load_s: 0.4,
            release_s: 0.18,
            stored_energy: 1.0,
            return_efficiency: 0.92,
        };
        assert!(spear_throw.load_release_ok(), "0.4s/0.18s must satisfy the 2x+ rule");
        let too_slow = ElasticMove {
            load_s: 0.4,
            release_s: 0.25,
            stored_energy: 1.0,
            return_efficiency: 0.92,
        };
        assert!(!too_slow.load_release_ok(), "0.25s release from a 0.4s load is a SHOVE, not a strike");
    }

    #[test]
    fn stored_energy_scales_output_by_exactly_the_spec_formula() {
        let base = 22.0; // e.g. the spear's throw v0
        let dead = ElasticMove { load_s: 0.4, release_s: 0.18, stored_energy: 0.0, return_efficiency: 0.92 };
        let full = ElasticMove { load_s: 0.4, release_s: 0.18, stored_energy: 1.0, return_efficiency: 0.92 };
        assert_eq!(dead.release_velocity(base), base, "zero stored energy = base, unscaled");
        assert!(
            (full.release_velocity(base) - base * 1.35).abs() < 1e-4,
            "full stored energy must be exactly base × 1.35"
        );

        // §C.3: `return_efficiency` was declared and never read, which
        // made "steel springs are worse than tendons, and the mech
        // should feel it" a comment rather than a mechanic. A mech coil
        // must now give back measurably less than a human one from the
        // identical load.
        let mech = ElasticMove {
            load_s: 0.4,
            release_s: 0.18,
            stored_energy: 1.0,
            return_efficiency: MECH_RETURN_EFFICIENCY,
        };
        assert!(
            mech.release_velocity(base) < full.release_velocity(base),
            "steel must give back less than tendon: {} vs {}",
            mech.release_velocity(base),
            full.release_velocity(base)
        );
        // and by the exact ratio of the two efficiencies, so the number
        // traces to the brief rather than to taste
        let want = base * (1.0 + 0.35 * (MECH_RETURN_EFFICIENCY / HUMAN_RETURN_EFFICIENCY));
        assert!(
            (mech.release_velocity(base) - want).abs() < 1e-4,
            "mech return must scale by 0.55/0.92, got {}",
            mech.release_velocity(base)
        );
    }

    #[test]
    fn counter_movement_grants_the_bonus_a_dead_start_does_not() {
        // moving down then releasing up = counter-movement = bonus
        assert_eq!(counter_movement_bonus(-1.0, 1.0, 0.35), 0.35);
        // moving up then releasing up = no counter-movement = no bonus
        assert_eq!(counter_movement_bonus(1.0, 1.0, 0.35), 0.0);
        // starting from rest = no counter-movement = no bonus
        assert_eq!(counter_movement_bonus(0.0, 1.0, 0.35), 0.0);
    }

    #[test]
    fn landing_rebound_never_reaches_exactly_zero_for_a_real_impact() {
        for impact in [-3.0_f32, -6.0, -9.5, -15.0] {
            let reb = landing_rebound_vy(impact);
            assert!(reb > 0.0, "a real impact ({impact} m/s) must return SOME upward rebound");
            assert!((reb - (-impact) * 0.08).abs() < 1e-4, "must be exactly 8% of impact speed");
        }
        assert_eq!(landing_rebound_vy(0.0), 0.0, "no impact = no rebound, not a negative dip");
    }

    /// Task 3.4 chain-timing test: angular-velocity peaks occur in strict
    /// order pelvis -> lumbar -> thorax -> shoulder -> elbow -> wrist ->
    /// tip, each with a minimum onset gap. A failure names which segment
    /// fired early (out of order relative to the one before it).
    #[test]
    fn kinetic_chain_peaks_fire_in_strict_proximal_to_distal_order() {
        let ramp_s = 0.05;
        let names = ["pelvis", "lumbar", "thorax", "clavicle", "upper_arm", "forearm", "hand", "tip"];
        let mut prev_peak_tick = -1.0_f32;
        for i in 0..8 {
            let peak_tick = chain_peak_tick(i, ramp_s);
            assert!(
                peak_tick > prev_peak_tick,
                "{} peaked at {peak_tick:.3}s, not after the previous segment ({prev_peak_tick:.3}s) - fired early",
                names[i]
            );
            prev_peak_tick = peak_tick;
        }
    }

    /// §3 (BRIEF_VIII_B): the chain table must still hit the MEASURED
    /// javelin anchors exactly. `JAVELIN_ANCHOR_S` is held as its own
    /// table so this compares two independent things rather than a
    /// constant against itself - if someone retunes the interpolated
    /// indices (1,2,5,6) that is a judgement call, but silently moving
    /// a measured anchor is a factual error and fails here.
    #[test]
    fn the_kinetic_chain_still_hits_every_measured_javelin_anchor() {
        for (idx, want) in JAVELIN_ANCHOR_S {
            let got = CHAIN_ONSET_OFFSETS[idx];
            assert!(
                (got - want).abs() < 1e-6,
                "index {idx} is a MEASURED anchor (Campos 2004 Table 3): \
                 expected {want}s, table says {got}s"
            );
        }
        // the interpolated indices must stay strictly inside their
        // bracketing anchors - an interpolation that escapes its own
        // window is not an interpolation
        assert!(
            CHAIN_ONSET_OFFSETS[1] > CHAIN_ONSET_OFFSETS[0]
                && CHAIN_ONSET_OFFSETS[2] > CHAIN_ONSET_OFFSETS[1]
                && CHAIN_ONSET_OFFSETS[2] < CHAIN_ONSET_OFFSETS[3],
            "trunk interpolation escaped the pelvis..clavicle window"
        );
        assert!(
            CHAIN_ONSET_OFFSETS[5] > CHAIN_ONSET_OFFSETS[4]
                && CHAIN_ONSET_OFFSETS[6] > CHAIN_ONSET_OFFSETS[5]
                && CHAIN_ONSET_OFFSETS[6] < CHAIN_ONSET_OFFSETS[7],
            "arm interpolation escaped the upper-arm..tip window"
        );
        // D5 (Thor, 2026-08-03): what stood here compared
        // OFFSETS[3]-OFFSETS[0] against OFFSETS[4]-OFFSETS[3] and called
        // it "distal compression". It was UNFALSIFIABLE - all three
        // indices are measured anchors already pinned to 1e-6 by the loop
        // twenty lines above, so no edit could ever fail it - and its
        // comment was backwards: 0->3 spans THREE hops and 3->4 spans
        // ONE, so per hop that is 13.3 ms then 30.0 ms. Across that
        // boundary the chain EXPANDS, not compresses, because the 5 ms
        // clavicle floor squeezes the trunk hops. Real distal compression
        // is a claim about the ARM window, and it is the only place where
        // an INTERPOLATED index (5, 6) can make it fail.
        //
        // F3 (Thor, 2026-08-03): the failure message used to end "indices
        // 5 and 6 are the interpolated ones", which reads as a claim that
        // this assertion CONSTRAINS them. It constrains them, but only
        // coarsely, and the bounds are cheap to write down. Holding the
        // other seven rows fixed and stepping one index a millisecond at
        // a time, per-hop monotonicity survives across:
        //
        //     idx5 in [0.093 .. 0.098]   (shipped 0.094: -1 / +4 ms slack)
        //     idx6 in [0.112 .. 0.117]   (shipped 0.114: -2 / +3 ms slack)
        //
        // So it fires on idx5 moved DOWN >=2 ms or UP >=5 ms, and on idx6
        // moved DOWN >=3 ms or UP >=4 ms - and it does NOT fire on a 1 ms
        // move of either. Thor measured exactly that: idx6 0.114 -> 0.115
        // PASSES here. The 1 ms guard is
        // `the_arm_onsets_reproduce_an_independently_solved_geometric_root`,
        // which rejects 0.115 by 9.75e-4 against its 5e-4 gate.
        //
        // MESSAGE NARROWED, ASSERTION LEFT ALONE - deliberately. The only
        // way to broaden this check to 1 ms resolution is to re-derive the
        // geometric ratio right here, and that is the other test's entire
        // job. A second copy of it would be redundancy wearing the costume
        // of coverage, which is the same error class in a new place. What
        // was false was the CLAIM, so the claim is what changed.
        let hop = |a: usize, b: usize| CHAIN_ONSET_OFFSETS[b] - CHAIN_ONSET_OFFSETS[a];
        let arm_hops = [hop(3, 4), hop(4, 5), hop(5, 6), hop(6, 7)];
        for w in arm_hops.windows(2) {
            assert!(
                w[1] < w[0],
                "the ARM window must compress hop by hop, got {arm_hops:?}. \
                 This is a SHAPE check on the interpolated indices 5 and 6, \
                 not a millisecond one: it only fires outside \
                 idx5 in [0.093, 0.098] / idx6 in [0.112, 0.117]. A 1 ms \
                 move of either passes HERE and is caught by \
                 the_arm_onsets_reproduce_an_independently_solved_geometric_root"
            );
        }
        // The honest version of the trunk-vs-arm statement, recorded as a
        // FACT and deliberately NOT asserted: 0->3 is 40 ms over three
        // hops (13.3 ms each), 3->4 is 30 ms over one, so leaving the
        // trunk the chain steps UP. It is not asserted because indices
        // 0, 3 and 4 are all measured anchors already pinned to 1e-6 by
        // the loop at the top of this test - an assertion over them
        // cannot fail, and an assertion that cannot fail is worse than a
        // comment, because it looks like coverage. That is what was here.
    }

    /// D3 (Thor, 2026-08-03): the spec's §3.3 arm derivation printed
    /// `q = 0.8107` for `q + q^2 + q^3 = 2`. That is wrong - the root is
    /// 0.81053571; 0.8107 sums to 2.0007545, so the three arm hops come to
    /// 60.023 ms and the tip lands at 130.023 ms, not "exactly" 130. The
    /// SHIPPED TABLE IS UNAFFECTED (the true root still rounds to
    /// 0.094/0.114/0.130), which is why no constant moved - but the spec
    /// said "exactly" while its own printed sum said 60.03, and nothing in
    /// the suite would have noticed either way.
    ///
    /// So: solve the root HERE, by bisection, from the MEASURED anchors
    /// only. Nothing in this test reads the spec's q, and nothing reads
    /// indices 5 or 6 except to check them. That makes it an independent
    /// source of truth for the two interpolated arm indices, which the
    /// anchor loop above cannot touch.
    #[test]
    fn the_arm_onsets_reproduce_an_independently_solved_geometric_root() {
        let anchor = |i: usize| {
            JAVELIN_ANCHOR_S.iter().find(|(k, _)| *k == i).expect("measured anchor").1 as f64
        };
        // seed and span come from the measurement, not from the table
        let base = anchor(4) - anchor(3); // 30 ms, the last MEASURED gap
        let span = anchor(7) - anchor(4); // 60 ms of arm left to fill
        let target = span / base; // == 2.0
        // solve base*(q + q^2 + q^3) == span for q
        let (mut lo, mut hi) = (0.0_f64, 1.5_f64);
        for _ in 0..200 {
            let m = 0.5 * (lo + hi);
            if m + m * m + m * m * m < target {
                lo = m;
            } else {
                hi = m;
            }
        }
        let q = 0.5 * (lo + hi);
        // The exact root of q + q^2 + q^3 = 2 is 0.8105357138. This
        // bisection cannot reach it: it solves for `target`, and `target`
        // is built from f32 consts - `0.130f32` is really 0.129999995231,
        // so `target` is 1.9999998 rather than 2, which walks the root
        // back by ~5e-8. That is a fact about reading the anchors instead
        // of hardcoding them, and reading them is the entire point. 1e-6
        // absorbs it and is still 164x tighter than the spec's 0.8107.
        assert!(
            (q - 0.810_535_713_8).abs() < 1e-6,
            "the geometric root is 0.8105357138, not {q} (the spec said 0.8107, \
             which sums to 2.0007545 and puts the tip at 130.023 ms)"
        );
        // the shipped table is these hops accumulated and rounded to the
        // nearest MILLISECOND - so 5e-4 is the exact rounding claim, not a
        // slack tolerance. Index 5's margin is the tight one: it lands
        // 3.16e-4 from 0.094, i.e. 1.6x inside the half-millisecond.
        let want = [anchor(4) + base * q, anchor(4) + base * (q + q * q), anchor(7)];
        for (i, w) in [(5usize, want[0]), (6, want[1]), (7, want[2])] {
            let got = CHAIN_ONSET_OFFSETS[i] as f64;
            assert!(
                (got - w).abs() < 5e-4,
                "index {i}: the geometric compression puts it at {w:.7}s, which \
                 rounds to {:.3}s; the table says {got}s",
                (w * 1000.0).round() / 1000.0
            );
        }
        // this test CANNOT tell 0.8107 from 0.81053571 at the table's 1 ms
        // resolution - both round to the same three values. That is D3's
        // point, and the reason no constant changed. What it CAN catch is
        // any 1 ms move of index 5 or 6.
    }

    /// D6 (Thor, 2026-08-03) - the worst of the seven. What stood here
    /// NEVER CALLED `spear_followthrough_yaw`. It retyped that function's
    /// internal drive expression and then asserted the retyped copy
    /// equalled the algebra, so it was guarding a LEMMA under the name of
    /// the THEOREM: delete the `+ onset` from the real function and this
    /// test stayed GREEN. That is not hypothetical - it is the exact bug
    /// already shipped once in this file (handback/AUDIT.md, "bugs I
    /// introduced this session" #1: the follow-through went silent for a
    /// whole tip-onset and then swung the wrong way).
    ///
    /// SCOPE OF THAT CLAIM, stated precisely because the loose version of
    /// it travelled (Thor, 2026-08-03): the sentence above is about THIS
    /// test, not about the suite. On the pre-change tree that mutation
    /// gave **144 passed, 1 failed** - `spear_followthrough_carries_past_
    /// the_release_then_settles` did catch it. The rewrite took detection
    /// from ONE test to THREE and moved it onto the function's own
    /// contract; it did not rescue the bug from zero coverage, and nobody
    /// should repeat it as though it had.
    ///
    /// Now it calls the real function. `spear_followthrough_yaw_from`
    /// takes the tip's two table rows as parameters, so the test can feed
    /// rows the consts do not contain - which is what makes "invariant to
    /// the tables" a statement the code can actually violate.
    ///
    /// FALSIFIABILITY: drop `+ tip_onset` and the (0.0, 1.0) variant
    /// diverges from the (0.500, 5.222) variant by ~0.39 rad at small
    /// `release_t`. Drop `/ tip_peak` and the peak variants diverge by
    /// ~0.3 rad. Both are ~5 orders over the tolerance.
    ///
    /// NOT BIT-IDENTICAL. The spec's Step 1 test table specified `==`;
    /// that is false in f32, because `(t + onset) - onset` and
    /// `peak * x / peak` each round. Measured worst divergence across
    /// these six variants over 0..0.6 s is 2.98e-8 rad - real, tiny, and
    /// **34x** inside the tolerance below (1e-6 / 2.98e-8 = 33.6). That
    /// is the margin for the 1 ms grid THIS test sweeps; refine the grid
    /// and it drops - 5.96e-8, 17x, on a 1 us grid. Before touching the
    /// tolerance or the step, read the UNITS paragraph in
    /// `spear_followthrough_yaw`'s doc block: the "5.6x" that used to be
    /// quoted for this same quantity was a drive-term residual divided by
    /// a yaw tolerance, and it is gone.
    ///
    /// Invariance alone is vacuous (a function returning 0.0 is invariant
    /// to everything). `spear_followthrough_matches_its_hand_computed_curve`
    /// is the other half: it pins the curve itself to numbers derived
    /// outside this file.
    #[test]
    fn spear_followthrough_is_invariant_to_the_chain_tables() {
        let variants: [(f32, f32); 6] = [
            (CHAIN_ONSET_OFFSETS[7], CHAIN_PEAK_SCALE[7]), // shipped
            (0.125, CHAIN_PEAK_SCALE[7]),                  // the pre-BRIEF_VIII_B onset
            (0.0, 1.0),                                    // no chain offset at all
            (0.500, 5.222),                                // ~4x onset, 2x peak
            (0.001, 0.25),                                 // a peak BELOW 1.0
            (0.250, 100.0),                                // absurd peak
        ];
        for step in 0..=600 {
            let release_t = step as f32 * 0.001;
            let base = spear_followthrough_yaw_from(release_t, variants[0].0, variants[0].1);
            // the shipped wrapper must BE the shipped-table variant, bit
            // for bit - same inputs, same arithmetic, no excuse to differ
            assert_eq!(
                spear_followthrough_yaw(release_t).to_bits(),
                base.to_bits(),
                "at release_t={release_t}: the public wrapper is not the \
                 parameterised function at the shipped table rows"
            );
            for (onset, peak) in &variants[1..] {
                let got = spear_followthrough_yaw_from(release_t, *onset, *peak);
                assert!(
                    (got - base).abs() < 1e-6,
                    "at release_t={release_t} with (onset {onset}, peak {peak}): \
                     the tables did NOT cancel (shipped {base}, substituted {got}) \
                     - the zero-risk argument for retuning the chain no longer holds"
                );
            }
        }
    }

    /// The independent half of D6's fix, and the reason the invariance
    /// test above is not vacuous.
    ///
    /// `RAMP_S`, `OVERSHOOT_RAD`, `HOLD_S` and `SETTLE_RATE` are function-
    /// local consts, so this test CANNOT reference them. It therefore has
    /// to carry the spec in some form, and the form matters enormously.
    ///
    /// **F1 (Thor, 2026-08-03) - THE HAZARD THIS STRUCTURE EXISTS TO
    /// KILL.** What stood here was seven literal output values with a
    /// comment saying they had been computed outside the crate. Nothing
    /// ENFORCED that. The cheapest move available to the next maintainer
    /// who breaks this curve is to run the code, paste its output over the
    /// seven literals, and watch the test go green - at which point the
    /// test has silently become a change-detector that pins the bug in
    /// place. That is D6's defect class exactly: a test named after the
    /// theorem that no longer tests it. A comment cannot prevent it,
    /// because the comment is the thing being ignored.
    ///
    /// So the curve is now pinned by a TRIANGLE, and every side is checked:
    ///
    /// 1. `closed_form` - the spec as an expression, in f64, containing
    ///    **no crate item whatsoever**: not `SPEAR_RELEASE_YAW`, not the
    ///    chain tables, not the local consts. Literally
    ///    `(0.35 + 0.10*min(t/0.12, 1)) * exp(-6*max(t - 0.05, 0))`.
    /// 2. `ANCHORS` - seven f64 values at 15 decimal places, computed by
    ///    evaluating that same expression outside this crate.
    ///    Asserted against `closed_form` at **1e-12**.
    /// 3. `spear_followthrough_yaw` - the real, shipped f32 function,
    ///    swept against `closed_form` on a 1 ms grid over 0..1.2 s, and
    ///    checked against `ANCHORS` directly, both at 1e-6.
    ///
    /// **Why regeneration-from-code now fails loudly.** The f32 function
    /// and the f64 closed form agree only to ~4e-8. So anchors pasted from
    /// this crate's own output miss the 1e-12 gate in step 2 by a factor
    /// of **41,697** - measured, not estimated - and the failure message
    /// says so by name. There is no table of numbers anywhere in this test
    /// that can be regenerated from the code to silence a real failure:
    /// break the function and step 3 fails; paste the broken output into
    /// `ANCHORS` and step 2 fails as well. The only way through is to edit
    /// `closed_form` itself, which is visibly editing the specification
    /// rather than refreshing a table - the distinction the old shape
    /// could not make. (The sibling
    /// `the_arm_onsets_reproduce_an_independently_solved_geometric_root`
    /// already worked this way, deriving its expectation in-test; this is
    /// that pattern applied here, with the external anchors kept on top.)
    ///
    /// MEASURED MARGINS (F4, Thor, 2026-08-03 - this block previously said
    /// "worst gap 5.5e-8, ~18x", and both figures were stale):
    ///
    /// - real f32 fn vs `closed_form`, 1 ms sweep 0..1.2 s: worst
    ///   **6.00e-8 rad** at t = 0.115, so 1e-6 leaves **16.7x**.
    /// - real f32 fn vs `ANCHORS` at the seven points: worst **4.17e-8**
    ///   at t = 0.06, i.e. 24x. (Against the OLD 7-significant-digit f32
    ///   literals it was 5.96e-8 / **16.8x** - Thor's figure, confirmed to
    ///   the digit. Carrying the anchors at f64 precision recovers the
    ///   rounding those literals threw away.)
    ///
    /// FALSIFIABILITY: drop `+ tip_onset` from `spear_followthrough_yaw_from`
    /// and t=0.03 returns 0.350 instead of 0.375 (the drive is silent until
    /// t >= 0.130) - 2.5e-2 off, 25000x the tolerance. That single mutation
    /// is the bug in AUDIT.md #1, and it is what the pre-D6 test could not
    /// see. Any retune of the four local consts, or of `SPEAR_RELEASE_YAW`,
    /// also fails here - deliberately. Retuning the feel means editing
    /// `closed_form` AND recomputing `ANCHORS` from it outside the crate,
    /// and having to do both, in that order, is the whole point.
    #[test]
    fn spear_followthrough_matches_its_hand_computed_curve() {
        // ---- 1. the spec, as an expression. NOTHING from this crate. ----
        // If you change a number in here you are changing the SPECIFICATION
        // of the follow-through, not fixing a test. `ANCHORS` below will
        // stop matching, and that is correct: recompute them from the new
        // expression OUTSIDE this crate before you touch them.
        fn closed_form(t: f64) -> f64 {
            let drive = (t / 0.12_f64).clamp(0.0, 1.0); //      RAMP_S
            let decay = (-6.0_f64 * (t - 0.05_f64).max(0.0)).exp(); // SETTLE_RATE, HOLD_S
            (0.35_f64 + 0.10_f64 * drive) * decay //  release yaw, OVERSHOOT_RAD
        }

        // ---- 2. the external anchors. NOT REGENERABLE FROM THIS CRATE. ----
        // f64, 15 decimal places, from the expression above. The shipped
        // f32 function cannot produce these: at t=0.06 it returns
        // 0.376705855131, which differs in the 8th decimal. If a diff ever
        // shows this table moving to values like that, someone pasted the
        // code's output in - the assert below is what catches it.
        const ANCHORS: [(f64, f64); 7] = [
            (0.00, 0.350_000_000_000_000), // starts exactly on the release yaw
            (0.03, 0.375_000_000_000_000), // drive 0.25, no decay yet
            (0.05, 0.391_666_666_666_667), // drive 5/12, last frame before the settle
            (0.06, 0.376_705_813_433_699), // drive 0.50, decay exp(-0.06)
            (0.12, 0.295_671_068_916_776), // drive saturated, decay exp(-0.42)
            (0.30, 0.100_408_572_066_793), // decay exp(-1.5)
            (1.00, 0.001_505_684_455_862), // decay exp(-5.7)
        ];
        for (t, want) in ANCHORS {
            let derived = closed_form(t);
            assert!(
                (derived - want).abs() < 1e-12,
                "anchor at release_t={t} is {want}, but the closed form gives \
                 {derived} (gap {:.3e}). Either the closed form was edited \
                 without recomputing the anchors, or the anchors were \
                 REGENERATED FROM THIS CRATE'S OWN f32 OUTPUT - which is the \
                 one thing they must never be, and which shows up as a gap \
                 near 4e-8 rather than near 1e-16",
                (derived - want).abs()
            );
        }

        // ---- 3. the real, shipped function against the spec ----
        // dense enough that no feature of the curve hides between samples:
        // the ramp (0..0.12), the hold corner (0.05), the clamp corner
        // (0.12) and the long decay tail all get swept.
        for step in 0..=1200 {
            let t = step as f32 * 0.001;
            let got = spear_followthrough_yaw(t) as f64;
            let want = closed_form(t as f64);
            assert!(
                (got - want).abs() < 1e-6,
                "follow-through at release_t={t}: the closed form says {want}, \
                 the shipped function returns {got} (gap {:.3e})",
                (got - want).abs()
            );
        }

        // ---- and directly against the external anchors ----
        for (t, want) in ANCHORS {
            let got = spear_followthrough_yaw(t as f32) as f64;
            assert!(
                (got - want).abs() < 1e-6,
                "follow-through at release_t={t}: externally computed {want}, got {got}"
            );
        }
    }

    #[test]
    fn kinetic_chain_segment_is_silent_before_its_own_onset() {
        // the tip (segment 7) must show ZERO activation while only the
        // pelvis (segment 0) has begun - proximal-to-distal, not all-at-once
        let t_early = CHAIN_ONSET_OFFSETS[1] * 0.5; // between pelvis onset and lumbar onset
        assert!(chain_segment_scale(0, t_early, 0.05) > 0.0, "pelvis should already be moving");
        assert_eq!(chain_segment_scale(7, t_early, 0.05), 0.0, "tip must still be silent");
    }

    /// Task 3.3 real consumer test: `spear_followthrough_yaw` is the
    /// spear throw-release AND thrust-recovery curve, routed through
    /// `torso_coil_yaw`'s final branch.
    ///
    /// This test replaces an earlier one that asserted the follow-through
    /// was SILENT at release. That was encoding a bug, not a spec: the
    /// old curve sampled the chain's tip from zero, so it returned 0 for
    /// the first 0.125 s (a hard snap to neutral from the release angle)
    /// and then swung NEGATIVE - back toward the coil - which is the
    /// opposite of the "carries past" the docs promise.
    #[test]
    fn spear_followthrough_carries_past_the_release_then_settles() {
        // 1. nothing thrown yet = no twist at all (a fighter merely
        //    holding a spear must not be born mid-unwind)
        assert_eq!(spear_followthrough_yaw(-1.0), 0.0, "no release yet: silent");

        // 2. it BEGINS at the release yaw - no snap to neutral
        assert!(
            (spear_followthrough_yaw(0.0) - SPEAR_RELEASE_YAW).abs() < 1e-5,
            "follow-through must start exactly where the windup ended, got {}",
            spear_followthrough_yaw(0.0)
        );

        // 3. handoff continuity: the last windup frame and the first
        //    follow-through frame must be within a couple of degrees
        // the last frame of the plant is plant-frac ~1
        let last_windup =
            torso_coil_yaw(GunKind::Spear, Some(1.0 - DT / SPEAR_WINDUP_S), 0.0, false, -1.0, 0.0);
        let first_follow = torso_coil_yaw(GunKind::Spear, None, 0.0, false, 0.0, 0.0);
        assert!(
            (last_windup - first_follow).abs() < 0.09, // ~5 deg
            "release must not pop: windup ended at {last_windup}, follow-through starts at {first_follow}"
        );

        // 4. it CARRIES PAST the release angle (same sign, larger
        //    magnitude) rather than reversing through neutral
        let mut peak = 0.0_f32;
        for i in 0..400 {
            let y = spear_followthrough_yaw(i as f32 * 0.002);
            assert!(
                y >= -1e-4,
                "must never swing back the other way (that is the coil direction), got {y}"
            );
            peak = peak.max(y);
        }
        assert!(
            peak > SPEAR_RELEASE_YAW,
            "must carry PAST the release yaw {SPEAR_RELEASE_YAW}, peaked at only {peak}"
        );

        // 5. and it actually relaxes back to neutral
        assert!(
            spear_followthrough_yaw(1.5).abs() < 0.001,
            "must settle to neutral, not hold the carry forever"
        );
    }
}

#[cfg(test)]
mod training_mode_tests {
    use crate::*;

    /// §2 (owner spec, 2026-08-10): "No settings menu, no selectable
    /// rules, no map/rules configuration, no setup screen."
    ///
    /// The property that statement reduces to, in code, is: the config
    /// training runs on does NOT depend on `Selected`. This test drives
    /// the same function with two maximally different setup screens and
    /// demands one answer - and then checks the OTHER direction, that a
    /// TDM still follows those choices, so a broken `match_config` that
    /// ignored `Selected` for everything could not pass it.
    #[test]
    fn training_ignores_every_setup_choice() {
        let plain = Selected::default();
        let fiddled = Selected {
            map: MapKind::Gardens,
            difficulty: Difficulty::Hard,
            tdm_target: 60,
            class: sim::Class::Marksman,
            loadout: [GunKind::Awm, GunKind::Deagle, GunKind::Bow],
            melee_axe: true,
            grenade_preset: 2,
            ..Selected::default()
        };
        let a = match_config(&plain, Mode::Training);
        let b = match_config(&fiddled, Mode::Training);
        let fixed = training_config();
        for (name, x, y, z) in [
            ("map", a.map as u8, b.map as u8, fixed.map as u8),
            (
                "difficulty",
                a.difficulty as u8,
                b.difficulty as u8,
                fixed.difficulty as u8,
            ),
            ("class", a.class as u8, b.class as u8, fixed.class as u8),
            ("mode", a.mode as u8, b.mode as u8, fixed.mode as u8),
        ] {
            assert_eq!(x, y, "{name} moved with the setup screen");
            assert_eq!(y, z, "{name} disagreed with the hardcoded scenario");
        }
        assert_eq!(b.tdm_target, fixed.tdm_target, "score target is not a training setting");
        assert_eq!(b.per_team, fixed.per_team, "battle size is not a training setting");
        assert_eq!(b.loadout, fixed.loadout, "the loadout is fixed on the range");
        assert!(!b.melee_axe, "the melee pick is not a training setting");
        assert_eq!(b.grenade_preset, 0, "the grenade budget is fixed");
        assert!(
            b.armor_pieces.is_none(),
            "the range issues the class default plate, not the Forge build"
        );

        // The control: everything above must be a statement about
        // TRAINING, not about `match_config` having stopped reading
        // `Selected` at all.
        let tdm = match_config(&fiddled, Mode::Tdm);
        assert_eq!(tdm.map as u8, MapKind::Gardens as u8);
        assert_eq!(tdm.tdm_target, 60);
        assert!(tdm.melee_axe);
        assert!(tdm.armor_pieces.is_some());
    }
}
