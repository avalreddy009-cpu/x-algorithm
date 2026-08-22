//! Post-Phoenix score multipliers for recency, creator equity, reply-farm
//! quality, reciprocal conversation, and semantic-id diversity.
//!
//! Defaults are conservative. Missing features (no views, no age, no SIDs)
//! multiply by 1.0 so existing tests keep the same scores.

use crate::models::candidate::PostCandidate;
use crate::models::query::ScoredPostsQuery;
use crate::params::*;
use std::collections::HashMap;
use std::time::Duration;
use xai_candidate_pipeline::component_library::utils::duration_since_creation_opt;

pub fn recency_multiplier(age: Option<Duration>, half_life: Duration, gamma: f64) -> f64 {
    if gamma == 0.0 || half_life.is_zero() {
        return 1.0;
    }
    let Some(age) = age else {
        return 1.0;
    };
    let decay = (-age.as_secs_f64() / half_life.as_secs_f64() * std::f64::consts::LN_2).exp();
    1.0 + gamma * decay
}

pub fn qei_multiplier(view_count: Option<u64>, gamma: f64, start_after: f64) -> f64 {
    if gamma <= 0.0 {
        return 1.0;
    }
    let views = view_count.unwrap_or(0) as f64;
    if views <= start_after {
        return 1.0;
    }
    1.0 / (1.0 + gamma * (1.0 + views - start_after).ln())
}

pub fn reply_farm_multiplier(
    candidate: &PostCandidate,
    factor: f64,
    min_replies: f64,
    reply_over_fav_ratio: f64,
) -> f64 {
    if candidate.in_network == Some(true) {
        return 1.0;
    }
    let is_reply_or_quote =
        candidate.in_reply_to_tweet_id.is_some() || candidate.quoted_tweet_id.is_some();
    if !is_reply_or_quote {
        return 1.0;
    }
    let replies = candidate.reply_count.unwrap_or(0).max(0) as f64;
    let favs = candidate.fav_count.unwrap_or(0).max(0) as f64;
    if replies < min_replies {
        return 1.0;
    }
    if replies > reply_over_fav_ratio * (favs + 1.0) {
        return factor.clamp(0.05, 1.0);
    }
    1.0
}

pub fn reciprocal_boost_multiplier(candidate: &PostCandidate, boost: f64) -> f64 {
    if boost == 0.0 {
        return 1.0;
    }
    if candidate.in_network != Some(true) {
        return 1.0;
    }
    if candidate.in_reply_to_tweet_id.is_some() || candidate.retweeted_tweet_id.is_some() {
        return 1.0;
    }
    let talking = candidate.is_mutual_follow_author == Some(true)
        || !candidate.following_replied_user_ids.is_empty();
    if talking {
        1.0 + boost.max(0.0)
    } else {
        1.0
    }
}

pub fn candidate_multiplier(query: &ScoredPostsQuery, candidate: &PostCandidate) -> f64 {
    let mut m = 1.0;

    if query.params.get(EnableRecencyBoost) {
        let half_life = Duration::from_secs(query.params.get(RecencyBoostHalfLifeSecs).max(1));
        let gamma = query.params.get(RecencyBoostGamma);
        m *= recency_multiplier(
            duration_since_creation_opt(candidate.tweet_id),
            half_life,
            gamma,
        );
    }

    if query.params.get(EnableQualityAtEqualImpressions) {
        m *= qei_multiplier(
            candidate.view_count,
            query.params.get(QeiGamma),
            query.params.get(QeiStartAfterImpressions),
        );
    }

    if query.params.get(EnableReplyFarmGate) {
        m *= reply_farm_multiplier(
            candidate,
            query.params.get(ReplyFarmFactor),
            query.params.get(ReplyFarmMinReplies) as f64,
            query.params.get(ReplyFarmReplyOverFavRatio),
        );
    }

    if query.params.get(EnableReciprocalConversationBoost) {
        m *= reciprocal_boost_multiplier(
            candidate,
            query.params.get(ReciprocalConversationBoost),
        );
    }

    m
}

pub fn apply_to_scores(
    query: &ScoredPostsQuery,
    candidates: &[PostCandidate],
    scores: &[f64],
) -> Vec<f64> {
    let mut out: Vec<f64> = candidates
        .iter()
        .zip(scores)
        .map(|(c, s)| s * candidate_multiplier(query, c))
        .collect();

    if query.params.get(EnableSidDiversity) {
        let decay = query.params.get(SidDiversityDecay);
        let floor = query.params.get(SidDiversityFloor);
        let multipliers = sid_diversity_multipliers(candidates, &out, decay, floor);
        for (score, mul) in out.iter_mut().zip(multipliers) {
            *score *= mul;
        }
    }

    out
}

fn sid_key(candidate: &PostCandidate) -> Option<(i32, i32)> {
    match candidate.semantic_ids.as_deref() {
        Some(ids) if ids.len() >= 2 => Some((ids[0], ids[1])),
        Some(ids) if ids.len() == 1 => Some((ids[0], 0)),
        _ => None,
    }
}

pub fn sid_diversity_multipliers(
    candidates: &[PostCandidate],
    scores: &[f64],
    decay_factor: f64,
    floor: f64,
) -> Vec<f64> {
    let mut order: Vec<usize> = (0..candidates.len()).collect();
    order.sort_by(|&a, &b| scores[b].total_cmp(&scores[a]).then(a.cmp(&b)));

    let mut seen: HashMap<(i32, i32), u32> = HashMap::new();
    let mut multipliers = vec![1.0; candidates.len()];
    for i in order {
        let Some(key) = sid_key(&candidates[i]) else {
            continue;
        };
        let k = *seen.entry(key).and_modify(|c| *c += 1).or_insert(0);
        multipliers[i] = (1.0 - floor) * decay_factor.powf(f64::from(k)) + floor;
    }
    multipliers
}

pub fn format_explain(multipliers: &[(&str, f64)], score: f64) -> String {
    let mut parts: Vec<(&str, f64)> = multipliers
        .iter()
        .copied()
        .filter(|(_, v)| (*v - 1.0).abs() > 1e-6)
        .collect();
    parts.sort_by(|a, b| b.1.total_cmp(&a.1));
    let adj = parts
        .iter()
        .take(4)
        .map(|(n, v)| format!("{n}={v:.3}"))
        .collect::<Vec<_>>()
        .join(",");
    if adj.is_empty() {
        format!("score={score:.4}")
    } else {
        format!("score={score:.4};{adj}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recency_is_one_without_age_or_gamma() {
        assert!((recency_multiplier(None, Duration::from_secs(12 * 3600), 0.2) - 1.0).abs() < 1e-12);
        assert!(
            (recency_multiplier(Some(Duration::from_secs(60)), Duration::from_secs(3600), 0.0)
                - 1.0)
                .abs()
                < 1e-12
        );
    }

    #[test]
    fn recency_is_higher_for_fresher_posts() {
        let half = Duration::from_secs(12 * 3600);
        let fresh = recency_multiplier(Some(Duration::from_secs(60)), half, 0.2);
        let stale = recency_multiplier(Some(Duration::from_secs(36 * 3600)), half, 0.2);
        assert!(fresh > stale);
        assert!(fresh > 1.0);
        assert!(stale > 1.0);
    }

    #[test]
    fn qei_taxes_high_impression_posts() {
        assert!((qei_multiplier(None, 0.1, 1_000.0) - 1.0).abs() < 1e-12);
        assert!((qei_multiplier(Some(10), 0.1, 1_000.0) - 1.0).abs() < 1e-12);
        let taxed = qei_multiplier(Some(1_000_000), 0.1, 1_000.0);
        assert!(taxed < 0.9);
        assert!(taxed > 0.3);
    }

    #[test]
    fn reply_farm_spares_in_network_and_originals() {
        let in_net = PostCandidate {
            in_network: Some(true),
            in_reply_to_tweet_id: Some(1),
            reply_count: Some(500),
            fav_count: Some(1),
            ..Default::default()
        };
        assert_eq!(reply_farm_multiplier(&in_net, 0.5, 20.0, 8.0), 1.0);

        let original = PostCandidate {
            in_network: Some(false),
            reply_count: Some(500),
            fav_count: Some(1),
            ..Default::default()
        };
        assert_eq!(reply_farm_multiplier(&original, 0.5, 20.0, 8.0), 1.0);

        let farm = PostCandidate {
            in_network: Some(false),
            quoted_tweet_id: Some(9),
            reply_count: Some(400),
            fav_count: Some(2),
            ..Default::default()
        };
        assert!((reply_farm_multiplier(&farm, 0.5, 20.0, 8.0) - 0.5).abs() < 1e-12);
    }

    #[test]
    fn reciprocal_boosts_mutual_originals() {
        let c = PostCandidate {
            in_network: Some(true),
            is_mutual_follow_author: Some(true),
            ..Default::default()
        };
        assert!((reciprocal_boost_multiplier(&c, 0.15) - 1.15).abs() < 1e-12);
    }

    #[test]
    fn sid_diversity_decays_later_same_prefix() {
        let a = PostCandidate {
            tweet_id: 1,
            semantic_ids: Some(vec![7, 8, 9]),
            ..Default::default()
        };
        let b = PostCandidate {
            tweet_id: 2,
            semantic_ids: Some(vec![7, 8, 1]),
            ..Default::default()
        };
        let scores = vec![2.0, 1.0];
        let m = sid_diversity_multipliers(&[a, b], &scores, 0.5, 0.25);
        assert!((m[0] - 1.0).abs() < 1e-12);
        let expected = 0.75 * 0.5 + 0.25;
        assert!((m[1] - expected).abs() < 1e-12);
    }
}
