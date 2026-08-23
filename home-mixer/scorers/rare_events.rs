//! Rare *real* events: breaking posts, not everyday viral clips.
//!
//! A post counts only if it is fresh, original, and the conversation rate is
//! extreme versus views. That combination is uncommon by design so the
//! rare-event slot does not become another dunk farm.

use crate::models::candidate::PostCandidate;
use std::time::Duration;
use xai_candidate_pipeline::component_library::utils::duration_since_creation_opt;

pub fn is_original(candidate: &PostCandidate) -> bool {
    candidate.in_reply_to_tweet_id.is_none() && candidate.retweeted_tweet_id.is_none()
}

/// 0 = ordinary, ~1 = rare real event.
pub fn rare_event_score(candidate: &PostCandidate, max_age: Duration, min_views: f64) -> f64 {
    if !is_original(candidate) {
        return 0.0;
    }
    if candidate.quoted_tweet_id.is_some() {
        return 0.0;
    }
    let Some(age) = duration_since_creation_opt(candidate.tweet_id) else {
        return 0.0;
    };
    if age > max_age {
        return 0.0;
    }
    let views = candidate.view_count.unwrap_or(0) as f64;
    if views < min_views {
        return 0.0;
    }
    let replies = candidate.reply_count.unwrap_or(0).max(0) as f64;
    let reposts = candidate.repost_count.unwrap_or(0).max(0) as f64;
    let quotes = candidate.quote_count.unwrap_or(0).max(0) as f64;
    let favs = candidate.fav_count.unwrap_or(0).max(0) as f64;
    let talk = replies + 1.5 * reposts + quotes;
    let talk_rate = talk / views;
    let fav_rate = favs / views;
    // Reply farms: many replies, almost no likes.
    if replies > 40.0 && favs > 0.0 && replies / (favs + 1.0) > 12.0 {
        return 0.0;
    }
    let freshness = 1.0 - (age.as_secs_f64() / max_age.as_secs_f64()).clamp(0.0, 1.0);
    // Extremely conversational *and* not empty of likes.
    let burst = (talk_rate / 0.12).clamp(0.0, 1.5) * (0.4 + 0.6 * (fav_rate / 0.04).clamp(0.0, 1.0));
    let score = freshness * burst;
    if score < 0.7 {
        0.0
    } else {
        score.min(1.5)
    }
}

pub fn rare_event_multiplier(candidate: &PostCandidate, max_age: Duration, min_views: f64, boost: f64) -> f64 {
    let s = rare_event_score(candidate, max_age, min_views);
    if s <= 0.0 {
        1.0
    } else {
        1.0 + boost.max(0.0) * s
    }
}

pub fn from_query_params(
    candidate: &PostCandidate,
    max_age_secs: u64,
    min_views: f64,
    boost: f64,
) -> f64 {
    rare_event_multiplier(
        candidate,
        Duration::from_secs(max_age_secs.max(1)),
        min_views,
        boost,
    )
}

pub fn is_rare_event(candidate: &PostCandidate, max_age_secs: u64, min_views: f64) -> bool {
    rare_event_score(
        candidate,
        Duration::from_secs(max_age_secs.max(1)),
        min_views,
    ) >= 0.7
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordinary_post_is_not_rare() {
        let c = PostCandidate {
            view_count: Some(10_000),
            reply_count: Some(3),
            fav_count: Some(50),
            ..Default::default()
        };
        assert_eq!(
            rare_event_score(&c, Duration::from_secs(7200), 50.0),
            0.0
        );
    }

    #[test]
    fn replies_and_reposts_are_not_events() {
        let c = PostCandidate {
            in_reply_to_tweet_id: Some(1),
            view_count: Some(5000),
            reply_count: Some(800),
            fav_count: Some(200),
            ..Default::default()
        };
        assert_eq!(
            rare_event_score(&c, Duration::from_secs(7200), 50.0),
            0.0
        );
    }
}
