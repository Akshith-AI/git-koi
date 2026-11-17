use std::io::Read;
use std::process::{Command, Stdio};

use serde_json::{Value, json};

use super::LeetCodeError;

/// The upstream website endpoint; not a versioned public API contract.
pub const LEETCODE_ENDPOINT: &str = "https://leetcode.com/graphql/";

/// An injectable transport returning an unmodified GraphQL response envelope.
pub trait LeetCodeClient {
    /// Execute a query with JSON variables. No session cookies are required.
    fn query(&self, query: &str, variables: &Value) -> Result<Value, LeetCodeError>;
}

/// Credential-free HTTPS transport using the locally installed curl executable.
///
/// Requests have bounded time and response size. No third-party proxy is used.
pub struct CurlLeetCodeClient;

impl LeetCodeClient for CurlLeetCodeClient {
    fn query(&self, query: &str, variables: &Value) -> Result<Value, LeetCodeError> {
        let mut child = Command::new("curl")
            .args([
                "--disable", "--silent", "--show-error", "--fail", "--get",
                "--proto", "=https", "--connect-timeout", "10", "--max-time", "30",
                "--max-filesize", "2097152", "--url", LEETCODE_ENDPOINT,
                "--data-urlencode",
            ])
            .arg(format!("query={query}"))
            .arg("--data-urlencode")
            .arg(format!("variables={variables}"))
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|_| LeetCodeError::Request("could not start curl; install curl first".into()))?;
        let mut bytes = Vec::new();
        let read_result = child.stdout.take().expect("piped stdout")
            .take(2_097_153).read_to_end(&mut bytes);
        if read_result.is_err() || bytes.len() > 2_097_152 {
            let _ = child.kill();
            let _ = child.wait();
            return Err(LeetCodeError::Request("response unreadable or larger than 2 MiB".into()));
        }
        let status = child.wait().map_err(|_| LeetCodeError::Request("could not wait for curl".into()))?;
        if !status.success() {
            return Err(LeetCodeError::Request(format!("curl exited with {status}; check connectivity or retry later")));
        }
        serde_json::from_slice(&bytes)
            .map_err(|_| LeetCodeError::Request("endpoint did not return valid JSON".into()))
    }
}

pub(super) fn variables(username: &str) -> Value {
    json!({"username": username})
}

pub(super) fn data(response: &Value) -> Result<&Value, LeetCodeError> {
    if let Some(errors) = response.get("errors") {
        let errors = errors.as_array().ok_or_else(|| LeetCodeError::InvalidData("errors must be an array".into()))?;
        if !errors.is_empty() {
            return Err(LeetCodeError::Request("endpoint returned GraphQL errors".into()));
        }
    }
    response.get("data").filter(|v| v.is_object())
        .ok_or_else(|| LeetCodeError::InvalidData("missing response data".into()))
}
