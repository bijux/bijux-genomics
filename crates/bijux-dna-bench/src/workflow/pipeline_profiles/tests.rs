//! Owner: bijux-dna-bench
use std::cell::Cell;
use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::BenchmarkLocalCorpusStageCompatibilityReport;

use super::{
    validate_benchmark_pipeline_profiles, LocalPipelineDagConfig, PipelineProfileEvidence,
    VcfStageMatrixRow,
};

#[derive(Default)]
struct UnavailableEvidence {
    calls: Cell<usize>,
}

impl PipelineProfileEvidence for UnavailableEvidence {
    fn corpus_stage_compatibility(&self) -> Result<BenchmarkLocalCorpusStageCompatibilityReport> {
        self.calls.set(self.calls.get() + 1);
        bail!("corpus evidence unavailable")
    }

    fn vcf_stage_matrix(&self) -> Result<BTreeMap<String, VcfStageMatrixRow>> {
        self.calls.set(self.calls.get() + 1);
        bail!("VCF evidence unavailable")
    }
}

fn load(pipeline: &str) -> Result<LocalPipelineDagConfig> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../benchmarks/configs/pipelines/local")
        .join(format!("{pipeline}.toml"));
    Ok(toml::from_str(&std::fs::read_to_string(path)?)?)
}

const PROFILE_INPUTS: &[(&str, &str, &str)] = &[
    ("adna-pseudohaploid-fastq-bam-vcf", "bam.align", "corpus evidence unavailable"),
    ("adna-gl-fastq-bam-vcf", "bam.align", "corpus evidence unavailable"),
    ("diploid-small-fastq-bam-vcf", "bam.filter", "corpus evidence unavailable"),
    ("bam-genotyping-to-vcf-downstream", "bam.filter", "corpus evidence unavailable"),
    ("reference-panel-imputation", "vcf.qc", "VCF evidence unavailable"),
    ("popgen-structure-vcf", "vcf.qc", "VCF evidence unavailable"),
    ("relatedness-segments-vcf", "vcf.qc", "VCF evidence unavailable"),
    ("edna-taxonomy-no-vcf", "fastq.validate_reads", "corpus evidence unavailable"),
    ("amplicon-asv-otu-no-vcf", "fastq.validate_reads", "corpus evidence unavailable"),
];

#[test]
fn profiles_reject_missing_required_stages_before_acquiring_evidence() -> Result<()> {
    for &(pipeline, missing, _) in PROFILE_INPUTS {
        let mut config = load(pipeline)?;
        assert!(config.nodes.iter().any(|node| node.stage_id == missing), "{pipeline}");
        config.nodes.retain(|node| node.stage_id != missing);
        let evidence = UnavailableEvidence::default();
        let error = validate_benchmark_pipeline_profiles(&config, &evidence)
            .err()
            .context("missing profile stage must fail")?
            .to_string();
        assert!(error.contains(&format!("must declare stage `{missing}`")), "{pipeline}: {error}");
        assert_eq!(
            evidence.calls.get(),
            0,
            "{pipeline}: evidence was requested before wiring admission"
        );
    }
    Ok(())
}

#[test]
fn profiles_preserve_evidence_acquisition_failures() -> Result<()> {
    for &(pipeline, _, expected) in PROFILE_INPUTS {
        let config = load(pipeline)?;
        let evidence = UnavailableEvidence::default();
        let error = validate_benchmark_pipeline_profiles(&config, &evidence)
            .err()
            .context("unavailable evidence cannot pass")?
            .to_string();
        assert_eq!(error, expected, "{pipeline}");
        assert_eq!(evidence.calls.get(), 1, "{pipeline}: failure must stop further acquisition");
    }
    Ok(())
}

#[test]
fn ordinary_preprocessing_does_not_acquire_unrelated_profile_evidence() -> Result<()> {
    let config = load("fastq-core-preprocess")?;
    let evidence = UnavailableEvidence::default();
    assert!(validate_benchmark_pipeline_profiles(&config, &evidence)?.is_empty());
    assert_eq!(evidence.calls.get(), 0);
    Ok(())
}
