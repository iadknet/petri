# Forgetting, weight decay and transient vs persistent learned weights in evolved plastic agents

Research for Petri (per-edge passive decay toward the genomic weight). Read-only; nothing in the repo was edited.
Date: 2026-10-19. PDFs and extracted text were kept in a scratch folder that was not retained. Page numbers come from the PDF page markers in those extracts.
In the quotes below, typographic ligatures were normalised (ﬁ→fi, ﬂ→fl).

## Bottom line

1. **No source I read stores one weight and pulls it toward a target.** Every "decay toward baseline" in the literature splits the weight in two: a persistent part (genomic or consolidated) plus a transient part that decays to zero. The effective weight is their sum, so it decays toward the persistent part. Examples are Miconi 2018 (`w + α·Hebb`, where Hebb decays), the Hinton–Plaut fast and slow weights in Bullinaria 2009, Soltoggio 2015's HTP (`W = w_st + w_lt`, where w_st decays) and Tyulmankov 2022 (`W1 + A(t)`, where A decays by a factor λ). Petri already stores `genomic` for each slot (`learned.rs`). So "pull the weight toward its anchor at rate d" gives the same dynamics as the two-part form, with less bookkeeping.
2. **One source evolves a decay rate directly: Bullinaria 2009.** Evolution drove the fast-weight decay rate to a nonzero value that was the same across runs (δ = 0.0011 ± 0.0002 over 5 independent runs). In the same runs, evolution drove the ordinary weight decay toward zero (λ, which pulls all weights toward 0) to negligible levels. **Supported reading:** decay of the learned part toward the stable part is selected for, and decay of the whole weight toward zero is selected against. Tyulmankov 2022 meta-learns a per-network decay factor (by gradient descent, not evolution) and finds it trades off against how many plasticity events happen.
3. **Main argument for decay in Petri's case: Soltoggio 2015's HTP random-walk demonstration.** A single-component reward-modulated weight that receives zero-mean coincidental updates loses its starting value ("random walk, or diffusion"). A decaying short-term component keeps it. Petri's reward-modulated edges are single-component weights, so they fit this description.
4. **Main argument against, and a null result.** Backpropamine (2019) dropped the 2018 decay term on the Hebbian trace and used a hard clip, because the clip "turned out to produce equal or superior performance". Petri already clamps. Yaman et al. (2021) found that evolution prefers rules that **stop** changing weights once the task is learned, because ongoing change causes "forgetting". Ellefsen/Velez (abstracts and partial reading) treat forgetting as a cost to avoid, not a feature.
5. **Most of the field sidesteps the question.** Most EPANN and meta-learning setups reset all plastic state at birth or at each episode. Examples: Floreano & Urzelai start from random weights, Miconi sets Hebb = 0 per lifetime, Najarro & Risi start from random weights, and Miconi 2023 sets P = 0 per lifetime. Miconi 2023 names the long-lifetime continual case as the "missing loop". Petri's long lives in a changing world are therefore under-studied ground. Petri cannot simply copy a design that has been validated in ALife.
6. **Anchor choice is a real design fork for Lamarckian edges** (see the synthesis section). Decaying toward `genomic` erases what inheritance delivered. Decaying toward the birth weight (the inherited seed) keeps it. Evidence on whether Lamarckian inheritance helps in changing environments is split.

---

## Sources (read in full unless stated otherwise)

### 1. Soltoggio, Stanley & Risi 2018, "Born to learn: the inspiration, progress, and future of evolved plastic artificial neural networks", *Neural Networks* 108:48–67
- URL: https://arxiv.org/abs/1703.10371 (v3). Obtained by downloading the arXiv PDF and extracting the full text. I searched the whole text and read the relevant sections closely.
- **Mechanism:** a review. It lists stabilising terms (Oja, BCM) and "dual weights for fast and slow decay" as building blocks.
- **On forgetting and decay:**
  - p.4: Hebbian variants include "more advanced update dynamics such as dual weights for fast and slow decay (Levy and Bairaktaris, 1995; Hinton and Plaut, 1987; Bullinaria, 2009a; Soltoggio, 2015)".
  - p.5: "Plasticity in neural networks, when continuously active, was also observed to cause catastrophic forgetting".
  - p.14: "hypothesis testing plasticity (HTP) (Soltoggio, 2015) implements fast and slow decay to consolidate weights … Plasticity rules that implement weight consolidation, given their promise to prevent catastrophic forgetting, are likely to become standard components in EPANNs."
  - p.10 (Baldwin effect): in static environments "Eventually, a system in those conditions can perform well without learning because it can be born knowing to perform well." Non-stationary, reward-based environments "are more typically used to study the evolution of learning in EPANNs."
- **Mapping to Petri:** The review supports a fast/slow (transient + consolidated) split as the EPANN-standard way to get forgetting without wiping innate wiring. It names consolidation as the expected next component. It does not discuss decay toward the genomic weight as such.

### 2. Miconi, Clune & Stanley 2018, "Differentiable plasticity: training plastic neural networks with backpropagation", *ICML 2018*
- URL: https://arxiv.org/abs/1804.02464. Full text via the arXiv PDF.
- **Mechanism (p.2):**
  - Eq. 1: `x_j(t) = σ{Σ_i [w_ij x_i(t−1) + α_ij Hebb_ij(t) x_i(t−1)]}`
  - Eq. 2: `Hebb_ij(t+1) = η x_i(t−1) x_j(t) + (1−η) Hebb_ij(t)`
  - w and α are learned between lifetimes. Hebb is "initialized to zero at the beginning of each lifetime/episode". η is a single learned scalar.
- **Finding (p.2):** "In Equation 2, η appears as a weight decay term, to prevent runaway positive feedback on Hebbian traces. However, because of this weight decay, Hebbian traces (and thus memories) decay to zero in the absence of input." The authors offer Oja's rule (Eq. 3, "note the absence of a decay term") as the alternative that can "maintain stable weight values indefinitely in the absence of stimulation". In this paper the decay rate and the learning rate are the same parameter.
- **Mapping to Petri:** This is decay toward the genomic weight in two-component form: the effective weight decays to w. It is a direct precedent. Two caveats:
  - The rate is coupled to η and shared network-wide, not per-synapse and not independently evolved.
  - The paper itself presents decay as a stability device whose side effect, the loss of memory, is a cost. Petri edges using the Oja rule already have the non-decaying alternative.

### 3. Miconi, Rawal, Clune & Stanley 2019, "Backpropamine: training self-modifying neural networks with differentiable neuromodulated plasticity", *ICLR 2019*
- URL: https://arxiv.org/abs/2002.10585. Full text via the arXiv PDF.
- **Mechanism:**
  - p.3, Eq. 2: `Hebb(t+1) = Clip(Hebb(t) + η x_i x_j)`.
  - p.4, retroactive modulation:
    - Eq. 4: `Hebb(t+1) = Clip(Hebb(t) + M(t) E(t))`
    - Eq. 5: `E(t+1) = (1−η)E(t) + η x_i(t−1) x_j(t)`, with a "trainable decay factor η".
- **Findings:**
  - p.3: "In previous work (Miconi et al., 2018), this function was either a simple decay term, or a normalization implementing Oja's rule … In the present paper it is simply a hard clip … this simple operation turned out to produce equal or superior performance on the tasks in this paper."
  - p.4: "Hebbian plasticity does not directly modify the synaptic weights, but creates a fast-decaying 'potential' weight change, which is only incorporated into the actual weights if the synapse receives dopamine within a short time window."
- **Mapping to Petri:** This is exactly Petri's current design: the trace decays, the weight is clipped, and the weight has no passive decay. It is the main **null result**. On the episodic tasks of that paper, removing weight decay in favour of a clamp did not hurt. Episodes are short and plastic state resets each episode, so the result says little about long lifetimes. Keep "trace decay" (which Petri already has as `trace_decay`) separate from "weight decay" (the gap).

### 4. Soltoggio 2015, "Short-term plasticity as cause–effect hypothesis testing in distal reward learning", *Biological Cybernetics* 109(1):75–94
- URL: https://arxiv.org/abs/1402.0710 (author's preprint). Full text via the arXiv PDF.
- **Mechanism (HTP), all hand-designed rather than evolved:**
  - p.5, Eq. 7: `ẇ_st = −w_st/τ_st + m(t)·E(t)`, with τ_st = 8 h.
  - p.5, Eq. 9: `W = w_st + w_lt`.
  - p.6, Eq. 10: `ẇ_lt = ρ·H(w_st − Ψ)`. This is consolidation when the short-term weight crosses a threshold Ψ = 0.95, with ρ = 1/1800 s.
  - The eligibility trace E has its own decay of "the order of seconds".
- **Findings:**
  - p.3: "The current model uses eligibility traces with a decay in the order of seconds … the decay of transient weights acts instead in the order of hours, thereby representing the forgetting of coincidental event-reward sequences that are not confirmed by consistent occurrences."
  - p.5: if reward-driven changes are "applied directly to the weights, they will eventually wear out existing topologies."
  - p.5, on choosing τ: "A brief time constant ensures that weights decay quickly if rewards are not delivered … but, if rewards are sparse in time, hypotheses are forgotten too quickly. With sporadic rewards, a longer decay may help preserve hypotheses longer in time."
  - p.8, the random-walk demonstration: "In the single-weight-component case (Fig. 3b), despite the updates have an expected value of zero, the weight loses its original value of 0.5 … there is no mechanism to 'remember' its initial value. The weight undergoes a random walk, or diffusion, that leads to information loss." The two-component rule "preserves its long-term component, while the short-term component … tends to return to low values".
  - p.6: long-term weights can only grow in the base model. Reversal (undoing consolidation) is added in Appendix 1. I did not read the appendix in detail.
- **Mapping to Petri:** This is the strongest mechanistic support for the proposal. The genomic weight plays the role of `w_lt`, and the learned deviation plays the role of `w_st` with decay τ. Petri's reward-modulated edges are single-component, so by this analysis coincidental reward pairings will make them drift randomly away from the genomic weight. The paper also shows that the right τ depends on how sparse rewards are, which argues for evolving τ rather than fixing it. HTP adds a third option beyond "decay to genomic": consolidation, where the anchor itself moves during life. Note that τ was hand-set, not evolved.

### 5. Bullinaria 2009, "Evolved dual weight neural architectures to facilitate incremental learning", *Proc. IJCCI 2009*, pp. 427–434
- URL: https://www.scitepress.org/Papers/2009/23153/23153.pdf. Full text via the SciTePress PDF.
- **Mechanism (p.2):**
  - Hinton & Plaut dual weights: "an additional set of 'fast weights' that have larger learning rates and a large weight decay rate that prevents them from having long term memory."
  - p.5, evolved parameters include "A weight decay regularization parameter λ" (ordinary decay of all weights) and "For the extra set of weights in the dual weight networks, there is also another weight decay rate δ and a scale factor σ".
  - Initial weights are random for each individual. Weights are learned by backprop on supervised classification batches, not by an agent in a world.
- **Findings:**
  - p.5: "the sigmoid prime offset o_SPO and standard weight decay parameters λ fall to values so low that they have no significant effect … the fast weight decay rate δ settles at a higher level".
  - p.7: "The improvements were also sufficient to drive the fast weight parameters to extremely consistent values across the five independent runs (scale factor σ = 17 ± 2, decay rate δ = 0.0011 ± 0.0002)".
  - Dual weights helped most when class composition changed between batches (t-test p = 0.00007).
- **Mapping to Petri:** This is the only primary source I found where **evolution sets a decay rate**, and it converged to a consistent nonzero value. It also shows the asymmetry Petri should expect: decay of the learned/fast part toward the stable part was kept, and global decay toward zero was evolved away. Caveats:
  - The rate is global, not per-synapse.
  - The task is supervised incremental learning, not ALife.
  - The "slow" weights here also learn; they are not innate.

### 6. Tyulmankov, Yang & Abbott 2022, "Meta-learning synaptic plasticity and memory addressing for continual familiarity detection", *Neuron* 110:544–557
- URL: https://par.nsf.gov/servlets/purl/10338011 (NSF public-access copy of the article). Full text via the PDF.
- **Mechanism (p.3–4):** `h(t) = σ((W1 + A(t)) x(t) + b1)` and `A(t+1) = λA(t) + η h(t) x(t)^T`. "we use backpropagation through time (BPTT) to meta-learn the parameters W1, b1, W2, b2, λ, η".
- **Findings (p.6):**
  - "the memory of a stimulus is degraded in two ways: plasticity events obscure existing memories, and plastic weights decay over time."
  - With an anti-Hebbian rule, "the smaller number of plasticity events allows a larger λ (slower decay rate) to be used while still controlling the amplitude of plastic weights".
  - The network "has a moving window in time within which it can successfully detect a familiar stimulus and forgets old stimuli gracefully without suffering from catastrophic interference." It runs "without any need to reset the synaptic weights."
- **Mapping to Petri:**
  - Same two-component form: decay of A means decay toward the fixed W1, which acts as an innate weight.
  - The decay factor is optimised, not hand-set. It is meta-learned, not evolved, and is one scalar per network.
  - Decay is shown to be the mechanism that allows continual operation without resets, which is Petri's situation.
  - It also shows decay rate and rule interact. A rule that produces fewer updates (anti-Hebbian at familiar inputs) can afford slower decay. So in Petri, decay rate and rule should co-evolve rather than one being fixed.

### 7. Soltoggio, Bullinaria, Mattiussi, Dürr & Floreano 2008, "Evolutionary advantages of neuromodulated plasticity in dynamic, reward-based scenarios", *Artificial Life XI*, pp. 569–576
- URL: https://repository.lboro.ac.uk/articles/conference_contribution/Evolutionary_advantages_of_neuromodulated_plasticity_in_dynamic_reward-based_scenarios/9403988 (file at ndownloader.figshare.com/files/17020676). Full text via the figshare PDF.
- **Mechanism (p.3):**
  - Eq. 3: `Δw_ji = tanh(m_i/2)·δ_ji`
  - Eq. 4: `δ_ji = η·[A o_j o_i + B o_j + C o_i + D]`
  - Initial weights, A–D and η are evolved. There is no passive decay.
- **Findings:**
  - p.4: "the position of the reward changes across trials. When this happens, the agent has to forget the position of the reward that was learnt previously and explore the maze again."
  - Forgetting is done by modulated plasticity. In the example evolved network (p.6) the rule is "A = 0, B = 0, C = −0.38, D = 0, η = −94.6", a modulation-gated postsynaptic term.
  - p.3: "A modulation of zero will result in no weight update, maintaining the weights at the current value".
  - Natural analog, p.2, citing Bailey et al. 2000: "the synaptic growth that occurs in the presence of modulatory chemicals is long lasting, i.e. has a substantially longer decay time than the same growth in absence of modulation."
- **Mapping to Petri:**
  - In the classic evolved-neuromodulation ALife benchmark (reversal in a T-maze), evolution solved within-life forgetting by **active, signal-gated relearning**, not by passive decay.
  - The cited biology gives a named natural analog for a design where *unmodulated* (pure Hebbian) changes decay faster than *reward-modulated* ones. In Petri terms, decay could be stronger on edges without `reward_source`, or the reward event could slow decay or consolidate the change.

### 8. Floreano & Urzelai 2000, "Evolutionary robots with on-line self-organization and behavioral fitness", *Neural Networks* 13(4–5):431–443
- URL: https://infoscience.epfl.ch/record/63912 (EPFL bitstream PDF, 31-page preprint). Full text.
- **Mechanism (p.7):**
  - `w_t = w_{t−1} + ηΔw`, with four genetically chosen rules. Example: plain Hebb `Δw = (1−w)xy`, postsynaptic rule `Δw = w(−1+x)y + (1−w)xy`.
  - "Each synaptic weight w_ij is randomly initialized at the beginning of the individual's life". Weights are bounded in [0,1] by the self-limiting `(1−w)` and `w` factors.
- **Findings:**
  - p.4: "A limitation of this approach is the 'Baldwin effect', whereby the evolutionary costs associated with learning give a selective advantage to the genetic assimilation of learned properties and consequently reduce the plasticity of the system over time".
  - p.4: "Since the synaptic weights are not encoded on the genetic string, there cannot be genetic assimilation of abilities developed during life."
  - p.6 footnote: "we use Darwinian evolution instead of Lamarckian evolution".
  - p.13: adaptive individuals "rapidly modify their weights in ways that reflect the subtask at hand".
- **Mapping to Petri:**
  - The forgetting here is activity-dependent depression (the `w(−1+x)y` terms), not passive decay. Inheritance is purely Darwinian with no innate weights.
  - The paper is relevant because it names the Baldwin trade-off. If innate weights can carry the solution, evolution tends to reduce plasticity.
  - Decay toward the genomic weight would strengthen the Baldwinian pull. Anything learned but not encoded keeps getting erased, which raises the selective value of encoding it genomically. That may be good for evolvability (assimilation), but it is a predictable effect to measure.

### 9. Najarro & Risi 2020, "Meta-learning through Hebbian plasticity in random networks", *NeurIPS 2020*
- URL: https://arxiv.org/abs/2007.02686. Full text via the arXiv PDF.
- **Mechanism (p.4):** `Δw_ij = η_w·(A_w o_i o_j + B_w o_i + C_w o_j + D_w)`. All coefficients are per-synapse and evolved by ES. Weights are random U[−0.1, 0.1] at the start of each episode. There is no decay term.
- **Findings (p.7, p.8):**
  - Weights "only need between 30 and 80 timesteps … to converge".
  - After zeroing a third of the weights, "the network is able to reconverge towards an optimal solution in a few timesteps".
  - p.9: "the weights in the Hebbian-based networks self-organize and converge to an attractor in weight space during their lifetime."
- **Mapping to Petri:**
  - This is an alternative to passive decay: evolved activity-driven dynamics with an attractor that restores useful weights after perturbation. Here the "anchor" is a dynamical attractor, not a stored weight.
  - It argues that per-synapse evolved rules can provide robustness without an explicit decay term.
  - It does not address keeping innate weights, since there are none.

### 10. Yaman, Iacca, Mocanu, Coler, Fletcher & Pechenizkiy 2021, "Evolving plasticity for autonomous learning under changing environmental conditions", *Evolutionary Computation* 29(3)
- URL: https://arxiv.org/abs/1904.01709 (v3). Full text via the arXiv PDF.
- **Mechanism:**
  - p.3–4: `Δw_ij = f(a_i, a_j, m)`, a discrete evolved table over binary activations and reward/punishment m.
  - p.4: per-neuron L2 normalisation `w' = w/‖w‖`, which "prevents indefinite synaptic growth". There is no decay.
  - Two "seasons" swap reward associations during a lifetime.
- **Findings:**
  - p.9: the best rules change weights only on punishment. "if the plasticity rule continues to perform synaptic changes when the network has already learnt the task, then these changes can disrupt the weights, causing the network to 'forget'."
  - p.12: "after the networks learn to perform the task successfully, continuing to perform synaptic changes may cause degradation in the synaptic weights and result in forgetting."
  - In the stochastic prey–predator task, by contrast, "the best ESP rules tend to perform frequent synaptic changes".
- **Mapping to Petri:**
  - Evidence that **evolution selects against continual drift** of learned weights when the environment is stable, and selects for ongoing change when it is stochastic.
  - Passive decay is a form of continual drift, so a fixed nonzero decay rate would be harmful in stable regimes. Decay should be evolvable down to zero, with zero an allowed value.

### 11. Giannakakis, Khajehabdollahi & Levina 2023, "Environmental variability and network structure determine the optimal plasticity mechanisms in embodied agents", *ALIFE 2023*
- URL: https://arxiv.org/abs/2303.06734. Full text via the arXiv PDF.
- **Mechanism (p.3):**
  - Evolved reward-modulated rule `ΔW_t = η_p[R_t(θ1 X y + θ2 y + θ3 X + θ4) + (θ5 X y + θ6 y + θ7 X + θ8)]`, followed by mean-subtraction normalisation.
  - The environment switches between E1 and E2 by a Markov process. Initial weights are set to the mean of the two environments.
  - The evolved rule converges to the delta rule `ΔW ≈ η_p X(R − y)` (p.5).
- **Findings (p.4):**
  - "the more unreliable an environment becomes, the less an agent relies on plasticity".
  - "the EA converges to a learning rate of η_p ≈ 0. This means that the agent opts to have no adaptation during its lifetime and remain at the mean of the two environments."
  - With transitions that are very fast, "the agents opt for slower or even no learning". With fast transitions, "the agent maintains an intermediate position between the two environments and never fully adapts to either of them."
- **Mapping to Petri:**
  - This is about the evolved **learning rate**, not a decay rate. In a delta rule, the learning rate also sets how fast old values are overwritten, so it is only an analogy.
  - It shows evolution tuning the speed of lifetime weight change to the statistics of environmental fluctuation, including choosing "stay at the innate, environment-averaged weight", which is the limit of infinitely fast decay to the genomic weight.
  - This supports making the decay rate heritable rather than global.

### 12. Miconi 2023, "Learning to acquire novel cognitive tasks with evolution, plasticity and meta-meta-learning", *ICML 2023*
- URL: https://arxiv.org/abs/2112.08588. Full text via the arXiv PDF.
- **Mechanism (p.4):**
  - `τ dx/dt = (W + Π ⊙ P(t)) r(t) − x(t)`
  - `τ_H dH/dt = Δx r^T − H`, a decaying eligibility trace
  - `P ← P + ηRH`, with no decay of P. W and Π are "innate … optimized across lifetimes by evolution".
  - "P(t) is initialized to 0 at the beginning of each block of trials ('lifetime') and changes … (without any reinitialization) during a whole block."
- **Findings:**
  - p.16, Appendix F: without evolution, one long lifetime with no reset between tasks gives poor performance.
  - p.18, Appendix G.4: "the model described here deliberately excludes an additional learning loop … the lifetime experience loop". Adding it would allow study of "prevention of interference between learned tasks ('catastrophic forgetting')".
- **Mapping to Petri:** This is the state-of-the-art evolved reward-modulated plastic RNN. It uses exactly Petri's current structure: innate W, a decaying trace, a non-decaying plastic weight, and a reset at birth. The author explicitly leaves the multi-task, long-lifetime case open. Nothing in this paper says whether weight decay helps there.

### 13. Dunlap & Stephens 2009, "Components of change in the evolution of learning and unlearned preference", *Proc. R. Soc. B* 276:3201–3208
- URL: https://pmc.ncbi.nlm.nih.gov/articles/PMC2817126/. Full text via PMC HTML.
- **Mechanism:** analytic model plus 30-generation experimental evolution in *Drosophila*. p is the fixity of the best action; q is the reliability of experience.
- **Findings:**
  - Appendix: "learning should be favoured whenever q > p."
  - Discussion: "randomness, and not fixity, is the most powerful and plausible way to select against learning."
  - In the fixed-best-action/unreliable-experience lines, they observed "reduced sensitivity to experience and increased reliance on unlearned preferences".
- **Mapping to Petri:** This is the evolutionary theory behind making decay heritable. Whether a creature should trust lifetime experience (keep learned weights) or innate preference (decay back to genomic) depends on the reliability of experience versus how fixed the best action is. Both vary across Petri worlds and niches, so no single global decay rate fits. This is not about forgetting directly. Stephens 1991 is cited here; I did not read it.

### 14. Guttenberg 2019, "Evolutionary rates of information gain and decay in fluctuating environments", *ALIFE 2019*, pp. 365–371
- URL: https://arxiv.org/abs/2104.03406. Full text via the arXiv PDF (7 pages).
- **Mechanism (p.3):** the population–environment mutual information follows `∂_t I = −I/λ+ + A/λ+` under selection and `−I/λ−` otherwise.
- **Finding (p.3–4):** "The fact that λ− and λ+ are not the same suggests that the decay is not just from mutation … but rather includes an effect where selection for one environment can influence the rate at which mutual information with a second environment is lost." The crossover between fluctuations helping and hurting is predicted where λ+ = λ−.
- **Mapping to Petri:** This is about information at the population/genome level, not synapses, so it is an **analogy only**. It does give a useful reminder: the genome itself forgets neglected environments. Lamarckian edges without decay, or genomic weights under drift, are the generational counterparts of lifetime forgetting.

### 15. Luo, Miras, Longhi, Weissl & Eiben 2024, "Lamarckian inheritance improves robot evolution in dynamic environments" (arXiv preprint)
- URL: https://arxiv.org/abs/2403.19545. Full text via the arXiv PDF. I read the results and discussion closely.
- **Mechanism:** modular robots whose controller (CPG) weights are learned by RevDE, a search algorithm rather than Hebbian plasticity. In the Lamarckian condition the learned weights are written back to the genotype. The environment changes between generations.
- **Finding (p.4):** "the Lamarckian system consistently outperforms the Darwinian system in all environmental conditions". When moving to harder terrain, the Lamarckian system dips and then "recovers immediately in the next generation". The Discussion (p.10) frames it as "a trade-off between overfitting and adaptability."
- **Mapping to Petri:** This is evidence for Lamarckian edges when the environment changes slowly relative to the generation time. It contrasts with Sasaki & Tokoro (abstract only, below). The interaction with decay is untested.

### Partially read: Ellefsen, Mouret & Clune 2015 (*PLoS Comput Biol* 11:e1004128) and Velez & Clune 2017 (*PLoS ONE* 12:e0187736)
- URLs: https://doi.org/10.1371/journal.pcbi.1004128 and https://doi.org/10.1371/journal.pone.0187736. I downloaded the PDFs. I read Ellefsen's abstract and Velez's abstract plus the "Learning and forgetting" and task sections (p.3–5).
- **Finding (Velez abstract):** diffusion-based neuromodulation "induces task-specific learning in groups of nodes and connections … and 3) yields higher performance by eliminating catastrophic forgetting." Task (p.5): seasons cycle, and the agent must not "forget what it learned in the prior season".
- **Mapping to Petri:** In *cyclic* environments, evolution solves forgetting by **localising** plasticity (modules, spatial modulation), not by decay. Passive decay would *hurt* in those tasks, because the old association is needed again next season. This bears on the group-scale idea: Petri's edge groups are a natural unit for localised plasticity.

### Excluded after checking
- **Pedersen & Risi 2021** (arXiv 2104.07959). Its "Weight Decay 0" (p.~5, Table 1) is a hyperparameter of the ES/Adam optimiser, not a synaptic term. Not relevant.
- **Fernando et al. 2018** (arXiv 1806.07917, Baldwin effect). Downloaded but not read; it has no decay matches.

---

## Synthesis

### Design options the literature supports, from strongest to weakest support

1. **A transient component that decays toward a persistent anchor.** Miconi 2018, Hinton–Plaut/Bullinaria 2009, HTP 2015 and Tyulmankov 2022 all use it. In Petri the equivalent single-weight form is `w ← g + (w − g)·d` per tick (or per visit), where g is the anchor and d in [0,1] is the retention. This matches the existing `trace_decay` form and the Classic/Oja/Covariance update path. The literature always decays the *deviation* to zero, never the whole weight to zero.
2. **An evolved decay rate.**
   - Bullinaria (evolved, global): converged to the same nonzero value in every run.
   - Tyulmankov (meta-learned, global): rate interacts with the rule.
   - Giannakakis and Dunlap & Stephens (theory and evolution of learning rate): the right timescale depends on environmental statistics.
   - **No source evolves a per-synapse decay rate.** Per-edge heritable decay is a reasonable extrapolation, not a replicated result. Per-synapse learning rates and rules *are* standard (Najarro & Risi, Miconi 2018's per-connection α).
3. **Consolidation: the anchor moves during life.** HTP's thresholded transfer from w_st to w_lt is hand-designed, and the Born to Learn review predicts such rules will become standard EPANN parts. In Petri, a consolidated weight could become the Lamarckian seed. This would give a principled "inherit only what was consolidated" rule.
4. **Decay that depends on modulation.** The biology cited by Soltoggio 2008 says modulated growth lasts longer. A design option is to decay pure-Hebbian deviations, and to let reward events reset or slow the decay of reward-modulated edges.
5. **No passive decay: active, gated relearning instead.** This is the most common choice in evolved agents (Soltoggio 2008, Yaman 2021, Najarro & Risi 2020, Backpropamine, Miconi 2023). It works there partly because lifetimes are short and reset.

### Natural analogs, all named in the sources read
- Short-term vs long-term synaptic potentiation (HTP; Born to Learn).
- Protein-synthesis-dependent, modulation-induced long-lasting growth (Soltoggio 2008, citing Bailey et al. 2000).
- Hinton–Plaut fast weights.
- Adaptive forgetting in animals (Kraemer & Golding 1997, abstract only).
- Evolution of learning vs. unlearned preference (Dunlap & Stephens 2009, *Drosophila*).

For Petri's natural-analog rule, "decay of the unconsolidated (early-LTP-like) component of a synapse back to its developmental set point" is the cleanest name. "Synaptic homeostasis toward the set point" is also used informally. Note that it is not in the sources read.

### On evolving the decay rate per synapse
- **Expected optimum:** depends on how reliable rewards and experience are versus how fixed the best action is (Dunlap & Stephens), and on reward sparsity (HTP's τ discussion). Petri's worlds differ on both, which argues for heritable rates, not a global constant.
- **Allow d = 1 (no decay).** Yaman shows evolution prefers no drift once a task is learned in stable regimes. Bullinaria shows global decay toward zero being evolved away.
- **Allow fast decay.** Giannakakis shows the "stay at innate" corner (η → 0, which is equivalent to instant decay) is sometimes optimal.
- **The rate interacts with the rule.** In Tyulmankov, an anti-Hebbian rule allows slower decay. Mutating rule and decay independently may create epistasis. This is not studied in evolved per-synapse settings.

### Known failure modes and risks
1. **Erasing innate or useful learned wiring.**
   - Miconi 2018: decay makes "memories decay to zero in the absence of input".
   - HTP: too short a τ means "hypotheses are forgotten too quickly" when rewards are sparse.
   - Velez/Ellefsen: in cyclic environments the forgotten association is needed again.
2. **The null result.** Backpropamine found a clamp alone did as well as or better than decay on its tasks. Petri already clamps (`weight_clamp`). Decay adds a parameter that may be neutral. Neutral heritable parameters drift, which raises the evolvability-noise question.
3. **Baldwin interaction.** Decay toward the genomic weight erases learning that is not genetically encoded. This raises the selective value of genetic assimilation (Floreano & Urzelai describe the Baldwin effect reducing plasticity over time; Born to Learn describes static environments ending with agents "born knowing"). Expect plasticity to fall in stable Petri worlds once decay exists. That outcome may be desired or not, so measure it.
4. **Lamarckian inheritance with decay — the anchor fork.** In Petri, Lamarckian slots start at the parent's learned seed, while `genomic` is a different stored number.
   - Decay toward `genomic`: the inherited learned weight is pulled back over the child's life. Lamarckian inheritance then becomes a fading head start. This is closest to partial heritability, the "various rates of inheritance of acquired characters" in Sasaki & Tokoro (abstract only).
   - Decay toward the birth seed: inherited learning becomes the new set point. Lamarckian lineages then lose any return path to the genomic weight, and drift can build up across generations (Guttenberg's generational information loss, by analogy).
   - The evidence is split: Sasaki & Tokoro 1999 (abstract only) found Darwinian populations adapt best to dynamic environments, while Luo et al. 2024 (read) found Lamarckian better. These two use different learning mechanisms and timescales of change.
5. **Scope of decay.**
   - Decaying the trace (exists) and decaying the weight (proposed) are different. HTP uses both at timescales thousands of times apart (seconds vs hours).
   - A group-scale analog has no direct precedent in the sources. The closest is Miconi 2018's comparison of a shared α (one plasticity coefficient for all connections, which the paper likens to fast weights) versus per-connection α. Per-connection α won on its task: "independent plasticity coefficients for each connection improve performance for this task" (p.5). That is weak evidence for per-edge decay rates over a single shared group rate.
6. **Most results come from reset-per-episode setups** (Miconi 2018/2019/2023, Najarro & Risi, Floreano & Urzelai, Bullinaria's per-generation random initial weights). Transfer to Petri's continual, long-lived creatures is not established. Tyulmankov is the one continual, no-reset case, and there decay is what makes continual operation work.

### Open questions for Petri
- Does decay toward `genomic` reduce the reward-coincidence random walk on reward-modulated edges (HTP Fig. 3) in live worlds, or is `weight_clamp` already enough (Backpropamine)? This can be measured from current telemetry by checking whether learned weights of reward-modulated edges drift with zero-mean reward correlation.
- Should the anchor for Lamarckian edges be `genomic` or the birth seed? Should consolidation (HTP-like) decide what becomes the seed?
- Should decay be per edge, per group (shared scale), or applied only to unmodulated edges (the Soltoggio 2008 analog)?
- What prior on retention d: a mass at d = 1 (no decay, the current behaviour) so the founder stays neutral?
- Do decay rate and rule co-evolve, given the Tyulmankov rule–rate interaction?

---

## Claims I could NOT verify in full text
- **Kraemer & Golding 1997**, "Adaptive forgetting in animals", *Psychon. Bull. Rev.* 4:480–491. Paywalled; abstract only, read via the browser. It argues "some cases of forgetting in animals are adaptive" and proposes "retrievability" as a mechanism.
- **Sasaki & Tokoro 1999**, *Artificial Life* 5:203–223, and **Sasaki & Tokoro 2000**, *KAIS*. Abstract only, from Sony CSL: "Populations with lower rates of heritability not only show more stable behavior against environmental changes, but also maintain greater adaptability … the Darwinian population, attains the highest level of adaptation to dynamic environments." Also Yamamoto, Sasaki & Tokoro, ECAL 1999: not read.
- **Ellefsen et al. 2015.** Abstract only. **Velez & Clune 2017.** Abstract plus methods sections only.
- **Stephens 1991**, *Behav. Ecol.* 2:77–89. Not read. Known only through Dunlap & Stephens 2009's description: it predicts non-learning under complete fixity because of an opportunity cost of learning.
- **Dukas 1998/1999** (costs of memory). Not obtained or read. Dunlap & Stephens 2014 (*PNAS*, prepared learning) and 2016 (*Curr. Opin. Behav. Sci.*). Not read.
- **Hinton & Plaut 1987, Levy & Bairaktaris 1995, Seipone & Bullinaria 2005, Kirkpatrick et al. 2017 (EWC), Bailey et al. 2000.** Known only as cited inside the papers above.
- **HTP Appendix 1** (reversal and undoing consolidation). Not read in detail.
- **Fernando et al. 2018** (Baldwin effect meta-learning). Downloaded, not read.
- **No source found that evolves a per-synapse decay rate in ALife agents.** Extended searches turned up none. This is an absence in what I found, not proof that none exists.
