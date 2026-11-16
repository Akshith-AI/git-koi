use std::collections::BTreeMap;
use std::fmt;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// A versioned observation of one public LeetCode account.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LeetCodeSnapshot {
    /// Snapshot format version, currently one.
    pub schema_version: u32,
    /// Canonical username returned by LeetCode.
    pub username: String,
    /// UTC date on which the endpoint was queried.
    pub observed_on: NaiveDate,
    /// Unique accepted problem totals, not submission attempts.
    pub solved: SolvedCounts,
    /// Overlapping topic counts; not a unique problem breakdown.
    pub topics: Availability<Vec<TopicCount>>,
    /// One UTC calendar year, potentially including unsuccessful attempts.
    pub calendar: Availability<ActivityCalendar>,
    /// A limited recent accepted-submission window, not full history.
    pub recent_accepted: Availability<Vec<AcceptedSubmission>>,
}

/// Optional data with explicit availability semantics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum Availability<T> {
    /// The endpoint successfully supplied this data, including empty data.
    Available {
        /// Confirmed observations.
        data: T,
    },
    /// Data could not be obtained and must not be treated as zero activity.
    Unavailable {
        /// A diagnostic that does not include credentials or request bodies.
        reason: String,
    },
}

/// Accepted unique problems partitioned by difficulty.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SolvedCounts {
    /// Total across all difficulties.
    pub all: u64,
    /// Easy problems solved.
    pub easy: u64,
    /// Medium problems solved.
    pub medium: u64,
    /// Hard problems solved.
    pub hard: u64,
}

/// An endpoint topic count; one problem may contribute to multiple topics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TopicCount {
    /// Stable endpoint tag slug.
    pub slug: String,
    /// Human-readable topic name.
    pub name: String,
    /// Accepted problems associated with this tag.
    pub solved: u64,
}

/// Calendar activity for a requested year, not an accepted-solve calendar.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActivityCalendar {
    /// Requested UTC year; other years are intentionally excluded.
    pub year: i32,
    /// Endpoint-reported streak; its semantics are not assumed to be current.
    pub reported_streak: u64,
    /// Endpoint-reported active day count for this calendar.
    pub total_active_days: u64,
    /// UTC days with reported submission counts, including possible failed attempts.
    pub submissions: BTreeMap<NaiveDate, u64>,
}

/// One accepted submission in a limited recent window; duplicates may exist.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcceptedSubmission {
    /// Submission identifier as returned by LeetCode.
    pub id: String,
    /// Problem display title.
    pub title: String,
    /// Stable problem slug.
    pub slug: String,
    /// Submission time as Unix seconds.
    pub timestamp: i64,
}

/// A failure to collect or interpret a public LeetCode profile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LeetCodeError {
    /// The endpoint explicitly reports an unknown username.
    AccountNotFound,
    /// Input or endpoint data violates the collection contract.
    InvalidData(String),
    /// A network request failed or returned GraphQL errors.
    Request(String),
}

impl fmt::Display for LeetCodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AccountNotFound => write!(f, "LeetCode account not found"),
            Self::InvalidData(message) => write!(f, "invalid LeetCode data: {message}"),
            Self::Request(message) => write!(f, "LeetCode request failed: {message}"),
        }
    }
}

impl std::error::Error for LeetCodeError {}
