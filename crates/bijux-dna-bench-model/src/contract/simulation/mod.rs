//! Owner: pure all-domain simulation input admission before artifact effects.
use crate::{
    all_domain_simulation_output_ids, all_domain_simulation_output_path,
    AllDomainExpectedBenchmarkResultRow, AllDomainOutputDeclarationRow,
    AllDomainOutputDeclarationStatus, AllDomainRenderedCommandRow,
};
use anyhow::{anyhow, Result};
use std::collections::{BTreeMap, BTreeSet};

/// An aligned, borrowed result ready for illustrative materialization.
pub struct AllDomainSimulationInput<'a> {
    pub expected: &'a AllDomainExpectedBenchmarkResultRow,
    pub command: &'a AllDomainRenderedCommandRow,
    pub outputs: &'a AllDomainOutputDeclarationRow,
}
/// Admit unique matching identities, output components and materialization destinations.
///
/// # Errors
/// Rejects missing, duplicated, contradictory or unsafe simulation inputs.
pub fn admit_all_domain_simulation_inputs<'a>(
    expected: &'a [AllDomainExpectedBenchmarkResultRow],
    commands: &'a [AllDomainRenderedCommandRow],
    outputs: &'a [AllDomainOutputDeclarationRow],
) -> Result<Vec<AllDomainSimulationInput<'a>>> {
    let expected_rows = unique_rows(expected, |row| &row.result_id)?;
    let command_rows = unique_rows(commands, |row| &row.result_id)?;
    let output_rows = unique_rows(outputs, |row| &row.result_id)?;
    ensure_fake_run_row_alignment(&expected_rows, &command_rows, &output_rows)?;
    let mut destinations = BTreeSet::new();
    let mut admitted = Vec::with_capacity(expected_rows.len());
    for expected in expected_rows.values() {
        let command = command_rows[&expected.result_id];
        let outputs = output_rows[&expected.result_id];
        admit_result_inputs(expected, command, outputs)?;
        anyhow::ensure!(
            destinations.insert((
                &expected.domain,
                &expected.corpus_id,
                &expected.stage_id,
                &expected.asset_profile_id,
                &expected.tool_id
            )),
            "simulation result destinations collide"
        );
        admitted.push(AllDomainSimulationInput { expected, command, outputs });
    }
    Ok(admitted)
}
fn unique_rows<T>(rows: &[T], id: impl Fn(&T) -> &str) -> Result<BTreeMap<String, &T>> {
    let mut mapped = BTreeMap::new();
    for row in rows {
        let key = id(row);
        anyhow::ensure!(!key.trim().is_empty(), "simulation row has empty result identity");
        anyhow::ensure!(
            mapped.insert(key.to_owned(), row).is_none(),
            "duplicate simulation result identity: {key}"
        );
    }
    anyhow::ensure!(!mapped.is_empty(), "simulation requires at least one result");
    Ok(mapped)
}
fn ensure_fake_run_row_alignment(
    expected_rows: &BTreeMap<String, &AllDomainExpectedBenchmarkResultRow>,
    command_rows: &BTreeMap<String, &AllDomainRenderedCommandRow>,
    output_rows: &BTreeMap<String, &AllDomainOutputDeclarationRow>,
) -> Result<()> {
    let expected_row_count = expected_rows.len();
    if command_rows.len() != expected_row_count || output_rows.len() != expected_row_count {
        return Err(anyhow!(
            "all-domain fake-runner requires exact expected-result, command, and output alignment; found expected={}, commands={}, outputs={}",
            expected_row_count,
            command_rows.len(),
            output_rows.len()
        ));
    }
    let expected_ids = expected_rows.keys().cloned().collect::<BTreeSet<_>>();
    let command_ids = command_rows.keys().cloned().collect::<BTreeSet<_>>();
    let output_ids = output_rows.keys().cloned().collect::<BTreeSet<_>>();
    if expected_ids != command_ids || expected_ids != output_ids {
        return Err(anyhow!(
            "all-domain fake-runner requires exact result_id alignment across expected results, rendered commands, and output declarations"
        ));
    }
    Ok(())
}
fn admit_result_inputs(
    expected: &AllDomainExpectedBenchmarkResultRow,
    command: &AllDomainRenderedCommandRow,
    output: &AllDomainOutputDeclarationRow,
) -> Result<()> {
    anyhow::ensure!(
        output.status == AllDomainOutputDeclarationStatus::Complete,
        "simulation requires complete output declarations for {}",
        expected.result_id
    );
    for (name, wanted, actual_command, actual_output) in [
        ("domain", &expected.domain, &command.domain, &output.domain),
        ("stage", &expected.stage_id, &command.stage_id, &output.stage_id),
        ("tool", &expected.tool_id, &command.tool_id, &output.tool_id),
        ("corpus", &expected.corpus_id, &command.corpus_id, &output.corpus_id),
        (
            "asset profile",
            &expected.asset_profile_id,
            &command.asset_profile_id,
            &output.asset_profile_id,
        ),
    ] {
        anyhow::ensure!(
            wanted == actual_command && wanted == actual_output,
            "simulation {name} identity mismatch for {}",
            expected.result_id
        );
        admit_path_component(wanted)?;
    }
    let ids = all_domain_simulation_output_ids(output);
    anyhow::ensure!(!ids.is_empty(), "simulation requires declared outputs");
    let mut paths = BTreeSet::new();
    for id in ids {
        admit_path_component(&id)?;
        anyhow::ensure!(
            paths.insert(all_domain_simulation_output_path(&id)),
            "simulation output names collide for {}",
            expected.result_id
        );
    }
    Ok(())
}
fn admit_path_component(value: &str) -> Result<()> {
    anyhow::ensure!(
        !value.trim().is_empty()
            && value != "."
            && value != ".."
            && !value.contains(['/', '\\', '\0']),
        "simulation output identifier must be one path component: {value:?}"
    );
    Ok(())
}

mod failure;
pub use failure::{
    validate_all_domain_failure_simulation_manifest, validate_all_domain_failure_simulation_record,
};

mod missing;
pub use missing::{
    admit_all_domain_missing_result_audit, admit_all_domain_missing_result_inputs,
    project_all_domain_missing_result_observation, validate_all_domain_missing_result_audit,
    AllDomainMissingResultObservation,
};
