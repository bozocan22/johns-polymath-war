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

---

# SESSION 2 — 2026-08-10 — the refusal above was overridden, and why that was right

**`DECISION.md` now exists.** Read it, not this file, for the decision.
This section is the working record: what was done, what was corrected,
and where the honest gaps are.

## The refusal was correct and its premise turned out to be wrong

Session 1 (above) refused to write the decision because axis 5 —
per-character CPU cost at crowd scale — had no evidence. That was the
right call **given what it had read.** It had read papers and crate
metadata. It had not read this repository.

Reading the source first changes the shape of the problem entirely:

1. **Families B and C are eliminated by the R5 licence/asset gate before
   axis 5 is ever consulted.** We hold zero hours of licence-cleared
   mocap and zero animation clips; LaFAN1 is CC BY-NC-ND 4.0. A
   per-character cost of 0 ms would not make an unavailable architecture
   available. **Axis 5 could only ever have decided between two
   available options, and there is only one.**
2. **The "crowd" in axis 5 is not animated.** `jk_wall` (the 250v250
   sim) has no rig, no pose layer, and no bevy dependency at all. The
   animated population is `jk_tdm`'s: 16 fighters (`per_team` clamped to
   8) + 40 zombies (`ZOMBIE_CAP`) = **56**. The number session 1 was
   blocked on was for a population that does not exist.
3. **The number is measurable here.** `jk_spike/src/bin/bench.rs` already
   walks a body-count ladder; `autoplay_report` already drives headless
   matches. `DECISION.md` §9 specifies BM-1 and BM-2 precisely enough to
   build. **Axis 5 stops being a research blocker and becomes a build
   task**, and the resulting figure is for this game on this hardware,
   which no talk could supply.

So: the gap session 1 named is still open as a *literature* question and
is marked as such in `DECISION.md` §2 (axis 5 declares **no winner**) and
§10.4. It is no longer load-bearing.

## The greps that did the work, recorded so nobody re-runs them blind

Against `engine/crates`, commit `e2866a9`:

- `AnimationPlayer|AnimationGraph|AnimationClip|AnimationNodeIndex|AnimationTransitions`
  → **4 hits, none in code** (two `Cargo.toml`, two research `.md`).
  The `bevy_animation` cargo feature is enabled and entirely unused.
- Glob `engine/crates/jk_tdm/assets/**` → **no files.** No clips, no glTF,
  no BVH. There is no motion content of any kind in this project.
- `foot_ik|ground_ik` in `jk_tdm/src` → **zero hits.** Legs are open-loop
  gait sinusoids (`main.rs:17759-17812`). **This is the one genuinely
  missing core-scope item** and `DECISION.md` §7 Step 2 closes it.
- `solve_arm_ik` → **8 call sites**, 6 on the fighter rig, 2 on the
  viewmodel, 1 in a test. Two-bone IK is already load-bearing here.
- Line counts, for whoever has to work in these files: `sim.rs` **27 737**
  lines, `main.rs` **29 261**. `DECISION.md` §7 Step 1 (extract the pure
  pose kernel into its own module) is partly motivated by that alone.

## The single sharpest finding, and it is not in any paper

`jk_tdm`'s sim classifies hits by **height fraction** (`HitZone`), and
the render is clamped to respect it — `gait_pose`'s own doc comment
(`main.rs:2300-2310`) records a real bug where a settle dip put the head
base at ~0.79 of height, "outside the 0.82 band the test claims to
guard, and classified as Arms by the sim while looking like a head."

**A pose retrieved from a motion database, or emitted by a network, does
not know about your hit bands.** It puts the head where the data put it.
Every frame of disagreement is a frame where the player shoots what he
sees and hits something else. Motion matching here would need a
constraint layer on top whose entire job is to undo the data. That is a
game-specific structural argument against family B that no amount of
reading SIGGRAPH would have produced, and it took one doc comment.

## Corrections to session 1

Both are written up in full in `SOURCES.md` §"Corrections to session 1":

1. **The `bevy_mod_inverse_kinematics` quick win is reversed.** Its
   licence and version facts stand (MIT/Apache-2.0, 0.8.0 → `bevy ^0.15`,
   confirmed via the crates.io API). Its premise does not: session 1
   wrote that the body's "grip poses and reach are hand-posed", and they
   are not — `solve_arm_ik` is a full two-bone solver with a pole vector,
   plus an elbow clamp and sprung targets the crate lacks. Rejected on
   axis 9, duplicate capability, `DECISION.md` §5.4, with the condition
   that would reopen it (chains longer than two bones — for which the
   crate would not help either, being two-bone only).
2. **This file's tier-V blocker paragraph is stale.** TOTO33 solved it on
   2026-08-08 (`youtube-transcript-api`, and Internet Archive
   `_djvu.txt` for Vault-gated decks). **It is nevertheless still 0 today
   for an unrelated reason:** this session's shell has no `curl`, no
   `wget`, no Python, no `git`, no coreutils — every probe exits 127.
   Only `WebSearch`/`WebFetch` reach the network, and the Learned Motion
   Matching PDF is over the 10 MiB fetch limit with no local-download
   route. **Tier-V reachability varies by session. Probe the shell
   before writing it off, and record which way it went.**

## What I would need to read next to close the gaps I left

1. **Nothing, to make this decision.** That is the point of §3 of
   `DECISION.md`. The R5 gate is decisive and it is already closed by a
   licence fact and an empty `assets/` directory. **Do not re-open this
   as a literature question.**
2. **BM-1, our own benchmark** (`DECISION.md` §9.2). This is the
   highest-value next artefact on the topic and it is a *build* task, not
   a read task. It closes axis 5 with a number that is better than any
   citation because it describes this engine.
3. **The Learned Motion Matching PDF** — only if risk (1) in
   `DECISION.md` §8.3 fires, i.e. if someone licences commercial mocap
   and family B comes back on the table. Needs a session with a shell
   that can download a 10 MiB file. Until then it is the classic
   over-valued unread source this log warned about after the Hatze pass.
4. **`bevy_animation_graph`'s compatibility table, the 0.15 row
   specifically** — the one unresolved factual question about a real,
   permissively-licensed, genuinely useful crate. Its partial-ragdoll
   feature (some bones simulated, some kinematic) is the most valuable
   third-party capability found on this topic, and it becomes relevant
   the day this project acquires a rigged character with clips.
5. **A talk on motion-matching search cost, via TOTO33's transcript
   route, from a session that has Python.** Explicitly *not* on the
   critical path — it would be a cross-check on an order of magnitude for
   a family we have rejected. Listed last on purpose.

**Note on the log:** the canonical entry belongs in
`research/TOTO_LOG.md`, but this dispatch restricted writes to
`research/motion-architecture/`, so it is recorded here instead. Whoever
lifts that restriction should copy this section across.

— session 2
