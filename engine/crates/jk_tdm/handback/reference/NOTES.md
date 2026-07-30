# Reference notes — mech and body (Task 1)

## Honest tooling limitation

This session's tools can search the web (`WebSearch`) and fetch/summarize a
page's text content (`WebFetch`), but there is no image-download capability —
nothing that pulls actual image bytes to disk. The task asked for 12-20
committed image files; that specific deliverable is **not achievable** with
the tools available this session. What follows instead is real research
(sources linked below) distilled into the design principles that actually
drive Task 5's implementation decisions — the part of Task 1 that matters
for the rebuild, even without the image folder.

## Mech design research

**Wanzers (Front Mission)** — the closest genre match to "utilitarian
military walker." Structurally: one body + two arms + legs, no more, no
less — the silhouette reads as an assembled vehicle, not a humanoid. Military
variants are visually plainer than civilian ones; the design logic is
industrial standardization, not heroics.
[Front Mission Wanzer Concept, ArtStation](https://www.artstation.com/artwork/PXN8G4) ·
[Wanzers — Front Mission Wiki](https://frontmission.fandom.com/wiki/Wanzers)

**Reverse-joint ("chicken walker") legs** — this is a well-known trope
across mecha fiction (AT-ST, ED-209, Battletech's Locust/Catapult/King Crab).
The key finding worth carrying into implementation: **this leg shape is
understood by its own fan/design community as primarily an aesthetic choice**
— it reads as more animalistic/less anthropomorphic, not as a functional
biomechanical advantage. That's useful permission: don't over-engineer the
joint physics to justify the shape, just commit to the silhouette.
[Chicken Walker — TV Tropes](https://tvtropes.org/pmwiki/pmwiki.php/Main/ChickenWalker)

**Titanfall (Legion/Scorch), Chappie/District 9 exosuits** — not
independently searched this pass (time-boxed), but already well-understood
from training data: Titanfall's titans read as chunky, wide-footed, and
GROUNDED — mass communicated through silhouette width more than height.
Chappie/District 9 read as scrappy, visibly maintained real-world hardware —
asymmetric wear, exposed cabling, nothing pristine.

## Design principles carried into Task 5

1. **Silhouette over surface detail.** A wanzer/chicken-walker reads at a
   glance from its leg shape and body-arms-legs proportions alone. Get the
   proportions and stance right before touching palette.
2. **Reverse-joint legs are a commitment to a LOOK, not a simulation.** No
   need to justify them mechanically — commit to the shape.
3. **Military-grade = plainer, not gaudier.** Olive-drab/khaki over gunmetal-
   gray reads as "real hardware," which is the whole point of Task 5's
   palette change.
4. **Exposed mechanism at the knee and waist is what sells "machine" over
   "robot costume"** — this matches Task 5.4's explicit rule and is echoed
   by the chicken-walker research (the joint IS the visual interest, not a
   panel hiding it).

## Body/hand reference

Not independently re-researched this pass — Brief VII v2's Section 2 (hand/
arm craft pass) already did this work in the same session: joint-limit
ranges, DIP/PIP coupling, finger phalanx ratios were sourced from measured
human active-range-of-motion literature at that time (see that section's
implementation for the actual numeric sources — `ELBOW_FLEX_MIN/MAX_DEG`,
`DIP_PIP_COUPLING`, etc. in `main.rs`). Re-deriving the same reference twice
in one session would be wasted effort; this note exists so a future session
knows where that grounding already lives.

**Done-when honesty check:** images NOT committed (tooling limitation, stated
above). Design principles ARE captured and DO inform Task 5's implementation.
