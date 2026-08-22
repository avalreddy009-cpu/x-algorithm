#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
"""Print a For You weighted-sum score so weights are not misread as counts.

Example:
  python phoenix/reference/score_explainer.py \\
    --favorite 0.2 --reply 0.05 --report 0.001 --dwell-time 12
"""

from __future__ import annotations

import argparse

# Production-like defaults from home-mixer/params/param.rs (FavoriteWeight, …).
DEFAULT_WEIGHTS = {
    "favorite": 0.5,
    "reply": 5.0,
    "retweet": 1.0,
    "quote": 1.0,
    "share": 1.0,
    "dwell": 0.5,
    "dwell_time": 0.03,
    "click": 0.3,
    "follow_author": 4.0,
    "not_interested": -7.0,
    "mute_author": -10.0,
    "block_author": -20.0,
    "report": -234.0,
    "not_dwelled": -0.5,
}

REPLY_BASE = 5.0
BIDIRECTIONAL_REPLY = 15.0  # param default for BidirectionalFollowReplyWeightBoost varies; CLI flag


def parse_args() -> argparse.Namespace:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--favorite", type=float, default=0.0, help="P(like)")
    p.add_argument("--reply", type=float, default=0.0, help="P(reply)")
    p.add_argument("--retweet", type=float, default=0.0)
    p.add_argument("--quote", type=float, default=0.0)
    p.add_argument("--share", type=float, default=0.0)
    p.add_argument("--dwell", type=float, default=0.0, help="P(dwell)")
    p.add_argument("--dwell-time", type=float, default=0.0, dest="dwell_time")
    p.add_argument("--click", type=float, default=0.0)
    p.add_argument("--follow-author", type=float, default=0.0, dest="follow_author")
    p.add_argument("--not-interested", type=float, default=0.0, dest="not_interested")
    p.add_argument("--mute-author", type=float, default=0.0, dest="mute_author")
    p.add_argument("--block-author", type=float, default=0.0, dest="block_author")
    p.add_argument("--report", type=float, default=0.0)
    p.add_argument("--not-dwelled", type=float, default=0.0, dest="not_dwelled")
    p.add_argument("--oon", action="store_true", help="apply out-of-network discount 0.75")
    p.add_argument("--oon-factor", type=float, default=0.75)
    p.add_argument("--mutual", action="store_true", help="use reply weight 5+boost (20 if --reply-boost 15)")
    p.add_argument("--reply-boost", type=float, default=15.0)
    p.add_argument("--recency", type=float, default=1.0, help="extra multiplier (freshness)")
    p.add_argument("--qei", type=float, default=1.0, help="impression-tax multiplier")
    return p.parse_args()


def main() -> None:
    args = parse_args()
    reply_w = REPLY_BASE + (args.reply_boost if args.mutual else 0.0)
    weights = dict(DEFAULT_WEIGHTS)
    weights["reply"] = reply_w

    probs = {
        "favorite": args.favorite,
        "reply": args.reply,
        "retweet": args.retweet,
        "quote": args.quote,
        "share": args.share,
        "dwell": args.dwell,
        "dwell_time": args.dwell_time,
        "click": args.click,
        "follow_author": args.follow_author,
        "not_interested": args.not_interested,
        "mute_author": args.mute_author,
        "block_author": args.block_author,
        "report": args.report,
        "not_dwelled": args.not_dwelled,
    }

    terms = []
    total = 0.0
    for name, p_hat in probs.items():
        w = weights[name]
        t = w * p_hat
        total += t
        if abs(t) > 1e-12:
            terms.append((name, w, p_hat, t))

    terms.sort(key=lambda x: abs(x[3]), reverse=True)
    oon = args.oon_factor if args.oon else 1.0
    final = total * oon * args.recency * args.qei

    print("For You score explainer (weights × predicted probabilities, not raw counts)")
    print(f"{'head':<18} {'weight':>10} {'P(action)':>12} {'term':>12}")
    for name, w, p_hat, t in terms:
        print(f"{name:<18} {w:10.3f} {p_hat:12.4f} {t:12.4f}")
    print(f"{'sum':<18} {'':>10} {'':>12} {total:12.4f}")
    print(f"multipliers: oon={oon:.3f} recency={args.recency:.3f} qei={args.qei:.3f}")
    print(f"final={final:.4f}")
    print()
    print(
        "A large |report| weight does not mean one report cancels hundreds of likes; "
        "it scales P(you would report), which is usually tiny."
    )


if __name__ == "__main__":
    main()
