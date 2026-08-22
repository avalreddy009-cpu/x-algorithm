# SPDX-License-Identifier: Apache-2.0
"""Maximal Marginal Relevance (Carbonell & Goldstein, 1998).

Used in search and feeds for 25+ years. Greedy, O(k·n), no extra model.
Does not replace a ranker — it only spreads a already-scored list.
"""

from __future__ import annotations

import numpy as np


def mmr_order(scores: np.ndarray, sim: np.ndarray, k: int, lam: float = 0.7) -> np.ndarray:
    """Return indices of a size-k MMR permutation.

    scores: [n] relevance
    sim: [n, n] pairwise similarity in [0, 1], sim[i,i] ignored
    """
    scores = np.asarray(scores, dtype=np.float64)
    sim = np.asarray(sim, dtype=np.float64)
    n = scores.shape[0]
    k = min(int(k), n)
    if k <= 0:
        return np.array([], dtype=np.int64)
    lam = float(np.clip(lam, 0.0, 1.0))
    selected: list[int] = []
    remaining = set(range(n))
    first = int(np.argmax(scores))
    selected.append(first)
    remaining.remove(first)
    while len(selected) < k and remaining:
        best_i = None
        best = -1e18
        for i in remaining:
            max_sim = max(sim[i, j] for j in selected)
            value = lam * scores[i] - (1.0 - lam) * max_sim
            if value > best:
                best = value
                best_i = i
        assert best_i is not None
        selected.append(best_i)
        remaining.remove(best_i)
    return np.asarray(selected, dtype=np.int64)
