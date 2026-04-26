use std::borrow::Cow;

use serde::Serialize;

use crate::domain::{
    BindingResolutionRow, ClaimEvidenceRow, DecisionReasoningRow, FastqClosureGateRow,
    FastqContainerReferenceRow, FastqDefaultBindingRiskRow, FastqDownloadBacklogRow,
    FastqEnvironmentRow, FastqMissingClosurePrerequisiteRow, FastqPaperArchiveRow,
    FastqTruthDeltaRow, ScienceIndex, SourceArchiveGapRow, SourceInventoryRow,
};

fn evidence_cell(value: &str) -> Cow<'_, str> {
    if value.trim().is_empty() {
        Cow::Borrowed("not_applicable")
    } else {
        tsv_cell(value)
    }
}

fn tsv_cell(value: &str) -> Cow<'_, str> {
    if value.contains(['\t', '\n', '\r']) {
        Cow::Owned(
            value
                .chars()
                .map(
                    |character| {
                        if matches!(character, '\t' | '\n' | '\r') {
                            ' '
                        } else {
                            character
                        }
                    },
                )
                .collect(),
        )
    } else {
        Cow::Borrowed(value)
    }
}

fn push_tsv_row(out: &mut String, cells: &[Cow<'_, str>]) {
    for (index, cell) in cells.iter().enumerate() {
        if index > 0 {
            out.push('\t');
        }
        out.push_str(cell.as_ref());
    }
    out.push('\n');
}

#[must_use]
pub fn source_inventory_tsv(rows: &[SourceInventoryRow]) -> String {
    let mut out = String::from(
        "source_id\tkind\taccess\tauthority\tlocator\tarchive_path\tarchive_status\tcitation\ttool_ids\n",
    );
    for row in rows {
        push_tsv_row(
            &mut out,
            &[
                evidence_cell(&row.source_id),
                evidence_cell(&row.kind),
                evidence_cell(&row.access),
                evidence_cell(&row.authority),
                evidence_cell(&row.locator),
                evidence_cell(&row.archive_path),
                evidence_cell(&row.archive_status),
                evidence_cell(&row.citation),
                evidence_cell(&row.tool_ids),
            ],
        );
    }
    out
}

#[must_use]
pub fn source_archive_gaps_tsv(rows: &[SourceArchiveGapRow]) -> String {
    let mut out = String::from(
        "source_id\tkind\taccess\tlocator\tarchive_path\tcitation\ttool_ids\treason\n",
    );
    for row in rows {
        push_tsv_row(
            &mut out,
            &[
                evidence_cell(&row.source_id),
                evidence_cell(&row.kind),
                evidence_cell(&row.access),
                evidence_cell(&row.locator),
                evidence_cell(&row.archive_path),
                evidence_cell(&row.citation),
                evidence_cell(&row.tool_ids),
                evidence_cell(&row.reason),
            ],
        );
    }
    out
}

#[must_use]
pub fn fastq_container_reference_tsv(rows: &[FastqContainerReferenceRow]) -> String {
    let mut out = String::from(
        "tool_id\tstage_ids\treference_status\tregistry_status\tversion\tdefault_version\tversion_rule\tupstream\tcitation\tlicense\tpinned_commit\tpin_strategy\truntimes\tcontainer_ref\tdockerfile\tapptainer_def\n",
    );
    for row in rows {
        push_tsv_row(
            &mut out,
            &[
                evidence_cell(&row.tool_id),
                evidence_cell(&row.stage_ids),
                evidence_cell(&row.reference_status),
                evidence_cell(&row.registry_status),
                evidence_cell(&row.version),
                evidence_cell(&row.default_version),
                evidence_cell(&row.version_rule),
                evidence_cell(&row.upstream),
                evidence_cell(&row.citation),
                evidence_cell(&row.license),
                evidence_cell(&row.pinned_commit),
                evidence_cell(&row.pin_strategy),
                evidence_cell(&row.runtimes),
                evidence_cell(&row.container_ref),
                evidence_cell(&row.dockerfile),
                evidence_cell(&row.apptainer_def),
            ],
        );
    }
    out
}

#[must_use]
pub fn fastq_download_backlog_tsv(rows: &[FastqDownloadBacklogRow]) -> String {
    let mut out = String::from(
        "source_id\ttool_id\tstage_ids\tacquisition_mode\tbacklog_status\tlocator\tcitation\tarchive_path\tarchive_status\tpaper_root\tpaper_status\tnotes\n",
    );
    for row in rows {
        push_tsv_row(
            &mut out,
            &[
                evidence_cell(&row.source_id),
                evidence_cell(&row.tool_id),
                evidence_cell(&row.stage_ids),
                evidence_cell(&row.acquisition_mode),
                evidence_cell(&row.backlog_status),
                evidence_cell(&row.locator),
                evidence_cell(&row.citation),
                evidence_cell(&row.archive_path),
                evidence_cell(&row.archive_status),
                evidence_cell(&row.paper_root),
                evidence_cell(&row.paper_status),
                evidence_cell(&row.notes),
            ],
        );
    }
    out
}

#[must_use]
pub fn fastq_paper_archive_tsv(rows: &[FastqPaperArchiveRow]) -> String {
    let mut out = String::from(
        "paper_id\ttool_id\tstage_ids\tpaper_root\tpaper_status\topen_access_status\tprimary_locator\tsupporting_locators\tarchive_status\tnotes\n",
    );
    for row in rows {
        push_tsv_row(
            &mut out,
            &[
                evidence_cell(&row.paper_id),
                evidence_cell(&row.tool_id),
                evidence_cell(&row.stage_ids),
                evidence_cell(&row.paper_root),
                evidence_cell(&row.paper_status),
                evidence_cell(&row.open_access_status),
                evidence_cell(&row.primary_locator),
                evidence_cell(&row.supporting_locators),
                evidence_cell(&row.archive_status),
                evidence_cell(&row.notes),
            ],
        );
    }
    out
}

#[must_use]
pub fn fastq_closure_gate_tsv(rows: &[FastqClosureGateRow]) -> String {
    let mut out = String::from(
        "stage_id\ttool_id\tis_default\trequested_execution_status\teffective_closure_status\tworld_class_closed\tblocking_reasons\twarning_reasons\n",
    );
    for row in rows {
        push_tsv_row(
            &mut out,
            &[
                evidence_cell(&row.stage_id),
                evidence_cell(&row.tool_id),
                evidence_cell(if row.is_default { "true" } else { "false" }),
                evidence_cell(&row.requested_execution_status),
                evidence_cell(&row.effective_closure_status),
                evidence_cell(if row.world_class_closed { "true" } else { "false" }),
                evidence_cell(&row.blocking_reasons),
                evidence_cell(&row.warning_reasons),
            ],
        );
    }
    out
}

#[must_use]
pub fn fastq_truth_delta_tsv(rows: &[FastqTruthDeltaRow]) -> String {
    let mut out =
        String::from("entity_type\tentity_id\tlayer\texpected_status\tobserved_status\treason\n");
    for row in rows {
        push_tsv_row(
            &mut out,
            &[
                evidence_cell(&row.entity_type),
                evidence_cell(&row.entity_id),
                evidence_cell(&row.layer),
                evidence_cell(&row.expected_status),
                evidence_cell(&row.observed_status),
                evidence_cell(&row.reason),
            ],
        );
    }
    out
}

#[must_use]
pub fn fastq_missing_closure_prerequisites_tsv(
    rows: &[FastqMissingClosurePrerequisiteRow],
) -> String {
    let mut out = String::from("stage_id\ttool_id\tprerequisite\tseverity\tdetail\n");
    for row in rows {
        push_tsv_row(
            &mut out,
            &[
                evidence_cell(&row.stage_id),
                evidence_cell(&row.tool_id),
                evidence_cell(&row.prerequisite),
                evidence_cell(&row.severity),
                evidence_cell(&row.detail),
            ],
        );
    }
    out
}

#[must_use]
pub fn fastq_default_binding_risk_tsv(rows: &[FastqDefaultBindingRiskRow]) -> String {
    let mut out = String::from(
        "stage_id\tdefault_tool_id\trequested_execution_status\teffective_closure_status\trisk_class\tblocking_reasons\twarning_reasons\n",
    );
    for row in rows {
        push_tsv_row(
            &mut out,
            &[
                evidence_cell(&row.stage_id),
                evidence_cell(&row.default_tool_id),
                evidence_cell(&row.requested_execution_status),
                evidence_cell(&row.effective_closure_status),
                evidence_cell(&row.risk_class),
                evidence_cell(&row.blocking_reasons),
                evidence_cell(&row.warning_reasons),
            ],
        );
    }
    out
}

#[must_use]
pub fn claim_evidence_tsv(rows: &[ClaimEvidenceRow]) -> String {
    let mut out = String::from("claim_id\tevidence_id\n");
    for row in rows {
        push_tsv_row(&mut out, &[evidence_cell(&row.claim_id), evidence_cell(&row.evidence_id)]);
    }
    out
}

#[must_use]
pub fn decision_reasoning_tsv(rows: &[DecisionReasoningRow]) -> String {
    let mut out = String::from("decision_id\treasoning_id\n");
    for row in rows {
        push_tsv_row(
            &mut out,
            &[evidence_cell(&row.decision_id), evidence_cell(&row.reasoning_id)],
        );
    }
    out
}

#[must_use]
pub fn binding_resolution_tsv(rows: &[BindingResolutionRow]) -> String {
    let mut out = String::from(
        "binding_id\tdecision_id\ttarget_type\ttarget_ref\tenforcement_level\tstatus\n",
    );
    for row in rows {
        push_tsv_row(
            &mut out,
            &[
                evidence_cell(&row.binding_id),
                evidence_cell(&row.decision_id),
                evidence_cell(&row.target_type),
                evidence_cell(&row.target_ref),
                evidence_cell(&row.enforcement_level),
                evidence_cell(&row.status),
            ],
        );
    }
    out
}

#[must_use]
pub fn fastq_environment_tsv(rows: &[FastqEnvironmentRow]) -> String {
    let mut out = String::from(
        "stage_id\ttool_id\tstage_status\ttool_status\tis_default\texecution_status\truntime_support\tnormalization_support\tbenchmark_support\tregistry_status\truntimes\tcontainer_ref\tdockerfile\tapptainer_def\tevidence_count\tclaim_ids\tdecision_id\tbinding_id\n",
    );
    for row in rows {
        push_tsv_row(
            &mut out,
            &[
                evidence_cell(&row.stage_id),
                evidence_cell(&row.tool_id),
                evidence_cell(&row.stage_status),
                evidence_cell(&row.tool_status),
                evidence_cell(if row.is_default { "true" } else { "false" }),
                evidence_cell(&row.execution_status),
                evidence_cell(&row.runtime_support),
                evidence_cell(&row.normalization_support),
                evidence_cell(&row.benchmark_support),
                evidence_cell(&row.registry_status),
                evidence_cell(&row.runtimes),
                evidence_cell(&row.container_ref),
                evidence_cell(&row.dockerfile),
                evidence_cell(&row.apptainer_def),
                evidence_cell(&row.evidence_count.to_string()),
                evidence_cell(&row.claim_ids),
                evidence_cell(&row.decision_id),
                evidence_cell(&row.binding_id),
            ],
        );
    }
    out
}

/// Render stable, pretty JSON for generated science outputs.
///
/// # Errors
///
/// Returns an error when the value cannot be serialized as JSON.
pub fn to_pretty_json<T: Serialize>(value: &T) -> anyhow::Result<String> {
    serde_json::to_string_pretty(value).map_err(Into::into)
}

/// Render the science index as stable, pretty JSON.
///
/// # Errors
///
/// Returns an error when the index cannot be serialized as JSON.
pub fn index_json(index: &ScienceIndex) -> anyhow::Result<String> {
    to_pretty_json(index)
}
