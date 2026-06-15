use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LegacyBenchmarkStageAlias {
    pub(crate) alias_stage_id: &'static str,
    pub(crate) canonical_stage_id: &'static str,
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
        alias_stage_id: "fastq.qc_post",
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

pub(crate) fn legacy_benchmark_stage_aliases() -> &'static [LegacyBenchmarkStageAlias] {
    LEGACY_BENCHMARK_STAGE_ALIASES
}

pub(crate) fn required_migration_stage_ids() -> &'static [&'static str] {
    REQUIRED_MIGRATION_STAGE_IDS
}

pub(crate) fn legacy_benchmark_stage_alias_target(stage_id: &str) -> Option<&'static str> {
    legacy_benchmark_stage_aliases()
        .iter()
        .find(|alias| alias.alias_stage_id == stage_id)
        .map(|alias| alias.canonical_stage_id)
}

pub(crate) fn stage_set_contains_canonical_or_migration_alias(
    stage_set: &BTreeSet<&str>,
    canonical_stage_id: &str,
) -> bool {
    stage_set.contains(canonical_stage_id)
        || legacy_benchmark_stage_aliases()
            .iter()
            .filter(|alias| alias.canonical_stage_id == canonical_stage_id)
            .any(|alias| stage_set.contains(alias.alias_stage_id))
}

pub(crate) fn normalize_tool_id(tool_id: &str) -> String {
    tool_id
        .chars()
        .filter(|character| *character != '-' && *character != '_')
        .flat_map(char::to_lowercase)
        .collect()
}

pub(crate) fn choose_canonical_tool_id(cluster_tool_ids: &[String]) -> String {
    cluster_tool_ids
        .iter()
        .min_by_key(|tool_id| canonical_tool_id_preference(tool_id))
        .cloned()
        .expect("canonical tool id")
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
        assert_eq!(legacy_benchmark_stage_alias_target("fastq.qc_post"), Some("fastq.report_qc"));
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
        assert_eq!(choose_canonical_tool_id(&cluster), "shapeit5");
    }
}
