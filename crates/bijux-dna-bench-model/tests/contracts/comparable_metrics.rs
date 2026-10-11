use bijux_dna_bench_model::{
    bam_comparable_metrics_schema, fastq_comparable_metrics_schema,
    validate_bam_comparable_metrics_report, validate_fastq_comparable_metrics_report,
    validate_vcf_comparable_metrics_report, vcf_comparable_metrics_schema,
    BamComparableMetricsReport, BamComparableMetricsRow, ComparableMetricContractStatus,
    ComparableMetricDirection, FastqComparableMetricsReport, FastqComparableMetricsRow,
    VcfComparableMetricsReport, VcfComparableMetricsRow, BAM_COMPARABLE_METRICS_SCHEMA_V1,
    FASTQ_COMPARABLE_METRICS_SCHEMA_V1, VCF_COMPARABLE_METRICS_SCHEMA_V1,
};

fn fastq_report() -> FastqComparableMetricsReport {
    FastqComparableMetricsReport {
        schema_version: FASTQ_COMPARABLE_METRICS_SCHEMA_V1.to_string(),
        output_path: "artifacts/readiness/fastq.tsv".to_string(),
        comparable_stage_count: 2,
        multi_tool_stage_count: 1,
        comparable_tool_row_count: 3,
        row_count: 1,
        declared_stage_count: 1,
        missing_shared_metric_stage_count: 0,
        shared_metric_field_count: 1,
        rows: vec![FastqComparableMetricsRow {
            stage_id: "fastq.validate_reads".to_string(),
            comparison_contract_status: ComparableMetricContractStatus::Declared,
            tool_count: 2,
            tool_ids: vec!["fastqvalidator".to_string(), "seqtk".to_string()],
            default_tool_id: "fastqvalidator".to_string(),
            corpus_status: "compatible".to_string(),
            shared_metric_field_count: 1,
            shared_metric_fields: vec!["valid_read_fraction".to_string()],
            reason: "governed shared metric".to_string(),
        }],
    }
}

fn bam_report() -> BamComparableMetricsReport {
    BamComparableMetricsReport {
        schema_version: BAM_COMPARABLE_METRICS_SCHEMA_V1.to_string(),
        output_path: "artifacts/readiness/bam.tsv".to_string(),
        comparable_stage_count: 1,
        multi_tool_stage_count: 1,
        comparable_tool_row_count: 2,
        row_count: 1,
        declared_stage_count: 0,
        missing_shared_metric_stage_count: 1,
        shared_metric_field_count: 0,
        rows: vec![BamComparableMetricsRow {
            stage_id: "bam.damage".to_string(),
            comparison_contract_status: ComparableMetricContractStatus::MissingSharedMetrics,
            tool_count: 2,
            tool_ids: vec!["mapdamage".to_string(), "ngsbriggs".to_string()],
            default_tool_id: "mapdamage".to_string(),
            corpus_status: "compatible".to_string(),
            shared_metric_field_count: 0,
            shared_metric_fields: Vec::new(),
            reason: "shared metric contract is missing".to_string(),
        }],
    }
}

fn vcf_report() -> VcfComparableMetricsReport {
    VcfComparableMetricsReport {
        schema_version: VCF_COMPARABLE_METRICS_SCHEMA_V1.to_string(),
        output_path: "artifacts/readiness/vcf.tsv".to_string(),
        stage_count: 1,
        multi_tool_stage_count: 1,
        retained_tool_row_count: 2,
        row_count: 1,
        rows: vec![VcfComparableMetricsRow {
            stage_id: "vcf.impute".to_string(),
            metric_id: "concordance".to_string(),
            metric_name: "genotype concordance".to_string(),
            unit: "fraction".to_string(),
            direction: ComparableMetricDirection::HigherIsBetter,
            required: true,
            tools_covered: vec!["beagle".to_string(), "impute5".to_string()],
        }],
    }
}

#[test]
fn comparable_metric_reports_round_trip_and_validate() -> anyhow::Result<()> {
    let fastq = fastq_report();
    validate_fastq_comparable_metrics_report(&fastq)?;
    assert_eq!(
        serde_json::from_slice::<FastqComparableMetricsReport>(&serde_json::to_vec(&fastq)?)?,
        fastq
    );

    let bam = bam_report();
    validate_bam_comparable_metrics_report(&bam)?;
    assert_eq!(
        serde_json::from_slice::<BamComparableMetricsReport>(&serde_json::to_vec(&bam)?)?,
        bam
    );

    let vcf = vcf_report();
    validate_vcf_comparable_metrics_report(&vcf)?;
    assert_eq!(
        serde_json::from_slice::<VcfComparableMetricsReport>(&serde_json::to_vec(&vcf)?)?,
        vcf
    );
    Ok(())
}

#[test]
fn comparable_metric_reports_reject_aggregate_and_roster_drift() {
    let mut fastq = fastq_report();
    fastq.rows[0].shared_metric_field_count = 0;
    assert!(validate_fastq_comparable_metrics_report(&fastq).is_err());

    let mut fastq = fastq_report();
    fastq.comparable_tool_row_count += 1;
    assert!(validate_fastq_comparable_metrics_report(&fastq).is_err());

    let mut bam = bam_report();
    bam.rows[0].default_tool_id = "ngsbriggs".to_string();
    bam.rows[0].tool_ids = vec!["mapdamage".to_string(), "mapdamage".to_string()];
    assert!(validate_bam_comparable_metrics_report(&bam).is_err());

    let mut vcf = vcf_report();
    vcf.retained_tool_row_count += 1;
    assert!(validate_vcf_comparable_metrics_report(&vcf).is_err());
}

#[test]
fn comparable_metric_schemas_match_model_authority() -> anyhow::Result<()> {
    let repository_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| anyhow::anyhow!("benchmark model crate is outside the workspace"))?;
    for (file_name, generated) in [
        (
            "fastq-comparable-metrics.schema.json",
            serde_json::to_value(fastq_comparable_metrics_schema())?,
        ),
        (
            "bam-comparable-metrics.schema.json",
            serde_json::to_value(bam_comparable_metrics_schema())?,
        ),
        (
            "vcf-comparable-metrics.schema.json",
            serde_json::to_value(vcf_comparable_metrics_schema())?,
        ),
    ] {
        let path = repository_root.join("benchmarks/contracts/schemas").join(file_name);
        if std::env::var_os("BIJUX_UPDATE_GENERATED_SCHEMAS").is_some() {
            let mut bytes = serde_json::to_vec_pretty(&generated)?;
            bytes.push(b'\n');
            std::fs::write(&path, bytes)?;
        }
        let checked: serde_json::Value = serde_json::from_slice(&std::fs::read(path)?)?;
        assert_eq!(checked, generated, "{file_name} drifted from model authority");
    }
    Ok(())
}

#[test]
fn comparable_metric_reports_reject_unknown_fields() -> anyhow::Result<()> {
    let mut value = serde_json::to_value(fastq_report())?;
    value
        .as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("FASTQ report is not an object"))?
        .insert("scheduler_state".to_string(), serde_json::json!("running"));
    assert!(serde_json::from_value::<FastqComparableMetricsReport>(value).is_err());

    let mut value = serde_json::to_value(vcf_report())?;
    value["rows"][0]["direction"] = serde_json::json!("sideways");
    assert!(serde_json::from_value::<VcfComparableMetricsReport>(value).is_err());
    Ok(())
}
