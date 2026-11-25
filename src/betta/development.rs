use std::collections::BTreeMap;

use crate::leetcode::{
    Availability, LeetCodeError, LeetCodeSnapshot, topic_weights, validate_username,
};

/// Validated, deterministic visual inputs for a koi-marble Halfmoon male.
#[derive(Debug, Clone, PartialEq)]
pub struct BettaDevelopment {
    /// Stable, case-insensitive username seed, independent of activity totals.
    pub identity_seed: u64,
    /// Unique solved problems, never submission attempts.
    pub solved: u64,
    /// Logarithmic growth in the inclusive range zero to one, mature at 3,000 solves.
    pub growth: f64,
    /// Easy, Medium, Hard fractions; all zero for a confirmed new account.
    pub difficulty_shares: [f64; 3],
    /// Normalized overlapping tag incidences; absent when topics are unavailable.
    pub topic_weights: BTreeMap<String, f64>,
}

/// Validate a saved snapshot and derive visual traits without network or clock access.
/// Optional activity is deliberately not interpreted as a daily streak.
pub fn derive_betta_development(
    snapshot: &LeetCodeSnapshot,
) -> Result<BettaDevelopment, LeetCodeError> {
    let invalid = |message: &str| LeetCodeError::InvalidData(message.into());
    if snapshot.schema_version != 1 {
        return Err(invalid("unsupported LeetCode snapshot version"));
    }
    validate_username(&snapshot.username)?;
    let counts = &snapshot.solved;
    if counts
        .easy
        .checked_add(counts.medium)
        .and_then(|n| n.checked_add(counts.hard))
        != Some(counts.all)
    {
        return Err(invalid("difficulty counts do not sum to All"));
    }
    let topics = match &snapshot.topics {
        Availability::Available { data } => {
            let mut seen = BTreeMap::new();
            for topic in data {
                if topic.slug.is_empty() || topic.name.is_empty() || topic.solved > counts.all {
                    return Err(invalid("invalid topic observation"));
                }
                if let Some(previous) = seen.insert(&topic.slug, topic)
                    && previous != topic
                {
                    return Err(invalid("conflicting topic observations"));
                }
            }
            topic_weights(data)
        }
        Availability::Unavailable { .. } => BTreeMap::new(),
    };
    let total = counts.all as f64;
    let difficulty_shares = if counts.all == 0 {
        [0.0; 3]
    } else {
        [
            counts.easy as f64 / total,
            counts.medium as f64 / total,
            counts.hard as f64 / total,
        ]
    };
    Ok(BettaDevelopment {
        identity_seed: stable_seed(&snapshot.username.to_ascii_lowercase()),
        solved: counts.all,
        growth: (1.0 + total.min(3_000.0) / 30.0).ln() / 101.0_f64.ln(),
        difficulty_shares,
        topic_weights: topics,
    })
}

pub(super) fn stable_seed(text: &str) -> u64 {
    text.bytes().fold(0xcbf29ce484222325, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3)
    })
}
