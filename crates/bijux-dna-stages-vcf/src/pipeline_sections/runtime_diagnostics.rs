fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn license_metadata_for_tool_exists(tool_id: &str) -> bool {
    workspace_root()
        .join("containers/licenses")
        .join(format!("{tool_id}.license.toml"))
        .exists()
}

fn resolve_tool_digest(tool_id: &str) -> Result<String> {
    let registry = workspace_root().join("configs/ci/registry/tool_registry_vcf_downstream.toml");
    let raw = std::fs::read_to_string(registry)?;
    let mut current_tool_id: Option<String> = None;
    let mut pinned_commit: Option<String> = None;
    let mut container_ref: Option<String> = None;
    let mut version: Option<String> = None;
    let flush_if_match = |current_tool_id: &Option<String>,
                          pinned_commit: &Option<String>,
                          container_ref: &Option<String>,
                          version: &Option<String>|
     -> Option<String> {
        if current_tool_id.as_deref() != Some(tool_id) {
            return None;
        }
        let digest_source = format!(
            "{}|{}|{}|{}",
            tool_id,
            pinned_commit.as_deref().unwrap_or("planned"),
            container_ref.as_deref().unwrap_or("registry_lock"),
            version.as_deref().unwrap_or("planned")
        );
        Some(format!("sha256:{}", checksum_hex(digest_source.as_bytes())))
    };
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed == "[[tools]]" {
            if let Some(found) =
                flush_if_match(&current_tool_id, &pinned_commit, &container_ref, &version)
            {
                return Ok(found);
            }
            current_tool_id = None;
            pinned_commit = None;
            container_ref = None;
            version = None;
            continue;
        }
        if let Some(value) = trimmed.strip_prefix("tool_id = ") {
            current_tool_id = Some(value.trim_matches('"').to_string());
        } else if let Some(value) = trimmed.strip_prefix("pinned_commit = ") {
            pinned_commit = Some(value.trim_matches('"').to_string());
        } else if let Some(value) = trimmed.strip_prefix("container_ref = ") {
            container_ref = Some(value.trim_matches('"').to_string());
        } else if let Some(value) = trimmed.strip_prefix("version = ") {
            version = Some(value.trim_matches('"').to_string());
        }
    }
    if let Some(found) = flush_if_match(&current_tool_id, &pinned_commit, &container_ref, &version)
    {
        return Ok(found);
    }
    bail!("could not resolve tool digest source for {tool_id}");
}

fn parse_format_index(fields: &[&str], name: &str) -> Option<usize> {
    fields
        .get(8)?
        .split(':')
        .enumerate()
        .find_map(|(idx, key)| if key == name { Some(idx) } else { None })
}

fn parse_threshold_value(raw: &str, key: &str) -> Option<f64> {
    raw.lines().find_map(|line| {
        let trimmed = line.trim();
        if trimmed.starts_with('#') || trimmed.is_empty() {
            return None;
        }
        let (lhs, rhs) = trimmed.split_once(':')?;
        if lhs.trim() != key {
            return None;
        }
        rhs.trim().parse::<f64>().ok()
    })
}

fn load_imputation_qc_thresholds() -> std::collections::BTreeMap<String, f64> {
    let raw = std::fs::read_to_string(workspace_root().join("assets/reference/qc_thresholds.yaml"))
        .unwrap_or_default();
    let mut out = std::collections::BTreeMap::new();
    let defaults = [
        ("vcf_imputation_info_warn", 0.75_f64),
        ("vcf_imputation_info_fail", 0.60_f64),
        ("vcf_rsq_warn", 0.70_f64),
        ("vcf_rsq_fail", 0.55_f64),
        ("vcf_missingness_post_warn", 0.08_f64),
        ("vcf_missingness_post_fail", 0.15_f64),
        ("vcf_variant_density_warn", 2.0_f64),
        ("vcf_variant_density_fail", 1.0_f64),
        ("vcf_missingness_block_warn", 3.0_f64),
        ("vcf_missingness_block_fail", 6.0_f64),
    ];
    for (key, fallback) in defaults {
        out.insert(
            key.to_string(),
            parse_threshold_value(&raw, key).unwrap_or(fallback),
        );
    }
    out
}

fn cleanup_policy() -> String {
    std::env::var("BIJUX_STAGE_CLEANUP_POLICY")
        .unwrap_or_else(|_| "keep".to_string())
        .to_ascii_lowercase()
}

fn backend_error_hint(stage_id: &str, backend: &str, err: &anyhow::Error) -> (&'static str, String) {
    let msg = err.to_string();
    if msg.contains("contig") || msg.contains("SpeciesContext") {
        return (
            "species_context_mismatch",
            "verify species/build/contig digest and input VCF contig namespace before rerun"
                .to_string(),
        );
    }
    if msg.contains("requires map") || msg.contains("map ") {
        return (
            "map_prerequisite_missing",
            format!("backend `{backend}` requires map compatibility; validate map_id + map locks"),
        );
    }
    if msg.contains("license") {
        return (
            "license_policy_block",
            format!("backend `{backend}` blocked by license metadata policy; add/update license metadata"),
        );
    }
    if msg.contains("ploidy") || msg.contains("GT") || msg.contains("GL/GP") {
        return (
            "input_field_contract_violation",
            format!("backend `{backend}` input field/ploidy contract failed; fix caller inputs (no silent coercions)"),
        );
    }
    if stage_id == "vcf.impute" && msg.contains("imputation_accept") {
        return (
            "qc_acceptance_failed",
            "imputation QC thresholds failed; inspect imputation_qc.json and decision.imputation_accept"
                .to_string(),
        );
    }
    (
        "backend_execution_error",
        format!("inspect stage logs/manifests and rerun with the same backend `{backend}` deterministically"),
    )
}

fn write_crash_provenance_artifact(
    out_dir: &Path,
    stage_id: &str,
    backend: &str,
    input_vcf: &Path,
    err: &anyhow::Error,
) -> Result<PathBuf> {
    bijux_dna_infra::ensure_dir(out_dir)?;
    let path = out_dir.join("crash_provenance.json");
    let (category, hint) = backend_error_hint(stage_id, backend, err);
    let err_text = format!("{err:#}");
    let stderr_tail = {
        let chars = err_text.chars().collect::<Vec<_>>();
        let keep = 800usize;
        if chars.len() <= keep {
            err_text
        } else {
            chars[chars.len() - keep..].iter().collect::<String>()
        }
    };
    let digest = resolve_tool_digest(backend).unwrap_or_else(|_| "unknown".to_string());
    let payload = serde_json::json!({
        "schema_version": "bijux.vcf.crash_provenance.v1",
        "stage_id": stage_id,
        "backend": backend,
        "error_category": category,
        "actionable_hint": hint,
        "command": serde_json::Value::Null,
        "stderr_tail": stderr_tail,
        "inputs": {
            "input_vcf": input_vcf,
        },
        "env_summary": {
            "cleanup_policy": cleanup_policy(),
            "hostname": std::env::var("HOSTNAME").ok(),
        },
        "tool_digest": digest,
    });
    atomic_write_json(&path, &payload)?;
    Ok(path)
}

fn apply_failure_cleanup_policy(out_dir: &Path) {
    if cleanup_policy() != "prune" {
        return;
    }
    for rel in ["tmp", "chunks", "intermediate", "scratch"] {
        let candidate = out_dir.join(rel);
        if candidate.exists() {
            let _ = bijux_dna_infra::remove_dir_all(&candidate);
        }
    }
}

/// # Errors
/// Returns an error if phasing prerequisites or species/map policies are violated.
pub fn run_phasing_stage(
    input_vcf: &Path,
    out_dir: &Path,
    species_context: &SpeciesContext,
    params: &PhasingStageParams,
) -> Result<PhasingStageOutputs> {
    match run_phasing_stage_inner(input_vcf, out_dir, species_context, params) {
        Ok(out) => Ok(out),
        Err(err) => {
            let _ = write_crash_provenance_artifact(
                out_dir,
                "vcf.phasing",
                params.backend.as_str(),
                input_vcf,
                &err,
            );
            apply_failure_cleanup_policy(out_dir);
            let (_, hint) = backend_error_hint("vcf.phasing", params.backend.as_str(), &err);
            Err(anyhow!("{err}; backend hint: {hint}"))
        }
    }
}
