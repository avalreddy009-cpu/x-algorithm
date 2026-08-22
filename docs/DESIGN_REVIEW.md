# For You design review: new signals, parameters, and architecture

Reviewer stance: senior recommender-systems research, grounded in this tree
(`phoenix/` two-tower retrieval + isolated ranking transformer, `home-mixer`
weighted-sum ranker, author cold-start / Thompson sampling, `vm-ranker` DPP).

The current stack is a **predicted-action utility ranker** with hard filters
and a few post-hoc multipliers. That is the right skeleton. Most failures
are not “need a new model class.” They are **objective misspecification**,
**retrieval coverage**, **feedback delay**, and **rich-get-richer through
impressions feeding the two-tower index**.

---

## How the live pieces constrain ideas

- Retrieval user tower is **history + coarse profile**, not a learned user-id
  embedding. Stale or bait-heavy history *is* the user vector.
- Ranking candidates **cannot attend to each other**. Slate effects belong in
  `RankingScorer` multipliers or `VMRanker` DPP, not inside Phoenix logits.
- Reply weight (~5) >> like (~0.5); report is a large negative on
  **P(report | you)**, not on global report counts.
- Creator equity already has **impression-threshold slot targeting** and
  optional **Thompson sampling** (`author_cold_start.rs`). Lottery is not new.
- `PostUnexploredWeight` and OON discount already exist; they are blunt.

---

## Angle 1 — Candidate generation

### Variation 1.1 — Quota-mixed retrieval (keep)

**Change.** Replace “one two-tower top-K plus SimClusters plus Thunder” with
**source quotas before ranking**: e.g. 50% follow-graph (Thunder), 25%
main two-tower, 10% **fresh SID prefix** (posts whose semantic-id prefix is
rare in the viewer history), 10% **follow-of-follow / co-engagement graph**,
5% **low-impression original posts** (same eligibility as cold-start:
not a reply/repost, follower cap, age cap).

**Why.** Two-tower ANN on a popularity-skewed index **cannot retrieve** what
it never embedded near the user. Phoenix ranking never sees those posts.
Echo chambers and new-creator failure start at retrieval, not at the
weighted sum.

**Tradeoff.** More hydrator/VF load; quota items will lose on P(like) and
look like “random junk” if ranking is unchanged. Gameability: farmers will
try to land in the exploration index (copy SID prefixes).

**Measure.** Retrieval recall@K of held-out future follows and of posts the
user later engaged after 7d (currently unretrieved). A/B: unique authors
in top 50, share of posts with impressions below cold-start threshold,
next-7d follow rate, D1/D7 retention. Guardrail: P95 latency, VF drop rate.

### Variation 1.2 — Second “explore” tower on a low-impression corpus (keep as infra)

**Change.** Train a **separate candidate tower** on posts below an impression
quantile; retrieve with a slightly noisier user vector (dropout on recent
viral history). Merge with a small K (50–100) into Home Mixer.

**Why.** Mixing explore items into the **same** index lets viral posts
dominate the embedding space. A second index is how you stop that without
destroying exploitation quality.

**Tradeoff.** 2× candidate-tower serving memory; index freshness jobs.
Weaker than quotas if you do not also change ranking (explore items still
lose the sort).

**Measure.** Same as 1.1 plus **explore-index hit rate → ranked top 20**.
If hits never survive ranking, the tower is wasted — that is a useful
negative result.

### Variation 1.3 — Real-time graph walk as a fourth source (drop)

**Change.** Online personalized PageRank / follow-of-follow expansion per
request.

**Why it looks good.** Classic in-network expansion; good for cold OON.

**Why weaker.** Thunder already covers in-network recency. User-cred already
runs PageRank **offline**. Online walks at For You QPS are expensive, leak
graph structure to latency, and **amplify dense spam graphs**. SimClusters
is the cheaper community proxy.

**Rule-out.** Do not add a live walk. If graph expansion is needed, **batch
follow-of-follow candidate lists** into the same quota mixer as 1.1.

---

## Angle 2 — Ranking objective

### Variation 2.1 — Reciprocal conversation value (keep)

**Change.** New Phoenix head: **P(author replies to viewer within 24h | viewer
replies)** or a simpler logged label `reciprocal_reply`. In `RankingScorer`,
replace part of `ReplyWeight` with
`w_reply * P(reply) * (1 + β * P(reciprocal | reply))`, with a **higher β
for in-network / bidirectional follows** (extends the existing bidirectional
reply boost, which only scales P(reply), not conversation quality).

**Why.** Reply=5.0 currently pays **reply-farming and dunk-quote threads**
the same as a conversation. Reciprocity is the missing causal bit: the
author treated the viewer as a person.

**Tradeoff.** Sparse labels; slow authors look “bad”; celebrities cannot
reciprocate. Must **gate on in-network or similar-size authors**, else you
punish large news accounts. Compute: one extra head, tiny.

**Measure.** Offline: calibration of reciprocal head; fraction of ranked
replies that get an author reply. A/B: replies **from authors back to
viewers**, unique conversation depth ≥2, mute/block rate on reply-heavy
posts, session length. Guardrail: total replies must not be the north star.

### Variation 2.2 — Session surplus / regret (keep, tighten existing)

**Change.** You already have `cont_active_secs_5m_residual_norm` and
click-dwell low-fav-rate penalty. Make **dwell-regret the primary
attention term**: predicted dwell minus predicted dwell of a **session-level
baseline** (what this viewer usually does at this hour). Downweight posts
with high P(click) and low residual dwell (rage-scroll).

**Why.** Raw dwell rewards outrage and autoplay. Residual dwell is closer to
**time well spent**. Click-dwell-without-like is a start; it is still
post-level, not session-level.

**Tradeoff.** Night-owl and news users have high dwell on “bad” content they
want. Residual can encode boredom as quality. Gameability: longer videos.

**Measure.** A/B: **active seconds per session** vs **sessions per DAU**
(do not pick a side). Survey “worth my time” on 1% of sessions. Negative:
not-interested, follow-graph density collapse.

### Variation 2.3 — Cross-ideology / unexpectedness bonus (drop for ranking)

**Change.** Boost posts whose SID prefix is far from the viewer’s history
centroid, or opposite a coarse ideology embedding.

**Why it looks good.** Echo chambers.

**Why weaker.** Ideology inference is noisy, contested, and **looks like
the product is pushing a side**. Unexpectedness ≠ quality; it often means
spam and out-group dunking. Exploration **quotas** (1.1) plus **topic
diversity in DPP** get the diversity without a political head.

**Rule-out as a ranking weight.** Keep surprise as a **retrieval quota**,
not a logit bonus.

---

## Angle 3 — Anti-gaming / robustness

### Variation 3.1 — Graph-normalized predicted engagement (keep)

**Change.** At training and/or scoring, divide (or residualize) positive
heads by **expected engagement given author user-cred, follower count, and
age**. Phoenix predicts **excess** P(like) over the author’s baseline, not
raw P(like).

**Why.** Coordinated rings raise **global** likes; they should not raise
**your** P(like) as much if your history is unlike the ring. Residualizing
against author baseline attacks **inauthentic inflation** that still leaks
into embeddings via SID/author hashes.

**Tradeoff.** Punishes genuinely popular good posts. Needs careful
calibration by follower bucket. Fairness: non-English and Global South
authors have different baselines — **segment the baseline**.

**Measure.** Offline: AUC on a **held-out spam-label** set (Agatha/BDSM)
while holding organic engagement AUC. A/B: impressions to accounts later
actioned for inauthentic behavior (should fall); organic reply quality.

### Variation 3.2 — Reply-farm / dunk-thread penalty (keep)

**Change.** Candidate features already include tweet type. Add **thread
shape**: unique repliers / replies, fraction of replies from accounts that
only reply not post, burstiness, quote-dunk (high quote P, low dwell
residual). Multiply `ReplyWeight` contribution by a **quality gate**
(0.2–1.0). This is a scorer param, not a VF drop, so followers still see
the fight if they follow the people.

**Why.** The objective **explicitly pays for replies**. Adversaries will
farm that head. VF already drops some OON spam at high recall; **in-network
dunk piles** still rank.

**Tradeoff.** Legitimate heated news threads look like farms. Over-penalize
sports and politics. Keep as a **soft multiplier**, never a drop.

**Measure.** Human rater: “reply bait” on sampled top-10. A/B: quote/reply
ratio, unique authors in first 20, report rate on conversation modules.

### Variation 3.3 — Impression-weighted / delayed positive labels (keep as training)

**Change.** In Phoenix training, **down-weight positives by log(impressions)**
and **delay like/repost labels** until T+6h so early ring bursts count less
than later organic. Reports/blocks stay immediate.

**Why.** Viral loops: early inauthentic engagement → retrieval index → more
impressions → more genuine-looking labels.

**Tradeoff.** Slower trend surfacing (World Cup problem they already hit with
bidirectional boost). Need a **parallel recency retrieval path** (1.1 fresh
quota) so news is not only the delayed ranker.

**Measure.** Time-to-surface for verified breaking events vs impressions to
later-suspended spam. Offline: label delay ablation on replay logs.

---

## Angle 4 — Feedback loops

### Variation 4.1 — Durable negative memory in the user tower (keep)

**Change.** Today “not interested,” mute, and block are **ranking heads** and
**filters**. They barely change **retrieval** unless they appear as history
actions. Add a compact **negative SID/author bag** (hashed, TTL 14–90d) as
an extra user-tower token, and a ranking feature `neg_sim(candidate, bag)`.

**Why.** Two-tower with no user-id embedding **forgets** a mute as soon as
it falls out of the action sequence. Explore/exploit then re-retrieves the
same cluster. This is the actual echo+harassment loop.

**Tradeoff.** Over-mute → empty explore. People mute during a mood. TTL and
decay are required. Privacy: bag is sensitive.

**Measure.** Re-impression rate of muted authors/SID prefixes at 7d (must
fall). A/B: mute/block **rate** (may fall if the system listens). Guardrail:
category collapse (all news muted because one outlet was).

### Variation 4.2 — Recency-weighted history + explicit explore dropout (keep)

**Change.** Retrieval already uses history. Apply **exponential recency
decay** on history tokens and **randomly drop the most viral 10% of history**
when building the user vector (train and serve, serve at lower rate). That
is ε-explore in embedding space.

**Why.** Stale embeddings: a week of World Cup or an outrage cycle **owns**
the user vector. Candidate isolation does not fix that; the history is the
context.

**Tradeoff.** Worse short-term P(click). Must be small (5–15% dropout).

**Measure.** Embedding cosine between day-0 and day-7 user vectors (want
**less** stickiness after a topical binge). A/B: topic entropy of impressions,
D7 retention after a binge week.

### Variation 4.3 — IPS / counterfactual logging for not-interested (drop as v1)

**Change.** Full inverse-propensity training on logged not-interested.

**Why it looks good.** Gold-standard for bias.

**Why weaker now.** Propensity of For You is a **product of retrieval +
filters + weighted sum + DPP + ads blender**. Getting p(show) wrong
**amplifies** noise. Need propensity scores in side effects first
(`served_candidates_kafka`); then IPS. Not v1.

**Rule-out for now.** Instrument **propensity logging**; do not train on IPS
until p(show) is calibrated.

---

## Angle 5 — Creator equity

### Variation 5.1 — Quality-at-equal-impressions (QEI) rerank (keep)

**Change.** After Phoenix scores, for authors above the cold-start impression
threshold, apply a **diminishing returns tax**:
`score *= (1 / (1 + γ log(1 + impressions)))` with γ small, **or** replace
the current “lift to slot” with **expected utility per impression**.
Cold-start slotting already lifts **low-impression original posts** into
`[ColdStartSlotMin, ColdStartSlotMax]`. QEI is the dual: **stop overserving
saturated authors** without a lottery.

**Why.** Slot boost helps the left tail; the right tail still eats rank
because P(like) is easier to predict for famous people. Rich-get-richer is
**predictability**, not just retrieval.

**Tradeoff.** Users **want** famous people. Too much tax feels like a worse
product. Gameability: authors delete/repost to reset impressions (need
**author-level 7d impressions**, not post-level only).

**Measure.** Gini of impressions among eligible authors; relevance: like/dwell
on top 10. A/B: follows of accounts with <10k followers, without dropping
DAU time spent more than X%.

### Variation 5.2 — Improve existing Thompson sampling cold-start (keep)

**Change.** `EnableColdStartThompsonSampling` with Beta(α0, β0) already
exists. Use **post-level reward = residual dwell + follow − report**, not
binary click. Cap **per-viewer** cold-start injections (1–2 per request).

**Why.** Slot targeting without a bandit **burns impressions on duds**.
Thompson sampling is the right family; the reward is probably too clicky.

**Tradeoff.** Variance for small creators; unfair if reward is global
engagement. Per-viewer cap protects UX.

**Measure.** Regret vs always-slot: downstream follows per cold-start
impression. A/B vs current slot-only.

### Variation 5.3 — Hard reserved lottery slots in top 10 (drop)

**Change.** Positions 4 and 8 always a random eligible small creator.

**Why it looks good.** Equity.

**Why weaker.** Destroyed relevance, screenshot-able “why is this here,”
incentives to make **many** tiny alts. You already have **soft** slot
targeting. Lottery is a worse VMRanker.

**Rule-out.** QEI + better TS dominate reserved slots.

---

## 1. Table of candidate ideas

| Angle | Idea | Mechanism changed | Expected effect | Key risk |
| --- | --- | --- | --- | --- |
| Candidate gen | Quota-mixed retrieval | Home Mixer sources + K allocation | Coverage of new authors/topics that two-tower never returns | Ranking still kills explore items; SID farming |
| Candidate gen | Second explore tower | Extra ANN index + merge K | Cleaner explore/exploit than one skewed index | Memory/QPS; wasted if ranker ignores hits |
| Candidate gen | Live graph walk | New per-request PPR source | Broader in-network expansion | Latency; spam-graph amp — **ruled out** |
| Ranking objective | Reciprocal conversation head | New Phoenix logit + scorer mix | Pay for talks, not dunk-reply farms | Sparse; celebrity bias if not gated |
| Ranking objective | Session-surplus dwell | Reweight attention terms in `RankingScorer` | Less rage-scroll / autoplay addiction | Video length gaming; night-owl false penalty |
| Ranking objective | Cross-ideology bonus | Distance-to-history in score | Less echo | Political product, dunking — **ruled out as weight** |
| Anti-gaming | Graph-normalized positives | Residual P(action) vs author baseline | Rings inflate less | Punish true hits; language/region baseline error |
| Anti-gaming | Reply-farm quality gate | Multiply reply utility by thread-shape | Stops farming the reply=5 head | Heated news looks like bait |
| Anti-gaming | Impression-weighted delayed labels | Phoenix training loss | Breaks early viral loops | Slower trends; need fresh retrieval quota |
| Feedback loops | Durable negative SID/author bag | User-tower token + ranking feature | Mutes/NI actually stick | Over-mute; privacy |
| Feedback loops | Recency decay + viral dropout | Retrieval user vector | Faster recovery from binge/outrage | Short-term CTR drop |
| Feedback loops | IPS on not-interested | Training on p(show) | Unbiased negatives | Bad propensities explode — **ruled out as v1** |
| Creator equity | Quality-at-equal-impressions tax | Post-Phoenix multiplier on saturated authors | Slow rich-get-richer | Users miss celebrities; repost-reset games |
| Creator equity | TS cold-start with better reward | Existing Beta bandit, new reward | Fewer wasted equity slots | Variance; clicky reward if unchanged |
| Creator equity | Hard lottery slots | Force positions | Cosmetic equity | Relevance collapse — **ruled out** |

---

## 2. Shortlist (5)

### 1) Quota-mixed retrieval + explore index (1.1 + 1.2)

Phoenix ranking with candidate isolation is **conditionally optimal given
the candidate set**. If the two-tower never returns a new creator, no
weight in `param.rs` can surface them. Quotas guarantee **support**; a
second low-impression index keeps viral geometry from eating explore.
**Test:** 2×2 A/B {quota on/off} × {explore tower on/off}. Primary:
7-day follows of <10k-follower authors, unique SID prefixes in top 30.
Guardrails: D1 retention, time spent, VF drops, p95 latency. Kill the
tower if explore retrieval→top-20 conversion < a pre-registered floor.

### 2) Reciprocal conversation utility (2.1)

The production objective **already overpays replies**. That is the right
instinct (conversation > like) and the wrong instrument (volume). A
reciprocal head, **gated to in-network / similar-size authors**, plus a
soft reply-farm gate (3.2) is the coherent pair: pay quality conversation,
tax dunk volume. **Test:** train head offline on 24h author-reply labels;
shadow-score 5% traffic; then A/B β ∈ {0, 0.5, 1.0} with bidirectional
boost held fixed. Primary: author→viewer replies per 1000 impressions,
not total replies. Guardrail: reports on conversation modules, celebrity
in-network share.

### 3) Durable negative memory (4.1)

Without a user-id embedding, **the user is their binge**. Ranking
P(not_interested) is too late (post already retrieved, often already
top-K). A TTL’d negative bag in the **user tower** is the smallest change
that makes mute/NI/block affect **generation**, which is where echo and
harassment loops live. **Test:** log re-retrieval rate of muted authors at
t+7d (offline replay). A/B: NI/mute/block **repeat exposure** (−), topic
entropy (+/neutral), empty-feed rate. Start with author-level bag only
(lower privacy and SID-farm risk).

### 4) Quality-at-equal-impressions + better cold-start reward (5.1 + 5.2)

You already lift the left tail into a slot. You do **not** tax the right
tail, so famous-author predictability dominates. QEI is a **continuous
fairness regularizer** that is harder to screenshot than a lottery. Pair
with Thompson sampling whose reward is **follow + residual dwell − report**,
capped at 1–2 injections per request. **Test:** A/B γ tax grid vs holdout;
primary Gini of author impressions **and** NDCG-proxy (dwell on top 10).
Pre-register: no more than X% drop in time spent. Bandit arm: compare
current slot vs TS with new reward on **follows per cold-start impression**.

### 5) Residualized positives + delayed labels (3.1 + 3.3)

Engagement-bait and rings win because **labels are cheap and early**.
Residualize vs author baseline (use `user-cred-v2` / follower bucket) and
delay public positives. Keep negatives fast. This is the robustness layer
that makes (2) and (4) harder to game. **Test:** delayed-label training
replay; metric: impression share of accounts suspended in the next 14d
(should drop) vs time-to-first-impression for verified breaking-news posts
(should not rise more than Y minutes — else 1.1 freshness quota is
mandatory companion).

---

## 3. Rejected but non-obvious

**Rejected: make Phoenix ranking listwise — let candidates attend to each
other so the transformer outputs a slate-aware score.**

This looks like the “real” fix for diversity, bait clusters, and
author-repetition: the model would see that five near-duplicate dunks are
in the same batch and downrank them jointly. It matches modern listwise
recommenders and would seem to obsolete VMRanker’s DPP.

**Hidden flaw.** The isolation mask exists so a post’s score is **a function
of (user, history, that post)** only — cacheable, stable, shardable, and
not manipulable by **which other 1,499 candidates retrieval happened to
return**. If candidates attend to each other:

1. Scores become **set-dependent**. Adversaries change a post’s rank by
   flooding retrieval with similar SIDs (or holding them back).
2. You **cannot** cache Phoenix logits per (user, post) across requests or
   shadow clusters.
3. You duplicate **slate logic you already paid for** in author-diversity
   decay + DPP. Those are the right layer: cheap, inspectable, parameterized.
4. Training becomes **off-policy on a moving candidate set**; IPS (already
   too hard for v1) becomes mandatory.

Slate awareness should stay in **Home Mixer / VMRanker**. Phoenix should
keep predicting **counterfactual action probabilities**. The product bug is
the **utility function and the candidate support**, not the isolation mask.
