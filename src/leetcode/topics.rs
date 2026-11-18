use std::collections::BTreeMap;

use serde_json::Value;

use super::profile::{field, invalid, string, unsigned};
use super::transport::data;
use super::{LeetCodeError, TopicCount};

/// Query overlapping public topic statistics across LeetCode's skill groups.
pub const TOPICS_QUERY: &str = "query($username: String!) { matchedUser(username: $username) { tagProblemCounts { fundamental { tagName tagSlug problemsSolved } intermediate { tagName tagSlug problemsSolved } advanced { tagName tagSlug problemsSolved } } } }";

/// Flatten topic groups into a stable slug ordering, rejecting conflicting duplicates.
pub fn parse_topics(response: &Value) -> Result<Vec<TopicCount>, LeetCodeError> {
    let groups = field(field(data(response)?, "matchedUser")?, "tagProblemCounts")?;
    let mut topics = BTreeMap::new();
    for group in ["fundamental", "intermediate", "advanced"] {
        for row in field(groups, group)?
            .as_array()
            .ok_or_else(|| invalid("topic group must be an array"))?
        {
            let topic = TopicCount {
                slug: string(row, "tagSlug")?.into(),
                name: string(row, "tagName")?.into(),
                solved: unsigned(row, "problemsSolved")?,
            };
            if let Some(previous) = topics.insert(topic.slug.clone(), topic.clone()) {
                if previous != topic {
                    return Err(invalid("conflicting topic counts"));
                }
            }
        }
    }
    Ok(topics.into_values().collect())
}

/// Normalize aggregate tag incidences into visual weights summing to one.
///
/// These are not per-problem fractional weights: the endpoint does not expose
/// enough information to remove multi-tag bias from aggregate counts.
pub fn topic_weights(topics: &[TopicCount]) -> BTreeMap<String, f64> {
    let mut counts: BTreeMap<String, f64> = BTreeMap::new();
    for topic in topics {
        // Repeated slugs cannot multiply their visual influence.
        counts
            .entry(topic.slug.clone())
            .and_modify(|n| *n = n.max(topic.solved as f64))
            .or_insert(topic.solved as f64);
    }
    let total: f64 = counts.values().sum();
    if total == 0.0 {
        return BTreeMap::new();
    }
    counts
        .into_iter()
        .filter(|(_, count)| *count > 0.0)
        .map(|(slug, count)| (slug, count / total))
        .collect()
}
