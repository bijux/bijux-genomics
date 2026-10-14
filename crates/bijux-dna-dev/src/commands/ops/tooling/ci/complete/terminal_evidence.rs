//! Owned terminal errors preserve diagnostics without entering qualified case acceptance.

use std::path::{Path, PathBuf};

use anyhow::{ensure, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{
    reference, ArtifactReference, LaneDefinition, LaneReceipt, LocalVerificationPlan,
    SourceIdentity, LANE_SCHEMA,
};

const ERROR_SCHEMA: &str = "bijux.ci.local_test_lane_error.v1";

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct LaneErrorReceipt {
    schema_version: String,
    lane: String,
    source: SourceIdentity,
    state: String,
    all_passed: bool,
    native_evidence_root: PathBuf,
    pub(super) execution_error: String,
    elapsed_milliseconds: u64,
    #[serde(default)]
    unqualified_native_evidence: Vec<UnqualifiedArtifact>,
    #[serde(default)]
    evidence_retention_error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct UnqualifiedArtifact {
    artifact: ArtifactReference,
    qualification: String,
    #[serde(default)]
    native_schema: Option<String>,
    #[serde(default)]
    schema_observation_error: Option<String>,
}

pub(super) struct TerminalEvidence {
    pub(super) receipt: ArtifactReference,
    pub(super) terminal: LaneTerminal,
}

pub(super) enum LaneTerminal {
    Outcomes(Box<LaneReceipt>),
    Error(Box<LaneErrorReceipt>),
}

#[derive(Deserialize)]
struct SchemaHeader {
    schema_version: String,
}

fn regular_owned(path: &Path, owner: &Path) -> Result<()> {
    ensure!(
        owner.is_absolute() && owner.is_dir() && owner.canonicalize()? == owner,
        "terminal evidence owner is missing or redirected"
    );
    ensure!(
        path.starts_with(owner)
            && path.symlink_metadata()?.is_file()
            && path.canonicalize()? == path,
        "terminal evidence path is missing, foreign or redirected"
    );
    Ok(())
}

fn evidence_paths(lane: &LaneDefinition, owner: &Path) -> Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    if matches!(lane.harness.as_str(), "pytest" | "unittest") {
        paths.extend([owner.join("inventory.json"), owner.join("cases.json")]);
    }
    for stage in ["metadata", "discovery", "execution"] {
        for stream in ["stdout", "stderr"] {
            paths.push(owner.join(format!("{stage}.{stream}.log")));
        }
    }
    if lane.harness == "rustdoc" {
        paths.push(owner.join("producer-admission.json"));
    }
    if lane.harness == "nextest" {
        paths.push(owner.join("ci/full-gate-timing.json"));
        let reports = owner.join("ci/full-shards");
        if reports.try_exists()? {
            ensure!(
                reports.is_dir() && reports.canonicalize()? == reports,
                "native report directory is redirected"
            );
            for entry in std::fs::read_dir(reports)? {
                let path = entry?.path();
                if path
                    .extension()
                    .is_some_and(|extension| extension == "json" || extension == "log")
                {
                    paths.push(path);
                }
            }
        }
    }
    paths.sort();
    paths.dedup();
    Ok(paths)
}

fn capture_unqualified(
    lane: &LaneDefinition,
    plan: &LocalVerificationPlan,
    owner: &Path,
) -> Result<Vec<UnqualifiedArtifact>> {
    evidence_paths(lane, owner)?.into_iter().filter_map(|path| {
        match path.try_exists() {
            Ok(false) => None,
            state => Some((|| -> Result<_> {
                state?;
                regular_owned(&path, owner)?;
                let artifact = reference(&path)?;
                let (native_schema, schema_observation_error) = if path.extension().is_some_and(|ext| ext == "json") {
                    let observation = (|| -> Result<Option<String>> {
                        let bytes = std::fs::read(&path)?;
                        ensure!(bijux_dna_infra::hash_bytes_sha256(&bytes) == artifact.sha256,
                            "unqualified evidence changed while captured");
                        let value: Value = serde_json::from_slice(&bytes)?;
                        let schema = value["schema_version"].as_str().map(str::to_string);
                        if path == owner.join("inventory.json") || path == owner.join("cases.json") {
                            ensure!(schema.as_deref() == Some("bijux.ci.python_cases.v1")
                                && value["source_root"] == plan.workspace.display().to_string()
                                && value["harness"] == lane.harness
                                && value["discovery_only"] == (path == owner.join("inventory.json")),
                                "Python evidence schema/source/harness/discovery role is unqualified");
                        }
                        Ok(schema)
                    })();
                    match observation {
                        Ok(schema) => (schema, None),
                        Err(error) => (None, Some(format!("{error:#}"))),
                    }
                } else { (None, None) };
                Ok(UnqualifiedArtifact { artifact, qualification: "unqualified".into(), native_schema,
                    schema_observation_error })
            })()),
        }
    }).collect()
}

pub(super) fn error_receipt(
    lane: &LaneDefinition,
    plan: &LocalVerificationPlan,
    owner: &Path,
    error: &anyhow::Error,
    elapsed_milliseconds: u64,
) -> LaneErrorReceipt {
    let (unqualified_native_evidence, evidence_retention_error) =
        match capture_unqualified(lane, plan, owner) {
            Ok(artifacts) => (artifacts, None),
            Err(error) => (Vec::new(), Some(format!("{error:#}"))),
        };
    LaneErrorReceipt {
        schema_version: ERROR_SCHEMA.into(),
        lane: lane.id.clone(),
        source: plan.source.clone(),
        state: "error".into(),
        all_passed: false,
        native_evidence_root: owner.to_path_buf(),
        execution_error: format!("{error:#}"),
        elapsed_milliseconds,
        unqualified_native_evidence,
        evidence_retention_error,
    }
}

pub(super) fn read_terminal(
    path: &Path,
    lane: &LaneDefinition,
    plan: &LocalVerificationPlan,
) -> Result<TerminalEvidence> {
    let owner = plan.output.join(&lane.id);
    ensure!(
        path == owner.join("receipt.json"),
        "terminal receipt does not belong to declared lane"
    );
    regular_owned(path, &owner)?;
    let bytes = std::fs::read(path)?;
    let header: SchemaHeader =
        serde_json::from_slice(&bytes).context("terminal schema missing or malformed")?;
    let terminal = match header.schema_version.as_str() {
        LANE_SCHEMA => {
            let receipt: LaneReceipt = serde_json::from_slice(&bytes)?;
            ensure!(
                receipt.lane == lane.id && receipt.inputs.lane == lane.id,
                "case receipt belongs to another declared lane"
            );
            LaneTerminal::Outcomes(Box::new(receipt))
        }
        ERROR_SCHEMA => {
            let error: LaneErrorReceipt = serde_json::from_slice(&bytes)?;
            ensure!(
                error.lane == lane.id
                    && error.source == plan.source
                    && error.native_evidence_root == owner
                    && error.state == "error"
                    && !error.all_passed
                    && !error.execution_error.trim().is_empty(),
                "terminal error has stale/foreign source, lane, owner or invalid failure status"
            );
            let permitted = evidence_paths(lane, &owner)?;
            let mut seen = std::collections::BTreeSet::new();
            for evidence in &error.unqualified_native_evidence {
                let path = Path::new(&evidence.artifact.path);
                ensure!(
                    evidence.qualification == "unqualified"
                        && permitted.iter().any(|item| item == path)
                        && seen.insert(path.to_path_buf()),
                    "terminal error carries undeclared or duplicate evidence"
                );
                regular_owned(path, &owner)?;
                ensure!(
                    reference(path)?.sha256 == evidence.artifact.sha256,
                    "terminal error evidence bytes changed"
                );
            }
            LaneTerminal::Error(Box::new(error))
        }
        _ => anyhow::bail!("lane has no supported terminal receipt: {}", header.schema_version),
    };
    let receipt = ArtifactReference {
        path: path.to_str().context("terminal path encoding")?.into(),
        sha256: bijux_dna_infra::hash_bytes_sha256(&bytes),
    };
    regular_owned(path, &owner)?;
    ensure!(reference(path)?.sha256 == receipt.sha256, "terminal receipt changed while decoded");
    Ok(TerminalEvidence { receipt, terminal })
}

#[cfg(test)]
mod tests {
    use super::super::write_value;
    use super::*;

    fn fixture() -> Result<(tempfile::TempDir, LocalVerificationPlan, LaneDefinition, PathBuf)> {
        let directory = tempfile::tempdir()?;
        let mut plan = super::super::execution::tests::plan(&directory.path().canonicalize()?);
        let mut lane = plan.lanes[0].clone();
        lane.id = "python-workflow".into();
        lane.harness = "unittest".into();
        lane.resource_units = 1;
        let owner = plan.output.join(&lane.id);
        std::fs::create_dir_all(&owner)?;
        plan.lanes.push(lane.clone());
        Ok((directory, plan, lane, owner))
    }

    fn failure(
        lane: &LaneDefinition,
        plan: &LocalVerificationPlan,
        owner: &Path,
    ) -> LaneErrorReceipt {
        error_receipt(
            lane,
            plan,
            owner,
            &anyhow::anyhow!("Python lane read an undeclared source input: .github"),
            10,
        )
    }

    #[test]
    fn discovered_python_evidence_survives_closure_failure_without_case_assurance() -> Result<()> {
        let (_directory, plan, lane, owner) = fixture()?;
        for (file, discovery) in [("inventory.json", true), ("cases.json", false)] {
            write_value(
                &owner.join(file),
                &serde_json::json!({
                    "schema_version":"bijux.ci.python_cases.v1","source_root":plan.workspace,
                    "harness":"unittest","discovery_only":discovery,"expected":["workflow.case"],
                    "outcomes":[{"id":"workflow.case","outcome":"passed"}],
                    "membership":{"all_passed":true}
                }),
            )?;
        }
        let error = failure(&lane, &plan, &owner);
        assert_eq!(error.unqualified_native_evidence.len(), 2);
        assert!(error
            .unqualified_native_evidence
            .iter()
            .all(|item| item.qualification == "unqualified"
                && item.schema_observation_error.is_none()));
        assert!(!error.all_passed);
        let path = owner.join("receipt.json");
        write_value(&path, &error)?;
        let LaneTerminal::Error(retained) = read_terminal(&path, &lane, &plan)?.terminal else {
            anyhow::bail!("failed input closure was admitted as case outcomes")
        };
        assert_eq!(retained.execution_error, error.execution_error);
        let value = serde_json::to_value(retained)?;
        assert!(value.get("expected").is_none() && value.get("outcomes").is_none());
        assert_eq!(value["all_passed"], false);
        Ok(())
    }

    #[test]
    fn unsupported_progress_stale_foreign_and_incomplete_errors_are_refused() -> Result<()> {
        let (_directory, plan, lane, owner) = fixture()?;
        let path = owner.join("receipt.json");
        let original = serde_json::to_value(failure(&lane, &plan, &owner))?;
        for (field, replacement) in [
            ("schema_version", serde_json::json!("bijux.ci.local_test_lane_progress.v1")),
            ("schema_version", serde_json::json!("bijux.ci.local_test_lane.v1")),
            ("state", serde_json::json!("running")),
            ("all_passed", serde_json::json!(true)),
            ("lane", serde_json::json!("rust-nextest")),
            ("native_evidence_root", serde_json::json!(plan.output.join("python-ordinary"))),
            (
                "source",
                serde_json::json!({"sha":"another-source","tree":"tree","patch_sha256":"patch"}),
            ),
            ("execution_error", serde_json::json!("")),
        ] {
            let mut value = original.clone();
            value[field] = replacement;
            write_value(&path, &value)?;
            assert!(read_terminal(&path, &lane, &plan).is_err(), "{field}");
        }
        let mut missing = original.clone();
        missing.as_object_mut().context("failure object")?.remove("execution_error");
        write_value(&path, &missing)?;
        assert!(read_terminal(&path, &lane, &plan).is_err());
        let mut unknown = original;
        unknown["invented_pass_count"] = serde_json::json!(1);
        write_value(&path, &unknown)?;
        assert!(read_terminal(&path, &lane, &plan).is_err());
        std::fs::remove_file(&path)?;
        assert!(read_terminal(&path, &lane, &plan).is_err());
        Ok(())
    }

    #[test]
    fn retained_raw_digest_duplicates_and_qualification_changes_are_refused() -> Result<()> {
        let (_directory, plan, lane, owner) = fixture()?;
        let artifact = owner.join("execution.stdout.log");
        std::fs::write(&artifact, b"native workflow execution passed; source closure failed")?;
        let mut error = failure(&lane, &plan, &owner);
        let path = owner.join("receipt.json");
        write_value(&path, &error)?;
        assert!(matches!(read_terminal(&path, &lane, &plan)?.terminal, LaneTerminal::Error(_)));
        std::fs::write(&artifact, b"changed")?;
        assert!(read_terminal(&path, &lane, &plan).is_err());
        std::fs::write(&artifact, b"native workflow execution passed; source closure failed")?;
        let duplicate: UnqualifiedArtifact =
            serde_json::from_value(serde_json::to_value(&error.unqualified_native_evidence[0])?)?;
        error.unqualified_native_evidence.push(duplicate);
        write_value(&path, &error)?;
        assert!(read_terminal(&path, &lane, &plan).is_err());
        error.unqualified_native_evidence.pop();
        error.unqualified_native_evidence[0].qualification = "qualified".into();
        write_value(&path, &error)?;
        assert!(read_terminal(&path, &lane, &plan).is_err());
        Ok(())
    }

    #[test]
    fn malformed_raw_json_is_retained_as_unqualified_foreign_root_is_refused() -> Result<()> {
        let (_directory, plan, lane, owner) = fixture()?;
        let artifact = owner.join("inventory.json");
        std::fs::write(&artifact, b"{truncated native discovery")?;
        let error = failure(&lane, &plan, &owner);
        assert_eq!(error.unqualified_native_evidence.len(), 1);
        assert!(error.unqualified_native_evidence[0].schema_observation_error.is_some());
        assert!(!error.all_passed);
        let path = owner.join("receipt.json");
        write_value(&path, &error)?;
        assert!(matches!(read_terminal(&path, &lane, &plan)?.terminal, LaneTerminal::Error(_)));
        let foreign = plan.output.join("python-ordinary");
        std::fs::create_dir_all(&foreign)?;
        let other = foreign.join("receipt.json");
        write_value(&other, &error)?;
        assert!(read_terminal(&other, &lane, &plan).is_err());
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn redirected_receipts_and_native_artifacts_cannot_be_owned_errors() -> Result<()> {
        use std::os::unix::fs::symlink;
        let (_directory, plan, lane, owner) = fixture()?;
        let artifact = owner.join("execution.stderr.log");
        std::fs::write(&artifact, b"native diagnostic")?;
        let path = owner.join("receipt.json");
        write_value(&path, &failure(&lane, &plan, &owner))?;
        let foreign = plan.output.join("outside-diagnostic");
        std::fs::write(&foreign, b"native diagnostic")?;
        std::fs::remove_file(&artifact)?;
        symlink(&foreign, &artifact)?;
        assert!(read_terminal(&path, &lane, &plan).is_err());
        std::fs::remove_file(&artifact)?;
        std::fs::write(&artifact, b"native diagnostic")?;
        let saved = owner.join("saved-receipt.json");
        std::fs::rename(&path, &saved)?;
        symlink(&saved, &path)?;
        assert!(read_terminal(&path, &lane, &plan).is_err());
        Ok(())
    }
}
