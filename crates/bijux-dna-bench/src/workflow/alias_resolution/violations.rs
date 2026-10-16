//! Owner: bijux-dna-bench
//! Classify stale benchmark aliases in acquired readiness surfaces.

use std::collections::BTreeMap;

use bijux_dna_bench_model::{
    AllDomainActiveStageToolMatrixRow, AllDomainExpectedBenchmarkResultRow,
    AllDomainRenderedCommandRow,
};
use serde::Serialize;

use super::{legacy_benchmark_stage_alias_target, BenchmarkToolAliasCluster};

const ALIAS_KIND_STAGE: &str = "stage";
const ALIAS_KIND_TOOL: &str = "tool";

/// An observed stale spelling and its original benchmark surface identity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BenchmarkStageToolAliasViolation {
    pub alias_kind: String,
    pub alias_id: String,
    pub canonical_id: String,
    pub surface: String,
    pub domain: String,
    pub stage_id: String,
    pub tool_id: String,
    pub result_id: Option<String>,
    pub corpus_id: Option<String>,
    pub asset_profile_id: Option<String>,
}

struct AliasObservation<'a> {
    domain: &'a str,
    stage_id: &'a str,
    tool_id: &'a str,
    result_id: Option<&'a str>,
    corpus_id: &'a str,
    asset_profile_id: &'a str,
}

/// Derive stage and tool violations without acquiring or publishing evidence.
///
/// The caller supplies the same candidate-derived tool clusters used in its report.
/// Other surfaces do not expand that tool alias scope. Observations, including
/// duplicates and sort-key ties, retain the existing stable ordering and metadata.
#[must_use]
pub fn derive_benchmark_stage_tool_alias_violations(
    candidate_rows: &[AllDomainActiveStageToolMatrixRow],
    active_rows: &[AllDomainActiveStageToolMatrixRow],
    expected_rows: &[AllDomainExpectedBenchmarkResultRow],
    rendered_rows: &[AllDomainRenderedCommandRow],
    tool_alias_clusters: &[BenchmarkToolAliasCluster],
) -> Vec<BenchmarkStageToolAliasViolation> {
    let tool_alias_canonical_by_id = tool_alias_clusters
        .iter()
        .flat_map(|cluster| {
            cluster
                .alias_tool_ids
                .iter()
                .map(|alias_tool_id| (alias_tool_id.clone(), cluster.canonical_tool_id.clone()))
        })
        .collect::<BTreeMap<_, _>>();
    let candidates = candidate_rows.iter().map(|row| {
        (
            "candidate_matrix",
            AliasObservation {
                domain: &row.domain,
                stage_id: &row.stage_id,
                tool_id: &row.tool_id,
                result_id: None,
                corpus_id: &row.corpus_id,
                asset_profile_id: &row.asset_profile_id,
            },
        )
    });
    let active = active_rows.iter().map(|row| {
        (
            "active_matrix",
            AliasObservation {
                domain: &row.domain,
                stage_id: &row.stage_id,
                tool_id: &row.tool_id,
                result_id: None,
                corpus_id: &row.corpus_id,
                asset_profile_id: &row.asset_profile_id,
            },
        )
    });
    let expected = expected_rows.iter().map(|row| {
        (
            "expected_results",
            AliasObservation {
                domain: &row.domain,
                stage_id: &row.stage_id,
                tool_id: &row.tool_id,
                result_id: Some(&row.result_id),
                corpus_id: &row.corpus_id,
                asset_profile_id: &row.asset_profile_id,
            },
        )
    });
    let rendered = rendered_rows.iter().map(|row| {
        (
            "rendered_commands",
            AliasObservation {
                domain: &row.domain,
                stage_id: &row.stage_id,
                tool_id: &row.tool_id,
                result_id: Some(&row.result_id),
                corpus_id: &row.corpus_id,
                asset_profile_id: &row.asset_profile_id,
            },
        )
    });

    let mut violations = Vec::new();
    for (surface, row) in candidates.chain(active).chain(expected).chain(rendered) {
        push_stage_alias_violation(surface, &row, &mut violations);
        push_tool_alias_violation(surface, &tool_alias_canonical_by_id, &row, &mut violations);
    }
    violations.sort_by(|left, right| {
        left.surface
            .cmp(&right.surface)
            .then_with(|| left.domain.cmp(&right.domain))
            .then_with(|| left.stage_id.cmp(&right.stage_id))
            .then_with(|| left.tool_id.cmp(&right.tool_id))
            .then_with(|| left.alias_kind.cmp(&right.alias_kind))
            .then_with(|| left.alias_id.cmp(&right.alias_id))
    });
    violations
}

fn push_stage_alias_violation(
    surface: &str,
    row: &AliasObservation<'_>,
    violations: &mut Vec<BenchmarkStageToolAliasViolation>,
) {
    let Some(canonical_id) = legacy_benchmark_stage_alias_target(row.stage_id) else {
        return;
    };
    violations.push(BenchmarkStageToolAliasViolation {
        alias_kind: ALIAS_KIND_STAGE.to_string(),
        alias_id: row.stage_id.to_string(),
        canonical_id: canonical_id.to_string(),
        surface: surface.to_string(),
        domain: row.domain.to_string(),
        stage_id: row.stage_id.to_string(),
        tool_id: row.tool_id.to_string(),
        result_id: row.result_id.map(str::to_string),
        corpus_id: Some(row.corpus_id.to_string()),
        asset_profile_id: Some(row.asset_profile_id.to_string()),
    });
}

fn push_tool_alias_violation(
    surface: &str,
    tool_alias_canonical_by_id: &BTreeMap<String, String>,
    row: &AliasObservation<'_>,
    violations: &mut Vec<BenchmarkStageToolAliasViolation>,
) {
    let Some(canonical_id) = tool_alias_canonical_by_id.get(row.tool_id) else {
        return;
    };
    violations.push(BenchmarkStageToolAliasViolation {
        alias_kind: ALIAS_KIND_TOOL.to_string(),
        alias_id: row.tool_id.to_string(),
        canonical_id: canonical_id.clone(),
        surface: surface.to_string(),
        domain: row.domain.to_string(),
        stage_id: row.stage_id.to_string(),
        tool_id: row.tool_id.to_string(),
        result_id: row.result_id.map(str::to_string),
        corpus_id: Some(row.corpus_id.to_string()),
        asset_profile_id: Some(row.asset_profile_id.to_string()),
    });
}

#[cfg(test)]
mod tests {
    use bijux_dna_bench_model::{
        AllDomainActiveStageToolMatrixRow, AllDomainExpectedBenchmarkResultRow,
        AllDomainRenderedCommandRow,
    };

    use super::derive_benchmark_stage_tool_alias_violations;
    use crate::workflow::alias_resolution::derive_benchmark_tool_alias_clusters;

    fn matrix(stage: &str, tool: &str) -> AllDomainActiveStageToolMatrixRow {
        AllDomainActiveStageToolMatrixRow {
            domain: "fastq".to_string(),
            stage_id: stage.to_string(),
            tool_id: tool.to_string(),
            corpus_id: "reads".to_string(),
            asset_profile_id: "profile".to_string(),
            adapter_id: "adapter".to_string(),
            parser_id: "parser".to_string(),
            schema_id: "schema".to_string(),
            status: "benchmark_ready".to_string(),
        }
    }

    fn expected(stage: &str, tool: &str, result: &str) -> AllDomainExpectedBenchmarkResultRow {
        AllDomainExpectedBenchmarkResultRow {
            result_id: result.to_string(),
            domain: "fastq".to_string(),
            stage_id: stage.to_string(),
            tool_id: tool.to_string(),
            corpus_id: "reads".to_string(),
            asset_profile_id: "profile".to_string(),
            expected_outputs: Vec::new(),
            expected_metrics: Vec::new(),
            report_section: "reads".to_string(),
        }
    }

    fn rendered(stage: &str, tool: &str) -> AllDomainRenderedCommandRow {
        AllDomainRenderedCommandRow {
            result_id: "rendered-result".to_string(),
            domain: "fastq".to_string(),
            stage_id: stage.to_string(),
            tool_id: tool.to_string(),
            corpus_id: "reads".to_string(),
            asset_profile_id: "profile".to_string(),
            readiness_kind: "stage".to_string(),
            benchmark_status: "benchmark_ready".to_string(),
            command_source: "adapter".to_string(),
            command_steps: Vec::new(),
            script_commands: Vec::new(),
        }
    }

    #[test]
    fn alias_audit_routes_both_alias_kinds_across_all_four_surfaces() -> anyhow::Result<()> {
        let candidates = [matrix("trim", "shapeit_5"), matrix("fastq.trim_reads", "shapeit5")];
        let clusters =
            derive_benchmark_tool_alias_clusters(candidates.iter().map(|r| r.tool_id.as_str()));
        let violations = derive_benchmark_stage_tool_alias_violations(
            &candidates,
            &[matrix("trim", "shapeit_5")],
            &[expected("trim", "shapeit_5", "expected-result")],
            &[rendered("trim", "shapeit_5")],
            &clusters,
        );
        let wire = serde_json::to_value(&violations)?;
        let mut wanted = Vec::new();
        for (surface, result) in [
            ("active_matrix", None),
            ("candidate_matrix", None),
            ("expected_results", Some("expected-result")),
            ("rendered_commands", Some("rendered-result")),
        ] {
            for (kind, alias, canonical) in
                [("stage", "trim", "fastq.trim_reads"), ("tool", "shapeit_5", "shapeit5")]
            {
                wanted.push(serde_json::json!({"alias_kind":kind, "alias_id":alias,
                    "canonical_id":canonical, "surface":surface, "domain":"fastq",
                    "stage_id":"trim", "tool_id":"shapeit_5", "result_id":result,
                    "corpus_id":"reads", "asset_profile_id":"profile"}));
            }
        }
        assert_eq!(wire, serde_json::Value::Array(wanted));
        Ok(())
    }

    #[test]
    fn alias_audit_keeps_candidate_cluster_scope_and_exact_stage_lookup() {
        let candidates = [matrix("fastq.trim_reads", "shapeit5")];
        let clusters =
            derive_benchmark_tool_alias_clusters(candidates.iter().map(|r| r.tool_id.as_str()));
        assert!(derive_benchmark_stage_tool_alias_violations(
            &candidates,
            &[matrix("TRIM", "shapeit_5")],
            &[expected("fastq.trim_reads", "shapeit-5", "expected-result")],
            &[rendered("fastq.trim_reads", "SHAPEIT5")],
            &clusters,
        )
        .is_empty());
        assert!(derive_benchmark_stage_tool_alias_violations(&[], &[], &[], &[], &[]).is_empty());
    }

    #[test]
    fn alias_audit_preserves_stable_sort_ties_and_duplicate_observations() {
        let expected_rows = [
            expected("trim", "fastp", "second"),
            expected("trim", "fastp", "first"),
            expected("trim", "fastp", "first"),
        ];
        let violations = derive_benchmark_stage_tool_alias_violations(
            &[],
            &[matrix("stats", "fastp"), matrix("filter", "fastp")],
            &expected_rows,
            &[],
            &[],
        );
        let actual = violations
            .iter()
            .map(|row| (row.surface.as_str(), row.stage_id.as_str(), row.result_id.as_deref()))
            .collect::<Vec<_>>();
        assert_eq!(
            actual,
            vec![
                ("active_matrix", "filter", None),
                ("active_matrix", "stats", None),
                ("expected_results", "trim", Some("second")),
                ("expected_results", "trim", Some("first")),
                ("expected_results", "trim", Some("first"))
            ]
        );
    }
}
