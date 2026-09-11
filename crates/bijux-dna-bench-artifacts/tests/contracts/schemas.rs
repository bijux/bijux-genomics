use anyhow::Result;
use bijux_dna_bench_artifacts::schemas::{paths, publication, validation};
use std::path::{Path, PathBuf};

fn publish_portfolio(root: &Path) -> Result<PathBuf> {
    let schemas = root.join("schemas");
    publication::write_fastq_normalized_metrics_schema(&paths::fastq_schema_path(&schemas))?;
    publication::write_bam_normalized_metrics_schema(&paths::bam_schema_path(&schemas))?;
    publication::write_vcf_normalized_metrics_schema(&paths::vcf_schema_path(&schemas))?;
    publication::write_vcf_normalized_metrics_stage_schemas(&paths::vcf_stage_schema_dir(
        &schemas,
    ))?;
    Ok(schemas)
}

fn read_report(path: &Path) -> Result<serde_json::Value> {
    Ok(serde_json::from_slice(&std::fs::read(path)?)?)
}

#[test]
fn schema_portfolio_validates_all_domains() -> Result<()> {
    let root = tempfile::tempdir()?;
    let schemas = publish_portfolio(root.path())?;
    let output = root.path().join("validation.json");
    let report = validation::validate_all_domain_schemas(
        root.path(),
        &output,
        &schemas,
        &[
            validation::SchemaDomain::Fastq,
            validation::SchemaDomain::Bam,
            validation::SchemaDomain::Vcf,
        ],
    )?;
    assert!(report.ok);
    assert_eq!(report.passed_domain_count, 3);
    assert_eq!(report.failed_domain_count, 0);
    assert_eq!(
        report.domains.iter().map(|row| row.domain.as_str()).collect::<Vec<_>>(),
        ["bam", "fastq", "vcf"]
    );
    assert_eq!(read_report(&output)?["ok"], true);
    Ok(())
}

#[test]
fn shared_schema_disagreement_retains_failed_diagnostic() -> Result<()> {
    let root = tempfile::tempdir()?;
    let schemas = publish_portfolio(root.path())?;
    for (domain, shared) in [
        (validation::SchemaDomain::Fastq, paths::fastq_schema_path(&schemas)),
        (validation::SchemaDomain::Bam, paths::bam_schema_path(&schemas)),
    ] {
        let mut schema = read_report(&shared)?;
        schema["$id"] = serde_json::json!("foreign.schema");
        std::fs::write(&shared, serde_json::to_vec(&schema)?)?;
        let output = root.path().join("failed.json");
        let failed = match domain {
            validation::SchemaDomain::Fastq => {
                validation::validate_fastq_schemas(root.path(), &output, &shared).is_err()
            }
            _ => validation::validate_bam_schemas(root.path(), &output, &shared).is_err(),
        };
        assert!(failed);
        let report = read_report(&output)?;
        assert_eq!(report["passes_gate"], false);
        assert_eq!(report["shared_schema_matches"], false);
    }
    Ok(())
}

#[test]
fn vcf_schema_portfolio_refuses_missing_extra_and_changed_files() -> Result<()> {
    let root = tempfile::tempdir()?;
    let schemas = publish_portfolio(root.path())?;
    let stages = paths::vcf_stage_schema_dir(&schemas);
    let descriptors = publication::vcf_normalized_metrics_stage_schema_descriptors()?;
    let stage = stages.join(&descriptors[0].file_name);
    let original = std::fs::read(&stage)?;
    let output = root.path().join("vcf.json");
    std::fs::remove_file(&stage)?;
    assert!(validation::validate_vcf_schemas(
        root.path(),
        &output,
        &paths::vcf_schema_path(&schemas),
        &stages
    )
    .is_err());
    assert_eq!(
        read_report(&output)?["missing_stage_schema_files"],
        serde_json::json!([descriptors[0].file_name])
    );
    std::fs::write(&stage, &original)?;
    let extra = stages.join("unregistered.json");
    std::fs::write(&extra, b"{}")?;
    assert!(validation::validate_vcf_schemas(
        root.path(),
        &output,
        &paths::vcf_schema_path(&schemas),
        &stages
    )
    .is_err());
    assert_eq!(
        read_report(&output)?["unexpected_stage_schema_files"],
        serde_json::json!(["unregistered.json"])
    );
    std::fs::remove_file(extra)?;
    std::fs::write(&stage, b"{}")?;
    assert!(validation::validate_vcf_schemas(
        root.path(),
        &output,
        &paths::vcf_schema_path(&schemas),
        &stages
    )
    .is_err());
    assert_eq!(read_report(&output)?["exact_stage_schema_file_count"], descriptors.len() - 1);
    Ok(())
}

#[test]
fn invalid_domain_selection_refuses_before_effects() -> Result<()> {
    let root = tempfile::tempdir()?;
    for domains in [Vec::new(), vec![validation::SchemaDomain::Vcf, validation::SchemaDomain::Vcf]]
    {
        assert!(validation::validate_all_domain_schemas(
            root.path(),
            &root.path().join("report.json"),
            &root.path().join("absent"),
            &domains
        )
        .is_err());
        assert_eq!(std::fs::read_dir(root.path())?.count(), 0);
    }
    Ok(())
}

#[test]
fn multidomain_failure_retains_all_domain_observations() -> Result<()> {
    let root = tempfile::tempdir()?;
    let schemas = publish_portfolio(root.path())?;
    std::fs::write(paths::bam_schema_path(&schemas), b"{}")?;
    let output = root.path().join("all.json");
    assert!(validation::validate_all_domain_schemas(
        root.path(),
        &output,
        &schemas,
        &[
            validation::SchemaDomain::Fastq,
            validation::SchemaDomain::Bam,
            validation::SchemaDomain::Vcf
        ]
    )
    .is_err());
    let report = read_report(&output)?;
    assert_eq!(report["domain_count"], 3);
    assert_eq!(report["passed_domain_count"], 2);
    assert_eq!(report["failed_domain_count"], 1);
    assert_eq!(report["ok"], false);
    Ok(())
}

#[test]
fn malformed_schema_preserves_previous_validation_report() -> Result<()> {
    let root = tempfile::tempdir()?;
    let shared = root.path().join("schema.json");
    let output = root.path().join("report.json");
    std::fs::write(&shared, b"{")?;
    std::fs::write(&output, b"retained evidence")?;
    assert!(validation::validate_fastq_schemas(root.path(), &output, &shared).is_err());
    assert_eq!(std::fs::read(&output)?, b"retained evidence");
    Ok(())
}
