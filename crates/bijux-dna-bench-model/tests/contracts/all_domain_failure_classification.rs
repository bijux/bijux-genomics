use std::collections::BTreeMap;

use bijux_dna_bench_model::{
    all_domain_failure_classification_schema, validate_all_domain_failure_classification,
    AllDomainFailureClassificationReport, AllDomainFailureClassificationRow,
    ALL_DOMAIN_FAILURE_CLASSIFICATION_SCHEMA_V1, REQUIRED_ALL_DOMAIN_FAILURE_CLASSES,
};

fn report() -> AllDomainFailureClassificationReport {
    let rows = REQUIRED_ALL_DOMAIN_FAILURE_CLASSES
        .into_iter()
        .map(|class_id| AllDomainFailureClassificationRow {
            class_id: class_id.to_string(),
            domain: "vcf".to_string(),
            stage_id: "vcf.filter".to_string(),
            tool_id: "bcftools".to_string(),
            result_id: None,
            source_surface: "governed_probe".to_string(),
            evidence_path: format!("artifacts/probes/{class_id}.json"),
            observed_status: class_id.to_string(),
            observed_error: String::new(),
            detail: "governed failure classification".to_string(),
            triggered: true,
        })
        .collect::<Vec<_>>();
    AllDomainFailureClassificationReport {
        schema_version: ALL_DOMAIN_FAILURE_CLASSIFICATION_SCHEMA_V1.to_string(),
        output_path: "artifacts/failure-classification.json".to_string(),
        fixture_root: "artifacts/probes".to_string(),
        row_count: rows.len(),
        triggered_row_count: rows.len(),
        required_class_count: rows.len(),
        triggered_class_count: rows.len(),
        missing_class_count: 0,
        class_counts: rows.iter().map(|row| (row.class_id.clone(), 1)).collect::<BTreeMap<_, _>>(),
        missing_class_ids: vec![],
        passes_behavior_test: false,
        rows,
    }
}

#[test]
fn failure_classification_round_trips_and_matches_schema() -> anyhow::Result<()> {
    let mut report = report();
    validate_all_domain_failure_classification(&mut report)?;
    assert!(report.passes_behavior_test);
    assert_eq!(
        serde_json::from_slice::<AllDomainFailureClassificationReport>(&serde_json::to_vec(
            &report
        )?)?,
        report
    );
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| anyhow::anyhow!("crate outside workspace"))?;
    let path =
        root.join("benchmarks/contracts/schemas/all-domain-failure-classification.schema.json");
    let generated = serde_json::to_value(all_domain_failure_classification_schema())?;
    if std::env::var_os("BIJUX_UPDATE_GENERATED_SCHEMAS").is_some() {
        let mut bytes = serde_json::to_vec_pretty(&generated)?;
        bytes.push(b'\n');
        std::fs::write(&path, bytes)?;
    }
    assert_eq!(serde_json::from_slice::<serde_json::Value>(&std::fs::read(path)?)?, generated);
    Ok(())
}

#[test]
fn failure_classification_rejects_class_counter_and_trigger_drift() {
    let mut duplicate = report();
    duplicate.rows[0].class_id = duplicate.rows[1].class_id.clone();
    assert!(validate_all_domain_failure_classification(&mut duplicate).is_err());
    let mut count = report();
    count.class_counts.insert("missing_input".to_string(), 2);
    assert!(validate_all_domain_failure_classification(&mut count).is_err());
    let mut untriggered = report();
    untriggered.rows[0].triggered = false;
    assert!(validate_all_domain_failure_classification(&mut untriggered).is_err());

    let mut unsafe_path = report();
    unsafe_path.rows[0].evidence_path = "../outside.json".to_string();
    assert!(validate_all_domain_failure_classification(&mut unsafe_path).is_err());

    let mut wrong_domain = report();
    wrong_domain.rows[0].domain = "bam".to_string();
    assert!(validate_all_domain_failure_classification(&mut wrong_domain).is_err());
}

#[test]
fn collector_projection_admits_classification_without_mutating_source_evidence(
) -> anyhow::Result<()> {
    use bijux_dna_bench_model::{
        project_full_benchmark_observation_rows as project, FullBenchmarkResultStatus,
        FullBenchmarkSourceObservation as Observation,
    };
    let mut source = report();
    validate_all_domain_failure_classification(&mut source)?;
    let before = serde_json::to_value(&source)?;
    let rows = project(Observation::FailureClassification(&source))?;
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].result_status, FullBenchmarkResultStatus::InsufficientData);
    assert_eq!(rows[1].result_status, FullBenchmarkResultStatus::UnsupportedPair);
    assert!(rows.iter().all(|row| row.execution_id.is_none()));
    assert_eq!(serde_json::to_value(&source)?, before);
    source.passes_behavior_test = false;
    assert!(project(Observation::FailureClassification(&source)).is_err());
    source.passes_behavior_test = true;
    source.rows[0].triggered = false;
    assert!(project(Observation::FailureClassification(&source)).is_err());
    Ok(())
}
