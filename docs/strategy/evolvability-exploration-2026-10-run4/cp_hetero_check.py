"""Check the run 4 rule 4 bound: one-sided 97.5 % Clopper-Pearson bounds with
the conventions CP- = 0 for c <= 1 and CP+ = 1 for b >= n - 1 keep their
coverage for the mean of independent heterogeneous Bernoulli trials.

1. Hoeffding (1956) Theorem 4 conditions on every rejection region, for every
   n in 16..48 over a fine grid of the mean p.
2. Exact Poisson-binomial miss probabilities on adversarial two-point and
   three-point designs (and Codex's counterexample)."""
import itertools
import math
import sys

from cpmath import beta_ppf_lower, beta_ppf_upper

ALPHA = 0.025


def cp_upper(b, n):
    if b >= n - 1:
        return 1.0
    return beta_ppf_upper(b, n, ALPHA)


def cp_lower(c, n):
    if c <= 1:
        return 0.0
    return beta_ppf_lower(c, n, ALPHA)


def poisson_binomial(ps):
    dist = [1.0]
    for p in ps:
        nxt = [0.0] * (len(dist) + 1)
        for k, v in enumerate(dist):
            nxt[k] += v * (1 - p)
            nxt[k + 1] += v * p
        dist = nxt
    return dist


bad = 0
for n in range(16, 49):
    up = [cp_upper(b, n) for b in range(n + 1)]
    lo = [cp_lower(c, n) for c in range(n + 1)]
    for i in range(1, 20000):
        p = i / 20000
        miss_up = [b for b in range(n + 1) if up[b] < p]
        if miss_up and max(miss_up) > n * p - 1 + 1e-12:
            bad += 1
            print("upper condition fails", n, p, max(miss_up))
        miss_lo = [c for c in range(n + 1) if lo[c] > p]
        if miss_lo and min(miss_lo) < n * p + 1 - 1e-12:
            bad += 1
            print("lower condition fails", n, p, min(miss_lo))
print("hoeffding-condition failures:", bad)

worst_up = worst_lo = 0.0
for n in (16, 24, 32, 40, 48):
    up = [cp_upper(b, n) for b in range(n + 1)]
    lo = [cp_lower(c, n) for c in range(n + 1)]
    grid = [0.0, 1e-4, 1e-3, 0.005, 0.01, 0.02, 0.05, 0.1, 0.15, 0.2, 0.25, 0.3,
            0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 0.95, 0.99, 0.999, 1.0]
    for k in (1, 2, 3, n // 4, n // 2, n - 1):
        for a, z in itertools.product(grid, grid):
            ps = [a] * k + [z] * (n - k)
            pbar = sum(ps) / n
            dist = poisson_binomial(ps)
            mu = sum(d for b, d in enumerate(dist) if up[b] < pbar)
            ml = sum(d for c, d in enumerate(dist) if lo[c] > pbar)
            worst_up, worst_lo = max(worst_up, mu), max(worst_lo, ml)
    for k in (1, 2, 3, 8):
        for a, z, w in itertools.product(grid[::3], grid[::3], grid[::3]):
            ps = [a] * k + [z] * k + [w] * (n - 2 * k)
            pbar = sum(ps) / n
            dist = poisson_binomial(ps)
            mu = sum(d for b, d in enumerate(dist) if up[b] < pbar)
            ml = sum(d for c, d in enumerate(dist) if lo[c] > pbar)
            worst_up, worst_lo = max(worst_up, mu), max(worst_lo, ml)
print(f"worst exact miss: upper {worst_up:.5f}, lower {worst_lo:.5f} (target <= {ALPHA})")

ps = [0.0] * 47 + [0.0252]
dist = poisson_binomial(ps)
lo = [cp_lower(c, 48) for c in range(49)]
print("codex counterexample miss:", sum(d for c, d in enumerate(dist) if lo[c] > sum(ps) / 48))
sys.exit(1 if bad or worst_up > ALPHA or worst_lo > ALPHA else 0)
