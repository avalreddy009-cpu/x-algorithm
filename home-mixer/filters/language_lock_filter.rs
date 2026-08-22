use crate::models::candidate::PostCandidate;
use crate::models::query::ScoredPostsQuery;
use crate::params::EnableLanguageLock;
use xai_candidate_pipeline::filter::{Filter, FilterResult};

/// Drop out-of-network posts whose language does not match the viewer.
/// In-network posts and posts with unknown language are kept.
pub struct LanguageLockFilter;

fn languages_match(viewer: &str, post: &str) -> bool {
    let viewer = viewer.trim().to_ascii_lowercase();
    let post = post.trim().to_ascii_lowercase();
    if viewer.is_empty() || post.is_empty() {
        return true;
    }
    let v = viewer.split(['-', '_']).next().unwrap_or(&viewer);
    let p = post.split(['-', '_']).next().unwrap_or(&post);
    v == p
}

impl Filter<ScoredPostsQuery, PostCandidate> for LanguageLockFilter {
    fn enable(&self, query: &ScoredPostsQuery) -> bool {
        query.params.get(EnableLanguageLock) && !query.language_code.trim().is_empty()
    }

    fn filter(
        &self,
        query: &ScoredPostsQuery,
        candidates: Vec<PostCandidate>,
    ) -> FilterResult<PostCandidate> {
        let viewer = query.language_code.as_str();
        let (kept, removed): (Vec<_>, Vec<_>) = candidates.into_iter().partition(|c| {
            if c.in_network == Some(true) {
                return true;
            }
            match &c.language_code {
                None => true,
                Some(code) => languages_match(viewer, code),
            }
        });
        FilterResult { kept, removed }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use xai_feature_switches::FeatureSwitches;

    fn query(lang: &str, lock: bool) -> ScoredPostsQuery {
        let mut query = ScoredPostsQuery {
            language_code: lang.to_string(),
            ..Default::default()
        };
        let fs = FeatureSwitches::new(vec![]).unwrap();
        let mut results = fs.match_recipient(&xai_feature_switches::RecipientBuilder::new().build());
        results.override_fs(
            "rust_home_mixer_enable_language_lock".to_string(),
            if lock { "true" } else { "false" },
        );
        query.params = results.into();
        query
    }

    fn post(id: u64, in_network: bool, lang: Option<&str>) -> PostCandidate {
        PostCandidate {
            tweet_id: id,
            in_network: Some(in_network),
            language_code: lang.map(str::to_string),
            ..Default::default()
        }
    }

    #[test]
    fn drops_oon_other_language_keeps_follows_and_unknown() {
        let filter = LanguageLockFilter;
        let q = query("en-US", true);
        assert!(filter.enable(&q));
        let result = filter.filter(
            &q,
            vec![
                post(1, true, Some("ja")),
                post(2, false, Some("ja")),
                post(3, false, Some("en")),
                post(4, false, None),
            ],
        );
        let kept: Vec<u64> = result.kept.iter().map(|c| c.tweet_id).collect();
        assert_eq!(kept, vec![1, 3, 4]);
        assert_eq!(result.removed.len(), 1);
        assert_eq!(result.removed[0].tweet_id, 2);
    }

    #[test]
    fn disabled_when_flag_off() {
        let filter = LanguageLockFilter;
        let q = query("en", false);
        assert!(!filter.enable(&q));
    }
}
