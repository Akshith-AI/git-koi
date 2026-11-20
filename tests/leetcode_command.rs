#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

use git_koi::leetcode::{Availability, LeetCodeSnapshot};

static NEXT: AtomicU64 = AtomicU64::new(0);
struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "ltkoi-cli-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn curl(&self, script: &str) {
        let path = self.0.join("curl");
        fs::write(&path, format!("#!/bin/sh\n{script}\n")).unwrap();
        fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
    }
    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_git-koi"))
            .args(args)
            .env("PATH", &self.0)
            .env(
                "LTKOI_FIXTURES",
                concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/leetcode"),
            )
            .output()
            .unwrap()
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

const NORMAL: &str = r#"
case "$*" in
  *submitStatsGlobal*) /bin/cat "$LTKOI_FIXTURES/profile.json" ;;
  *tagProblemCounts*) /bin/cat "$LTKOI_FIXTURES/topics.json" ;;
  *userCalendar*) /bin/cat "$LTKOI_FIXTURES/restricted.json" ;;
  *recentAcSubmissionList*) /bin/cat "$LTKOI_FIXTURES/recent.json" ;;
  *) exit 7 ;;
esac
"#;

#[test]
fn command_writes_versioned_json_to_stdout_and_file_without_network() {
    let scratch = Scratch::new();
    scratch.curl(NORMAL);
    let output = scratch.run(&["collect-leetcode", "examplecoder"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let snapshot: LeetCodeSnapshot = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(snapshot.solved.all, 10);
    assert_eq!(snapshot.username, "ExampleCoder");
    assert!(matches!(
        snapshot.calendar,
        Availability::Unavailable { .. }
    ));
    let path = scratch.0.join("snapshot.json");
    let output = scratch.run(&[
        "collect-leetcode",
        "examplecoder",
        "--output",
        path.to_str().unwrap(),
    ]);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert_eq!(
        serde_json::from_slice::<LeetCodeSnapshot>(&fs::read(&path).unwrap()).unwrap(),
        snapshot
    );
    assert!(!scratch.0.join("snapshot.json.tmp").exists());
}

#[test]
fn required_failures_preserve_previous_snapshot() {
    let scratch = Scratch::new();
    let path = scratch.0.join("snapshot.json");
    fs::write(&path, "previous snapshot").unwrap();
    for script in [
        "exit 22",
        "printf 'not json'",
        "/bin/cat \"$LTKOI_FIXTURES/unknown.json\"",
        "exit 28",
    ] {
        scratch.curl(script);
        let output = scratch.run(&[
            "collect-leetcode",
            "examplecoder",
            "--output",
            path.to_str().unwrap(),
        ]);
        assert_eq!(output.status.code(), Some(1));
        assert!(!output.stderr.is_empty());
        assert_eq!(fs::read_to_string(&path).unwrap(), "previous snapshot");
    }
}

#[test]
fn missing_curl_and_bad_usernames_produce_actionable_errors() {
    let scratch = Scratch::new();
    let output = scratch.run(&["collect-leetcode", "examplecoder"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("install curl"));
    let output = scratch.run(&["collect-leetcode", "bad/name"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("username"));
}

#[test]
fn command_help_and_invalid_arguments_do_not_start_requests() {
    let scratch = Scratch::new();
    let help = scratch.run(&["collect-leetcode", "--help"]);
    assert!(help.status.success());
    assert!(String::from_utf8_lossy(&help.stdout).contains("collect-leetcode"));
    for args in [
        vec!["collect-leetcode"],
        vec!["collect-leetcode", "a", "--output"],
        vec!["collect-leetcode", "a", "--bad", "file"],
        vec!["collect-leetcode", "--bad"],
    ] {
        assert_eq!(scratch.run(&args).status.code(), Some(2));
    }
}

#[test]
fn failed_output_write_leaves_no_partial_file() {
    let scratch = Scratch::new();
    scratch.curl(NORMAL);
    let path = scratch.0.join("missing/snapshot.json");
    let output = scratch.run(&[
        "collect-leetcode",
        "examplecoder",
        "--output",
        path.to_str().unwrap(),
    ]);
    assert_eq!(output.status.code(), Some(1));
    assert!(!path.exists());
}
