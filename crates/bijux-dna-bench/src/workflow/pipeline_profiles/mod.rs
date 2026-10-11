//! Owner: bijux-dna-bench
//! Benchmark scientific profile admission over explicit evidence providers.

use std::collections::{BTreeMap, BTreeSet};

use anyhow::{anyhow, Result};
use bijux_dna_bench_model::{
    BenchmarkLocalCorpusStageCompatibilityEntry as LocalCorpusStageCompatibilityEntryReport,
    BenchmarkLocalPipelineDagConfig as LocalPipelineDagConfig,
    BenchmarkLocalPipelineDagDomain as LocalPipelineDagDomain,
    BenchmarkLocalPipelineDagNode as LocalPipelineDagNode,
    BenchmarkLocalPipelineDagValidationProfile as LocalPipelineDagValidationProfileReport,
    VcfStageMatrixRow,
};

mod amplicon;
mod ancient_dna;
mod evidence;
mod germline;
mod imputation;
mod population;
mod taxonomy;

pub use evidence::PipelineProfileEvidence;

/// Validate selected benchmark profiles while retaining declared evidence checks.
///
/// Profile selection follows stage membership and governed pipeline identities.
/// This operation does not execute pipelines or qualify real tool substitutions.
///
/// # Errors
/// Rejects incompatible profile wiring, corpus/binding evidence, or provider errors.
pub fn validate_benchmark_pipeline_profiles(
    config: &LocalPipelineDagConfig,
    evidence: &dyn PipelineProfileEvidence,
) -> Result<Vec<LocalPipelineDagValidationProfileReport>> {
    let mut profiles = Vec::new();
    let stage_ids = config.nodes.iter().map(|node| node.stage_id.as_str()).collect::<BTreeSet<_>>();

    if stage_ids.contains("vcf.call_pseudohaploid") || stage_ids.contains("vcf.damage_filter") {
        profiles.push(ancient_dna::validate_ancient_dna_pseudohaploid_profile(evidence, config)?);
    }
    if stage_ids.contains("vcf.call_gl") || stage_ids.contains("vcf.gl_propagation") {
        profiles.push(ancient_dna::validate_ancient_dna_gl_profile(evidence, config)?);
    }
    if stage_ids.contains("vcf.call_diploid") && stage_ids.contains("bam.recalibration") {
        profiles.push(germline::validate_diploid_small_sample_profile(evidence, config)?);
    }
    if stage_ids.contains("vcf.prepare_reference_panel") && stage_ids.contains("vcf.impute") {
        profiles.push(imputation::validate_reference_panel_imputation_profile(evidence, config)?);
    }
    if stage_ids.contains("vcf.population_structure") && stage_ids.contains("vcf.admixture") {
        profiles.push(population::validate_population_structure_vcf_profile(evidence, config)?);
    }
    if stage_ids.contains("vcf.ibd")
        && stage_ids.contains("vcf.roh")
        && stage_ids.contains("vcf.demography")
    {
        profiles.push(population::validate_relatedness_segments_vcf_profile(evidence, config)?);
    }
    if stage_ids.contains("bam.genotyping")
        && stage_ids.contains("vcf.filter")
        && stage_ids.contains("vcf.stats")
        && stage_ids.contains("vcf.qc")
        && stage_ids.contains("vcf.pca")
        && stage_ids.contains("vcf.roh")
    {
        profiles.push(germline::validate_bam_genotyping_vcf_downstream_profile(evidence, config)?);
    }
    if config.pipeline_id == "edna-taxonomy-no-vcf" {
        profiles.push(taxonomy::validate_edna_taxonomy_no_vcf_profile(evidence, config)?);
    }
    if config.pipeline_id == "amplicon-asv-otu-no-vcf" {
        profiles.push(amplicon::validate_amplicon_asv_otu_no_vcf_profile(evidence, config)?);
    }

    Ok(profiles)
}

fn require_profile_stage<'a>(
    stage_index: &'a BTreeMap<String, &'a LocalPipelineDagNode>,
    profile_id: &str,
    stage_id: &str,
) -> Result<&'a LocalPipelineDagNode> {
    stage_index
        .get(stage_id)
        .copied()
        .ok_or_else(|| anyhow!("{profile_id} local pipeline DAG must declare stage `{stage_id}`"))
}

fn require_list_contains(
    values: &[String],
    expected: &str,
    profile_id: &str,
    message: &str,
) -> Result<()> {
    if values.iter().any(|value| value == expected) {
        Ok(())
    } else {
        Err(anyhow!("{profile_id}: {message}"))
    }
}

fn require_compatibility_fixture(
    compatibility_index: &BTreeMap<&str, &LocalCorpusStageCompatibilityEntryReport>,
    profile_id: &str,
    stage_id: &str,
    expected_corpus_family_id: &str,
    expected_fixture_id: &str,
) -> Result<()> {
    let Some(entry) = compatibility_index.get(stage_id).copied() else {
        return Err(anyhow!(
            "{profile_id} local pipeline DAG cannot confirm corpus compatibility for missing stage `{stage_id}`"
        ));
    };
    if entry.corpus_family_id.as_deref() != Some(expected_corpus_family_id)
        || entry.fixture_id.as_deref() != Some(expected_fixture_id)
    {
        return Err(anyhow!(
            "{profile_id} local pipeline DAG expected `{stage_id}` to remain governed by `{expected_corpus_family_id}` / `{expected_fixture_id}`, found {:?} / {:?}",
            entry.corpus_family_id,
            entry.fixture_id
        ));
    }
    Ok(())
}

fn require_vcf_matrix_binding(
    matrix_index: &BTreeMap<String, VcfStageMatrixRow>,
    profile_id: &str,
    stage_id: &str,
    expected_asset_profile_id: &str,
    expected_corpus_id: &str,
) -> Result<()> {
    let Some(row) = matrix_index.get(stage_id) else {
        return Err(anyhow!(
            "{profile_id} local pipeline DAG cannot confirm governed VCF stage-matrix coverage for missing stage `{stage_id}`"
        ));
    };
    if row.asset_profile_id != expected_asset_profile_id || row.corpus_id != expected_corpus_id {
        return Err(anyhow!(
            "{profile_id} local pipeline DAG expected `{stage_id}` to remain governed by asset profile `{expected_asset_profile_id}` and corpus `{expected_corpus_id}`, found `{}` / `{}`",
            row.asset_profile_id,
            row.corpus_id
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
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
        fn corpus_stage_compatibility(
            &self,
        ) -> Result<BenchmarkLocalCorpusStageCompatibilityReport> {
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
            assert!(
                error.contains(&format!("must declare stage `{missing}`")),
                "{pipeline}: {error}"
            );
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
            assert_eq!(
                evidence.calls.get(),
                1,
                "{pipeline}: failure must stop further acquisition"
            );
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
}
