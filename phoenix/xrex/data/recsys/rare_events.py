# SPDX-License-Identifier: Apache-2.0
"""Rare real-event score. Age is passed in seconds (no Snowflake needed)."""

from __future__ import annotations


def rare_event_score(
    *,
    age_seconds: float,
    views: float,
    replies: float,
    reposts: float,
    quotes: float,
    favs: float,
    is_reply: bool = False,
    is_repost: bool = False,
    is_quote: bool = False,
    max_age_seconds: float = 5400.0,
    min_views: float = 80.0,
) -> float:
    if is_reply or is_repost or is_quote:
        return 0.0
    if age_seconds > max_age_seconds or views < min_views:
        return 0.0
    talk = replies + 1.5 * reposts + quotes
    talk_rate = talk / views
    fav_rate = favs / views
    if replies > 40 and favs > 0 and replies / (favs + 1.0) > 12.0:
        return 0.0
    freshness = 1.0 - min(max(age_seconds / max_age_seconds, 0.0), 1.0)
    burst = min(talk_rate / 0.12, 1.5) * (0.4 + 0.6 * min(fav_rate / 0.04, 1.0))
    score = freshness * burst
    return 0.0 if score < 0.7 else min(score, 1.5)
