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
