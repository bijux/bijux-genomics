#[derive(Debug, Clone)]
pub(super) struct PrimerGovernanceResult {
    marker_id: String,
    primer_set_id: String,
    expected_amplicon_min_bp: u64,
    expected_amplicon_max_bp: u64,
    primer_db_sha256: String,
    lock_path: PathBuf,
}

#[derive(Debug, Clone)]
pub(crate) struct PrimerSetGovernance {
    pub marker_id: String,
    pub primer_set_id: String,
    pub primer_fasta: PathBuf,
    pub primer_db_sha256: String,
}

#[derive(Debug, Clone)]
struct MergeDeterminismPolicy {
    min_overlap_bp: u64,
    max_mismatch_rate: f64,
}

fn reference_assets_root() -> Result<PathBuf> {
    if let Some(root) = std::env::var_os("BIJUX_REFERENCE_ROOT")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
    {
        return Ok(root);
    }
    Ok(crate::support::workspace::resolve_repo_root()?.join("assets/reference"))
}

fn governance_config_path() -> Result<PathBuf> {
    Ok(reference_assets_root()?.join("amplicon_governance.toml"))
}

fn read_f64_from_toml(value: Option<&toml::Value>, default: f64) -> f64 {
    value
        .and_then(toml::Value::as_float)
        .or_else(|| {
            value
                .and_then(toml::Value::as_integer)
                .and_then(|v| v.to_string().parse::<f64>().ok())
        })
        .unwrap_or(default)
}

fn read_u64_from_toml(value: Option<&toml::Value>, default: u64) -> u64 {
    value
        .and_then(toml::Value::as_integer)
        .and_then(|v| u64::try_from(v).ok())
        .unwrap_or(default)
}

fn load_amplicon_governance() -> Result<toml::Value> {
    let path = governance_config_path()?;
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))
}

fn read_marker_id() -> Result<String> {
    std::env::var("BIJUX_MARKER_ID")
        .ok()
        .filter(|v| !v.trim().is_empty())
        .ok_or_else(|| anyhow!("BIJUX_MARKER_ID must be declared for amplicon governance"))
}

fn primer_set_governance_for_marker(
    governance: &toml::Value,
    marker_id: &str,
) -> Result<PrimerSetGovernance> {
    let marker_cfg = governance
        .get("markers")
        .and_then(|v| v.get(marker_id))
        .ok_or_else(|| anyhow!("missing marker governance for `{marker_id}`"))?;
    let primer_set_id = marker_cfg
        .get("primer_set_id")
        .and_then(toml::Value::as_str)
        .unwrap_or("default")
        .to_string();
    let primer_fasta = marker_cfg
        .get("primer_fasta")
        .and_then(toml::Value::as_str)
        .ok_or_else(|| anyhow!("marker `{marker_id}` missing primer_fasta path"))?;
    let primer_path = reference_assets_root()?.join(primer_fasta);
    if !primer_path.exists() {
        return Err(anyhow!(
            "primer governance refusal: primer fasta missing for marker `{marker_id}`: {}",
            primer_path.display()
        ));
    }
    let primer_db_sha256 = bijux_dna_infra::hash_file_sha256(&primer_path)?;
    let primer_sha256_locked = marker_cfg
        .get("primer_sha256")
        .and_then(toml::Value::as_str)
        .ok_or_else(|| anyhow!("marker `{marker_id}` missing primer_sha256 lock"))?;
    if primer_db_sha256 != primer_sha256_locked {
        return Err(anyhow!(
            "primer governance refusal: checksum mismatch for marker `{marker_id}` (expected {primer_sha256_locked}, observed {primer_db_sha256})"
        ));
    }
    Ok(PrimerSetGovernance {
        marker_id: marker_id.to_string(),
        primer_set_id,
        primer_fasta: primer_path,
        primer_db_sha256,
    })
}

pub(crate) fn resolve_primer_set_governance(
    requested_primer_set_id: Option<&str>,
) -> Result<PrimerSetGovernance> {
    let governance = load_amplicon_governance()?;
    if let Some(requested_primer_set_id) = requested_primer_set_id
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        let markers = governance
            .get("markers")
            .and_then(toml::Value::as_table)
            .ok_or_else(|| anyhow!("missing markers section in amplicon_governance.toml"))?;
        for (marker_id, marker_cfg) in markers {
            if marker_cfg
                .get("primer_set_id")
                .and_then(toml::Value::as_str)
                .is_some_and(|candidate| candidate == requested_primer_set_id)
            {
                return primer_set_governance_for_marker(&governance, marker_id);
            }
        }
        return Err(anyhow!(
            "unknown primer_set_id `{requested_primer_set_id}` in amplicon governance"
        ));
    }
    primer_set_governance_for_marker(&governance, &read_marker_id()?)
}

pub(super) fn enforce_primer_governance(
    run_root: &std::path::Path,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqPreprocessArgs,
    invariants: &FastqInvariantsReport,
) -> Result<Option<PrimerGovernanceResult>> {
    if !matches!(
        args.mode,
        bijux_dna_planner_fastq::stage_api::args::FastqPlannerMode::EdnaAmplicon
            | bijux_dna_planner_fastq::stage_api::args::FastqPlannerMode::PollenAmplicon
    ) {
        return Ok(None);
    }
    let governance = load_amplicon_governance()?;
    let marker_id = read_marker_id()?;
    let primer = primer_set_governance_for_marker(&governance, &marker_id)?;
    let marker_cfg = governance
        .get("markers")
        .and_then(|v| v.get(&marker_id))
        .ok_or_else(|| anyhow!("missing marker governance for `{marker_id}`"))?;
    let expected_amplicon_min_bp =
        read_u64_from_toml(marker_cfg.get("expected_amplicon_min_bp"), 80);
    let expected_amplicon_max_bp =
        read_u64_from_toml(marker_cfg.get("expected_amplicon_max_bp"), 350);
    let lock_path = run_root.join("primer_db.lock.json");
    let lock_payload = serde_json::json!({
        "schema_version": "bijux.fastq.primer_db_lock.v1",
        "marker_id": marker_id,
        "primer_set_id": primer.primer_set_id,
        "primer_fasta": primer.primer_fasta,
        "primer_db_sha256": primer.primer_db_sha256,
        "expected_amplicon_min_bp": expected_amplicon_min_bp,
        "expected_amplicon_max_bp": expected_amplicon_max_bp
    });
    bijux_dna_infra::atomic_write_json(&lock_path, &lock_payload)?;

    let observed_mean = invariants.r1.read_length_mean;
    let expected_min = expected_amplicon_min_bp
        .to_string()
        .parse::<f64>()
        .unwrap_or(0.0);
    let expected_max = expected_amplicon_max_bp
        .to_string()
        .parse::<f64>()
        .unwrap_or(f64::MAX);
    let observed_ok = observed_mean >= expected_min && observed_mean <= expected_max;
    let priors_payload = serde_json::json!({
        "schema_version": "bijux.fastq.amplicon_length_priors.v1",
        "marker_id": marker_id,
        "observed_mean_bp": observed_mean,
        "expected_amplicon_min_bp": expected_amplicon_min_bp,
        "expected_amplicon_max_bp": expected_amplicon_max_bp,
        "pass": observed_ok
    });
    bijux_dna_infra::atomic_write_json(
        &run_root.join("amplicon_length_priors.json"),
        &priors_payload,
    )?;
    if !observed_ok {
        return Err(anyhow!(
            "amplicon length prior refusal: observed mean read length {observed_mean:.2} not in [{expected_amplicon_min_bp}, {expected_amplicon_max_bp}] for marker {marker_id}",
        ));
    }
    Ok(Some(PrimerGovernanceResult {
        marker_id,
        primer_set_id: primer.primer_set_id,
        expected_amplicon_min_bp,
        expected_amplicon_max_bp,
        primer_db_sha256: primer.primer_db_sha256,
        lock_path,
    }))
}

pub(super) fn write_reference_db_validation_artifact(
    run_root: &std::path::Path,
    contaminant_bank: Option<&serde_json::Value>,
    primer: Option<&PrimerGovernanceResult>,
) -> Result<()> {
    let governance = load_amplicon_governance()
        .unwrap_or_else(|_| toml::Value::Table(toml::map::Map::default()));
    let taxonomy_cfg = governance
        .get("taxonomy")
        .cloned()
        .unwrap_or(toml::Value::Table(toml::map::Map::default()));
    let db_path = taxonomy_cfg
        .get("db_path")
        .and_then(toml::Value::as_str)
        .map(|rel| reference_assets_root().map(|root| root.join(rel)))
        .transpose()?;
    let db_license = taxonomy_cfg
        .get("license")
        .and_then(toml::Value::as_str)
        .unwrap_or("unspecified");
    let db_sha256_locked = taxonomy_cfg
        .get("db_sha256")
        .and_then(toml::Value::as_str)
        .unwrap_or_default();
    let db_exists = db_path.as_ref().is_some_and(|p| p.exists());
    let db_sha256 = db_path
        .as_ref()
        .and_then(|p| bijux_dna_infra::hash_file_sha256(p).ok());
    let db_hash_match = db_sha256
        .as_ref()
        .is_some_and(|computed| !db_sha256_locked.is_empty() && computed == db_sha256_locked);
    let has_bank_meta = contaminant_bank
        .and_then(|v| v.get("hash"))
        .and_then(serde_json::Value::as_str)
        .is_some();
    let pass = db_exists
        && db_sha256.is_some()
        && db_license != "unspecified"
        && !db_sha256_locked.is_empty()
        && db_hash_match;
    let payload = serde_json::json!({
        "schema_version": "bijux.reference.db_validation.v1",
        "stage_id": "reference.db_validation",
        "taxonomy_db": {
            "path": db_path,
            "exists": db_exists,
            "sha256": db_sha256,
            "sha256_locked": db_sha256_locked,
            "sha256_match": db_hash_match,
            "license": db_license,
            "bank_hash_present": has_bank_meta
        },
        "primer_db": primer.map(|p| serde_json::json!({
            "marker_id": p.marker_id,
            "primer_set_id": p.primer_set_id,
            "sha256": p.primer_db_sha256,
            "expected_amplicon_min_bp": p.expected_amplicon_min_bp,
            "expected_amplicon_max_bp": p.expected_amplicon_max_bp,
            "lock_path": p.lock_path
        })),
        "pass": pass
    });
    bijux_dna_infra::atomic_write_json(&run_root.join("reference.db_validation.json"), &payload)?;
    if !pass {
        return Err(anyhow!(
            "reference.db_validation refusal: taxonomy DB lock/checksum/license validation failed"
        ));
    }
    Ok(())
}

pub(super) fn write_contamination_controls_report(
    run_root: &std::path::Path,
    sample_id: &str,
) -> Result<()> {
    let controls = std::env::var("BIJUX_NEGATIVE_CONTROL_SAMPLES")
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    let is_negative = controls.iter().any(|c| c == sample_id);
    let payload = serde_json::json!({
        "schema_version": "bijux.fastq.contamination_controls.v1",
        "sample_id": sample_id,
        "negative_controls": controls,
        "is_negative_control": is_negative
    });
    bijux_dna_infra::atomic_write_json(&run_root.join("contamination_controls.json"), &payload)?;
    Ok(())
}

pub(super) fn write_batch_effect_summary(
    run_root: &std::path::Path,
    sample_id: &str,
    invariants: &FastqInvariantsReport,
) -> Result<()> {
    let path = run_root.join("batch_effect_summary.json");
    let mut samples = if path.exists() {
        std::fs::read_to_string(&path)
            .ok()
            .and_then(|raw| serde_json::from_str::<serde_json::Value>(&raw).ok())
            .and_then(|json| json.get("samples").cloned())
            .and_then(|v| v.as_array().cloned())
            .unwrap_or_default()
    } else {
        Vec::new()
    };
    samples.push(serde_json::json!({
        "sample_id": sample_id,
        "read_count": invariants.r1.read_count,
        "mean_read_length": invariants.r1.read_length_mean,
        "quality_encoding": invariants.r1.quality_encoding,
    }));
    samples.sort_by(|a, b| {
        a.get("sample_id")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .cmp(
                b.get("sample_id")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or_default(),
            )
    });
    let mean_read_length = if samples.is_empty() {
        0.0
    } else {
        samples
            .iter()
            .filter_map(|s| {
                s.get("mean_read_length")
                    .and_then(serde_json::Value::as_f64)
            })
            .sum::<f64>()
            / samples.len().to_string().parse::<f64>().unwrap_or(1.0)
    };
    let payload = serde_json::json!({
        "schema_version": "bijux.fastq.batch_effect_summary.v1",
        "sample_count": samples.len(),
        "mean_read_length": mean_read_length,
        "samples": samples
    });
    bijux_dna_infra::atomic_write_json(&path, &payload)?;
    Ok(())
}

fn amplicon_merge_policy() -> Result<MergeDeterminismPolicy> {
    let governance = load_amplicon_governance()?;
    let merge = governance
        .get("merge_policy")
        .ok_or_else(|| anyhow!("missing merge_policy section in amplicon_governance.toml"))?;
    Ok(MergeDeterminismPolicy {
        min_overlap_bp: read_u64_from_toml(merge.get("min_overlap_bp"), 12),
        max_mismatch_rate: read_f64_from_toml(merge.get("max_mismatch_rate"), 0.10),
    })
}

pub(super) fn enforce_amplicon_merge_determinism(
    stage_root: &std::path::Path,
    mode: bijux_dna_planner_fastq::stage_api::args::FastqPlannerMode,
    execution: &StageResultV1,
) -> Result<()> {
    if !matches!(
        mode,
        bijux_dna_planner_fastq::stage_api::args::FastqPlannerMode::EdnaAmplicon
            | bijux_dna_planner_fastq::stage_api::args::FastqPlannerMode::PollenAmplicon
    ) {
        return Ok(());
    }
    let policy = amplicon_merge_policy()?;
    let merged = format!("{}\n{}", execution.stdout, execution.stderr);
    let overlap = parse_first_u64_after_key(&merged, "overlap").unwrap_or(policy.min_overlap_bp);
    let mismatch_pct = parse_first_u64_after_key(&merged, "mismatch")
        .map_or(0.0, |v| v.to_string().parse::<f64>().unwrap_or(0.0) / 100.0);
    let pass = overlap >= policy.min_overlap_bp && mismatch_pct <= policy.max_mismatch_rate;
    let payload = serde_json::json!({
        "schema_version": "bijux.fastq.amplicon_merge_determinism.v1",
        "min_overlap_bp_required": policy.min_overlap_bp,
        "max_mismatch_rate_allowed": policy.max_mismatch_rate,
        "observed_overlap_bp": overlap,
        "observed_mismatch_rate": mismatch_pct,
        "pass": pass
    });
    bijux_dna_infra::atomic_write_json(&stage_root.join("amplicon_merge_policy.json"), &payload)?;
    if !pass {
        return Err(anyhow!(
            "amplicon merge refusal: overlap/mismatch policy not satisfied (overlap={overlap}, mismatch_rate={mismatch_pct:.4})",
        ));
    }
    Ok(())
}

pub(super) fn write_edna_report_summary(
    run_root: &std::path::Path,
    stage_runs: &[StageExecutionSummary],
) -> Result<()> {
    let mut chimera_rate = None;
    let mut asv_rows = None;
    let mut otu_rows = None;
    let mut normalization = None;
    for stage in stage_runs {
        let stage_root = run_root.join(stage.plan.step_id.as_str());
        if stage.plan.step_id.as_str() == "fastq.remove_chimeras" {
            let report_path = stage.plan.out_dir.join("remove_chimeras_report.json");
            if let Ok(raw) = std::fs::read_to_string(&report_path) {
                if let Ok(report) =
                    bijux_dna_domain_fastq::observer::parse_remove_chimeras_report(&raw)
                {
                    chimera_rate = report.chimera_fraction;
                    continue;
                }
            }
            let path = stage_root.join("metrics.json");
            if let Ok(raw) = std::fs::read_to_string(path) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&raw) {
                    chimera_rate = json
                        .get("chimera_fraction")
                        .and_then(serde_json::Value::as_f64);
                }
            }
        }
        if stage.plan.step_id.as_str() == "fastq.infer_asvs" {
            let table = stage.plan.out_dir.join("asv_abundance.tsv");
            if let Ok(raw) = std::fs::read_to_string(table) {
                asv_rows =
                    Some(raw.lines().skip(1).filter(|l| !l.trim().is_empty()).count() as u64);
            }
        }
        if stage.plan.step_id.as_str() == "fastq.cluster_otus" {
            let report_path = stage.plan.out_dir.join("cluster_otus_report.json");
            if let Ok(raw) = std::fs::read_to_string(report_path) {
                if let Ok(report) =
                    bijux_dna_domain_fastq::observer::parse_cluster_otus_report(&raw)
                {
                    otu_rows = Some(report.otu_count);
                }
            }
        }
        if stage.plan.step_id.as_str() == "fastq.normalize_abundance" {
            let path = stage_root.join("normalize_abundance_report.json");
            if let Ok(raw) = std::fs::read_to_string(path) {
                normalization =
                    bijux_dna_domain_fastq::observer::parse_normalize_abundance_report(&raw)
                        .ok()
                        .map(|report| {
                            serde_json::json!({
                                "tool": report.tool_id,
                                "method": report.method,
                                "table_rows": report.table_rows,
                                "sample_count": report.sample_count,
                                "feature_count": report.feature_count,
                                "zero_fraction": report.zero_fraction,
                                "normalized_value_column": report.normalized_value_column,
                                "compositional_rule": report.compositional_rule,
                            })
                        });
            }
        }
    }
    let payload = serde_json::json!({
        "schema_version": "bijux.fastq.edna_summary.v1",
        "chimera_rate": chimera_rate,
        "asv_table_rows": asv_rows,
        "otu_table_rows": otu_rows,
        "normalize_abundance": normalization
    });
    bijux_dna_infra::atomic_write_json(&run_root.join("edna_summary.json"), &payload)?;
    Ok(())
}
use super::{
    anyhow, parse_first_u64_after_key, Context, FastqInvariantsReport, PathBuf, Result,
    StageExecutionSummary, StageResultV1,
};
