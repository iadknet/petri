# How learned synaptic changes fade, persist or consolidate: natural analogs for Petri's per-edge weight decay

Research note for Petri (read-only research, 2026-10-19). The question: should a learned edge weight fade, what should it fade toward, should the fade rate be heritable per edge, and how does consolidation fit in?

## Bottom line

**Named analog for the spec:** decay of early-phase LTP/LTD back to the synapse's pre-induction baseline. This covers the LTP1/LTP2 decay families of Abraham 2003 and the tag reset in Clopath 2008. At organism level, Davis & Zhong 2017 frame it as intrinsic forgetting, which brings the brain "back to its basal state".

**Premise check:** the brief says plastic edges belong to groups. In the current code they cannot: `learned.rs` (`Slot.genomic`) says "a plastic edge is never a member" of a group. So the per-edge decay applies to an edge's own weight, and a decay path through the group scale is ruled out by the code as it stands.

1. **The best-grounded analog is the decay of early LTP/LTD back to the synapse's own pre-learning value.** In the models, that value is the "reference weight" or consolidated component. Petri's version would be: per edge, every tick, `w ← b + ρ·(w − b)`.
   - `b` is the slot's birth value. `ρ` is a heritable retention factor in [0, 1].
   - Direct support: Abraham 2003 (LTP "eventually returned to baseline"); Clopath 2008 (E-LTP "decays back to baseline"); Ziegler 2015 (a tagged synapse "decays back to its resting state within a few hours"); Zenke 2015 (each synapse "'remembers' its baseline value"); Leimer 2019 (the fast component "converges to baseline value").
   - Closest whole-organism analog: in *C. elegans* the synapse reverts toward the **naive** state (Hadziselimovic 2014, via Richards & Frankland 2017). The naive state is close to "genetically specified wiring".
2. **No paper I read supports decay toward literal zero.**
   - Where a model decays "to zero", the variable is a *deviation* from baseline: the Benna–Fusi u_k, or Leimer's fast component w_f. In absolute terms that is decay toward baseline.
   - Driving the whole weight to zero matches a different mechanism: synapse **elimination** or spine shrinkage ("weight elimination" in Richards & Frankland's ML mapping). That is not forgetting a learned change; it is removing the innate connection.
3. **Multiplicative scaling of all synapses on a neuron (Turrigiano) is a different mechanism**, not a substitute.
   - It acts per neuron, over hours to days, and preserves weight ratios ("without changing the relative strength of synaptic inputs").
   - Turrigiano also says a per-synapse homeostatic mechanism "would tend to erase the effects of LTP or LTD", a "forgetting" mechanism.
   - So per-edge decay toward baseline is the "forgetting" mechanism; neuron-wide scaling is not.
4. **There is real support for a heritable rate that differs by synapse class:**
   - Brea 2014: forgetting rates "can be considered to be genetically encoded", and the model's approach and avoid synapses decay at different rates.
   - Abraham 2003: three LTP decay families (τ ≈ 2.1 h, 3.5 d, 20.3 d), and persistence differs between the dentate gyrus and CA1.
   - Davis & Zhong 2017: Rac1 level tunes the forgetting rate, and the effect depends on memory type.
   - Caveat: biology regulates the rate mostly per neuron or circuit compartment, not per individual synapse. A per-edge rate is a finer grain than the evidence; see open questions.
5. **Consolidation slows decay by moving the change into a slow component. It does not make the change immune.**
   - Clopath 2008: `w = w̄(1 + h − αl + βz)`.
   - Leimer 2019: `w = w_f + w_s`; the decaying part of `w_f` is moved into `w_s` when a gate is open.
   - Ziegler 2015: a write-protected scaffold, opened by dopamine.
   - Abraham 2003: LTP that had been stable for 120 days was still reversed by heterosynaptic stimulation.
   - So a two-component weight (a fast decaying part on top of a slow consolidated part) is well grounded. A one-component decay toward a fixed `b` is its simplest case, with consolidation switched off.
6. **Trace decay and weight decay are separate mechanisms (Gerstner 2018).**
   - Petri's `trace_decay` is the eligibility flag e_ij: about 1 s in reinforcement-learning data, about 1 h for the synaptic tag.
   - A weight-decay parameter would be a second, separate timescale.
7. **Accuracy note on the stated gap.** "No passive forgetting" is correct for *activity-independent* decay. Petri's Oja rule already contains an activity-gated decay term, `−η·w·post²` (`rules.rs`).

---

## Sources (full text read unless marked otherwise)

**Quote locations.** XML/BioC/HTML sources have no page numbers, so quotes are tagged with section headings or sentence context. PDF sources give page numbers.

### 1. Benna & Fusi: "Computational principles of biological memory" (arXiv preprint 1507.07580v1, 2015)

- **Citation:** Benna MK, Fusi S. arXiv:1507.07580v1, 27 Jul 2015. https://arxiv.org/abs/1507.07580
- **Note:** This preprint was later published as *"Computational principles of synaptic memory consolidation"*, Nature Neuroscience 2016. I read the preprint, not the journal version.
- **How obtained:** arXiv PDF via curl, PyMuPDF; 67 pages, read in full.
- **Mechanism:**
  - Each synapse is a chain of coupled variables u_1…u_m. Only u_1 (the efficacy) is driven by plasticity.
  - The hidden u_k equilibrate bidirectionally with their neighbours, and the last one leaks.
  - Timescales grow geometrically along the chain (C_k ∝ 2^(k−1), g_{k,k+1} ∝ 2^(−k−2)).
  - Memory then decays as a power law, about 1/√t, rather than exponentially.
- **Equation (Fig. 1A):** `C_k du_k/dt = g_{k−1,k}(u_{k−1} − u_k) + g_{k,k+1}(u_{k+1} − u_k)`, with u_{m+1} = 0 as the leak.
- **Quotes:**
  - p. 2: "This initial memory trace is strong but labile; it decays quickly when other memories are stored. Memories can be consolidated if the information about each new memory is progressively transferred to the slow components."
  - p. 7, Fig. 1 legend: "The uk variables measure the deviation of the liquid level from equilibrium". The first beaker's level "represents the synaptic strength."
  - p. 13: "metaplasticity is an obvious consequence of the existence of hidden variables".
  - S.11 (p. S-21), on running the dynamics in real time or only at plasticity events: "Both scenarios are conceivable."
  - S.8: a single-timescale synapse corresponds to "an exponential memory decay function r(t) ∼ exp(−t/τ)". A population with different learning rates (the Roxin–Fusi heterogeneous model) scales like the cascade model but performs worse.
- **Mapping to Petri:**
  - u_1 decaying toward u_2 is decay toward a slow hidden variable. In a two-variable reduction, u_2 plays the role of Petri's birth weight b when consolidation is frozen.
  - A single per-edge ρ gives exponential forgetting. Power-law retention would need ≥2 coupled components per edge.
  - The "zero" in this model is equilibrium, i.e. the baseline, not zero conductance.
  - S.11 supports either per-tick or per-visit decay. Petri's trace decay already runs per world tick.

### 2. Clopath, Ziegler, Vasilaki, Büsing, Gerstner: "Tag-trigger-consolidation" (PLoS Comput Biol 2008)

- **Citation:** PLoS Comput Biol 4(12):e1000248. https://doi.org/10.1371/journal.pcbi.1000248 (PMC2596310)
- **How obtained:** Europe PMC full-text XML.
- **Mechanism:**
  - Early LTP/LTD is a binary tag (h_i or l_i) that resets stochastically at rate k_H or k_L.
  - The late phase is a bistable variable z_i, which can switch only when a tag coincides with plasticity-related proteins (PRPs).
  - Protein synthesis is triggered when the neuron-wide tag count exceeds N_p, which depends on dopamine.
- **Equations (Methods, "Synaptic Weight"):**
  - "wi = w̅(1+hi−αli+βzi) where w̅ is the value of a non-tagged synapse", with α = 0.5 and β = 2.
  - The bistable z has `f(z) = z(1−z)(z−0.5)`. A synapse "can change its consolidated value only if both a tag … and protein p>0.47 is present".
- **Timescales:**
  - Results ("Consolidation of Synaptic Changes"): weak tetanus gives "+15 percent" LTP, which "decays back to baseline over the time course of two hours".
  - Strong repeated tetanus consolidates and lasts ">10 hours" (as long as simulated).
- **Other quotes:**
  - Discussion: "For maintenance, it is not the synapse which decides individually, but it is the neuron as a whole … which 'decides' whether it is going to store the present information".
  - Introduction: "synaptic plasticity on multiple time scales cannot prevent, but at most delay the erasure of memories in the presence of ongoing synaptic activity".
- **Mapping to Petri:**
  - This is the cleanest additive fast+slow template: decay target = w̄(1+βz), the consolidated baseline.
  - With z frozen for life (no consolidation feature), the target is a constant per-edge baseline. That is exactly decay toward birth weight.
  - Consolidation in this model is gated per neuron and by a neuromodulator. A Petri analog would gate on an outcome signal, not per edge alone.

### 3. Ziegler, Zenke, Kastner, Gerstner: "Synaptic consolidation: from synapses to behavioral modeling" (J Neurosci 2015)

- **Citation:** J Neurosci 35(3):1319–1334. https://doi.org/10.1523/JNEUROSCI.3989-14.2015 (PMC6605543)
- **How obtained:**
  - The jneurosci.org page sat behind a Cloudflare challenge. I did not attempt to get past it.
  - The PMC HTML (full article text) came via curl. The equations are images and were not extracted.
- **Mechanism:**
  - Three layers per synapse: weight w, tagging variable T, scaffold z, each in a double-well potential.
  - The couplings run from scaffold to T to w and are write-protected.
  - A synapse-specific gate G_ij (set by plasticity) opens the w→T coupling. A neuron-wide PRP variable p_i (requires dopamine) opens the T→z coupling.
- **Quotes:**
  - Introduction: "Synaptic strength changes induced by Hebbian protocols deteriorate over a few hours (early LTP or e-LTP) unless they are consolidated".
  - Model description: "In the absence of external input, a tagged synapse decays back to its resting state within a few hours. The timescale of the decay is determined by the time needed for noise to push T out of the metastable potential well."
  - "Selectivity of memory implies that some synaptic traces decay whereas others persist."
- **Parameters (Table 1):** τ_w = 200 s, τ_T = 200 s, τ_z = 200 s, τ_γ = 600 s, k_down = 1/7200 s⁻¹.
- **Mapping to Petri:**
  - The decay target is the "resting state", which is set by the scaffold. The weight is pulled back toward its consolidated layer, not toward zero.
  - The dopamine-gated write is the natural analog if Petri ever adds consolidation driven by reward signals. Petri already carries reward_source signals.

### 4. Zenke, Agnes & Gerstner: "Diverse synaptic plasticity mechanisms orchestrated to form and retrieve memories in spiking neural networks" (Nat Commun 2015)

- **Citation:** Nat Commun 6:6922. https://doi.org/10.1038/ncomms7922 (PMC4411307)
- **How obtained:** Europe PMC full-text XML. The formulas are images and were not extracted.
- **Mechanism:**
  - Triplet STDP is combined with:
    - a fast heterosynaptic term, which acts on all synapses of a neuron during postsynaptic bursts (fourth power of postsynaptic activity), with direction set by w − w̃;
    - transmitter-induced potentiation;
    - a consolidation variable w̃ that follows w through a double-well potential.
- **Equation in words** (from the text; the formula was not extractable): the heterosynaptic term is about −β·(w − w̃)·post⁴. The reference weight w̃ "follows the negative gradient of a double well potential".
- **Quotes:**
  - Discussion: "The interaction of heterosynaptic plasticity with consolidation implies that each synapse in our model 'remembers' its baseline value so that induced synaptic changes can be reverted during a time window of up to 20–60 min".
  - "This is achieved by a bistable time-dependent reference weight that follows the synaptic weight w on a much longer timescale".
  - Consolidation result: "a network without synaptic consolidation can no longer recall memories from external cues."
- **Mapping to Petri:**
  - The return toward the reference weight here is **activity-gated** (bursts), not passive.
  - It supports "decay toward own baseline". It also shows that a biologically motivated pull can be conditioned on postsynaptic activity, which is an alternative to a constant per-tick ρ.

### 5. Zenke & Gerstner: "Hebbian plasticity requires compensatory processes on multiple timescales" (Phil Trans R Soc B 2017)

- **Citation:** Phil Trans R Soc B 372:20160259. https://doi.org/10.1098/rstb.2016.0259 (PMC5247595)
- **How obtained:** Europe PMC full-text XML. Equation 7.1 is an image.
- **Mechanism:** Hebbian plasticity needs rapid compensatory processes (heterosynaptic depression, transmitter-induced plasticity) alongside slow homeostasis.
- **Quotes:**
  - Section 1: "Under suitable conditions the changes induced by a protocol of LTP or LTD are further consolidated after about an hour."
  - On the generic plasticity terms: "g0: Finally, a slow drift, a spontaneous growth or decay of the synaptic strength that does not depend on the input or the state of the postsynaptic neuron is captured by the function g0(wij)."
  - Section 7: w̃ "serves as a reference weight that can be related to consolidation dynamics". "In general the reference weight is not fixed, but follows its own temporal dynamics on a slower timescale (≈20 min and more)."
  - Discussion: "Simulations predict that synaptic consolidation renders synapses inert against heterosynaptic plasticity. Intuitively, the measured synaptic weights become 'sticky' and are always attracted back to their momentary stable state".
  - On set points: "RCPs which enforce a single set point are hardly desirable from a functional point of view."
- **Mapping to Petri:**
  - The g0 term is the formal home of an activity-independent decay.
  - The "sticky" reference weight is the attractor; Petri's birth weight b plays w̃ with its slow dynamics frozen.
  - The single-set-point warning argues against pulling *all* edges toward a common value (e.g. 0) and for per-edge baselines.

### 6. Gerstner, Lehmann, Liakoni, Corneil & Brea: "Eligibility traces and plasticity on behavioral time scales" (Front Neural Circuits 2018)

- **Citation:** Front Neural Circuits 12:53. https://doi.org/10.3389/fncir.2018.00053 (PMC6079224)
- **How obtained:** Europe PMC full-text XML.
- **Equation 1 (extracted):** `d/dt e_ij = η f_j(x_j) g_i(y_i) − e_ij/τ_e`. The weight changes only via the third factor, M_3rd(t), in Eq. 2. Eq. 2's formula did not come through in the extraction; its standard form, `dw_ij/dt = e_ij · M_3rd(t)`, is given here from the framework, not read off the text.
- **Timescales:**
  - "we predict … a decay constant τe of the synaptic flag in the range of 1 s".
  - Experimental data: "pre-before-post pairing leaves an eligibility trace that decays over 5–10 s" (noradrenaline); post-before-pre "decays over 3 s" (serotonin).
  - For tagging and capture: the tag "decays back to zero over the time of 1 h".
- **Quotes:**
  - On consolidation: "in slices, the measured synaptic weights wij are increased a few minutes after the end of the induction protocol and decay back with the time course of the synaptic tag whereas in the simplest implementation of the three-factor rule framework … the visible weight is only updated at the moment when the third factor is present."
  - "However, slightly more involved models where the visible weight depends on both the synaptic tag variable and the long-term stable weight (Clopath et al., 2008; Ziegler et al., 2015) correctly account for the time course".
- **Mapping to Petri:**
  - Petri's `trace_decay` is e_ij/τ_e. It is a flag that decays to **zero**, which is correct for a flag.
  - Weight decay toward baseline is a different, orthogonal mechanism. It corresponds to "the visible weight decays back with the time course of the tag", which Petri's pure three-factor update lacks.

### 7. Abraham: "How long will long-term potentiation last?" (Phil Trans R Soc B 2003)

- **Citation:** Phil Trans R Soc Lond B 358:735–744. https://doi.org/10.1098/rstb.2002.1222 (PMC1693170)
- **How obtained:**
  - The PMC HTML had abstract and references only; the body is scanned.
  - I opened the PMC PDF in the built-in browser and extracted all 10 pages with pdf.js.
- **Mechanism:** In vivo, LTP decays exponentially in distinct families. Decay is regulated by active processes (NMDA-receptor-dependent activity, enrichment, neurogenesis).
- **Timescales and quotes:**
  - p. 736, Fig. 1: "Three families of curves were identified and calculated to have average decay time constants of 2.1 h, 3.5 days and 20.3 days. They were termed LTP1, LTP2 and LTP3 … LTP1 is protein-synthesis independent, and corresponds to early-phase LTP".
  - p. 736: "LTP in the dentate gyrus was decremental, that is, it eventually returned to baseline even if it took many weeks to do so".
  - p. 736: "under the right conditions, [dentate synapses] have the capacity for stable LTP lasting at least many months, and in one case up to a year".
  - p. 737, regional differences: "it seems likely that the persistence of LTP will vary across brain regions, perhaps according to the information processing requirements of those regions". Also: "CA1 has a lower threshold for exhibiting stable LTP".
  - p. 739, on Villarreal 2002, where LTP decay was blocked by an NMDA antagonist: "LTP can be caused to decay by an active NMDA receptor-dependent process in the days following induction."
  - On enrichment reversing LTP: 1 h per day gave partial reversal over 3 weeks; overnight exposure gave "complete depotentiation rapidly … within a week". Reversal "did not occur when exposure commenced 90 days post-induction … suggesting that there is a long-term consolidation process that takes weeks to complete".
  - "Establishment of the late-phase does not fully protect the LTP against subsequent reversal". Stable LTP (120 days) was still reversed heterosynaptically and could be reinstated.
- **Mapping to Petri:**
  - Exponential decay toward **baseline**, with time constants spanning about four orders of magnitude.
  - The rate differs by pathway/region and by induction strength.
  - Consolidation is slow and partial, and it does not immunize.
  - Directly supports a heritable per-edge ρ with a wide range that includes ρ = 1.

### 8. Turrigiano: "The self-tuning neuron: synaptic scaling of excitatory synapses" (Cell 2008)

- **Citation:** Cell 135:422–435. https://doi.org/10.1016/j.cell.2008.10.008 (PMC2834419, author manuscript)
- **How obtained:** NCBI BioC JSON full text.
- **Mechanism:** Neuron-wide, multiplicative, slow scaling of AMPA receptor content, hours to days. It is driven by deviation of firing from a set point and is transcription-dependent.
- **Quotes:**
  - "Such a scaling process has the attractive property of allowing neurons to normalize firing without changing the relative strength of synaptic inputs".
  - "Synaptic scaling is fundamentally different: it appears to be a slow, cumulative, and dynamic form of plasticity".
  - Timescale: "robustly observed after 4–24 hours of activity-blockade".
  - Key passage (section on local versus global mechanisms): "A very local mechanism (operating on individual synapses) would tend to erase the effects of LTP or LTD; such a 'forgetting' mechanism might be useful if it could be selectively turned on or off (for example, by neuromodulators), but otherwise would be counterproductive."
  - Also: "a 'quasi-local' mechanism that operates on groups of synapses distributed along a dendritic branch could have useful computational properties".
- **Mapping to Petri:**
  - Neuron-wide multiplicative scaling is **not** decay toward the birth weight. It preserves learned ratios and regulates activity, and belongs to a separate feature (per-node gain).
  - Turrigiano's own wording makes per-synapse decay a "forgetting" mechanism. Her caveat (useful if "selectively turned on or off") argues for a modulatable or evolvable rate rather than a fixed global one.
  - The "quasi-local" branch-level grain is loosely like Petri's edge groups. But plastic edges are never group members in the current code (`learned.rs`), so groups are not the place for this.

### 9. Tononi & Cirelli: "Sleep and the price of plasticity" (Neuron 2014)

- **Citation:** Neuron 81:12–34. https://doi.org/10.1016/j.neuron.2013.12.025 (PMC3921176, author manuscript)
- **How obtained:** NCBI BioC JSON full text.
- **Mechanism:** The synaptic homeostasis hypothesis (SHY). Waking produces net potentiation; sleep renormalizes by down-selection (proportional downscaling, or protecting strongly reactivated synapses while others depress).
- **Quotes:**
  - "stronger synapses consume more energy, require extra supplies, and lead to cellular stress".
  - "Computer implementations of down-selection include: a downscaling rule where all synapses decrease in strength proportionally, but those that end up below a minimal threshold become virtually ineffective; a modified STDP rule by which stronger synapses are depressed less than weaker ones; and a 'protection from depression' rule."
  - Fly data: "the number of dendritic spines increases after 12 hours of wake … and returns to pre-enrichment levels only if the flies are allowed to sleep".
  - "spine turnover is limited and is not impacted by sleep and wake in adult mice".
- **Mapping to Petri:**
  - Return "to pre-enrichment levels" is decay toward the pre-learning baseline, but it is state-gated (sleep).
  - The energy-cost argument suggests a Petri analog in which a learned deviation carries a metabolic cost. This is an alternative grounding I have not checked against Petri's energy model.
  - Proportional downscaling of all synapses is the Turrigiano-type mechanism, not per-edge decay.

### 10. Davis & Zhong: "The biology of forgetting — a perspective" (Neuron 2017)

- **Citation:** Neuron 95:490–503. https://doi.org/10.1016/j.neuron.2017.05.039 (PMC5657245, author manuscript)
- **How obtained:** NCBI BioC JSON full text.
- **Mechanism:**
  - "Intrinsic forgetting" is a chronic, regulated, active erosion of memory traces.
  - In *Drosophila*, dopamine "forgetting cells" act through the DAMB receptor, then Scribble and Rac1/cofilin, on mushroom-body neurons.
- **Quotes:**
  - "Intrinsic forgetting can be viewed as a homeostatic mechanism to bring the brain back to its basal state."
  - "intrinsic forgetting may operate chronically at a low level to slowly remove each newly acquired memory, although its strength may be regulated by internal or external factors."
  - "Consolidation would play the role of the judge … We can therefore view consolidation as being in constant competition with a 'homeostatic' signal from intrinsic forgetting". "This does not necessarily mean that consolidated memories are immune from intrinsic forgetting."
  - On the dopamine forgetting cells: "these neurons are chronically active, but modulated in intensity … providing an 'ongoing' signal for forgetting".
  - Mice: "Increased activation of Rac1 in the mouse hippocampus … accelerates decay of object-recognition memory by about 3-fold". The same perturbations "were without effect on contextual fear conditioned memory or cued (tone:shock) trace memory."
  - Flies: "anesthesia-sensitive memory (ASM) that persists for a few hours and anesthesia-resistant memory (ARM) that persists for more than 24 hours".
- **Mapping to Petri:**
  - "Bring … back to its basal state" plus "chronically at a low level" is the most direct literal grounding for a constant per-tick decay toward the birth value.
  - The rate is a regulated, heritable molecular level (Rac1, DAMB). Different memory types have different decay rates and sensitivities, which supports a heritable rate that differs by edge class.

### 11. Richards & Frankland: "The persistence and transience of memory" (Neuron 2017)

- **Citation:** Neuron 94:1071–1084. https://doi.org/10.1016/j.neuron.2017.04.037
- **How obtained:** cell.com full text (Cell Press open archive) read in the built-in browser, about 85k characters. A third-party PDF copy surfaced in search; I did not use it.
- **Thesis:** Transience is adaptive. It improves flexibility in changing environments and acts as regularization in noisy ones.
- **Quotes:**
  - On *C. elegans*: "associative learning induces synapse growth at a specific neuron (AVA). However, within hours of learning, this modified synapse shrinks, and reversion to a naive state is accompanied by loss of the associative memory" (citing Hadziselimovic et al. 2014).
  - "Weight decay arguably corresponds to neurobiological mechanisms that weaken previously potentiated synaptic connections, including depotentiation … and synaptic downscaling". In contrast, "weight elimination corresponds to … Rac-mediated spine shrinkage".
  - "weaker stimulation produced a decaying LTP that returned to baseline within 2 hr of tetanization".
  - "neurogenesis provides a continuous decay signal".
  - Citing Brea 2014: "tuning the rate of forgetting to the temporal dynamics of the environment maximized reward rates. When action-outcome contingencies changed frequently, faster forgetting was optimal."
- **Mapping to Petri:**
  - *C. elegans* (a small, largely genetically specified nervous system) reverting to the **naive** state is the closest organism-level analog to "decay toward genomic weight".
  - The regularization framing justifies decay toward a prior, which is Petri's birth weight.

### 12. Brea, Urbanczik & Senn: "A normative theory of forgetting: lessons from the fruit fly" (PLoS Comput Biol 2014)

- **Citation:** PLoS Comput Biol 10(6):e1003640. https://doi.org/10.1371/journal.pcbi.1003640 (PMC4046926)
- **How obtained:** Europe PMC full-text XML. Equation 5 is an image.
- **Mechanism:**
  - Optimal behaviour in a stochastically changing environment requires forgetting at a rate matched to the environment's rate of change.
  - A near-optimal implementation uses two uncoupled synaptic low-pass filters (approach and avoid). They decay toward a baseline at different rates.
- **Equation 5 in words:** a parameter "controls the speed of forgetting", another "sets a baseline value" and another "sets a maximum value".
- **Quotes:**
  - "optimal behavior in a stochastically changing environment requires a forgetting rate that is adapted to the time constant of the changes."
  - "In the absence of odor and behavioral feedback the synaptic strengths decay with different time scales towards a stationary level: decay is faster for synapses targeting the 'avoid' neurons than for the 'approach' neurons."
  - "As the parameters describing the changing environment are assumed to be constant across generations, the neuronal architecture and the forgetting rates can be considered to be genetically encoded."
- **Mapping to Petri:**
  - This is the strongest normative argument for a **heritable, per-synapse-class** decay rate toward a **baseline**, not toward zero.
  - Petri's evolution can tune the rate to world volatility (e.g. boom/bust worlds).

### 13. Leimer, Herzog & Senn: "Synaptic weight decay with selective consolidation enables fast learning without catastrophic forgetting" (bioRxiv 613265, 2019)

- **Citation:** https://doi.org/10.1101/613265
- **How obtained:** biorxiv.org full-text HTML read in the built-in browser (curl was blocked). I did not check whether it was later published in a journal.
- **Mechanism:**
  - Two-component reward-based rule: `w = w_f + w_s`.
  - w_f follows the reward gradient plus a penalty on w_f². That penalty gives a passive decay −λ_w·w_f.
  - The decaying amount is transferred into the non-decaying w_s only when |w_f| > θ_w and low-pass-filtered postsynaptic voltage > θ_V.
- **Quotes:**
  - "This term results in a passive decay of the fast component such that, in the absence of new plasticity events, it converges to baseline value".
  - "The decay term … causes a passive decay in the order of 1/(2λw) = 42 biological minutes". The Methods give λ_w = 0.012 and updates "every 30 s". Those do not obviously reconcile with 42 min (1/(2·0.012) steps × 30 s ≈ 21 min). Both figures are the paper's own; I did not resolve the discrepancy.
  - "the decay of the fast component is transferred to the slow component provided the two conditions are simultaneously satisfied".
  - "Weak synaptic modifications associated with low postsynaptic depolarizations must be extinguished on the timescale of the context duration".
  - Prediction: "Context switches should not be faster than the decay time of early LTP/LTD (some ten minutes)".
  - "The passive decay of the early LTP/LTD is typically seen as a weakness of the synapses that needs to be counteracted. We suggest that this passive forgetting is the expression of minimizing the interference between subsequent contexts in the presence of fast learning."
  - Its one-component control (λ_w = 0, the same as Petri today) suffered interference: performance in context 1 "decayed roughly to a third".
- **Mapping to Petri:**
  - The closest computational precedent: a reward-modulated learner plus passive decay of the fast component toward baseline, plus optional gated consolidation.
  - Its consolidation is elegant for Petri: no new rate parameter, it just redirects the decay flux.
  - Decay time should match the timescale over which contingencies change.

### 14. Loewenstein, Kuras & Rumpel: "Multiplicative dynamics underlie the emergence of the log-normal distribution of spine sizes in the neocortex in vivo" (J Neurosci 2011)

- **Citation:** J Neurosci 31(26):9481–9488. https://doi.org/10.1523/JNEUROSCI.6130-10.2011 (PMC6623170)
- **How obtained:** PMC HTML full text via curl.
- **Mechanism:**
  - Without explicit learning, log spine size behaves like the sum of two Ornstein–Uhlenbeck processes.
  - Each OU process is a mean-reverting random walk: past contributions decay exponentially.
  - Changes are multiplicative.
- **Fitted timescales:** "τ1 = 212 d and τ2 = 2.87 d", with σ1² = 0.0683 and σ2² = 0.0292.
- **Quotes:**
  - "in contrast to the random walk process, the Ornstein–Uhlenbeck process describes continuous dynamics in which the contributions of past steps to the current state decay exponentially".
  - "large spines tend to become smaller, whereas small ones tend to increase … on average, spines tend to contract to a medium size".
- **Mapping to Petri:**
  - The deterministic part of an OU process is exactly `w ← b + ρ(w − b)`. Two timescales (days and months) are measured directly in vivo.
  - **Counter-evidence to a lifelong fixed attractor:** here the attractor is the *population* mean of log size (μ = 1.74), not each spine's own birth size, and spines wander. The per-synapse "own baseline" grounding therefore comes from the LTP/tag literature (sources 2–4, 7, 13), not from spine volatility.

### 15. Mongillo, Rumpel & Loewenstein: "Inhibitory connectivity defines the realm of excitatory plasticity" (Nat Neurosci 2018)

- **Citation:** Nat Neurosci 21:1463–1470. https://doi.org/10.1038/s41593-018-0226-x
- **How obtained:** PDF from the Loewenstein lab site.
- **Quotes:**
  - p. 1: "the rate of learning-driven spine formation and elimination is not much higher than the 'spontaneous' rate observed during basal conditions".
  - Measurements: "over the course of 20 d, >70% of the spines changed their size by at least 50%. Nevertheless, density and size distributions of the spines remained stationary".
  - "Differences between rates have been attributed to cortical region, age of the animal, and differences in imaging techniques".
  - Model result: network activity is robust to excitatory volatility, and inhibitory connectivity carries long-term memory.
- **Mapping to Petri:**
  - Supports synapse-class-dependent stability (excitatory volatile, inhibitory more decisive).
  - Background drift exists even without learning. Petri has no weight noise; I am not recommending adding it.

---

## Synthesis

### (1) Which natural mechanism grounds which design?

| Design | Natural analog | Grounding strength |
|---|---|---|
| **Per-edge decay toward the edge's own birth value** | Early-LTP/LTD and tag decay back to baseline (Frey & Morris 1997 [abstract]; Clopath 2008; Ziegler 2015; Abraham 2003 LTP1–3). Intrinsic forgetting "back to its basal state" (Davis & Zhong 2017). *C. elegans* AVA synapse "reversion to a naive state" (Richards & Frankland 2017, citing Hadziselimovic 2014). Leimer 2019 fast-component decay. | **Strong.** Multiple full-text sources, both models and data. |
| Decay toward zero (absolute) | None for weakening a *learned change*. Closest is synapse elimination or spine shrinkage ("weight elimination"; Rac1-driven spine shrinkage). Decay to a common population value is supported only statistically (spine size mean reversion; Loewenstein 2011). | **Weak** as a forgetting analog. It would also erase innate, genome-specified wiring, which the biology does not do: the AVA synapse returns to naive, not to absent. |
| Neuron-wide multiplicative scaling | Turrigiano synaptic scaling; SHY proportional downscaling in sleep | Strong as its own mechanism. It **preserves** learned ratios and is a separate feature (activity homeostasis), not forgetting. |
| Consolidation that stops decay | Tag-and-capture (Frey & Morris; Redondo & Morris [not read]); Clopath z; Ziegler scaffold; Zenke w̃; Leimer w_s; Abraham's weeks-long consolidation | Strong. But consolidation slows rather than ends forgetting (Abraham: 120-day LTP reversed; Davis & Zhong: not immune). |

**Coordinate point.** Every model that decays "to zero" does so for a deviation variable (Benna–Fusi u_k, Leimer w_f, Clopath h/l tags). In absolute weight that means decay to the baseline. So "toward birth weight" and the biology are the same claim, provided `b` is the pre-learning value of *that* synapse.

**Lamarckian slots.** For a seeded Lamarckian slot, `learned` starts at the inherited seed, not at `def.genomic_weight` (`learned.rs`, `from_seeds`).
- The analog (pre-learning state, naive state, reference weight) points to the **slot's birth value**: the seed when seeded, otherwise the genomic weight. Decaying a seeded slot toward the genomic weight would erase the inherited learning at the rate ρ.
- This is a design choice; biology has no Lamarckian weight inheritance to settle it. Read literally, "decay toward genomic weight" conflicts with the Lamarckian flag; "decay toward birth weight" does not.

**Minimal form consistent with the evidence:** `w_{t+1} = b + ρ·(w_t − b)`, with ρ heritable per edge in [0, 1].
- ρ = 1 means no forgetting, which matches Abraham's "stable LTP lasting months".
- This is the deterministic half of an OU process (Loewenstein 2011). It is also the λ_w term of Leimer 2019 and the tag reset of Clopath 2008.
- The learning update then acts on top of it, as in Leimer's Eq. 3.

### (2) Do decay rates differ by synapse type or region?

Yes. This is the evidence for making the rate heritable and per edge (or per edge class):

- **Abraham 2003:** three decay families, τ ≈ 2.1 h / 3.5 d / 20.3 d. Persistence depends on induction strength and repetition, and differs between the dentate gyrus and CA1 ("CA1 has a lower threshold for exhibiting stable LTP").
- **Brea 2014:** approach and avoid synapses decay at different rates. The rates are argued to be genetically encoded and matched to environmental volatility.
- **Davis & Zhong 2017:** Rac1 activation speeds up object-recognition forgetting about 3-fold but leaves contextual fear memory unaffected. Fly ASM (hours) and ARM (>24 h) are distinct components.
- **Mongillo 2018:** turnover rates vary with region and age. Excitatory and inhibitory synapses differ in how much they matter for long-term memory.

**Caveat on grain.** In flies and mice the rate is set by per-neuron or per-compartment molecular state (Rac1 in mushroom-body neurons; one dopamine neuron per mushroom-body zone), not per individual synapse. Clopath 2008 likewise argues "it is the neuron as a whole … which 'decides'".
- A per-edge heritable ρ is finer than the evidence but matches the Brea model, which uses per-synapse-class rates.
- A per-node ρ would be closer to the molecular grain. The trade-off is between evolvability and fidelity, and this research does not settle it.

### (3) How does consolidation interact with forgetting?

- **Two-timescale additive weight** (Clopath; Leimer; Benna–Fusi reduced to two variables): `w = w_slow + w_fast`. w_fast decays toward 0, so w decays toward w_slow; w_slow changes only through a gated write.
- **The write gate in biology** is a neuromodulator (dopamine for PRP synthesis), neuron-level tag counts, or strong postsynaptic depolarization. Leimer's version: decay flux moves into w_slow only when |w_fast| > θ_w and the voltage is high.
- **In Petri terms:** the one-component design (decay toward a fixed b) equals the two-component design with the gate always closed and w_slow = b.
  - A later consolidation feature could gate a transfer of the decay flux into b on an outcome signal.
  - That would need one extra state per edge but no new rate parameter (Leimer).
- **Consolidation does not immunize** (Abraham; Davis & Zhong). Even a two-component design keeps forgetting active.
- **Single-ρ caveat:** one exponential gives exponential forgetting. Power-law retention, which is efficient for many memories, needs a chain of components (Benna–Fusi).

### (4) Timescales in Petri units (ratios only; no tick values invented)

Biological decay constants as fractions of lifespan:

- Early LTP ≈ 2 h against a rodent lifespan of about 2 years (lifespan is general knowledge, not from these sources): about 10⁻⁴ of life.
- LTP2 3.5 d ≈ 0.5%; LTP3 20 d ≈ 3%.
- Spine OU processes 2.9 d and 212 d ≈ 0.4% and about 30%.
- Fly ASM, a few hours, against a lifespan of about 2 months (general knowledge): about 0.2%.

With Petri lifespans of thousands of ticks, early-LTP-proportional decay would be shorter than one tick. So the meaningful analogs are the LTP2/LTP3, ASM and intrinsic-forgetting range: roughly 0.1–3% of lifespan, or tens of ticks, up to "no decay".

Normatively (Brea; Leimer), the decay time should match how fast contingencies change. That is an argument to let selection set ρ over a wide range that includes 1.

### Open questions

1. **Grain:** per-edge or per-node ρ? (See the caveat in section 2.)
2. **Passive or gated:**
   - Zenke's return to w̃ is gated by bursts; Tononi's return to pre-enrichment needs sleep; Davis's forgetting signal is "modulated in intensity".
   - Option: a constant ρ now, with modulation by state or outcome later.
   - Turrigiano says a per-synapse forgetting mechanism is useful only if "it could be selectively turned on or off".
3. **Per tick or per visit:** Benna–Fusi say both are conceivable. Biology (OU, LTP decay) runs in real time. Petri's `trace_decay` already runs per world tick, so per-tick is the consistent choice.
4. **Interaction with Oja and AntiHebb:** Oja already decays w in proportion to post². Adding ρ gives two pulls with different targets (Oja: toward a normalized w; ρ: toward b). It is unknown whether they conflict.
5. **Energy cost as a grounding alternative:** Tononi & Cirelli say "stronger synapses consume more energy". A cost on |w − b| or on |w| would let selection discover decay without a rate gene. Not evaluated here.
6. **Mean reversion toward a population value** (Loewenstein; Mongillo) is a real phenomenon. It is better modelled as noise plus reversion to a distribution than as a forgetting gene. Not recommended now.

---

## Claims I could not verify in full text

- **Frey & Morris 1997, Nature 385:533** (abstract only, Europe PMC): early LTP lasts "less than three hours"; "The synaptic tag decays in less than three hours."
- **Redondo & Morris 2011, Nat Rev Neurosci** (tagging-and-capture review): not obtained; paywalled, not attempted.
- **Fusi, Drew & Abbott 2005, Neuron 45:599** (cascade model): abstract only. Covered in substance by Benna–Fusi.
- **Mongillo, Rumpel & Loewenstein 2017, Curr Opin Neurobiol 46:7** (intrinsic volatility review): abstract only. ScienceDirect refused the browser. Substance covered by Mongillo 2018 and Loewenstein 2011.
- **Ryan & Frankland 2022, Nat Rev Neurosci 23:173** (forgetting as engram plasticity): abstract only. It says "forgetting rates are modulated by environmental conditions" and that forgetting switches engram cells to an "inaccessible state".
- **Hadziselimovic et al. 2014, Cell** (*C. elegans* msi-1/Arp2/3 forgetting): abstract only; the "naive state" wording is Richards & Frankland's. The abstract says msi-1 mutants show "persistence of learning-induced GLR-1 synaptic size increase".
- **Villarreal et al. 2002, Nat Neurosci** (NMDA antagonist blocks LTP decay): known only through Abraham 2003 and Richards & Frankland 2017.
- **Equations that were images and not extracted:** Gerstner 2018 Eq. 2 (standard form given); Zenke 2017 Eq. 7.1; Zenke 2015 Eqs. 11–17; Ziegler 2015 Eq. 1; Brea 2014 Eq. 5; Leimer Eqs. 3–4. I described them from surrounding text. My statement that the heterosynaptic term is about −β(w − w̃)·post⁴ is reconstructed from the text ("fourth power", "relative difference of … w and the reference weight"), not read off the formula.
- **Clopath 2008 numeric k_H:** in the parameter table, which was not extracted. The ~2 h decay is stated in the text.
- **Lifespans** (rodent ~2 y, fly ~2 months) used in the timescale ratios are general knowledge, not taken from these sources.
