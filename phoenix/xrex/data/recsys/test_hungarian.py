# SPDX-License-Identifier: Apache-2.0
from __future__ import annotations

import unittest

import numpy as np

from xrex.data.recsys.hungarian import assign_slots, hungarian_maximize


class HungarianTest(unittest.TestCase):
    def test_two_by_two_max(self):
        profit = np.array([[1.0, 2.0], [3.0, 0.0]])
        assign = hungarian_maximize(profit)
        self.assertEqual(int(assign[0]), 1)
        self.assertEqual(int(assign[1]), 0)

    def test_diagonal(self):
        profit = np.eye(4) * 10.0 + 0.1
        assign = hungarian_maximize(profit)
        np.testing.assert_array_equal(assign, np.arange(4))

    def test_slot_order(self):
        # Post 1 is a rare event: huge profit only in slot 1.
        profit = np.array(
            [
                [5.0, 0.1],
                [1.0, 9.0],
            ]
        )
        pairs = assign_slots(profit)
        self.assertEqual(pairs[0], (0, 0))
        self.assertEqual(pairs[1], (1, 1))


if __name__ == "__main__":
    unittest.main()
