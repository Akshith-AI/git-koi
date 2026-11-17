//! Public LeetCode profile collection, separate from fish rendering.
//!
//! Snapshots distinguish unavailable optional data from confirmed empty data.

mod model;

pub use model::*;
mod transport;
pub use transport::{CurlLeetCodeClient, LEETCODE_ENDPOINT, LeetCodeClient};
