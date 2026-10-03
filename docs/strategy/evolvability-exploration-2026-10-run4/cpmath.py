"""Exact one-sided Clopper-Pearson bounds by bisection on the binomial CDF."""
from math import comb


def cdf(k, n, p):
    return sum(comb(n, i) * p**i * (1 - p) ** (n - i) for i in range(k + 1))


def beta_ppf_upper(b, n, alpha):
    # largest p with P(Bin(n, p) <= b) >= alpha
    lo, hi = 0.0, 1.0
    for _ in range(80):
        mid = (lo + hi) / 2
        if cdf(b, n, mid) > alpha:
            lo = mid
        else:
            hi = mid
    return (lo + hi) / 2


def beta_ppf_lower(c, n, alpha):
    # smallest p with P(Bin(n, p) >= c) >= alpha
    lo, hi = 0.0, 1.0
    for _ in range(80):
        mid = (lo + hi) / 2
        if 1 - cdf(c - 1, n, mid) < alpha:
            lo = mid
        else:
            hi = mid
    return (lo + hi) / 2
