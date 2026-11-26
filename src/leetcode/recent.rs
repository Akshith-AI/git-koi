use std::collections::BTreeMap;

use chrono::NaiveDate;
use serde_json::Value;

use super::profile::{field, invalid, string};
use super::transport::data;
use super::{AcceptedSubmission, LeetCodeError};

/// Request the latest twenty accepted submissions, not a complete solve history.
pub const RECENT_QUERY: &str = "query($username: String!) { recentAcSubmissionList(username: $username, limit: 20) { id title titleSlug timestamp } }";

/// Decode recent accepts, preserving repeated problems but deduplicating submission IDs.
pub fn parse_recent(
    response: &Value,
    observed_on: NaiveDate,
) -> Result<Vec<AcceptedSubmission>, LeetCodeError> {
    let rows = field(data(response)?, "recentAcSubmissionList")?
        .as_array()
        .ok_or_else(|| invalid("recent accepts must be an array"))?;
    if rows.len() > 20 {
        return Err(invalid("recent accepts exceed the requested limit"));
    }
    let mut submissions = BTreeMap::new();
    for row in rows {
        let timestamp = string(row, "timestamp")?
            .parse::<i64>()
            .map_err(|_| invalid("accepted timestamp is invalid"))?;
        let date = chrono::DateTime::from_timestamp(timestamp, 0)
            .ok_or_else(|| invalid("accepted timestamp is out of range"))?
            .date_naive();
        if timestamp < 0 || date > observed_on {
            return Err(invalid(
                "accepted timestamp is outside the observation window",
            ));
        }
        let submission = AcceptedSubmission {
            id: string(row, "id")?.into(),
            title: string(row, "title")?.into(),
            slug: string(row, "titleSlug")?.into(),
            timestamp,
        };
        if let Some(previous) = submissions.insert(submission.id.clone(), submission.clone())
            && previous != submission
        {
            return Err(invalid("conflicting accepted submission IDs"));
        }
    }
    let mut submissions: Vec<_> = submissions.into_values().collect();
    submissions.sort_by(|a, b| b.timestamp.cmp(&a.timestamp).then_with(|| a.id.cmp(&b.id)));
    Ok(submissions)
}
