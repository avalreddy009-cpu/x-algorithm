# Changes by Mr Reddy

All For You work in this fork is by **Mr Reddy**.

## What a human would actually ship

Phoenix (two-tower retrieval + isolated transformer ranker) is already in the
same family as YouTube / TikTok ranking. Replacing it with an LLM-on-every-
request ranker would **not** work at For You latency. What *does* work, and
is now in the code:

| Algorithm | Human verdict | Where |
| --- | --- | --- |
| Multi-action weighted sum | Keep — already production | `ranking_scorer.rs` |
| Friends-first OON discount | Works (For You “too many strangers”) | `param.rs` + `effective_oon_weight` |
| Recency + QEI + reply-farm + reciprocal | Works as light multipliers | `value_adjustments.rs` |
| Empirical-Bayes probability shrink | Works when view counts exist; no-op if n=0 | `empirical_bayes_shrink` |
| MMR slate rerank (1998) | Works; cheap diversity if VMRanker is down | `slate_algorithms.rs`, `mmr.py` |
| In-network quota in top K | Works as a product floor | `apply_in_network_quota` |
| Language lock | Works; default off (too sharp) | `language_lock_filter.rs` |
| Impression log1p training weights | Works; milder than 1/count | `ethical_weights.py` |
| LLM listwise ranking on the request path | **Would not work** (QPS, cache, gaming) | not built |
| Ideology / surprise logit | **Would not work** as a ranker head | not built |
| Hard lottery slots | **Would not work** (relevance collapse) | not built |

## File list

- `docs/FEATURES.md`, `docs/DESIGN_REVIEW.md`, `docs/HABITS.md`, this file
- `home-mixer/scorers/value_adjustments.rs`, `ranking_scorer.rs`, `params/param.rs`
- `home-mixer/filters/language_lock_filter.rs`
- `home-mixer/selectors/slate_algorithms.rs`, `top_k_score_selector.rs`
- `phoenix/xrex/data/recsys/ethical_weights.py`, `mmr.py`
- `phoenix/reference/score_explainer.py`, `dump_gen.py`
- `phoenix/xrex/models/recsys_model.py`, `xrex/configs/xrecsys.py`
