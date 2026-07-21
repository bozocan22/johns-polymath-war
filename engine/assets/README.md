# Assets — drop your Tripo3D (or Blender) exports here

## Characters (`assets/characters/`)

Export from Tripo3D as **GLB** (glTF binary) and name by role:

| File | Used for |
|---|---|
| `commander.glb` | the player character |
| `soldier_a.glb` | your side's soldiers (blue side) |
| `soldier_b.glb` | the enemy's soldiers (red side) |

Rules of thumb for exports that "just work":
- **GLB** format (single file, textures embedded).
- Model facing **+Z**, standing on the origin (feet at y = 0).
- Around **1.7–1.8 m tall** (the sim's men are 1.72 m). If your export is
  a different size, the client auto-scales it to man-height at load.
- Rigged + animated exports load too; animation playback lands in the
  next client milestone — today the model follows the physics body.

Missing files are fine: any role without a GLB falls back to the built-in
low-poly capsule men, so you can add characters one at a time.

Run with: `cargo run -p jk_bevy --release` (from `engine/`).
