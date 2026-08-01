# Motion & Maneuvering — session 1 progress notes

**Not a decision.** `DECISION.md` per Section 4 of the prompt requires
14 core + 7 adjacent sources with ≥4P/≥3V; this session logged 5/14
core (all P, 0 V) and 0/7 adjacent. Writing the decision now would
mean writing it without having read a single talk with numbers on
per-character CPU cost at crowd scale — axis 5, which the brief calls
"decisive" — so it stays unwritten. This file is the honest checkpoint
the brief's own R10 asks for, nothing more.

## The one quick win worth flagging now (Section 4, closing clause)

`bevy_mod_inverse_kinematics` **0.8.0** is a real, version-matched,
MIT/Apache-2.0 crate for two-bone IK, confirmed via the crates.io
dependency API to require `bevy = "^0.15"` — this engine's exact
version [S-03]. `jk_tdm`/`jk_wall` have no IK crate today; the 20-segment
body's grip poses and reach are hand-posed. This is cheap to trial and
low-risk to revert (it's an additive plugin, not a rewrite) — noted per
the brief's instruction, **not started**, because Section 4 says stop
until the decision is made, and one crate is not the architecture
decision.

## The trap the brief warned about, independently confirmed

Fetched `orangeduck/Motion-Matching` (the reference implementation
behind Learned Motion Matching, DReCon, and Robust Motion In-betweening)
directly rather than trusting the brief's own claim. Its own docs name
the Ubisoft LaFAN1 dataset as the training data source, and LaFAN1 is
CC BY-NC-ND 4.0 [S-05]. Confirmed independently: the code is MIT and
readable for technique; the data path behind three of the anchor papers
is not shippable. This matches R3's worked example exactly and is now
verified from a primary source rather than taken on the brief's word.

## What's actually blocking quota completion

The V-tier (≥3 sources with timestamped talk quotes) is 0/3 across
BOTH this topic and all eight master-brief topics from the earlier
research pass this session — GDC Vault access and most conference-talk
video transcripts are not fetchable by the tools available here. This
is the same honest limitation recorded in `research/aiming/SOURCES.md`,
`research/traversal/SOURCES.md`, etc. — not new to this topic.

## Next session's queue (not started, per R10)

Papers: PFNN (Holden/Komura/Saito 2017), DReCon, Robust Motion
In-betweening, DeepMimic lineage — 4 more P-tier sources, would bring
core P count well past the ≥4 floor.
Technique-only: UE5 Motion Matching / Pose Search docs, For Honor GDC
talk (the latter is also the first real V-tier candidate if a
transcript is fetchable).
Adjacent (half quota, 0/7 today): active ragdoll/get-up, ORCA/RVO at
high density, Game AI Pro cover/peek/suppression material.
