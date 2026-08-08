# vertical-maps — SOURCES

Practitioner tier (talks, dev blogs, postmortems) on: flight vs. level
design, altitude bands, multi-level readability, mixed area types, and
what breaks for bots on vertical geometry.

Opened 2026-08-08 by **TOTO33**. Append, do not rewrite.

---

## THE HARD LIMIT, STATED UP FRONT

**No agent in this operation can watch video or hear audio.** Everything
below marked `READ-TRANSCRIPT` was read as *text* — YouTube auto-captions
pulled to disk and read with `Read`. Everything marked `READ-SLIDES` is a
slide deck with no narration. A talk whose title I know but whose text I
do not have is recorded as `NO-TRANSCRIPT`, which is a finding, not a
failure.

**Auto-caption precision ceiling.** Auto-captions have no punctuation, no
speaker marks, and they mis-hear words. They routinely drop negations
(see VM-02 [20:13], where the sentence is almost certainly missing a
"not" and I say so at the point of use). **Quote conservatively; never
carry a number that appears only once in a caption without checking it
against the speaker's own arithmetic.** One such number is rejected
below (CONTRADICTION-2).

---

## Sources

| ID | Tier | Type | Title | Author / Studio | Year | URL | Accessed | Status | What it gave us |
|---|---|---|---|---|---|---|---|---|---|
| VM-01 | V | GDC talk | The Importance of Nothing: Using Negative Space in Level Design | Jim Brown / Epic Games | GDC 2014 | https://www.youtube.com/watch?v=GZ99gAb4T0o | 2026-08-08 | **READ-TRANSCRIPT** (auto-caption, full 51 min, 150 blocks on disk) | The single richest source here. Four shipped remakes of one map (UT *Facing Worlds* / CTF-Face) each measurably worse; Gears of War *Gridlock* ruined and then rescued **by art alone**. Nav-graph complexity as a readability proxy. |
| VM-02 | V | GDC talk | Level Design Workshop: The Holy Grail of Multiplayer Level Design — Maps for Casual and Competitive Play | Andrew Yoder / Hi-Rez Studios (Paladins) | GDC 2018 (inferred, see note) | https://www.youtube.com/watch?v=NhMDTxnzQuA (Vault 1025183 / 1025497) | 2026-08-08 | **READ-TRANSCRIPT** (auto-caption, full 27 min) | **Upgrades `maps/SOURCES.md` S-02 from SNIPPET-ONLY.** Flight in a shipped MP game and the map rule it forced. Two named experiments with measured outcomes, one of them a rejection. |
| VM-03 | V | GDC talk | Level Design Workshop: Singleplayer vs. Multiplayer Level Design: A Paradigm Shift | Elisabeth Beinke-Schwartz | GDC 2017 | https://www.youtube.com/watch?v=PxpjRuATxKE | 2026-08-08 | **READ-TRANSCRIPT (PARTIAL)** — full 31 min transcript on disk, scanned end to end, **5 passages read in full**; the rest not read | One load-bearing claim only: getting lost costs far less in MP than in SP. |
| VM-04 | P | dev blog | Gliding in Central Tyria | Crystin Cox / ArenaNet (Guild Wars 2) | 2016-01-13 | https://www.guildwars2.com/en/news/gliding-in-central-tyria/ | 2026-08-08 | **READ-WRITEUP** — raw HTML fetched, stripped, read verbatim | The only source found that describes **retrofitting flight onto maps built for walking**, by the team that did it. |
| VM-05 | P | book chapter (author's own writeup of his GDC talk) | 3D Flight Navigation Using Sparse Voxel Octrees, *Game AI Pro 3* ch. 21 | Daniel Brewer / Digital Extremes (Warframe) | 2017 | https://www.gameaipro.com/GameAIPro3/GameAIPro3_Chapter21_3D_Flight_Navigation_Using_Sparse_Voxel_Octrees.pdf | 2026-08-08 | **READ** (full 10 pp, PDF on disk) | Names and **rejects** stacked-navmesh-layers and 3D waypoint graphs for flight, with the reason and the boundary condition. |
| VM-06 | V | GDC talk slides | Getting off the NavMesh: Navigating in Fully 3D Environments (GDC 2015 AI Summit) | Daniel Brewer / Digital Extremes | 2015 | https://archive.org/details/GDC2015Brewer (`GDC2015-Brewer_djvu.txt`) | 2026-08-08 | **READ-SLIDES** (OCR text of the deck; no narration) | The measured numbers VM-05 omits: real level build stats and a 6-row A\* optimisation ladder. |
| VM-07 | P | book chapter | Hierarchical AI for Multiplayer Bots in Killzone 3, *Game AI Pro* ch. 29 | Straatman, Verweij, Champandard, Morcus, Kleve / Guerrilla Games | 2013 | https://www.gameaipro.com/GameAIPro/GameAIPro_Chapter29_Hierarchical_AI_for_Multiplayer_Bots_in_Killzone_3.pdf | 2026-08-08 | **READ** (full 14 pp) | Shipped AAA multiplayer bots. What the bots need **from the map**, and what the studio decided to place by hand and why. |
| VM-08 | P | book chapter | Extending the Spatial Coverage of a Voxel-Based Navigation Mesh, *Game AI Pro 2* ch. 32 | Kevin A. Kirst | 2015 | https://www.gameaipro.com/GameAIPro2/GameAIPro2_Chapter32_Extending_the_Spatial_Coverage_of_a_Voxel-Based_Navigation_Mesh.pdf | 2026-08-08 | **READ** (full 16 pp) | The exact parameters that decide whether a slope or ledge is navigable at all, and what they exclude. |
| VM-09 | V | GDC talk slides | The Illusion of Intelligence: The Integration of AI and Level Design in Halo | Chris Butcher & Jaime Griesemer / Bungie | GDC 2002 | https://www.jmeiners.com/shamans/papers/ai/the_illusion_of_intelligence.pdf | 2026-08-08 | **READ-SLIDES** (27 slides, no narration) | The original statement of "AI is a level-design problem". Firing points; two playtest tables. |
| VM-10 | P | reference | Verticality — The Level Design Book | Robert Yang et al. | — | https://book.leveldesignbook.com/process/layout/flow/verticality | 2026-08-08 | **READ** (`.md` source on disk) | The **three floor planes** heuristic — the closest thing to an answer on altitude banding. |
| VM-11 | P | reference | Composition — The Level Design Book | Robert Yang et al. | — | https://book.leveldesignbook.com/process/blockout/massing/composition | 2026-08-08 | **READ** | Landmark / hierarchy / sightline definitions; an explicit attack on "leading lines". Landmark section is a **stub**. |
| VM-12 | P | reference | Wayfinding — The Level Design Book | Robert Yang et al. | — | https://book.leveldesignbook.com/process/blockout/wayfinding | 2026-08-08 | **READ** | Lynch's five mental-map elements; a 20-row wayfinding-aid table the authors themselves call **non-scientific**. |
| VM-13 | P | reference | Circulation — The Level Design Book | Robert Yang et al. | — | https://book.leveldesignbook.com/process/layout/flow/circulation | 2026-08-08 | **READ** | The **1–3 lanes** heuristic and the named failure ("will function like a maze"). |
| VM-14 | P | dev/critic blog | that's not fun: *zk map for stranger* | Bennett Foddy (on a map by Marek Kapolk) | 2021-09 | https://foddy.net/blog/2021/09/zk-map-for-stranger/ | 2026-08-08 | **READ-WRITEUP** | The only credible defence of *deliberately unreadable* vertical geometry — and the price it names. |
| VM-15 | X | third-party news writeup of a GDC talk | Understanding Titanfall 2's "action block" level prototyping process | Alex Wawro / Gamasutra, reporting Vince Zampella | 2016-11-16 | https://www.gamedeveloper.com/design/understanding-i-titanfall-2-i-s-action-block-level-prototyping-process | 2026-08-08 | **SECOND-HAND** | 100–200 throwaway level prototypes. **Explicitly does NOT answer the verticality question** — recorded so nobody chases it again. |
| VM-16 | — | conference talk | Automatic Annotations in Killzone 3 and Beyond (Paris Game/AI 2011) | Mikko Mononen / Guerrilla | 2011 | referenced from VM-07; slides said to be on `digestingduck.blogspot.com` + Guerrilla publications page | 2026-08-08 | **NO-TRANSCRIPT** — identified, not retrieved this pass | Would be the primary source on auto-generating cover/annotations from geometry. **Top unread target.** |
| VM-17 | — | — | Any ArenaNet primary on designing Heart of Thorns maps *for* gliding | ArenaNet | — | — | 2026-08-08 | **NOT FOUND** | Searched; only press reviews and wiki. The "built for flight" half of the GW2 story has no primary source I could reach. |
| VM-18 | — | — | Any Respawn primary on Titanfall/Apex vertical or flight level design | Respawn | — | GDC Vault 1025105 etc. | 2026-08-08 | **PAYWALLED / NOT FOUND** | The obvious source for Q1 does not exist in reachable text. Stated plainly rather than substituted. |

**Counted this pass: 14 read end-to-end or in-full-transcript (VM-01…VM-14),
of which 5 are tier-V.**

---

## ⚑ METHOD NOTE — the tier-V blocker is broken

Every ledger in this repo has reported **tier-V = 0** since the project
began, and `TOTO_LOG.md` says so three times. The stated blocker was GDC
Vault gating. **That was the wrong blocker.**

```
pip install youtube-transcript-api
python -c "from youtube_transcript_api import YouTubeTranscriptApi as Y; print(Y().fetch('GZ99gAb4T0o'))"
```

Returns timestamped caption snippets for GDC talks published on YouTube.
It worked first try. Three talks were read this pass (VM-01/02/03) and
VM-02 **is a talk already sitting in `maps/SOURCES.md` as SNIPPET-ONLY,
uncounted, since 2026-08-01.**

Also proven this pass: **Internet Archive carries OCR'd GDC slide decks**
(`archive.org/download/<item>/<item>_djvu.txt`) for talks that are Vault-
gated — that is how VM-06 was read. `aiming/SOURCES.md` S-01 (Weihs, aim
assist) is listed at `archive.org/details/GDC2013Weihs`, i.e. **the same
route is already sitting unused in another ledger.**

Two standing gaps in other topics are therefore reachable, and neither
needs Vault access. Handed to Friday/Thor separately.

---

## Q1 — What changes about level design when players can FLY?

### The one shipped rule, from a studio with flying characters

**VM-02, Andrew Yoder, Hi-Rez, on Paladins [19:57 → 20:13]:**

> "another principle that we have this is a little bit more on the
> environment art side is that we want to make sure that **what you see is
> what you get** we have characters that can fly and we want to make sure
> that as you're going over rooftops you're [**sic**] hitting invisible
> walls and that really defines a lot of kind of the level design problems
> we run into"

**Caption warning, stated at the point of use:** the second clause is
almost certainly *"you're **not** hitting invisible walls"* — the
auto-caption has dropped a negation, and the surrounding sentence ("what
you see is what you get") fixes the meaning beyond reasonable doubt. **I
am carrying the principle, not the sentence.** Do not re-quote that line
verbatim from this ledger.

**Label: DESIGN CHOICE (shipped, studio-wide principle).**

The claim is that flight collapses the distinction between *scenery* and
*playable space*. On a grounded map a roof can be set dressing, because
nobody can get there. Add flight and every surface a player can see
becomes a surface a player will try to stand on, and every one you refuse
them has to be refused *visibly* — not with an invisible wall. Yoder says
this "defines a lot of the level design problems we run into", i.e. it is
their dominant constraint, not a detail.

**This is the finding that matters most for a map being built now,
pre-flight.** It is cheap to satisfy while blocking out and expensive to
retrofit: every roof, ledge, tower top and cliff shelf on the castle must
be decided *now* as either genuinely standable or genuinely, visibly
unreachable.

### What broke when a shipped game added flight to grounded maps

**VM-04, ArenaNet, adding gliding to Central Tyria (maps built years
earlier, without flight).** Verbatim from the post:

> "It will come as no surprise that the maps in Central Tyria were not
> designed with gliding in mind. **It would have been very time consuming
> to redesign all the Central Tyria maps to add verticality, updrafts, and
> ley lines, and we wouldn't want to.** … We've made some small changes
> here and there, but for the most part, you'll find Central Tyria just as
> you left it"

Three distinct breakages, and three different responses:

1. **Content skipping.** "These zones have been set up around sensitive
   areas like the ends of jumping puzzles to prevent skipping content."
   → **No-Fly Zones**, and explicitly "we've used the No-Fly Zones very
   sparingly."
2. **Content that cannot cope at all.** "Instanced content and dungeons
   created before *Heart of Thorns* were just not designed to handle
   gliding, so we'll be keeping you out of the sky in these instances."
   → **flight disabled wholesale** in those spaces.
3. **Unintended geometry access.** "These new freedoms might allow you to
   get to places in these maps we did not originally intend you to see.
   As long as your explorations remain harmless, it's all part of the
   fun!" → **accepted, not fixed.**

**Label: DESIGN CHOICE / postmortem-grade admission. Not a measurement.**

**Ranking of what breaks first, DERIVED from the above:** progression
gates and skippable content break first (they got a dedicated mechanism);
scripted/enclosed spaces break next and hard enough to be opted out of
entirely; raw out-of-bounds geometry breaks last and was tolerated.

**Transfer caveat — read this before applying VM-04.** GW2 gliding is
*descent-only with limited updrafts*: one-way, budgeted, cannot hover,
cannot climb freely. If this project's "flight" is powered and can gain
altitude at will, GW2's tolerance for breakage does **not** transfer —
their No-Fly Zone was cheap precisely because a glider runs out of
altitude on its own. **A powered flier does not.**

### The relevant precedent nobody names as a flight map

**VM-10** on Q3DM17 "The Longest Yard" (Quake 3, Brandon James, 1999):
a dozen jump pads and teleporters, "very little cover and limited floor
area", so "players must dodge gunfire by flying through the air, while
being careful not to fall to their deaths." The designer's stated
counter-balance: a far platform "where snipers can easily dodge incoming
rockets while sniping all the players **taking predictable flight arcs**."

**Label: DESIGN CHOICE, reported by VM-10 (second-hand w.r.t. James).**
The transferable half is the failure mode it was built to answer: **a
player in flight is on a predictable ballistic arc and cannot take cover.**
Any open volume between two altitude bands is a shooting gallery unless
something breaks the arc or blocks the sightline.

### Honest gap on Q1

No Respawn, DICE, BioWare or Avalanche primary was reachable (VM-18,
VM-17). **Titanfall, Battlefield, Anthem and Just Cause contribute
nothing to this ledger.** The two flight sources here are an MMO glider
retrofit and one principle from a hero shooter. That is thin, and it is
the honest state.

---

## Q2 — How do shipped games structure ALTITUDE BANDS?

### The only quantitative guidance found

**VM-10, verbatim:**

> "When planning the verticality in a level, try to chunk it together into
> **floor planes** and merge minor height changes into a single floor.
> Don't try to hold 10 different distinct overlapping layers of floorplans
> in your mind, because players probably won't be able to process that
> much complexity either.
>
> **Most maps tend to max-out at three different floor planes for any
> given area.** Why three? Much like three lane typology, three plane
> format consists of a bottom, middle, and top layer. Comparatively, a
> fourth (p)lane doesn't add new dynamics, because it would simply yet add
> another middle layer or path."

**Label: DESIGN HEURISTIC / observation of the field. NOT a measurement.**
No study, no telemetry, no N. "Most maps tend to" is the strength of the
claim and the ledger will not upgrade it.

**Two things about this rule get misread, and both matter here:**

1. **It is "for any given area", not for the map.** A castle-on-a-cliff
   plus a city plus open ground is *three areas*. Three planes each is
   nine distinct heights across the map and the rule does not forbid that.
   What it forbids is nine planes **stacked in one place**.
2. **The stated reason is cognitive, not mechanical** — a fourth plane
   "doesn't add new dynamics." So the rule is about *distinguishability*,
   not about count. Two planes 3 m apart are one plane wearing a hat.

**VM-13** gives the horizontal twin, and names the failure:

> "Lanes help players predict and coordinate movement. **A large map with
> too many lanes or no clear lane hierarchy will function like a maze;
> players won't know where to focus their efforts, get lost, and miss each
> other.** For this reason, most maps use only 1-3 lanes."

**DERIVED (mine, not theirs): 1–3 lanes × ≤3 planes per area is a rough
upper bound of about 9 discrete "places" per area before the map reads as
a maze.** Nobody in these sources multiplies those two numbers. I am
doing it and labelling it. Do not cite it as practitioner guidance.

### What makes a vertical map read as navigable — one concrete answer

**VM-02 [13:11], on Halo's *Lockout*:**

> "it's a very dense space it's multi-tiered space there's a lot of
> verticality to it but **it's also these kind of additive structures
> floating in space that are connected by bridges which makes it much
> easier to understand where you are relative to other structures** and
> very easy to navigate"

**Label: DESIGN OBSERVATION by a shipping MP level designer.** The
mechanism claimed is *relative* localisation: discrete named masses plus
**visible** connections between them, so a player answers "where am I" as
"on the tower, two bridges from the sniper perch" rather than by
coordinates. This is the single most directly actionable answer to the
"how do bands connect" half of Q2.

Its negative twin, same talk **[09:43]**, on *Chill Out*:

> "even though it's a very simple layout that's not too hard to understand
> **those portal routes make it very confusing for new players and can be
> very disorienting**"

**DERIVED consequence for this project: any traversal that is not
spatially continuous damages the mental map even when the layout is
simple.** Flight is continuous and therefore safe by this criterion.
Teleporters, launchers and one-shot vertical elevators are not. If the
castle is reached by a lift or a cannon rather than by a visible route,
expect Chill Out's problem.

### The question that is NOT answered

**How far apart, in metres, should altitude bands be? Not answered by any
source in this ledger.** No practitioner text found gives a vertical
spacing figure. `maps/SOURCES.md` S-01 has TF2's max no-damage drop
(256 u) but that is a fall-damage threshold, not a band spacing.

**Do not let anyone build a numeric band-spacing rule out of this
ledger.** The only ceiling I can defend is the one in Q3 below, and it is
about *silhouettes*, not about metres.

---

## Q3 — How do you keep a multi-level map READABLE?

This is the best-evidenced question here, because VM-01 is a talk built
entirely out of a studio getting it wrong four times in public.

### The headline result: they changed nothing but the art, and lost the map

**VM-01 [19:41 → 22:54], Gears of War *Gridlock*, Epic's own telemetry:**

- Gears 1: "probably our most popular map in the franchise" **[19:41]**.
- Gears 2 re-release, aged/"overgrown" theme: "the overall sense of
  negative space is completely lost along the floor there's no figure
  ground distinction … visually muddy" **[20:55]**; "here's a chart that
  shows map popularity in Gears of War [2] as a factor of time played and
  you'll notice that **the new version of gridlock is at the very bottom
  of the list**" **[21:16]**.
- Gears 3, aged further: "**the gameplay completely fell apart** what was
  once the most popular map in the game became that one map that our
  testers hated playing — **we hadn't changed the physical play space the
  weapon load out or even the collision** but just that shift in tone and
  color meant a change in perception" **[21:37]**.
- Rebuilt late in the project back to a cleaner style: "here's a chart
  showing this version of gridlock **back in the top three most played
  maps of Gears of War 3's lifetime**" **[22:17]**.

**Label: MEASURED (studio telemetry, time-played rank), reported by the
speaker; I read the report, not the chart.** This is as close to a
controlled experiment as level design gets: geometry, collision and
weapon layout held constant, visual read varied, popularity moved from
top to bottom and back to top.

**Independently corroborated by VM-02 [24:20], a different studio, a
different decade, a different genre.** Hi-Rez's map *Corey* tested well
as grey-box and went lukewarm after the art pass: "once we actually
released it with the final art we found that it was a little bit
lukewarm … players were confused and disoriented they felt
claustrophobic". **Two shipped studios independently report that the art
pass, not the layout, changed how a map played.**

**Consequence for the plan: a "deliberately messy" map's biggest risk is
not its layout. It is that the art pass makes the messiness illegible.**
And the blockout will not tell you — Yoder says so directly at [24:36]
("most players' inability to evaluate gray box maps").

### Good messy vs. bad messy — the mechanism VM-01 names

**Figure/ground [18:11]:** "if your players can't discern figure from
ground they'll have difficulty understanding how to navigate and they'll
struggle with target [acquisition]". Both halves — navigation *and*
target acquisition — from one property.

**Bad messy, type 1 — too much competing detail.** Call of Duty's
*Favela* **[12:35]**: "there's so many paths so much detail so much going
on that it's just visually overwhelming **there's nowhere for your eye to
stop**".

**Bad messy, type 2 — not enough variation.** The Egyptian *Face*
**[29:05]**: "fa[ce]'s lack of variation still has a similar outcome for
the player **there's nowhere for your brain to focus** there's no balance
to this scene". Same symptom, opposite cause. **A monochrome map and a
cluttered map fail identically.** For a castle-and-city map this is the
live risk: stone, stone and more stone.

**Good messy, the counter-example [09:52]:** the original *Facing Worlds*
— "the tall narrowing **Towers anchor the eye** to the important gameplay
spaces and the players in those spaces **can be picked out against their
dark backgrounds**". Note what is doing the work: a tall silhouette
against an empty sky, i.e. **negative space**, not detail.

### The scale trap — read this one twice

**VM-01 [36:39 → 38:16]:**

> "the new version is actually **three times the size of the original
> version** … since we needed more room to fit in all this new stuff we
> just made our environments bigger — note that **we didn't necessarily
> increase the size of our players only the scale of the environments** and
> the obvious result of that decision was that **individual players and
> important gameplay elements became more difficult to identify**"

He then asks the GDC audience to count players in a screenshot; they
manage a handful. "**there are 13 different players on screen** in this
version seeing those teeny tiny players dwarfed in this massive
environment is completely overwhelming."

**Label: DESIGN OBSERVATION + a live audience demonstration. Not a
metric.** But it is the closest thing this ledger has to a bound on
altitude spacing, and it is the right shape of bound:

> **The vertical extent of the map is limited above by the distance at
> which a 1.78 m fighter still reads as a figure against whatever is
> behind him.** Height that exceeds that is height in which players cannot
> find each other.

**This project can measure that**, and should. `maps/SOURCES.md` already
records a `max_unobstructed_sightline` instrument built for the IX-A 40 m
rule, with baselines (Arena 80.2 m … Battlefield 509.9 m). The band
spacing question is the same question pointed upward.

### CONTRADICTION-2 — a caption number that does not reproduce. NOT CARRIED.

VM-01 **[23:54 → 24:15]** states the *Facing Worlds* detail increase as:
original "**540 actors** in the entirety of the map", new version "over
**4,000**"; original "**130 polygons** in view from the sniping Towers",
Egyptian version "over **60,000 triangles** in view from that same
position — **it's a 280 times increase in detail**".

Per the standing rule from the body-rig pass, I divided the two numbers
before using the adjective:

```
60,000 / 130   = 461.5      (speaker says 280)
 4,000 / 540   =   7.4
```

**461.5 ≠ 280.** The speaker's own later comparison *does* reproduce —
at **[39:18]** he says the UT2004 version had "30,000 triangles in view
from the Towers, more than 200 times the amount of visual information
from the original", and 30,000 / 130 = **230.8**, which is indeed "more
than 200". So the method is consistent and it is the **280** that is
anomalous.

Two candidate explanations and I cannot choose between them: (a) the
comparison mixes **polygons** with **triangles**, which are not the same
unit; (b) the auto-caption mis-heard the figure. Either way:

**The 280× is NOT CARRIED.** The raw pairs (130 → 60,000 and 130 →
30,000, same viewpoint) are carried, with the unit mismatch attached.
**Precision ceiling: "two to three orders of magnitude more visual
information from the same viewpoint" is the strongest honest form of this
claim.** Do not put a specific multiplier in a spec.

This is exactly the failure mode `aiming/SOURCES.md` was written to warn
about, arriving through a new channel — auto-captions rather than a tool
summary. **Assume it will happen on every transcript.**

### The nav-graph complexity proxy — the most portable idea in the talk

**VM-01 [33:11 → 34:30]:**

- Original *Facing Worlds*: "you can get there in **as little as four
  branch points** and there's only **59 navigation points** in the entirety
  of the map".
- Egyptian version: "the minimum number of branch points I could find is
  over [a] dozen and there's **806 navigation points** more than 13 times
  [that of] the original".
- **[34:11]:** "you can certainly see the impact that a complicated
  navigation Network would have on AI and the point that I'm trying to get
  across is that **an AI Network serves as an indicator of potential player
  behaviors** — clearly a player trying to cross the same map has more
  choices to make more obstacles and more potential points for contention
  or confusion".

806 / 59 = **13.66×**, and the speaker says "more than 13 times", so his
own arithmetic reproduces. (Ratio written out per the standing rule from
the body-rig pass — no order-of-magnitude adjectives.)

**Label: MEASURED (counts from Epic's own editor), interpretation is the
speaker's OPINION.**

**This is directly actionable here and cheap:** the AI navigation graph is
a *measuring instrument for map complexity that does not require a
playtest*. If this project ever builds a nav graph for the new map,
node count and minimum branch-point count between two landmarks are a
readability metric, available at blockout time.

### Landmark theory — what is actually there, and what is not

**VM-12** gives Kevin Lynch's five mental-map elements verbatim: **Paths,
Edges, Districts, Nodes, Landmarks**. This is the lineage of essentially
all level-design landmark talk (Lynch, *The Image of the City*, 1960).

**VM-12's three composition guidelines**, given around the Half-Life 2
Citadel example:

> "1. **Players don't look upwards unless something draws their eye.**
> 2. Players look in the direction they are moving.
> 3. Players focus on contrast (in color, shape, lighting, and movement.)"

**Label: HEURISTIC, and note the attribution weakness** — VM-12 presents
these alongside a Valve example but cites no Valve primary. **SECOND-HAND
w.r.t. Valve. Do not attribute these to Valve.**

**Guideline 1 is the one that bites this plan.** A castle placed high on a
cliff is, by default, in the part of the frame players do not look at.

**VM-11's landmark section is a stub** — literally "TODO: examples". The
authors do leave two real assertions in it:

> "landmarks have to feel relevant and useful !! otherwise they don't
> function as landmarks!"
>
> "Actual landmarks (what the player notices during play) vs. fake set
> dressing landmarks (e.g. random skybox elements in The Last Of Us don't
> orient players, aren't actually useful)"

**Label: OPINION, unelaborated.** Recorded because it is a real
distinction and because the ledger should show that the canonical
practitioner reference on landmarks **has not been written yet**. If
anyone wants a landmark theory with evidence behind it, this ledger does
not have one and did not find one.

**VM-11 also takes an explicit minority position:** "leading lines"
(architecture and set dressing subliminally directing the eye) are called
"brain poison" and "a fake imagined phenomenon". The authors themselves
flag that this is **not the industry norm** and name Miriam Bellard's
GDC 2019 Rockstar talk as the credible opposing view. **Recorded as a
live disagreement, not settled.** I read neither eye-tracking study; I
cannot adjudicate it and am not going to.

### The wayfinding-aid table: DO NOT BUILD A RULE ON IT

VM-12 has a 20-row table of wayfinding aids with "% certainty" figures
(allegory 1% … static hard barriers 98%). **The authors state in their own
text that these are "a non-scientific estimate".** They are the strongest
temptation in this ledger and they are **NOT CARRIED as numbers.** The
*ordering* is useful; the percentages are vibes with a decimal point.

This is the same failure family as `aiming/SOURCES.md`'s fabrication
incident and `armor-damage`'s AD-09: a plausible-looking number in a
credible-looking table. Flagged loudly on purpose.

### The one credible defence of "deliberately messy"

**VM-14, Bennett Foddy on *zk map for stranger*** — a first-person game
about descending a haphazard vertical pile. Verbatim:

> "The core activity is route-finding. You look across a haphazard pile of
> shapes, and your eyes trace possible ways down, imagining the lines of
> sight and obstructions along each meandering path."
>
> "Route-finding is one part forward planning and two parts 'dead
> reckoning': making a choice and then figuring out how to get yourself one
> step out of the mess you just made."
>
> "You can't really get there by designing great routes … if the player is
> following a path you laid out for them, they aren't really route-finding
> at all."
>
> "You can construct a level that players can route-find through, but you
> can't design it … you can't crack out the Good Game Design if you want
> players to experience route-finding."

And the map author Marek Kapolk's own method, quoted by Foddy:

> "My process for making the levels was to scatter geometry more or less
> randomly and then try to traverse it. Sometimes when I was going down a
> map if I thought that an area shouldn't be a dead end I'd add some more
> stuff to it, but that's about as far as it went."

**Label: OPINION / critical essay, and a single non-commercial map.**

**Why it is in this ledger anyway:** it is the only source found that
argues *for* the owner's brief, and it states the price honestly. Messy
vertical geometry is not a failure of craft if **route-finding is the
activity you are selling**. But Foddy's claim is that you cannot have both
— authored flow and genuine route-finding are mutually exclusive. **That
is a decision the owner should make consciously rather than discover.**

**VM-03 [23:30] softens the cost specifically for this project's genre:**
"for multiplayer First Impressions aren't a huge deal — if players aren't
really sure where they're going or if they're slightly lost or they aren't
able to make a mental map of the whole layout of the level it's not a huge
deal because … a well-designed multiplayer map will guide" [caption
truncates]. Contrasted at [22:28] with single-player, where "if players
are lost then you should probably fix it."

**DERIVED: this is a TDM game, so the readability bar for getting lost is
genuinely lower than a campaign's — but the readability bar for
*spotting an enemy* (VM-01's figure/ground) is not lowered at all, and
that is the one a big vertical map threatens.**

---

## Q4 — castle on a cliff + half a city + open ground, in one map

**Direct practitioner sources on blending dissimilar area types in one
map: NOT FOUND.** I searched and did not get one. Saying so.

What the ledger can honestly offer is one theoretical frame and one
shipped warning.

**The frame (VM-12, via Lynch):** *Districts* — "areas which share similar
characteristics" — are one of the five elements people use to build a
mental map, alongside *Edges* ("roads which define the boundaries and
breaks in continuity") and *Nodes* ("strong intersection points"). On this
model, three sharply different area types are **not** a defect; districts
are how legibility is achieved. What must exist is a deliberate **edge**
between them and a legible **node** where they meet. Three districts that
fade into each other is the failure; three districts with an honest seam
is the intent.

**Label: THEORY (Lynch 1960, reached second-hand through VM-12). I did not
read Lynch. Do not cite this ledger as having read Lynch.**

**The shipped warning (VM-01 [21:37], the Gridlock result again):** it is
*tone and colour* that killed a map whose geometry was untouched. Three
districts is therefore three chances to lose figure/ground separately, and
one of them going muddy is enough — Gridlock was one map, and one bad
palette decision took it to the bottom of the popularity chart.

**DERIVED recommendation, mine:** the seams between castle, city and open
ground are the highest-value places to spend contrast — different value
range, not just different props — and the cheapest test is VM-01's, which
he performs live on stage several times: **blur the screenshot, or squint
at it.** He says at [28:27] "a simple check as easy as squinting could
have identified this problem quickly", and at [31:31] fixes a shipped map
in "literally less than one minute worth of effort" by lowering the clouds
and shrinking the central pyramid.

---

## Q5 — What breaks for AI/bots on a vertical map  ⚠ URGENT

### FIRST: what this codebase actually does today

Before any literature. Read-only inspection of `sim.rs` (I did not and
will not edit it):

- `Fighter` carries `pub waypoint: [f32; 2]` — **two components, x and z.
  There is no height in a bot's destination.**
- Waypoints are chosen (`sim.rs` ≈ L11976–12016) by **uniform random
  sampling inside a square** of half-extent `self.half`, with a team
  direction bias, then `clamp`ed to `[-half+3, half-3]` on both axes.
- Re-rolled when within 2.0 m of the current waypoint (`< 4.0` squared) or
  on a 15% per-think random.
- Grep across `src/` for `navmesh|nav_mesh|pathfind|path_find|a_star|astar`
  (case-insensitive): **exactly two hits, and neither is code.**
  `main.rs:24132` is a false positive (the test name
  `head_glance_is_a_glance_not_a_stare` matching `a_star`), and
  `sim.rs:14758` is a comment in a morale test explaining that the fixture
  "had drifted into testing **PATHFINDING**" and was rewritten to start
  the squads in contact instead. **There is no navmesh, no path graph and
  no pathfinder in this codebase.**

  *(Correction, same session: I first reported "20 hits, all comments".
  That regex included `waypoint`, which matched the real bot field and
  several research markdown files, and the summary sentence was wrong on
  both count and character. Re-run narrowed and re-read. The corrected
  finding is stronger than the one I got wrong — per this repo's rule,
  "verified false" is not "never checked", and it is not "checked
  sloppily" either.)*

**So: bots steer in a straight line toward a randomly chosen 2D point that
is never checked for reachability, never checked against collision, and
has no height.** On a flat arena that reads as "roaming" and works. It is
the correct design for the map that exists.

**On a castle-on-a-cliff map it degenerates, and predictably:**

1. A sampled waypoint can land **inside the mountain, inside a castle
   wall, or on the far side of a cliff**. Nothing prevents it. The bot
   walks into the face and presses.
2. It will only leave when the 15% re-roll fires — so the failure is not
   permanent, it is **intermittent**, which is worse for diagnosis and
   worse to watch.
3. Bots have **no notion of "up"**. They cannot choose to go to the
   castle. They will reach high ground only by accident, when a sampled
   (x, z) happens to sit on a ramp they happen to be standing on.
4. The clamp is to a **square**, so on a non-square or non-convex playable
   area the clamp does not keep waypoints in bounds in any useful sense.

**Label: MEASURED — read directly from the shipping source this session.**

**This is the answer to "what can bots cope with", and it is not a
literature question.** The literature below says what it would take to fix
it; this says what happens if nobody does.

### What shipped studios actually give their bots — and what they refused to automate

**VM-07, Guerrilla, Killzone 3 multiplayer bots.** Not a navmesh — a
**waypoint graph** with per-waypoint **cover data** ("cover data is stored
that describes the cover available in each direction… automatically
created in an offline process"), plus a **strategic graph** that clusters
waypoints into *areas* for squad-level reasoning. The invariant is worth
copying verbatim:

> "A connection between two areas exists when there is a link between the
> waypoints of the two areas in the waypoint graph. **This ensures that
> when a path exists between areas in the strategic graph, it also exists
> in the waypoint graph, and vice versa.**"

**And the sentence the map builder needs most:**

> "The Commander AI uses a number of level annotations to make better
> map-specific strategic decisions. **We chose to manually place these,
> because they would require complex terrain reasoning to generate
> automatically but can be identified easily for each map by observing
> play tests.**"

The hand-placed set: **Regroup locations**, **Sniping locations**
("areas that have good visibility over key locations"), **Assassination
hiding locations** ("typically inside defensible buildings"), **Defend
locations**.

**Label: DESIGN CHOICE, shipped, with the reason stated.**

**This is the budget item.** A studio with a dedicated AI team, shipping
on PS3, decided that terrain reasoning good enough to find sniper perches
and defensible interiors automatically was **not worth writing**, and
hand-placed them per map instead. A castle-on-a-cliff map is exactly the
geometry that makes automatic derivation hardest — and exactly the
geometry that makes those annotations most valuable.

**VM-09, Bungie, Halo, 2002 — the same answer, twenty-four years ago.**
Slide 22, "Location, Location, Location":

> "**'This is my goal. Where should I be standing?'** — Need a discrete
> answer to a continuous problem. **Solution: Firing Points** — weighted
> and selected [by] line of sight, distance to target, proximity of cover,
> friends and enemies, vehicles, grenades, etc."

Slide 18 names the level-design side: "**Strategic Spaces** —
Interconnectivity, Killing Zone, Attacking/Defending States, Aggressive
Territory, Retreat Conditions, Defensive Fortification."

**Two independent studios, two decades apart, converge on: give the AI a
discrete set of authored positions, because deriving them from the
geometry is the hard part.** That is the strongest cross-checked finding
in this section.

*What the slides could not tell me:* how firing points were authored (by
hand, by tool, or both), how many per level, and everything Butcher and
Griesemer said out loud. Slide 24 is just "Demonstration". VM-09 also
carries two real playtest tables (slide 16) — weak enemies rated
12% too hard / 52% about right / 36% too easy and 8/72/20 on perceived
intelligence; tough enemies 7/92/0 and 43/57/0 — supporting the slide's
own claim "**Smarter = Tougher, Tougher = Smarter**". Off-topic for maps,
recorded because it is measured and rare.

### Navmesh on cliffs: the parameters that decide it, and what they throw away

**VM-08** is the clearest statement of why a cliff map loses navmesh. A
voxel navmesh is generated from agent parameters, and the coverage is
"constrained by the physical world: **ledges that are too high to jump
over, slopes that are too steep to walk up**, and walls that block the way
forward should be omitted from the final navmesh."

The governing parameters, from the chapter's example table:

| Parameter | Standing value | What it excludes |
|---|---|---|
| `radius` | 0.4 m | anything narrower |
| `height` | 2.0 m | anything lower |
| `maxStepHeight` | 0.5 m | any ledge taller than this |
| `maxSlopeRad` | 0.5 rad = **28.65°** | any slope steeper than this |

**Label: ILLUSTRATIVE — the chapter says "an example of what an
agent-driven parameter table might look like". These are NOT shipped
values from a named game. Do not treat 0.5 rad as a standard.**

**CONTRADICTION-1, recorded not averaged.** `maps/SOURCES.md` S-01 (Level
Design Book metrics) says "modern stairs should follow a **30–35 degree**
slope". VM-08's illustrative walkable-slope limit is **28.65°**. Taken at
face value, **a staircase built to the recommended human-comfort slope is
steeper than the example navmesh will walk up.** These do not actually
conflict — voxel navmesh generators resolve stairs through
`maxStepHeight` per riser, not through the aggregate ramp angle — but the
two numbers are 1.05–1.22× apart and someone will eventually put a smooth
ramp at 33° and find bots refuse it. **Whoever owns navigation must
check which of the two paths a given piece of castle geometry takes.**
Unresolved here; flagged.

**The honest gap in VM-08:** its whole technique is about recovering space
by *shrinking the agent* — crouch, prone, swim, sidestep — with per-pass
parameter tables and triangle metadata. **It does not address cliffs,
drops or jumps at all.** Recovering vertical connectivity across a ledge
needs off-mesh / jump links, which this chapter never mentions.
**Nothing in this ledger tells you how to author a drop-down link.**

### If flight ever reaches the bots — the shape of the problem

**VM-05 (Brewer, Warframe) names the two obvious approaches and rejects
both, with reasons:**

Stacked navmesh layers:
> "A series of navmeshes can be created at various heights above the
> ground. Special flight-links can be used to connect these meshes… **This
> technique can work well in confined spaces such as indoors or for
> creatures restricted to hovering near the ground.** In very large volumes,
> such as a 2 km by 2 km by 2 km cube in an asteroid field, **it becomes
> impossible to decide how many layers of NavMesh will be required** to
> cover the volume adequately."

3D waypoint graphs:
> "There are a limited number of connections between volumes, which
> results in **unnatural flight-paths as agents deviate to go back to a
> specific connection**… the graphs are typically made by hand and are
> therefore static and cannot easily adapt to changes in the level."

Regular 3D grids: "A 3D regular grid covering the aforementioned 2 km cube
at 2 m resolution would require **a billion grid locations!**" — and
VM-06's slide states the memory form of the same fact: "2Km cube at 1m
resolution uses **8 Gb!**" (2000³ = 8×10⁹ cells; the two statements are
consistent).

**The boundary condition is the useful part, and it favours this
project.** Brewer's rejection of layered navmesh is explicitly scoped to
*very large volumes*; he says layers "work well in confined spaces… or for
creatures restricted to hovering near the ground". **A castle, a
half-city and some open ground is not a 2 km asteroid field.** If flight
comes to this game and bots ever need it, **stacked navmesh layers with
explicit links is the approach the source endorses for our scale** — and
VM-10's "three floor planes" is a natural place to put the layers.

**Measured, from VM-06's slides (Warframe, "Typical Level", 1024³):**

| Quantity | Value |
|---|---|
| collision polygons | 481,417 |
| equivalent regular-grid nodes | 1,073,741,824 |
| octree layers | 8 |
| octree nodes / leaf nodes | 43,648 / 30,800 |
| pathfind nodes | 2,014,848 |
| memory | 2,398,960 bytes (≈2.29 MiB) |

**A* tuning ladder on their "Complex Case" (VM-06 slides, MEASURED):**

| Configuration | Iterations | Path steps |
|---|---|---|
| A\*, node centres, straight-line | 32,916 | 50 |
| A\*, face centres, Manhattan | 10,692 | 57 |
| Greedy A\*, face centres, Manhattan | 3,378 | 58 |
| Greedy A\*, face centres, Manhattan, size compensation | 2,425 | 49 |
| Greedy A\*, node centres, straight-line, size compensation | 1,625 | 59 |
| Greedy A\*, node centres, straight-line, size comp., **unit node cost** | **213** | **42** |

**32,916 / 213 = 154.5×** end to end, and the final configuration also
produced the *shortest* path in steps (42 vs 50). Ratio written out per
the standing rule. VM-05's prose describes this as "at least an order of
magnitude" then "an extra order of magnitude"; **the slides' raw counts
are the better source and they say 154.5×, not 100×.**

**Tried and rejected, VM-05:** 3D Jump Point Search. "We attempted this
approach and found a great reduction in the number of nodes expanded
during the search, **however the time taken for the search was actually an
order of magnitude slower** than the tweaked heuristic A\*… in 3D, this
becomes an O(n³) flood fill instead of O(n²)." He also flags his own
implementation as "admittedly quite naive and unoptimized" — recorded with
that caveat attached.

**Also from VM-06 slides, for whenever flying agents need to avoid each
other:** "ORCA assume instant velocity changes / Real vehicles have
momentum and inertia / Additional constraints for maximum attainable ΔV /
Increase time horizon to find obstacles earlier."

*What the slides could not tell me:* VM-06 has a slide titled
"**Doughnut of Doom!**" with no readable body text, and another called
"Leap Ahead and Catch Up". VM-05's prose covers the second
("leap-ahead-and-back-fill"); **the Doughnut of Doom is a named failure
case whose content exists only in the narration and is lost to me.** If
anyone ever gets audio, that slide is the thing to listen for.

---

## Contradictions with the plan as briefed

Recorded, not averaged away.

1. **"Bigger" is the specific thing VM-01 says destroyed a map.** Three
   times the size, same-size players, "individual players and important
   gameplay elements became more difficult to identify" [37:16]. The plan's
   size increase has no counterpart increase in player size.
2. **"Messy" is defensible but expensive, and the sources disagree about
   for whom.** VM-02's *Sandbridge* — deliberately built to break Hi-Rez's
   own rules, with "really elaborate flank routes and these really dominant
   sniper towers and there are portals in the map" — was "**one of our
   highest rated maps in the test queue**" while competitive players said
   "this is why I don't play the test queue this is terrible" [23:05].
   Messy tested *well* with the casual majority and *badly* with the
   competitive minority. **Whose map is this?** is a question the sources
   cannot answer for us.
3. **Building for flight before flight exists is endorsed by the one
   studio that did it both ways.** VM-04's team built Heart of Thorns
   glider-first ("we worked hard to make the tall trees and towering
   ravines of the Heart of Maguuma glider friendly") and retrofitted
   Central Tyria, and said outright they would not redo the retrofit
   properly. **The plan's premise is sound.** What is missing is Yoder's
   what-you-see-is-what-you-get rule, which has to be applied *while*
   blocking out, not after.
4. **Three floor planes per area is a soft ceiling the plan may exceed by
   design.** Not fatal — the rule is per-area — but "big height
   differences" plus a castle plus a cliff plus a city is easily 4–5
   stacked planes in the castle approach alone. Nobody has measured what
   happens past three; VM-10 only asserts a fourth "doesn't add new
   dynamics."
5. **The bots cannot cope. Not "will struggle" — cannot.** See Q5 §1.
   Two-component random waypoints with no reachability test is a flat-arena
   design. This is the only item in this ledger that is a defect rather
   than a risk.
6. **Deliberate messiness plus flight plus altitude bands is three
   readability risks stacked**, and VM-01's Gridlock result says one bad
   palette decision is sufficient to cash them all in at once.

---

## What I would read next

1. **VM-16, Mononen, "Automatic Annotations in Killzone 3 and Beyond"** —
   the primary on generating cover and annotations *from geometry*,
   i.e. the thing VM-07 says Guerrilla refused to do for strategic
   markers but evidently did do for cover. Slides reportedly on
   `digestingduck.blogspot.com`. **Highest value, and it is a blog, so it
   should be reachable.**
2. **A Respawn or DICE primary on flight/vertical map design** (VM-18).
   Now that the YouTube-transcript route works, the GDC channel should be
   swept for level-design talks by title before concluding it does not
   exist.
3. **Recast/Detour off-mesh connections and jump links** — the documented
   gap in VM-08. **This is a code-reading job: hand it to TOTO22**, per
   the split. The question is "what does a drop-down link cost to author,
   and can it be generated from a cliff edge?"
4. **Bellard, "Environment Design as Spatial Cinematography", Rockstar,
   GDC 2019** — the credible opposing view to VM-11's leading-lines
   position, named by VM-11 itself. On YouTube, so transcriptable.
   Would let this ledger stop saying "unresolved".
5. **Kevin Lynch, *The Image of the City* (1960)** — currently reached
   only second-hand through VM-12. Districts/edges/nodes is the entire
   theoretical basis of the Q4 answer and nobody here has read it.

---

## Quota

**14 counted (VM-01…VM-14). Tier V: 5 — VM-01, VM-02, VM-03, VM-06,
VM-09.**

**Tier-V is no longer 0 in this repository.** The route is documented
above and applies to every other ledger.
