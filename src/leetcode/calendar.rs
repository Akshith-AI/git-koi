use std::collections::BTreeMap;

use chrono::{DateTime, Datelike, NaiveDate};
use serde_json::Value;

use super::profile::{field, invalid, string, unsigned};
use super::transport::data;
use super::{ActivityCalendar, LeetCodeError};

/// Query one explicit calendar year, without assuming lifetime history.
pub const CALENDAR_QUERY: &str = "query($username: String!, $year: Int!) { matchedUser(username: $username) { userCalendar(year: $year) { streak totalActiveDays submissionCalendar } } }";

/// Decode the nested JSON calendar and validate UTC dates and activity counts.
pub fn parse_calendar(
    response: &Value,
    observed_on: NaiveDate,
) -> Result<ActivityCalendar, LeetCodeError> {
    let calendar = field(field(data(response)?, "matchedUser")?, "userCalendar")?;
    let encoded = string(calendar, "submissionCalendar")?;
    let raw: BTreeMap<String, u64> = serde_json::from_str(encoded)
        .map_err(|_| invalid("submissionCalendar must encode timestamp/count pairs"))?;
    if raw.len() > 366 {
        return Err(invalid("calendar has more than 366 days"));
    }
    let mut submissions = BTreeMap::new();
    for (timestamp, count) in raw {
        let timestamp = timestamp
            .parse::<i64>()
            .map_err(|_| invalid("calendar timestamp is invalid"))?;
        let day = DateTime::from_timestamp(timestamp, 0)
            .ok_or_else(|| invalid("calendar timestamp is out of range"))?
            .date_naive();
        if timestamp.rem_euclid(86_400) != 0
            || day.year() != observed_on.year()
            || day > observed_on
        {
            return Err(invalid("calendar date is outside the requested UTC window"));
        }
        if submissions.insert(day, count).is_some() {
            return Err(invalid("duplicate calendar date"));
        }
    }
    let total_active_days = unsigned(calendar, "totalActiveDays")?;
    if total_active_days != submissions.values().filter(|&&count| count > 0).count() as u64 {
        return Err(invalid("active day count disagrees with the calendar"));
    }
    Ok(ActivityCalendar {
        year: observed_on.year(),
        reported_streak: unsigned(calendar, "streak")?,
        total_active_days,
        submissions,
    })
}
