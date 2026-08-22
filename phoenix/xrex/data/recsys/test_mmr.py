# SPDX-License-Identifier: Apache-2.0
from __future__ import annotations

import unittest

import numpy as np

from xrex.data.recsys.mmr import mmr_order


class MmrTest(unittest.TestCase):
    def test_picks_diverse_second_item(self):
        scores = np.array([10.0, 9.0, 8.0])
        sim = np.array(
            [
                [1.0, 0.99, 0.0],
                [0.99, 1.0, 0.0],
                [0.0, 0.0, 1.0],
            ]
        )
        order = mmr_order(scores, sim, k=2, lam=0.2)
        np.testing.assert_array_equal(order, np.array([0, 2]))


if __name__ == "__main__":
    unittest.main()
