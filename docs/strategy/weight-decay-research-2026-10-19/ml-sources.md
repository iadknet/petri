# Decaying learned weights toward the genomic weight: what ML research shows

Date: 2026-10-19. Read-only research for Petri (heritable per-edge decay toward the genomic weight).

All PDFs came from arXiv with `curl`. Their text was extracted locally with PyMuPDF (`python3 -I`) in a scratch folder (not kept). Hinton & Plaut 1987 came as Plaut's plain-text copy from ni.cmu.edu. Quotes are verbatim from those extractions. Notation is the paper's own.

## TL;DR

1. **Fast-weight and plastic-network models have always used "decay toward the initial weight".** The effective weight is a fixed or slow baseline plus a lifetime offset, and the offset decays toward zero. So the effective weight relaxes toward the baseline, which is Petri's genomic weight. This holds from Hinton & Plaut 1987 through Ba 2016, Miconi 2018 and STPN 2022. None of these papers decays the plastic part toward absolute zero. Petri's proposal is the standard design here, not a new one.
2. **In continual learning with gradient descent, decaying toward the initial weight beats decaying toward zero on plasticity in one solid study.** Kumar et al. (L2 Init) also show that a fixed initial point matters: a resampled random target fails. The evidence is mixed overall:
   - Lyle 2024 found layer norm plus plain L2 the strongest combination.
   - Lyle 2023 found that weight decay and shrink-and-perturb did not help in toy RL.
   - Elsayed 2024 argues any pull toward a fixed point overwrites useful weights. In their tests a clamp alone kept improving where L2 Init plateaued.
   - Kumar's own appendix shows L2 Init buys plasticity with more forgetting.
3. **Per-parameter decay rates beat a single global rate in every study that isolates the rate.** FADE 2026 shows this cleanly, but it decays toward zero and only in the output layer. STPN shows per-synapse beats uniform, but it learns the decay and learning rates jointly. Evidence about per-connection plasticity magnitude (Miconi 2018) is related but does not isolate the decay rate.
4. **Petri already has the runaway guard: the clamp.** Backpropamine found a hard clip as good as or better than decay or Oja's rule for bounding Hebbian traces. Those were short episodes with the plastic state reset each episode. What decay adds in Petri's long-lifetime, changing-world setting is regeneration: unreinforced learning returns to the evolved prior. Bounding runaway is not the new contribution.
5. **Cheapest form: store the deviation `d = w − w_g`, not the absolute weight, and multiply `d` by a retention factor.**
   - Reward-modulated slots: do it eagerly inside the existing per-tick trace-decay loop.
   - Pure-Hebbian slots: do it lazily on visit, with integer exponentiation by squaring for determinism.
   - The retention range must include exactly 1.0, so evolution can keep the current behaviour.

---

## Sources (read in full unless marked)

### 1. Hinton & Plaut 1987, "Using fast weights to deblur old memories"
- Proc. 9th Cognitive Science Society, pp. 177–186. Obtained as the author's plain-text copy: https://ni.cmu.edu/~plaut/papers/txt/HintonPlaut87CogSciConf.fastWeights.txt. The text copy has missing figures, so the figure contents were not seen.
- **Update:** effective weight = slow weight + fast weight. Both receive the same error derivative, but the fast weight has its own rate and also shrinks by a fixed fraction after each update. In their simulation the fast weights decay "towards zero by 1% after each weight update". That is `f ← 0.99·f` after each step, so the effective weight relaxes toward the slow weight.
- **Finding:** the fast offset lets a few rehearsals temporarily cancel interference in the slow weights. When it decays, the newer knowledge returns.
- Quotes:
  - "a fast-changing, elastic weight which stores temporary knowledge and spontaneously decays towards zero" (Abstract).
  - "The effective weight on the connection is the sum of these two" (Introduction).
  - "As soon as the fast weights have decayed back to zero, the new knowledge is restored."
- **Petri mapping:** the slow weight corresponds to the genomic weight, which evolution changes between lifetimes. The fast weight corresponds to the learned deviation `w − w_g`. This is the oldest precedent for decaying toward the genomic weight. One difference: Petri's genomic weight is fixed during life, while H&P's slow weight also learns.

### 2. Ba, Hinton, Mnih, Leibo, Ionescu 2016, "Using fast weights to attend to the recent past"
- arXiv:1610.06258, full text.
- **Update (eq. 1):** `A(t) = λA(t−1) + η h(t)h(t)ᵀ`, with slow weights W and C. Inner loop: `h_{s+1}(t+1) = f([Wh(t) + Cx(t)] + A(t)h_s(t+1))`.
- **Closed form (eq. 3):** `A(t) = η Σ_{τ=1..t} λ^{t−τ} h(τ)h(τ)ᵀ`.
- Defaults: "fast learning rate of 0.5 and decay rate of 0.95". Layer norm was added because the fast memory was unstable: "applying layer normalization on each iteration of the inner loop makes the fast associative memory more robust to the choice of learning rate and decay hyper-parameters" (§3.1).
- Quotes:
  - "we will use an outer product rule to store hidden activity vectors in fast weights that decay rapidly" (§3).
  - The update rule is "simply to multiply the current fast weights by a decay rate, λ, and add the outer product of the hidden state vector … multiplied by a learning rate, η" (§3).
- **Petri mapping:**
  - The total weight W + A decays toward W, which plays the role of the genomic weight.
  - Eq. 3 is the precedent for computing decay lazily in closed form as `λ^{Δt}`.
  - λ is a single global constant here, not per synapse.

### 3. Miconi, Clune, Stanley 2018, "Differentiable plasticity"
- arXiv:1804.02464 (ICML 2018), full text.
- **Update:**
  - eq. 1: `x_j(t) = σ(Σ_i [w_ij x_i(t−1) + α_ij Hebb_ij(t) x_i(t−1)])`
  - eq. 2: `Hebb_ij(t+1) = η x_i(t−1)x_j(t) + (1−η) Hebb_ij(t)`
  - eq. 3 (Oja variant): `Hebb_ij(t+1) = Hebb_ij(t) + η x_j(t)(x_i(t−1) − x_j(t)Hebb_ij(t))`
  - Hebb resets to 0 each lifetime. w, α and η are meta-learned.
- **Finding on decay:**
  - "In Equation 2, η appears as a weight decay term, to prevent runaway positive feedback on Hebbian traces. However, because of this weight decay, Hebbian traces (and thus memories) decay to zero in the absence of input." (§2)
  - They offer Oja's rule as an alternative "(note the absence of a decay term)" that keeps memories stable while still bounding the weights.
  - η, which is both the learning rate and the decay rate, is a single network-wide scalar: "all connections share the same value of η" (§2).
- **Finding on per-connection vs shared parameters (magnitude, not decay):**
  - "independent plasticity coefficients for each connection improve performance for this task" (§4.3, image reconstruction).
  - In the maze task, "homogenous-plasticity networks also settle on a low performance plateau… individually sculpting the plasticity of each connection is crucial" (§4.5).
  - This compares per-connection α against shared α. Decay is tied to η and shared in both arms, so this is not evidence about per-edge decay rates.
- **Petri mapping:**
  - The effective weight is `w + α·Hebb`, with Hebb decaying toward 0. That is decay toward the meta-learned weight w, the analogue of Petri's genomic weight.
  - Miconi names the trade-off directly: decay bounds runaway but erases memories.
  - Petri's Oja rule already has a bounding term. See synthesis §3.

### 4. Miconi, Rawal, Clune, Stanley 2019, "Backpropamine"
- arXiv:2002.10585 (ICLR 2019), full text.
- **Update:**
  - eq. 2: `Hebb_ij(t+1) = Clip(Hebb_ij(t) + η x_i(t−1)x_j(t))`, a hard clip to [−1, 1].
  - Retroactive (three-factor) variant, eqs. 4–5: `Hebb_ij(t+1) = Clip(Hebb_ij(t) + M(t)E_ij(t))` and `E_ij(t+1) = (1−η)E_ij(t) + η x_i(t−1)x_j(t)`.
- **Finding:**
  - "In previous work … this function was either a simple decay term, or a normalization implementing Oja's rule … In the present paper it is simply a hard clip … this simple operation turned out to produce equal or superior performance on the tasks in this paper." (§3.1)
  - α_ij scales "the maximum magnitude of the plastic component".
  - The eligibility trace is "a simple exponential average … with trainable decay factor η".
- **Petri mapping:**
  - This is almost exactly Petri today: a decaying eligibility trace, three-factor accumulation, a clamp, and no passive decay of the learned weight.
  - Two differences matter:
    - (a) Backpropamine's clip bounds the deviation from the baseline w, so it is centred on the evolved weight. Petri's clamp bounds the absolute weight and is centred on 0.
    - (b) Hebb resets every episode, so long-run staleness never arises. Petri creatures live long lives in changing worlds.
  - The finding that clipping is enough concerns runaway control in short episodes. It says nothing against decay as a regeneration mechanism over long lifetimes.

### 5. Rodriguez, Guo, Moraitis 2022, "Short-Term Plasticity Neurons Learning to Learn and Forget" (STPN)
- arXiv:2206.14048 (ICML 2022), full text.
- **Update:**
  - eqs. 1–3: `G(t) = W + F(t)`, `h(t) = σ(G(t)x(t))`, `F(t+1) = Γ⊙(x(t)⊗h(t)) + (1−Λ)⊙F(t)`.
  - λ_ij and γ_ij are trained per synapse. In Algorithm 1, F is normalized by ‖G‖, described as "optional but practically impactful".
- **Finding:**
  - "γ is also the Hebbian learning rate of the synapse, whereas λ is its forget rate" (§2.3).
  - "explicitly learning decaying dynamics of synaptic memories helps adding stability … as less relevant synapses will be closer to their long-term value w_ij" (§4.1). The authors call this a hypothesis.
- **Per-synapse vs uniform (Appendix A.2, Tables 1–2):**
  - ART: recurrent per-synapse 99.99 vs uniform 96.89. Feed-forward 89.40 vs 60.21.
  - Maze: recurrent 115.7 vs 74.0.
  - Continual familiarity task, recurrent: per-synapse 97.53 / 99.99 / 98.98 vs uniform 78.52 / 94.13 / 90.88.
  - Caveat: "uniform" shares both λ and γ, so the decay rate's contribution is not isolated.
- **Petri mapping:** this is the closest ML precedent for a heritable per-edge retention toward the genomic weight. It has a per-synapse learned forget rate, the effective weight relaxes toward the learned long-term weight, and the rates are set by an outer loop, which in Petri would be evolution.

### 6. Li, Grandvalet, Davoine 2018, "Explicit Inductive Bias for Transfer Learning with Convolutional Networks" (L2-SP)
- arXiv:1802.01483 (ICML 2018), full text.
- **Penalty:**
  - eq. 1, ordinary L2: `Ω(w) = α/2 ‖w‖²`.
  - eq. 2, L2-SP: `Ω(w) = α/2 ‖w − w⁰‖²`.
  - eq. 3 applies L2-SP to the inherited layers and plain L2 to new layers.
  - With SGD at step size s, one step is `w ← w⁰ + (w − w⁰)(1 − sα)` plus the gradient. This is exactly the proposed per-tick retention form (my derivation).
- **Finding:**
  - "all the fine-tuning strategies based on penalties using the starting point -SP as a reference perform consistently better than standard fine-tuning (L2). There is thus a benefit in having an explicit bias towards the starting point" (§4.3.2).
  - Theory: L2-SP converges to "a compromise between the optimum of the unregularized objective function and the pre-trained parameter vector", while L2 converges toward the origin (§4.4.2).
  - Cost: "less than 1%" extra FLOPs.
- **Petri mapping:** the evolved genome is the "pre-trained model" and lifetime learning is the "fine-tuning". This is the cleanest case for preferring the prior over zero as the decay target. Caveat: this is a transfer setting, not a non-stationary one.

### 7. Kirkpatrick et al. 2017, EWC
- arXiv:1612.00796 (PNAS), full text.
- **Penalty (eq. 3):** `L(θ) = L_B(θ) + Σ_i λ/2 F_i (θ_i − θ*_{A,i})²`.
- **Finding on per-parameter stiffness:**
  - "the stiffness of this spring should not be the same for all parameters; rather, it should be greater for those parameters that matter most" (§2).
  - Fig. 1 caption: "if we constrain each weight with the same coefficient … the restriction imposed is too severe".
- **Petri mapping:**
  - EWC anchors to the end of the previous task, weighted by importance. That supports per-parameter anchor strength, which matches a heritable per-edge retention.
  - The Fig. 1 argument is conceptual and schematic. It is not a measured ablation of decay rates.

### 8. Zenke, Poole, Ganguli 2017, Synaptic Intelligence
- arXiv:1703.04200 (ICML 2017), full text.
- **Penalty:**
  - eq. 4: `L̃_μ = L_μ + c Σ_k Ω_k^μ (θ̃_k − θ_k)²`
  - eq. 5: `Ω_k^μ = Σ_{ν<μ} ω_k^ν / ((Δ_k^ν)² + ξ)`
  - ω is a path integral of each parameter's contribution to loss reduction, computed locally and online.
- **Finding:** "the synaptic state tracks the past and current parameter value, and maintains an online estimate of the synapse's 'importance'" (Intro).
- **Petri mapping:** a per-synapse anchor strength computed locally from the synapse's own history. A future refinement could make retention depend on accumulated reward credit (consolidation). That is out of scope for a fixed heritable retention.

### 9. Kumar, Marklund, Van Roy 2024, "Maintaining Plasticity in Continual Learning via Regenerative Regularization" (L2 Init)
- arXiv:2308.11958v3 (CoLLAs 2024), full text.
- **Penalty and update:**
  - Penalty: `L_reg(θ) = L_train(θ) + λ‖θ − θ₀‖²`.
  - SGD form from Appendix A.3: `θ_{t+1} = (1 − αλ)θ_t + αλθ₀ − α∇L_train(θ_t)`, i.e. shrink plus a pull toward θ₀.
- **Finding:**
  - Mechanism: "when recent losses are insensitive to particular parameters, these parameters should drift toward their initial values. This prepares parameters to adapt quickly to new tasks" (Abstract).
  - Across 5 problems, "L2 Init consistently retains high plasticity". Plain L2 "performs poorly on Random Label MNIST, Random Label CIFAR, and 5+1 CIFAR" (Fig. 1 caption).
  - Feature rank: "When using L2, the effective feature rank is smaller than it is when applying L2 Init."
  - **Ablation:** regularizing "towards the initial parameters rather than sampling a new set of parameters at each time step performs much better". L1 Init helps but is slightly worse.
  - **Cost (Appendix A.2.6, Permuted MNIST, 20 tasks):** backward transfer is −63.4% for L2 Init vs −39.0% for EWC. The authors: "this study has focused exclusively on maintaining plasticity, leaving aside the issue of forgetting."
- **Petri mapping:**
  - The strongest direct evidence for decaying toward the initial weight rather than toward zero, and for using the specific fixed starting point rather than a random one.
  - The genomic weight is exactly such a specific fixed point, and evolution has tuned it.
  - Caveat: the mechanisms Kumar cites (dead ReLUs, rank collapse, Adam step bounds) belong to deep gradient-trained networks and may not apply to small Hebbian graphs.

### 10. Dohare, Hernandez-Garcia, Rahman, Mahmood, Sutton, "Maintaining Plasticity in Deep Continual Learning"
- arXiv:2306.13812v3, April 2024, full text. This is the preprint behind the Nature 2024 paper "Loss of plasticity in deep continual learning". I read the preprint, not the Nature text.
- **Updates:**
  - Shrink-and-perturb: L2 plus small noise every step.
  - Continual backprop: reinitialize a fraction ρ of low-utility units each step, with utility `c_{l,i,t} = η c_{l,i,t−1} + (1−η)|h_{l,i,t}| Σ_k |w_{l,i,k,t}|` (eq. 2).
  - The maturity threshold m protects new units from immediate reinitialization.
- **Finding (§5):**
  - "L2-regularization and shrink-and-perturb reduce the loss of plasticity … both shrink-and-perturb and L2-regularization are very sensitive to hyperparameter values."
  - "L2-regularization on its own does not mitigate the loss of plasticity … continual injection of randomness is important."
  - L2 to zero "increases the percentage of dead units and decreases the effective rank."
- **Petri mapping:**
  - Decay toward zero alone is insufficient and fragile. That is consistent with preferring the genomic weight as the target, which is the role Kumar casts as "regenerative".
  - Petri's mutation already injects fresh randomness between generations.

### 11. Ash & Adams 2020, "On Warm-Starting Neural Network Training" (shrink-and-perturb)
- arXiv:1910.08475 (NeurIPS 2020), full text.
- **Update:** at each round, `θ_i^t ← λθ_i^{t−1} + p^t`, with `p^t ∼ N(0, σ²)` and `0 < λ < 1` (§4). Applied every step this gives `θ_i ← λ(θ_i − η ∂L/∂θ_i) + p`, making "the shrinkage term λ behave like a weight decay parameter" (§4.1).
- **Finding:** "This L2 regularization benefit is not enough to explain the success of the shrink-perturb trick … L2-regularized models are still vulnerable to the warm-start generalization gap" (§4.1).
- **Petri mapping:** the shrink targets zero. Kumar's A.3 argues L2 Init is the version with a fixed target, `p = 1 − αλ` and `σ = αλ`, where the "noise" is θ₀ itself.

### 12. Lyle et al. 2023, "Understanding Plasticity in Neural Networks"; Lyle et al. 2024, "Disentangling the Causes of Plasticity Loss in Neural Networks"
- arXiv:2303.01486 (ICML 2023) and arXiv:2402.18762, full text.
- **Counter-evidence:**
  - 2023, Appendix B.1: "resetting the last layer, incorporating a two-hot output representation, and performing layer normalization have beneficial effects on plasticity, whereas shrink and perturb, weight decay, and resetting only the optimizer state do not improve plasticity" (toy RL, ℓ2 coefficient 1e−5).
  - 2023 §5 sets up a test: a statistic that correlates positively with plasticity in one setting and negatively in another "can be ruled out as a causal factor". I read the premise but not the full per-statistic verdict, which is in the figures.
  - 2024, Abstract: "a combination of layer normalization and weight decay is highly effective at maintaining plasticity".
  - 2024, Fig. 4 caption: "combining layer normalization with L2 regularization addressed plasticity loss in all classification problems we considered. Many other strategies also improve over doing nothing, but do not outperform this baseline". The Fig. 4 legend includes `LN | L2-Init`, but the text gives no number for it. I can only say the caption ranks no strategy above LN+L2.
  - 2024 also cautions that regularizing "towards their initial value … can run the risk of interfering with the primary objective", and that L2 "does so at the expense of convergence speed".
- **Petri mapping:** decay is not a universal fix, and its benefit depends on the architecture. Petri has no layer norm, and its brains are small evolved graphs.

### 13. Elsayed, Lan, Lyle, Mahmood 2024, "Weight Clipping for Deep Continual and Reinforcement Learning"
- arXiv:2407.01704 (RLC 2024), full text.
- **Update:** after each optimizer step, `W_l ← Clip(W_l, −κs_l, κs_l)`, where s_l is the initialization bound and κ is for example 2 (Algorithm 1).
- **Finding:**
  - "The issue of biasing toward a particular point is that it applies to all weights regardless of their usefulness, resulting in overwriting and re-learning of previously useful weights" (§1).
  - On label-permuted EMNIST and mini-ImageNet, "while all methods addressing loss of plasticity can maintain their performance, only weight clipping can keep improving its performance, likely due to not biasing the weights toward a specific point" (§4.2). This compares against L2 Init and shrink-and-perturb.
- **Petri mapping:**
  - This is the strongest argument against decay. Petri already clamps, which is the weight-clipping analogue. So decay should be justified as a forgetting mechanism, not a norm-control mechanism.
  - A per-edge heritable retention with 1.0 allowed lets evolution pick "clamp only" wherever decay costs useful learning.

### 14. Ramesh, Lewandowski, Schmidhuber 2026, "Learning to Forget: Continual Learning with Adaptive Weight Decay" (FADE)
- arXiv:2604.27063 (preprint, April 2026), full text.
- **Update (eqs. 1–4):** `w_{t+1}^i = (1 − λ_{t+1}^i) w_t^i + α δ_t x_t^i`, with `λ^i = exp(γ^i)`. γ is adapted online by an IDBD-style meta-gradient with sensitivity trace `g_{t+1}^i = g_t^i[1 − λ − α(x^i)²]_+ − λ w_t^i`.
- **Finding:**
  - "a fixed scalar weight decay drives this forgetting uniformly over time and uniformly across all parameters, even when some encode stable knowledge while others track rapidly changing targets" (Abstract).
  - "unrolling the weight update yields an exponentially weighted sum with an effective memory horizon of ∼1/λ^i" (§2).
  - Linear tracking: irrelevant weights converge to λ ≈ 0.9 and relevant ones to λ ≈ 0.02 (Fig. 1). MSE was 1.653 for FADE vs 2.726 for SGD + fixed weight decay (Table 1).
  - Nonlinear tracking: FADE + SGD 0.0073 vs AdamW 0.0138.
  - Label-permuted EMNIST: 0.807 vs 0.612 for weight clipping.
  - Fixed decay applied only to the output layer is a strong but "fragile" baseline: accuracy ranged 0.531–0.801 depending on λ₀.
  - Extending FADE naively to all layers gave only 0.535.
- **Caveats:** the decay target is zero, not the initial weight. Only the final, linear layer is adapted. It is a recent preprint, not peer reviewed as far as I know.
- **Petri mapping:**
  - The best isolated evidence that per-parameter forgetting rates beat a single global one, and that the right rate differs by how fast each target changes.
  - Evolution can find per-edge retentions offline, much as FADE's meta-gradient finds them online.
  - A second-hand claim from this paper, not verified in Elsayed: weight clipping "was the best-performing approach on this problem in the experiment conducted by Elsayed et al. (2024), outperforming other strategies such as L2 init".

### 15. Schlag, Irie, Schmidhuber 2021, "Linear Transformers Are Secretly Fast Weight Programmers"
- arXiv:2102.11174 (ICML 2021), full text.
- **Update:**
  - Delta rule (eq. 24): `W(i) = W(i−1) + β(i)(v(i) − v̄(i)) ⊗ φ(k(i))`, with `v̄ = W(i−1)φ(k(i))`.
  - Peng et al.'s gated rule, quoted in Appendix B (eq. 52): `W(i) = (1−β(i))W(i−1) + β(i) v(i) ⊗ φ(k(i))`.
- **Finding (Appendix B):**
  - Under the gated rule, an unrelated stored association becomes `W′k₁ = (1−β)v₁`. The gate "also modifies or in the worst case erases the value associated with the key k₁".
  - Under the delta rule, `W′k₁ = v₁`, which "keeps the value associated with k₁ unmodified".
- **Petri mapping:**
  - Uniform passive decay erases unrelated learned associations too. That is the same objection Elsayed raises.
  - Error-correcting rules (Oja behaves somewhat like this) correct targeted values without blanket forgetting.

### 16. Yang, Kautz, Hatamizadeh 2025, "Gated Delta Networks: Improving Mamba2 with Delta Rule"
- arXiv:2412.06464 (ICLR 2025), full text.
- **Update:**
  - Mamba2: `S_t = α_t S_{t−1} + v_t k_tᵀ`, with α_t ∈ (0,1) "a data-dependent scalar-valued decay term".
  - Gated delta rule (eq. 10): `S_t = S_{t−1}(α_t(I − β_t k_t k_tᵀ)) + β_t v_t k_tᵀ`.
  - Read as test-time SGD, this is "incorporating an adaptive weight decay term α_t into the SGD update" (§3.1).
- **Finding:**
  - "gating enables rapid memory erasure while the delta rule facilitates targeted" updates (Abstract).
  - S-NIAH case study, "Decay hurts memory retention": Mamba2 "decays historical information too quickly". "Gating facilitates filtering": without clearance, "memory collision" (§3.2).
  - Related work: the field moved "from using data-independent decay mechanisms" (S4, RetNet, …) "to incorporating data-dependent decay mechanisms" (Mamba, HGRN, …).
- **Petri mapping:**
  - Engineering precedent that a forgetting gate plus a targeted rule beats either alone.
  - The field's shift is from fixed decay to input-dependent decay. It is not a shift from shared to per-element decay; α_t here is a scalar per head.
  - Petri's analogue of data-dependent decay would be retention modulated by a reward or outcome signal. Possible later; not needed for a first step.

### 17. Najarro & Risi 2020, "Meta-Learning through Hebbian Plasticity in Random Networks"
- arXiv:2007.02686 (NeurIPS 2020), full text.
- **Update (eq. 1):** `Δw_ij = η_w(A_w o_i o_j + B_w o_i + C_w o_j + D_w)`. The coefficients are evolved per synapse by evolution strategies, and the weights are random at the start of every episode.
- **Finding:**
  - Pure per-synapse Hebb ("a single unique coefficient per synapse A") "is incapable of solving the task" (§6.3).
  - The car-racing weights are "normalised to the interval [−1,+1], while the quadruped agents have unbounded weights" (appendix).
  - Co-evolving initial weights "does not increase the training efficiency" and caused 1 of 3 runs to stall (§6.4).
- **Petri mapping:**
  - Evolved per-synapse rules with no decay term work when lifetimes are episodes.
  - Weak hint: the evolved initial weights mattered less than the rules there. For Petri that means decaying toward the genomic weight only helps if the genomic weight carries useful information. Under T25 it usually should, since evolution shapes it directly.

---

## Synthesis

### (1) Decay toward the initial weight vs toward zero vs no decay

**Two different literatures. Lead with the first.**

- **Plastic and fast-weight networks** (H&P 1987, Ba 2016, Miconi 2018, STPN 2022):
  - The plastic offset decays toward 0 inside an effective weight `baseline + offset`. That is uniformly decay toward the baseline.
  - None of these papers decays the whole effective weight toward zero.
  - Petri's proposal `w ← w_g + (w − w_g)·r` is exactly this design, with the genomic weight as the slow weight.
  - Within this line the debate is not about the target. It is about decay vs Oja vs clip as the bounding mechanism. Backpropamine settled on clipping for short episodes with a reset each episode.
- **Continual learning with gradient descent** (L2-SP, Kumar, Dohare, Lyle, Elsayed, FADE):
  - Evidence for the initial weight over zero: L2-SP for transfer; Kumar on 5 continual problems, plus the ablation showing the specific θ₀ matters; Dohare's finding that L2 to zero alone is insufficient.
  - Evidence against any fixed target, or for zero plus normalization: Lyle 2024 (LN+L2 best); Lyle 2023 (weight decay and shrink-and-perturb don't help in toy RL); Elsayed (clipping keeps improving where pulling toward a point plateaus); Kumar's own backward-transfer cost.
  - This literature transfers to Petri only weakly. Its failure mechanisms (dead ReLUs, rank collapse, Adam step bounds) belong to large gradient-trained networks, not small evolved Hebbian graphs.
- **Reading for Petri:**
  - **Decay toward the genomic weight, never toward zero.** Zero carries no evolved information. Petri's genomic weight is a specific prior, which Kumar's ablation shows is what matters.
  - **No decay**, the status quo, is defensible if the clamp alone bounds runaway, as in Backpropamine and Elsayed.
  - Petri's gap is different: an edge whose correlation or reward has vanished keeps its learned value for life. Kumar ("drift toward their initial values … recruited for future adaptation") and FADE ("controlled forgetting … freeing up capacity") address exactly that gap. The clamp does not.
  - **The cost is real.** Decay forgets useful learning that isn't refreshed. Kumar measured backward transfer of −63.4% for L2 Init vs −39.0% for EWC, and Schlag App. B and Elsayed §1 explain the mechanism. A heritable retention that includes 1.0 lets selection make this trade per edge.

### (2) Learned or evolved per-unit decay vs one global rate

Every source that compares the two favours per-unit rates. Each isolates something different:

| Source | What is per-unit | Isolates decay rate? | Result |
|---|---|---|---|
| FADE 2026 | λ per parameter, toward 0, output layer only | **Yes** | Beats every fixed λ. Relevant weights λ≈0.02, irrelevant λ≈0.9. Robust to λ₀ |
| STPN 2022 | λ_ij and γ_ij per synapse | No (joint) | Per-synapse ≫ uniform (e.g. maze 115.7 vs 74.0) |
| Miconi 2018 | α_ij (plastic magnitude); η global | No (magnitude) | Per-connection α > shared α |
| EWC 2017 | anchor stiffness F_i | Conceptual | Uniform stiffness "too severe" (Fig. 1 schematic) |
| Gated DeltaNet 2025 | α_t data-dependent, scalar per head | No (time, not unit) | Field moved from fixed to data-dependent decay |

- **Petri implication:** a heritable per-edge retention fits the evidence. Evolution can do offline what FADE does online. Per edge is the right granularity; a per-genome or global knob would sit in the "fragile" regime FADE reports for fixed λ.
- **Caveats:**
  - FADE's own all-layer extension did much worse (0.535), so per-parameter rates in hidden layers are harder to set well.
  - Evolution's credit assignment for a per-edge scalar is much weaker than a meta-gradient's. Expect slow tuning, which the per-group option below partly addresses.
  - Petri's edge groups could share a retention, the way groups already share a learnable scale. That gives fewer parameters for evolution to tune. No source tests group-shared decay; this is my inference.

### (3) Interaction with clamps and with Hebbian runaway

- **Decay is a soft bound.** For Classic Hebb with constant pre and post, decay gives a finite fixed-point deviation `d* ≈ η·pre·post / (1 − r)` per tick when the edge is visited every tick (my derivation). Miconi 2018 introduced decay explicitly "to prevent runaway positive feedback". With recurrent positive feedback, decay only bounds runaway if `1 − r` exceeds the loop gain. The clamp remains the hard guard, as in Backpropamine's hard clip and Elsayed's clipping.
- **The clamp and the decay target can disagree in Petri** (code check):
  - `from_seeds` clamps Lamarckian seeds but not `slot.genomic` (`learned.rs:58–73`).
  - `perturb_weight` scales edge weights multiplicatively with no bound (`mutation/graph/operators.rs:561`).
  - `weight_clamp` ∈ [0.01, 10] (`genome/mod.rs:202`).
  - So nothing I found guarantees `|w_g| ≤ weight_clamp`. If `|w_g|` exceeds the clamp, decay pulls the weight toward a target outside the box and the clamp pins it at the edge.
  - Options: clamp the target, or adopt Backpropamine's centring and clamp the deviation `w − w_g` instead of the absolute weight. That is a design choice with re-pinning consequences; flagged, not resolved.
- **Oja already contains a decay toward zero.** `ΔW = η·post·(pre − w·post)` includes `−η·post²·w`, a weight-proportional decay toward 0 gated by post-synaptic activity. Adding decay toward w_g creates two attractors (0 when active, w_g when idle). Miconi 2018 used Oja to avoid passive decay ("note the absence of a decay term"). Options: exempt Oja edges, or let evolution sort it out through retention = 1.0.
- **AntiHebb and Covariance:** no source studies decay with these rules specifically.
- **Interaction with the trace:** Petri already decays the eligibility trace per tick (`trace_decay`). Weight retention is a second, independent timescale: Backpropamine's E with η, plus Miconi-2018-style decay on Hebb. The trace sets credit-assignment memory; retention sets learned-weight memory. Both are heritable, and nothing ties them together.

### (4) Cheap implementation forms and their numerical caveats

**Where updates happen today in `learned.rs`:**
- Reward-modulated slots are touched every tick in `begin_tick` (trace decay) and `apply_reward` (three-factor update).
- Pure-Hebbian slots change only in `Learner::commit`, on a visit.

**Option A: eager, per tick.** Multiply the deviation by r in the existing `BackendLearning::decay` loop.
- Nearly free for modulated slots, since the loop and the cache lines are already there.
- For pure-Hebbian slots it adds a per-tick pass over slots that are otherwise idle. That is a cost proportional to plastic edges × creatures × ticks.

**Option B: lazy, closed form.** Store a `last_tick` per slot and apply `d ← d·r^{Δt}` on read or commit. This is Ba et al. eq. 3, `Σ λ^{t−τ}`, used as an equivalent unrolled form.
- Natural for pure-Hebbian slots: decay only at visit, birth capture (Lamarckian seeds) and view or telemetry reads.
- Every reader must go through one resolver. `applied_weight` is already "the only resolver", which is the right hook.
- Costs one extra integer of state per slot.

**Recommended hybrid:** eager for modulated slots in the existing loop; lazy for pure-Hebbian slots at `commit` and `applied_weight`.

**Numerical caveats:**
1. **Store the deviation `d = w − w_g`, not the absolute weight.** Absolute-form updates `w ← w_g + (w − w_g)·r` stop moving once `|d|·(1−r) < ulp(w)/2`. For |w| ≈ 1 and 1 − r = 1e−4 that leaves a permanent residual of ≈6e−4. Decaying d directly shrinks it cleanly. Every fast-weight paper (Ba's A, Miconi's Hebb, STPN's F) stores the offset separately.
2. **Subnormals.** d decays toward zero indefinitely. With r = 0.99, starting from |d| = 1, it reaches f32 subnormals (< 1.18e−38) in about 8,700 ticks. Subnormal arithmetic is slow on x86 unless flush-to-zero is enabled. Flush d to exactly 0 below a small threshold. That is also deterministic, and it makes "back at genomic" an exact state, useful for telemetry.
3. **Determinism.** `f32::powf` and `exp` go through the platform libm and are not guaranteed bit-identical across targets. With integer Δt, exponentiation by squaring uses only IEEE multiplies, so it is deterministic. Precomputing `ln r` and using `exp(Δt·ln r)` is not. Lazy and eager also round differently (r^Δt by squaring ≠ Δt successive multiplies), so pick one canonical form per slot class and one place in the tick order relative to `commit` and `apply_reward`. Decaying before the update matches H&P's "after each weight change" only up to ordering; state it in the spec.
4. **Retention granularity near 1.** f32 values just below 1 are spaced 2⁻²⁴ ≈ 5.96e−8 apart. The slowest expressible decay is therefore 1 − r ≈ 6e−8, a time constant of about 1.7e7 ticks. Exactly 1.0 must be in the heritable range and reachable by mutation, so evolution can switch decay off. Consider mutating in log(1 − r) space, as FADE parameterizes λ = exp(γ): the useful range spans decades (FADE found 0.02 vs 0.9 on one problem).
5. **Group-scaled edges.** The genomic weight of a grouped edge is `compose(own_weight, scale, pattern)`. The `Slot.genomic` doc comment in `learned.rs` says "a plastic edge is never a member", so the target is just `own_weight`. Confirm this still holds if groups' learnable scale gets decay too.

### (5) Open questions to hand back

- **Decay target for Lamarckian slots.**
  - For a seeded slot, `learned` starts at the inherited seed, not `genomic` (`learned.rs:58–73`).
  - Decaying toward `genomic` erodes Lamarckian inheritance over the creature's life. Decaying toward the birth value (seed if seeded, else genomic) preserves it.
  - That is a design decision, not resolved here. H&P, Miconi and STPN have no Lamarckian analogue. Kumar's θ₀ corresponds to the birth value.
- **Decay vs the clamp's centre** (synthesis §3): clamp the absolute weight, the deviation, or the target?
- **Oja edges:** exempt, or allow two attractors?
- **Constant or modulated retention?** Gated DeltaNet's data-dependent α and EWC/SI importance suggest retention could later depend on reward or credit, i.e. consolidation. Fixed and heritable is the cheaper first step.
- **Does decay help in Petri at all?** The ML evidence for decay comes from deep networks and is mixed (Lyle 2023/2024, Elsayed). The decisive test is Petri's own changing worlds. Is the population retention distribution pulled away from 1.0? If evolution leaves it at 1.0 everywhere, the clamp already suffices, as in Backpropamine.
- **Evolvability of a per-edge scalar.** FADE's meta-gradient tuned rates within thousands of steps. Evolution gets one fitness signal per lifetime. Group-shared retention may be needed for selection to resolve it.

---

## Claims I could NOT verify in full text

- **Schmidhuber 1992/1993 fast weight programmers:** known only through Schlag 2021 and Ba 2016's citations. Not read.
- **Peng et al. 2021, Random Feature Attention gated rule:** only as quoted in Schlag App. B, eq. 52.
- **Mamba / Mamba2 (Gu & Dao):** only as described in Gated DeltaNet ("scalar-valued decay term"). Per-channel Δ in Mamba1 was not checked.
- **Tyulmankov et al. 2022, HebbFF:** only via STPN's comparison tables.
- **Lewandowski et al. 2024, Wasserstein regularization toward init:** only via Elsayed §1.
- **Elsayed & Mahmood 2024, UPGD:** not read.
- **Najarro's ABCD refs [45, 46]:** not read.
- **Dohare et al. 2024 Nature version:** not read; I read the arXiv v3 preprint. The two texts may differ: the preprint abstract says plasticity loss "was substantially eased by L2-regularization", while §5 says L2 "on its own does not mitigate the loss of plasticity".
- **Lyle 2024, L2-Init vs LN+L2:** `LN | L2-Init` appears in the Fig. 4 legend, but its numbers are in the plot, which I could not see. Only the caption claim was verified.
- **FADE's statement that weight clipping beat L2 Init on label-permuted EMNIST "in the experiment conducted by Elsayed et al.":** Elsayed's text says only weight clipping "can keep improving". I did not see their exact EMNIST numbers, which are in the figures.
- **Hinton & Plaut 1987 figures:** missing from the text copy. The "1% per update" decay and the results are from the prose.
- **Biological analogues (early-LTP decay, consolidation, Benna & Fusi cascade models):** not researched. Out of ML scope.
