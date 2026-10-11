//! Owner: bijux-dna-bench
//! Bounded loading for domain-neutral benchmark data source authorities.

use std::path::Path;

use anyhow::{Context, Result};
use bijux_dna_bench_model::{validate_benchmark_data_source_spec, BenchmarkDataSourceSpec};

const MAX_DATA_SOURCE_SPEC_BYTES: usize = 4 * 1024 * 1024;

/// Load and validate a governed production data source declaration.
///
/// # Errors
/// Returns an error when the declaration is unreadable, malformed, or violates source policy.
pub fn load_benchmark_data_source_spec(path: &Path) -> Result<BenchmarkDataSourceSpec> {
    let bytes = bijux_dna_infra::read_to_end_bounded(path, MAX_DATA_SOURCE_SPEC_BYTES)
        .with_context(|| format!("read benchmark data source specification {}", path.display()))?;
    let spec: BenchmarkDataSourceSpec = toml::from_str(
        std::str::from_utf8(&bytes).context("benchmark data source specification is not UTF-8")?,
    )
    .with_context(|| format!("parse benchmark data source specification {}", path.display()))?;
    validate_benchmark_data_source_spec(&spec)
        .context("validate benchmark data source specification")?;
    Ok(spec)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use anyhow::Result;
    use bijux_dna_bench_model::{
        BenchmarkCorpusClass, BenchmarkDataSourceCollection, CorpusEvaluationDesign,
        CorpusSourcePolicy, BENCHMARK_DATA_SOURCE_SCHEMA_V1,
    };

    use super::{load_benchmark_data_source_spec, BenchmarkDataSourceSpec};

    #[test]
    fn loads_only_valid_production_data_source_authorities() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let path = temp.path().join("variant-truth-panel.toml");
        let source = source_spec();
        fs::write(&path, toml::to_string_pretty(&source)?)?;
        assert_eq!(load_benchmark_data_source_spec(&path)?, source);

        let mut fixture = source_spec();
        fixture.policy.class = BenchmarkCorpusClass::TestFixture;
        fs::write(&path, toml::to_string_pretty(&fixture)?)?;
        assert!(load_benchmark_data_source_spec(&path).is_err());
        Ok(())
    }

    fn source_spec() -> BenchmarkDataSourceSpec {
        BenchmarkDataSourceSpec {
            schema_version: BENCHMARK_DATA_SOURCE_SCHEMA_V1.to_string(),
            portfolio_id: "variant-truth-panel".to_string(),
            policy: CorpusSourcePolicy {
                class: BenchmarkCorpusClass::ProductionReal,
                access_classification: "public".to_string(),
                redistribution_classification: "accession-referenced".to_string(),
                access_terms_url: "https://example.org/access".to_string(),
                attribution: "Cite the source collection.".to_string(),
                projected_storage_bytes: 1_000_000,
                projected_cpu_hours: 10,
                projection_basis: "Ten samples with bounded verification work.".to_string(),
                truth_roles: vec!["variant_truth".to_string()],
                truth_strategy: "independent_reference_material".to_string(),
                known_limitations: vec!["The source is not population representative.".to_string()],
            },
            evaluation: CorpusEvaluationDesign {
                sample_context: "Ten reference-material samples.".to_string(),
                assay_context: "Whole-genome sequencing.".to_string(),
                strata: vec!["pedigree_role".to_string()],
                controls: vec!["mendelian_consistency".to_string()],
                held_out_split: "Mask truth genotypes before evaluation.".to_string(),
                lineage: "Archive accession to source catalog to derived panel.".to_string(),
                allowed_uses: vec!["accuracy".to_string(), "performance".to_string()],
            },
            description: "Public reference material for variant benchmarking.".to_string(),
            data_types: vec!["bam".to_string(), "vcf".to_string()],
            target_sample_count: 10,
            collections: vec![BenchmarkDataSourceCollection {
                id: "reference-material".to_string(),
                authority: "Reference authority".to_string(),
                accession: "REFERENCE-001".to_string(),
                source_url: "https://example.org/record".to_string(),
                access_terms_url: "https://example.org/access".to_string(),
                data_types: vec!["bam".to_string(), "vcf".to_string()],
                planned_sample_count: Some(10),
                selected_accessions: Vec::new(),
                selection_scope: "All ten independently characterized samples.".to_string(),
                truth_basis: "Orthogonal measurements define benchmark genotypes.".to_string(),
                known_limitations: vec![
                    "Difficult regions remain outside the benchmark.".to_string()
                ],
            }],
        }
    }
}
