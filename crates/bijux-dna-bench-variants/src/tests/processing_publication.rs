#![cfg(test)]
use crate::{inputs::VariantBenchmarkInputs, qc::run_local_vcf_qc_smoke};

#[test]
fn qc_report_formats_disclose_internal_execution_and_preserve_unknown_evidence(
) -> anyhow::Result<()> {
    let repository = tempfile::tempdir()?;
    for tool in ["bcftools", "plink", "plink2"] {
        let report =
            run_local_vcf_qc_smoke(&crate::test_support::inputs(), repository.path(), tool)?;
        let root = repository.path().join(&report.output_root);
        let original: serde_json::Value =
            serde_json::from_slice(&std::fs::read(root.join("execution-provenance.json"))?)?;
        assert_eq!(original["executor"], "bijux-dna-stages-vcf::pipeline::run_qc_stage");
        assert_eq!(original["requested_report_format"], tool);
        assert_eq!(original["external_tool_executed"], false);
        let inventory: serde_json::Value =
            serde_json::from_slice(&std::fs::read(root.join("variant-artifact-ownership.json"))?)?;
        assert!(inventory["files"]
            .as_array()
            .expect("owned files")
            .iter()
            .any(|path| path == "execution-provenance.json"));
        std::fs::write(root.join("user-evidence.txt"), b"keep")?;
        let repeated =
            run_local_vcf_qc_smoke(&crate::test_support::inputs(), repository.path(), tool)?;
        assert_eq!(repeated.output_root, report.output_root);
        assert_eq!(std::fs::read(root.join("user-evidence.txt"))?, b"keep");
        assert_eq!(std::fs::read_dir(root.join("executions"))?.count(), 2);
    }
    Ok(())
}

#[test]
fn unknown_qc_report_format_refuses_before_output_creation() -> anyhow::Result<()> {
    let repository = tempfile::tempdir()?;
    let (governed, mut qc) = crate::test_support::rows();
    qc[0].tool_id = "unsupported_qc".to_owned();
    let inputs = VariantBenchmarkInputs::new(governed, qc)?;
    assert!(run_local_vcf_qc_smoke(&inputs, repository.path(), "unsupported_qc").is_err());
    assert_eq!(std::fs::read_dir(repository.path())?.count(), 0);
    Ok(())
}

#[test]
fn processing_declarations_reject_foreign_duplicate_and_escaping_inputs() {
    let (governed, qc) = crate::test_support::rows();
    let mut duplicate = governed.clone();
    duplicate.push(governed[0].clone());
    assert!(VariantBenchmarkInputs::new(duplicate, qc.clone()).is_err());
    let mut foreign = governed.clone();
    foreign[0].stage_id = "vcf.phasing".to_owned();
    assert!(VariantBenchmarkInputs::new(foreign, qc.clone()).is_err());
    let mut escaping = qc.clone();
    escaping[0].tool_id = "../plink".to_owned();
    assert!(VariantBenchmarkInputs::new(governed.clone(), escaping).is_err());
    let mut duplicate_output = qc.clone();
    let repeated = duplicate_output[0].expected_outputs[0].clone();
    duplicate_output[0].expected_outputs.push(repeated);
    assert!(VariantBenchmarkInputs::new(governed, duplicate_output).is_err());
}

#[test]
fn postprocess_retains_native_paths_and_exact_checksum_representations_across_publications(
) -> anyhow::Result<()> {
    use bijux_dna_infra::{hash_bytes_sha256, hash_file_sha256};
    let repository = tempfile::tempdir()?;
    let report = crate::postprocess::run_local_vcf_postprocess_smoke(
        &crate::test_support::inputs(),
        repository.path(),
        "bcftools",
    )?;
    let root = repository.path().join(&report.output_root);
    let manifest_bytes = std::fs::read(repository.path().join(&report.final_manifest_path))?;
    let manifest: serde_json::Value = serde_json::from_slice(&manifest_bytes)?;
    let checksums: serde_json::Value = serde_json::from_slice(&std::fs::read(
        repository.path().join(&report.artifact_checksums_path),
    )?)?;
    assert_eq!(checksums["final_manifest.json"], hash_bytes_sha256(&manifest_bytes));
    let mut native_files = Vec::new();
    for pointer in ["/outputs/vcf", "/outputs/tbi", "/validate_outputs", "/normalization_contract"]
    {
        let native = std::path::PathBuf::from(
            manifest
                .pointer(pointer)
                .and_then(serde_json::Value::as_str)
                .expect("native manifest path"),
        );
        assert!(native.starts_with(root.join("executions")), "{pointer}: {}", native.display());
        assert!(native.is_file());
        native_files.push((native.clone(), std::fs::read(&native)?));
        let name = native.file_name().and_then(|name| name.to_str()).expect("native artifact name");
        if pointer.starts_with("/outputs/") {
            assert_eq!(checksums[name], hash_file_sha256(&native)?);
        } else {
            let value: serde_json::Value = serde_json::from_slice(&std::fs::read(&native)?)?;
            assert_eq!(
                checksums[name],
                hash_bytes_sha256(&serde_json::to_vec(&value)?),
                "native JSON contract hashes its compact value"
            );
            if pointer == "/normalization_contract" {
                for key in ["raw_input", "normalized_output"] {
                    let path = std::path::Path::new(
                        value["view_retention"][key].as_str().expect("native view"),
                    );
                    assert!(path.starts_with(root.join("executions")));
                    assert!(path.is_file());
                }
            }
        }
    }
    let result: serde_json::Value = serde_json::from_slice(&std::fs::read(
        repository.path().join(&report.stage_result_manifest_path),
    )?)?;
    for output in result["outputs"].as_array().expect("declared outputs") {
        let path = repository.path().join(output["realized_path"].as_str().expect("output path"));
        assert!(path.starts_with(&root));
        assert!(
            !path.starts_with(root.join("executions")),
            "public index must name stable addresses"
        );
        assert!(path.is_file());
    }
    crate::postprocess::run_local_vcf_postprocess_smoke(
        &crate::test_support::inputs(),
        repository.path(),
        "bcftools",
    )?;
    for (path, bytes) in native_files {
        assert_eq!(std::fs::read(path)?, bytes);
    }
    Ok(())
}

#[test]
fn processing_publication_refuses_unowned_output_replacement() -> anyhow::Result<()> {
    let repository = tempfile::tempdir()?;
    let root = repository.path().join("artifacts/runs/bench/local-smoke/vcf.filter/bcftools");
    std::fs::create_dir_all(&root)?;
    let protected = root.join(crate::filter::model::DEFAULT_OUTPUT_VCF_NAME);
    std::fs::write(&protected, b"unowned evidence")?;
    assert!(crate::filter::run_local_vcf_filter_smoke(
        &crate::test_support::inputs(),
        repository.path(),
        "bcftools"
    )
    .is_err());
    assert_eq!(std::fs::read(&protected)?, b"unowned evidence");
    Ok(())
}

#[test]
fn processing_declarations_reject_foreign_adapter_and_parser_semantics() {
    let (governed, qc) = crate::test_support::rows();
    let mut adapter = governed.clone();
    adapter[0].adapter_id = "vcf.adapter.phasing".to_owned();
    assert!(VariantBenchmarkInputs::new(adapter, qc.clone()).is_err());
    let mut parser = governed;
    parser[0].parser_id = "vcf.parser.qc_report".to_owned();
    assert!(VariantBenchmarkInputs::new(parser, qc).is_err());
}

#[test]
fn qc_output_contract_drift_refuses_before_effects() -> anyhow::Result<()> {
    let repository = tempfile::tempdir()?;
    let (governed, mut qc) = crate::test_support::rows();
    for row in &mut qc {
        row.expected_outputs = vec!["unrelated_report".to_owned()];
    }
    let inputs = VariantBenchmarkInputs::new(governed, qc)?;
    assert!(run_local_vcf_qc_smoke(&inputs, repository.path(), "plink2").is_err());
    assert_eq!(std::fs::read_dir(repository.path())?.count(), 0);
    Ok(())
}
