use anyhow::{ensure, Context, Result};
use bijux_dna_infra::{atomic_write_json, ensure_dir};
use bijux_dna_policies::{
    evaluate_no_wait_gate, is_governed_no_wait_source, NoWaitAllowlist, NoWaitSource, NoWaitVerdict,
};

use crate::commands::command_support::{fail, pass, run_command};
use crate::model::check::{CheckDefinition, CheckOutcome};
use crate::runtime::workspace::Workspace;

const ALLOWLIST_PATH: &str = "configs/ci/no_wait_allowlist.toml";
const RECEIPT_PATH: &str = "artifacts/architecture/no-wait-gate.json";

pub(crate) fn check_no_wait_gate(
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    let source_commit = git_text(workspace, &["rev-parse", "HEAD"])?;
    let sources = tracked_sources(workspace)?;
    let allowlist_raw = std::fs::read_to_string(workspace.path(ALLOWLIST_PATH))
        .with_context(|| format!("read {ALLOWLIST_PATH}"))?;
    let allowlist: NoWaitAllowlist =
        toml::from_str(&allowlist_raw).context("parse no-wait allowlist")?;
    let receipt = evaluate_no_wait_gate(&source_commit, &sources, &allowlist)?;

    ensure_dir(workspace.path("artifacts/architecture"))
        .context("create architecture artifact root")?;
    atomic_write_json(&workspace.path(RECEIPT_PATH), &receipt)
        .with_context(|| format!("write {RECEIPT_PATH}"))?;

    if receipt.verdict == NoWaitVerdict::Passed {
        return pass(
            check,
            format!(
                "scanned {} governed files at {}; wrote {RECEIPT_PATH}",
                receipt.summary.scanned_files, source_commit
            ),
        );
    }
    let mut details = receipt
        .violations
        .iter()
        .map(|finding| format!("{}:{}:{:?}", finding.path, finding.line, finding.rule))
        .collect::<Vec<_>>();
    details.extend(
        receipt
            .stale_allowlist_fingerprints
            .iter()
            .map(|fingerprint| format!("stale allowlist fingerprint {fingerprint}")),
    );
    fail(check, format!("no-wait gate failed; receipt={RECEIPT_PATH}\n{}", details.join("\n")))
}

fn tracked_sources(workspace: &Workspace) -> Result<Vec<NoWaitSource>> {
    let output = run_command(workspace, "git", &["ls-files", "-z"])?;
    ensure!(output.status.success(), "git ls-files failed");
    let mut sources = Vec::new();
    for raw_path in output.stdout.split(|byte| *byte == 0).filter(|path| !path.is_empty()) {
        let path = std::str::from_utf8(raw_path).context("tracked path is not UTF-8")?;
        if !is_governed_no_wait_source(path) {
            continue;
        }
        let content = std::fs::read_to_string(workspace.path(path))
            .with_context(|| format!("read governed source {path}"))?;
        sources.push(NoWaitSource { path: path.to_string(), content });
    }
    sources.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(sources)
}

fn git_text(workspace: &Workspace, args: &[&str]) -> Result<String> {
    let output = run_command(workspace, "git", args)?;
    ensure!(output.status.success(), "git {} failed", args.join(" "));
    String::from_utf8(output.stdout)
        .context("git output is not UTF-8")
        .map(|value| value.trim().to_string())
}
