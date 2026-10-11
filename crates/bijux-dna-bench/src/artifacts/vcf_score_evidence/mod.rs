//! Owner: bijux-dna-bench
//! Scientific VCF score adapters, independent of report acquisition.

use bijux_dna_bench_model::VcfScoreEvidence as VcfEvidenceAggregate;
use std::collections::BTreeMap;
pub(crate) type EvidenceRows = BTreeMap<(String, String), VcfEvidenceAggregate>;

mod fields;
mod genotypes;
mod merge;
pub(crate) use genotypes::{
    merge_call_gl_summary, merge_call_pseudohaploid_summary, merge_call_summary,
    merge_gl_propagation_summary, CallCountContract,
};
mod quality;
pub(crate) use quality::{
    merge_damage_filter_summary, merge_filter_summary, merge_postprocess_summary, merge_qc_summary,
    merge_stats_summary,
};
mod imputation;
pub(crate) use imputation::{
    merge_imputation_metrics_summary, merge_impute_summary, merge_phasing_summary,
    merge_prepare_reference_panel_summary,
};
mod population;
pub(crate) use population::{
    merge_admixture_summary, merge_pca_summary, merge_population_structure_summary,
};
mod relatedness;
pub(crate) use relatedness::{merge_demography_summary, merge_ibd_summary, merge_roh_summary};

#[cfg(test)]
mod tests {
    use super::{
        merge_call_summary, merge_imputation_metrics_summary, merge_phasing_summary, EvidenceRows,
    };
    use std::path::Path;

    #[test]
    fn vcf_score_adapters_preserve_call_count_units_and_missing_field_rejection(
    ) -> anyhow::Result<()> {
        let value = serde_json::json!({"stage_id":"vcf.call", "tool_id":"bcftools", "variant_count":35, "sample_count":2, "exit_code":0});
        let mut rows = EvidenceRows::new();
        merge_call_summary(
            &value,
            Path::new("call/metrics.json"),
            &mut rows,
            super::CallCountContract::Variants,
        )?;
        let row = rows
            .get(&("vcf.call".into(), "bcftools".into()))
            .ok_or_else(|| anyhow::anyhow!("missing call observation"))?;
        assert_eq!(row.genotype_truth_metric_value, Some(35.0));
        assert_eq!(row.genotype_truth_metric_basis.as_deref(), Some("variant_count"));
        assert_eq!(row.contract_correctness_score, Some(1.0));
        assert_eq!(row.truth_correctness_score, None);
        assert!(row.source_paths.contains("call/metrics.json"));
        let mut missing = value;
        missing
            .as_object_mut()
            .ok_or_else(|| anyhow::anyhow!("expected object"))?
            .remove("variant_count");
        assert!(merge_call_summary(
            &missing,
            Path::new("call/metrics.json"),
            &mut EvidenceRows::new(),
            super::CallCountContract::Variants
        )
        .is_err());
        Ok(())
    }

    #[test]
    fn vcf_score_adapters_preserve_phasing_failure_and_source_precedence() -> anyhow::Result<()> {
        let metrics = serde_json::json!({"stage_id":"vcf.phasing","tool_id":"shapeit5","input_genotypes":10,"phased_genotypes":8,"unphased_genotypes":2,"phase_set_count":2,"sample_count":1,"exit_code":0});
        let result = serde_json::json!({"runtime":{"status":"failed","elapsed_seconds":2.5}});
        let mut rows = EvidenceRows::new();
        merge_phasing_summary(
            &metrics,
            Path::new("phasing/metrics.json"),
            Some(&result),
            Path::new("phasing/stage-result.json"),
            &mut rows,
        )?;
        let row = rows
            .get(&("vcf.phasing".into(), "shapeit5".into()))
            .ok_or_else(|| anyhow::anyhow!("missing phasing observation"))?;
        assert_eq!(row.contract_correctness_score, Some(0.0));
        assert_eq!(row.phasing_imputation_metric_value, Some(0.8));
        assert_eq!(row.runtime_seconds, Some(2.5));
        assert_eq!(row.runtime_source.as_deref(), Some("stage_result_manifest"));
        assert_eq!(row.source_paths.len(), 2);
        assert!(row.source_paths.contains("phasing/stage-result.json"));
        let mut without_result = EvidenceRows::new();
        merge_phasing_summary(
            &metrics,
            Path::new("phasing/metrics.json"),
            None,
            Path::new("phasing/stage-result.json"),
            &mut without_result,
        )?;
        let row = without_result
            .values()
            .next()
            .ok_or_else(|| anyhow::anyhow!("missing fallback observation"))?;
        assert_eq!(row.contract_correctness_score, Some(1.0));
        assert_eq!(row.runtime_seconds, None);
        assert_eq!(row.source_paths.len(), 1);
        Ok(())
    }

    #[test]
    fn vcf_score_adapters_preserve_imputation_metric_meanings() -> anyhow::Result<()> {
        let value = serde_json::json!({"stage_id":"vcf.imputation_metrics","tool_id":"beagle","concordance":0.9,"dosage_r2":0.8,"mean_info_score":0.7,"low_confidence_sites":3,"masked_truth_sites":30,"elapsed_seconds":4.0,"exit_code":0});
        let mut rows = EvidenceRows::new();
        merge_imputation_metrics_summary(&value, Path::new("imputation/metrics.json"), &mut rows)?;
        let row = rows
            .values()
            .next()
            .ok_or_else(|| anyhow::anyhow!("missing imputation observation"))?;
        assert_eq!(row.truth_correctness_score, Some(0.9));
        assert_eq!(row.phasing_imputation_metric_value, Some(0.8));
        assert_eq!(row.phasing_imputation_metric_basis.as_deref(), Some("dosage_r2"));
        assert_eq!(row.runtime_seconds, Some(4.0));
        Ok(())
    }
    #[test]
    fn vcf_score_adapters_reject_misbound_call_contracts_and_noncanonical_tools(
    ) -> anyhow::Result<()> {
        let valid = serde_json::json!({"stage_id":"vcf.call","tool_id":"bcftools","variant_count":3,"sample_count":1,"exit_code":0});
        for stage in ["vcf.call_diploid", "vcf.qc", "vcf.call ", ""] {
            let mut value = valid.clone();
            value["stage_id"] = serde_json::json!(stage);
            assert!(merge_call_summary(
                &value,
                Path::new("metrics.json"),
                &mut EvidenceRows::new(),
                super::CallCountContract::Variants
            )
            .is_err());
        }
        for tool in ["", " bcftools", "bcftools "] {
            let mut value = valid.clone();
            value["tool_id"] = serde_json::json!(tool);
            assert!(merge_call_summary(
                &value,
                Path::new("metrics.json"),
                &mut EvidenceRows::new(),
                super::CallCountContract::Variants
            )
            .is_err());
        }
        let diploid = serde_json::json!({"stage_id":"vcf.call_diploid","tool_id":"bcftools","called_genotypes":8,"sample_count":1,"exit_code":0});
        let mut rows = EvidenceRows::new();
        merge_call_summary(
            &diploid,
            Path::new("metrics.json"),
            &mut rows,
            super::CallCountContract::DiploidGenotypes,
        )?;
        let row =
            rows.values().next().ok_or_else(|| anyhow::anyhow!("missing diploid observation"))?;
        assert_eq!(row.genotype_truth_metric_value, Some(8.0));
        assert_eq!(row.genotype_truth_metric_basis.as_deref(), Some("called_genotypes"));
        Ok(())
    }

    #[test]
    fn vcf_score_adapters_reject_unknown_qc_report_schema() -> anyhow::Result<()> {
        let valid = serde_json::json!({"schema_version":"bijux.vcf.qc.v1","missingness_pre":0.2,"missingness_post":0.1,"imputation_info_mean":0.8,"rsq_mean":0.7,"excluded_samples":[],"excluded_variants":[]});
        super::merge_qc_summary(&valid, Path::new("qc.json"), &mut EvidenceRows::new(), "plink2")?;
        for schema in
            [serde_json::Value::Null, serde_json::json!("bijux.vcf.qc.v999"), serde_json::json!("")]
        {
            let mut value = valid.clone();
            value["schema_version"] = schema;
            assert!(super::merge_qc_summary(
                &value,
                Path::new("qc.json"),
                &mut EvidenceRows::new(),
                "plink2"
            )
            .is_err());
        }
        Ok(())
    }
    #[test]
    fn vcf_score_adapters_admit_versioned_stats_without_stage_envelope_fields() -> anyhow::Result<()>
    {
        let mut value = serde_json::json!({"schema_version":"bijux.vcf.stats.v1","variants_total":35,"snps":30,"indels":5,"missingness_post":0.1,"annotation_coverage":1.0,"ti_tv":1.5,"heterozygosity_ratio":0.4});
        let mut rows = EvidenceRows::new();
        super::merge_stats_summary(&value, Path::new("stats.json"), &mut rows)?;
        let row = rows
            .get(&("vcf.stats".into(), "bcftools".into()))
            .ok_or_else(|| anyhow::anyhow!("missing stats observation"))?;
        assert_eq!(row.contract_correctness_score, Some(1.0));
        assert_eq!(row.missingness_metric_value, Some(0.9));
        value["schema_version"] = serde_json::json!("bijux.vcf.stats.v999");
        assert!(super::merge_stats_summary(
            &value,
            Path::new("stats.json"),
            &mut EvidenceRows::new()
        )
        .is_err());
        Ok(())
    }
    #[test]
    fn vcf_score_adapters_reject_overflow_before_failure_scoring() -> anyhow::Result<()> {
        type Adapter = fn(&serde_json::Value, &Path, &mut EvidenceRows) -> anyhow::Result<()>;
        let cases: [(Adapter, serde_json::Value, &str, &str, &str); 3] = [
            (
                super::merge_call_gl_summary,
                serde_json::json!({"stage_id":"vcf.call_gl","tool_id":"angsd","sites_with_likelihoods":3,"missing_likelihoods":2,"exit_code":0,"sample_count":1,"samples_with_likelihoods":1,"likelihood_field":"GL"}),
                "sites_with_likelihoods",
                "missing_likelihoods",
                "likelihood sites",
            ),
            (
                super::merge_filter_summary,
                serde_json::json!({"stage_id":"vcf.filter","tool_id":"bcftools","input_variants":5,"pass_variants":3,"failed_variants":2,"exit_code":1,"quality_threshold":10.0,"depth_threshold":2.0}),
                "pass_variants",
                "failed_variants",
                "filtered variants",
            ),
            (
                super::merge_stats_summary,
                serde_json::json!({"schema_version":"bijux.vcf.stats.v1","variants_total":5,"snps":3,"indels":2,"annotation_coverage":0.0,"missingness_post":0.1,"ti_tv":1.5,"heterozygosity_ratio":0.4}),
                "snps",
                "indels",
                "classified variants",
            ),
        ];
        for (adapter, mut value, left, right, label) in cases {
            adapter(&value, Path::new("observed.json"), &mut EvidenceRows::new())?;
            value[left] = serde_json::json!(u64::MAX);
            value[right] = serde_json::json!(1);
            let mut rows = EvidenceRows::new();
            let error = adapter(&value, Path::new("observed.json"), &mut rows)
                .expect_err("overflowing observation must be rejected");
            assert!(error.to_string().contains(label), "{error:#}");
            assert!(rows.is_empty(), "invalid observation entered the score index");
        }
        Ok(())
    }
}
