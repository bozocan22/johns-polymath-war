# aiming — SOURCES

| ID | Tier | Type | Title | Author / Studio | Year | URL | Accessed | Status | What it gave us |
|---|---|---|---|---|---|---|---|---|---|
| S-01 | P/V | GDC talk | Techniques for Building Aim Assist in Console Shooters | Nick Weihs / Insomniac (Resistance 3) | 2013 | https://archive.org/details/GDC2013Weihs (Vault 1017942) | 2026-08-01 | SNIPPET-ONLY | Snippet names the three canonical assist systems — magnetism, centering, friction — plus camera acceleration and deadzoning. THE talk for this topic; NOT COUNTED until watched with timestamps. |

## Explicit negative result (carried from the master brief, re-confirmed)

Searches for aim response curves (linear / exponential / dynamic
reverse-S) and controller deadzone values return almost entirely
affiliate-SEO content quoting each other's numbers. No primary source
found. Every such page is tier X. No curve or deadzone number is carried
into this project until an engine doc, input-system source file, or a
talk provides one.

## Applied to this codebase

Mouse-only today: raw delta, no smoothing, no acceleration (asserted in
code comments and by the zoom-sensitivity monitor-distance match, which
now tracks the player's CHOSEN hip FOV). Aim assist is a controller
feature; nothing to wire until controller input exists. The three Weihs
mechanism names are recorded so the eventual implementation starts from
the canonical decomposition rather than folklore.

## Quota: 0/12 counted. HONESTLY EMPTY - this topic needs the video pass.
