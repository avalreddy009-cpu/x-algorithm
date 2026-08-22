# SPDX-License-Identifier: Apache-2.0
"""Sample weights that damp viral loops without hiding negatives.

These helpers are used by Phoenix training. They do not invent engagement;
they reweight labels the model already sees.

- impression_log1p_weight: down-weight posts that already got huge reach
  (smoother than 1/count).
- delayed_positive_weight: public positives (like/repost) count less until
  delayed feedback arrives; reports/blocks stay at 1.0.
- history_recency_weights: exponential decay so a binge week does not own
  the user tower forever.
- viral_history_dropout: randomly drop the most-impressed history rows
  (explore in embedding space).
"""

from __future__ import annotations

import math

import numpy as np


def impression_log1p_weight(
    impression_count: np.ndarray,
    *,
    gamma: float = 1.0,
    floor: float = 0.05,
) -> np.ndarray:
    counts = np.asarray(impression_count, dtype=np.float64)
    counts = np.maximum(counts, 0.0)
    weights = 1.0 / (1.0 + gamma * np.log1p(counts))
    return np.clip(weights, floor, 1.0).astype(np.float32)


def delayed_positive_weight(
    is_delayed_feedback: np.ndarray,
    is_public_positive: np.ndarray,
    *,
    early_positive_scale: float = 0.25,
) -> np.ndarray:
    """Positives that are *not* delayed get scaled down; negatives stay 1."""
    delayed = np.asarray(is_delayed_feedback, dtype=bool)
    positive = np.asarray(is_public_positive, dtype=bool)
    weights = np.ones(delayed.shape, dtype=np.float32)
    early_pos = positive & ~delayed
    weights[early_pos] = np.float32(early_positive_scale)
    return weights


def history_recency_weights(
    ages_seconds: np.ndarray,
    *,
    half_life_seconds: float = 3 * 24 * 3600,
) -> np.ndarray:
    ages = np.asarray(ages_seconds, dtype=np.float64)
    if half_life_seconds <= 0:
        return np.ones_like(ages, dtype=np.float32)
    decay = np.exp(-np.maximum(ages, 0.0) / half_life_seconds * math.log(2.0))
    return decay.astype(np.float32)


def viral_history_dropout_mask(
    impression_count: np.ndarray,
    *,
    drop_frac: float = 0.1,
    rng: np.random.Generator | None = None,
) -> np.ndarray:
    """Keep-mask: drop the top `drop_frac` most-impressed history rows at random among that tail."""
    counts = np.asarray(impression_count, dtype=np.float64)
    n = counts.size
    keep = np.ones(n, dtype=bool)
    if n == 0 or drop_frac <= 0:
        return keep
    k = max(1, int(math.ceil(n * drop_frac)))
    if k >= n:
        k = n - 1 if n > 1 else 0
    if k <= 0:
        return keep
    order = np.argsort(-counts)
    tail = order[:k]
    rng = rng or np.random.default_rng(0)
    # Always drop the hottest row; randomly drop the rest of the tail.
    keep[tail[0]] = False
    if k > 1:
        extra = rng.random(k - 1) < 0.5
        keep[tail[1:][extra]] = False
    return keep


def combine_sample_weights(*weights: np.ndarray) -> np.ndarray:
    out = np.ones_like(weights[0], dtype=np.float32)
    for w in weights:
        out = out * np.asarray(w, dtype=np.float32)
    return np.clip(out, 0.0, None)
