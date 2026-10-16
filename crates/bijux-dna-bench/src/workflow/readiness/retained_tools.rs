//! Owner: bijux-dna-bench
//! Pure retained-tool scope aggregation over acquired benchmark and delivery records.

use std::collections::{BTreeMap, BTreeSet};

use anyhow::{anyhow, Result};
use bijux_dna_bench_model::{AllDomainActiveStageToolMatrixRow, AllDomainRetainedToolRow};
use bijux_dna_containers::evidence::tool_contract::delivery::ExternalArtifactToolContract;

const BENCHMARK_READY_STATUS: &str = "benchmark_ready";

#[derive(Default)]
struct ToolAccumulator {
    domains: BTreeSet<String>,
    benchmark_statuses: BTreeSet<String>,
    active_stage_ids: BTreeSet<String>,
    benchmark_ready_stage_ids: BTreeSet<String>,
    active_binding_count: usize,
    benchmark_ready_binding_count: usize,
}

/// Derive the retained-tool inventory from acquired executable bindings and delivery contracts.
///
/// Binding observations remain distinct while stage and domain identities are deduplicated.
/// External contracts extend stage retention without inventing benchmark-ready observations.
///
/// # Errors
/// Returns an error when retained scope is incomplete, counters drift, or an external
/// artifact tool does not retain the existing benchmark-ready status contract.
pub fn derive_all_domain_retained_tool_rows(
    source_rows: &[AllDomainActiveStageToolMatrixRow],
    external_artifacts: &[ExternalArtifactToolContract],
) -> Result<Vec<AllDomainRetainedToolRow>> {
    let mut by_tool = BTreeMap::<String, ToolAccumulator>::new();
    for row in source_rows {
        let entry = by_tool.entry(row.tool_id.clone()).or_default();
        entry.domains.insert(row.domain.clone());
        entry.benchmark_statuses.insert(row.status.clone());
        entry.active_stage_ids.insert(row.stage_id.clone());
        entry.active_binding_count += 1;
        if row.status == BENCHMARK_READY_STATUS {
            entry.benchmark_ready_stage_ids.insert(row.stage_id.clone());
            entry.benchmark_ready_binding_count += 1;
        }
    }

    for contract in external_artifacts {
        let entry = by_tool.entry(contract.tool_id.clone()).or_default();
        entry.domains.extend(contract.domains.iter().cloned());
        for stage_id in &contract.stage_ids {
            if entry.active_stage_ids.insert(stage_id.clone()) {
                entry.active_binding_count += 1;
            }
        }
    }

    let rows = by_tool
        .into_iter()
        .map(|(tool_id, accumulator)| AllDomainRetainedToolRow {
            tool_id,
            domains: accumulator.domains.into_iter().collect(),
            active_stage_count: accumulator.active_stage_ids.len(),
            benchmark_ready_stage_count: accumulator.benchmark_ready_stage_ids.len(),
            active_binding_count: accumulator.active_binding_count,
            benchmark_ready_binding_count: accumulator.benchmark_ready_binding_count,
            benchmark_statuses: accumulator.benchmark_statuses.into_iter().collect(),
            active_stage_ids: accumulator.active_stage_ids.into_iter().collect(),
            benchmark_ready_stage_ids: accumulator.benchmark_ready_stage_ids.into_iter().collect(),
        })
        .collect::<Vec<_>>();

    ensure_all_domain_retained_tool_contract(source_rows, external_artifacts, &rows)?;
    Ok(rows)
}

fn ensure_all_domain_retained_tool_contract(
    source_rows: &[AllDomainActiveStageToolMatrixRow],
    external_artifacts: &[ExternalArtifactToolContract],
    rows: &[AllDomainRetainedToolRow],
) -> Result<()> {
    let inventory_tool_ids = rows.iter().map(|row| row.tool_id.as_str()).collect::<BTreeSet<_>>();
    if inventory_tool_ids.len() != rows.len() {
        return Err(anyhow!("retained tool inventory must keep exactly one row per tool_id"));
    }

    let expected_tool_ids = source_rows
        .iter()
        .map(|row| row.tool_id.as_str())
        .chain(external_artifacts.iter().map(|contract| contract.tool_id.as_str()))
        .collect::<BTreeSet<_>>();
    if inventory_tool_ids != expected_tool_ids {
        return Err(anyhow!(
            "retained tool inventory drifted from active bindings and external artifact contracts"
        ));
    }

    for row in rows {
        if row.tool_id.trim().is_empty()
            || row.domains.is_empty()
            || row.active_stage_ids.is_empty()
            || row.active_binding_count == 0
            || row.active_stage_count == 0
            || row.benchmark_statuses.is_empty()
        {
            return Err(anyhow!(
                "retained tool inventory row `{}` is missing a required retained-scope field",
                row.tool_id
            ));
        }
        if row.active_stage_count != row.active_stage_ids.len() {
            return Err(anyhow!(
                "retained tool inventory row `{}` drifted from its active stage ids",
                row.tool_id
            ));
        }
        if row.benchmark_ready_stage_count != row.benchmark_ready_stage_ids.len() {
            return Err(anyhow!(
                "retained tool inventory row `{}` drifted from its benchmark-ready stage ids",
                row.tool_id
            ));
        }
        if row.benchmark_ready_binding_count > row.active_binding_count
            || row.benchmark_ready_stage_count > row.active_stage_count
        {
            return Err(anyhow!(
                "retained tool inventory row `{}` overcounts benchmark-ready coverage",
                row.tool_id
            ));
        }
        if row.active_stage_count == 0 {
            return Err(anyhow!("retained tool `{}` serves zero active stages", row.tool_id));
        }
    }

    for contract in external_artifacts {
        let row = rows
            .iter()
            .find(|row| row.tool_id == contract.tool_id)
            .ok_or_else(|| anyhow!("external artifact tool `{}` is missing", contract.tool_id))?;
        if row.benchmark_statuses.as_slice() != [BENCHMARK_READY_STATUS] {
            return Err(anyhow!(
                "external artifact tool `{}` must remain benchmark-ready independently of artifact availability",
                contract.tool_id
            ));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use anyhow::{anyhow, Result};
    use bijux_dna_bench_model::AllDomainActiveStageToolMatrixRow;
    use bijux_dna_containers::evidence::tool_contract::delivery::ExternalArtifactToolContract;

    use super::{derive_all_domain_retained_tool_rows, ensure_all_domain_retained_tool_contract};

    fn binding(
        domain: &str,
        stage: &str,
        tool: &str,
        status: &str,
    ) -> AllDomainActiveStageToolMatrixRow {
        AllDomainActiveStageToolMatrixRow {
            domain: domain.to_string(),
            stage_id: stage.to_string(),
            tool_id: tool.to_string(),
            corpus_id: "corpus".to_string(),
            asset_profile_id: "assets".to_string(),
            adapter_id: "adapter".to_string(),
            parser_id: "parser".to_string(),
            schema_id: "schema".to_string(),
            status: status.to_string(),
        }
    }

    fn external(tool: &str, stages: &[&str]) -> ExternalArtifactToolContract {
        ExternalArtifactToolContract {
            tool_id: tool.to_string(),
            domains: vec!["fastq".to_string(), "vcf".to_string()],
            stage_ids: stages.iter().map(|stage| (*stage).to_string()).collect(),
            expected_bin: tool.to_string(),
            version_command: format!("{tool} --version"),
            help_command: format!("{tool} --help"),
            expected_version_regex: "1".to_string(),
            declared_version: "1".to_string(),
        }
    }

    #[test]
    fn retained_tool_rows_distinguish_bindings_unique_stages_and_ready_observations() -> Result<()>
    {
        let source = [
            binding("fastq", "stage-b", "shared", "benchmark_ready"),
            binding("fastq", "stage-b", "shared", "not_benchmark_ready"),
            binding("bam", "stage-a", "shared", "benchmark_ready"),
            binding("fastq", "stage-c", "solo", "not_benchmark_ready"),
        ];
        let rows = derive_all_domain_retained_tool_rows(&source, &[])?;
        assert_eq!(
            serde_json::to_value(&rows)?,
            serde_json::json!([
                {"tool_id":"shared", "domains":["bam","fastq"], "active_stage_count":2,
                 "benchmark_ready_stage_count":2, "active_binding_count":3, "benchmark_ready_binding_count":2,
                 "benchmark_statuses":["benchmark_ready","not_benchmark_ready"],
                 "active_stage_ids":["stage-a","stage-b"], "benchmark_ready_stage_ids":["stage-a","stage-b"]},
                {"tool_id":"solo", "domains":["fastq"], "active_stage_count":1,
                 "benchmark_ready_stage_count":0, "active_binding_count":1, "benchmark_ready_binding_count":0,
                 "benchmark_statuses":["not_benchmark_ready"], "active_stage_ids":["stage-c"],
                 "benchmark_ready_stage_ids":[]}
            ])
        );
        let mut reversed = source.to_vec();
        reversed.reverse();
        assert_eq!(derive_all_domain_retained_tool_rows(&reversed, &[])?, rows);
        Ok(())
    }

    #[test]
    fn external_retention_merges_stage_overlap_without_creating_ready_bindings() -> Result<()> {
        let source = [binding("fastq", "stage-a", "external", "benchmark_ready")];
        let contracts = [
            external("external", &["stage-a", "stage-b", "stage-b"]),
            external("external", &["stage-b"]),
        ];
        let rows = derive_all_domain_retained_tool_rows(&source, &contracts)?;
        assert_eq!(
            serde_json::to_value(&rows)?,
            serde_json::json!([
                {"tool_id":"external", "domains":["fastq","vcf"], "active_stage_count":2,
                 "benchmark_ready_stage_count":1, "active_binding_count":2, "benchmark_ready_binding_count":1,
                 "benchmark_statuses":["benchmark_ready"], "active_stage_ids":["stage-a","stage-b"],
                 "benchmark_ready_stage_ids":["stage-a"]}
            ])
        );
        Ok(())
    }

    #[test]
    fn retained_tool_admission_rejects_missing_ready_evidence_and_source_drift() -> Result<()> {
        let contracts = [external("external", &["stage-a"])];
        let source = [binding("fastq", "stage-a", "external", "not_benchmark_ready")];
        let error = derive_all_domain_retained_tool_rows(&source, &contracts)
            .err()
            .ok_or_else(|| anyhow!("non-ready external binding must refuse"))?;
        assert_eq!(error.to_string(),
            "external artifact tool `external` must remain benchmark-ready independently of artifact availability");
        let error = derive_all_domain_retained_tool_rows(&[], &contracts)
            .err()
            .ok_or_else(|| anyhow!("external metadata cannot invent ready source observations"))?;
        assert_eq!(
            error.to_string(),
            "retained tool inventory row `external` is missing a required retained-scope field"
        );

        let source = [
            binding("fastq", "stage-a", "external", "benchmark_ready"),
            binding("bam", "stage-b", "other", "benchmark_ready"),
        ];
        let mut rows = derive_all_domain_retained_tool_rows(&source, &contracts)?;
        rows.pop();
        let error = ensure_all_domain_retained_tool_contract(&source, &contracts, &rows)
            .err()
            .ok_or_else(|| anyhow!("missing source tool must refuse"))?;
        assert_eq!(
            error.to_string(),
            "retained tool inventory drifted from active bindings and external artifact contracts"
        );
        Ok(())
    }
}
