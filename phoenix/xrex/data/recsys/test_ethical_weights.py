# SPDX-License-Identifier: Apache-2.0
from __future__ import annotations

import unittest

import numpy as np

from xrex.data.recsys.ethical_weights import (
    combine_sample_weights,
    delayed_positive_weight,
    history_recency_weights,
    impression_log1p_weight,
    viral_history_dropout_mask,
)


class EthicalWeightsTest(unittest.TestCase):
    def test_high_impressions_get_smaller_weight(self):
        w = impression_log1p_weight(np.array([0.0, 10.0, 1_000_000.0]))
        self.assertAlmostEqual(float(w[0]), 1.0, places=5)
        self.assertGreater(float(w[1]), float(w[2]))
        self.assertGreater(float(w[2]), 0.05)

    def test_early_likes_damped_reports_not(self):
        delayed = np.array([False, True, False])
        positive = np.array([True, True, False])
        w = delayed_positive_weight(delayed, positive, early_positive_scale=0.25)
        np.testing.assert_array_almost_equal(w, np.array([0.25, 1.0, 1.0], dtype=np.float32))

    def test_recency_decay(self):
        w = history_recency_weights(np.array([0.0, 3 * 24 * 3600, 30 * 24 * 3600.0]))
        self.assertAlmostEqual(float(w[0]), 1.0, places=5)
        self.assertAlmostEqual(float(w[1]), 0.5, places=2)
        self.assertLess(float(w[2]), float(w[1]))

    def test_viral_dropout_drops_hottest(self):
        counts = np.array([1.0, 5.0, 100.0, 2.0])
        mask = viral_history_dropout_mask(counts, drop_frac=0.25, rng=np.random.default_rng(0))
        self.assertFalse(mask[2])
        self.assertGreaterEqual(mask.sum(), 2)

    def test_combine_multiplies(self):
        a = np.array([1.0, 0.5], dtype=np.float32)
        b = np.array([0.5, 2.0], dtype=np.float32)
        np.testing.assert_array_almost_equal(combine_sample_weights(a, b), np.array([0.5, 1.0]))


if __name__ == "__main__":
    unittest.main()
