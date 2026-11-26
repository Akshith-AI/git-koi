//! Generate deterministic still-image examples from synthetic LeetCode profiles.
use chrono::NaiveDate;
use ltkoi::betta::{derive_betta_development, render_betta};
use ltkoi::leetcode::{Availability, LeetCodeSnapshot, SolvedCounts, TopicCount};
use ltkoi::output::encode_png;
use std::{error::Error, fs, path::Path};

fn main() -> Result<(), Box<dyn Error>> {
    let directory = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "target/betta-gallery".into());
    fs::create_dir_all(&directory)?;
    for (name, counts, topics) in [
        ("beginner", [0, 0, 0], vec![]),
        (
            "arrays",
            [180, 90, 30],
            vec![("array", 200), ("string", 100)],
        ),
        (
            "graphs-dp",
            [128, 426, 155],
            vec![("graph", 210), ("dynamic-programming", 300)],
        ),
        (
            "expert",
            [600, 1400, 1000],
            vec![("tree", 700), ("dynamic-programming", 900)],
        ),
    ] {
        let snapshot = LeetCodeSnapshot {
            schema_version: 1,
            username: "MarbleBetta".into(),
            observed_on: NaiveDate::from_ymd_opt(2026, 11, 25).unwrap(),
            solved: SolvedCounts {
                all: counts.iter().sum(),
                easy: counts[0],
                medium: counts[1],
                hard: counts[2],
            },
            topics: Availability::Available {
                data: topics
                    .into_iter()
                    .map(|(slug, solved)| TopicCount {
                        slug: slug.into(),
                        name: slug.into(),
                        solved,
                    })
                    .collect(),
            },
            calendar: Availability::Unavailable {
                reason: "synthetic example".into(),
            },
            recent_accepted: Availability::Unavailable {
                reason: "synthetic example".into(),
            },
        };
        let development = derive_betta_development(&snapshot)?;
        let image = render_betta(&development);
        fs::write(
            Path::new(&directory).join(format!("{name}.png")),
            encode_png(&image)?,
        )?;
        fs::write(
            Path::new(&directory).join(format!("{name}.json")),
            serde_json::to_string_pretty(&snapshot)?,
        )?;
    }
    Ok(())
}
