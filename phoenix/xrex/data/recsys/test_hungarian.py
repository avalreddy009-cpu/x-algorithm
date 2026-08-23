# SPDX-License-Identifier: Apache-2.0
from __future__ import annotations

import itertools
import unittest

import numpy as np

from xrex.data.recsys.hungarian import assign_slots, hungarian_maximize
from xrex.data.recsys.mmr import mmr_order
from xrex.data.recsys.rare_events import rare_event_score


def brute_max_assignment(profit: np.ndarray) -> float:
    n = profit.shape[0]
    best = -1e18
    for perm in itertools.permutations(range(n)):
        total = sum(profit[i, perm[i]] for i in range(n))
        best = max(best, total)
    return best


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
        profit = np.array(
            [
                [5.0, 0.1],
                [1.0, 9.0],
            ]
        )
        pairs = assign_slots(profit)
        self.assertEqual(pairs[0], (0, 0))
        self.assertEqual(pairs[1], (1, 1))

    def test_matches_brute_force_on_random_5x5(self):
        rng = np.random.default_rng(42)
        for _ in range(8):
            profit = rng.random((5, 5))
            assign = hungarian_maximize(profit)
            got = float(sum(profit[i, assign[i]] for i in range(5)))
            self.assertAlmostEqual(got, brute_max_assignment(profit), places=6)

    def test_feed_puts_rare_event_in_breaking_slot(self):
        """Slot 0 = friends, slot 1 = rare event, slot 2 = general."""
        posts = [
            {"name": "friend", "score": 2.5, "in_network": True, "rare": 0.0},
            {"name": "viral_oon", "score": 1.8, "in_network": False, "rare": 0.0},
            {"name": "breaking", "score": 0.9, "in_network": False, "rare": 1.2},
        ]
        k = 3
        profit = np.zeros((3, k))
        for i, p in enumerate(posts):
            for j in range(k):
                profit[i, j] = p["score"] * (0.92**j)
                if p["in_network"] and j == 0:
                    profit[i, j] += 0.22 * p["score"]
                if j == 1:
                    profit[i, j] += 4.0 * max(p["score"], 0.05) * p["rare"]
        pairs = assign_slots(profit)
        slot_of = {slot: posts[i]["name"] for i, slot in pairs}
        self.assertEqual(slot_of[1], "breaking")
        self.assertEqual(slot_of[0], "friend")


class FeedAlgorithmsTogether(unittest.TestCase):
    def test_mmr_then_hungarian_still_assigns_all_slots(self):
        scores = np.array([5.0, 4.9, 2.0])
        sim = np.array(
            [
                [1.0, 0.95, 0.0],
                [0.95, 1.0, 0.0],
                [0.0, 0.0, 1.0],
            ]
        )
        order = mmr_order(scores, sim, k=3, lam=0.3)
        self.assertEqual(len(order), 3)
        profit = np.eye(3) * scores[order]
        pairs = assign_slots(profit)
        self.assertEqual(len(pairs), 3)

    def test_only_one_in_five_posts_is_a_rare_event(self):
        catalog = [
            rare_event_score(
                age_seconds=600, views=200_000, replies=40, reposts=20, quotes=5, favs=8000
            ),
            rare_event_score(
                age_seconds=400, views=200, replies=30, reposts=20, quotes=8, favs=25
            ),
            rare_event_score(
                age_seconds=200, views=500, replies=80, reposts=40, quotes=10, favs=30, is_quote=True
            ),
            rare_event_score(
                age_seconds=10_000, views=300, replies=40, reposts=20, quotes=8, favs=20
            ),
            rare_event_score(
                age_seconds=100, views=20, replies=8, reposts=4, quotes=2, favs=3
            ),
        ]
        rare = [s for s in catalog if s >= 0.7]
        self.assertEqual(len(rare), 1)


if __name__ == "__main__":
    unittest.main()
