use super::{anyhow, Context, FastqInvariantsReport, Result};

#[derive(Debug, Clone, Copy)]
struct FastqCoverageThresholds {
    gl_max: f64,
    pseudohaploid_max: f64,
    diploid_min: f64,
}

pub(super) fn maybe_write_fastq_coverage_classifier(
    root: &std::path::Path,
    invariants: &FastqInvariantsReport,
) -> Result<()> {
    let expected_genome_size_bp =
        std::env::var("BIJUX_EXPECTED_GENOME_SIZE_BP").ok().and_then(|v| v.parse::<u64>().ok());
    let Some(genome_bp) = expected_genome_size_bp else {
        return Ok(());
    };
    if genome_bp == 0 {
        return Ok(());
    }
    let reads = invariants.r1.read_count + invariants.r2.as_ref().map_or(0, |r2| r2.read_count);
    let mean_len = if let Some(r2) = invariants.r2.as_ref() {
        f64::midpoint(invariants.r1.read_length_mean, r2.read_length_mean)
    } else {
        invariants.r1.read_length_mean
    };
    let estimated_depth_x = (reads.to_string().parse::<f64>().unwrap_or(0.0) * mean_len)
        / genome_bp.to_string().parse::<f64>().unwrap_or(1.0);
    let thresholds = load_coverage_thresholds_for_fastq("default")?;
    let (selected_regime, trigger) = if estimated_depth_x <= thresholds.gl_max {
        (
            "gl",
            format!(
                "estimated_depth_x <= gl_max_depth ({estimated_depth_x:.4} <= {})",
                thresholds.gl_max
            ),
        )
    } else if estimated_depth_x <= thresholds.pseudohaploid_max {
        (
            "pseudohaploid",
            format!(
                "gl_max_depth < estimated_depth_x <= pseudohaploid_max_depth ({} < {estimated_depth_x:.4} <= {})",
                thresholds.gl_max, thresholds.pseudohaploid_max
            ),
        )
    } else if estimated_depth_x >= thresholds.diploid_min {
        (
            "diploid",
            format!(
                "estimated_depth_x >= diploid_min_depth ({estimated_depth_x:.4} >= {})",
                thresholds.diploid_min
            ),
        )
    } else {
        (
            "pseudohaploid",
            "fallback band between pseudohaploid_max_depth and diploid_min_depth".to_string(),
        )
    };
    let payload = serde_json::json!({
        "schema_version": "bijux.fastq.coverage_classifier.v1",
        "expected_genome_size_bp": genome_bp,
        "reads": reads,
        "mean_read_length": mean_len,
        "estimated_depth_x": estimated_depth_x,
        "selected_regime": selected_regime,
        "trigger": trigger,
        "thresholds_used": {
            "gl_max_depth": thresholds.gl_max,
            "pseudohaploid_max_depth": thresholds.pseudohaploid_max,
            "diploid_min_depth": thresholds.diploid_min,
        }
    });
    bijux_dna_infra::atomic_write_json(&root.join("coverage_regime.fastq.json"), &payload)
        .context("write coverage_regime.fastq.json")?;
    bijux_dna_infra::atomic_write_json(&root.join("coverage_explain.fastq.json"), &payload)
        .context("write coverage_explain.fastq.json")
}

fn load_coverage_thresholds_for_fastq(profile: &str) -> Result<FastqCoverageThresholds> {
    let root = crate::support::workspace::resolve_repo_root()?;
    let raw = std::fs::read_to_string(root.join("configs/runtime/coverage_regimes.toml"))?;
    let parsed: toml::Value = toml::from_str(&raw)?;
    let decision = parsed
        .get("decision")
        .and_then(|v| v.get("coverage_regime"))
        .ok_or_else(|| anyhow!("missing decision.coverage_regime"))?;
    let base = decision
        .get("thresholds")
        .ok_or_else(|| anyhow!("missing decision.coverage_regime.thresholds"))?;
    let profile_thresholds = decision.get("profiles").and_then(|v| v.get(profile)).unwrap_or(base);
    let read_f = |key: &str| -> Result<f64> {
        profile_thresholds
            .get(key)
            .and_then(toml::Value::as_float)
            .or_else(|| {
                profile_thresholds
                    .get(key)
                    .and_then(toml::Value::as_integer)
                    .and_then(|v| v.to_string().parse::<f64>().ok())
            })
            .ok_or_else(|| anyhow!("missing threshold key `{key}`"))
    };
    Ok(FastqCoverageThresholds {
        gl_max: read_f("gl_max_depth")?,
        pseudohaploid_max: read_f("pseudohaploid_max_depth")?,
        diploid_min: read_f("diploid_min_depth")?,
    })
}
