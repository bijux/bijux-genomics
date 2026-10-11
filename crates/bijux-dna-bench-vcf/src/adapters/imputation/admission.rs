//! Owner: bijux-dna-bench-vcf
//! imputation adapter admission contracts.
use super::{
    anyhow, artifact, BTreeMap, BTreeSet, Path, RegistryToolContract, Result,
    VcfImputationFamilyAdapterArtifact, VcfImputationFamilyAdapterCommandStep,
    VcfImputationFamilyAdapterRow, GOVERNED_IMPUTATION_STAGE_IDS, GOVERNED_IMPUTATION_TOOL_IDS,
};

pub(super) fn validate_required_inputs(
    repo_root: &Path,
    stage_id: &str,
    inputs: &[VcfImputationFamilyAdapterArtifact],
) -> Result<()> {
    crate::adapters::inputs::validate_required_files(
        repo_root,
        stage_id,
        crate::adapters::inputs::InputContext::Adapter {
            family: "imputation-family",
            descent: false,
        },
        inputs.iter().map(|input| (input.artifact_id.as_str(), input.path.as_str())),
    )
}

pub(super) fn validate_command_steps(
    tool_id: &str,
    stage_id: &str,
    steps: &[VcfImputationFamilyAdapterCommandStep],
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
    if !joined.contains("bcftools index") || !joined.contains(".vcf.gz") {
        return Err(anyhow!(
            "VCF {tool_id} adapter row `{stage_id}` must keep indexed VCF output wiring: {joined}"
        ));
    }
    match tool_id {
        "beagle" => {
            for needle in ["beagle", "gt=", "ref=", "map=", "impute=true"] {
                if !joined.contains(needle) {
                    return Err(anyhow!(
                        "VCF beagle adapter row `{stage_id}` is missing `{needle}`: {joined}"
                    ));
                }
            }
        }
        "glimpse" => {
            for needle in
                ["GLIMPSE_phase", "--reference", "--map", "--input-region", "--output-region"]
            {
                if !joined.contains(needle) {
                    return Err(anyhow!(
                        "VCF glimpse adapter row `{stage_id}` is missing `{needle}`: {joined}"
                    ));
                }
            }
        }
        "impute5" => {
            for needle in ["impute5", "--g", "--h", "--m", "--r"] {
                if !joined.contains(needle) {
                    return Err(anyhow!(
                        "VCF impute5 adapter row `{stage_id}` is missing `{needle}`: {joined}"
                    ));
                }
            }
        }
        "minimac4" => {
            for needle in ["minimac4", "--threads", "--map", "--output", "panel.msav"] {
                if !joined.contains(needle) {
                    return Err(anyhow!(
                        "VCF minimac4 adapter row `{stage_id}` is missing `{needle}`: {joined}"
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
    inputs: &[VcfImputationFamilyAdapterArtifact],
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
            "VCF imputation-family adapter row `{stage_id}` unexpectedly accepted missing input `{}`",
            probe.artifact_id
        ),
        Err(error) => error.to_string(),
    };
    let passed = observed_error.contains(&expected_error_fragment);
    (probe.artifact_id, expected_error_fragment, observed_error, passed)
}

pub(super) fn ensure_vcf_imputation_family_adapter_contract(
    registry_by_tool: &BTreeMap<String, RegistryToolContract>,
    rows: &[VcfImputationFamilyAdapterRow],
) -> Result<()> {
    let observed_tools = rows.iter().map(|row| row.tool_id.as_str()).collect::<BTreeSet<_>>();
    let expected_tools = GOVERNED_IMPUTATION_TOOL_IDS.iter().copied().collect::<BTreeSet<_>>();
    if observed_tools != expected_tools {
        return Err(anyhow!(
            "VCF imputation-family adapter tool set drifted: expected {expected_tools:?}, found {observed_tools:?}"
        ));
    }
    for tool_id in GOVERNED_IMPUTATION_TOOL_IDS {
        let registry_tool = registry_by_tool
            .get(tool_id)
            .ok_or_else(|| anyhow!("missing registry tool `{tool_id}`"))?;
        let tool_rows = rows.iter().filter(|row| row.tool_id == tool_id).collect::<Vec<_>>();
        let observed_stage_ids =
            tool_rows.iter().map(|row| row.stage_id.as_str()).collect::<BTreeSet<_>>();
        let expected_stage_ids =
            GOVERNED_IMPUTATION_STAGE_IDS.iter().copied().collect::<BTreeSet<_>>();
        if !registry_tool
            .stage_ids
            .iter()
            .map(String::as_str)
            .collect::<BTreeSet<_>>()
            .is_superset(&expected_stage_ids)
        {
            return Err(anyhow!(
                "VCF registry row for `{tool_id}` does not cover the governed imputation stage slice"
            ));
        }
        if observed_stage_ids != expected_stage_ids {
            return Err(anyhow!(
                "VCF imputation-family adapter stage set drifted for `{tool_id}`: expected {expected_stage_ids:?}, found {observed_stage_ids:?}"
            ));
        }
        for row in tool_rows {
            if row.tool_status != registry_tool.tool_status {
                return Err(anyhow!(
                    "VCF imputation-family row `{}` / `{tool_id}` drifted from registry tool status `{}`",
                    row.stage_id,
                    registry_tool.tool_status
                ));
            }
            if !row.argv_validation_passed {
                return Err(anyhow!(
                    "VCF imputation-family row `{}` / `{tool_id}` failed argv validation",
                    row.stage_id
                ));
            }
            if !row.missing_input_test_passed {
                return Err(anyhow!(
                    "VCF imputation-family row `{}` / `{tool_id}` failed missing-input validation: {}",
                    row.stage_id,
                    row.missing_input_observed_error
                ));
            }
            if row.parser_output_ids.is_empty() {
                return Err(anyhow!(
                    "VCF imputation-family row `{}` / `{tool_id}` is missing parser output declarations",
                    row.stage_id
                ));
            }
        }
    }
    let benchmark_ready_row_count =
        rows.iter().filter(|row| row.benchmark_status == "benchmark_ready").count();
    if benchmark_ready_row_count != 8 {
        return Err(anyhow!(
            "VCF imputation-family adapter benchmark-ready count drifted: expected 8, found {benchmark_ready_row_count}"
        ));
    }
    Ok(())
}
