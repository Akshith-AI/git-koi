use std::cell::RefCell;
use std::collections::VecDeque;

use chrono::NaiveDate;
use git_koi::leetcode::*;
use serde_json::{Value, json};

struct Client {
    replies: RefCell<VecDeque<Result<Value, LeetCodeError>>>,
    calls: RefCell<Vec<(String, Value)>>,
}
impl LeetCodeClient for Client {
    fn query(&self, query: &str, variables: &Value) -> Result<Value, LeetCodeError> {
        self.calls
            .borrow_mut()
            .push((query.into(), variables.clone()));
        self.replies
            .borrow_mut()
            .pop_front()
            .expect("unexpected request")
    }
}
fn fixture(name: &str) -> Value {
    let file = match name {
        "profile" => include_str!("fixtures/leetcode/profile.json"),
        "topics" => include_str!("fixtures/leetcode/topics.json"),
        "calendar" => include_str!("fixtures/leetcode/calendar.json"),
        "recent" => include_str!("fixtures/leetcode/recent.json"),
        "unknown" => include_str!("fixtures/leetcode/unknown.json"),
        "restricted" => include_str!("fixtures/leetcode/restricted.json"),
        _ => panic!("unknown fixture"),
    };
    serde_json::from_str(file).unwrap()
}
fn client(replies: Vec<Result<Value, LeetCodeError>>) -> Client {
    Client {
        replies: RefCell::new(replies.into()),
        calls: RefCell::default(),
    }
}
fn date() -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 11, 20).unwrap()
}

#[test]
fn collection_is_deterministic_year_scoped_and_serializable() {
    let client = client(
        ["profile", "topics", "calendar", "recent"]
            .map(|name| Ok(fixture(name)))
            .into(),
    );
    let snapshot = collect_leetcode_snapshot(&client, "examplecoder", date()).unwrap();
    assert_eq!(snapshot.username, "ExampleCoder");
    assert_eq!(snapshot.schema_version, 1);
    assert_eq!(snapshot.solved.all, 10);
    assert!(matches!(snapshot.calendar, Availability::Available { .. }));
    let calls = client.calls.borrow();
    assert_eq!(calls.len(), 4);
    assert_eq!(
        calls[0],
        (PROFILE_QUERY.into(), json!({"username":"examplecoder"}))
    );
    assert_eq!(
        calls[2],
        (
            CALENDAR_QUERY.into(),
            json!({"username":"examplecoder", "year":2026})
        )
    );
    let serialized = serde_json::to_string(&snapshot).unwrap();
    assert_eq!(
        serde_json::from_str::<LeetCodeSnapshot>(&serialized).unwrap(),
        snapshot
    );
}

#[test]
fn calendar_permission_failure_keeps_solved_counts_and_other_optional_data() {
    let client = client(
        ["profile", "topics", "restricted", "recent"]
            .map(|name| Ok(fixture(name)))
            .into(),
    );
    let snapshot = collect_leetcode_snapshot(&client, "ExampleCoder", date()).unwrap();
    assert_eq!(snapshot.solved.all, 10);
    assert!(matches!(
        snapshot.calendar,
        Availability::Unavailable { .. }
    ));
    assert!(matches!(snapshot.topics, Availability::Available { .. }));
    assert!(matches!(
        snapshot.recent_accepted,
        Availability::Available { .. }
    ));
}

#[test]
fn malformed_or_failed_optional_requests_do_not_fabricate_inactivity() {
    let client = client(vec![
        Ok(fixture("profile")),
        Ok(json!({})),
        Err(LeetCodeError::Request("offline".into())),
        Ok(json!({"errors":[{"message":"blocked"}]})),
    ]);
    let snapshot = collect_leetcode_snapshot(&client, "ExampleCoder", date()).unwrap();
    assert!(matches!(snapshot.topics, Availability::Unavailable { .. }));
    assert!(matches!(
        snapshot.calendar,
        Availability::Unavailable { .. }
    ));
    assert!(matches!(
        snapshot.recent_accepted,
        Availability::Unavailable { .. }
    ));
    assert_eq!(client.calls.borrow().len(), 4);
}

#[test]
fn unknown_accounts_and_required_failures_stop_before_optional_queries() {
    for response in [
        Ok(fixture("unknown")),
        Ok(json!({})),
        Err(LeetCodeError::Request("offline".into())),
    ] {
        let client = client(vec![response]);
        assert!(collect_leetcode_snapshot(&client, "ExampleCoder", date()).is_err());
        assert_eq!(client.calls.borrow().len(), 1);
    }
    let client = client(vec![]);
    assert!(collect_leetcode_snapshot(&client, "bad/name", date()).is_err());
    assert!(client.calls.borrow().is_empty());
}

#[test]
fn empty_account_is_confirmed_empty_not_unavailable() {
    let mut profile = fixture("profile");
    for row in profile["data"]["matchedUser"]["submitStatsGlobal"]["acSubmissionNum"]
        .as_array_mut()
        .unwrap()
    {
        row["count"] = json!(0);
    }
    let client = client(vec![
        Ok(profile),
        Ok(
            json!({"data":{"matchedUser":{"tagProblemCounts":{"fundamental":[],"intermediate":[],"advanced":[]}}}}),
        ),
        Ok(
            json!({"data":{"matchedUser":{"userCalendar":{"streak":0,"totalActiveDays":0,"submissionCalendar":"{}"}}}}),
        ),
        Ok(json!({"data":{"recentAcSubmissionList":[]}})),
    ]);
    let snapshot = collect_leetcode_snapshot(&client, "ExampleCoder", date()).unwrap();
    assert_eq!(snapshot.solved.all, 0);
    assert_eq!(snapshot.topics, Availability::Available { data: vec![] });
    assert_eq!(
        snapshot.recent_accepted,
        Availability::Available { data: vec![] }
    );
    let Availability::Available { data } = snapshot.calendar else {
        panic!("empty calendar became unavailable")
    };
    assert!(data.submissions.is_empty());
}

#[test]
fn topics_cannot_exceed_the_unique_solved_total() {
    let mut topics = fixture("topics");
    topics["data"]["matchedUser"]["tagProblemCounts"]["fundamental"][0]["problemsSolved"] =
        json!(11);
    let client = client(vec![
        Ok(fixture("profile")),
        Ok(topics),
        Ok(fixture("calendar")),
        Ok(fixture("recent")),
    ]);
    let snapshot = collect_leetcode_snapshot(&client, "ExampleCoder", date()).unwrap();
    assert!(matches!(snapshot.topics, Availability::Unavailable { .. }));
}
