use serde::Serialize;

use crate::domain::{
    BindingResolutionRow, ClaimEvidenceRow, DecisionReasoningRow, FastqEnvironmentRow,
    ScienceIndex, SourceArchiveGapRow, SourceInventoryRow,
};

pub fn source_inventory_tsv(rows: &[SourceInventoryRow]) -> String {
    let mut out = String::from(
        "source_id\tkind\taccess\tauthority\tlocator\tarchive_path\tarchive_status\tcitation\ttool_ids\n",
    );
    for row in rows {
        out.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            row.source_id,
            row.kind,
            row.access,
            row.authority,
            row.locator,
            row.archive_path,
            row.archive_status,
            row.citation,
            row.tool_ids
        ));
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
