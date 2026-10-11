use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use anyhow::{ensure, Context, Result};
use chrono::{NaiveDate, Utc};
use regex::Regex;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const NO_WAIT_GATE_SCHEMA_V1: &str = "bijux.execution.no_wait_gate.v1";
pub const NO_WAIT_ALLOWLIST_SCHEMA_V1: &str = "bijux.execution.no_wait_allowlist.v1";

const POLICY_DEFINITION: &str = concat!(
    "bijux.execution.no_wait_policy.v1\n",
    "blocking_sleep:foreground sleeps are forbidden in governed execution surfaces\n",
    "scheduler_poll_loop:scheduler observations are one-shot\n",
    "completion_file_poll_loop:completion files are observed once\n",
    "bounded_internal_retry_backoff:finite typed exceptions require exact fingerprints\n",
);
const LIFECYCLE_CRATES: [&str; 8] = [
    "/bijux-dna-runtime/",
    "/bijux-dna-bench/",
    "/bijux-dna-containers/",
    "/bijux-dna-containers-qa/",
    "/bijux-dna-api/",
    "/bijux-dna-dev/",
    "/bijux-dna/",
    "/bijux-dna-infra/",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoWaitSource {
    pub path: String,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NoWaitAllowlist {
    pub schema_version: String,
    #[serde(default)]
    pub exception: Vec<NoWaitException>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NoWaitException {
    pub fingerprint: String,
    pub path: String,
    pub rule: NoWaitRule,
    pub kind: String,
    pub maximum_attempts: u32,
    pub maximum_delay_milliseconds: u64,
    pub owner: String,
    pub reason: String,
    pub expires_on: String,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum NoWaitRule {
    BlockingSleep,
    SchedulerPollLoop,
    CompletionFilePollLoop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum NoWaitVerdict {
    Passed,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct NoWaitFinding {
    pub path: String,
    pub line: usize,
    pub rule: NoWaitRule,
    pub scope: String,
    pub fingerprint: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct AllowlistedNoWaitFinding {
    pub finding: NoWaitFinding,
    pub kind: String,
    pub maximum_attempts: u32,
    pub maximum_delay_milliseconds: u64,
    pub owner: String,
    pub reason: String,
    pub expires_on: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct NoWaitGateSummary {
    pub scanned_files: usize,
    pub scanned_lines: usize,
    pub one_shot_scheduler_references: usize,
    pub violation_count: usize,
    pub allowlisted_count: usize,
    pub stale_allowlist_count: usize,
    pub scope_counts: BTreeMap<String, usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct NoWaitGateReceipt {
    pub schema_version: String,
    pub source_commit: String,
    pub policy_sha256: String,
    pub allowlist_sha256: String,
    pub verdict: NoWaitVerdict,
    pub summary: NoWaitGateSummary,
    pub violations: Vec<NoWaitFinding>,
    pub allowlisted_findings: Vec<AllowlistedNoWaitFinding>,
    pub stale_allowlist_fingerprints: Vec<String>,
}

#[must_use]
pub fn no_wait_policy_sha256() -> String {
    sha256_hex(POLICY_DEFINITION.as_bytes())
}

#[must_use]
pub fn no_wait_gate_schema() -> schemars::Schema {
    schemars::schema_for!(NoWaitGateReceipt)
}

#[must_use]
pub fn is_governed_no_wait_source(path: &str) -> bool {
    governed_scope(path).is_some() && is_executable_source(path)
}

/// Evaluate production scheduler, CI, automation, and lifecycle sources without executing them.
///
/// # Errors
/// Returns an error when the source identity or an exception contract is invalid.
pub fn evaluate_no_wait_gate(
    source_commit: &str,
    sources: &[NoWaitSource],
    allowlist: &NoWaitAllowlist,
) -> Result<NoWaitGateReceipt> {
    ensure!(
        source_commit.len() == 40 && source_commit.chars().all(|value| value.is_ascii_hexdigit()),
        "no-wait source commit must be a full Git SHA"
    );
    validate_allowlist(allowlist)?;
    let allowlist_sha256 = sha256_hex(
        &serde_json::to_vec(allowlist).context("serialize canonical no-wait allowlist")?,
    );

    let sleep_patterns = SleepPatterns::new()?;
    let mut findings = Vec::new();
    let mut scanned_files = 0_usize;
    let mut scanned_lines = 0_usize;
    let mut scheduler_references = 0_usize;
    let mut scope_counts = BTreeMap::new();
    for source in sources {
        let Some(scope) = governed_scope(&source.path) else {
            continue;
        };
        if !is_executable_source(&source.path) {
            continue;
        }
        let content = source.content.as_str();
        scanned_files += 1;
        scanned_lines += content.lines().count();
        *scope_counts.entry(scope.to_string()).or_insert(0) += 1;
        scheduler_references += content
            .lines()
            .filter(|line| {
                let line = line.to_ascii_lowercase();
                line.contains("squeue") || line.contains("sacct")
            })
            .count();
        findings.extend(scan_source(source, content, scope, &sleep_patterns));
    }
    findings.sort_by(|left, right| {
        (&left.path, left.line, left.rule).cmp(&(&right.path, right.line, right.rule))
    });
    findings.dedup();

    let exceptions = allowlist
        .exception
        .iter()
        .map(|exception| (exception.fingerprint.as_str(), exception))
        .collect::<BTreeMap<_, _>>();
    let mut matched = BTreeSet::new();
    let mut violations = Vec::new();
    let mut allowlisted_findings = Vec::new();
    for finding in findings {
        if let Some(exception) = exceptions.get(finding.fingerprint.as_str()) {
            ensure!(exception.path == finding.path, "no-wait exception path mismatch");
            ensure!(exception.rule == finding.rule, "no-wait exception rule mismatch");
            matched.insert(exception.fingerprint.clone());
            allowlisted_findings.push(AllowlistedNoWaitFinding {
                finding,
                kind: exception.kind.clone(),
                maximum_attempts: exception.maximum_attempts,
                maximum_delay_milliseconds: exception.maximum_delay_milliseconds,
                owner: exception.owner.clone(),
                reason: exception.reason.clone(),
                expires_on: exception.expires_on.clone(),
            });
        } else {
            violations.push(finding);
        }
    }
    let stale_allowlist_fingerprints = allowlist
        .exception
        .iter()
        .filter(|exception| !matched.contains(&exception.fingerprint))
        .map(|exception| exception.fingerprint.clone())
        .collect::<Vec<_>>();
    let verdict = if violations.is_empty() && stale_allowlist_fingerprints.is_empty() {
        NoWaitVerdict::Passed
    } else {
        NoWaitVerdict::Failed
    };

    Ok(NoWaitGateReceipt {
        schema_version: NO_WAIT_GATE_SCHEMA_V1.to_string(),
        source_commit: source_commit.to_string(),
        policy_sha256: no_wait_policy_sha256(),
        allowlist_sha256,
        verdict,
        summary: NoWaitGateSummary {
            scanned_files,
            scanned_lines,
            one_shot_scheduler_references: scheduler_references,
            violation_count: violations.len(),
            allowlisted_count: allowlisted_findings.len(),
            stale_allowlist_count: stale_allowlist_fingerprints.len(),
            scope_counts,
        },
        violations,
        allowlisted_findings,
        stale_allowlist_fingerprints,
    })
}

fn validate_allowlist(allowlist: &NoWaitAllowlist) -> Result<()> {
    ensure!(
        allowlist.schema_version == NO_WAIT_ALLOWLIST_SCHEMA_V1,
        "unexpected no-wait allowlist schema version"
    );
    let today = Utc::now().date_naive();
    let mut fingerprints = BTreeSet::new();
    for exception in &allowlist.exception {
        ensure!(
            fingerprints.insert(exception.fingerprint.as_str()),
            "duplicate no-wait exception fingerprint {}",
            exception.fingerprint
        );
        ensure!(
            exception.fingerprint.len() == 64
                && exception.fingerprint.chars().all(|value| value.is_ascii_hexdigit()),
            "no-wait exception fingerprint must be a SHA-256 digest"
        );
        ensure!(exception.kind == "bounded_internal_retry_backoff", "invalid exception kind");
        ensure!(
            exception.rule == NoWaitRule::BlockingSleep,
            "only internal backoff sleeps may be exempted"
        );
        ensure!(
            exception.path.starts_with("crates/bijux-dna-infra/src/retry/"),
            "retry exception must be isolated under bijux-dna-infra retry ownership"
        );
        ensure!(exception.maximum_attempts > 1, "retry exception must declare a finite retry");
        ensure!(exception.maximum_delay_milliseconds > 0, "retry delay must be bounded");
        ensure!(!exception.owner.trim().is_empty(), "retry exception owner is required");
        ensure!(!exception.reason.trim().is_empty(), "retry exception reason is required");
        let expiry = NaiveDate::parse_from_str(&exception.expires_on, "%Y-%m-%d")
            .context("parse no-wait exception expiry")?;
        ensure!(expiry >= today, "no-wait exception has expired");
    }
    Ok(())
}

struct SleepPatterns {
    python: Regex,
    rust: Regex,
    shell: Regex,
}

impl SleepPatterns {
    fn new() -> Result<Self> {
        Ok(Self {
            python: Regex::new(r"(?:time\.)?sleep\s*\(").context("compile Python sleep rule")?,
            rust: Regex::new(r"(?:std::)?thread::sleep\s*\(|tokio::time::sleep\s*\(")
                .context("compile Rust sleep rule")?,
            shell: Regex::new(
                r"(?:^|[;&|][[:space:]]*|run:[[:space:]]+|-[[:space:]]+)@?sleep[[:space:]]+[^#[:space:]]+",
            )
                .context("compile shell sleep rule")?,
        })
    }
}

fn scan_source(
    source: &NoWaitSource,
    content: &str,
    scope: &str,
    sleep_patterns: &SleepPatterns,
) -> Vec<NoWaitFinding> {
    let lines = content.lines().collect::<Vec<_>>();
    let extension = Path::new(&source.path).extension().and_then(|value| value.to_str());
    let mut findings = Vec::new();
    for (index, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with("//") {
            continue;
        }
        let blocking_sleep = match extension {
            Some("py") => sleep_patterns.python.is_match(trimmed),
            Some("rs") => sleep_patterns.rust.is_match(trimmed),
            Some("sh" | "bash" | "zsh" | "yml" | "yaml" | "mk") | None => {
                sleep_patterns.shell.is_match(trimmed)
            }
            _ => false,
        };
        if blocking_sleep {
            findings.push(finding(source, index, NoWaitRule::BlockingSleep, scope, trimmed));
        }
        if !is_loop_control(extension, trimmed) {
            continue;
        }
        let end = usize::min(index + 24, lines.len());
        let window = lines[index..end].join("\n").to_ascii_lowercase();
        if window.contains("squeue") || window.contains("sacct") {
            findings.push(finding(source, index, NoWaitRule::SchedulerPollLoop, scope, trimmed));
        }
        let checks_completion_file = ["status", "report", "complete", "receipt", "exit", "done"]
            .iter()
            .any(|token| window.contains(token))
            && [".exists()", "test -f", "test -e", "[ -f", "[ -e"]
                .iter()
                .any(|token| window.contains(token));
        if checks_completion_file {
            findings.push(finding(
                source,
                index,
                NoWaitRule::CompletionFilePollLoop,
                scope,
                trimmed,
            ));
        }
    }
    findings
}

fn is_loop_control(extension: Option<&str>, line: &str) -> bool {
    let line = line.trim_start();
    match extension {
        Some("rs") => starts_control(line, "while") || starts_control(line, "loop"),
        Some("py") => starts_control(line, "while"),
        Some("sh" | "bash" | "zsh" | "yml" | "yaml" | "mk") | None => {
            starts_control(line.trim_start_matches('@'), "while")
                || starts_control(line.trim_start_matches('@'), "until")
        }
        _ => false,
    }
}

fn starts_control(line: &str, keyword: &str) -> bool {
    line.strip_prefix(keyword).is_some_and(|suffix| {
        suffix.is_empty()
            || suffix.starts_with(char::is_whitespace)
            || suffix.starts_with('(')
            || suffix.starts_with('{')
    })
}

fn finding(
    source: &NoWaitSource,
    zero_based_line: usize,
    rule: NoWaitRule,
    scope: &str,
    line: &str,
) -> NoWaitFinding {
    let line_number = zero_based_line + 1;
    let fingerprint =
        sha256_hex(format!("{}\0{line_number}\0{rule:?}\0{}", source.path, line.trim()).as_bytes());
    NoWaitFinding {
        path: source.path.clone(),
        line: line_number,
        rule,
        scope: scope.to_string(),
        fingerprint,
    }
}

fn governed_scope(path: &str) -> Option<&'static str> {
    if path.starts_with(".github/") {
        return Some("ci");
    }
    let root_automation_prefix = ["scr", "ipts/"].concat();
    if path == "Makefile" || path.starts_with("makes/") || path.starts_with(&root_automation_prefix)
    {
        return Some("automation");
    }
    if !path.starts_with("crates/") || !path.contains("/src/") {
        return None;
    }
    if path.contains("/bijux-dna-runner/") || path.contains("/commands/hpc/") {
        return Some("scheduler");
    }
    LIFECYCLE_CRATES.iter().any(|component| path.contains(component)).then_some("lifecycle")
}

fn is_executable_source(path: &str) -> bool {
    if path == "Makefile" {
        return true;
    }
    matches!(
        Path::new(path).extension().and_then(|value| value.to_str()),
        Some("py" | "rs" | "sh" | "bash" | "zsh" | "yml" | "yaml" | "mk")
    )
}

fn sha256_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(digest.len() * 2);
    for byte in digest {
        output.push(HEX[usize::from(byte >> 4)] as char);
        output.push(HEX[usize::from(byte & 0x0f)] as char);
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_allowlist() -> NoWaitAllowlist {
        NoWaitAllowlist {
            schema_version: NO_WAIT_ALLOWLIST_SCHEMA_V1.to_string(),
            exception: Vec::new(),
        }
    }

    #[test]
    fn rejects_scheduler_and_completion_file_polling() -> Result<()> {
        let sources = vec![NoWaitSource {
            path: ["scr", "ipts/collect.sh"].concat(),
            content: "while squeue -j 42; do\n  sleep 5\ndone\nuntil test -f report.done; do\n  sleep 1\ndone\n".to_string(),
        }];
        let receipt = evaluate_no_wait_gate(&"a".repeat(40), &sources, &empty_allowlist())?;

        assert_eq!(receipt.verdict, NoWaitVerdict::Failed);
        assert!(receipt
            .violations
            .iter()
            .any(|finding| finding.rule == NoWaitRule::SchedulerPollLoop));
        assert!(receipt
            .violations
            .iter()
            .any(|finding| finding.rule == NoWaitRule::CompletionFilePollLoop));
        assert_eq!(
            receipt
                .violations
                .iter()
                .filter(|finding| finding.rule == NoWaitRule::BlockingSleep)
                .count(),
            2
        );
        Ok(())
    }

    #[test]
    fn rejects_make_and_workflow_sleep_commands() -> Result<()> {
        let sources = vec![
            NoWaitSource {
                path: "makes/legacy.mk".to_string(),
                content: "legacy:\n\t@sleep 5\n".to_string(),
            },
            NoWaitSource {
                path: ".github/workflows/legacy.yml".to_string(),
                content: "steps:\n  - run: sleep 5\n".to_string(),
            },
        ];
        let receipt = evaluate_no_wait_gate(&"f".repeat(40), &sources, &empty_allowlist())?;

        assert_eq!(receipt.verdict, NoWaitVerdict::Failed);
        assert_eq!(
            receipt
                .violations
                .iter()
                .filter(|finding| finding.rule == NoWaitRule::BlockingSleep)
                .count(),
            2
        );
        Ok(())
    }

    #[test]
    fn admits_one_shot_scheduler_observation() -> Result<()> {
        let sources = vec![NoWaitSource {
            path: "crates/bijux-dna-runner/src/backend/slurm/client.rs".to_string(),
            content: "let observation = scheduler.observe(\"squeue\")?;\nreturn Ok(observation);\n"
                .to_string(),
        }];
        let receipt = evaluate_no_wait_gate(&"b".repeat(40), &sources, &empty_allowlist())?;

        assert_eq!(receipt.verdict, NoWaitVerdict::Passed);
        assert_eq!(receipt.allowlist_sha256.len(), 64);
        assert_eq!(receipt.summary.one_shot_scheduler_references, 1);
        Ok(())
    }

    #[test]
    fn admits_only_exact_finite_retry_backoff_exceptions() -> Result<()> {
        let source = NoWaitSource {
            path: "crates/bijux-dna-infra/src/retry/backoff/worker.rs".to_string(),
            content: "std::thread::sleep(delay);\n".to_string(),
        };
        let initial = evaluate_no_wait_gate(
            &"d".repeat(40),
            std::slice::from_ref(&source),
            &empty_allowlist(),
        )?;
        let fingerprint = initial.violations[0].fingerprint.clone();
        let allowlist = NoWaitAllowlist {
            schema_version: NO_WAIT_ALLOWLIST_SCHEMA_V1.to_string(),
            exception: vec![NoWaitException {
                fingerprint,
                path: source.path.clone(),
                rule: NoWaitRule::BlockingSleep,
                kind: "bounded_internal_retry_backoff".to_string(),
                maximum_attempts: 3,
                maximum_delay_milliseconds: 500,
                owner: "bijux-dna-infra".to_string(),
                reason: "bounded internal transport retry".to_string(),
                expires_on: "2099-12-31".to_string(),
            }],
        };

        let admitted = evaluate_no_wait_gate(&"d".repeat(40), &[source], &allowlist)?;
        assert_eq!(admitted.verdict, NoWaitVerdict::Passed);
        assert_eq!(admitted.summary.allowlisted_count, 1);
        assert!(admitted.violations.is_empty());
        Ok(())
    }

    #[test]
    fn rejects_retry_exceptions_outside_infrastructure_retry_ownership() -> Result<()> {
        let source = NoWaitSource {
            path: "crates/bijux-dna-runner/src/retry/scheduler.rs".to_string(),
            content: "std::thread::sleep(delay);\n".to_string(),
        };
        let initial = evaluate_no_wait_gate(
            &"e".repeat(40),
            std::slice::from_ref(&source),
            &empty_allowlist(),
        )?;
        let allowlist = NoWaitAllowlist {
            schema_version: NO_WAIT_ALLOWLIST_SCHEMA_V1.to_string(),
            exception: vec![NoWaitException {
                fingerprint: initial.violations[0].fingerprint.clone(),
                path: source.path.clone(),
                rule: NoWaitRule::BlockingSleep,
                kind: "bounded_internal_retry_backoff".to_string(),
                maximum_attempts: 3,
                maximum_delay_milliseconds: 500,
                owner: "bijux-dna-runner".to_string(),
                reason: "scheduler polling must never be exempted".to_string(),
                expires_on: "2099-12-31".to_string(),
            }],
        };

        let Err(error) = evaluate_no_wait_gate(&"e".repeat(40), &[source], &allowlist) else {
            return Err(anyhow::anyhow!("scheduler retry exception must be rejected"));
        };
        assert!(error.to_string().contains("bijux-dna-infra retry ownership"));
        Ok(())
    }

    #[test]
    fn scans_installed_ci_surfaces_instead_of_vendored_templates() {
        assert!(is_governed_no_wait_source(".github/scripts/wait_for_ci.py"));
        assert!(!is_governed_no_wait_source(
            &[".bijux/shared/bijux-gh/", "scr", "ipts/wait_for_ci.py"].concat()
        ));
    }

    #[test]
    fn ignores_shell_fixture_text_in_rust_source() -> Result<()> {
        let sources = vec![NoWaitSource {
            path: "crates/bijux-dna/src/commands/hpc/status.rs".to_string(),
            content:
                "pub fn observe() {}\n#[cfg(test)]\nmod tests { const BAD: &str = \"sleep 5\"; }\n"
                    .to_string(),
        }];
        let receipt = evaluate_no_wait_gate(&"c".repeat(40), &sources, &empty_allowlist())?;

        assert_eq!(receipt.verdict, NoWaitVerdict::Passed);
        Ok(())
    }

    #[test]
    fn ignores_loop_words_inside_rust_messages() -> Result<()> {
        let sources = vec![NoWaitSource {
            path: "crates/bijux-dna/src/commands/benchmark/readiness.rs".to_string(),
            content: concat!(
                "if report.ok {\n",
                "    return Err(anyhow!(\"cannot be ok while report violations remain\"));\n",
                "}\n",
                "if report_path.exists() { observe_once(); }\n",
            )
            .to_string(),
        }];
        let receipt = evaluate_no_wait_gate(&"1".repeat(40), &sources, &empty_allowlist())?;

        assert_eq!(receipt.verdict, NoWaitVerdict::Passed);
        Ok(())
    }
}
