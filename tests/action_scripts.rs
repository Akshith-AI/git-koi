#![cfg(unix)]
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::PathBuf,
    process::{Command, Output},
    sync::atomic::{AtomicU64, Ordering},
};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!(
            "ltkoi-action-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(p.join("bin")).unwrap();
        Self(p)
    }
    fn stub(&self, name: &str, body: &str) {
        let p = self.0.join("bin").join(name);
        fs::write(&p, format!("#!/usr/bin/env bash\nset -eu\n{body}\n")).unwrap();
        fs::set_permissions(p, fs::Permissions::from_mode(0o755)).unwrap();
    }
    fn run(&self, script: &str, env: &[(&str, &str)]) -> Output {
        let mut c = Command::new("bash");
        c.arg(format!("{}/scripts/{script}", env!("CARGO_MANIFEST_DIR")))
            .current_dir(&self.0)
            .env(
                "PATH",
                format!(
                    "{}:{}",
                    self.0.join("bin").display(),
                    std::env::var("PATH").unwrap()
                ),
            )
            .env("RUNNER_TEMP", &self.0)
            .env("GITHUB_ACTION_PATH", env!("CARGO_MANIFEST_DIR"))
            .env("GITHUB_OUTPUT", self.0.join("outputs"))
            .env("GITHUB_PATH", self.0.join("paths"))
            .env("LTKOI_TEST_LOG", self.0.join("calls"));
        for &(name, value) in env {
            c.env(name, value);
        }
        c.output().unwrap()
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn cached_modes_do_not_collect_profiles_and_preserve_paths_with_spaces() {
    for (mode, command) in [
        ("betta", "animate-betta"),
        ("pond", "animate-pond"),
        ("koi", "render"),
    ] {
        let s = Scratch::new();
        s.stub("ltkoi", "printf '%s\\n' \"$@\" >> \"$LTKOI_TEST_LOG\"");
        let result = s.run(
            "run-action.sh",
            &[
                ("LTKOI_MODE", mode),
                ("LTKOI_OUTPUT_DIR", "my fish"),
                ("LTKOI_LEETCODE_SNAPSHOT", "saved profile.json"),
                ("LTKOI_GITHUB_SNAPSHOT", "github profile.json"),
            ],
        );
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        let calls = fs::read_to_string(s.0.join("calls")).unwrap();
        assert!(calls.starts_with(command));
        assert!(!calls.contains("collect"));
        assert!(calls.contains("my fish\n"));
        let output = fs::read_to_string(s.0.join("outputs")).unwrap();
        assert!(output.contains(&format!("gif-path=my fish/{mode}.gif")));
    }
}
#[test]
fn bad_modes_missing_usernames_and_unsafe_destinations_fail_before_running_cli() {
    for env in [
        vec![("LTKOI_MODE", "bad")],
        vec![("LTKOI_MODE", "pond")],
        vec![("LTKOI_MODE", "koi"), ("LTKOI_OUTPUT_DIR", "../escape")],
        vec![("LTKOI_MODE", "koi"), ("LTKOI_OUTPUT_DIR", ".git/refs")],
        vec![("LTKOI_MODE", "koi"), ("LTKOI_OUTPUT_DIR", "/absolute")],
    ] {
        let s = Scratch::new();
        s.stub("ltkoi", "touch \"$LTKOI_TEST_LOG\"");
        assert_eq!(s.run("run-action.sh", &env).status.code(), Some(2));
        assert!(!s.0.join("calls").exists());
    }
}
#[test]
fn failed_collection_does_not_publish_success_outputs() {
    let s = Scratch::new();
    s.stub("ltkoi", "exit 1");
    let result = s.run(
        "run-action.sh",
        &[
            ("LTKOI_MODE", "betta"),
            ("LTKOI_LEETCODE_USERNAME", "ExampleCoder"),
        ],
    );
    assert!(!result.status.success());
    assert!(!s.0.join("outputs").exists());
}
#[test]
fn release_installer_checks_actual_archive_hash_and_uses_the_fork() {
    let s = Scratch::new();
    let version = env!("CARGO_PKG_VERSION");
    let stem = format!("ltkoi-{version}-aarch64-apple-darwin");
    let assets = s.0.join("assets");
    fs::create_dir_all(assets.join(&stem)).unwrap();
    fs::copy(
        env!("CARGO_BIN_EXE_ltkoi"),
        assets.join(&stem).join("ltkoi"),
    )
    .unwrap();
    let archive = format!("{stem}.tar.gz");
    assert!(
        Command::new("tar")
            .current_dir(&assets)
            .args(["-czf", &archive, &stem])
            .status()
            .unwrap()
            .success()
    );
    let hash = Command::new("shasum")
        .current_dir(&assets)
        .args(["-a", "256", &archive])
        .output()
        .unwrap();
    assert!(hash.status.success());
    fs::write(assets.join(format!("{archive}.sha256")), hash.stdout).unwrap();
    s.stub(
        "gh",
        r#"destination=''
while [[ $# -gt 0 ]]; do
  case "$1" in
    --repo) [[ "$2" = 'https://github.com/Akshith-AI/git-koi' ]] || exit 9; shift 2 ;;
    --dir) destination=$2; shift 2 ;;
    *) shift ;;
  esac
done
cp "$LTKOI_TEST_ASSETS"/*.tar.gz "$LTKOI_TEST_ASSETS"/*.sha256 "$destination/"
"#,
    );
    let env = [
        ("RUNNER_OS", "macOS"),
        ("RUNNER_ARCH", "ARM64"),
        ("LTKOI_TEST_ASSETS", assets.to_str().unwrap()),
    ];
    let result = s.run("install-release.sh", &env);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let paths = fs::read_to_string(s.0.join("paths")).unwrap();
    let binary = PathBuf::from(paths.trim()).join("ltkoi");
    let version_output = Command::new(binary).arg("--version").output().unwrap();
    assert!(version_output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&version_output.stdout),
        format!("ltkoi {version}\n")
    );
    fs::remove_file(s.0.join("paths")).unwrap();
    fs::write(assets.join(&archive), "corrupted archive").unwrap();
    assert!(!s.run("install-release.sh", &env).status.success());
    assert!(!s.0.join("paths").exists());
}
