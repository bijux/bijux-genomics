//! Owner: bijux-dna-bench
//! BAM score artifact adapters; filesystem acquisition belongs to the repository owner.

use bijux_dna_bench_model::BamScoreEvidence as BamEvidenceAggregate;
use std::collections::BTreeMap;

pub(crate) type EvidenceRows = BTreeMap<(String, String), BamEvidenceAggregate>;

mod alignment;
mod correction;
mod damage;
mod duplicates_coverage;
mod fields;
mod identity;
mod library;
mod merge;

pub(crate) use alignment::{
    merge_filter_summary, merge_length_filter_summary, merge_mapping_summary,
    merge_mapq_filter_summary, merge_qc_pre_summary, merge_validate_summary,
};
pub(crate) use correction::{
    merge_gc_bias_summary, merge_overlap_correction_summary, merge_recalibration_summary,
};
pub(crate) use damage::{merge_authenticity_summary, merge_damage_summary};
pub(crate) use duplicates_coverage::{
    merge_coverage_summary, merge_duplication_metrics_summary, merge_markdup_summary,
};
pub(crate) use fields::parse_tsv_rows;
pub(crate) use identity::{
    merge_contamination_summary, merge_haplogroups_summary, merge_kinship_summary,
    merge_sex_summary,
};
pub(crate) use library::{
    merge_complexity_summary, merge_endogenous_content_summary, merge_insert_size_summary,
};

#[cfg(test)]
mod tests {
    use super::{merge_qc_pre_summary, merge_validate_summary, EvidenceRows};
    use std::path::Path;

    #[test]
    fn bam_score_adapters_preserve_observed_binding_and_metric_units() -> anyhow::Result<()> {
        let report = serde_json::json!({"stage_id":"bam.qc_pre","cases":[{
            "tool_id":"samtools","total_reads":20,"mapped_reads":15,"unmapped_reads":5,
            "duplicate_flagged_reads":2,"expectation_matched":true
        }]});
        let mut rows = EvidenceRows::new();
        merge_qc_pre_summary(&report, Path::new("observed/qc.json"), &mut rows)?;
        let row = rows
            .get(&("bam.qc_pre".into(), "samtools".into()))
            .ok_or_else(|| anyhow::anyhow!("missing observed binding"))?;
        assert_eq!(rows.len(), 1);
        assert_eq!(row.retained_reads, Some(20));
        assert_eq!(row.truth_correctness_score, Some(0.75));
        assert_eq!(row.contract_correctness_score, Some(1.0));
        assert!(row.source_paths.contains("observed/qc.json"));
        Ok(())
    }

    #[test]
    fn bam_score_adapters_reject_missing_required_observations() {
        for report in [
            serde_json::json!({}),
            serde_json::json!({"stage_id":"bam.qc_pre","cases":[{"tool_id":"samtools"}]}),
        ] {
            assert!(merge_qc_pre_summary(
                &report,
                Path::new("observed/qc.json"),
                &mut EvidenceRows::new()
            )
            .is_err());
        }
    }

    #[test]
    fn bam_score_adapters_preserve_case_expectation_and_read_denominators() -> anyhow::Result<()> {
        let report = serde_json::json!({"stage_id":"bam.validate","case_count":2,"all_cases_matched":false,"cases":[
            {"expectation_matched":true,"validation_status":"pass","total_reads":7},
            {"expectation_matched":false,"validation_status":"refusal","refusal_codes":["missing_index"],"total_reads":3}
        ]});
        let mut rows = EvidenceRows::new();
        merge_validate_summary(&report, Path::new("observed/validation.json"), &mut rows)?;
        let row = rows
            .get(&("bam.validate".into(), "samtools".into()))
            .ok_or_else(|| anyhow::anyhow!("missing validation evidence"))?;
        assert_eq!(row.truth_correctness_score, Some(0.5));
        assert_eq!(row.alignment_qc_metric_value, Some(0.5));
        assert_eq!(row.retained_reads, Some(10));
        assert_eq!(row.contract_correctness_score, Some(0.0));
        Ok(())
    }

    fn validation_with_read_counts(first: u64, second: u64) -> serde_json::Value {
        serde_json::json!({"stage_id":"bam.validate","case_count":2,"all_cases_matched":true,"cases":[
            {"expectation_matched":true,"validation_status":"pass","total_reads":first},
            {"expectation_matched":true,"validation_status":"pass","total_reads":second}
        ]})
    }

    #[test]
    fn bam_score_adapters_reject_overflow_and_inconsistent_validation_counts() -> anyhow::Result<()>
    {
        let path = Path::new("observed/validation.json");
        let mut rows = EvidenceRows::new();
        merge_validate_summary(&validation_with_read_counts(u64::MAX, 0), path, &mut rows)?;
        assert_eq!(rows.values().next().and_then(|row| row.retained_reads), Some(u64::MAX));
        assert!(merge_validate_summary(
            &validation_with_read_counts(u64::MAX, 1),
            path,
            &mut EvidenceRows::new()
        )
        .is_err());
        let mut wrong_count = validation_with_read_counts(7, 3);
        wrong_count["case_count"] = serde_json::json!(3);
        assert!(merge_validate_summary(&wrong_count, path, &mut EvidenceRows::new()).is_err());
        let mut wrong_summary = validation_with_read_counts(7, 3);
        wrong_summary["all_cases_matched"] = serde_json::json!(false);
        assert!(merge_validate_summary(&wrong_summary, path, &mut EvidenceRows::new()).is_err());
        Ok(())
    }

    #[test]
    fn bam_score_adapters_reject_invalid_coverage_totals_and_keep_depth_units() -> anyhow::Result<()>
    {
        use super::{merge_coverage_summary, parse_tsv_rows};
        let path = Path::new("observed/coverage.tsv");
        for (first, second, depth, pass) in [
            (u64::MAX, 0, "35", true),
            (u64::MAX, 1, "35", false),
            (1, 1, "NaN", false),
            (1, 1, "inf", false),
            (1, 1, "1e308", false),
        ] {
            let raw=format!("breadth_1x\tmean_depth\tcovered_bases\tcase_expectation_matched\n0.5\t{depth}\t{first}\ttrue\n0.5\t{depth}\t{second}\ttrue\n");
            let values = parse_tsv_rows(&raw, path)?;
            let mut rows = EvidenceRows::new();
            let result = merge_coverage_summary(&values, path, &mut rows);
            assert_eq!(result.is_ok(), pass, "{raw}: {result:?}");
            if pass {
                assert_eq!(
                    rows.values().next().and_then(|row| row.coverage_metric_value),
                    Some(35.0)
                );
            }
        }
        Ok(())
    }
    #[test]
    fn bam_score_adapters_reject_cross_stage_and_noncanonical_tool_observations(
    ) -> anyhow::Result<()> {
        let valid = serde_json::json!({"stage_id":"bam.qc_pre","cases":[{
            "tool_id":"samtools","total_reads":20,"mapped_reads":15,"unmapped_reads":5,
            "duplicate_flagged_reads":2,"expectation_matched":true
        }]});
        for stage in ["bam.filter", "vcf.qc", "", " bam.qc_pre", "bam.qc_pre "] {
            let mut value = valid.clone();
            value["stage_id"] = serde_json::json!(stage);
            let mut rows = EvidenceRows::new();
            assert!(merge_qc_pre_summary(&value, Path::new("qc.json"), &mut rows).is_err());
            assert!(rows.is_empty());
        }
        for tool in [
            serde_json::Value::Null,
            serde_json::json!(""),
            serde_json::json!(" samtools"),
            serde_json::json!("samtools "),
        ] {
            let mut value = valid.clone();
            value["cases"][0]["tool_id"] = tool.clone();
            assert!(merge_qc_pre_summary(&value, Path::new("qc.json"), &mut EvidenceRows::new())
                .is_err());
            for (stage, adapter) in [
                (
                    "bam.sex",
                    super::merge_sex_summary
                        as fn(&serde_json::Value, &Path, &mut EvidenceRows) -> anyhow::Result<()>,
                ),
                ("bam.contamination", super::merge_contamination_summary),
            ] {
                let value = serde_json::json!({"stage_id":stage,"rows":[{"tool_id":tool,"proof_case":"unavailable"}]});
                assert!(
                    adapter(&value, Path::new("identity.json"), &mut EvidenceRows::new()).is_err()
                );
            }
        }
        merge_qc_pre_summary(&valid, Path::new("qc.json"), &mut EvidenceRows::new())?;
        Ok(())
    }
}
