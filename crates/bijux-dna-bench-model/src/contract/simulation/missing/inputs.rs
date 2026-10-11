//! Owner: complete source admission before illustrative missing-result fixture effects.
use crate::{
    admit_all_domain_simulation_inputs, AllDomainExpectedBenchmarkResultRow,
    AllDomainOutputDeclarationRow, AllDomainRenderedCommandRow, AllDomainSimulationInput,
    BAM_REMOVED_RESULT_ID, FASTQ_REMOVED_RESULT_ID, VCF_REMOVED_RESULT_ID,
};
use anyhow::{ensure, Result};
use std::collections::BTreeSet;

/// Admit the complete simulation source and its governed omission identities before effects.
/// # Errors
/// Rejects ambiguous sources, missing governed omissions and incomplete report declarations.
pub fn admit_all_domain_missing_result_inputs<'a>(
    expected: &'a [AllDomainExpectedBenchmarkResultRow],
    commands: &'a [AllDomainRenderedCommandRow],
    outputs: &'a [AllDomainOutputDeclarationRow],
) -> Result<Vec<AllDomainSimulationInput<'a>>> {
    let inputs = admit_all_domain_simulation_inputs(expected, commands, outputs)?;
    for input in &inputs {
        let row = input.expected;
        ensure!(!row.report_section.trim().is_empty(), "missing-result report section is empty");
        super::unique_values(&row.expected_metrics)?;
        super::unique_values(&row.expected_outputs)?;
    }
    let identities = expected
        .iter()
        .map(|row| (row.result_id.as_str(), row.domain.as_str()))
        .collect::<BTreeSet<_>>();
    for omission in [
        (FASTQ_REMOVED_RESULT_ID, "fastq"),
        (BAM_REMOVED_RESULT_ID, "bam"),
        (VCF_REMOVED_RESULT_ID, "vcf"),
    ] {
        ensure!(
            identities.contains(&omission),
            "missing-result source lacks governed omission {}",
            omission.0
        );
    }
    Ok(inputs)
}
