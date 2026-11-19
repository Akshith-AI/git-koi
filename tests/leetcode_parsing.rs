use chrono::NaiveDate;
use git_koi::leetcode::*;
use serde_json::{Value, json};

fn date() -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 11, 20).unwrap()
}
fn profile() -> Value {
    serde_json::from_str(include_str!("fixtures/leetcode/profile.json")).unwrap()
}
fn topics() -> Value {
    serde_json::from_str(include_str!("fixtures/leetcode/topics.json")).unwrap()
}
fn calendar() -> Value {
    serde_json::from_str(include_str!("fixtures/leetcode/calendar.json")).unwrap()
}
fn recent() -> Value {
    serde_json::from_str(include_str!("fixtures/leetcode/recent.json")).unwrap()
}

#[test]
fn solved_counts_require_a_complete_consistent_partition() {
    let (username, counts) = parse_profile(&profile()).unwrap();
    assert_eq!(username, "ExampleCoder");
    assert_eq!(counts.all, 10);
    for mutation in [json!(-1), json!("4"), json!(100), Value::Null] {
        let mut response = profile();
        response["data"]["matchedUser"]["submitStatsGlobal"]["acSubmissionNum"][1]["count"] =
            mutation;
        assert!(parse_profile(&response).is_err());
    }
    let mut response = profile();
    let rows = response["data"]["matchedUser"]["submitStatsGlobal"]["acSubmissionNum"]
        .as_array_mut()
        .unwrap();
    rows.push(rows[1].clone());
    assert!(parse_profile(&response).is_err());
}

#[test]
fn usernames_and_unknown_accounts_are_distinct_from_network_errors() {
    for username in ["", " a", "a/b", "a?b", "$(whoami)", "é", &"a".repeat(65)] {
        assert!(validate_username(username).is_err());
    }
    validate_username("Example-coder_42").unwrap();
    let unknown: Value =
        serde_json::from_str(include_str!("fixtures/leetcode/unknown.json")).unwrap();
    assert_eq!(parse_profile(&unknown), Err(LeetCodeError::AccountNotFound));
    let restricted: Value =
        serde_json::from_str(include_str!("fixtures/leetcode/restricted.json")).unwrap();
    assert!(matches!(
        parse_profile(&restricted),
        Err(LeetCodeError::Request(_))
    ));
    assert!(parse_profile(&json!({})).is_err());
}

#[test]
fn zero_solves_are_valid_and_missing_counts_are_not_zero() {
    let mut response = profile();
    for row in response["data"]["matchedUser"]["submitStatsGlobal"]["acSubmissionNum"]
        .as_array_mut()
        .unwrap()
    {
        row["count"] = json!(0);
    }
    assert_eq!(parse_profile(&response).unwrap().1.all, 0);
    response["data"]["matchedUser"]["submitStatsGlobal"]["acSubmissionNum"]
        .as_array_mut()
        .unwrap()
        .pop();
    assert!(parse_profile(&response).is_err());
}

#[test]
fn topic_weights_preserve_overlaps_without_claiming_unique_problem_counts() {
    let topics = parse_topics(&topics()).unwrap();
    assert_eq!(topics.iter().map(|t| t.solved).sum::<u64>(), 12);
    let weights = topic_weights(&topics);
    assert_eq!(weights["array"], 0.5);
    assert!((weights.values().sum::<f64>() - 1.0).abs() < 1e-12);
    let mut duplicated = topics.clone();
    duplicated.extend(topics);
    assert_eq!(topic_weights(&duplicated), weights);
    assert!(topic_weights(&[]).is_empty());
    assert!(
        topic_weights(&[TopicCount {
            slug: "array".into(),
            name: "Array".into(),
            solved: 0
        }])
        .is_empty()
    );
}

#[test]
fn conflicting_topics_and_missing_groups_are_rejected() {
    let mut response = topics();
    let duplicate = response["data"]["matchedUser"]["tagProblemCounts"]["fundamental"][0].clone();
    response["data"]["matchedUser"]["tagProblemCounts"]["advanced"]
        .as_array_mut()
        .unwrap()
        .push(duplicate);
    assert_eq!(parse_topics(&response).unwrap().len(), 3);
    response["data"]["matchedUser"]["tagProblemCounts"]["advanced"][1]["problemsSolved"] = json!(7);
    assert!(parse_topics(&response).is_err());
    response["data"]["matchedUser"]["tagProblemCounts"]["fundamental"] = Value::Null;
    assert!(parse_topics(&response).is_err());
}

#[test]
fn calendar_is_a_year_scoped_submission_observation() {
    let parsed = parse_calendar(&calendar(), date()).unwrap();
    assert_eq!(parsed.year, 2026);
    assert_eq!(parsed.total_active_days, 2);
    assert_eq!(parsed.submissions.values().sum::<u64>(), 5);
    assert_eq!(parsed.reported_streak, 2);
    assert!(parse_calendar(&calendar(), NaiveDate::from_ymd_opt(2026, 11, 16).unwrap()).is_err());
    assert!(parse_calendar(&calendar(), NaiveDate::from_ymd_opt(2025, 11, 20).unwrap()).is_err());
}

#[test]
fn malformed_and_inconsistent_calendars_never_become_empty_activity() {
    for encoded in [
        "not json",
        "[]",
        "{\"bad\": 1}",
        "{\"1794787201\": 1}",
        "{\"1794787200\": -1}",
    ] {
        let mut response = calendar();
        response["data"]["matchedUser"]["userCalendar"]["submissionCalendar"] = json!(encoded);
        assert!(parse_calendar(&response, date()).is_err());
    }
    let mut response = calendar();
    response["data"]["matchedUser"]["userCalendar"]["totalActiveDays"] = json!(0);
    assert!(parse_calendar(&response, date()).is_err());
}

#[test]
fn recent_accepts_keep_repeated_problems_but_not_duplicate_ids() {
    let mut response = recent();
    let duplicate = response["data"]["recentAcSubmissionList"][0].clone();
    response["data"]["recentAcSubmissionList"]
        .as_array_mut()
        .unwrap()
        .push(duplicate);
    let parsed = parse_recent(&response, date()).unwrap();
    assert_eq!(parsed.len(), 2);
    assert_eq!(parsed[0].slug, parsed[1].slug);
    assert!(parsed[0].timestamp > parsed[1].timestamp);
    response["data"]["recentAcSubmissionList"][2]["title"] = json!("conflict");
    assert!(parse_recent(&response, date()).is_err());
}

#[test]
fn invalid_recent_timestamps_and_oversized_windows_fail() {
    for timestamp in ["-1", "bad", "99999999999999", "1893456000"] {
        let mut response = recent();
        response["data"]["recentAcSubmissionList"][0]["timestamp"] = json!(timestamp);
        assert!(parse_recent(&response, date()).is_err());
    }
    let mut response = recent();
    response["data"]["recentAcSubmissionList"] =
        json!(vec![
            response["data"]["recentAcSubmissionList"][0].clone();
            21
        ]);
    assert!(parse_recent(&response, date()).is_err());
}
