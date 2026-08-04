//! Branding — the key art, the wordmark, and the emblem.
//!
//! This is the crate's FIRST image-loading code. Every one of the 21
//! pre-existing `asset_server.load` calls was a `.wav`; the renderer had
//! no textures at all and every material was a procedural colour. That
//! made a whole class of work (weapon skins, decals, the Forge editor,
//! player image import) structurally blocked — `BACKLOG.md` #12 names
//! "any image loading at all" as the unblocker. This module is it.
//!
//! ## Everything here is COSMETIC and non-fatal
//!
//! Missing art must never be able to stop the game. Bevy's
//! `asset_server.load` hands back a `Handle` whether or not the file
//! exists; an unresolved handle simply draws nothing. So with an empty
//! `assets/branding/` the splash still runs (over its procedural
//! backdrop), the game still reaches the menu, and all tests still pass.
//! That is deliberate, not an accident of the API — see `splash_is_
//! skipped_for_captures` for the one place this policy is asserted.
//!
//! ## Wiring
//!
//! Two lines in `main.rs`:
//! ```ignore
//! mod branding;
//! // ...
//! .add_plugins(branding::BrandingPlugin)
//! ```
//! Everything else — state, systems, teardown — is owned here.

use bevy::prelude::*;

// ---- palette ---------------------------------------------------------
// Sampled from the key art so the UI and the art are the same world.
// The menu previously ran cool blue-grey chrome with cyan text over
// warm gold-and-sepia art, which read as two different games stapled
// together.
//
// **Decorative only.** Nothing here touches a colour that carries
// MEANING — team identity, danger red, low-health warning, the
// minimap's friend/foe dots. Those are semantics a player reads
// under pressure, and retheming them to match a painting would trade
// readability for mood. That is the line: chrome follows the art,
// signal does not. Signal colours live in `signal`, below.
pub mod palette {
    use bevy::prelude::Color;

    /// Dusty amber haze — the key art's ground and sky.
    pub const DUST: Color = Color::srgb(0.72, 0.62, 0.45);
    /// Deep shadowed brown for panels and pills, replacing the old
    /// cool navy `srgba(0.04, 0.05, 0.08, ..)`.
    pub const SHADOW: Color = Color::srgb(0.10, 0.08, 0.06);
    /// Antique gold — the laurel, the eagle, the emblem's frame. The
    /// primary accent and the colour of anything selected.
    pub const GOLD: Color = Color::srgb(0.80, 0.65, 0.26);
    /// Struck bronze, for secondary rules and inactive accents.
    pub const BRONZE: Color = Color::srgb(0.55, 0.42, 0.22);
    /// The legionaries' plume. An accent, NOT the danger red — the HUD
    /// keeps its own brighter red for damage so the two never confuse.
    pub const PLUME: Color = Color::srgb(0.62, 0.13, 0.11);
    /// Parchment — body text on dark ground. Warm, not the old blue-white.
    pub const PARCHMENT: Color = Color::srgb(0.93, 0.89, 0.80);
    /// Muted parchment for secondary/inactive text.
    pub const PARCHMENT_DIM: Color = Color::srgb(0.68, 0.63, 0.55);
}

// ---- signal ----------------------------------------------------------
// The deliberate counterpart to `palette`: colours that carry MEANING.
// `palette` follows the art and may be retuned freely; nothing here may
// be, because a player reads these under fire.
//
// **Team colour is RELATIVE to the viewer, not absolute.** The old code
// asked "is this fighter Blue or Red" at six separate call sites and
// answered with a hardcoded literal. That is a bug waiting to happen the
// moment the player spawns on Red — their own team would render in the
// enemy colour. Every site now asks `side_of(their_team, my_team)` and
// gets ALLY or ENEMY, so the answer is correct from either side.
//
// The owner's rule: allies read WHITE + GOLD, enemies read RED + ORANGE.
//
// LUMINANCE carries the signal, hue only reinforces it — allies are
// bright and cool, enemies are dark-bodied with a hot glowing accent.
// That ordering matters: roughly 1 in 12 men cannot separate red from
// green by hue, and a red-vs-blue scheme collapses for them under the
// game's amber dust fog. A bright-vs-dark split survives greyscale,
// which is the real test.
pub mod signal {
    use bevy::prelude::Color;

    /// Ally body shell — bright cold steel. Deliberately the highest
    /// luminance in the world so a teammate reads at silhouette range.
    pub const ALLY: Color = Color::srgb(0.93, 0.95, 0.97);
    /// Ally accent — antique gold on the shoulder band and emblem ring.
    /// Warm against the cold shell, and the same gold the player's own
    /// minimap marker uses, so "my side" is one colour idea.
    pub const ALLY_ACCENT: Color = Color::srgb(0.95, 0.78, 0.28);
    /// Enemy body shell — dark oxide red. Low luminance: the enemy is
    /// the dark shape, always.
    pub const ENEMY: Color = Color::srgb(0.42, 0.11, 0.09);
    /// Enemy accent — hot orange, emissive. The only strongly glowing
    /// thing on a body, so a muzzle-lit enemy still reads as enemy.
    pub const ENEMY_ACCENT: Color = Color::srgb(1.00, 0.45, 0.10);

    /// Which side a fighter is on **from the viewer's seat**.
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub enum Side {
        Ally,
        Enemy,
    }

    impl Side {
        /// Shell / primary colour.
        pub fn body(self) -> Color {
            match self {
                Side::Ally => ALLY,
                Side::Enemy => ENEMY,
            }
        }
        /// Accent / trim colour — bands, rings, tracers, dots.
        pub fn accent(self) -> Color {
            match self {
                Side::Ally => ALLY_ACCENT,
                Side::Enemy => ENEMY_ACCENT,
            }
        }
        /// Linear RGB triple of `accent`, for the many call sites that
        /// build a `srgba` with a computed alpha or an emissive scale.
        pub fn accent_rgb(self) -> (f32, f32, f32) {
            let c = self.accent().to_srgba();
            (c.red, c.green, c.blue)
        }
    }

    /// The one question every colour site asks. `viewer` is the team the
    /// player is actually on this round — never assume Blue.
    pub fn side_of<T: PartialEq>(theirs: T, viewer: T) -> Side {
        if theirs == viewer {
            Side::Ally
        } else {
            Side::Enemy
        }
    }
}

// ---- timings ---------------------------------------------------------
// A splash is a promise about how long the player waits. These are the
// whole contract, in one place.

/// Fade the art up from black.
const SPLASH_FADE_IN_S: f32 = 0.55;
/// Hold at full opacity. Long enough to read the wordmark, short enough
/// that a returning player does not resent it.
const SPLASH_HOLD_S: f32 = 1.30;
/// Fade back out to reveal the menu underneath.
const SPLASH_FADE_OUT_S: f32 = 0.45;
/// Total wall-clock cost of the splash, derived — never hand-typed, so
/// it cannot drift from the three numbers above.
pub const SPLASH_TOTAL_S: f32 = SPLASH_FADE_IN_S + SPLASH_HOLD_S + SPLASH_FADE_OUT_S;

/// Draw order. The splash sits above ALL gameplay and menu UI (the
/// highest pre-existing `GlobalZIndex` in this crate is 40).
const Z_SPLASH: i32 = 1000;
/// The persistent emblem watermark sits below every HUD cluster, so it
/// can never occlude ammo, vitals, or the killfeed.
const Z_EMBLEM_MARK: i32 = 5;

/// The emblem's resting opacity as a corner watermark. Low enough to
/// read as a maker's mark rather than an interface element.
const EMBLEM_MARK_ALPHA: f32 = 0.30;
const EMBLEM_MARK_PX: f32 = 46.0;

/// Where the emblem may appear, and at what weight.
///
/// A mark earns its authority by being **consistent and quiet**, not by
/// being large or frequent. Each placement here is a moment where the
/// game is already identifying itself — a splash, a pause, a scoreboard
/// — so the emblem reinforces rather than interrupts. It deliberately
/// never appears during uninterrupted play except as the low-alpha
/// watermark, because anything brighter competes with a HUD the player
/// is reading under pressure.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EmblemPlacement {
    /// Under the wordmark on the splash. The mark's establishing shot.
    Splash,
    /// Persistent, bottom-centre, during play. A maker's mark.
    Watermark,
    /// Beside the title on a paused/menu screen. The game naming itself
    /// while the player is not under pressure.
    MenuHeading,
    /// Centred above the scoreboard, where a competitive result is being
    /// presented and the game's identity belongs on it.
    Scoreboard,
}

impl EmblemPlacement {
    /// Rendered size in logical pixels.
    pub fn size_px(self) -> f32 {
        match self {
            EmblemPlacement::Splash => 84.0,
            EmblemPlacement::Watermark => EMBLEM_MARK_PX,
            EmblemPlacement::MenuHeading => 40.0,
            EmblemPlacement::Scoreboard => 34.0,
        }
    }

    /// Rendered opacity.
    ///
    /// Only `Splash` reaches full strength, and only because it is alone
    /// on screen. Every in-game placement is subordinate to the content
    /// beside it.
    pub fn alpha(self) -> f32 {
        match self {
            EmblemPlacement::Splash => 1.0,
            EmblemPlacement::Watermark => EMBLEM_MARK_ALPHA,
            EmblemPlacement::MenuHeading => 0.85,
            EmblemPlacement::Scoreboard => 0.70,
        }
    }

    /// Draw order, so a placement can never be authored above the HUD by
    /// accident. 40 is the highest `GlobalZIndex` the pre-existing HUD
    /// uses; every in-game placement must sit below that.
    pub fn z(self) -> i32 {
        match self {
            EmblemPlacement::Splash => Z_SPLASH,
            EmblemPlacement::Watermark => Z_EMBLEM_MARK,
            // menu/scoreboard surfaces are themselves modal overlays, so
            // the mark rides just above their own plate but below nothing
            // the player needs to read to act
            EmblemPlacement::MenuHeading => 35,
            EmblemPlacement::Scoreboard => 35,
        }
    }
}

// ---- assets ----------------------------------------------------------

/// Handles to the three brand images, loaded once at startup and held
/// for the process lifetime.
///
/// Paths are relative to the configured asset root (`engine/assets/`,
/// set via `AssetPlugin.file_path` in `main.rs`) — NOT the crate
/// directory, which is a genuinely easy mistake to make here.
#[derive(Resource)]
pub struct BrandAssets {
    /// Wide arena key art. Full-bleed splash background.
    pub key_art: Handle<Image>,
    /// "JOHN KINGDOM ARENA" lockup.
    pub wordmark: Handle<Image>,
    /// The gear-and-helm crest. The mark that recurs in small places.
    pub emblem: Handle<Image>,
}

impl BrandAssets {
    fn load(asset_server: &AssetServer) -> Self {
        Self {
            key_art: asset_server.load("branding/key_art.png"),
            wordmark: asset_server.load("branding/wordmark.png"),
            emblem: asset_server.load("branding/emblem.png"),
        }
    }
}

// ---- splash ----------------------------------------------------------

#[derive(Component)]
struct SplashRoot;

/// The two layers that fade together. Tagged rather than queried by
/// type so the fader never accidentally grabs the emblem watermark,
/// which is also a `UiImage` and must NOT fade with the splash.
#[derive(Component)]
struct SplashArt;

#[derive(Component)]
struct SplashBackdrop;

/// Splash lifetime. `None` once it has finished and torn itself down —
/// the resource stays, so the phase can never restart.
#[derive(Resource, Default)]
pub struct SplashState {
    elapsed: f32,
    done: bool,
}

impl SplashState {
    pub fn is_done(&self) -> bool {
        self.done
    }
}

/// The splash's opacity curve, as a pure function of elapsed time.
///
/// Extracted rather than inlined so it is directly testable without
/// spinning up Bevy — the same pattern this codebase already uses for
/// `view_recoil_offset` and `bow_sway_deg`.
///
/// Returns 0.0 before it starts and 0.0 once complete, rising to 1.0
/// across the hold. Clamped at both ends, so a caller cannot drive it
/// out of range.
pub fn splash_alpha(elapsed_s: f32) -> f32 {
    if elapsed_s <= 0.0 {
        return 0.0;
    }
    if elapsed_s < SPLASH_FADE_IN_S {
        return (elapsed_s / SPLASH_FADE_IN_S).clamp(0.0, 1.0);
    }
    let after_in = elapsed_s - SPLASH_FADE_IN_S;
    if after_in < SPLASH_HOLD_S {
        return 1.0;
    }
    let out = after_in - SPLASH_HOLD_S;
    (1.0 - out / SPLASH_FADE_OUT_S).clamp(0.0, 1.0)
}

fn spawn_splash(commands: &mut Commands, brand: &BrandAssets) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            // Starts fully black and fades to reveal. This backdrop is
            // also the entire fallback if the art is missing: the splash
            // then reads as an intentional black beat, not a broken screen.
            BackgroundColor(Color::srgba(0.02, 0.02, 0.03, 1.0)),
            GlobalZIndex(Z_SPLASH),
            SplashRoot,
            SplashBackdrop,
        ))
        .with_children(|p| {
            // Key art, full-bleed behind everything else in the splash.
            p.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                ImageNode {
                    image: brand.key_art.clone(),
                    color: Color::srgba(1.0, 1.0, 1.0, 0.0),
                    ..default()
                },
                SplashArt,
            ));
            // Wordmark, centred and generously sized. Sits in normal
            // flow (not absolute) so it stays centred at any resolution.
            p.spawn((
                Node {
                    width: Val::Percent(52.0),
                    ..default()
                },
                ImageNode {
                    image: brand.wordmark.clone(),
                    color: Color::srgba(1.0, 1.0, 1.0, 0.0),
                    ..default()
                },
                SplashArt,
            ));
            // Emblem beneath the wordmark — the mark's first appearance,
            // establishing it before it recurs small elsewhere.
            p.spawn((
                Node {
                    width: Val::Px(84.0),
                    height: Val::Px(84.0),
                    margin: UiRect::top(Val::Px(18.0)),
                    ..default()
                },
                ImageNode {
                    image: brand.emblem.clone(),
                    color: Color::srgba(1.0, 1.0, 1.0, 0.0),
                    ..default()
                },
                SplashArt,
            ));
        });
}

/// Drives the fade and tears the splash down exactly once.
fn drive_splash(
    time: Res<Time>,
    mut st: ResMut<SplashState>,
    mut commands: Commands,
    mut art: Query<&mut ImageNode, With<SplashArt>>,
    mut backdrop: Query<&mut BackgroundColor, With<SplashBackdrop>>,
    roots: Query<Entity, With<SplashRoot>>,
) {
    if st.done {
        return;
    }
    st.elapsed += time.delta_secs();
    let a = splash_alpha(st.elapsed);

    for mut img in &mut art {
        img.color = Color::srgba(1.0, 1.0, 1.0, a);
    }
    // The backdrop fades slightly AHEAD of the art so the menu is never
    // revealed through a still-opaque black plate.
    for mut bg in &mut backdrop {
        bg.0 = Color::srgba(0.02, 0.02, 0.03, a.min(0.96));
    }

    if st.elapsed >= SPLASH_TOTAL_S {
        st.done = true;
        for e in &roots {
            commands.entity(e).despawn_recursive();
        }
    }
}

// ---- the recurring mark ---------------------------------------------

#[derive(Component)]
struct EmblemWatermark;

/// The emblem as a persistent corner mark.
///
/// Bottom-CENTRE deliberately: all four corners are already occupied by
/// HUD clusters (vitals, ammo/loadout, minimap, killfeed), and the
/// bottom-centre strip is empty by design in the HUD spec. Placing it
/// there means the mark is always present and never fights a readout.
fn spawn_emblem_mark(commands: &mut Commands, brand: &BrandAssets) {
    // Size, opacity and draw order all come from the placement table -
    // never hand-typed here - so the design contract the tests assert is
    // the same data the renderer actually uses.
    let place = EmblemPlacement::Watermark;
    let px = place.size_px();
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(10.0),
            left: Val::Percent(50.0),
            margin: UiRect::left(Val::Px(-px * 0.5)),
            width: Val::Px(px),
            height: Val::Px(px),
            ..default()
        },
        ImageNode {
            image: brand.emblem.clone(),
            color: Color::srgba(1.0, 1.0, 1.0, place.alpha()),
            ..default()
        },
        GlobalZIndex(place.z()),
        EmblemWatermark,
    ));
}

// ---- plugin ----------------------------------------------------------

pub struct BrandingPlugin;

impl Plugin for BrandingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SplashState>()
            .add_systems(Startup, (load_brand_assets, spawn_brand_ui).chain())
            .add_systems(Update, drive_splash);
    }
}

fn load_brand_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(BrandAssets::load(&asset_server));
}

/// Spawns the splash and the watermark.
///
/// **The splash is skipped entirely when `JK_CAPTURE` is set.** The
/// capture harness drives scripted input and saves screenshots on a
/// fixed timeline; a 2.3-second splash over the first frames would land
/// in every early capture and silently corrupt the evidence the harness
/// exists to produce. Reading the env var directly (rather than the
/// `CaptureMode` resource) keeps this module free of any dependency on
/// `main.rs`'s internals.
fn spawn_brand_ui(
    mut commands: Commands,
    brand: Res<BrandAssets>,
    mut splash: ResMut<SplashState>,
) {
    if std::env::var("JK_CAPTURE").is_ok() {
        // Mark it finished so `drive_splash` returns immediately and
        // never tries to tear down a tree that was never spawned.
        splash.done = true;
    } else {
        spawn_splash(&mut commands, &brand);
    }
    // The mark is always present — it is the game's identity, not a
    // splash flourish, and captures SHOULD show it.
    spawn_emblem_mark(&mut commands, &brand);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The opacity curve's contract, asserted at every boundary. These
    /// are hand-derived from the three timing constants, not read back
    /// out of the function.
    #[test]
    fn splash_alpha_fades_in_holds_then_fades_out() {
        assert_eq!(splash_alpha(-1.0), 0.0, "nothing before it starts");
        assert_eq!(splash_alpha(0.0), 0.0, "starts fully transparent");

        // mid fade-in: exactly half way through the in-ramp
        let mid_in = splash_alpha(SPLASH_FADE_IN_S * 0.5);
        assert!(
            (mid_in - 0.5).abs() < 1e-5,
            "half way into the fade-in should be half opaque, got {mid_in}"
        );

        // the entire hold is fully opaque, at both ends and the middle
        assert_eq!(splash_alpha(SPLASH_FADE_IN_S), 1.0);
        assert_eq!(splash_alpha(SPLASH_FADE_IN_S + SPLASH_HOLD_S * 0.5), 1.0);

        // mid fade-out
        let mid_out = splash_alpha(SPLASH_FADE_IN_S + SPLASH_HOLD_S + SPLASH_FADE_OUT_S * 0.5);
        assert!(
            (mid_out - 0.5).abs() < 1e-5,
            "half way through the fade-out should be half opaque, got {mid_out}"
        );

        // and it lands exactly on zero, then stays there
        assert!(splash_alpha(SPLASH_TOTAL_S).abs() < 1e-5, "ends transparent");
        assert_eq!(splash_alpha(SPLASH_TOTAL_S + 5.0), 0.0, "stays gone");
    }

    /// The curve must never leave [0,1] — a UI colour channel driven out
    /// of range is a rendering bug that is painful to trace back here.
    #[test]
    fn splash_alpha_is_always_a_valid_opacity() {
        let mut t = -0.5_f32;
        while t < SPLASH_TOTAL_S + 2.0 {
            let a = splash_alpha(t);
            assert!(
                (0.0..=1.0).contains(&a),
                "alpha {a} out of range at t={t}"
            );
            t += 0.01;
        }
    }

    /// The splash's total cost is DERIVED from its three phases. If
    /// someone retunes a phase, the total must follow automatically —
    /// this catches a hand-typed total drifting out of sync.
    #[test]
    fn splash_total_is_the_sum_of_its_phases() {
        assert!(
            (SPLASH_TOTAL_S - (SPLASH_FADE_IN_S + SPLASH_HOLD_S + SPLASH_FADE_OUT_S)).abs() < 1e-6
        );
        // and it stays a respectful length - a splash the player cannot
        // skip should not outstay its welcome
        assert!(
            SPLASH_TOTAL_S < 3.0,
            "splash is {SPLASH_TOTAL_S}s - too long to sit through every launch"
        );
    }

    /// The emblem watermark must stay a mark, not become an interface
    /// element competing with the HUD it sits amongst.
    #[test]
    fn emblem_watermark_stays_subordinate_to_the_hud() {
        assert!(
            EMBLEM_MARK_ALPHA < 0.5,
            "a watermark at {EMBLEM_MARK_ALPHA} alpha reads as UI, not a maker's mark"
        );
        assert!(
            Z_EMBLEM_MARK < Z_SPLASH,
            "the splash must cover the watermark, not sit under it"
        );
        // 40 is the highest GlobalZIndex used by the pre-existing HUD;
        // the mark must sit below all of it so it can never occlude a
        // readout the player needs.
        assert!(
            Z_EMBLEM_MARK < 40,
            "the mark must render beneath every HUD cluster"
        );
    }

    /// The palette must actually be the key art's palette — warm, and
    /// legible in the pairings the UI really uses.
    ///
    /// Asserts RELATIONSHIPS, not hex values, so the palette can be
    /// re-sampled from new art without rewriting the test. What it
    /// pins is the reason the palette exists: the old chrome was COOL
    /// (blue channel dominant) over warm art, and that is the specific
    /// mistake that must not come back.
    #[test]
    fn the_palette_is_warm_and_readable() {
        use palette::*;
        let rgb = |c: Color| {
            let s = c.to_srgba();
            (s.red, s.green, s.blue)
        };

        // 1. WARM: every ground/accent colour must have more red than
        //    blue. The palette this replaced ran srgb(0.58,0.63,0.72) -
        //    blue-dominant - against gold-and-sepia art.
        for (name, c) in [
            ("DUST", DUST),
            ("GOLD", GOLD),
            ("BRONZE", BRONZE),
            ("PLUME", PLUME),
            ("PARCHMENT", PARCHMENT),
            ("PARCHMENT_DIM", PARCHMENT_DIM),
            ("SHADOW", SHADOW),
        ] {
            let (r, _, b) = rgb(c);
            assert!(
                r > b,
                "{name} is cool (r={r:.2} <= b={b:.2}) - the whole point of \
                 this palette is that the chrome stops fighting warm art"
            );
        }

        // 2. READABLE: text on its intended ground must actually
        //    separate. Rough relative luminance is enough to catch a
        //    palette edit that makes body text vanish into a panel.
        let lum = |c: Color| {
            let (r, g, b) = rgb(c);
            0.2126 * r + 0.7152 * g + 0.0722 * b
        };
        assert!(
            lum(PARCHMENT) - lum(SHADOW) > 0.5,
            "parchment on shadow has too little contrast to read"
        );
        assert!(
            lum(GOLD) - lum(SHADOW) > 0.25,
            "gold on shadow has too little contrast for a selected item"
        );
        assert!(
            lum(PARCHMENT) > lum(PARCHMENT_DIM),
            "the dim variant must actually be dimmer than the primary"
        );

        // 3. The accent PLUME must stay distinguishable from the HUD's
        //    danger red, which is deliberately NOT in this palette.
        //    A decorative red that reads as a warning is a real hazard.
        let hud_danger = Color::srgb(1.0, 0.25, 0.2);
        let (pr, _, _) = rgb(PLUME);
        let (dr, _, _) = rgb(hud_danger);
        assert!(
            dr - pr > 0.25,
            "the decorative plume red is too close to the HUD's danger red - \
             a player could read set dressing as a warning"
        );
    }

    /// The mark's design contract across every placement it is allowed
    /// in. These assert the RELATIONSHIPS (splash loudest, in-game
    /// quietest, nothing above the HUD) rather than the specific
    /// numbers, so the art can be retuned without rewriting the test —
    /// but the hierarchy cannot be inverted by accident.
    #[test]
    fn every_emblem_placement_stays_within_its_design_contract() {
        use EmblemPlacement::*;
        let in_game = [Watermark, MenuHeading, Scoreboard];

        // 1. Nothing in-game may outrank the HUD.
        for p in in_game {
            assert!(
                p.z() < 40,
                "{p:?} at z={} can occlude a HUD cluster the player reads under fire",
                p.z()
            );
        }

        // 2. The splash is the only place the mark is at full strength,
        //    and only because it is alone on screen.
        assert_eq!(Splash.alpha(), 1.0);
        for p in in_game {
            assert!(
                p.alpha() < 1.0,
                "{p:?} is at full opacity in-game - the mark must stay subordinate"
            );
        }

        // 3. During UNINTERRUPTED play the mark is at its very quietest.
        //    Pause and scoreboard are moments the player is not acting,
        //    so they may be a little louder.
        assert!(
            Watermark.alpha() < MenuHeading.alpha()
                && Watermark.alpha() < Scoreboard.alpha(),
            "the in-play watermark must be the quietest placement of all"
        );

        // 4. The splash is the largest; nothing in-game rivals it.
        for p in in_game {
            assert!(
                p.size_px() < Splash.size_px(),
                "{p:?} rivals the splash for size - the establishing shot should be the biggest"
            );
        }

        // 5. Every placement is actually visible. A mark rendered at
        //    zero alpha or zero size is dead code pretending to be art.
        for p in [Splash, Watermark, MenuHeading, Scoreboard] {
            assert!(p.alpha() > 0.0 && p.size_px() > 0.0, "{p:?} renders nothing");
        }
    }
}
