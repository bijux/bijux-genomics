//! Owner: bijux-dna-bench
//! Canonical benchmark stage and tool alias resolution policy.

use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LegacyBenchmarkStageAlias {
    pub alias_stage_id: &'static str,
    pub canonical_stage_id: &'static str,
}

/// Distinct benchmark tool spellings that share one normalized identity.
///
/// Field order and names retain the nested stage/tool alias-check wire representation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BenchmarkToolAliasCluster {
    /// Normalized lookup identity shared by every member.
    pub normalized_tool_id: String,
    /// Preferred declared spelling chosen by the existing separator/case policy.
    pub canonical_tool_id: String,
    /// Other distinct spellings in lexical order, excluding the canonical identity.
    pub alias_tool_ids: Vec<String>,
}

const LEGACY_BENCHMARK_STAGE_ALIASES: &[LegacyBenchmarkStageAlias] = &[
    LegacyBenchmarkStageAlias {
        alias_stage_id: "validate_pre",
        canonical_stage_id: "fastq.validate_reads",
    },
    LegacyBenchmarkStageAlias { alias_stage_id: "trim", canonical_stage_id: "fastq.trim_reads" },
    LegacyBenchmarkStageAlias {
        alias_stage_id: "filter",
        canonical_stage_id: "fastq.filter_reads",
    },
    LegacyBenchmarkStageAlias {
        alias_stage_id: "stats",
        canonical_stage_id: "fastq.profile_reads",
    },
    LegacyBenchmarkStageAlias {
        alias_stage_id: "report_qc",
        canonical_stage_id: "fastq.report_qc",
    },
    LegacyBenchmarkStageAlias { alias_stage_id: "qc_post", canonical_stage_id: "fastq.report_qc" },
    LegacyBenchmarkStageAlias {
        alias_stage_id: concat!("fastq", ".qc_post"),
        canonical_stage_id: "fastq.report_qc",
    },
];

const REQUIRED_MIGRATION_STAGE_IDS: &[&str] = &[
    "fastq.validate_reads",
    "fastq.trim_reads",
    "fastq.filter_reads",
    "fastq.profile_reads",
    "fastq.report_qc",
];

#[must_use]
pub fn legacy_benchmark_stage_aliases() -> &'static [LegacyBenchmarkStageAlias] {
    LEGACY_BENCHMARK_STAGE_ALIASES
}

#[must_use]
pub fn required_migration_stage_ids() -> &'static [&'static str] {
    REQUIRED_MIGRATION_STAGE_IDS
}

#[must_use]
pub fn legacy_benchmark_stage_alias_target(stage_id: &str) -> Option<&'static str> {
    legacy_benchmark_stage_aliases()
        .iter()
        .find(|alias| alias.alias_stage_id == stage_id)
        .map(|alias| alias.canonical_stage_id)
}

#[must_use]
pub fn stage_set_contains_canonical_or_migration_alias(
    stage_set: &BTreeSet<&str>,
    canonical_stage_id: &str,
) -> bool {
    stage_set.contains(canonical_stage_id)
        || legacy_benchmark_stage_aliases()
            .iter()
            .filter(|alias| alias.canonical_stage_id == canonical_stage_id)
            .any(|alias| stage_set.contains(alias.alias_stage_id))
}

#[must_use]
pub fn normalize_tool_id(tool_id: &str) -> String {
    tool_id
        .chars()
        .filter(|character| *character != '-' && *character != '_')
        .flat_map(char::to_lowercase)
        .collect()
}

#[must_use]
pub fn choose_canonical_tool_id(cluster_tool_ids: &[String]) -> Option<String> {
    cluster_tool_ids.iter().min_by_key(|tool_id| canonical_tool_id_preference(tool_id)).cloned()
}

/// Derive distinct alias clusters from the caller-selected benchmark tool scope.
///
/// Repeated identical IDs and singleton normalized groups do not create aliases.
/// Group, canonical and alias ordering use the existing deterministic tool policy.
#[must_use]
pub fn derive_benchmark_tool_alias_clusters<'a>(
    tool_ids: impl IntoIterator<Item = &'a str>,
) -> Vec<BenchmarkToolAliasCluster> {
    let mut tool_ids_by_normalized = BTreeMap::<String, BTreeSet<String>>::new();
    for tool_id in tool_ids {
        tool_ids_by_normalized
            .entry(normalize_tool_id(tool_id))
            .or_default()
            .insert(tool_id.to_string());
    }

    let mut rows = Vec::new();
    for (normalized_tool_id, cluster) in tool_ids_by_normalized {
        if cluster.len() <= 1 {
            continue;
        }
        let cluster_tool_ids = cluster.into_iter().collect::<Vec<_>>();
        let Some(canonical_tool_id) = choose_canonical_tool_id(&cluster_tool_ids) else {
            continue;
        };
        let alias_tool_ids = cluster_tool_ids
            .into_iter()
            .filter(|tool_id| tool_id != &canonical_tool_id)
            .collect::<Vec<_>>();
        rows.push(BenchmarkToolAliasCluster {
            normalized_tool_id,
            canonical_tool_id,
            alias_tool_ids,
        });
    }
    rows.sort_by(|left, right| left.normalized_tool_id.cmp(&right.normalized_tool_id));
    rows
}

fn canonical_tool_id_preference(tool_id: &str) -> (usize, usize, String) {
    let hyphen_count = tool_id.matches('-').count();
    let underscore_count = tool_id.matches('_').count();
    let separator_penalty = hyphen_count + underscore_count;
    (separator_penalty, hyphen_count, tool_id.to_ascii_lowercase())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::{
        choose_canonical_tool_id, legacy_benchmark_stage_alias_target, normalize_tool_id,
        required_migration_stage_ids, stage_set_contains_canonical_or_migration_alias,
    };

    #[test]
    fn migration_alias_target_maps_legacy_stage_ids() {
        assert_eq!(legacy_benchmark_stage_alias_target("trim"), Some("fastq.trim_reads"));
        assert_eq!(
            legacy_benchmark_stage_alias_target(concat!("fastq", ".qc_post")),
            Some("fastq.report_qc")
        );
        assert_eq!(legacy_benchmark_stage_alias_target("fastq.trim_reads"), None);
    }

    #[test]
    fn migration_stage_set_accepts_canonical_or_legacy_ids() {
        let canonical_only = BTreeSet::from(["fastq.trim_reads"]);
        assert!(stage_set_contains_canonical_or_migration_alias(
            &canonical_only,
            "fastq.trim_reads"
        ));

        let legacy_only = BTreeSet::from(["trim"]);
        assert!(stage_set_contains_canonical_or_migration_alias(&legacy_only, "fastq.trim_reads"));

        let qc_alias_only = BTreeSet::from(["qc_post"]);
        assert!(stage_set_contains_canonical_or_migration_alias(&qc_alias_only, "fastq.report_qc"));
    }

    #[test]
    fn required_stage_ids_keep_canonical_contract() {
        assert_eq!(
            required_migration_stage_ids(),
            &[
                "fastq.validate_reads",
                "fastq.trim_reads",
                "fastq.filter_reads",
                "fastq.profile_reads",
                "fastq.report_qc",
            ]
        );
    }

    #[test]
    fn tool_normalization_folds_separator_aliases() {
        assert_eq!(normalize_tool_id("bowtie2-build"), "bowtie2build");
        assert_eq!(normalize_tool_id("bowtie2_build"), "bowtie2build");
    }

    #[test]
    fn canonical_tool_preference_favors_fewer_separators() {
        let cluster =
            vec!["shapeit-5".to_string(), "shapeit_5".to_string(), "shapeit5".to_string()];
        assert_eq!(choose_canonical_tool_id(&cluster), Some("shapeit5".to_string()));
        assert_eq!(choose_canonical_tool_id(&[]), None);
    }

    #[test]
    fn tool_alias_clusters_keep_distinct_members_and_canonical_precedence() -> anyhow::Result<()> {
        use super::derive_benchmark_tool_alias_clusters;

        let rows = derive_benchmark_tool_alias_clusters([
            "shapeit_5",
            "solo",
            "bowtie2-build",
            "shapeit5",
            "solo",
            "shapeit-5",
            "bowtie2_build",
            "shapeit5",
        ]);
        assert_eq!(
            serde_json::to_value(&rows)?,
            serde_json::json!([
                {"normalized_tool_id":"bowtie2build", "canonical_tool_id":"bowtie2_build",
                 "alias_tool_ids":["bowtie2-build"]},
                {"normalized_tool_id":"shapeit5", "canonical_tool_id":"shapeit5",
                 "alias_tool_ids":["shapeit-5", "shapeit_5"]},
            ])
        );
        assert!(derive_benchmark_tool_alias_clusters(["solo", "solo"]).is_empty());
        assert!(derive_benchmark_tool_alias_clusters(std::iter::empty::<&str>()).is_empty());
        Ok(())
    }

    #[test]
    fn tool_alias_clusters_ignore_input_order_and_keep_case_tie_order() {
        use super::{derive_benchmark_tool_alias_clusters, BenchmarkToolAliasCluster};

        let forward = derive_benchmark_tool_alias_clusters(["fastp", "FASTP", "fastp"]);
        let reverse = derive_benchmark_tool_alias_clusters(["FASTP", "fastp", "FASTP"]);
        assert_eq!(forward, reverse);
        assert_eq!(
            forward,
            vec![BenchmarkToolAliasCluster {
                normalized_tool_id: "fastp".to_string(),
                canonical_tool_id: "FASTP".to_string(),
                alias_tool_ids: vec!["fastp".to_string()],
            }]
        );
    }
}
