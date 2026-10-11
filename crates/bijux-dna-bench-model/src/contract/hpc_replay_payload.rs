//! Replay payload admission after transport integrity checks and before plaintext publication.
//! This projection checks replay structure and identity; domain result schemas own scientific meaning.
use crate::BundleSidecar;
use anyhow::{Context, Result};
use serde_json::Value;

/// An inspection result cannot certify execution or scientific completeness.
pub struct HpcReplayPayloadAdmission {
    results_complete: bool,
    code_complete: bool,
    appraiser_policy_ok: bool,
    issues: Vec<String>,
}

impl HpcReplayPayloadAdmission {
    #[must_use]
    pub fn results_complete(&self) -> bool {
        self.results_complete
    }
    #[must_use]
    pub fn code_complete(&self) -> bool {
        self.code_complete
    }
    #[must_use]
    pub fn appraiser_policy_ok(&self) -> bool {
        self.appraiser_policy_ok
    }
    #[must_use]
    pub fn admitted(&self) -> bool {
        self.issues.is_empty()
    }
    #[must_use]
    pub fn into_issues(self) -> Vec<String> {
        self.issues
    }
}

/// Decode each plaintext once, bind payloads to their transport metadata, and admit their pair.
/// # Errors
/// Returns malformed JSON errors. Well-formed but incompatible evidence returns diagnostic issues.
/// Callers must verify ciphertext/plaintext integrity through the transport owner first.
pub fn inspect_hpc_replay_payloads(
    results_meta: &BundleSidecar,
    results_bytes: &[u8],
    code_meta: &BundleSidecar,
    code_bytes: &[u8],
) -> Result<HpcReplayPayloadAdmission> {
    let results: Value =
        serde_json::from_slice(results_bytes).context("parse results replay payload")?;
    let code: Value = serde_json::from_slice(code_bytes).context("parse code replay payload")?;
    let mut result_issues = inspect_results(&results);
    let mut code_issues = inspect_code(&code);
    inspect_identity(&results, results_meta, "results", &mut result_issues);
    inspect_identity(&code, code_meta, "code", &mut code_issues);
    let same_pair = results_meta.campaign_id == code_meta.campaign_id
        && results_meta.domain == code_meta.domain
        && results_meta.stage == code_meta.stage
        && results_meta.tool == code_meta.tool
        && results_meta.sample == code_meta.sample
        && results_meta.planned_job_id == code_meta.planned_job_id
        && results_meta.scheduler_job_id == code_meta.scheduler_job_id
        && results_meta.submitted_at == code_meta.submitted_at
        && results.pointer("/job/job_name") == code.pointer("/job/job_name");
    if !same_pair {
        result_issues
            .push("results and code bundles belong to different campaign/job identities".into());
        code_issues.push("code bundle does not match the results campaign/job identity".into());
    }
    let appraiser_issues = inspect_appraiser_policy(&results);
    let admission = HpcReplayPayloadAdmission {
        results_complete: result_issues.is_empty(),
        code_complete: code_issues.is_empty(),
        appraiser_policy_ok: appraiser_issues.is_empty(),
        issues: result_issues.into_iter().chain(code_issues).chain(appraiser_issues).collect(),
    };
    Ok(admission)
}

fn inspect_identity(payload: &Value, meta: &BundleSidecar, kind: &str, issues: &mut Vec<String>) {
    let expected_schema = format!("bijux.hpc.{kind}_bundle.v1");
    if payload.get("schema_version").and_then(Value::as_str) != Some(expected_schema.as_str()) {
        issues.push(format!("unsupported {kind} payload schema"));
    }
    if meta.bundle_kind != kind {
        issues.push(format!("{kind} transport declares a different bundle kind"));
    }
    for (path, expected) in [
        ("/campaign/id", &meta.campaign_id),
        ("/campaign/domain", &meta.domain),
        ("/job/stage", &meta.stage),
        ("/job/tool", &meta.tool),
        ("/job/sample", &meta.sample),
        ("/job/planned_job_id", &meta.planned_job_id),
        ("/job/scheduler_job_id", &meta.scheduler_job_id),
        ("/job/submitted_at", &meta.submitted_at),
    ] {
        if expected.trim().is_empty()
            || payload.pointer(path).and_then(Value::as_str) != Some(expected.as_str())
        {
            issues.push(format!(
                "{kind} payload identity `{path}` does not match transport metadata"
            ));
        }
    }
    require(payload, "job.job_name", kind, nonempty_string, issues);
}

fn field<'a>(payload: &'a Value, path: &str) -> Option<&'a Value> {
    path.split('.').try_fold(payload, |node, part| node.get(part))
}
fn nonempty_string(value: &Value) -> bool {
    value.as_str().is_some_and(|s| !s.trim().is_empty())
}
fn nonempty_object(value: &Value) -> bool {
    value.as_object().is_some_and(|rows| !rows.is_empty())
}
fn object_rows(value: &Value) -> bool {
    value.as_array().is_some_and(|rows| rows.iter().all(nonempty_object))
}
fn require(
    payload: &Value,
    path: &str,
    kind: &str,
    accepts: fn(&Value) -> bool,
    issues: &mut Vec<String>,
) {
    match field(payload, path) {
        None => issues.push(format!("missing {kind} field `{path}`")),
        Some(value) if !accepts(value) => issues.push(format!("invalid {kind} field `{path}`")),
        Some(_) => {}
    }
}
fn inspect_results(payload: &Value) -> Vec<String> {
    let mut issues = Vec::new();
    require(payload, "metrics", "results", nonempty_object, &mut issues);
    for path in ["artifacts.inventory", "reports", "appraiser_outputs"] {
        require(payload, path, "results", object_rows, &mut issues);
    }
    if let Some(rows) = field(payload, "artifacts.inventory").and_then(Value::as_array) {
        for row in rows {
            for key in ["kind", "path"] {
                if !row.get(key).is_some_and(nonempty_string) {
                    issues.push(format!("invalid results artifact inventory `{key}`"));
                }
            }
        }
    }
    issues
}
fn inspect_code(payload: &Value) -> Vec<String> {
    let mut issues = Vec::new();
    for path in ["code_freeze.repository_state", "code_freeze.config_references", "plan.outputs"] {
        require(payload, path, "code", nonempty_object, &mut issues);
    }
    for path in
        ["code_freeze.slurm_script", "locks.corpus_lock", "locks.database_lock", "locks.image_lock"]
    {
        require(payload, path, "code", nonempty_string, &mut issues);
    }
    for name in ["scratch_dir", "log", "out", "err", "results", "code"] {
        require(payload, &format!("plan.outputs.{name}"), "code", nonempty_string, &mut issues);
    }
    issues
}
fn inspect_appraiser_policy(payload: &Value) -> Vec<String> {
    let mut issues = Vec::new();
    match field(payload, "appraiser_outputs") {
        None => issues.push("results payload lacks `appraiser_outputs`".into()),
        Some(value) if !object_rows(value) => {
            issues.push("results payload has invalid `appraiser_outputs`".into());
        }
        Some(_) => {}
    }
    if field(payload, "artifacts.inventory").and_then(Value::as_array).is_some_and(|rows| {
        rows.iter().any(|row| row.get("kind").and_then(Value::as_str) == Some("appraiser_output"))
    }) {
        issues.push("appraiser output must not be listed as plaintext artifact".into());
    }
    issues
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::anyhow;
    use bijux_dna_core::id_catalog as stage;
    use serde_json::json;
    fn fixture() -> Result<(BundleSidecar, Value, BundleSidecar, Value)> {
        let base = json!({"schema_version":crate::BUNDLE_SIDECAR_SCHEMA_VERSION,"bundle_kind":"results",
            "campaign_id":"campaign","domain":"fastq","stage":stage::FASTQ_FILTER,"tool":"tool","sample":"sample",
            "planned_job_id":"job","scheduler_job_id":"mock-1","submitted_at":"123",
            "backend":"mock-envelope-v1","recipients":["recipient"],"recipient_fingerprints":["identity"],
            "ciphertext_path":"job.results","plaintext_sha256":"0".repeat(64),"ciphertext_sha256":"0".repeat(64),"plaintext_bytes":0});
        let results_meta: BundleSidecar = serde_json::from_value(base)?;
        let mut code_meta = results_meta.clone();
        code_meta.bundle_kind = "code".into();
        code_meta.ciphertext_path = "job.code".into();
        let identity = json!({"campaign":{"id":"campaign","domain":"fastq"},"job":{"stage":stage::FASTQ_FILTER,"tool":"tool","sample":"sample","planned_job_id":"job","scheduler_job_id":"mock-1","submitted_at":"123","job_name":"job"}});
        let mut results = identity.clone();
        results["schema_version"] = json!("bijux.hpc.results_bundle.v1");
        results["metrics"] = json!({"status":"pending_execution"});
        results["artifacts"] = json!({"inventory":[{"kind":"log","path":"job.log"}]});
        results["reports"] = json!([{"kind":"submission_receipt"}]);
        results["appraiser_outputs"] = json!([]);
        let mut code = identity;
        code["schema_version"] = json!("bijux.hpc.code_bundle.v1");
        code["code_freeze"] = json!({"repository_state":{"git_head":null},"config_references":{"campaign_config":"campaign.toml"},"slurm_script":"#!/bin/sh\ntrue\n"});
        code["locks"] = json!({"corpus_lock":"corpus.lock","database_lock":"db.lock","image_lock":"images.lock"});
        code["plan"] = json!({"outputs":{"scratch_dir":"scratch","log":"job.log","out":"job.out","err":"job.err","results":"job.results","code":"job.code"}});
        Ok((results_meta, results, code_meta, code))
    }
    #[test]
    fn replay_rejects_null_wrong_shapes_unknown_versions_and_identity_substitution() -> Result<()> {
        let (results_meta, results, code_meta, code) = fixture()?;
        let inspect = |r: &Value, c: &Value| {
            inspect_hpc_replay_payloads(
                &results_meta,
                &serde_json::to_vec(r)?,
                &code_meta,
                &serde_json::to_vec(c)?,
            )
        };
        assert!(inspect(&results, &code)?.admitted());
        for path in [
            "/metrics",
            "/artifacts/inventory",
            "/reports",
            "/appraiser_outputs",
            "/job/stage",
            "/campaign/id",
        ] {
            for value in [Value::Null, json!(5), json!("foreign")] {
                let mut bad = results.clone();
                *bad.pointer_mut(path).ok_or_else(|| anyhow!("missing fixture field"))? = value;
                assert!(!inspect(&bad, &code)?.admitted(), "{path}");
            }
        }
        for path in [
            "/code_freeze/repository_state",
            "/code_freeze/config_references",
            "/code_freeze/slurm_script",
            "/locks/corpus_lock",
            "/plan/outputs",
            "/plan/outputs/results",
        ] {
            let mut bad = code.clone();
            *bad.pointer_mut(path).ok_or_else(|| anyhow!("missing fixture field"))? = Value::Null;
            assert!(!inspect(&results, &bad)?.admitted(), "{path}");
        }
        let mut bad = code.clone();
        bad["schema_version"] = json!("unknown");
        assert!(!inspect(&results, &bad)?.admitted());
        let mut other_meta = code_meta.clone();
        other_meta.campaign_id = "other".into();
        let mut other_code = code.clone();
        other_code["campaign"]["id"] = json!("other");
        let mismatch = inspect_hpc_replay_payloads(
            &results_meta,
            &serde_json::to_vec(&results)?,
            &other_meta,
            &serde_json::to_vec(&other_code)?,
        )?;
        assert!(!mismatch.results_complete() && !mismatch.code_complete());
        let mut bad = results.clone();
        bad["artifacts"]["inventory"][0]["kind"] = json!("appraiser_output");
        assert!(!inspect(&bad, &code)?.appraiser_policy_ok());
        assert!(inspect_hpc_replay_payloads(
            &results_meta,
            b"{",
            &code_meta,
            &serde_json::to_vec(&code)?
        )
        .is_err());
        Ok(())
    }
}
