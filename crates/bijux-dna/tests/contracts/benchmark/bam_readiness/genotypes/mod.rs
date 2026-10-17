mod genotyping_complete;
mod genotyping_complete_file;
mod kinship_complete;
mod kinship_complete_file;
mod kinship_ready;
mod kinship_ready_file;
mod recalibration_complete;
mod recalibration_complete_file;
mod recalibration_genotyping_ready;
mod recalibration_genotyping_ready_file;

#[cfg(test)]
mod fixture_inputs {
    use std::cell::RefCell;
    use std::path::PathBuf;

    use anyhow::Result;

    use crate::support::RepoSandbox;

    const MANIFEST: &str = "benchmarks/tests/fixtures/corpora/captured/manifest.toml";

    fn indexed_inputs() -> Vec<PathBuf> {
        [
            "Cargo.toml",
            "Cargo.lock",
            "crates/bijux-dna/Cargo.toml",
            "configs/bench/fixture.toml",
            "configs/runtime/fixture.toml",
            "configs/ci/registry/fixture.toml",
            "configs/ci/tools/fixture.toml",
            "configs/vcf/fixture.toml",
            "domain/fastq/fixture.toml",
            "domain/bam/fixture.toml",
            "domain/vcf/fixture.toml",
            "containers/contracts/fixture.toml",
            "containers/docker/Dockerfile",
            "containers/apptainer/fixture.def",
            "benchmarks/contracts/fixture.toml",
            "benchmarks/configs/pipelines/local/captured.toml",
            "benchmarks/corpora/fixture.toml",
            MANIFEST,
            "benchmarks/tests/fixtures/bench/parsers/vcf/bcftools/vcf.stats/raw.txt",
            "benchmarks/tests/fixtures/bench/parsers/vcf/bcftools/vcf.stats/expected.normalized.json",
            "assets/reference/captured/reference.fa",
            "assets/toy/captured/reads.fastq",
            "assets/toy/captured/reads.fastq.metadata.json",
        ]
        .into_iter()
        .map(PathBuf::from)
        .collect()
    }

    fn manifest() -> Vec<u8> {
        b"fixture_id = 'captured'\nreads = ['assets/toy/captured/reads.fastq']\n".to_vec()
    }

    #[test]
    fn fixture_projection_keeps_declared_asset_family_and_excludes_unrelated_inputs() -> Result<()>
    {
        let mut indexed = indexed_inputs();
        indexed.extend([
            PathBuf::from("crates/bijux-dna/src/lib.rs"),
            PathBuf::from("docs/COMMANDS.md"),
            PathBuf::from("artifacts/benchmark-smoke/previous.json"),
            PathBuf::from("assets/toy/unrelated/reads.fastq"),
        ]);
        let selected = RepoSandbox::bam_genotype_inputs(&indexed, &|_| Ok(manifest()))?;
        assert!(selected.contains(&PathBuf::from(MANIFEST)));
        assert!(selected.contains(&PathBuf::from(
            "benchmarks/tests/fixtures/bench/parsers/vcf/bcftools/vcf.stats/raw.txt"
        )));
        assert!(selected.contains(&PathBuf::from(
            "benchmarks/tests/fixtures/bench/parsers/vcf/bcftools/vcf.stats/expected.normalized.json"
        )));
        assert!(
            selected.contains(&PathBuf::from("benchmarks/configs/pipelines/local/captured.toml"))
        );
        assert!(selected.contains(&PathBuf::from("assets/toy/captured/reads.fastq")));
        assert!(selected.contains(&PathBuf::from("assets/toy/captured/reads.fastq.metadata.json")));
        for omitted in &indexed[indexed.len() - 4..] {
            assert!(!selected.contains(omitted), "unexpected fixture input: {}", omitted.display());
        }
        Ok(())
    }

    #[test]
    fn fixture_projection_rejects_missing_components_and_asset_paths() {
        let mut indexed = indexed_inputs();
        indexed.retain(|path| !path.starts_with("domain/bam"));
        indexed.push(PathBuf::from("domain/bam-lookalike/fixture.toml"));
        assert!(RepoSandbox::bam_genotype_inputs(&indexed, &|_| Ok(manifest())).is_err());
        let mut without_pipeline = indexed_inputs();
        without_pipeline.retain(|path| !path.starts_with("benchmarks/configs/pipelines/local"));
        assert!(RepoSandbox::bam_genotype_inputs(&without_pipeline, &|_| Ok(manifest())).is_err());
        let mut without_parser = indexed_inputs();
        without_parser
            .retain(|path| !path.starts_with("benchmarks/tests/fixtures/bench/parsers/vcf"));
        without_parser.push(PathBuf::from(
            "benchmarks/tests/fixtures/bench/parsers/vcf-lookalike/bcftools/vcf.stats/raw.txt",
        ));
        assert!(RepoSandbox::bam_genotype_inputs(&without_parser, &|_| Ok(manifest())).is_err());
        for reference in
            ["assets/toy/captured/../outside.fastq", "assets/toy/captured/absent.fastq"]
        {
            let body = format!("reads = ['{reference}']\n");
            assert!(RepoSandbox::bam_genotype_inputs(&indexed_inputs(), &|_| {
                Ok(body.as_bytes().to_vec())
            })
            .is_err());
        }
    }

    #[test]
    fn fixture_projection_reads_manifest_bytes_through_index_authority() -> Result<()> {
        let reads = RefCell::new(Vec::new());
        RepoSandbox::bam_genotype_inputs(&indexed_inputs(), &|path| {
            reads.borrow_mut().push(path.to_path_buf());
            Ok(manifest())
        })?;
        assert_eq!(*reads.borrow(), [PathBuf::from(MANIFEST)]);
        let unavailable = RepoSandbox::bam_genotype_inputs(&indexed_inputs(), &|_| {
            anyhow::bail!("admitted index blob unavailable")
        });
        let error = unavailable
            .err()
            .ok_or_else(|| anyhow::anyhow!("unavailable indexed manifest must refuse"))?;
        assert!(error.to_string().contains("admitted index blob unavailable"));
        assert!(RepoSandbox::bam_genotype_inputs(&indexed_inputs(), &|_| {
            Ok(b"reads = [".to_vec())
        })
        .is_err());
        Ok(())
    }
}
