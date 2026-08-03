# Branding assets

Bevy resolves `asset_server.load("branding/x.png")` relative to the
crate's `assets/` directory. Drop the three brand files here with these
exact names — the code already references them:

| File | What it is | Used by |
|---|---|---|
| `key_art.png` | The wide arena key art (legionaries vs. mech) | Loading screen, full-bleed background |
| `wordmark.png` | "JOHN KINGDOM ARENA" lockup | Loading screen, under the key art |
| `emblem.png` | The gear-and-helm crest | The small persistent mark: HUD corner, pause, scoreboard, loading spinner |

**Missing files are non-fatal.** Bevy's `asset_server.load` returns a
handle regardless; an unresolved handle simply renders nothing, and the
loading screen falls back to its procedural colour background. The game
still launches, still plays, and every test still passes with this
directory empty. That is deliberate — the branding is presentation, and
presentation must never be able to stop the build.

Source images are 1920x1080 (key art), ~1500x760 (wordmark), and
1024x1024 (emblem). Any reasonable size works; the UI scales them.
