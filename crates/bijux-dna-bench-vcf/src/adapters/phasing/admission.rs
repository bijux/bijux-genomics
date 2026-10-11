//! Owner: bijux-dna-bench-vcf
//! phasing adapter admission contracts.
use super::{
    anyhow, artifact, BTreeSet, Path, RegistryToolContract, Result,
    VcfPhasingFamilyAdapterArtifact, VcfPhasingFamilyAdapterCommandStep,
    VcfPhasingFamilyAdapterRow,
};

pub(super) fn validate_required_inputs(
    repo_root: &Path,
    stage_id: &str,
    inputs: &[VcfPhasingFamilyAdapterArtifact],
) -> Result<()> {
    crate::adapters::inputs::validate_required_files(
        repo_root,
        stage_id,
        crate::adapters::inputs::InputContext::Adapter { family: "phasing-family", descent: false },
        inputs.iter().map(|input| (input.artifact_id.as_str(), input.path.as_str())),
    )
}

pub(super) fn validate_command_steps(
    tool_id: &str,
    stage_id: &str,
    steps: &[VcfPhasingFamilyAdapterCommandStep],
) -> Result<()> {
    if steps.is_empty() {
        return Err(anyhow!(
            "VCF {tool_id} adapter row `{stage_id}` must declare at least one command step"
        ));
    }
    let joined =
        steps.iter().flat_map(|step| step.argv.iter().cloned()).collect::<Vec<_>>().join(" ");
    if joined.contains("--help") || joined.to_ascii_lowercase().contains("placeholder") {
        return Err(anyhow!(
            "VCF {tool_id} adapter row `{stage_id}` still contains placeholder argv: {joined}"
        ));
    }
    for needle in ["panel", "map", "bcftools index", ".vcf.gz"] {
        if !joined.contains(needle) {
            return Err(anyhow!(
                "VCF {tool_id} adapter row `{stage_id}` is missing required token `{needle}`: {joined}"
            ));
        }
    }
    match tool_id {
        "shapeit5" => {
            for needle in ["shapeit5", "phase_common", "--reference", "--map", "--output"] {
                if !joined.contains(needle) {
                    return Err(anyhow!(
                        "VCF shapeit5 adapter row `{stage_id}` is missing `{needle}`: {joined}"
                    ));
                }
            }
        }
        "shapeit" => {
            for needle in ["shapeit", "--input", "--reference", "--map", "--output"] {
                if !joined.contains(needle) {
                    return Err(anyhow!(
                        "VCF shapeit adapter row `{stage_id}` is missing `{needle}`: {joined}"
                    ));
                }
            }
        }
        "eagle" => {
            for needle in ["eagle", "--vcfTarget", "--vcfRef", "--geneticMapFile", "--outPrefix"] {
                if !joined.contains(needle) {
                    return Err(anyhow!(
                        "VCF eagle adapter row `{stage_id}` is missing `{needle}`: {joined}"
                    ));
                }
            }
        }
        "beagle" => {
            for needle in ["beagle", "gt=", "ref=", "map=", "out="] {
                if !joined.contains(needle) {
                    return Err(anyhow!(
                        "VCF beagle adapter row `{stage_id}` is missing `{needle}`: {joined}"
                    ));
                }
            }
        }
        _ => {}
    }
    Ok(())
}

pub(super) fn run_missing_input_probe(
    repo_root: &Path,
    stage_id: &str,
    inputs: &[VcfPhasingFamilyAdapterArtifact],
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
            "VCF phasing-family adapter row `{stage_id}` unexpectedly accepted missing input `{}`",
            probe.artifact_id
        ),
        Err(error) => error.to_string(),
    };
    let passed = observed_error.contains(&expected_error_fragment);
    (probe.artifact_id, expected_error_fragment, observed_error, passed)
}

pub(super) fn ensure_vcf_phasing_family_adapter_contract(
    registry_tool: &RegistryToolContract,
    rows: &[VcfPhasingFamilyAdapterRow],
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
        if row.parser_output_ids.is_empty() {
            return Err(anyhow!(
                "VCF {} adapter row `{}` is missing parser output declarations",
                registry_tool.tool_id,
                row.stage_id
            ));
        }
    }
    Ok(())
}
