# Motion & Maneuvering System — source ledger

Per `briefs/PROMPT_motion_system_research.md` R1-R7. Quota: 14 counted
core + 7 adjacent, ≥4 tier-P, ≥3 tier-V overall. This is session 1 of
what the brief itself estimates as a multi-pass effort (Section 3's
task list is Papers / Code / Unusable-source-technique / Adjacent,
each independently large) — what's below is real, everything else is
honestly not yet started, not silently skipped.

Status enum per R2: READ / SKIMMED / SNIPPET-ONLY / UNREACHABLE.
Licence classes per R3: PERMISSIVE / WEAK-COPYLEFT / STRONG-COPYLEFT /
NON-COMMERCIAL / PROPRIETARY / UNCLEAR.

## Task 2 — Rust/Bevy code (the decisive, checkable axis: version compat)

| ID | Tier | Artifact | URL | Accessed | Status | Licence (verbatim/class) | Bevy compat | What it gave us |
|---|---|---|---|---|---|---|---|---|
| S-01 | P | `bevy_animation_graph` (mbrea-c) | https://github.com/mbrea-c/bevy_animation_graph | 2026-08-01 | READ | "Apache-2.0, MIT licenses found" — dual, **PERMISSIVE** | master branch targets Bevy 0.19; compat table extends back to 0.12 — **0.15 compatibility not individually confirmed, needs the actual compat table read next pass**, recorded honestly as unconfirmed rather than assumed | State machines as animation-graph nodes, two-bone IK, ragdoll incl. partial ragdoll (some bones simulated, some kinematic) — exactly the shape of an active-ragdoll hit reaction the brief names. Published to crates.io as a real library, not a demo. 14 open issues at read time. |
| S-02 | P | `bevy_motion_matching` (voxell-tech) | https://github.com/voxell-tech/bevy_motion_matching | 2026-08-01 | READ | "dual-licensed under either MIT... or Apache License 2.0" — **PERMISSIVE** | **Bevy version not stated anywhere in the README at read time** — recorded as unconfirmed, not assumed compatible | Motion-matching plugin: queries a pose/trajectory database by desired attributes instead of a state machine. Explicitly WIP — "being split into library and example crates, to be published on crates.io" is NOT YET published. Not integration-ready today. |
| S-03 | P | `bevy_mod_inverse_kinematics` (Kurble) | https://crates.io/crates/bevy_mod_inverse_kinematics (+ /versions, /0.8.0/dependencies, /0.11.0/dependencies API endpoints) | 2026-08-01 | READ | "MIT OR Apache-2.0" per crates.io metadata — **PERMISSIVE** | **Version 0.8.0 (released 2025-02-08) depends on `bevy = "^0.15"` exactly** — confirmed via the crates.io dependencies API, not inferred. Latest 0.11.0 (2026-03-12) has moved to `^0.18`, i.e. the crate has since drifted past our engine version — pin to 0.8.0. | Positional + pole-target two-bone IK. This is the one confirmed, version-matched, ready-today building block of the three Rust candidates. |

Task 2 quota: 3/14 core counted so far (all P, 0 V — code repos don't
carry a video tier by nature; the brief's ≥3V requirement is satisfied
from Task 1/3 talks, not here).

## Task 1 — Papers (anchor work + the licence trap it sits on)

| ID | Tier | Artifact | URL | Accessed | Status | Licence (verbatim/class) | What it gave us |
|---|---|---|---|---|---|---|---|
| S-04 | P | Learned Motion Matching — summary page, Daniel Holden | https://theorangeduck.com/page/learned-motion-matching | 2026-08-01 | SKIMMED | article itself: no licence stated on the page | Problem/method: motion matching's memory cost scales with database size; three trained networks replace the runtime database lookup, keeping quality/control/iteration speed while removing the memory scaling. **No numeric values (memory MB, latency ms, training hours) were recoverable from this page** — the actual SIGGRAPH 2020 paper PDF has not yet been fetched; this is a SKIM of the landing page, not the paper, and is marked so honestly rather than counted as a full READ of the paper's numbers. |
| S-05 | P | `orangeduck/Motion-Matching` reference implementation | https://github.com/orangeduck/Motion-Matching | 2026-08-01 | READ | MIT (repo footer metadata; LICENSE file text itself not directly quoted — flagged for verbatim confirmation next pass) — **PERMISSIVE, code only** | **Confirms the brief's stated trap directly, from the primary source, not the brief's own say-so**: the repo's own docs state training data "is from" the Ubisoft LaForge Animation Dataset (LaFAN1), and separately record that dataset as **CC BY-NC-ND 4.0 — NON-COMMERCIAL**. Contains `controller.cpp`/`database.h` (classical motion-matching core) and `nnet.h` (the learned nets), no pretrained weights bundled. **Verdict for our project: the CODE (MIT) is readable/reusable for technique; the DATA path (LaFAN1) is not shippable** — exactly the R3 gate the brief exists to enforce, verified rather than assumed. |

Task 1 quota: 2/14 core counted (2 P, 0 V). The paper PDF itself
(SIGGRAPH 2020 proceedings, hosted at theorangeduck.com per the brief)
and the video tier (GDC/SIGGRAPH talks) are the honest next-pass work —
not started this session.

## Not yet started (stated per R2/R10, not silently dropped)

- Phase-Functioned Neural Networks (Holden/Komura/Saito 2017) — not fetched
- DReCon (Bergamin et al.) — not fetched
- Robust Motion In-betweening (Harvey et al.) — not fetched
- DeepMimic lineage / adversarial motion priors — not fetched
- UE5 Motion Matching / Pose Search docs (Task 3, technique-only) — not fetched
- Ubisoft For Honor motion-matching GDC talk — not fetched (this is the
  tier-V material; ≥3 V is currently 0/3, the honest gap)
- Task 4 (adjacent, half quota): active ragdoll/get-up, ORCA/RVO crowd
  avoidance, tactical AI (Game AI Pro) — none fetched

## Quota status (honest)

Core: 5/14 counted (4 P, 0 V — **V tier is the hard gap**, needs GDC
Vault or a fetchable talk transcript, same limitation hit on the other
eight master-brief topics this session).
Adjacent: 0/7.

---

# SESSION 2 — 2026-08-10 (owner override; `DECISION.md` written)

Session 1 refused to write the decision. The owner overrode that. The
refusal's substance is honoured in `DECISION.md` §9/§10 rather than by
continuing to withhold the deliverable. **The quota was NOT met and was
NOT pursued** — per the owner's instruction not to open a new research
cycle. Two targeted fetches were made for one specific decisive number;
one landed, one failed. Both recorded below.

## New rows

| ID | Tier | Artifact | URL | Accessed | Status | Licence (verbatim/class) | What it gave us |
|---|---|---|---|---|---|---|---|
| S-06 | P | O3DE engineering blog, "Motion Matching in O3DE, a Data-Driven Animation Technique" | https://docs.o3de.org/blog/posts/blog-motionmatching/ | 2026-08-10 | **READ** (fetched with an extraction prompt demanding verbatim quotes and "NOT PRESENT" for absent quantities; the one returned figure was then independently re-derived — see below) | **NOT VERIFIED.** O3DE is widely stated to be Apache-2.0 but I did **not** read its LICENSE file this session. **Cited for a number only; NOT recommended for shipping**, so R3/R6's gate is not triggered. Do not upgrade this cell without reading the file. | The database-scaling figure, verbatim: *"A motion capture database holding 1 hour of animation data together with a sample rate of 30 Hz to extract features will generate 108,000 frames. Using the default feature schema, comprising of 59 features, will result in a feature matrix holding ~6.4 million values and use ~24.3 MB of memory."* Also verbatim on feature composition: *"The root trajectory along with the left and right foot positions and velocities have been proven to be a good start here."* **Explicitly NOT PRESENT in the article** (asked for, answered as absent, not estimated): ms per character per frame; character count; hardware; the mocap data's source and licence. |
| S-07 | P | Learned Motion Matching (Holden, Kanoun, Perepichka, Popa — SIGGRAPH 2020), the paper PDF | https://theorangeduck.com/media/uploads/other_stuff/Learned_Motion_Matching.pdf | 2026-08-10 | **UNREACHABLE-BY-TOOL** | n/a | Exact path obtained from the landing page [S-04]. Fetch failed: `maxContentLength size of 10485760 exceeded` — the PDF exceeds 10 MiB. This environment has **no `curl`, `wget`, or Python**, so there is no download-and-read-locally route (contrast the body-rig pass, where that route was the whole method). Same failure class as the Bournemouth parkour thesis in `traversal/SOURCES.md`. **Zero numbers carried.** |
| S-08 | P | "GPU-based Motion Matching for Crowds in the Unreal Engine", SIGGRAPH Asia 2020 Posters, DOI 10.1145/3415264.3425474 | https://dl.acm.org/doi/fullHtml/10.1145/3415264.3425474 | 2026-08-10 | **UNREACHABLE — HTTP 403 Forbidden** | n/a | A search snippet offers *"decreased computation times up-to 95%"*. **SNIPPET-ONLY, relative, no absolute ms, NOT CARRIED.** It is a 2-page poster; expected yield was low even on success. |
| S-09 | P (repo-primary) | This repository at commit `e2866a9` — `jk_tdm/src/{main,sim}.rs`, `jk_wall/src/*`, `jk_core/src/timestep.rs`, `jk_spike/src/bin/bench.rs`, all seven `Cargo.toml` | local | 2026-08-10 | **READ** (targeted: ~1 500 lines read in full, plus exhaustive greps whose exact patterns are recorded in `NOTES.md` §S2) | `MIT OR Apache-2.0` per `engine/Cargo.toml:8` — **PERMISSIVE**, it is ours | **The source that actually decided the topic.** Every factual claim in `DECISION.md` §1 traces to a file and line here. Headline findings: zero uses of Bevy's animation API anywhere in code; `assets/` empty; a complete closed-form procedural rig already exists including two-bone IK with pole vector and elbow clamp; legs have no IK; the SIM/COSMETIC boundary is compiler-enforced (`Res<Game>`) and crate-enforced (`jk_wall` has no bevy, `jk_tdm` has no rapier); the animated population ceiling is 56, not 250. |

## The arithmetic check on S-06 — reproduced, not trusted

Per this project's standing rule (a tool's summary is not the source),
S-06's single figure was re-derived before being carried:

```
3600 s × 30 Hz            = 108 000 frames        ✓ matches "108,000"
108 000 × 59 features     = 6 372 000 values      ✓ matches "~6.4 million"
6 372 000 × 4 B (f32)     = 25 488 000 B
25 488 000 / 1 048 576    = 24.31 MiB             ✓ matches "~24.3 MB"
```

All three chain correctly, which additionally pins the element type to
`f32` and the unit to MiB — neither of which the article states. A
fabricated triple would be very unlikely to close this way. **This is the
consistency test the `aiming/SOURCES.md` incident teaches, applied
prospectively.**

**Precision ceiling [R8]:** the schema samples at **30 Hz ⇒ 33.3 ms**.
Our sim runs at 120 Hz (8.33 ms). **Nothing at 120 Hz resolution may be
built on this figure.**

## Corrections to session 1's record

1. **S-03 / the "quick win" is REVERSED.** Session 1 recommended trialling
   `bevy_mod_inverse_kinematics` 0.8.0 on the stated basis that
   "`jk_tdm`/`jk_wall` have no IK crate today; the 20-segment body's grip
   poses and reach are hand-posed." The first clause is true; **the second
   is false.** `solve_arm_ik` (`jk_tdm/src/main.rs:2579`) is a closed-form
   two-bone solver with a pole vector — the crate's exact feature set —
   plus a biomechanical elbow clamp (`clamp_elbow_flex`, `:2417`) and
   per-side critically-damped sprung hand/pole targets (`:2488-2489`) that
   the crate does not have. Session 1 searched crates.io for a capability
   that was already in the codebase. **The licence and version facts in
   S-03 stand and are unchanged; only the recommendation is reversed.**
   Rejected in `DECISION.md` §5.4 on axis 9 (duplicate capability), with
   the condition that would reopen it stated there.
2. **`NOTES.md`'s tier-V blocker is STALE.** It reads "GDC Vault access
   and most conference-talk video transcripts are not fetchable by the
   tools available here." **`TOTO_LOG.md`'s 2026-08-08 entry (TOTO33)
   disproved that** with two working routes: `youtube-transcript-api`
   against the official GDC YouTube channel, and Internet Archive
   `_djvu.txt` OCR of Vault-gated slide decks.
3. **…and tier-V is nevertheless still 0 today, for a NEW reason.** Both
   of TOTO33's routes need a shell. Probed directly this session: `curl`,
   `wget`, `python`, `python3`, `py`, `node`, `cmd.exe`, `powershell.exe`,
   `pwsh.exe` — **every one exits 127, command not found**; `ls`, `grep`,
   `head`, `wc` and `git` are absent too. Only `WebSearch` and `WebFetch`
   reach the network. **Tier-V availability is environment-dependent and
   varies between sessions — check the shell before declaring a talk
   unreachable, and record which way it went.**

## Quota status after session 2 (honest, and it fails)

Core: **6/14 counted** (6 P, **0 V**). Adjacent: **0/7**.
The brief's `source_quota` test would **FAIL**. This is recorded, not
worked around: the owner overrode the quota in favour of shipping the
decision, and `DECISION.md` §10.6 says so in the deliverable itself.
