# Interaction Features Prior-Art Note (2026-10-20)

Prior art for five planned creature-interaction features:

1. colour sensing;
2. signal channels;
3. energy donation;
4. evolvable storage capacity;
5. carrying and placing objects.

The note closes with a paragraph on action-space design. Each feature section
gives precedent with short quotes, lessons, pitfalls and a one-line natural
analog.

**Source labels.** Every source carries one of these labels:

- **[full]**: the primary text was read.
- **[abstract]**: only the abstract was read (Europe PMC or arXiv).
- **[secondary]**: the claim rests on a search snippet or a citing paper. Treat
  it as unverified.

Quotes are at most 15 words. Everything else is paraphrase.

**Local facts used** (read from the code on 2026-10-20):

- Colour is six HSL channels per creature, mapped to RGB in
  `crates/v3-core/src/mutation/phenotype.rs::channels_to_rgb` and mutated one
  channel at a time per birth. It is not linked to any brain gene.
- `StealEnergy` is a directed vote sink. `PredationConfig::steal_cost_rate`
  defaults to 0.05.
- `max_energy` is a single world constant (200).
- Motor kinds use kind drives plus per-direction sinks and ring projections
  (T11.F30, T11.F31; see
  [factored-motor-output-research-2026-10-11.md](factored-motor-output-research-2026-10-11.md)).

---

## 1. Sensing other creatures' colour

### Precedent

- **Polyworld (Yaeger 1994) [full].** Polyworld gives each organism a
  one-dimensional colour retina. Vision is rendered at 22×22 and read as a
  strip, and the number of neurons per colour evolves independently, typically
  1 to 16. Display colour is partly genetic and partly behavioural:
  - green comes from an ID gene ("The ID gene's only function is to provide the
    green component");
  - red shows the fight drive;
  - blue shows the mate drive.

  Yaeger notes that seeing each other "could, in principle, support mimicry."
  The reported evolved behaviours that use vision are locomotor: speeding up
  at visual stimuli, following, and suggested flocking. No colour-based social
  discrimination is reported. Citation: L. Yaeger, "Computational Genetics,
  Physiology, Metabolism, Neural Systems, Learning, Vision, and Behavior or
  PolyWorld: Life in a New Context", *Artificial Life III*, SFI Studies XVII,
  Addison-Wesley, 1994, pp. 263–298. Text mirror:
  https://mirror.cyberbits.eu/textfiles.com/computers/CYBERSPACE/yaegerxa.txt
  (PDF: https://shinyverse.org/larryy/Yaeger.ALife3.pdf).
- **Tag-based donation (Riolo, Cohen & Axelrod 2001) [full].**
  - Mechanism: agents donate to others whose tag is within their own tolerance,
    "sufficiently similar to themselves in some arbitrary characteristic."
  - Result: cooperation reaches a 73.6% donation rate at c = 0.1, b = 1.
  - Cycle: a dominant tag cluster forms. Its tolerance then drifts up, and a
    low-tolerance mutant with a similar tag invades and founds the next
    cluster. "The cycle continues indefinitely."
  - Conditions: cooperation needs at least 3 pairings per generation. With 1 to
    2 pairings it stays below 5%. It collapses once c/b exceeds 0.4 (Table 2:
    60% at 0.4, 25% at 0.5, 2% at 0.6).

  Citation: *Nature* 414:441–443, doi:10.1038/35106555,
  https://public.websites.umich.edu/~axe/research/EC_wo_reciprocity.pdf
- **Critique (Roberts & Sherratt 2002) [secondary].** In the Riolo model an agent
  must donate to an identical tag. When tolerance may go below zero, so that
  agents can refuse even identical tags, cooperation is reported to collapse.
  Citation: "Does similarity breed cooperation?", *Nature* 418:499–500,
  doi:10.1038/418499b. Riolo et al. replied in the same issue (418:500).
- **Greenbeards (Gardner & West 2010) [full].**
  - Greenbeard versus kin recognition: a greenbeard links beard and behaviour
    loci, while genetic kin recognition uses a marker that tracks genealogy.
  - The falsebeard risk: cheats that show the beard without cooperating "could
    invade and overrun the population."
  - Where greenbeards survive: mostly in microbes, where a simple
    genotype–phenotype link may "prohibit the evolution of falsebeards."

  Citation: "Greenbeards", *Evolution* 64:25–38,
  doi:10.1111/j.1558-5646.2009.00842.x,
  https://www.icts.res.in/sites/default/files/Gardner_Evolution_2010_greenbeards.pdf
- **Beard chromodynamics (Jansen & van Baalen 2006) [abstract].** If the tag and
  altruism are always inherited together, altruism is lost. If they are carried
  by loosely coupled separate genes, "many beard colours co-occur" and altruism
  persists even in weakly structured populations. Citation: *Nature*
  440:663–666, doi:10.1038/nature04387.
- **Kin feeding with a kin cue (Taylor-Davies et al. 2024) [full].** Agents see a
  binary "is my offspring" flag for each visible agent. Disabling that flag
  weakened kin-selective feeding, but population viscosity mattered more (see
  §3). Citation: arXiv:2411.10536, https://arxiv.org/abs/2411.10536

### Lessons

- **Petri's colour is a kin label, not a greenbeard.** It is a neutral marker
  that mutates per birth and is unlinked to behaviour genes, so it tracks
  genealogy. This is Gardner & West's genetic kin recognition (phenotype
  matching), and Jansen & van Baalen's loosely coupled case, the one that
  stabilises. Falsebeards are not a structural risk, because no gene says
  "I am a helper". The open risk is that colour divergence is too slow or too
  fast relative to lineage turnover, so that colour carries little relatedness
  information.
- **Is heritable colour plus colour vision enough for signalling to evolve? No.**
  - It is enough for kin discrimination, provided relatives are locally common
    (viscosity) and some interaction (donation, steal, avoidance) rewards
    discrimination.
  - Polyworld had a colour retina plus a behaviour-driven display (red and
    blue), and no colour-based social use was reported.
  - Signalling needs a context whose payoff depends on the receiver's response
    (§2).
- **Per-channel scalar versus retina.**
  - Polyworld's evolvable retina gave locomotor use only.
  - Locally, the ring sensors are the ones creatures actually read. The
    2026-10-12 live sensor survey found only OccRing→Move avoidance in use, and
    `Nearest1.present` nearly constant.
  - So prefer a per-direction ring reading (one value per neighbour cell) over
    a retina or a single nearest-neighbour reading. This also fits the T11.F31
    projection gene.
- **Report colour relative to self.** A "colour distance to self" per
  direction (self-referent phenotype matching) makes kin discrimination one
  edge. Absolute RGB per direction needs the brain to compute a comparison
  first. If absolute channels are kept, give hue, not RGB. Mutation hits one
  of six channels, two per hue, saturation and lightness, so drift is spread
  evenly. But hue maps the full wrapping 0–255 range
  (`channels[0] + channels[1]`), while saturation and lightness are compressed
  to [0.35, 1] and [0.25, 0.75]. Hue therefore shows the widest visible change
  per mutation.

### Pitfalls

- **Discrimination decays without use (Riolo).** Tolerance drifts upward when
  nothing selects for it. An unread colour sensor will be lost or never wired.
- **Optional refusal can break tag cooperation (Roberts & Sherratt).** A design
  where an identical colour forces help is not a natural analog. Petri's
  discrimination should be evolved, which means it can fail.
- **Interaction count matters.** Riolo's cooperation fails at 1 to 2 pairings
  per generation. If neighbour encounters per lifetime are rare, colour use
  cannot be selected.

### Natural analog

Self-referent phenotype matching, the "armpit effect": a golden hamster or a
Belding's ground squirrel compares a neighbour's cue with its own.
*(Analog named from general knowledge. Hauber & Sherman was not retrieved; a
Mateo & Johnston 2003 review with the same theme exists, doi:10.1007/s10071-003-0165-z,
title seen only.)*

---

## 2. Evolved communication channels

### Precedent

- **Floreano, Mitri, Magnenat & Keller 2007 [abstract].** Robots signal with a
  light.
  - Communication "readily evolves when colonies consist of genetically similar
    individuals" under colony-level selection.
  - Distinct systems emerged, and an established one "constrained the evolution
    of more efficient" ones.
  - Under individual selection, signalling led to "deceptive communication
    strategies" among unrelated robots, and colony performance fell.

  Citation: *Current Biology* 17:514–519, doi:10.1016/j.cub.2007.01.058.
- **Mitri, Floreano & Keller 2009 [full].**
  - Setup: a blue light, explicitly "cost-free", and no kin structure.
  - Early random emission near food leaked information. Competing robots were
    "quickly selected to conceal this information."
  - They never fully stopped. The outcome was "a stable equilibrium with low
    information."

  Citation: *PNAS* 106:15786–15790, doi:10.1073/pnas.0903152106,
  https://pmc.ncbi.nlm.nih.gov/articles/PMC2747196
- **Mitri, Floreano & Keller 2011 [abstract].** Relatedness and signal
  reliability correlate strongly. Signal frequency is "a poor predictor of
  signal reliability". Citation: "Relatedness influences signal reliability in
  evolving robots", *Proc R Soc B*, doi:10.1098/rspb.2010.1407.
- **Wagner, Reggia, Uriagereka & Wilkinson 2003 review [full].** Citation:
  *Adaptive Behavior* 11:37–69, doi:10.1177/10597123030111003,
  https://science.umd.edu/faculty/wilkinson/Wagneretal03.pdf. Findings:
  - **Kin and space.** Across studies, "Kin selection and spatial constraints"
    encourage altruistic food and alarm calls. Placing offspring near parents
    favoured food calls, while dispersal broke them up.
  - **Cost.** In Wagner (2000), with free signals, agents "evolved to emit
    signals constantly", and receivers ignored them. A cost restored meaning.
    Food calls also needed moderate density: at high density, wandering found
    partners and signals paid nothing.
  - **Built-in honesty.** In Reggia et al. (2001), anyone who heard alarm calls
    also sent them. The review calls that "no cheating was possible", an
    important limitation.
  - **Non-signalling solutions win.** In Werner & Dyer (1994), with predators,
    food and mates, "signaling did not evolve" because a non-communicative
    solution won. Noble (1998) found the same for intention displays.
    de Bourcier & Wheeler (1995) found reliable signals only when costly or
    when they were the only information source.
  - **Werner & Dyer (1991)**, as summarised in the review: females that could
    see but not move signalled to blind males. Signals came to encode
    directions ("turn left", "straight ahead"), and competing dialects formed.
    *Artificial Life II*, pp. 659–687 (1992 volume); primary text not read.
- **Ackley & Littman 1994 [abstract via search snippet].**
  - Communication range was made to approximate breeding range, "you'll be
    talking to family," so kin selection could favour it.
  - Altruistic communication then opened niches for "information parasites."

  Citation: "Altruism in the evolution of communication", *Artificial Life IV*,
  pp. 40–48.
- **Quinn 2001 [full intro].** ALife models had implemented communication over
  "dedicated and functionally isolated channels". Quinn argues this cannot
  explain origins. In his model, signals grew out of movement behaviour that
  was already functional. Citation: ECAL 2001, LNAI 2159:357–366,
  https://langev.com/pdf/quinn01evolvingCommunication.pdf
- **Polyworld [full].** Polyworld has a lighting neuron that sets the
  brightness of the organism's front polygons, so signalling was possible in
  principle. Yaeger: "No evidence of their use of this form of communication"
  had been found or sought.
- **JaxLife (Lu, Beukman, Matthews & Foerster 2024) [full].** Each agent sends a
  self-message to its closest agents. Over a run, saliency of the
  communication input rises gradually. With 256 agents it stayed "relatively
  constant at near zero". Source: arXiv:2409.00853,
  https://arxiv.org/abs/2409.00853
- **Signalling theory.**
  - Grafen 1990 [abstract] formalises the handicap principle: a signalling ESS
    satisfies its conditions. *J Theor Biol* 144:517–546.
  - Lachmann, Szamado & Bergstrom 2001 [abstract]: "signal cost is
    unnecessary for honest signaling even when interests conflict". What
    matters is the "cost of deviation from equilibrium". *PNAS*
    98:13189–13194, doi:10.1073/pnas.231216498.

### Lessons

- **Relatedness is the main switch.** Every positive result in the robot and
  ALife record has kin structure or shared payoff:
  - Floreano 2007, Mitri 2011, Ackley & Littman, and the review's spatial
    results all show it.
  - Without kin structure, signals become deceptive (Floreano 2007) or are
    suppressed to low information (Mitri 2009).
  - Petri's births are local, so viscosity exists. The question is whether
    signal range matches family range (Ackley & Littman).
- **Cost.**
  - A free signal with conflicting interests degrades to constant noise
    (Wagner 2000) or to concealment (Mitri 2009).
  - Lachmann et al. show that what must be costly is deviation. A cheater must
    pay through the receiver's response.
  - For Petri this argues for a small production cost (natural: muscle or
    chromatophore work), such as the existing per-action floor. A large
    handicap tax would mostly suppress use.
  - Honesty then comes from receivers evolving to ignore unreliable senders,
    which needs the colour/kin cue from §1.
- **How many channels.** No controlled study of channel count was found. Every
  successful case used one signal, sometimes graded (a light; a female's
  sound). JaxLife's message vector was ignored at large population. Start with
  one channel and add more only once it is read.
- **Persistence.** No comparative evidence was found. The successful robot
  light was a state that stayed on while the controller drove it. Polyworld's
  lighting neuron was the same. A held level (refreshed every tick) is the
  established form. A pulse that self-erases has no precedent here.
- **Range and broadcast versus directed.** All positive cases are broadcast to
  whoever is near, with range at about the breeding range. JaxLife's
  closest-agent send is the only directed case, and it went unused at scale.
  Broadcast to the eight-neighbour ring keeps range at the kin scale.
- **Dedicated channel or existing body (Quinn).** A dedicated abstract channel
  has no use before receivers exist. Polyworld's channel modulated
  already-visible colour. The cheapest bootstrapping route for Petri is to let
  creatures change an already-sensed property, such as colour lightness or a
  "flash", rather than a new invisible register. Then any colour reader from §1
  is already a receiver.

### Pitfalls

- **Non-signalling solutions usually win (Werner & Dyer 1994; Noble 1998).** An
  unused channel is the expected default, not a bug. Predeclare readings
  (saliency, the share of genomes wiring a signal input) and accept zero as a
  finding.
- **Information parasites and deception** (Ackley & Littman; Floreano 2007) when
  relatedness is low.
- **Early lock-in (Floreano 2007).** The first system to establish blocks
  better ones.
- **Density.** Too high and signals pay nothing (Wagner 2000; JaxLife at 256
  agents). Too low and nobody hears.
- **Frequency is not information (Mitri 2011).** Measure reliability, not emission
  rate.

### Natural analog

Cephalopod chromatophore display or firefly flashing: a creature modulates its
own visible body colour. Chromatophore muscles cost energy to hold. *(Analog
from general knowledge; not sourced here.)*

---

## 3. Energy transfer to a neighbour

### Precedent

- **Taylor-Davies, Hamon, Boulet & Moulin-Frier 2024 [full].** This is the
  closest template.
  - World: a 100×100 grid, one agent per cell, with neuroevolution and no
    explicit fitness.
  - Action: "feed" moves one plant's energy (20 units) to an adjacent agent the
    feeder faces. It is zero-sum, and any non-idle action costs 1.0.
  - Inputs: a binary "is an infant" flag and a binary "is my offspring" flag for
    each visible agent.
  - Result: zero-sum transfer to offspring evolves "in environments where it is
    hard for offspring to survive alone". Both the amount and the selectivity
    of feeding scale with benefit.
  - Ablations: removing kin recognition or randomising birth location weakened
    feeding, and "population viscosity is more important".

  Citation: arXiv:2411.10536, https://arxiv.org/abs/2411.10536
- **Polyworld [full].** Polyworld's only transfer is at birth. A physiology gene
  sets the fraction of remaining energy the parent gives the offspring, and
  both parents contribute.
- **Echo (Hraber, Jones & Forrest 1997) [full].**
  - Trade needs mutual tag matching.
  - Each agent contributes only its "excess" resource: whatever it holds above
    the amount needed to replicate, plus a reserve. That amount may be zero, so
    the partner can get nothing.
  - Agents also have "a reservoir, or stomach".

  Citation: *Artificial Life* 3:165–190,
  https://www.cs.unm.edu/~forrest/publications/echo-97.pdf
- **Vampire bats.**
  - Wilkinson 1984 [secondary: abstract via search]: sharing depends "equally
    and independently" on relatedness and opportunity for reciprocation.
    *Nature* 308:181–184, doi:10.1038/308181a0.
  - Carter & Wilkinson 2013 [abstract]: under controlled conditions, food
    received was "8.5 times more important than relatedness". 64% of sharing
    pairs were unrelated. *Proc R Soc B*, doi:10.1098/rspb.2012.2573.
- **Trophallaxis (LeBoeuf et al. 2016) [abstract].** Ant trophallactic fluid
  carries hormones and proteins as well as food, which suggests the act doubles
  as communication. *eLife* 5:e20375, doi:10.7554/elife.20375.
- **Viscosity theory (Mitteldorf & Wilson 2000) [abstract].**
  - Limited dispersal focuses gifts on kin but also makes kin compete.
  - With fixed density the two effects can cancel.
  - When density may fluctuate, so that patches of altruists grow denser,
    viscosity supports altruism.

  Citation: *J Theor Biol*, doi:10.1006/jtbi.2000.2007.
- **Kin selection.** Hamilton 1964, "The genetical evolution of social
  behaviour I/II", *J Theor Biol* 7:1–52 (citation only; Hamilton's rule
  rB > C).
- **Cost/benefit bound (Riolo 2001, Table 2) [full].** Tag-based donation holds
  at c/b ≤ 0.3 and collapses above 0.4. Here c and b are fitness units, not
  energy units: c = 0.1, b = 1 is a tenfold fitness return. This is an analogy
  for Petri, not a measurement.

### Lessons

- **Parental care first.** The only evolved-neural grid precedent (Taylor-Davies)
  got transfer to evolve as parent-to-infant feeding, where relatedness is
  highest and infant survival is hard. Petri's local births give the
  viscosity. Its offspring-energy rules decide whether infants need help. If
  newborns are already provisioned to self-sufficiency, donation has little to
  select it.
- **Transfer efficiency and where donation pays.**
  - A lossless energy transfer has c/b = 1 in energy units, which would be deep
    in Riolo's collapse zone. Yet zero-sum feeding evolved in Taylor-Davies.
  - Hamilton's rule reconciles the two: it is counted in fitness. A unit of
    energy is worth far more to a near-starving infant than to a fed parent,
    so the fitness ratio stays well below 1 even when energy is conserved.
  - Two consequences:
    - Donation pays only where the recipient's marginal value of energy is
      much higher than the donor's (newborns, patchy food, famine).
    - Transfer should be lossless or nearly so, because any loss pushes the
      fitness c/b toward collapse.
  - Recommended form: lossless transfer plus the existing per-action effort
    floor.
  - Natural analog: regurgitation loses little. Bats give a blood meal.
- **Give from surplus (Echo).** Tie the amount to what the donor holds above a
  reserve, like a bat regurgitating from a full stomach. A fixed amount
  argument can starve the donor.
- **Recognition plus viscosity.** Pair donation with §1's colour-distance input.
  Taylor-Davies shows the cue helps but viscosity carries most of the effect.
- **Relation to steal.**
  - Donate is the same physical act as steal, reversed: energy crosses between
    adjacent creatures in one chosen direction.
  - Mirror the steal sink structure: a kind drive plus 8 directions, with the
    same cost floor. This also makes the two acts symmetric for readings
    (net flow per pair).
  - Local memory warns that every live steal requests 0 energy (predation
    inert). A Give with a free scalar amount argument inherits that risk; see
    the action-space paragraph.

### Pitfalls

- **Exploitation by non-donors.** In tag models, a low-tolerance mutant similar
  enough to receive but not give invades (Riolo). With colour as a genealogy
  marker, this is ordinary kin selection with recognition error.
- **Viscosity cancellation (Mitteldorf & Wilson).** Kin compete locally for the
  same food, so donation gains can be cancelled unless helped patches can grow
  denser.
- **Reciprocity needs memory** (bats). Petri creatures carry no partner identity
  memory, so the realistic route is kin and viscosity, not reciprocity.
- **A free gift with no floor cost** lets a Give–Steal loop between kin cycle
  energy for nothing. Both acts should pay the floor.

### Natural analog

Parental provisioning and regurgitation feeding: a bird feeding nestlings, ant
trophallaxis, a vampire bat sharing a blood meal.

---

## 4. Evolvable energy storage capacity with a movement-cost tradeoff

### Precedent

- **Polyworld (Yaeger 1994) [full].** One size gene sets several things.
  - Capacity: "The size of the organism directly affects the maximum amount of
    energy" it can store, by a linear interpolation:
    `ECap = minECap + (size − minSize) · (maxECap − minECap) / (maxSize − minSize)`.
  - Movement cost: "Similar linear relations" set how size affects energy spent
    on forward and turning movement.
  - Fighting: size also gives a fight advantage.
  - Yaeger frames the tradeoff as "clear energy conservation benefits to being
    small and weak" against "clear predatory advantages to being large".
  - No size trend was reported, and he lists tracking it as future work.
- **Cost of transport (Taylor, Heglund & Maloiy 1982) [abstract].** From 62 bird
  and mammal species, mass-specific locomotion cost is
  `E/Mb = 10.7·Mb^−0.316·v + 6.03·Mb^−0.303` W/kg. Total locomotion cost
  therefore grows sub-linearly with mass, about Mb^0.68–0.70, so between
  species, larger is cheaper per kg. *J Exp Biol* 97:1–21,
  doi:10.1242/jeb.97.1.1. Schmidt-Nielsen 1972, "Locomotion: energy cost of
  swimming, flying, and running", *Science* 177:222–228 (citation only; content
  not retrieved).
- **Carrying a load (Maloiy et al. 1986) [abstract].**
  - Running animals pay in direct proportion to load: a 20% body-weight load
    raised energy use by 20% (horses, humans, dogs, rats).
  - Luo and Kikuyu women carried up to 20% for free, and paid proportionally
    above that.

  *Nature* 319:668–669, doi:10.1038/319668a0.
- **Fat versus predation.**
  - Witter & Cuthill 1993 [secondary]: the real title is "The ecological costs
    of avian fat storage", *Phil Trans R Soc B* 340:73–92. Citing papers report
    it as the balance between starvation risk and the flight-cost and
    manoeuvrability cost of fat.
  - Lima 1986 [secondary]: the starvation–predation model of optimal winter
    mass. *Ecology* 67:377–385.
  - Gosler, Greenwood & Perrins 1995 [secondary]: great tits carried less fat
    when sparrowhawks were common. The change was individual adjustment, not
    selection. *Nature* 377:621–623.
  - McNamara et al. 2005 [abstract]: depending on the predator, optimal mass
    can rise or fall after a predator encounter. *Proc R Soc B*,
    doi:10.1098/rspb.2004.3037.
- **Reserve versus structure (Kooijman & Troost 2007, DEB theory)
  [secondary].** Reserves "do not require maintenance", while structure carries
  maintenance costs. https://www.bio.vu.nl/thb/research/bib/KooyTroo2007

### Lessons

- **Separate capacity from contents.** Polyworld charges size, so a large
  creature pays its movement penalty even when empty. Biology splits the cost
  in two:
  - **Structure (capacity)** pays maintenance: a small per-tick upkeep that
    scales with capacity (DEB structure, adipose tissue mass).
  - **Load (stored energy actually held)** pays transport: movement cost rises
    linearly with stored energy as a fraction of body mass (Maloiy/Taylor:
    20% load gives +20% cost).
- **Natural selection pressure from the world.** Petri already has an energy
  predator. If steal takes more from a fuller victim, mass-dependent predation
  risk (Lima, Gosler) arises from the world rather than from a rule.
- **Scaling exponents.** Between species, cost grows like M^0.7 (economy of
  scale), but within an individual a load costs linearly. For a capacity gene
  inside one body plan, use linear load cost and linear or near-linear upkeep.
  A sub-linear exponent on capacity would hand a monotonic advantage to large
  capacity: the runaway case.
- **Capacity's benefit is starvation insurance.** It only pays where food is
  patchy or seasonal. Petri's fruit episodes and boom-bust worlds supply that.
  In a uniform food world, expect capacity to shrink.

### Pitfalls

- **Runaway size.** A size gene that also buys fight advantage (Polyworld)
  stacks benefits. Petri's local history of genome-bloat extinction
  (2026-10-11) shows a free lever does run away. Keep capacity buying storage
  only.
- **Charging capacity always versus only when full** are different equilibria.
  Charging upkeep on capacity even when empty (Polyworld style) penalises
  insurance in good times. Charging only on load makes capacity nearly free,
  since contents are paid for anyway. Use both, with small upkeep, and measure
  the evolved capacity distribution.
- **Interaction with offspring energy.** If offspring inherit a fraction of
  parent energy, capacity also changes provisioning. Read it next to §3.

### Natural analog

Fat reserves in small wintering birds: insurance against starvation, paid for
in flight cost and predation risk.

---

## 5. Carrying and placing objects (food or barriers)

### Precedent

- **Stigmergy (Grassé 1959, via Theraulaz & Bonabeau 1999) [abstract].**
  Individuals "work as if they were alone" yet the collective work appears
  coordinated. The paper distinguishes quantitative from qualitative
  stigmergy. *Artificial Life* 5:97–116, doi:10.1162/106454699568700.
- **Distributed building (Theraulaz & Bonabeau 1995) [abstract].** Agents moving
  randomly on a 3D lattice build coherent structures from local rules. *Science*
  269:686–688, doi:10.1126/science.269.5224.686.
- **TERMES (Werfel, Petersen & Nagpal 2014) [abstract].** Climbing robots place
  bricks using local sensing and coordinate "via the shared environment". The
  rules are compiled from a target structure, not evolved. *Science* 343:754–758,
  doi:10.1126/science.1245842.
- **Collective sorting (Deneubourg et al. 1991) [secondary].** Pick-up
  probability is `(k1/(k1+f))²` and drop probability is `(f/(k2+f))²`, where f
  is the local density of items. Isolated items get picked up and dense spots
  get deposits. *Simulation of Adaptive Behavior* 1:356–365.
- **Niche construction.**
  - Laland, Odling-Smee & Feldman 1999 [abstract]: niche construction can fix
    "otherwise deleterious alleles". Even weak construction can alter
    ecological and evolutionary patterns. *PNAS* 96:10242–10247,
    doi:10.1073/pnas.96.18.10242. The book is Odling-Smee, Laland & Feldman
    2003, *Niche Construction*, Princeton (citation only).
  - Silver & Di Paolo 2006 [abstract]: in a spatial, individual-based model,
    niche-construction traits drive themselves to fixation. Local
    resource-mediated interaction links them to the recipient trait.
    *Theor Pop Biol* 70:387–400, doi:10.1016/j.tpb.2006.08.003.
- **JaxLife [full].** Agents "spend energy to alter the properties of the
  terrain". Terrain "slowly regress[es] to this base state". Agents evolved
  terraformed striped regions that raised food production. No pick-up or
  deposit action exists.
- **Food hoarding (Andersson & Krebs 1978) [secondary].** A game-theoretic model
  shows that hoarding evolves only if the hoarder has a recovery advantage over
  others. *Animal Behaviour* 26:707–711. Pilferage is the standard threat in
  the scatter-hoarding literature (Vander Wall 1990, *Food Hoarding in
  Animals*, citation only).
- **Load cost.** See §4 (Maloiy et al. 1986): cost is proportional to load.
- **Absences.** Avida and Polyworld have no carry action. The evolved
  construction found (Nitschke 2008, GECCO, gathering and collective
  construction [secondary]) is fitness-driven robot simulation, not open-ended
  ecology.

### Lessons

- **Slot count.** Every model found carries one item: Deneubourg's ants,
  Theraulaz's wasps, the TERMES robots. That is a modelling convention, not
  evidence about evolvability. One slot avoids a slot-index argument entirely
  (see the action-space paragraph) and has the clearest analog: a mouth or
  mandibles.
- **Carrying cost.** Use load cost on movement, linear in carried mass, which
  is the same rule as §4's stored energy. A carried food item can then be
  "stored energy outside the body". This unifies §4 and §5.
- **Placed barriers must decay or cost upkeep.** JaxLife's terraforming reverts
  to base. Permanent placed barriers would accumulate and freeze the map, a
  frozen-niche pitfall. Erosion, or an energy price for placement that scales
  with nearby barrier density, keeps it bounded.
- **Hoarding needs a recovery advantage.** Cached food is open to any creature
  (pilfering) unless the cacher is more likely to return. Viscosity (kin
  nearby) and local return paths supply that. Without it, caching is a gift
  to neighbours.
- **Action design.** Pick up and place should be directed kinds with the same
  drive-plus-8-directions shape as `Move`, `Reproduce` and `StealEnergy`, not
  a scalar direction argument.

### Failure handling

No literature covers this. Locally, T16.F02 makes every attempted action pay
at least the floor. Therefore:

- a failed pick-up (empty cell, or hands already full) pays the floor;
- a failed placement (occupied cell, or nothing carried) pays the floor;
- the carried item survives a failure.
- On death, drop the carried item on the death cell. This keeps food
  conserved.

### Natural analog

Food caching (jays, squirrels) for food. Nest material placement (termite
mound building, wasp nest building) for barriers.

---

## Action-space design for evolved controllers

The local record already answers this, and the external literature adds
little.

- **Local findings.** The 2026-09-16 note found that a rounded scalar-index
  decode had no evolved precedent. A PPO comparison (Kanervisto et al. 2020)
  showed a single continuous angle output learned slowest, and discretising it
  restored learning. Petri replaced its scalar direction with per-direction
  sinks, then added kind drives and projections (2026-10-11).
- **External findings.** Parameterised actions, a discrete action plus continuous
  arguments, work in gradient RL. Hausknecht & Stone 2016 [abstract] report
  "structured (parameterized) continuous action spaces" in RoboCup.
  Masson, Ranchod & Konidaris 2016 [abstract] give Q-PAMDP. These are RL
  results with gradients through the parameter head, not evidence for
  mutation-only search. For evolution, Kohl & Miikkulainen 2009 [secondary]
  show that neuroevolution struggles on "fractured" mappings, where the right
  output changes abruptly. Decoding a slot index or direction by rounding a
  continuous output creates exactly that fracture: one weight mutation flips
  the target.
- **Recommendation.**
  - **Direction.** New directed acts (give, pick up, place) use the established
    kind-drive plus per-direction-sink form, so a sensor ring reaches them
    through one projection gene.
  - **Slot index.** Avoid it by using one carry slot.
  - **Amount.** Use a fixed rule (surplus above a reserve, or a fixed
    fraction) or a graded drive strength, never a free scalar that is 0 when
    unwired. The 2026-10-12 live sensor survey found every live steal
    requesting 0 energy, which is that failure.
  - **Signal level.** It is a level, not an index, so a single continuous
    output is natural (Polyworld's lighting neuron).

Sources: [motor-output-encoding-research-2026-09-16.md](motor-output-encoding-research-2026-09-16.md),
[factored-motor-output-research-2026-10-11.md](factored-motor-output-research-2026-10-11.md);
Hausknecht & Stone, ICLR 2016, arXiv:1511.04143; Masson et al., AAAI 2016,
arXiv:1509.01644; Kohl & Miikkulainen, *Neural Networks* 22:326–337 (2009).

---

## Not retrieved or unverified

- Floreano 2007 full text: abstract only. The blue-light deception details come
  from press coverage and are not used here.
- Werner & Dyer 1991 and 1994: through the Wagner review only.
- Hauber & Sherman 2001: not retrieved.
- Wilkinson 1984 donor/recipient starvation economics: not retrieved.
- Witter & Cuthill, Lima, Gosler, Andersson & Krebs, Deneubourg, Kooijman: from
  citing papers or search snippets only.
- Sugarscape (Epstein & Axtell 1996) was not checked.
