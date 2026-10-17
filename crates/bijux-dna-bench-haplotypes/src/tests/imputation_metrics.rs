#![cfg(test)]
//! Original scientific and binding behavior at its owning capability.
use crate::imputation_metrics::observations::variants::summarize_imputation_metrics;
#[test]
fn imputation_metrics_summary_reports_missing_quality_fields_explicitly() {
    let qc = serde_json::json!({
        "imputation_info_mean": 0.81,
        "low_confidence_count": 3,
        "concordance": {
            "genotype_concordance": serde_json::Value::Null,
            "dosage_r2": serde_json::Value::Null
        }
    });
    let summary = summarize_imputation_metrics(&qc).expect("summarize quality payload");
    assert_eq!(summary.concordance, None);
    assert!(!summary.r2_available);
    assert_eq!(summary.masked_truth_sites, 0);
    assert_eq!(
        summary.missing_quality_fields,
        vec![
            "concordance".to_string(),
            "dosage_r2".to_string(),
            "maf_strata".to_string(),
            "masked_truth_sites".to_string()
        ]
    );
    assert_eq!(summary.status, "explicit_missing_quality_fields");
}

#[test]
fn imputation_metrics_admits_own_declaration_before_caller_root_effects() -> anyhow::Result<()> {
    use crate::inputs::HaplotypeBenchmarkInputs;
    use crate::panel_contracts::resolve_governed_vcf_panel_workflow_smoke_contract;
    use anyhow::Context as _;
    let admitted = crate::test_support::inputs();
    let rows = admitted.matrix_rows().cloned().collect::<Vec<_>>();
    let bindings = bijux_dna_stage_contract::canonical_tool_bindings()?;
    let complete = HaplotypeBenchmarkInputs::new(rows.clone(), bindings)?;
    let contract = resolve_governed_vcf_panel_workflow_smoke_contract(
        &complete,
        "vcf.imputation_metrics",
        "beagle",
        "imputation_metrics_json",
    )?;
    assert_eq!(contract.stage, "vcf.imputation_metrics");
    assert_eq!(contract.tool, "beagle");
    let workspace = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .context("haplotype crate must be in the workspace")?;
    let output_owner = workspace.join("artifacts/tests/imputation-metrics-admission");
    bijux_dna_infra::ensure_dir(&output_owner)?;
    let fixture = bijux_dna_infra::temp_dir_in(&output_owner, "declared-inputs-")?;
    for directory_root in [false, true] {
        let caller_root = fixture.path().join(if directory_root {
            "retained-directory"
        } else {
            "retained-file"
        });
        let retained_path = if directory_root {
            std::fs::create_dir(&caller_root)?;
            caller_root.join("retained-evidence.txt")
        } else {
            caller_root.clone()
        };
        std::fs::write(&retained_path, b"retained caller root")?;
        for (kind, diagnostic) in [
            ("missing", "haplotype stage template is absent"),
            ("output", "expected outputs drifted for `vcf.imputation_metrics`"),
        ] {
            let mut invalid = rows.clone();
            if kind == "missing" {
                invalid.retain(|row| row.stage_id != "vcf.imputation_metrics");
            } else {
                invalid
                    .iter_mut()
                    .find(|row| row.stage_id == "vcf.imputation_metrics")
                    .context("metrics declaration")?
                    .expected_outputs = vec!["foreign_report".into()];
            }
            let scoped = HaplotypeBenchmarkInputs::new(
                invalid,
                bijux_dna_stage_contract::canonical_tool_bindings()?,
            )?;
            let error = crate::imputation_metrics::run_local_vcf_imputation_metrics_smoke(
                &scoped,
                &caller_root,
                "beagle",
            )
            .err()
            .context("invalid own metrics declaration was admitted")?;
            assert!(format!("{error:#}").contains(diagnostic), "{error:#}");
            assert_eq!(std::fs::read(&retained_path)?, b"retained caller root");
            assert!(!caller_root.join("artifacts").exists());
        }
        if directory_root {
            assert_eq!(std::fs::read_dir(&caller_root)?.count(), 1);
        }
    }
    Ok(())
}
