//! Slate algorithms that have held up in production search and feeds:
//! Carbonell & Goldstein MMR (1998) and a hard in-network quota.
//!
//! Replacing Phoenix with an LLM ranker on the request path would not work
//! at For You QPS. These run in microseconds on scores we already have.

use crate::models::candidate::{CandidateHelpers, PostCandidate};
use crate::models::query::ScoredPostsQuery;
use crate::params::*;
use crate::scorers::rare_events;
use crate::selectors::hungarian;
use xai_candidate_pipeline::selector::SelectResult;

pub fn sid_jaccard(a: &PostCandidate, b: &PostCandidate) -> f64 {
    let Some(sa) = a.semantic_ids.as_deref() else {
        return 0.0;
    };
    let Some(sb) = b.semantic_ids.as_deref() else {
        return 0.0;
    };
    if sa.is_empty() || sb.is_empty() {
        return 0.0;
    }
    let mut overlap = 0usize;
    for x in sa {
        if sb.contains(x) {
            overlap += 1;
        }
    }
    let union = sa.len() + sb.len() - overlap;
    if union == 0 {
        0.0
    } else {
        overlap as f64 / union as f64
    }
}

pub fn similarity(a: &PostCandidate, b: &PostCandidate) -> f64 {
    if a.author_id != 0 && a.author_id == b.author_id {
        return 1.0;
    }
    if a.get_original_tweet_id() == b.get_original_tweet_id() {
        return 1.0;
    }
    sid_jaccard(a, b)
}

/// Greedy MMR: pick λ·relevance − (1−λ)·max similarity to already picked items.
pub fn mmr_rerank(mut candidates: Vec<PostCandidate>, lambda: f64, k: usize) -> Vec<PostCandidate> {
    if candidates.len() <= 1 || k == 0 {
        return candidates;
    }
    let lambda = lambda.clamp(0.0, 1.0);
    candidates.sort_by(|a, b| {
        b.score
            .unwrap_or(f64::NEG_INFINITY)
            .total_cmp(&a.score.unwrap_or(f64::NEG_INFINITY))
    });
    let mut selected: Vec<PostCandidate> = Vec::with_capacity(k.min(candidates.len()));
    selected.push(candidates.remove(0));
    while selected.len() < k && !candidates.is_empty() {
        let mut best_i = 0;
        let mut best_mmr = f64::NEG_INFINITY;
        for (i, c) in candidates.iter().enumerate() {
            let rel = c.score.unwrap_or(0.0);
            let max_sim = selected
                .iter()
                .map(|s| similarity(c, s))
                .fold(0.0_f64, f64::max);
            let mmr = lambda * rel - (1.0 - lambda) * max_sim;
            if mmr > best_mmr {
                best_mmr = mmr;
                best_i = i;
            }
        }
        selected.push(candidates.remove(best_i));
    }
    selected.append(&mut candidates);
    selected
}

/// Keep score order, but guarantee a minimum share of in-network posts in the top k.
pub fn apply_in_network_quota(
    ranked: Vec<PostCandidate>,
    k: usize,
    fraction: f64,
) -> Vec<PostCandidate> {
    if ranked.len() <= 1 || k == 0 {
        return ranked;
    }
    let want = ((k.min(ranked.len()) as f64) * fraction.clamp(0.0, 1.0)).ceil() as usize;
    if want == 0 {
        return ranked;
    }
    let mut head: Vec<PostCandidate> = ranked.into_iter().collect();
    let take = k.min(head.len());
    let in_head = head[..take]
        .iter()
        .filter(|c| c.in_network == Some(true))
        .count();
    if in_head >= want {
        return head;
    }
    let mut need = want - in_head;
    let mut i = take;
    while need > 0 && i < head.len() {
        if head[i].in_network == Some(true) {
            let mut swap_at = None;
            for j in (0..take).rev() {
                if head[j].in_network != Some(true) {
                    swap_at = Some(j);
                    break;
                }
            }
            if let Some(j) = swap_at {
                head.swap(j, i);
                need -= 1;
            } else {
                break;
            }
        }
        i += 1;
    }
    head
}

fn slot_profit(candidate: &PostCandidate, slot: usize, k: usize, query: &ScoredPostsQuery) -> f64 {
    let score = candidate.score.unwrap_or(0.0).max(0.0);
    let decay = query.params.get(HungarianPositionDecay).clamp(0.5, 1.0);
    let mut profit = score * decay.powi(slot as i32);
    if candidate.in_network == Some(true) && slot * 2 < k {
        profit += 0.22 * score;
    }
    let rare_slot = 1.min(k.saturating_sub(1));
    let explore_slot = ((k / 4).max(2)).min(k.saturating_sub(1));
    let max_age = query.params.get(RareEventMaxAgeSecs);
    let min_views = query.params.get(RareEventMinViews);
    if slot == rare_slot {
        let rare = rare_events::rare_event_score(
            candidate,
            std::time::Duration::from_secs(max_age.max(1)),
            min_views,
        );
        profit += 4.0 * score.max(0.05) * rare;
    }
    if slot == explore_slot
        && candidate.in_network != Some(true)
        && candidate.view_count.unwrap_or(u64::MAX) < 800
    {
        profit += 0.4 * score;
    }
    profit
}

/// Optimal post→slot matching (Hungarian). Remainder keeps score order.
pub fn hungarian_assign(
    mut candidates: Vec<PostCandidate>,
    k: usize,
    query: &ScoredPostsQuery,
) -> Vec<PostCandidate> {
    if candidates.len() <= 1 || k == 0 {
        return candidates;
    }
    candidates.sort_by(|a, b| {
        b.score
            .unwrap_or(f64::NEG_INFINITY)
            .total_cmp(&a.score.unwrap_or(f64::NEG_INFINITY))
    });
    let max_n = (query.params.get(HungarianMaxN) as usize).clamp(4, 48);
    let pool_n = candidates.len().min(max_n).min(k.saturating_mul(2).max(k));
    let pool: Vec<PostCandidate> = candidates.drain(..pool_n).collect();
    let rest = candidates;
    let slots = k.min(pool.len());
    let mut profit = vec![vec![0.0; slots]; pool.len()];
    for (i, c) in pool.iter().enumerate() {
        for j in 0..slots {
            profit[i][j] = slot_profit(c, j, k, query);
        }
    }
    let square = hungarian::pad_square(&profit);
    let col_of_row = hungarian::hungarian_maximize(&square);
    let mut slot_to_post = vec![None; slots];
    for (i, &j) in col_of_row.iter().enumerate().take(pool.len()) {
        if j < slots {
            slot_to_post[j] = Some(i);
        }
    }
    let mut used = vec![false; pool.len()];
    let mut ordered = Vec::with_capacity(pool.len() + rest.len());
    for j in 0..slots {
        if let Some(i) = slot_to_post[j] {
            if !used[i] {
                ordered.push(pool[i].clone());
                used[i] = true;
            }
        }
    }
    for (i, c) in pool.into_iter().enumerate() {
        if !used[i] {
            ordered.push(c);
        }
    }
    ordered.extend(rest);
    ordered
}

pub fn select_slate(
    query: &ScoredPostsQuery,
    candidates: Vec<PostCandidate>,
    k: usize,
) -> SelectResult<PostCandidate> {
    let mut ranked = if query.params.get(EnableHungarianAssignment) {
        hungarian_assign(candidates, k, query)
    } else if query.params.get(EnableMmrRerank) {
        mmr_rerank(candidates, query.params.get(MmrLambda), k)
    } else {
        let mut sorted = candidates;
        sorted.sort_by(|a, b| {
            b.score
                .unwrap_or(f64::NEG_INFINITY)
                .total_cmp(&a.score.unwrap_or(f64::NEG_INFINITY))
        });
        sorted
    };

    if query.params.get(EnableInNetworkQuota) {
        ranked = apply_in_network_quota(
            ranked,
            k,
            query.params.get(InNetworkQuotaFraction),
        );
    }

    let limit = k.min(ranked.len());
    let non_selected = ranked.split_off(limit);
    SelectResult {
        selected: ranked,
        non_selected,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn post(id: u64, author: u64, score: f64, in_network: bool, sids: Option<Vec<i32>>) -> PostCandidate {
        PostCandidate {
            tweet_id: id,
            author_id: author,
            score: Some(score),
            in_network: Some(in_network),
            semantic_ids: sids,
            ..Default::default()
        }
    }

    #[test]
    fn mmr_avoids_same_author_when_lambda_allows() {
        let ranked = mmr_rerank(
            vec![
                post(1, 10, 10.0, true, None),
                post(2, 10, 9.0, true, None),
                post(3, 11, 8.0, true, None),
            ],
            0.5,
            2,
        );
        assert_eq!(ranked[0].tweet_id, 1);
        assert_eq!(ranked[1].tweet_id, 3);
    }

    #[test]
    fn quota_pulls_in_network_into_top_k() {
        let ranked = apply_in_network_quota(
            vec![
                post(1, 1, 5.0, false, None),
                post(2, 2, 4.0, false, None),
                post(3, 3, 3.0, true, None),
            ],
            2,
            0.5,
        );
        assert_eq!(ranked[0].tweet_id, 1);
        assert_eq!(ranked[1].tweet_id, 3);
    }
}
