# Branding assets

Loaded by `src/branding.rs` — this crate's **first image-loading code**.
Every one of the 21 pre-existing `asset_server.load` calls was a `.wav`;
the renderer had no textures at all.

Paths resolve against this directory (`engine/assets/`), set by
`AssetPlugin.file_path` in `main.rs`. Note that is the **workspace**
assets dir, not the crate's — an easy mistake.

| File | What it is | Used by |
|---|---|---|
| `key_art.png` | Wide arena key art (legionaries vs. mech) | Splash, full-bleed |
| `wordmark.png` | "JOHN KINGDOM ARENA" lockup | Splash, centred |
| `emblem.png` | Gear-and-helm crest | Splash, and the persistent corner mark |

## Missing files are non-fatal, by design

Bevy's `asset_server.load` returns a `Handle` whether or not the file
exists; an unresolved handle draws nothing. With this directory empty
the splash still runs over its black backdrop (reading as an intentional
beat, not a broken screen), the game still reaches the menu, and every
test still passes. Branding is presentation and must never be able to
stop the build.
