//! Owner: bijux-dna-bench-vcf
//! angsd adapter admission contracts.
use super::{
    anyhow, artifact, BTreeSet, Path, Result, VcfAngsdAdapterArtifact, VcfAngsdAdapterCommandStep,
    VcfAngsdAdapterRow, GOVERNED_ANGSD_TOOL_ID, GOVERNED_ANGSD_TOOL_STATUS,
};

pub(super) fn validate_required_inputs(
    repo_root: &Path,
    stage_id: &str,
    inputs: &[VcfAngsdAdapterArtifact],
) -> Result<()> {
    for input in inputs {
        let resolved = repo_root.join(&input.path);
        if !resolved.is_file() {
            return Err(anyhow!(
                "VCF angsd adapter for `{stage_id}` is missing required input `{}` at `{}`",
                input.artifact_id,
                input.path
            ));
        }
    }
    Ok(())
}

pub(super) fn validate_command_steps(
    stage_id: &str,
    steps: &[VcfAngsdAdapterCommandStep],
) -> Result<()> {
    for step in steps {
        if step.argv.is_empty() {
            return Err(anyhow!(
                "VCF angsd adapter step `{}` for `{stage_id}` has empty argv",
                step.step_id
            ));
        }
        if step.argv.iter().any(|part| {
            let lowered = part.to_ascii_lowercase();
            lowered.contains("placeholder") || lowered == "--help" || lowered.contains("todo")
        }) {
            return Err(anyhow!(
                "VCF angsd adapter step `{}` for `{stage_id}` still contains placeholder argv: {:?}",
                step.step_id,
                step.argv
            ));
        }
    }

    let call_step = steps
        .first()
        .ok_or_else(|| anyhow!("VCF angsd adapter row `{stage_id}` has no call step"))?;
    if call_step.argv.first().map(String::as_str) != Some(GOVERNED_ANGSD_TOOL_ID) {
        return Err(anyhow!(
            "VCF angsd adapter call step for `{stage_id}` must execute `{GOVERNED_ANGSD_TOOL_ID}`"
        ));
    }
    ensure_arg_present(stage_id, &call_step.argv, "-out")?;
    let normalizer_step = steps.get(1).ok_or_else(|| {
        anyhow!("VCF angsd adapter row `{stage_id}` is missing its result normalizer step")
    })?;
    if normalizer_step.step_kind != "result_normalizer"
        || normalizer_step.argv.first().map(String::as_str) != Some("bcftools")
        || normalizer_step.argv.get(1).map(String::as_str) != Some("view")
        || !normalizer_step.argv.iter().any(|part| part == "-Oz")
    {
        return Err(anyhow!(
            "VCF angsd adapter row `{stage_id}` must normalize the native VCF with `bcftools view -Oz`"
        ));
    }
    let index_step = steps
        .get(2)
        .ok_or_else(|| anyhow!("VCF angsd adapter row `{stage_id}` is missing its index step"))?;
    if index_step.step_kind != "index"
        || index_step.argv.first().map(String::as_str) != Some("bcftools")
        || index_step.argv.get(1).map(String::as_str) != Some("index")
        || !index_step.argv.iter().any(|part| part == "-t")
    {
        return Err(anyhow!(
            "VCF angsd adapter row `{stage_id}` must index the normalized VCF with `bcftools index -t`"
        ));
    }

    let argv = &call_step.argv;
    match stage_id {
        "vcf.call_gl" => {
            ensure_arg_present(stage_id, argv, "-b")?;
            ensure_arg_present(stage_id, argv, "-ref")?;
            ensure_arg_present(stage_id, argv, "-sites")?;
            ensure_arg_present(stage_id, argv, "-GL")?;
            ensure_arg_present(stage_id, argv, "-doGlf")?;
        }
        "vcf.call_pseudohaploid" => {
            ensure_arg_present(stage_id, argv, "-b")?;
            ensure_arg_present(stage_id, argv, "-ref")?;
            ensure_arg_present(stage_id, argv, "-sites")?;
            ensure_arg_present(stage_id, argv, "-doHaploCall")?;
            ensure_arg_present(stage_id, argv, "-seed")?;
        }
        _ => {}
    }
    Ok(())
}

pub(super) fn ensure_arg_present(stage_id: &str, argv: &[String], needle: &str) -> Result<()> {
    if argv.iter().any(|part| part == needle) {
        Ok(())
    } else {
        Err(anyhow!(
            "VCF angsd adapter row `{stage_id}` is missing required argv flag `{needle}`: {argv:?}"
        ))
    }
}

pub(super) fn run_missing_input_probe(
    repo_root: &Path,
    stage_id: &str,
    inputs: &[VcfAngsdAdapterArtifact],
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
            "VCF angsd adapter row `{stage_id}` unexpectedly accepted missing input `{}`",
            probe.artifact_id
        ),
        Err(error) => error.to_string(),
    };
    let passed = observed_error.contains(&expected_error_fragment);
    (probe.artifact_id, expected_error_fragment, observed_error, passed)
}

pub(super) fn ensure_vcf_angsd_adapter_contract(rows: &[VcfAngsdAdapterRow]) -> Result<()> {
    if rows.len() != 2 {
        return Err(anyhow!(
            "VCF angsd adapter must cover exactly 2 admitted registry rows, found {}",
            rows.len()
        ));
    }
    let expected_stages = BTreeSet::from(["vcf.call_gl", "vcf.call_pseudohaploid"]);
    let observed_stages = rows.iter().map(|row| row.stage_id.as_str()).collect::<BTreeSet<_>>();
    if observed_stages != expected_stages {
        return Err(anyhow!(
            "VCF angsd adapter stage set drifted: expected {expected_stages:?}, found {observed_stages:?}"
        ));
    }
    for row in rows {
        if row.tool_status != GOVERNED_ANGSD_TOOL_STATUS {
            return Err(anyhow!(
                "VCF angsd adapter row `{}` drifted from external-artifact tool status: {}",
                row.stage_id,
                row.tool_status
            ));
        }
        if !row.argv_validation_passed {
            return Err(anyhow!("VCF angsd adapter row `{}` failed argv validation", row.stage_id));
        }
        if !row.missing_input_test_passed {
            return Err(anyhow!(
                "VCF angsd adapter row `{}` failed missing-input validation: {}",
                row.stage_id,
                row.missing_input_observed_error
            ));
        }
    }
    Ok(())
}
