//! Owner: bijux-dna-bench
//! Read and admit domain-local stage matrix declarations from an explicit repository root.

mod domain;

pub use domain::BenchmarkLocalStageInventoryDomain;

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use anyhow::{anyhow, Context, Result};
use bijux_dna_bench_model::{
    validate_benchmark_local_stage_inventory, BenchmarkLocalStageInventory,
    BenchmarkLocalStageInventoryEntry, BenchmarkLocalStageMatrixConfig,
    BenchmarkLocalStageReadinessKind as LocalStageReadinessKind, LocalVcfStageMatrixConfig,
    BENCHMARK_LOCAL_STAGE_INVENTORY_SCHEMA_V1,
};

const VCF_LOCAL_STAGE_READINESS_KIND: LocalStageReadinessKind = LocalStageReadinessKind::Smoke;

/// Load one domain's stage inventory without creating outputs.
///
/// # Errors
/// Returns an error for missing/malformed matrices, incompatible schema versions, or invalid stages.
pub fn load_benchmark_local_stage_inventory(
    repo_root: &Path,
    domain: BenchmarkLocalStageInventoryDomain,
) -> Result<BenchmarkLocalStageInventory> {
    match domain {
        BenchmarkLocalStageInventoryDomain::Fastq | BenchmarkLocalStageInventoryDomain::Bam => {
            load_matrix_backed_stage_inventory(repo_root, domain)
        }
        BenchmarkLocalStageInventoryDomain::Vcf => load_vcf_stage_inventory(repo_root),
    }
}

fn load_matrix_backed_stage_inventory(
    repo_root: &Path,
    domain: BenchmarkLocalStageInventoryDomain,
) -> Result<BenchmarkLocalStageInventory> {
    let matrix_path = repo_root.join(domain.matrix_relative_path());
    let raw = fs::read_to_string(&matrix_path)
        .with_context(|| format!("read {}", matrix_path.display()))?;
    let matrix: BenchmarkLocalStageMatrixConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", matrix_path.display()))?;

    if matrix.schema_version != domain.expected_matrix_schema_version() {
        return Err(anyhow!(
            "{} declares `{}` but `{}` is required for `{}` inventory",
            matrix_path.display(),
            matrix.schema_version,
            domain.expected_matrix_schema_version(),
            domain.as_str()
        ));
    }

    let stages = validate_matrix_backed_stage_entries(
        &matrix_path,
        domain,
        matrix.stages.into_iter().map(|entry| (entry.stage_id, entry.readiness_kind)),
    )?;

    let inventory = BenchmarkLocalStageInventory {
        schema_version: BENCHMARK_LOCAL_STAGE_INVENTORY_SCHEMA_V1.to_string(),
        domain: domain.as_str().to_string(),
        stage_matrix_schema_version: matrix.schema_version,
        stage_matrix_path: domain.matrix_relative_path().to_string(),
        stage_count: stages.len(),
        stages,
    };
    validate_benchmark_local_stage_inventory(&inventory)?;
    Ok(inventory)
}

fn load_vcf_stage_inventory(repo_root: &Path) -> Result<BenchmarkLocalStageInventory> {
    let domain = BenchmarkLocalStageInventoryDomain::Vcf;
    let matrix_path = repo_root.join(domain.matrix_relative_path());
    let raw = fs::read_to_string(&matrix_path)
        .with_context(|| format!("read {}", matrix_path.display()))?;
    let matrix: LocalVcfStageMatrixConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", matrix_path.display()))?;

    if matrix.schema_version != domain.expected_matrix_schema_version() {
        return Err(anyhow!(
            "{} declares `{}` but `{}` is required for `{}` inventory",
            matrix_path.display(),
            matrix.schema_version,
            domain.expected_matrix_schema_version(),
            domain.as_str()
        ));
    }

    let stages = validate_matrix_backed_stage_entries(
        &matrix_path,
        domain,
        matrix.rows.into_iter().map(|row| (row.stage_id, VCF_LOCAL_STAGE_READINESS_KIND)),
    )?;

    let inventory = BenchmarkLocalStageInventory {
        schema_version: BENCHMARK_LOCAL_STAGE_INVENTORY_SCHEMA_V1.to_string(),
        domain: domain.as_str().to_string(),
        stage_matrix_schema_version: matrix.schema_version,
        stage_matrix_path: domain.matrix_relative_path().to_string(),
        stage_count: stages.len(),
        stages,
    };
    validate_benchmark_local_stage_inventory(&inventory)?;
    Ok(inventory)
}

fn validate_matrix_backed_stage_entries(
    matrix_path: &Path,
    domain: BenchmarkLocalStageInventoryDomain,
    entries: impl Iterator<Item = (String, LocalStageReadinessKind)>,
) -> Result<Vec<BenchmarkLocalStageInventoryEntry>> {
    let mut seen_stage_ids = BTreeSet::new();
    let mut stages = Vec::new();
    for (stage_id, readiness_kind) in entries {
        if !stage_id.starts_with(domain.stage_prefix()) {
            return Err(anyhow!(
                "{} contains out-of-domain stage `{}` for `{}` inventory",
                matrix_path.display(),
                stage_id,
                domain.as_str()
            ));
        }
        if !seen_stage_ids.insert(stage_id.clone()) {
            return Err(anyhow!(
                "{} repeats stage `{}` in `{}` inventory",
                matrix_path.display(),
                stage_id,
                domain.as_str()
            ));
        }
        stages.push(BenchmarkLocalStageInventoryEntry { stage_id, readiness_kind });
    }
    Ok(stages)
}

#[cfg(test)]
mod tests {
    //! Owner: bijux-dna-bench
    use std::fs;
    use std::path::{Path, PathBuf};

    use anyhow::{Context, Result};

    use super::{
        load_benchmark_local_stage_inventory, BenchmarkLocalStageInventoryDomain as Domain,
    };
    use crate::workflow::render_benchmark_all_domain_stage_inventory;

    fn write_matrix(root: &Path, domain: &str, text: &str) -> Result<()> {
        let directory = root.join("configs/bench/local");
        fs::create_dir_all(&directory)?;
        fs::write(directory.join(format!("{domain}-stage-matrix.toml")), text)?;
        Ok(())
    }

    fn matrix(domain: &str) -> String {
        format!(
            "schema_version = \"bijux.bench.{domain}.local_stage_matrix.v1\"\n\
             [[stages]]\nstage_id = \"{domain}.second\"\nreadiness_kind = \"smoke\"\n\
             [[stages]]\nstage_id = \"{domain}.first\"\nreadiness_kind = \"dry_run\"\n"
        )
    }

    #[test]
    fn matrix_admission_uses_the_explicit_root_and_preserves_declaration_order() -> Result<()> {
        let root = tempfile::tempdir()?;
        for domain in [Domain::Fastq, Domain::Bam] {
            let input = matrix(domain.as_str());
            write_matrix(root.path(), domain.as_str(), &input)?;
            let report = load_benchmark_local_stage_inventory(root.path(), domain)?;
            assert_eq!(report.domain, domain.as_str());
            assert_eq!(report.stage_count, 2);
            assert_eq!(report.stages[0].stage_id, format!("{}.second", domain.as_str()));
            assert_eq!(report.stages[1].stage_id, format!("{}.first", domain.as_str()));
            assert_eq!(fs::read_to_string(root.path().join(domain.matrix_relative_path()))?, input);
        }
        Ok(())
    }

    #[test]
    fn matrix_admission_rejects_schema_shape_domain_and_duplicate_violations() -> Result<()> {
        let root = tempfile::tempdir()?;
        let valid = matrix("fastq");
        let cases = [
            (valid.replace("local_stage_matrix.v1", "local_stage_matrix.unknown"), "declares"),
            (valid.replace("fastq.first", "bam.first"), "out-of-domain"),
            (valid.replace("fastq.first", "fastq.second"), "repeats stage"),
            (valid.replace("[[stages]]", "unexpected = true\n[[stages]]"), "unknown field"),
            (valid.replace("dry_run", "unsupported_readiness"), "unknown variant"),
            (
                "schema_version = \"bijux.bench.fastq.local_stage_matrix.v1\"\nstages = []\n"
                    .to_string(),
                "non-empty",
            ),
        ];
        for (input, expected) in cases {
            write_matrix(root.path(), "fastq", &input)?;
            let error = load_benchmark_local_stage_inventory(root.path(), Domain::Fastq)
                .err()
                .context("invalid matrix was admitted")?;
            assert!(format!("{error:#}").contains(expected), "expected {expected}, got {error:#}");
        }
        Ok(())
    }

    #[test]
    fn missing_and_malformed_matrices_fail_with_input_path_context() -> Result<()> {
        let root = tempfile::tempdir()?;
        let missing = load_benchmark_local_stage_inventory(root.path(), Domain::Fastq)
            .err()
            .context("missing matrix was admitted")?;
        assert!(missing.to_string().contains("fastq-stage-matrix.toml"));
        write_matrix(root.path(), "fastq", "not valid TOML !")?;
        let malformed = load_benchmark_local_stage_inventory(root.path(), Domain::Fastq)
            .err()
            .context("malformed matrix was admitted")?;
        assert!(malformed.to_string().contains("parse"));
        Ok(())
    }

    #[test]
    fn governed_domain_inventories_retain_their_stage_counts() -> Result<()> {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        for (domain, count) in [(Domain::Fastq, 27), (Domain::Bam, 24), (Domain::Vcf, 20)] {
            let report = load_benchmark_local_stage_inventory(&root, domain)?;
            assert_eq!(report.stage_count, count);
            assert_eq!(report.stages.len(), count);
            assert_eq!(report.domain, domain.as_str());
        }
        Ok(())
    }

    #[test]
    fn vcf_matrix_schema_admission_uses_the_declared_repository_input() -> Result<()> {
        let root = tempfile::tempdir()?;
        let governed = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../configs/bench/local/vcf-stage-matrix.toml");
        let input = fs::read_to_string(governed)?;
        write_matrix(root.path(), "vcf", &input)?;
        assert_eq!(load_benchmark_local_stage_inventory(root.path(), Domain::Vcf)?.stage_count, 20);
        write_matrix(
            root.path(),
            "vcf",
            &input.replace("local_stage_matrix.v1", "local_stage_matrix.unknown"),
        )?;
        assert!(load_benchmark_local_stage_inventory(root.path(), Domain::Vcf).is_err());
        Ok(())
    }

    #[test]
    fn aggregate_publication_retains_requested_domain_order_and_exact_output() -> Result<()> {
        let root = tempfile::tempdir()?;
        for domain in ["fastq", "bam"] {
            write_matrix(root.path(), domain, &matrix(domain))?;
        }
        let output = PathBuf::from("artifacts/inventory.json");
        let report = render_benchmark_all_domain_stage_inventory(
            root.path(),
            &[Domain::Bam, Domain::Fastq],
            output.clone(),
        )?;
        assert_eq!(report.selected_domains, ["bam", "fastq"]);
        assert_eq!(report.total_stage_count, 4);
        let stored: bijux_dna_bench_model::BenchmarkLocalAllDomainStageInventory =
            serde_json::from_slice(&fs::read(root.path().join(output))?)?;
        assert_eq!(stored, report);
        Ok(())
    }

    #[test]
    fn invalid_aggregate_selection_preserves_existing_output() -> Result<()> {
        let root = tempfile::tempdir()?;
        write_matrix(root.path(), "fastq", &matrix("fastq"))?;
        let output = root.path().join("artifacts/inventory.json");
        fs::create_dir_all(output.parent().context("output parent")?)?;
        fs::write(&output, "previous report")?;
        for domains in [Vec::new(), vec![Domain::Fastq, Domain::Fastq]] {
            assert!(render_benchmark_all_domain_stage_inventory(
                root.path(),
                &domains,
                output.clone()
            )
            .is_err());
            assert_eq!(fs::read_to_string(&output)?, "previous report");
        }
        Ok(())
    }
}
