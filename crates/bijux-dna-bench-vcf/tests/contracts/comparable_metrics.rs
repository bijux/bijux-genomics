use bijux_dna_bench_vcf::evidence::metrics::{
    render_vcf_comparable_metrics, DEFAULT_VCF_COMPARABLE_METRICS_PATH,
    VCF_COMPARABLE_METRICS_SCHEMA_V1,
};
use std::path::PathBuf;
fn repo_root() -> PathBuf {
    crate::support::repo_root()
}
#[test]
fn render_vcf_comparable_metrics_reports_governed_metric_rows() {
    let root = repo_root();
    let inputs = crate::support::inputs(&root);
    let observations =
        bijux_dna_bench_vcf::adapters::observations::VcfAdapterObservations::new(&inputs, &root);
    let report = render_vcf_comparable_metrics(
        &observations,
        &PathBuf::from(DEFAULT_VCF_COMPARABLE_METRICS_PATH),
    )
    .expect("render VCF comparable metrics");

    assert_eq!(report.schema_version, VCF_COMPARABLE_METRICS_SCHEMA_V1);
    assert_eq!(report.output_path, DEFAULT_VCF_COMPARABLE_METRICS_PATH);
    assert_eq!(report.stage_count, 10);
    assert_eq!(report.multi_tool_stage_count, 10);
    assert_eq!(report.retained_tool_row_count, 29);
    assert_eq!(report.row_count, 29);

    assert!(report.rows.iter().any(|row| {
        row.stage_id == "vcf.call_gl"
            && row.metric_id == "sites_with_likelihoods"
            && row.metric_name == "sites with likelihoods"
            && row.unit == "sites"
            && row.direction.as_str() == "higher_is_better"
            && row.required
            && row.tools_covered == vec!["angsd".to_string(), "bcftools".to_string()]
    }));
    assert!(report.rows.iter().any(|row| {
        row.stage_id == "vcf.qc"
            && row.metric_id == "concordance"
            && row.unit == "fraction"
            && row.direction.as_str() == "higher_is_better"
            && row.tools_covered
                == vec!["bcftools".to_string(), "plink".to_string(), "plink2".to_string()]
    }));
    assert!(report.rows.iter().any(|row| {
        row.stage_id == "vcf.phasing"
            && row.metric_id == "switch_error_proxy"
            && row.direction.as_str() == "lower_is_better"
            && row.tools_covered
                == vec![
                    "beagle".to_string(),
                    "eagle".to_string(),
                    "shapeit".to_string(),
                    "shapeit5".to_string(),
                ]
    }));
    assert!(report.rows.iter().any(|row| {
        row.stage_id == "vcf.impute"
            && row.metric_id == "masked_truth_match_count"
            && row.direction.as_str() == "higher_is_better"
            && row.tools_covered
                == vec![
                    "beagle".to_string(),
                    "glimpse".to_string(),
                    "impute5".to_string(),
                    "minimac4".to_string(),
                ]
    }));
    assert!(report.rows.iter().any(|row| {
        row.stage_id == "vcf.ibd"
            && row.metric_id == "pair_count"
            && row.direction.as_str() == "exact_match_preferred"
            && row.tools_covered
                == vec!["germline".to_string(), "ibdhap".to_string(), "ibdseq".to_string()]
    }));
}
