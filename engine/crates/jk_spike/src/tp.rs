//! Third-person software renderer — an over-the-shoulder perspective view of
//! the battle, following the player-controlled soldier. No GPU: pinhole
//! projection + painter's algorithm, good enough to prove the camera and the
//! feel; the real-time client (jk_client) renders the same scene with GL.

use image::{Rgba, RgbaImage};
use jk_wall::{Side, WallSim};

pub const W: u32 = 420;
pub const H: u32 = 280;
const FOCAL: f32 = 300.0; // px; ~70° horizontal FOV at 420 px
const NEAR: f32 = 0.3;

pub struct Camera {
    pub eye: [f32; 3],
    pub yaw: f32, // 0 looks along +z
}

impl Camera {
    /// Spring-damped chase camera behind the player (research/05: decouple
    /// the camera from body jitter; the crowd shoves the body, not the view).
    pub fn follow(&mut self, target: [f32; 3], yaw: f32, dt: f32) {
        let stiffness = 4.0;
        let desired = [
            target[0] - yaw.sin() * 3.8,
            target[1] + 3.8,
            target[2] - yaw.cos() * 3.8,
        ];
        for i in 0..3 {
            self.eye[i] += (desired[i] - self.eye[i]) * (stiffness * dt).min(1.0);
        }
        self.yaw = yaw;
    }

    /// World → screen. Returns (px, py, view_depth) if in front of the lens.
    pub fn project(&self, p: [f32; 3]) -> Option<(f32, f32, f32)> {
        let (dx, dy, dz) = (p[0] - self.eye[0], p[1] - self.eye[1], p[2] - self.eye[2]);
        let (s, c) = (self.yaw.sin(), self.yaw.cos());
        // view space: z' forward, x' right
        let vx = dx * c - dz * s;
        let vz = dx * s + dz * c;
        let vy = dy;
        if vz < NEAR {
            return None;
        }
        let px = W as f32 / 2.0 + FOCAL * vx / vz;
        // vy is CAMERA-relative; +1.8 pitches the view down so a point 1.0 m
        // above the ground (vy = -1.8 from a 2.8 m eye) hits screen center.
        let py = H as f32 * 0.45 - FOCAL * (vy + 1.8) / vz;
        Some((px, py, vz))
    }
}

fn fog(base: Rgba<u8>, depth: f32) -> Rgba<u8> {
    let f = (depth / 55.0).clamp(0.0, 0.85);
    let sky = [155.0, 165.0, 185.0];
    Rgba([
        (base[0] as f32 * (1.0 - f) + sky[0] * f) as u8,
        (base[1] as f32 * (1.0 - f) + sky[1] * f) as u8,
        (base[2] as f32 * (1.0 - f) + sky[2] * f) as u8,
        255,
    ])
}

fn vline(img: &mut RgbaImage, x: i32, y0: f32, y1: f32, c: Rgba<u8>) {
    if x < 0 || x as u32 >= W {
        return;
    }
    let (a, b) = (y0.min(y1).max(0.0) as u32, y0.max(y1).min(H as f32 - 1.0) as u32);
    for y in a..=b.min(H - 1) {
        img.put_pixel(x as u32, y, c);
    }
}

pub fn render(sim: &WallSim, cam: &Camera) -> RgbaImage {
    render_opts(sim, cam, true)
}

pub fn render_opts(sim: &WallSim, cam: &Camera, draw_shields: bool) -> RgbaImage {
    let mut img = RgbaImage::new(W, H);
    // sky gradient + ground
    let horizon = (H as f32 * 0.45) as u32;
    for y in 0..H {
        let c = if y < horizon {
            let t = y as f32 / horizon as f32;
            Rgba([
                (110.0 + 45.0 * t) as u8,
                (125.0 + 40.0 * t) as u8,
                (150.0 + 35.0 * t) as u8,
                255,
            ])
        } else {
            let t = (y - horizon) as f32 / (H - horizon) as f32;
            Rgba([
                (72.0 + 18.0 * t) as u8,
                (78.0 + 22.0 * t) as u8,
                (58.0 + 14.0 * t) as u8,
                255,
            ])
        };
        for x in 0..W {
            img.put_pixel(x, y, c);
        }
    }
    // ground grid (battle-axis stripes every 2 m for motion parallax)
    for gz in -40..=40 {
        let z = gz as f32 * 2.0;
        for gx in -40..400 {
            let x = gx as f32 * 0.1 - 12.0;
            if let Some((px, py, d)) = cam.project([x, 0.0, z]) {
                if py >= horizon as f32 && py < H as f32 && px >= 0.0 && px < W as f32 {
                    let c = fog(Rgba([60, 66, 48, 255]), d);
                    img.put_pixel(px as u32, py as u32, c);
                }
            }
        }
    }

    // painter's sort over ITEMS (torsos and shields separately — a shield on
    // the far side of its owner must not overdraw him).
    #[derive(Clone, Copy)]
    enum Item {
        Torso(usize),
        Shield(usize),
    }
    // Occlusion cull (research/05, Game AI Pro ch.47 pattern): bodies between
    // the lens and the player are not drawn — the crowd must never wall off
    // the view. The player is exempt.
    let occlusion_depth = sim
        .player
        .and_then(|p| {
            let (x, z) = sim.position(&sim.agents[p]);
            cam.project([x, 0.9, z]).map(|(_, _, d)| d - 0.25)
        })
        .unwrap_or(0.0);

    let mut items: Vec<(Item, f32)> = Vec::with_capacity(sim.agents.len() * 2);
    for (i, a) in sim.agents.iter().enumerate() {
        let is_player = sim.player == Some(i);
        let (x, z) = sim.position(a);
        if let Some((_, _, d)) = cam.project([x, 0.9, z]) {
            if is_player || d >= occlusion_depth {
                items.push((Item::Torso(i), d));
            }
        }
        if draw_shields && !a.downed {
            let sz = z + a.side.forward_sign() * 0.34;
            if let Some((_, _, d)) = cam.project([x, 0.9, sz]) {
                if is_player || d >= occlusion_depth {
                    items.push((Item::Shield(i), d));
                }
            }
        }
    }
    items.sort_by(|p, q| q.1.partial_cmp(&p.1).unwrap());

    for (item, _) in items {
        match item {
            Item::Torso(i) => {
                let a = &sim.agents[i];
                let (x, z) = sim.position(a);
                let is_player = sim.player == Some(i);
                if a.downed {
                    if let Some((px, py, d)) = cam.project([x, 0.12, z]) {
                        let r = (FOCAL * 0.34 / d) as i32;
                        let c = fog(Rgba([52, 50, 48, 255]), d);
                        for dx in -r..=r {
                            vline(
                                &mut img,
                                px as i32 + dx,
                                py - (r as f32 * 0.3),
                                py + (r as f32 * 0.3),
                                c,
                            );
                        }
                    }
                    continue;
                }
                let base = match (is_player, a.side, a.routing) {
                    (true, _, _) => Rgba([235, 190, 70, 255]),
                    (_, Side::A, false) => Rgba([85, 125, 215, 255]),
                    (_, Side::B, false) => Rgba([200, 75, 65, 255]),
                    // fear has a color: drained, running
                    (_, Side::A, true) => Rgba([120, 130, 160, 255]),
                    (_, Side::B, true) => Rgba([160, 120, 110, 255]),
                };
                let foot = cam.project([x, 0.0, z]);
                let head = cam.project([x, 1.72, z]);
                if let (Some((fx, fy, d)), Some((_, hy, _))) = (foot, head) {
                    let half_w = FOCAL * 0.22 / d;
                    let c = fog(base, d);
                    for dx in (-half_w as i32)..=(half_w as i32) {
                        vline(&mut img, fx as i32 + dx, hy + half_w, fy, c);
                    }
                    let skin = fog(Rgba([205, 170, 140, 255]), d);
                    let hr = (half_w * 0.38).max(1.0);
                    for dx in (-hr as i32)..=(hr as i32) {
                        vline(&mut img, fx as i32 + dx, hy - hr, hy + hr, skin);
                    }
                    if is_player {
                        let mc = Rgba([255, 235, 140, 255]);
                        for dx in -1..=1 {
                            vline(&mut img, fx as i32 + dx, hy - 8.0, hy - 3.0, mc);
                        }
                    }
                }
            }
            Item::Shield(i) => {
                let a = &sim.agents[i];
                let (x, z) = sim.position(a);
                let fwd = a.side.forward_sign();
                let sz = z + fwd * 0.34;
                let s_l = cam.project([x - 0.45, 0.35, sz]);
                let s_r = cam.project([x + 0.45, 0.35, sz]);
                let s_t = cam.project([x, 1.35, sz]);
                if let (Some((lx, _, ds)), Some((rx, _, _)), Some((_, ty, _))) =
                    (s_l, s_r, s_t)
                {
                    let sb = cam
                        .project([x, 0.35, sz])
                        .map(|(_, y, _)| y)
                        .unwrap_or(ty + 40.0);
                    let shield_c = fog(
                        match a.side {
                            Side::A => Rgba([150, 175, 230, 255]),
                            Side::B => Rgba([225, 150, 135, 255]),
                        },
                        ds,
                    );
                    let rim = fog(Rgba([40, 38, 34, 255]), ds);
                    let (x0, x1) = (lx.min(rx) as i32, lx.max(rx) as i32);
                    for xp in x0..=x1 {
                        let edge = xp == x0 || xp == x1;
                        vline(&mut img, xp, ty, sb, if edge { rim } else { shield_c });
                    }
                }
            }
        }
    }
    img
}
