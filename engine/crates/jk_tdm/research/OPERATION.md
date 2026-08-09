# The operation — Thor · Toto · Friday

Three named agents with one job each, defined as real reusable subagent
types in `.claude/agents/`. Invoke them by name; they carry their own
system prompts, tool access, and persistent memory.

```
        ┌──────────────────────────────────────────────┐
        │                                              │
        ▼                                              │
   ┌─────────┐   question    ┌────────┐   spec    ┌─────────┐
   │  THOR   │──────────────▶│  TOTO  │──────────▶│ FRIDAY  │
   │ verify  │               │research│           │  build  │
   │ manage  │◀──────────────┴────────┘           └────┬────┘
   └─────────┘        evidence back                    │
        ▲                                              │
        └──────────────── verify the build ────────────┘
```

| Agent | Owns | Log | Can edit source? |
|---|---|---|---|
| **Thor** | verification, management, dispatch | `THOR_LOG.md` | **No — never** |
| **Toto** | peer-reviewed research, evidence | `TOTO_LOG.md` | No (research files only) |
| **Friday** | implementation, tests, captures | `FRIDAY_LOG.md` | Yes |

**Thor cannot edit source. That is the load-bearing rule.** A verifier
that fixes what it finds cannot be trusted to report what it missed.

---

## The handoff contracts

A vague handoff wastes a whole cycle, so each one has a required shape.

### Thor → Toto (a research dispatch)
1. **The question**, specifically. Not "research grip" — "what is the
   measured time from peak hip velocity to release in an overarm throw,
   and how does it distribute across the arm segments?"
2. **What is in the code today**, with the current values and file:line.
3. **What changes if the answer differs** — so Toto knows whether 5 ms
   or 50 ms resolution matters.
4. **The precision actually needed.** Prevents both over-research and
   false precision.

### Toto → Friday (an implementable extraction)
1. **The values**: units, conditions, sample size, population, method.
2. **The arithmetic in full**, if anything was derived — so it can be
   re-checked rather than trusted.
3. **MEASURED / DERIVED / ASSUMED**, labelled per value. Never blended.
4. **What this contradicts** in the current design.
5. **The precision ceiling** — "source is 50 fps, so 20 ms; build
   nothing finer."
6. **What could not be answered**, plainly.

### Friday → Thor (a build report)
1. **file:line for every change.**
2. **Exact test command, before and after counts.**
3. **Live capture evidence** if anything visual moved: exit code,
   screenshot count, panic count.
4. **What Friday is least sure about** — volunteered, not extracted.
5. **What was deferred and why.** A stated deferral is fine; a silent
   one is a defect.

---

## Optimising the operation

**1. Pipeline, don't alternate.** The naive loop (research → build →
verify → repeat) leaves two agents idle at all times. Instead:

```
   Toto researches step N+1  ──┐
   Friday builds step N       ─┼── concurrently
   Thor verifies step N-1     ──┘
```

Only serialise where there is a genuine dependency: Friday cannot build
what Toto has not researched, and Thor cannot verify what Friday has
not built. Everything else overlaps.

**2. Never let two agents write the same file.** Reads are free;
concurrent writes are not. Thor reads `main.rs` while Friday edits it →
Thor analyses a file that is shifting underneath it. Either sequence
them or scope them to different files. This is the single most common
way to waste a cycle.

**3. Batch Toto's dispatches.** Research has long latency (fetches,
paywalls, dead ends) and near-zero conflict risk — several Totos can run
at once on unrelated questions. Building is the opposite. So: fan out
research wide, keep implementation narrow.

**4. Dispatch on real uncertainty, not on schedule.** Do not send Toto
after a value that is already measured and cited. Do not send Thor after
a change that is provably inert. Both are real costs.

**5. Verify the load-bearing claim, not the whole diff.** Thor's time is
best spent re-deriving the ONE argument that licensed the change. If
"this is safe because X cancels" is false, nothing else about the diff
matters.

**6. Prefer one shared function to two parallel ones.** Player/bot
parity has been a repeated defect class here — the mech turn rate and
the acceleration model both shipped bot-broken first. A shared function
cannot drift; two implementations always will.

**7. Commit an agent's output as soon as it lands — the working tree is
shared state.** Learned the hard way on 2026-08-02: Toto finished a
research ledger (untracked) while Thor was mid-mutation-test on
`main.rs`. Thor had been told it could revert with `git stash`. A bare
`git stash` sweeps the ENTIRE working tree — it would have taken Toto's
untracked ledger with it, and the spec already cited that file
throughout. File-scoping prevents two agents *editing* the same file; it
does **not** protect one agent's uncommitted work from another agent's
git command.

Two rules follow:
- **Commit finished work before dispatching an agent that runs git.**
- **Never tell an agent to `git stash` bare.** Scope the revert:
  `git checkout -- <the one file you mutated>`. An agent doing
  mutation-testing must revert surgically, because it does not know what
  else is in the tree.

**7b. The same rule, one level in: `git checkout` reverts to HEAD, not
to "before my mutation."** Learned the hard way on 2026-08-03, doing it
to myself. I mutation-tested a determinism test that was still
UNCOMMITTED, then reverted the mutation with
`git checkout -- sim.rs` — which correctly restored HEAD and deleted the
test along with the mutation. Recovered only because a file copy had
been taken first.

Mutation-testing your own uncommitted work has exactly two safe shapes:
- **commit first**, then `git checkout --` reverts the mutation only; or
- **revert from a file copy** (`cp file /tmp/backup` before, `cp` back
  after) and never let git near it.

Choose one deliberately. The failure is silent — a green suite after the
revert looks identical whether your test survived or vanished.

---

## What the 2026-08-08 session taught (rules 8-12)

A single very long session, ~19 commits, six agents. These are the
lessons that cost something to learn.

**8. THE CAPTURE IS THE INSTRUMENT. Three defects this session were
invisible to the compiler, invisible to 328 tests, and obvious the
moment someone looked at a picture:**

- a GORGET placed at y=0.80 inside a torso whose crown is 0.82. The
  heaviest armour trim's one distinguishing part could never be seen.
  The table said four trims; the screen showed three.
- a BARREL SHROUD written as "a half-cowl, open below, so the barrels
  stay the thing you read" — built from cylinders facing down the barrel
  axis, which are DISCS. It capped the minigun and hid all six barrels.
  The comment described the intent; the geometry did the opposite.
- a MECH BARRIER that shipped, was redesigned twice, and had never once
  been photographed. Every claim about it came from reading the code
  that spawned it.

The rule: **a visual claim with no screenshot behind it is not a claim,
it is a hope.** Say "not verified" in the commit when it is not. Three
commits this session did exactly that and they are the honest ones.

**8b. If no capture can see the thing, BUILD THE CAPTURE.** The
soldier's fingers were unverifiable because every script shoots from the
2.2 m boom. Adding `CapBeat.boom` took one build and made the claim
checkable forever. Framing it took three attempts and each was a
property of the RIG worth writing down: the boom anchors on the HEAD;
closing the distance magnifies the offset between anchor and subject;
and pitch orbits the CAMERA about the anchor rather than tilting the
view, so positive pitch photographs the top of a hat.

**9. "FEELS BAD" IS OFTEN DEAD CODE, NOT TUNING.** Check before you
touch a number. This session found three:

- the SPEAR halved its release speed for `i == self.player`, but the
  charge path called `try_fire` unconditionally, so EVERY player javelin
  took it — while the client's preview drew the full-speed arc. The
  player was aiming along a line the spear could never follow.
- the GRENADE's fuse and wind-up were one clock, so G armed a live
  grenade in the hand, and the wind-up started at EQUIP so every throw
  arrived near maximum power.
- `AUTOCANNON_UNBRACED_KICK = 6.0` was inert: punch sheds 18 deg/s
  linearly, so a 6 deg/s impulse is erased inside the tick it lands. Its
  own doc claimed it "costs you the next shot's picture". It cost
  nothing.

Retuning any of those would have made them worse.

**10. AUDIT BEFORE BUILDING, AND REPORT A CLEAN AUDIT AS A RESULT.**
The §7 first-person aiming complaint produced five hypotheses and all
five came back clear — the aim is geometrically exact. That is a real
finding, not a wasted cycle: it moved the question from "find the bug"
to "the owner must choose a forgiveness value", which is a decision
nobody should make silently on their behalf. What shipped was a test
pinning the property, because it held by a COINCIDENCE (muzzle == eye)
rather than by construction.

**11. THE SPEC'S PREMISE IS A CLAIM, NOT A FACT.** Three of the owner's
36 sections were wrong about this build, and saying so was worth more
than the ticks:

- "the bow randomly rotates sideways" — it was deliberately laid
  horizontal to keep the upper limb out of the sight line.
- "the shield is missing from the HUD" — it was there; its STATE was
  missing. And only ONE of the two shields has the current/max/recharge
  the spec asked for, so the other reports block % rather than a
  fabricated 0/0.
- "restore the rotating turret" — never lost. It existed in the
  VIEWMODEL alone, so the pilot watched it spin and nobody else did.

Two sections had NO target at all (an "x-ray" hit effect that does not
exist; uploaded assets that are not in the repo). Both were reported as
not-found rather than guessed at. **Do not delete something on the
strength of a guess.**

**12. THE SESSION'S OWN TEST DISCIPLINE CAUGHT THE SESSION.** A
first-person aim test placed its camera at `muzzle_origin` — the very
function under test — so the mutation moved both together and the
assertion could never fail. It was proven vacuous by exactly the
mutation it existed to catch. Mutation-testing is not paperwork; it is
the only thing that distinguishes a test from a comment.

## Running six agents at once — what actually parallelises

- **`sim.rs` and `main.rs` are the only real lanes.** friday22 and
  friday33 can run concurrently forever. A third builder has nowhere to
  go, so scale by adding SCOUTS and RESEARCH, not more builders.
- **Read-only agents parallelise without limit.** Thor, the scouts and
  Toto never collide.
- **Warn every agent about the transient.** Both builders run
  `cargo test`, which compiles BOTH files, so a suite run during the
  other's write fails for no reason. Every dispatch must say: re-run
  before concluding a failure is real. Without that warning an agent
  reports a phantom defect and the cycle is wasted.
- **Commit before dispatching** (rule 7) — and stage ONLY your own file
  when the other lane is mid-work. Two commits this session did that
  deliberately.
- **Hand the next agent the trap.** The crouch dispatch named the guard
  it must not simply delete, and the builder solved the cause instead of
  the symptom. A dispatch that names the known trap is worth more than
  one that describes the feature.

---

## RULE 13 — THE RESEARCH TIER IS RETIRED. SCOUTS SCALE INSTEAD.

Owner's instruction, 2026-08-09: *"dont be doing reserch try to be more
built and friday orienatted"*, and then *"cancel research but also you
can tell them to become scouts to help you build"*.

**The evidence backs it.** Measured over this week:

| tier | cost | what reached the code |
|---|---|---|
| `toto` armour failure | ~187k tokens | a positional-damage model the builder weighed and did NOT adopt |
| `toto33` vertical maps | ~211k tokens | one usable line ("what you see is what you get") + a bot finding the builder could have grepped in a minute |
| `scout-defect` | ~150k | dead grenade throw, turret spinning off the wrong fighter, 3 inert constants, doc rot cluster |
| `scout-gap` | ~127k | 8 boarding stages rendering nothing, medic rendering man-sized, two castle centrepieces that are solid boxes |

The scouts found things that SHIPPED AS FIXES. The researchers produced
knowledge that mostly did not survive contact with a builder's judgement.
On a game where the owner iterates from screenshots, a defect someone can
see beats a citation.

**The new roster:**
- **Two builder LANES, and only two**: `friday22` owns `sim.rs`,
  `friday33` owns `main.rs` + client modules. A third builder has
  nowhere to go. This is the hard ceiling on parallel building.
- **Scale with SCOUTS, not researchers.** `scout-defect`, `scout-gap`,
  `scout-map` are read-only, collide with nothing, and can run many at
  once. Their output is a work queue for the two Fridays.
- **`thor` stays.** Verification is NOT research: it caught a totally
  dead grenade throw that 320 passing tests missed. Wake it to check
  shipped claims, not to study process.
- **`toto*` only when a specific unknown NUMBER blocks a build**, and it
  must be named in the dispatch. Never to survey a topic, never "for
  background", never speculatively ahead of a build.

**The pattern that works:** scouts sweep -> findings become a ranked
queue -> the two Fridays build the queue -> thor verifies -> repeat.
Research enters only when a builder stops and says "I cannot pick this
number".

---

## Standing rules all three inherit

- **Never invent a source.** This project has caught one fabricated
  extraction (five invented numbers in a real paper's summary). A tool's
  summary is not the source.
- **An honest gap beats a plausible invention.** Always.
- **"Verified false" ≠ "never checked."** Conflating them has caused two
  incidents here, both in the instrument rather than the game.
- **Determinism is law.** Fixed 120 Hz, seeded RNG, bit-identical replay
  with tests enforcing it. Anything entering `sim.rs` becomes replay
  state.
- **SIM vs COSMETIC is a bright line**, declared per system.
- **A test that cannot fail is worse than no test.**

---

## Invoking them

```
Task(subagent_type="toto",   prompt="<dispatch per the contract above>")
Task(subagent_type="friday", prompt="<spec per the contract above>")
Task(subagent_type="thor",   prompt="<what to verify, and the claim it rests on>")
```

Run the independent ones concurrently in a single message. Sequence only
real dependencies.
