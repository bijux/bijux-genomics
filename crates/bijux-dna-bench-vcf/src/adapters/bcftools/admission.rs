//! Owner: bijux-dna-bench-vcf
//! bcftools adapter admission contracts.
use super::{
    anyhow, artifact, BTreeSet, Path, Result, VcfBcftoolsAdapterArtifact,
    VcfBcftoolsAdapterCommandStep, VcfBcftoolsAdapterRow, GOVERNED_BCFTOOLS_TOOL_ID,
};

pub(super) fn validate_required_inputs(
    repo_root: &Path,
    stage_id: &str,
    inputs: &[VcfBcftoolsAdapterArtifact],
) -> Result<()> {
    crate::adapters::inputs::validate_required_files(
        repo_root,
        stage_id,
        crate::adapters::inputs::InputContext::Adapter { family: "bcftools", descent: false },
        inputs.iter().map(|input| (input.artifact_id.as_str(), input.path.as_str())),
    )
}

pub(super) fn validate_command_steps(
    stage_id: &str,
    steps: &[VcfBcftoolsAdapterCommandStep],
) -> Result<()> {
    let mut has_bcftools_entrypoint = false;
    for step in steps {
        if step.argv.is_empty() {
            return Err(anyhow!(
                "VCF bcftools adapter step `{}` for `{stage_id}` has empty argv",
                step.step_id
            ));
        }
        if step.argv[0] == GOVERNED_BCFTOOLS_TOOL_ID
            || (stage_id == "vcf.qc"
                && matches!(step.argv.first().map(String::as_str), Some("sh" | "bash"))
                && step.argv.iter().any(|part| part.contains("bcftools query")))
        {
            has_bcftools_entrypoint = true;
        } else if stage_id != "vcf.qc" {
            return Err(anyhow!(
                "VCF bcftools adapter step `{}` for `{stage_id}` must execute `{}` first, found `{}`",
                step.step_id,
                GOVERNED_BCFTOOLS_TOOL_ID,
                step.argv[0]
            ));
        }
        if step.argv.iter().any(|part| {
            let lowered = part.to_ascii_lowercase();
            lowered.contains("placeholder") || lowered == "--help" || lowered.contains("todo")
        }) {
            return Err(anyhow!(
                "VCF bcftools adapter step `{}` for `{stage_id}` still contains placeholder argv: {:?}",
                step.step_id,
                step.argv
            ));
        }
    }
    if !has_bcftools_entrypoint {
        return Err(anyhow!(
            "VCF bcftools adapter stage `{stage_id}` must retain at least one concrete bcftools command step"
        ));
    }
    Ok(())
}

pub(super) fn run_missing_input_probe(
    repo_root: &Path,
    stage_id: &str,
    inputs: &[VcfBcftoolsAdapterArtifact],
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
            "VCF bcftools adapter row `{stage_id}` unexpectedly accepted missing input `{}`",
            probe.artifact_id
        ),
        Err(error) => error.to_string(),
    };
    let passed = observed_error.contains(&expected_error_fragment);
    (probe.artifact_id, expected_error_fragment, observed_error, passed)
}

pub(super) fn ensure_vcf_bcftools_adapter_contract(rows: &[VcfBcftoolsAdapterRow]) -> Result<()> {
    if rows.len() != 11 {
        return Err(anyhow!(
            "VCF bcftools adapter must cover exactly 11 governed benchmark bindings, found {}",
            rows.len()
        ));
    }
    let expected_stages = BTreeSet::from([
        "vcf.prepare_reference_panel",
        "vcf.call",
        "vcf.call_gl",
        "vcf.call_diploid",
        "vcf.call_pseudohaploid",
        "vcf.damage_filter",
        "vcf.filter",
        "vcf.gl_propagation",
        "vcf.postprocess",
        "vcf.qc",
        "vcf.stats",
    ]);
    let observed_stages = rows.iter().map(|row| row.stage_id.as_str()).collect::<BTreeSet<_>>();
    if observed_stages != expected_stages {
        return Err(anyhow!(
            "VCF bcftools adapter stage set drifted: expected {expected_stages:?}, found {observed_stages:?}"
        ));
    }
    for row in rows {
        if !row.argv_validation_passed {
            return Err(anyhow!(
                "VCF bcftools adapter row `{}` failed argv validation",
                row.stage_id
            ));
        }
        if !row.missing_input_test_passed {
            return Err(anyhow!(
                "VCF bcftools adapter row `{}` failed missing-input validation: {}",
                row.stage_id,
                row.missing_input_observed_error
            ));
        }
    }
    Ok(())
}
