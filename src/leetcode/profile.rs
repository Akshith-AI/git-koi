use std::collections::BTreeMap;

use serde_json::Value;

use super::transport::data;
use super::{LeetCodeError, SolvedCounts};

/// Query for canonical identity and unique accepted problem counts.
pub const PROFILE_QUERY: &str = "query($username: String!) { matchedUser(username: $username) { username submitStatsGlobal { acSubmissionNum { difficulty count } } } }";

/// Validate a username before starting requests or launching external processes.
pub fn validate_username(username: &str) -> Result<(), LeetCodeError> {
    if username.is_empty()
        || username.len() > 64
        || !username
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || c == b'_' || c == b'-')
    {
        return Err(invalid(
            "username must contain 1–64 letters, digits, underscores, or hyphens",
        ));
    }
    Ok(())
}

/// Parse required profile data; never infer empty counts from a missing account.
pub fn parse_profile(response: &Value) -> Result<(String, SolvedCounts), LeetCodeError> {
    if response
        .get("errors")
        .is_some_and(|errors| !errors.is_array())
    {
        return Err(invalid("errors must be an array"));
    }
    let missing_user = response
        .pointer("/data/matchedUser")
        .is_some_and(Value::is_null);
    let errors = response.get("errors").and_then(Value::as_array);
    if missing_user
        && errors.is_none_or(|e| {
            e.is_empty()
                || e.iter().all(|e| {
                    e.get("message").and_then(Value::as_str) == Some("That user does not exist.")
                })
        })
    {
        return Err(LeetCodeError::AccountNotFound);
    }
    let user = field(data(response)?, "matchedUser")?;
    let username = string(user, "username")?.to_owned();
    validate_username(&username)?;
    let rows = field(field(user, "submitStatsGlobal")?, "acSubmissionNum")?
        .as_array()
        .ok_or_else(|| invalid("accepted counts must be an array"))?;
    let mut counts = BTreeMap::new();
    for row in rows {
        let difficulty = string(row, "difficulty")?;
        if !["All", "Easy", "Medium", "Hard"].contains(&difficulty)
            || counts.insert(difficulty, unsigned(row, "count")?).is_some()
        {
            return Err(invalid("unknown or duplicate difficulty"));
        }
    }
    let count = |name| {
        counts
            .get(name)
            .copied()
            .ok_or_else(|| invalid("missing difficulty count"))
    };
    let solved = SolvedCounts {
        all: count("All")?,
        easy: count("Easy")?,
        medium: count("Medium")?,
        hard: count("Hard")?,
    };
    if solved
        .easy
        .checked_add(solved.medium)
        .and_then(|n| n.checked_add(solved.hard))
        != Some(solved.all)
    {
        return Err(invalid("difficulty counts do not sum to All"));
    }
    Ok((username, solved))
}

pub(super) fn invalid(message: &str) -> LeetCodeError {
    LeetCodeError::InvalidData(message.to_owned())
}

pub(super) fn field<'a>(value: &'a Value, name: &str) -> Result<&'a Value, LeetCodeError> {
    value
        .get(name)
        .filter(|v| !v.is_null())
        .ok_or_else(|| invalid(&format!("missing {name}")))
}

pub(super) fn string<'a>(value: &'a Value, name: &str) -> Result<&'a str, LeetCodeError> {
    field(value, name)?
        .as_str()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| invalid(&format!("{name} must be nonempty text")))
}

pub(super) fn unsigned(value: &Value, name: &str) -> Result<u64, LeetCodeError> {
    field(value, name)?
        .as_u64()
        .ok_or_else(|| invalid(&format!("{name} must be an unsigned integer")))
}
