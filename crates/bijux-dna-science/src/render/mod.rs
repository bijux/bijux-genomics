use serde::Serialize;

use crate::domain::{
    BindingResolutionRow, ClaimEvidenceRow, DecisionReasoningRow, FastqClosureGateRow,
    FastqContainerReferenceRow, FastqDefaultBindingRiskRow, FastqDownloadBacklogRow,
    FastqEnvironmentRow, FastqMissingClosurePrerequisiteRow, FastqPaperArchiveRow,
    FastqTruthDeltaRow, ScienceIndex, SourceArchiveGapRow, SourceInventoryRow,
};

fn evidence_cell(value: &str) -> &str {
    if value.trim().is_empty() {
        "not_applicable"
    } else {
        value
    }
}

pub fn source_inventory_tsv(rows: &[SourceInventoryRow]) -> String {
    let mut out = String::from(
        "source_id\tkind\taccess\tauthority\tlocator\tarchive_path\tarchive_status\tcitation\ttool_ids\n",
    );
    for row in rows {
        let line = [
            evidence_cell(&row.source_id),
            evidence_cell(&row.kind),
            evidence_cell(&row.access),
            evidence_cell(&row.authority),
            evidence_cell(&row.locator),
            evidence_cell(&row.archive_path),
            evidence_cell(&row.archive_status),
            evidence_cell(&row.citation),
            evidence_cell(&row.tool_ids),
        ]
        .join("\t");
        out.push_str(&line);
        out.push('\n');
    }
    out
}

pub fn source_archive_gaps_tsv(rows: &[SourceArchiveGapRow]) -> String {
    let mut out = String::from(
        "source_id\tkind\taccess\tlocator\tarchive_path\tcitation\ttool_ids\treason\n",
    );
    for row in rows {
        out.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            row.source_id,
            row.kind,
            row.access,
            row.locator,
            row.archive_path,
            row.citation,
            row.tool_ids,
            row.reason
        ));
    }
    out
}

pub fn fastq_container_reference_tsv(rows: &[FastqContainerReferenceRow]) -> String {
    let mut out = String::from(
        "tool_id\tstage_ids\treference_status\tregistry_status\tversion\tdefault_version\tversion_rule\tupstream\tcitation\tlicense\tpinned_commit\tpin_strategy\truntimes\tcontainer_ref\tdockerfile\tapptainer_def\n",
    );
    for row in rows {
        out.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            row.tool_id,
            row.stage_ids,
            row.reference_status,
            row.registry_status,
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
            evidence_cell(&row.apptainer_def)
        ));
    }
    out
}

pub fn fastq_download_backlog_tsv(rows: &[FastqDownloadBacklogRow]) -> String {
    let mut out = String::from(
        "source_id\ttool_id\tstage_ids\tacquisition_mode\tbacklog_status\tlocator\tcitation\tarchive_path\tarchive_status\tpaper_root\tpaper_status\tnotes\n",
    );
    for row in rows {
        out.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            row.source_id,
            row.tool_id,
            row.stage_ids,
            row.acquisition_mode,
            row.backlog_status,
            row.locator,
            row.citation,
            row.archive_path,
            row.archive_status,
            row.paper_root,
            row.paper_status,
            row.notes
        ));
    }
    out
}

pub fn fastq_paper_archive_tsv(rows: &[FastqPaperArchiveRow]) -> String {
    let mut out = String::from(
        "paper_id\ttool_id\tstage_ids\tpaper_root\tpaper_status\topen_access_status\tprimary_locator\tsupporting_locators\tarchive_status\tnotes\n",
    );
    for row in rows {
        out.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            row.paper_id,
            row.tool_id,
            row.stage_ids,
            row.paper_root,
            row.paper_status,
            row.open_access_status,
            row.primary_locator,
            row.supporting_locators,
            row.archive_status,
            row.notes
        ));
    }
    out
}

pub fn fastq_closure_gate_tsv(rows: &[FastqClosureGateRow]) -> String {
    let mut out = String::from(
        "stage_id\ttool_id\tis_default\trequested_execution_status\teffective_closure_status\tworld_class_closed\tblocking_reasons\twarning_reasons\n",
    );
    for row in rows {
        let line = [
            row.stage_id.as_str(),
            row.tool_id.as_str(),
            if row.is_default { "true" } else { "false" },
            row.requested_execution_status.as_str(),
            row.effective_closure_status.as_str(),
            if row.world_class_closed { "true" } else { "false" },
            row.blocking_reasons.as_str(),
            row.warning_reasons.as_str(),
        ]
        .join("\t");
        out.push_str(line.trim_end_matches('\t'));
        out.push('\n');
    }
    out
}

pub fn fastq_truth_delta_tsv(rows: &[FastqTruthDeltaRow]) -> String {
    let mut out =
        String::from("entity_type\tentity_id\tlayer\texpected_status\tobserved_status\treason\n");
    for row in rows {
        out.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\n",
            row.entity_type,
            row.entity_id,
            row.layer,
            row.expected_status,
            row.observed_status,
            row.reason
        ));
    }
    out
}

pub fn fastq_missing_closure_prerequisites_tsv(
    rows: &[FastqMissingClosurePrerequisiteRow],
) -> String {
    let mut out = String::from("stage_id\ttool_id\tprerequisite\tseverity\tdetail\n");
    for row in rows {
        out.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\n",
            row.stage_id, row.tool_id, row.prerequisite, row.severity, row.detail
        ));
    }
    out
}

pub fn fastq_default_binding_risk_tsv(rows: &[FastqDefaultBindingRiskRow]) -> String {
    let mut out = String::from(
        "stage_id\tdefault_tool_id\trequested_execution_status\teffective_closure_status\trisk_class\tblocking_reasons\twarning_reasons\n",
    );
    for row in rows {
        let line = [
            row.stage_id.as_str(),
            row.default_tool_id.as_str(),
            row.requested_execution_status.as_str(),
            row.effective_closure_status.as_str(),
            row.risk_class.as_str(),
            row.blocking_reasons.as_str(),
            row.warning_reasons.as_str(),
        ]
        .join("\t");
        out.push_str(line.trim_end_matches('\t'));
        out.push('\n');
    }
    out
}

pub fn claim_evidence_tsv(rows: &[ClaimEvidenceRow]) -> String {
    let mut out = String::from("claim_id\tevidence_id\n");
    for row in rows {
        out.push_str(&format!("{}\t{}\n", row.claim_id, row.evidence_id));
    }
    out
}

pub fn decision_reasoning_tsv(rows: &[DecisionReasoningRow]) -> String {
    let mut out = String::from("decision_id\treasoning_id\n");
    for row in rows {
        out.push_str(&format!("{}\t{}\n", row.decision_id, row.reasoning_id));
    }
    out
}

pub fn binding_resolution_tsv(rows: &[BindingResolutionRow]) -> String {
    let mut out = String::from(
        "binding_id\tdecision_id\ttarget_type\ttarget_ref\tenforcement_level\tstatus\n",
    );
    for row in rows {
        out.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\n",
            row.binding_id,
            row.decision_id,
            row.target_type,
            row.target_ref,
            row.enforcement_level,
            row.status
        ));
    }
    out
}

pub fn fastq_environment_tsv(rows: &[FastqEnvironmentRow]) -> String {
    let mut out = String::from(
        "stage_id\ttool_id\tstage_status\ttool_status\tis_default\texecution_status\truntime_support\tnormalization_support\tbenchmark_support\tregistry_status\truntimes\tcontainer_ref\tdockerfile\tapptainer_def\tevidence_count\tclaim_ids\tdecision_id\tbinding_id\n",
    );
    for row in rows {
        out.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            row.stage_id,
            row.tool_id,
            row.stage_status,
            row.tool_status,
            row.is_default,
            row.execution_status,
            row.runtime_support,
            row.normalization_support,
            row.benchmark_support,
            row.registry_status,
            row.runtimes,
            row.container_ref,
            row.dockerfile,
            row.apptainer_def,
            row.evidence_count,
            row.claim_ids,
            row.decision_id,
            row.binding_id
        ));
    }
    out
}

pub fn to_pretty_json<T: Serialize>(value: &T) -> anyhow::Result<String> {
    serde_json::to_string_pretty(value).map_err(Into::into)
}

pub fn index_json(index: &ScienceIndex) -> anyhow::Result<String> {
    to_pretty_json(index)
}
