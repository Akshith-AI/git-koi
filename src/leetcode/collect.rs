use chrono::{Datelike, NaiveDate};

use super::profile::invalid;
use super::transport::variables;
use super::{
    Availability, CALENDAR_QUERY, LeetCodeClient, LeetCodeError, LeetCodeSnapshot, PROFILE_QUERY,
    RECENT_QUERY, TOPICS_QUERY, parse_calendar, parse_profile, parse_recent, parse_topics,
    validate_username,
};

/// Collect required solved totals and independently optional public observations.
///
/// Exactly four queries are made for a valid account. Required profile failures
/// stop immediately; each optional failure is recorded without becoming zero.
/// The caller supplies the UTC observation date so fixtures are deterministic.
pub fn collect_leetcode_snapshot(
    client: &impl LeetCodeClient,
    username: &str,
    observed_on: NaiveDate,
) -> Result<LeetCodeSnapshot, LeetCodeError> {
    validate_username(username)?;
    let variables = variables(username);
    let requested_username = username;
    let (username, solved) = parse_profile(&client.query(PROFILE_QUERY, &variables)?)?;
    if !username.eq_ignore_ascii_case(requested_username) {
        return Err(invalid(
            "returned profile does not match the requested username",
        ));
    }
    let topics = available(
        client
            .query(TOPICS_QUERY, &variables)
            .and_then(|value| parse_topics(&value))
            .and_then(|topics| {
                if topics.iter().any(|topic| topic.solved > solved.all) {
                    return Err(invalid("a topic count exceeds all unique solved problems"));
                }
                Ok(topics)
            }),
    );
    let mut calendar_variables = variables.clone();
    calendar_variables["year"] = observed_on.year().into();
    let calendar = available(
        client
            .query(CALENDAR_QUERY, &calendar_variables)
            .and_then(|value| parse_calendar(&value, observed_on)),
    );
    let recent_accepted = available(
        client
            .query(RECENT_QUERY, &variables)
            .and_then(|value| parse_recent(&value, observed_on)),
    );
    Ok(LeetCodeSnapshot {
        schema_version: 1,
        username,
        observed_on,
        solved,
        topics,
        calendar,
        recent_accepted,
    })
}

fn available<T>(result: Result<T, LeetCodeError>) -> Availability<T> {
    match result {
        Ok(data) => Availability::Available { data },
        Err(error) => Availability::Unavailable {
            reason: error.to_string(),
        },
    }
}
