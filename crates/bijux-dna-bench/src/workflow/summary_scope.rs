//! Owner: bijux-dna-bench
//! Summary grouping and fairness scopes for benchmark summarization.

pub(super) type StageDatasetScope = (String, String, Option<String>, Option<String>);
pub(super) type StageDatasetToolScope = (String, String, Option<String>, Option<String>, String);
pub(super) type SummaryGroupKey = (
    String,
    String,
    Option<String>,
    Option<String>,
    String,
    String,
);
pub(super) type SummaryStratumKey = (String, Option<String>, Option<String>, String);

pub(super) fn stage_scope_label(
    stage_id: &str,
    stage_instance_id: Option<&str>,
    lineage_id: Option<&str>,
    dataset_id: &str,
) -> String {
    let mut parts = vec![stage_id.to_string(), dataset_id.to_string()];
    if let Some(stage_instance_id) = stage_instance_id {
        parts.push(stage_instance_id.to_string());
    }
    if let Some(lineage_id) = lineage_id {
        parts.push(lineage_id.to_string());
    }
    parts.join(":")
}
