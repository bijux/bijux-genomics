use std::collections::BTreeMap;

use bijux_dna_bench_model::{
    all_domain_output_declarations_schema, validate_all_domain_output_declaration_rows,
    validate_all_domain_output_declarations_report, AllDomainOutputDeclarationRow,
    AllDomainOutputDeclarationStatus, AllDomainOutputDeclarationsReport,
    ALL_DOMAIN_OUTPUT_DECLARATIONS_SCHEMA_V1,
};

#[test]
fn all_domain_output_declarations_schema_matches_checked_in_contract() -> anyhow::Result<()> {
    let generated = serde_json::to_value(all_domain_output_declarations_schema())?;
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| anyhow::anyhow!("crate outside workspace"))?;
    let path = root.join("benchmarks/contracts/schemas/all-domain-output-declarations.schema.json");
    if std::env::var_os("BIJUX_UPDATE_GENERATED_SCHEMAS").is_some() {
        let mut bytes = serde_json::to_vec_pretty(&generated)?;
        bytes.push(b'\n');
        std::fs::write(&path, bytes)?;
    }
    assert_eq!(serde_json::from_slice::<serde_json::Value>(&std::fs::read(path)?)?, generated);
    Ok(())
}

#[test]
fn all_domain_output_declaration_contracts_admit_complete_proof_rows() -> anyhow::Result<()> {
    validate_all_domain_output_declaration_rows(&valid_rows())?;
    validate_all_domain_output_declarations_report(&valid_report())?;
    Ok(())
}

#[test]
fn all_domain_output_declaration_contracts_reject_identity_path_and_aggregate_drift() {
    let mut rows = valid_rows();
    rows[0].result_id = rows[1].result_id.clone();
    assert!(validate_all_domain_output_declaration_rows(&rows).is_err());

    let mut rows = valid_rows();
    rows[2].manifest = "../outside/stage-result.json".to_string();
    assert!(validate_all_domain_output_declaration_rows(&rows).is_err());

    let mut rows = valid_rows();
    rows[1].result_id.push_str(":extra");
    assert!(validate_all_domain_output_declaration_rows(&rows).is_err());

    let mut report = valid_report();
    report.output_path = "/tmp/out.tsv".to_string();
    assert!(validate_all_domain_output_declarations_report(&report).is_err());

    let mut report = valid_report();
    report.complete_row_count = 2;
    assert!(validate_all_domain_output_declarations_report(&report).is_err());
}

fn valid_rows() -> Vec<AllDomainOutputDeclarationRow> {
    vec![
        row(
            "bam:corpus-01-kinship-mini:bam.kinship:sample-set:king",
            "bam",
            "bam.kinship",
            "king",
            "corpus-01-kinship-mini",
            "sample-set",
            &["summary"],
            &["kinship_report"],
            "runs/bench/slurm-dry-run/runs/local-benchmark-dry-run/corpus-01-kinship-mini/bam.kinship/sample-set/king/stage-result.json",
            &[],
        ),
        row(
            "fastq:corpus-02-edna-mini:fastq.screen_taxonomy:sample-set:kraken2",
            "fastq",
            "fastq.screen_taxonomy",
            "kraken2",
            "corpus-02-edna-mini",
            "sample-set",
            &["screen_report_tsv"],
            &["classification_report_json"],
            "runs/bench/slurm-dry-run/runs/local-benchmark-dry-run/corpus-02-edna-mini/fastq.screen_taxonomy/sample-set/kraken2/stage-result.json",
            &[],
        ),
        row(
            "vcf:vcf_production_regression:vcf.call:bam_bundle:bcftools",
            "vcf",
            "vcf.call",
            "bcftools",
            "vcf_production_regression",
            "corpus_only",
            &["called_vcf"],
            &["called_vcf"],
            "runs/bench/slurm-dry-run/runs/local-benchmark-dry-run/vcf_production_regression/vcf.call/bam_bundle/bcftools/stage-result.json",
            &["called_vcf_tbi"],
        ),
    ]
}

#[allow(clippy::too_many_arguments)]
fn row(
    result_id: &str,
    domain: &str,
    stage_id: &str,
    tool_id: &str,
    corpus_id: &str,
    asset_profile_id: &str,
    raw_outputs: &[&str],
    normalized_metrics: &[&str],
    manifest: &str,
    index_outputs: &[&str],
) -> AllDomainOutputDeclarationRow {
    let root = manifest.trim_end_matches("/stage-result.json");
    AllDomainOutputDeclarationRow {
        result_id: result_id.to_string(),
        domain: domain.to_string(),
        stage_id: stage_id.to_string(),
        tool_id: tool_id.to_string(),
        corpus_id: corpus_id.to_string(),
        asset_profile_id: asset_profile_id.to_string(),
        raw_outputs: raw_outputs.iter().map(|value| (*value).to_string()).collect(),
        normalized_metrics: normalized_metrics.iter().map(|value| (*value).to_string()).collect(),
        logs: vec![format!("stdout={root}/stdout.log"), format!("stderr={root}/stderr.log")],
        manifest: manifest.to_string(),
        index_outputs: index_outputs.iter().map(|value| (*value).to_string()).collect(),
        status: AllDomainOutputDeclarationStatus::Complete,
    }
}

fn valid_report() -> AllDomainOutputDeclarationsReport {
    AllDomainOutputDeclarationsReport {
        schema_version: ALL_DOMAIN_OUTPUT_DECLARATIONS_SCHEMA_V1.to_string(),
        output_path: "artifacts/benchmark-smoke/readiness/output-declarations-all-domains.tsv"
            .to_string(),
        row_count: 3,
        result_id_count: 3,
        complete_row_count: 3,
        incomplete_row_count: 0,
        domain_counts: BTreeMap::from([
            ("bam".to_string(), 1),
            ("fastq".to_string(), 1),
            ("vcf".to_string(), 1),
        ]),
        status_counts: BTreeMap::from([("complete".to_string(), 3)]),
        rows: valid_rows(),
    }
}
