# Infantry-vs-mech: hull climbing — source ledger

R&D Cycle 3, backlog #2. R4 depth floor: ≥1 tier-P source read end to
end.

| ID | Tier | Type | Title | Author | Year | URL | Accessed | Status | What it gave us |
|---|---|---|---|---|---|---|---|---|---|
| S-01 | X | GDC talk | Postmortem: The Emotional Character Control of Shadow of the Colossus | Sugiyama, Hosono, Tanaka, Fukuyama / Team Ico | 2006 | https://gdcvault.com/play/1013376 (archive.org: https://archive.org/details/GDC2006Fukuyama) | 2026-08-01 | SNIPPET-ONLY | THE canonical reference for grip-climbing a giant enemy - each colossus has fur/ledge attachment points, the creature thrashes to dislodge the climber. Vault-gated; the archive.org mirror hosts only audio + a text overview, no transcript, no extractable numbers. Not counted, same GDC Vault limitation hit repeatedly this session. |
| S-02 | P | peer-reviewed (Frontiers in Sports and Active Living, 2022) | Acute Handgrip Fatigue and Forearm Girth in Recreational Sport Rock Climbers | MacDonald et al. | 2022 | https://pmc.ncbi.nlm.nih.gov/articles/PMC9362893/ | 2026-08-01 | READ (full text via PMC open access, verified against the primary page - direct quotes extracted, not a paraphrase-only summary) | n=10 intermediate climbers. Protocol: continuous top-rope ascents (5.9 YDS) for 30 min, rest only at the bottom between routes, average natural rest 78±3.3s. Result: dominant handgrip strength fell **22.1%**, non-dominant **23%**, over the 30 min session; forearm girth rose ~4.5% (swelling). Their own stated mechanism: **"repeated isometric contractions of the forearm... result in a reduction in blood flow and increased swelling"** - fatigue is cumulative and swelling-driven, not an instant cliff. |

## What this grounds in the design

Real climbing grip fatigue is **cumulative across a session, recovers
with real rest, and is driven by sustained isometric load** - not a
binary "can/can't grip" toggle and not a single long timer that empties
once. This argues for a **stamina pool that drains while gripping,
recovers only when NOT gripping** (matching the study's own rest-only
recovery pattern), rather than a fixed climb-duration timer independent
of how the player actually climbs (grip-release-grip should cost less
than one continuous hold of the same total duration, mirroring how the
study's climbers recovered between top-rope pitches).

Real-world absolute numbers (30-minute sessions, ~78s rests) are the
wrong TIMESCALE for a fast-paced arena shooter and are not copied
literally - what transfers is the SHAPE of the mechanism (cumulative
drain under hold, real rest to recover, diminishing grip under
fatigue), scaled to gameplay pacing in the design below.

Quota: 1/1 tier-P read (R4 depth floor). GDC talk stays SNIPPET-ONLY -
honest, not a blocker per R4 (no breadth quota applies).
