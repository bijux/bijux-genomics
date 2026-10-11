//! Owner: bijux-dna-containers
//! Qualification registry acquisition of current and historical run observations.

use super::identity::{parse_run_directory, parse_run_id, validate_sha256};
use super::QualificationRunIdTombstone;
use anyhow::{bail, Context, Result};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct CurrentRunEvidence {
    schema_version: String,
    run_id: String,
    status: String,
    #[serde(flatten)]
    remaining: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct AttemptLedger {
    schema_version: String,
    attempts: Vec<AttemptLedgerRow>,
    summary: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct AttemptLedgerRow {
    run_id: String,
    tool: String,
    #[serde(flatten)]
    remaining: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SupplementalRunHistory {
    schema_version: String,
    identities: Vec<SupplementalRunIdentity>,
    #[serde(default)]
    tombstones: Vec<QualificationRunIdTombstone>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SupplementalRunIdentity {
    run_id: String,
    tool_id: String,
    terminal_state: String,
    sources: Vec<SupplementalRunIdentitySource>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SupplementalRunIdentitySource {
    uri: String,
    sha256: String,
}

pub(super) fn scan_current_runs(
    root: &Path,
    historical: &mut BTreeMap<u64, String>,
) -> Result<BTreeMap<String, u64>> {
    let mut current = BTreeMap::new();
    for entry in
        fs::read_dir(root).with_context(|| format!("read current runs {}", root.display()))?
    {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        let Some((number, tool)) = parse_run_directory(&name)? else {
            continue;
        };
        let evidence_path = entry.path().join("evidence.json");
        if !evidence_path.is_file() {
            continue;
        }
        let evidence: CurrentRunEvidence = serde_json::from_slice(&fs::read(&evidence_path)?)
            .with_context(|| format!("parse final run evidence {}", evidence_path.display()))?;
        let _ = evidence.remaining;
        if !matches!(
            evidence.schema_version.as_str(),
            "bijux.container.evidence.v1" | "bijux.container.evidence.v2"
        ) {
            bail!("checked-in final run {name} has an unsupported evidence schema");
        }
        if evidence.run_id != format!("run-{number}") {
            bail!("checked-in final run {name} disagrees with its evidence run ID");
        }
        if evidence.status != "passed" {
            bail!("checked-in final run {name} is not passed");
        }
        insert_historical(historical, number, tool)?;
        if current.insert(tool.to_string(), number).is_some() {
            bail!("multiple checked-in final runs exist for tool {tool}");
        }
    }
    Ok(current)
}

pub(super) fn scan_attempt_ledger(
    path: &Path,
    historical: &mut BTreeMap<u64, String>,
) -> Result<()> {
    let ledger: AttemptLedger = serde_json::from_slice(&fs::read(path)?)
        .with_context(|| format!("parse qualification attempt ledger {}", path.display()))?;
    if ledger.schema_version != "bijux.container.run_attempt_ledger.v1" {
        bail!("unsupported qualification attempt ledger schema");
    }
    let _ = ledger.summary;
    for row in ledger.attempts {
        let number = parse_run_id(&row.run_id)?;
        if row.tool.trim().is_empty() {
            bail!("qualification attempt ledger contains an empty tool ID");
        }
        let _ = row.remaining;
        insert_historical(historical, number, &row.tool)?;
    }
    Ok(())
}

pub(super) fn scan_supplemental_history(
    path: &Path,
    historical: &mut BTreeMap<u64, String>,
) -> Result<Vec<QualificationRunIdTombstone>> {
    let history: SupplementalRunHistory = toml::from_slice(&fs::read(path)?)
        .with_context(|| format!("parse supplemental qualification history {}", path.display()))?;
    if history.schema_version != "bijux.container.qualification_run_history.v2" {
        bail!("unsupported supplemental qualification history schema");
    }
    let mut seen_identities = BTreeSet::new();
    for identity in history.identities {
        let number = parse_run_id(&identity.run_id)?;
        if identity.tool_id.trim().is_empty() || identity.terminal_state != "submission_failed" {
            bail!("supplemental qualification identity is malformed");
        }
        if !seen_identities.insert(number) {
            bail!("supplemental qualification history contains a duplicate run ID");
        }
        if identity.sources.is_empty() {
            bail!("supplemental qualification identity has no immutable sources");
        }
        let expected_path_segment = format!("/{}-{}/", identity.run_id, identity.tool_id);
        let mut seen_sources = BTreeSet::new();
        for source in identity.sources {
            if !source.uri.starts_with("study://")
                || !source.uri.contains(&expected_path_segment)
                || !seen_sources.insert(source.uri.clone())
            {
                bail!("supplemental qualification source identity is malformed");
            }
            validate_sha256(&source.sha256, "supplemental qualification source")?;
        }
        insert_historical(historical, number, &identity.tool_id)?;
    }
    let mut seen_tombstones = BTreeSet::new();
    for tombstone in &history.tombstones {
        let number = parse_run_id(&tombstone.run_id)?;
        if !(1000..2000).contains(&number)
            || tombstone.disposition != "reserved_never_reuse"
            || tombstone.reason.trim().is_empty()
            || !seen_tombstones.insert(number)
        {
            bail!("supplemental qualification run-ID tombstone is malformed");
        }
        if historical.contains_key(&number) {
            bail!("supplemental qualification run-ID tombstone conflicts with a real run");
        }
    }
    Ok(history.tombstones)
}

fn insert_historical(
    historical: &mut BTreeMap<u64, String>,
    number: u64,
    tool: &str,
) -> Result<()> {
    if let Some(existing) = historical.insert(number, tool.to_string()) {
        if existing != tool {
            bail!("historical run-{number} is assigned to both {existing} and {tool}");
        }
    }
    Ok(())
}
