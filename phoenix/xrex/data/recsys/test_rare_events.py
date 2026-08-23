# SPDX-License-Identifier: Apache-2.0
from __future__ import annotations

import unittest

from xrex.data.recsys.rare_events import rare_event_score


class RareEventTest(unittest.TestCase):
    def test_ordinary_viral_clip_is_not_an_event(self):
        s = rare_event_score(
            age_seconds=600,
            views=200_000,
            replies=40,
            reposts=20,
            quotes=5,
            favs=8000,
        )
        self.assertEqual(s, 0.0)

    def test_fresh_high_talk_rate_is_rare(self):
        s = rare_event_score(
            age_seconds=400,
            views=200,
            replies=30,
            reposts=20,
            quotes=8,
            favs=25,
        )
        self.assertGreaterEqual(s, 0.7)

    def test_quote_dunk_is_never_an_event(self):
        s = rare_event_score(
            age_seconds=200,
            views=500,
            replies=80,
            reposts=40,
            quotes=10,
            favs=30,
            is_quote=True,
        )
        self.assertEqual(s, 0.0)


if __name__ == "__main__":
    unittest.main()
