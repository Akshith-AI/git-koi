//! Public LeetCode profile collection, separate from fish rendering.
//!
//! Snapshots distinguish unavailable optional data from confirmed empty data.

mod model;

pub use model::*;
mod transport;
pub use transport::{CurlLeetCodeClient, LEETCODE_ENDPOINT, LeetCodeClient};
mod profile;
pub use profile::{PROFILE_QUERY, parse_profile, validate_username};
mod topics;
pub use topics::{TOPICS_QUERY, parse_topics, topic_weights};
mod calendar;
pub use calendar::{CALENDAR_QUERY, parse_calendar};
