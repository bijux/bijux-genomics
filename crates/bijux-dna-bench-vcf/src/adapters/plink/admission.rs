//! Owner: bijux-dna-bench-vcf
//! plink adapter admission contracts.
use super::{
    anyhow, artifact, BTreeSet, Path, RegistryToolContract, Result, VcfPlinkFamilyAdapterArtifact,
    VcfPlinkFamilyAdapterCommandStep, VcfPlinkFamilyAdapterRow,
};

pub(super) fn validate_required_inputs(
    repo_root: &Path,
    stage_id: &str,
    inputs: &[VcfPlinkFamilyAdapterArtifact],
) -> Result<()> {
    for input in inputs {
        let resolved = repo_root.join(&input.path);
        if !resolved.is_file() {
            return Err(anyhow!(
                "VCF plink-family adapter for `{stage_id}` is missing required input `{}` at `{}`",
                input.artifact_id,
                input.path
            ));
        }
    }
    Ok(())
}

pub(super) fn validate_command_steps(
    tool_id: &str,
    stage_id: &str,
    steps: &[VcfPlinkFamilyAdapterCommandStep],
) -> Result<()> {
    if steps.is_empty() {
        return Err(anyhow!(
            "VCF {tool_id} adapter row `{stage_id}` must declare at least one command step"
        ));
    }
    for step in steps {
        if step.argv.is_empty() {
            return Err(anyhow!(
                "VCF {tool_id} adapter step `{}` for `{stage_id}` has empty argv",
                step.step_id
            ));
        }
        if step.argv.iter().any(|part| {
            let lowered = part.to_ascii_lowercase();
            lowered.contains("placeholder") || lowered == "--help" || lowered.contains("todo")
        }) {
            return Err(anyhow!(
                "VCF {tool_id} adapter step `{}` for `{stage_id}` still contains placeholder argv: {:?}",
                step.step_id,
                step.argv
            ));
        }
        if !step.argv.iter().any(|part| part == "--out") {
            return Err(anyhow!(
                "VCF {tool_id} adapter step `{}` for `{stage_id}` is missing `--out`",
                step.step_id
            ));
        }
    }
    let joined =
        steps.iter().flat_map(|step| step.argv.iter().cloned()).collect::<Vec<_>>().join(" ");
    match (tool_id, stage_id) {
        ("plink" | "plink2", "vcf.qc") => {
            ensure_join_contains(stage_id, &joined, "--missing")?;
            ensure_join_contains(stage_id, &joined, "--freq")?;
            ensure_join_contains(stage_id, &joined, "--het")?;
            ensure_join_contains(stage_id, &joined, "--hardy")?;
        }
        ("plink", "vcf.admixture") => {
            ensure_join_contains(stage_id, &joined, "--make-bed")?;
            ensure_join_contains(stage_id, &joined, "--freq")?;
            ensure_join_contains(stage_id, &joined, "--missing")?;
        }
        ("plink", "vcf.population_structure") => {
            ensure_join_contains(stage_id, &joined, "--cluster")?;
            ensure_join_contains(stage_id, &joined, "--mds-plot")?;
        }
        ("plink2", "vcf.pca") => ensure_join_contains(stage_id, &joined, "--pca")?,
        ("plink2", "vcf.admixture") => {
            ensure_join_contains(stage_id, &joined, "--pca")?;
            ensure_join_contains(stage_id, &joined, " 2 ")?;
        }
        ("plink2", "vcf.population_structure") => {
            ensure_join_contains(stage_id, &joined, "--indep-pairwise")?;
            ensure_join_contains(stage_id, &joined, "--pca")?;
        }
        ("plink2", "vcf.roh") => ensure_join_contains(stage_id, &joined, "--homozyg")?,
        _ => {}
    }
    Ok(())
}

pub(super) fn ensure_join_contains(stage_id: &str, joined: &str, needle: &str) -> Result<()> {
    if joined.contains(needle) {
        Ok(())
    } else {
        Err(anyhow!(
            "VCF plink-family adapter row `{stage_id}` is missing required argv token `{needle}`: {joined}"
        ))
    }
}

pub(super) fn run_missing_input_probe(
    repo_root: &Path,
    stage_id: &str,
    inputs: &[VcfPlinkFamilyAdapterArtifact],
) -> (String, String, String, bool) {
    let mut mutated_inputs = inputs.to_vec();
    let probe = mutated_inputs.first().cloned().unwrap_or_else(|| {
        artifact(
            "missing_input",
            "unknown",
            "artifacts/benchmark-smoke/readiness/adapters/none.missing",
        )
    });
    if let Some(first) = mutated_inputs.first_mut() {
        first.path = format!(
            "artifacts/benchmark-smoke/readiness/adapters/probes/{stage_id}/{}.missing",
            first.artifact_id
        );
    }
    let expected_error_fragment = format!("required input `{}`", probe.artifact_id);
    let observed_error = match validate_required_inputs(repo_root, stage_id, &mutated_inputs) {
        Ok(()) => format!(
            "VCF plink-family adapter row `{stage_id}` unexpectedly accepted missing input `{}`",
            probe.artifact_id
        ),
        Err(error) => error.to_string(),
    };
    let passed = observed_error.contains(&expected_error_fragment);
    (probe.artifact_id, expected_error_fragment, observed_error, passed)
}

pub(super) fn ensure_vcf_plink_family_adapter_contract(
    registry_tool: &RegistryToolContract,
    rows: &[VcfPlinkFamilyAdapterRow],
) -> Result<()> {
    let expected_stage_ids =
        registry_tool.stage_ids.iter().map(String::as_str).collect::<BTreeSet<_>>();
    let observed_stage_ids = rows.iter().map(|row| row.stage_id.as_str()).collect::<BTreeSet<_>>();
    if observed_stage_ids != expected_stage_ids {
        return Err(anyhow!(
            "VCF {} adapter stage set drifted: expected {:?}, found {:?}",
            registry_tool.tool_id,
            expected_stage_ids,
            observed_stage_ids
        ));
    }
    for row in rows {
        if row.tool_status != registry_tool.tool_status {
            return Err(anyhow!(
                "VCF {} adapter row `{}` drifted from registry tool status `{}`: {}",
                registry_tool.tool_id,
                row.stage_id,
                registry_tool.tool_status,
                row.tool_status
            ));
        }
        if !row.argv_validation_passed {
            return Err(anyhow!(
                "VCF {} adapter row `{}` failed argv validation",
                registry_tool.tool_id,
                row.stage_id
            ));
        }
        if !row.missing_input_test_passed {
            return Err(anyhow!(
                "VCF {} adapter row `{}` failed missing-input validation: {}",
                registry_tool.tool_id,
                row.stage_id,
                row.missing_input_observed_error
            ));
        }
        if row.normalized_metrics_path.is_empty() {
            return Err(anyhow!(
                "VCF {} adapter row `{}` is missing normalized metrics mapping",
                registry_tool.tool_id,
                row.stage_id
            ));
        }
    }
    Ok(())
}
