//! Owner: bijux-dna-bench-vcf
//! descent adapter admission contracts.
use super::{
    bail, BTreeSet, Path, Result, VcfDescentFamilyAdapterArtifact,
    VcfDescentFamilyAdapterCommandStep, VcfDescentFamilyAdapterRow, GOVERNED_DESCENT_ROWS,
};

pub(super) fn validate_required_inputs(
    repo_root: &Path,
    stage_id: &str,
    required_inputs: &[VcfDescentFamilyAdapterArtifact],
) -> Result<()> {
    crate::adapters::inputs::validate_required_files(
        repo_root,
        stage_id,
        crate::adapters::inputs::InputContext::Adapter { family: "descent-family", descent: true },
        required_inputs.iter().map(|input| (input.artifact_id.as_str(), input.path.as_str())),
    )
}

pub(super) fn validate_command_steps(
    tool_id: &str,
    stage_id: &str,
    command_steps: &[VcfDescentFamilyAdapterCommandStep],
) -> Result<()> {
    if command_steps.is_empty() {
        bail!("VCF descent-family adapter row `{stage_id}` must retain at least one command step");
    }
    let joined = command_steps
        .iter()
        .flat_map(|step| step.argv.iter())
        .map(String::as_str)
        .collect::<Vec<_>>()
        .join(" ");
    if joined.contains("--help") {
        bail!("VCF descent-family adapter row `{stage_id}` still contains placeholder argv");
    }
    let expected_fragments: &[&str] = match (tool_id, stage_id) {
        ("plink2", "vcf.roh") => &["plink2", "--homozyg"],
        ("germline", "vcf.ibd") => {
            &["plink2", "--recode ped", "germline", "-input", ".ped", ".map", "-output"]
        }
        ("ibdseq", "vcf.ibd") => &["ibdseq", "gt=", "out=", "nthreads="],
        ("ibdhap", "vcf.ibd") => &[
            "ibdhap --vcf",
            "--out",
            "--genetic-map",
            "--sample sample_a --sample sample_b",
            "--scores",
            "--identifiers",
        ],
        ("ibdne", "vcf.demography") => &["ibdne", "map=", "out=", "nthreads=", "nboots=", "seed="],
        _ => &[tool_id],
    };
    for expected in expected_fragments {
        if !joined.contains(expected) {
            bail!(
                "VCF descent-family adapter row `{stage_id}` must retain `{expected}` in its command rendering"
            );
        }
    }
    Ok(())
}

pub(super) fn run_missing_input_probe(
    repo_root: &Path,
    stage_id: &str,
    required_inputs: &[VcfDescentFamilyAdapterArtifact],
) -> Result<(String, String, String, bool)> {
    let input = required_inputs
        .first()
        .ok_or_else(|| super::anyhow!("governed descent rows must keep a required input"))?;
    let mut mutated = required_inputs.to_vec();
    mutated[0].path = format!(
        "artifacts/benchmark-smoke/readiness/adapters/probes/{stage_id}/{}.missing",
        input.artifact_id
    );
    let expected = format!("required input `{}`", input.artifact_id);
    let observed = match validate_required_inputs(repo_root, stage_id, &mutated) {
        Ok(()) => format!(
            "VCF descent-family adapter row `{stage_id}` unexpectedly accepted missing input `{}`",
            input.artifact_id
        ),
        Err(error) => error.to_string(),
    };
    let passed = observed.contains(&expected);
    Ok((input.artifact_id.clone(), expected, observed, passed))
}

pub(super) fn ensure_vcf_descent_family_adapter_contract(
    rows: &[VcfDescentFamilyAdapterRow],
) -> Result<()> {
    if rows.len() != GOVERNED_DESCENT_ROWS.len() {
        bail!(
            "VCF descent-family adapter must cover exactly {} governed rows, found {}",
            GOVERNED_DESCENT_ROWS.len(),
            rows.len()
        );
    }
    let expected = GOVERNED_DESCENT_ROWS
        .iter()
        .map(|(tool_id, stage_id)| format!("{tool_id}:{stage_id}"))
        .collect::<BTreeSet<_>>();
    let actual =
        rows.iter().map(|row| format!("{}:{}", row.tool_id, row.stage_id)).collect::<BTreeSet<_>>();
    if expected != actual {
        bail!(
            "VCF descent-family adapter row set drifted: expected {expected:?}, found {actual:?}"
        );
    }
    for row in rows {
        if !row.argv_validation_passed {
            bail!("VCF descent-family adapter row `{}` failed argv validation", row.stage_id);
        }
        if !row.missing_input_test_passed {
            bail!(
                "VCF descent-family adapter row `{}` failed missing-input validation: {}",
                row.stage_id,
                row.missing_input_observed_error
            );
        }
    }
    Ok(())
}
