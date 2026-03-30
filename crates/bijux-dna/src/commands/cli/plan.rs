#![allow(clippy::too_many_lines)]

use anyhow::{anyhow, Result};
use bijux_dna_api::v1::api::bench::fastq_args as engine_args;
use bijux_dna_api::v1::api::run::{StageId, ToolId};
use std::path::PathBuf;

use crate::commands::cli::env::registry_tools_for_stage;
use crate::commands::cli::parse::{
    BamCommand, BenchFastqClusterOtusArgs, BenchFastqCorrectArgs, BenchFastqDepleteHostArgs,
    BenchFastqDepleteReferenceContaminantsArgs, BenchFastqDepleteRrnaArgs,
    BenchFastqDetectAdaptersArgs, BenchFastqFilterArgs, BenchFastqFilterLowComplexityArgs,
    BenchFastqIndexReferenceArgs, BenchFastqInferAsvsArgs, BenchFastqMergeArgs,
    BenchFastqNormalizeAbundanceArgs, BenchFastqNormalizePrimersArgs, BenchFastqPreprocessArgs,
    BenchFastqProfileOverrepresentedArgs, BenchFastqProfileReadLengthsArgs, BenchFastqQcPostArgs,
    BenchFastqRemoveChimerasArgs, BenchFastqRemoveDuplicatesArgs, BenchFastqScreenArgs,
    BenchFastqStatsArgs, BenchFastqTrimArgs, BenchFastqTrimPolygArgs,
    BenchFastqTrimTerminalDamageArgs, BenchFastqUmiArgs, BenchFastqValidateArgs, CommonArgs,
    DnaCommand, FastqCommand, FastqPreprocessArgs, FastqTrimArgs, FastqValidateArgs, VcfCommand,
};

const TOOL_CUTADAPT: &str = concat!("cuta", "dapt");
const TOOL_FASTP: &str = concat!("fast", "p");
const TOOL_MULTIQC: &str = "multiqc";
const TOOL_SAMTOOLS: &str = concat!("sam", "tools");
const TOOL_SEQKIT_STATS: &str = "seqkit_stats";

#[must_use]
pub fn resolve_stage_tool(command: &DnaCommand) -> (StageId, ToolId, CommonArgs) {
    match command {
        DnaCommand::Fastq(args) => match &args.command {
            FastqCommand::ListStages
            | FastqCommand::Stages
            | FastqCommand::Doctor
            | FastqCommand::ListTools { .. }
            | FastqCommand::Explain { .. }
            | FastqCommand::Compare(_)
            | FastqCommand::Run(_) => (
                StageId::from_static("fastq.trim_reads"),
                ToolId::from_static(TOOL_FASTP),
                CommonArgs::default(),
            ),
            FastqCommand::Trim(args) => (
                StageId::from_static("fastq.trim_reads"),
                ToolId::from_static(TOOL_FASTP),
                args.common.clone(),
            ),
            FastqCommand::ValidateReads(args) => (
                StageId::from_static("fastq.validate_reads"),
                ToolId::from_static("fastqvalidator"),
                args.common.clone(),
            ),
            FastqCommand::ProfileReads(common) => (
                StageId::from_static("fastq.profile_reads"),
                ToolId::from_static(TOOL_SEQKIT_STATS),
                common.clone(),
            ),
            FastqCommand::Filter(args) => (
                StageId::from_static("fastq.filter_reads"),
                ToolId::from_static(TOOL_FASTP),
                args.common.clone(),
            ),
            FastqCommand::Merge(common) => (
                StageId::from_static("fastq.merge_pairs"),
                ToolId::from_static("vsearch"),
                common.clone(),
            ),
            FastqCommand::Contam(common) => (
                StageId::from_static("fastq.deplete_reference_contaminants"),
                ToolId::from_static("bowtie2"),
                common.clone(),
            ),
            FastqCommand::Umi(common) => (
                StageId::from_static("fastq.extract_umis"),
                ToolId::from_static("umi_tools"),
                common.clone(),
            ),
            FastqCommand::ErrorCorrect(common) => (
                StageId::from_static("fastq.correct_errors"),
                ToolId::from_static("rcorrector"),
                common.clone(),
            ),
            FastqCommand::Qc(common) => (
                StageId::from_static("fastq.report_qc"),
                ToolId::from_static(TOOL_MULTIQC),
                common.clone(),
            ),
            FastqCommand::Align(common) => (
                StageId::from_static("fastq.index_reference"),
                ToolId::from_static("bowtie2_build"),
                common.clone(),
            ),
            FastqCommand::Preprocess(args) => (
                StageId::from_static("fastq.validate_reads"),
                ToolId::from_static("fastqvalidator"),
                args.common.clone(),
            ),
        },
        DnaCommand::Bam(args) => match &args.command {
            BamCommand::ListStages | BamCommand::Explain { .. } => (
                StageId::from_static("bam.validate"),
                ToolId::from_static(TOOL_SAMTOOLS),
                CommonArgs::default(),
            ),
            BamCommand::Run(args) => (
                StageId::new(args.stage.stage().as_str()),
                ToolId::new(
                    args.tool
                        .clone()
                        .unwrap_or_else(|| TOOL_SAMTOOLS.to_string()),
                ),
                CommonArgs::default(),
            ),
        },
        DnaCommand::Vcf(args) => match &args.command {
            VcfCommand::Plan { .. } | VcfCommand::Explain { .. } => (
                StageId::from_static("vcf.stats"),
                ToolId::from_static("bcftools"),
                CommonArgs::default(),
            ),
            VcfCommand::Run(args) => (
                StageId::from_static("vcf.stats"),
                ToolId::new(args.tool.clone().unwrap_or_else(|| "bcftools".to_string())),
                CommonArgs::default(),
            ),
        },
        _ => (
            StageId::from_static("fastq.trim_reads"),
            ToolId::from_static(TOOL_FASTP),
            CommonArgs::default(),
        ),
    }
}

/// # Errors
/// Returns an error if tool mode cannot be resolved for this stage.
pub fn bench_args_trim(args: &BenchFastqTrimArgs) -> Result<engine_args::BenchFastqTrimArgs> {
    Ok(engine_args::BenchFastqTrimArgs {
        sample_id: args.sample_id.clone(),
        r1: args.r1.clone(),
        r2: args.r2.clone(),
        out: args.out.clone(),
        tools: resolve_bench_tools("fastq.trim_reads", &args.tools)?,
        explain: args.explain,
        replicates: args.replicates,
        jobs: args.jobs,
        ci_bootstrap: args.ci_bootstrap,
        threads: args.threads,
        adapter_bank_preset: args.adapter_bank_preset.clone(),
        adapter_bank: args.adapter_bank.clone(),
        adapter_bank_file: args.adapter_bank_file.clone(),
        enable_adapters: args.enable_adapter.clone(),
        disable_adapters: args.disable_adapter.clone(),
        polyx_preset: args.polyx_preset.clone(),
        contaminant_preset: args.contaminant_preset.clone(),
        min_length: args.min_length,
        quality_cutoff: args.quality_cutoff,
        n_policy: args.n_policy.clone(),
        adapter_policy: args.adapter_policy.clone(),
        polyx_policy: args.polyx_policy.clone(),
        contaminant_policy: args.contaminant_policy.clone(),
    })
}

/// # Errors
/// Returns an error if tool mode cannot be resolved for this stage.
pub fn bench_args_trim_polyg(
    args: &BenchFastqTrimPolygArgs,
) -> Result<engine_args::BenchFastqTrimPolygArgs> {
    Ok(engine_args::BenchFastqTrimPolygArgs {
        sample_id: args.sample_id.clone(),
        r1: args.r1.clone(),
        r2: args.r2.clone(),
        out: args.out.clone(),
        tools: resolve_bench_tools("fastq.trim_polyg_tails", &args.tools)?,
        explain: args.explain,
        replicates: args.replicates,
        jobs: args.jobs,
        ci_bootstrap: args.ci_bootstrap,
        threads: args.threads,
        trim_polyg: args.trim_polyg,
        polyx_preset: args.polyx_preset.clone(),
        min_polyg_run: args.min_polyg_run,
    })
}

/// # Errors
/// Returns an error if tool mode cannot be resolved for this stage.
pub fn bench_args_trim_terminal_damage(
    args: &BenchFastqTrimTerminalDamageArgs,
) -> Result<engine_args::BenchFastqTrimTerminalDamageArgs> {
    Ok(engine_args::BenchFastqTrimTerminalDamageArgs {
        sample_id: args.sample_id.clone(),
        r1: args.r1.clone(),
        r2: args.r2.clone(),
        out: args.out.clone(),
        tools: resolve_bench_tools("fastq.trim_terminal_damage", &args.tools)?,
        explain: args.explain,
        replicates: args.replicates,
        jobs: args.jobs,
        ci_bootstrap: args.ci_bootstrap,
        threads: args.threads,
        damage_mode: args.damage_mode.clone(),
        execution_policy: args.execution_policy.clone(),
        trim_5p_bases: args.trim_5p_bases,
        trim_3p_bases: args.trim_3p_bases,
    })
}

/// # Errors
/// Returns an error if tool mode cannot be resolved for this stage.
pub fn bench_args_validate(
    args: &BenchFastqValidateArgs,
) -> Result<engine_args::BenchFastqValidateArgs> {
    let (strict, validation_mode) =
        normalize_validate_failure_flags(args.strict, args.validation_mode.as_deref())?;
    Ok(engine_args::BenchFastqValidateArgs {
        sample_id: args.sample_id.clone(),
        r1: args.r1.clone(),
        r2: args.r2.clone(),
        out: args.out.clone(),
        tools: resolve_bench_tools("fastq.validate_reads", &args.tools)?,
        explain: args.explain,
        strict,
        replicates: args.replicates,
        jobs: args.jobs,
        ci_bootstrap: args.ci_bootstrap,
        threads: args.threads,
        validation_mode,
        pair_sync_policy: args.pair_sync_policy.clone(),
    })
}

/// # Errors
/// Returns an error if tool mode cannot be resolved for this stage.
pub fn bench_args_detect_adapters(
    args: &BenchFastqDetectAdaptersArgs,
) -> Result<engine_args::BenchFastqDetectAdaptersArgs> {
    Ok(engine_args::BenchFastqDetectAdaptersArgs {
        sample_id: args.sample_id.clone(),
        r1: args.r1.clone(),
        r2: args.r2.clone(),
        out: args.out.clone(),
        tools: resolve_bench_tools("fastq.detect_adapters", &args.tools)?,
        explain: args.explain,
        replicates: args.replicates,
        jobs: args.jobs,
        ci_bootstrap: args.ci_bootstrap,
        threads: args.threads,
    })
}

/// # Errors
/// Returns an error if tool mode cannot be resolved for this stage.
pub fn bench_args_profile_read_lengths(
    args: &BenchFastqProfileReadLengthsArgs,
) -> Result<engine_args::BenchFastqProfileReadLengthsArgs> {
    Ok(engine_args::BenchFastqProfileReadLengthsArgs {
        sample_id: args.sample_id.clone(),
        r1: args.r1.clone(),
        r2: args.r2.clone(),
        out: args.out.clone(),
        tools: resolve_bench_tools("fastq.profile_read_lengths", &args.tools)?,
        explain: args.explain,
        replicates: args.replicates,
        jobs: args.jobs,
        ci_bootstrap: args.ci_bootstrap,
        threads: args.threads,
        histogram_bins: args.histogram_bins,
    })
}

/// # Errors
/// Returns an error if tool mode cannot be resolved for this stage.
pub fn bench_args_filter(args: &BenchFastqFilterArgs) -> Result<engine_args::BenchFastqFilterArgs> {
    Ok(engine_args::BenchFastqFilterArgs {
        sample_id: args.sample_id.clone(),
        r1: args.r1.clone(),
        r2: args.r2.clone(),
        out: args.out.clone(),
        tools: resolve_bench_tools("fastq.filter_reads", &args.tools)?,
        explain: args.explain,
        replicates: args.replicates,
        jobs: args.jobs,
        ci_bootstrap: args.ci_bootstrap,
        threads: args.threads,
        max_n: args.max_n,
        max_n_fraction: args.max_n_fraction,
        max_n_count: args.max_n_count,
        low_complexity_threshold: args.low_complexity_threshold,
        entropy_threshold: args.entropy_threshold,
        kmer_ref: args.kmer_ref.clone(),
        polyx_policy: args.polyx_policy.clone(),
    })
}

/// # Errors
/// Returns an error if tool mode cannot be resolved for this stage.
pub fn bench_args_filter_low_complexity(
    args: &BenchFastqFilterLowComplexityArgs,
) -> Result<engine_args::BenchFastqFilterLowComplexityArgs> {
    Ok(engine_args::BenchFastqFilterLowComplexityArgs {
        sample_id: args.sample_id.clone(),
        r1: args.r1.clone(),
        r2: args.r2.clone(),
        out: args.out.clone(),
        tools: resolve_bench_tools("fastq.filter_low_complexity", &args.tools)?,
        explain: args.explain,
        replicates: args.replicates,
        jobs: args.jobs,
        ci_bootstrap: args.ci_bootstrap,
        entropy_threshold: args.entropy_threshold,
        polyx_threshold: args.polyx_threshold,
    })
}

/// # Errors
/// Returns an error if tool mode cannot be resolved for this stage.
pub fn bench_args_merge(args: &BenchFastqMergeArgs) -> Result<engine_args::BenchFastqMergeArgs> {
    Ok(engine_args::BenchFastqMergeArgs {
        sample_id: args.sample_id.clone(),
        r1: args.r1.clone(),
        r2: args.r2.clone(),
        out: args.out.clone(),
        tools: resolve_bench_tools("fastq.merge_pairs", &args.tools)?,
        explain: args.explain,
        threads: args.threads,
        merge_overlap: args.merge_overlap,
        min_length: args.min_length,
        unmerged_read_policy: args.unmerged_read_policy.clone(),
        replicates: args.replicates,
        jobs: args.jobs,
        ci_bootstrap: args.ci_bootstrap,
    })
}

/// # Errors
/// Returns an error if tool mode cannot be resolved for this stage.
pub fn bench_args_remove_duplicates(
    args: &BenchFastqRemoveDuplicatesArgs,
) -> Result<engine_args::BenchFastqRemoveDuplicatesArgs> {
    Ok(engine_args::BenchFastqRemoveDuplicatesArgs {
        sample_id: args.sample_id.clone(),
        r1: args.r1.clone(),
        r2: args.r2.clone(),
        out: args.out.clone(),
        tools: resolve_bench_tools("fastq.remove_duplicates", &args.tools)?,
        tools_resolved_implicitly: bench_tools_resolved_implicitly(&args.tools),
        explain: args.explain,
        threads: args.threads,
        dedup_mode: args.dedup_mode.clone(),
        keep_order: args.keep_order,
        replicates: args.replicates,
        jobs: args.jobs,
        ci_bootstrap: args.ci_bootstrap,
    })
}

/// # Errors
/// Returns an error if tool mode cannot be resolved for this stage.
pub fn bench_args_remove_chimeras(
    args: &BenchFastqRemoveChimerasArgs,
) -> Result<engine_args::BenchFastqRemoveChimerasArgs> {
    Ok(engine_args::BenchFastqRemoveChimerasArgs {
        sample_id: args.sample_id.clone(),
        r1: args.r1.clone(),
        r2: args.r2.clone(),
        out: args.out.clone(),
        tools: resolve_bench_tools("fastq.remove_chimeras", &args.tools)?,
        explain: args.explain,
        replicates: args.replicates,
        jobs: args.jobs,
        ci_bootstrap: args.ci_bootstrap,
        threads: args.threads,
    })
}

/// # Errors
/// Returns an error if tool mode cannot be resolved for this stage.
pub fn bench_args_normalize_primers(
    args: &BenchFastqNormalizePrimersArgs,
) -> Result<engine_args::BenchFastqNormalizePrimersArgs> {
    Ok(engine_args::BenchFastqNormalizePrimersArgs {
        sample_id: args.sample_id.clone(),
        r1: args.r1.clone(),
        r2: args.r2.clone(),
        out: args.out.clone(),
        tools: resolve_bench_tools("fastq.normalize_primers", &args.tools)?,
        explain: args.explain,
        replicates: args.replicates,
        jobs: args.jobs,
        ci_bootstrap: args.ci_bootstrap,
        primer_set_id: args.primer_set_id.clone(),
        orientation_policy: args.orientation_policy.clone(),
        max_mismatch_rate: args.max_mismatch_rate,
        min_overlap_bp: args.min_overlap_bp,
        strict_5p_anchor: args.strict_5p_anchor,
        allow_iupac_codes: args.allow_iupac_codes,
    })
}

/// # Errors
/// Returns an error if tool mode cannot be resolved for this stage.
pub fn bench_args_infer_asvs(
    args: &BenchFastqInferAsvsArgs,
) -> Result<engine_args::BenchFastqInferAsvsArgs> {
    Ok(engine_args::BenchFastqInferAsvsArgs {
        sample_id: args.sample_id.clone(),
        r1: args.r1.clone(),
        r2: args.r2.clone(),
        out: args.out.clone(),
        tools: resolve_bench_tools("fastq.infer_asvs", &args.tools)?,
        explain: args.explain,
        replicates: args.replicates,
        jobs: args.jobs,
        ci_bootstrap: args.ci_bootstrap,
        denoising_method: args.denoising_method.clone(),
        pooling_mode: args.pooling_mode.clone(),
        chimera_policy: args.chimera_policy.clone(),
        threads: args.threads,
    })
}

/// # Errors
/// Returns an error if tool mode cannot be resolved for this stage.
pub fn bench_args_cluster_otus(
    args: &BenchFastqClusterOtusArgs,
) -> Result<engine_args::BenchFastqClusterOtusArgs> {
    Ok(engine_args::BenchFastqClusterOtusArgs {
        sample_id: args.sample_id.clone(),
        r1: args.r1.clone(),
        r2: args.r2.clone(),
        out: args.out.clone(),
        tools: resolve_bench_tools("fastq.cluster_otus", &args.tools)?,
        explain: args.explain,
        replicates: args.replicates,
        jobs: args.jobs,
        ci_bootstrap: args.ci_bootstrap,
        otu_identity: args.otu_identity,
        threads: args.threads,
    })
}

/// # Errors
/// Returns an error if tool mode cannot be resolved for this stage.
pub fn bench_args_normalize_abundance(
    args: &BenchFastqNormalizeAbundanceArgs,
) -> Result<engine_args::BenchFastqNormalizeAbundanceArgs> {
    Ok(engine_args::BenchFastqNormalizeAbundanceArgs {
        sample_id: args.sample_id.clone(),
        table: args.table.clone(),
        out: args.out.clone(),
        tools: resolve_bench_tools("fastq.normalize_abundance", &args.tools)?,
        explain: args.explain,
        replicates: args.replicates,
        jobs: args.jobs,
        ci_bootstrap: args.ci_bootstrap,
        method: args.method.clone(),
    })
}

/// # Errors
/// Returns an error if tool mode cannot be resolved for this stage.
pub fn bench_args_correct(
    args: &BenchFastqCorrectArgs,
) -> Result<engine_args::BenchFastqCorrectArgs> {
    Ok(engine_args::BenchFastqCorrectArgs {
        sample_id: args.sample_id.clone(),
        r1: args.r1.clone(),
        r2: args.r2.clone(),
        out: args.out.clone(),
        tools: resolve_bench_tools("fastq.correct_errors", &args.tools)?,
        explain: args.explain,
        replicates: args.replicates,
        jobs: args.jobs,
        ci_bootstrap: args.ci_bootstrap,
        threads: args.threads,
        quality_encoding: args.quality_encoding.clone(),
        kmer_size: args.kmer_size,
        musket_kmer_budget: args.musket_kmer_budget,
        genome_size: args.genome_size,
        max_memory_gb: args.max_memory_gb,
        trusted_kmer_artifact: args.trusted_kmer_artifact.clone(),
        conservative_mode: args.conservative_mode,
    })
}

/// # Errors
/// Returns an error if tool mode cannot be resolved for this stage.
pub fn bench_args_qc_post(
    args: &BenchFastqQcPostArgs,
) -> Result<engine_args::BenchFastqQcPostArgs> {
    Ok(engine_args::BenchFastqQcPostArgs {
        sample_id: args.sample_id.clone(),
        r1: args.r1.clone(),
        r2: args.r2.clone(),
        out: args.out.clone(),
        tools: resolve_bench_tools("fastq.report_qc", &args.tools)?,
        explain: args.explain,
        replicates: args.replicates,
        jobs: args.jobs,
        ci_bootstrap: args.ci_bootstrap,
        aggregation_engine: args.aggregation_engine.clone(),
        aggregation_scope: args.aggregation_scope.clone(),
        governed_qc_manifest: args.governed_qc_manifest.clone(),
    })
}

/// # Errors
/// Returns an error if tool mode cannot be resolved for this stage.
pub fn bench_args_umi(args: &BenchFastqUmiArgs) -> Result<engine_args::BenchFastqUmiArgs> {
    Ok(engine_args::BenchFastqUmiArgs {
        sample_id: args.sample_id.clone(),
        r1: args.r1.clone(),
        r2: args.r2.clone(),
        out: args.out.clone(),
        umi_pattern: args.umi_pattern.clone(),
        threads: args.threads,
        tools: resolve_bench_tools("fastq.extract_umis", &args.tools)?,
        explain: args.explain,
        replicates: args.replicates,
        jobs: args.jobs,
        ci_bootstrap: args.ci_bootstrap,
    })
}

/// # Errors
/// Returns an error if tool mode cannot be resolved for this stage.
pub fn bench_args_index_reference(
    args: &BenchFastqIndexReferenceArgs,
) -> Result<engine_args::BenchFastqIndexReferenceArgs> {
    Ok(engine_args::BenchFastqIndexReferenceArgs {
        sample_id: args.sample_id.clone(),
        reference_fasta: args.reference_fasta.clone(),
        out: args.out.clone(),
        tools: resolve_bench_tools("fastq.index_reference", &args.tools)?,
        explain: args.explain,
        replicates: args.replicates,
        jobs: args.jobs,
        ci_bootstrap: args.ci_bootstrap,
        threads: args.threads,
    })
}

/// # Errors
/// Returns an error if tool mode cannot be resolved for this stage.
pub fn bench_args_screen(args: &BenchFastqScreenArgs) -> Result<engine_args::BenchFastqScreenArgs> {
    Ok(engine_args::BenchFastqScreenArgs {
        sample_id: args.sample_id.clone(),
        r1: args.r1.clone(),
        r2: args.r2.clone(),
        out: args.out.clone(),
        database_root: args.database_root.clone(),
        tools: resolve_bench_tools("fastq.screen_taxonomy", &args.tools)?,
        explain: args.explain,
        replicates: args.replicates,
        jobs: args.jobs,
        ci_bootstrap: args.ci_bootstrap,
        threads: args.threads,
    })
}

/// # Errors
/// Returns an error if tool mode cannot be resolved for this stage.
pub fn bench_args_deplete_host(
    args: &BenchFastqDepleteHostArgs,
) -> Result<engine_args::BenchFastqDepleteHostArgs> {
    Ok(engine_args::BenchFastqDepleteHostArgs {
        sample_id: args.sample_id.clone(),
        r1: args.r1.clone(),
        r2: args.r2.clone(),
        reference_index: args.reference_index.clone(),
        out: args.out.clone(),
        threads: args.threads,
        host_identity_threshold: args.host_identity_threshold,
        retain_unmapped_only: args.retain_unmapped_only,
        tools: resolve_bench_tools("fastq.deplete_host", &args.tools)?,
        explain: args.explain,
        replicates: args.replicates,
        jobs: args.jobs,
        ci_bootstrap: args.ci_bootstrap,
    })
}

/// # Errors
/// Returns an error if tool mode cannot be resolved for this stage.
pub fn bench_args_deplete_reference_contaminants(
    args: &BenchFastqDepleteReferenceContaminantsArgs,
) -> Result<engine_args::BenchFastqDepleteReferenceContaminantsArgs> {
    Ok(engine_args::BenchFastqDepleteReferenceContaminantsArgs {
        sample_id: args.sample_id.clone(),
        r1: args.r1.clone(),
        r2: args.r2.clone(),
        reference_index: args.reference_index.clone(),
        out: args.out.clone(),
        threads: args.threads,
        decoy_mode: args.decoy_mode.clone(),
        tools: resolve_bench_tools("fastq.deplete_reference_contaminants", &args.tools)?,
        explain: args.explain,
        replicates: args.replicates,
        jobs: args.jobs,
        ci_bootstrap: args.ci_bootstrap,
    })
}

/// # Errors
/// Returns an error if tool mode cannot be resolved for this stage.
pub fn bench_args_deplete_rrna(
    args: &BenchFastqDepleteRrnaArgs,
) -> Result<engine_args::BenchFastqDepleteRrnaArgs> {
    Ok(engine_args::BenchFastqDepleteRrnaArgs {
        sample_id: args.sample_id.clone(),
        r1: args.r1.clone(),
        r2: args.r2.clone(),
        out: args.out.clone(),
        threads: args.threads,
        rrna_db: args.rrna_db.clone(),
        min_identity: args.min_identity,
        tools: resolve_bench_tools("fastq.deplete_rrna", &args.tools)?,
        explain: args.explain,
        replicates: args.replicates,
        jobs: args.jobs,
        ci_bootstrap: args.ci_bootstrap,
    })
}

/// # Errors
/// Returns an error if tool mode cannot be resolved for this stage.
pub fn bench_args_stats(args: &BenchFastqStatsArgs) -> Result<engine_args::BenchFastqStatsArgs> {
    Ok(engine_args::BenchFastqStatsArgs {
        sample_id: args.sample_id.clone(),
        r1: args.r1.clone(),
        r2: args.r2.clone(),
        out: args.out.clone(),
        threads: args.threads,
        tools: resolve_bench_tools("fastq.profile_reads", &args.tools)?,
        explain: args.explain,
        replicates: args.replicates,
        jobs: args.jobs,
        ci_bootstrap: args.ci_bootstrap,
    })
}

/// # Errors
/// Returns an error if tool mode cannot be resolved for this stage.
pub fn bench_args_profile_overrepresented(
    args: &BenchFastqProfileOverrepresentedArgs,
) -> Result<engine_args::BenchFastqProfileOverrepresentedArgs> {
    Ok(engine_args::BenchFastqProfileOverrepresentedArgs {
        sample_id: args.sample_id.clone(),
        r1: args.r1.clone(),
        r2: args.r2.clone(),
        out: args.out.clone(),
        threads: args.threads,
        top_k: args.top_k,
        tools: resolve_bench_tools("fastq.profile_overrepresented_sequences", &args.tools)?,
        explain: args.explain,
        replicates: args.replicates,
        jobs: args.jobs,
        ci_bootstrap: args.ci_bootstrap,
    })
}

fn resolve_bench_tools(stage: &str, raw_tools: &[String]) -> Result<Vec<String>> {
    let mut normalized = raw_tools
        .iter()
        .map(|tool| tool.trim().to_lowercase())
        .filter(|tool| !tool.is_empty())
        .collect::<Vec<_>>();
    normalized.sort();
    normalized.dedup();

    let mode = if normalized.is_empty() {
        "auto"
    } else if normalized.len() == 1 && normalized[0] == "all" {
        "all"
    } else if normalized.len() == 1 && normalized[0] == "auto" {
        "auto"
    } else {
        if normalized.iter().any(|v| v == "auto" || v == "all") {
            return Err(anyhow!(
                "--tools accepts either `auto`, `all`, or an explicit CSV list"
            ));
        }
        "csv"
    };

    let registry_path = resolve_registry_path()?;
    let all_tools = registry_tools_for_stage(&registry_path, stage, None, "all")?;
    if all_tools.is_empty() {
        return Err(anyhow!("no compatible tools found for stage `{stage}`"));
    }
    let mut selected = match mode {
        "auto" => registry_tools_for_stage(&registry_path, stage, None, "primary")?,
        "all" => all_tools.clone(),
        _ => normalized,
    };
    selected.sort();
    selected.dedup();

    if selected.is_empty() {
        return Err(anyhow!("resolved empty tool set for stage `{stage}`"));
    }
    for tool in &selected {
        if !all_tools.contains(tool) {
            return Err(anyhow!(
                "tool `{tool}` is not compatible with stage `{stage}`"
            ));
        }
    }
    Ok(selected)
}

fn resolve_registry_path() -> Result<PathBuf> {
    let cwd = std::env::current_dir().map_err(|err| anyhow!("resolve cwd: {err}"))?;
    let cwd_registry = bijux_dna_infra::configs_file(&cwd, "ci/registry/tool_registry.toml");
    if cwd_registry.exists() {
        return Ok(cwd_registry);
    }

    let workspace_registry = bijux_dna_infra::configs_file(
        crate::commands::repo_root::resolve_repo_root()?.as_path(),
        "ci/registry/tool_registry.toml",
    );
    if workspace_registry.exists() {
        return Ok(workspace_registry);
    }

    Ok(cwd_registry)
}

fn bench_tools_resolved_implicitly(raw_tools: &[String]) -> bool {
    let mut normalized = raw_tools
        .iter()
        .map(|tool| tool.trim().to_lowercase())
        .filter(|tool| !tool.is_empty())
        .collect::<Vec<_>>();
    normalized.sort();
    normalized.dedup();
    normalized.is_empty()
        || (normalized.len() == 1 && matches!(normalized[0].as_str(), "auto" | "all"))
}

#[must_use]
pub fn bench_args_preprocess(
    args: &BenchFastqPreprocessArgs,
) -> engine_args::BenchFastqPreprocessArgs {
    engine_args::BenchFastqPreprocessArgs {
        sample_id: args.sample_id.clone(),
        profile: args.pipeline_profile.clone(),
        r1: args.r1.clone(),
        r2: args.r2.clone(),
        reference_fasta: args.reference_fasta.clone(),
        out: args.out.clone(),
        strict: args.strict,
        auto: false,
        objective: bijux_dna_api::v1::api::bench::Objective::Balanced,
        bench_corpus: None,
        allow_partial: false,
        dry_run: false,
        replicates: args.replicates,
        jobs: args.jobs,
        ci_bootstrap: args.ci_bootstrap,
        adapter_bank_preset: args.adapter_bank_preset.clone(),
        adapter_bank: args.adapter_bank.clone(),
        adapter_bank_file: args.adapter_bank_file.clone(),
        enable_adapters: args.enable_adapter.clone(),
        disable_adapters: args.disable_adapter.clone(),
        polyx_preset: args.polyx_preset.clone(),
        contaminant_preset: args.contaminant_preset.clone(),
        enable_contaminant_removal: args.enable_contaminant_removal,
        no_qc_post: args.no_qc_post,
        force_merge: args.force_merge,
        enable_correct: args.enable_correct,
        run_all_governed_tools: args.run_all_governed_tools,
        allow_planned: args.allow_planned,
        mode: engine_args::FastqPlannerMode::Shotgun,
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::items_after_test_module)]
mod tests {
    use super::{
        bench_args_filter, bench_args_from_validate, bench_args_normalize_primers,
        bench_args_profile_overrepresented, bench_args_profile_read_lengths,
        bench_args_remove_chimeras, bench_args_remove_duplicates, bench_args_trim,
        bench_args_trim_polyg, bench_args_trim_terminal_damage, resolve_stage_tool, TOOL_CUTADAPT,
        TOOL_FASTP,
    };
    use crate::commands::cli::parse::{
        BenchFastqFilterArgs, BenchFastqNormalizePrimersArgs, BenchFastqProfileOverrepresentedArgs,
        BenchFastqProfileReadLengthsArgs, BenchFastqRemoveChimerasArgs,
        BenchFastqRemoveDuplicatesArgs, BenchFastqTrimArgs, BenchFastqTrimPolygArgs,
        BenchFastqTrimTerminalDamageArgs, CommonArgs, DnaCommand, FastqCommand, FastqValidateArgs,
    };
    use std::path::PathBuf;

    #[test]
    fn fastq_command_routing_uses_canonical_stage_ids() {
        let cases = [
            (
                DnaCommand::Fastq(crate::commands::cli::parse::FastqRootArgs {
                    command: FastqCommand::Filter(crate::commands::cli::parse::FastqFilterArgs {
                        common: CommonArgs::default(),
                        sample_id: None,
                        r1: None,
                        out: None,
                        tools: Vec::new(),
                        max_n: None,
                        low_complexity_threshold: None,
                        kmer_ref: None,
                    }),
                }),
                "fastq.filter_reads",
            ),
            (
                DnaCommand::Fastq(crate::commands::cli::parse::FastqRootArgs {
                    command: FastqCommand::Merge(CommonArgs::default()),
                }),
                "fastq.merge_pairs",
            ),
            (
                DnaCommand::Fastq(crate::commands::cli::parse::FastqRootArgs {
                    command: FastqCommand::Contam(CommonArgs::default()),
                }),
                "fastq.deplete_reference_contaminants",
            ),
            (
                DnaCommand::Fastq(crate::commands::cli::parse::FastqRootArgs {
                    command: FastqCommand::Umi(CommonArgs::default()),
                }),
                "fastq.extract_umis",
            ),
            (
                DnaCommand::Fastq(crate::commands::cli::parse::FastqRootArgs {
                    command: FastqCommand::ErrorCorrect(CommonArgs::default()),
                }),
                "fastq.correct_errors",
            ),
            (
                DnaCommand::Fastq(crate::commands::cli::parse::FastqRootArgs {
                    command: FastqCommand::Qc(CommonArgs::default()),
                }),
                "fastq.report_qc",
            ),
            (
                DnaCommand::Fastq(crate::commands::cli::parse::FastqRootArgs {
                    command: FastqCommand::Align(CommonArgs::default()),
                }),
                "fastq.index_reference",
            ),
        ];

        for (command, expected_stage) in cases {
            let (stage, _tool, _common) = resolve_stage_tool(&command);
            assert_eq!(stage.as_str(), expected_stage);
        }
    }

    #[test]
    fn fastq_qc_command_uses_multiqc_backend() {
        let (stage, tool, _common) = resolve_stage_tool(&DnaCommand::Fastq(
            crate::commands::cli::parse::FastqRootArgs {
                command: FastqCommand::Qc(CommonArgs::default()),
            },
        ));
        assert_eq!(stage.as_str(), "fastq.report_qc");
        assert_eq!(tool.as_str(), "multiqc");
    }

    #[test]
    fn validate_bench_args_preserve_configured_policy_flags() {
        let args = FastqValidateArgs {
            common: CommonArgs::default(),
            sample_id: Some("sample".to_string()),
            r1: Some(PathBuf::from("reads_R1.fastq.gz")),
            r2: Some(PathBuf::from("reads_R2.fastq.gz")),
            out: Some(PathBuf::from("out")),
            tools: vec!["fastqvalidator".to_string()],
            strict: false,
            threads: Some(6),
            validation_mode: Some("report_only".to_string()),
            pair_sync_policy: Some("skip_header_sync".to_string()),
        };

        let bench = bench_args_from_validate(&args).expect("bench args");
        assert_eq!(bench.threads, Some(6));
        assert_eq!(bench.validation_mode.as_deref(), Some("report_only"));
        assert_eq!(bench.pair_sync_policy.as_deref(), Some("skip_header_sync"));
        assert!(!bench.strict);
    }

    #[test]
    fn validate_bench_args_reject_conflicting_strict_report_only_flags() {
        let args = FastqValidateArgs {
            common: CommonArgs::default(),
            sample_id: Some("sample".to_string()),
            r1: Some(PathBuf::from("reads_R1.fastq.gz")),
            r2: Some(PathBuf::from("reads_R2.fastq.gz")),
            out: Some(PathBuf::from("out")),
            tools: vec!["fastqvalidator".to_string()],
            strict: true,
            threads: Some(6),
            validation_mode: Some("report_only".to_string()),
            pair_sync_policy: Some("skip_header_sync".to_string()),
        };

        let error = bench_args_from_validate(&args).expect_err("conflicting flags must fail");
        assert!(error
            .to_string()
            .contains("--strict conflicts with --validation-mode report_only"));
    }

    #[test]
    fn filter_bench_args_preserve_extended_filter_surface() {
        let args = BenchFastqFilterArgs {
            sample_id: "sample".to_string(),
            r1: PathBuf::from("reads_R1.fastq.gz"),
            r2: Some(PathBuf::from("reads_R2.fastq.gz")),
            out: PathBuf::from("out"),
            tools: vec![TOOL_FASTP.to_string()],
            explain: false,
            allow_experimental: false,
            replicates: 2,
            jobs: 3,
            ci_bootstrap: Some(50),
            threads: Some(8),
            max_n: Some(0),
            max_n_fraction: Some(0.05),
            max_n_count: Some(3),
            low_complexity_threshold: Some(20.0),
            entropy_threshold: Some(18.0),
            kmer_ref: Some(PathBuf::from("contaminants.fa")),
            polyx_policy: Some("trim".to_string()),
        };

        let bench = bench_args_filter(&args).expect("bench args");
        assert_eq!(bench.threads, Some(8));
        assert_eq!(bench.max_n_fraction, Some(0.05));
        assert_eq!(bench.max_n_count, Some(3));
        assert_eq!(bench.entropy_threshold, Some(18.0));
        assert_eq!(bench.polyx_policy.as_deref(), Some("trim"));
        assert_eq!(
            bench.kmer_ref.as_deref(),
            Some(PathBuf::from("contaminants.fa").as_path())
        );
    }

    #[test]
    fn remove_chimeras_bench_args_preserve_thread_setting() {
        let args = BenchFastqRemoveChimerasArgs {
            sample_id: "sample".to_string(),
            r1: PathBuf::from("reads.fastq.gz"),
            r2: None,
            out: PathBuf::from("out"),
            tools: vec!["vsearch".to_string()],
            explain: false,
            threads: Some(6),
            allow_experimental: false,
            replicates: 2,
            jobs: 3,
            ci_bootstrap: Some(25),
        };

        let bench = bench_args_remove_chimeras(&args).expect("bench args");
        assert_eq!(bench.threads, Some(6));
    }

    #[test]
    fn profile_overrepresented_bench_args_preserve_thread_setting() {
        let args = BenchFastqProfileOverrepresentedArgs {
            sample_id: "sample".to_string(),
            r1: PathBuf::from("reads_R1.fastq.gz"),
            r2: Some(PathBuf::from("reads_R2.fastq.gz")),
            out: PathBuf::from("out"),
            threads: Some(4),
            top_k: Some(25),
            tools: vec!["fastqc".to_string()],
            explain: false,
            allow_experimental: false,
            replicates: 2,
            jobs: 3,
            ci_bootstrap: Some(25),
        };

        let bench = bench_args_profile_overrepresented(&args).expect("bench args");
        assert_eq!(bench.threads, Some(4));
        assert_eq!(bench.top_k, Some(25));
    }

    #[test]
    fn trim_polyg_bench_args_preserve_thread_setting() {
        let args = BenchFastqTrimPolygArgs {
            sample_id: "sample".to_string(),
            r1: PathBuf::from("reads_R1.fastq.gz"),
            r2: Some(PathBuf::from("reads_R2.fastq.gz")),
            out: PathBuf::from("out"),
            tools: vec![TOOL_FASTP.to_string()],
            explain: false,
            allow_experimental: false,
            replicates: 2,
            jobs: 3,
            ci_bootstrap: Some(25),
            threads: Some(6),
            trim_polyg: Some(true),
            polyx_preset: Some("illumina_twocolor".to_string()),
            min_polyg_run: Some(12),
        };

        let bench = bench_args_trim_polyg(&args).expect("bench args");
        assert_eq!(bench.threads, Some(6));
        assert_eq!(bench.min_polyg_run, Some(12));
    }

    #[test]
    fn trim_terminal_damage_bench_args_preserve_thread_setting() {
        let args = BenchFastqTrimTerminalDamageArgs {
            sample_id: "sample".to_string(),
            r1: PathBuf::from("reads_R1.fastq.gz"),
            r2: Some(PathBuf::from("reads_R2.fastq.gz")),
            out: PathBuf::from("out"),
            tools: vec![TOOL_CUTADAPT.to_string()],
            explain: false,
            allow_experimental: false,
            replicates: 2,
            jobs: 3,
            ci_bootstrap: Some(25),
            threads: Some(5),
            damage_mode: Some("ancient".to_string()),
            execution_policy: Some("explicit_terminal_trim".to_string()),
            trim_5p_bases: Some(2),
            trim_3p_bases: Some(1),
        };

        let bench = bench_args_trim_terminal_damage(&args).expect("bench args");
        assert_eq!(bench.threads, Some(5));
        assert_eq!(
            bench.execution_policy.as_deref(),
            Some("explicit_terminal_trim")
        );
    }

    #[test]
    fn normalize_primers_bench_args_preserve_governed_settings() {
        let args = BenchFastqNormalizePrimersArgs {
            sample_id: "sample".to_string(),
            r1: PathBuf::from("reads_R1.fastq.gz"),
            r2: None,
            out: PathBuf::from("out"),
            tools: vec![TOOL_CUTADAPT.to_string()],
            explain: false,
            allow_experimental: false,
            replicates: 2,
            jobs: 3,
            ci_bootstrap: Some(25),
            primer_set_id: Some("16S_universal_v1".to_string()),
            orientation_policy: Some("normalize_to_reverse_complement".to_string()),
            max_mismatch_rate: Some(0.05),
            min_overlap_bp: Some(14),
            strict_5p_anchor: Some(false),
            allow_iupac_codes: Some(false),
        };

        let bench = bench_args_normalize_primers(&args).expect("bench args");
        assert_eq!(bench.primer_set_id.as_deref(), Some("16S_universal_v1"));
        assert_eq!(
            bench.orientation_policy.as_deref(),
            Some("normalize_to_reverse_complement")
        );
        assert_eq!(bench.max_mismatch_rate, Some(0.05));
        assert_eq!(bench.min_overlap_bp, Some(14));
        assert_eq!(bench.strict_5p_anchor, Some(false));
        assert_eq!(bench.allow_iupac_codes, Some(false));
    }

    #[test]
    fn trim_reads_bench_args_preserve_thread_setting() {
        let args = BenchFastqTrimArgs {
            sample_id: "sample".to_string(),
            r1: PathBuf::from("reads_R1.fastq.gz"),
            r2: Some(PathBuf::from("reads_R2.fastq.gz")),
            out: PathBuf::from("out"),
            tools: vec![TOOL_FASTP.to_string()],
            explain: false,
            allow_experimental: false,
            replicates: 2,
            jobs: 3,
            ci_bootstrap: Some(25),
            threads: Some(8),
            adapter_bank_preset: Some("illumina-default".to_string()),
            adapter_bank: None,
            adapter_bank_file: None,
            enable_adapter: Vec::new(),
            disable_adapter: Vec::new(),
            polyx_preset: Some("illumina_twocolor".to_string()),
            contaminant_preset: None,
            min_length: Some(40),
            quality_cutoff: Some(20),
            n_policy: Some("retain".to_string()),
            adapter_policy: Some("none".to_string()),
            polyx_policy: Some("trim".to_string()),
            contaminant_policy: Some("none".to_string()),
        };

        let bench = bench_args_trim(&args).expect("bench args");
        assert_eq!(bench.threads, Some(8));
        assert_eq!(bench.min_length, Some(40));
    }

    #[test]
    fn profile_read_lengths_bench_args_preserve_thread_setting() {
        let args = BenchFastqProfileReadLengthsArgs {
            sample_id: "sample".to_string(),
            r1: PathBuf::from("reads_R1.fastq.gz"),
            r2: Some(PathBuf::from("reads_R2.fastq.gz")),
            out: PathBuf::from("out"),
            tools: vec!["seqkit_stats".to_string()],
            explain: false,
            allow_experimental: false,
            replicates: 2,
            jobs: 3,
            ci_bootstrap: Some(25),
            threads: Some(4),
            histogram_bins: Some(64),
        };

        let bench = bench_args_profile_read_lengths(&args).expect("bench args");
        assert_eq!(bench.threads, Some(4));
        assert_eq!(bench.histogram_bins, Some(64));
    }

    #[test]
    fn remove_duplicates_bench_args_preserve_thread_setting() {
        let args = BenchFastqRemoveDuplicatesArgs {
            sample_id: "sample".to_string(),
            r1: PathBuf::from("reads_R1.fastq.gz"),
            r2: Some(PathBuf::from("reads_R2.fastq.gz")),
            out: PathBuf::from("out"),
            tools: vec!["clumpify".to_string()],
            explain: false,
            threads: Some(6),
            dedup_mode: Some("exact".to_string()),
            keep_order: Some(true),
            allow_experimental: false,
            replicates: 2,
            jobs: 3,
            ci_bootstrap: Some(25),
        };

        let bench = bench_args_remove_duplicates(&args).expect("bench args");
        assert_eq!(bench.threads, Some(6));
        assert_eq!(bench.keep_order, Some(true));
    }
}

/// # Errors
/// Returns an error if CLI arguments are invalid for benchmarking.
pub fn bench_args_from_trim(args: &FastqTrimArgs) -> Result<engine_args::BenchFastqTrimArgs> {
    Ok(engine_args::BenchFastqTrimArgs {
        sample_id: args
            .sample_id
            .clone()
            .ok_or_else(|| anyhow::anyhow!("sample_id required for benchmark"))?,
        r1: args
            .r1
            .clone()
            .ok_or_else(|| anyhow::anyhow!("r1 required for benchmark"))?,
        r2: args.r2.clone(),
        out: args
            .out
            .clone()
            .ok_or_else(|| anyhow::anyhow!("out required for benchmark"))?,
        tools: args.tools.clone(),
        explain: false,
        replicates: 1,
        jobs: 1,
        ci_bootstrap: None,
        threads: None,
        adapter_bank_preset: args.adapter_bank_preset.clone(),
        adapter_bank: args.adapter_bank.clone(),
        adapter_bank_file: args.adapter_bank_file.clone(),
        enable_adapters: args.enable_adapter.clone(),
        disable_adapters: args.disable_adapter.clone(),
        polyx_preset: args.polyx_preset.clone(),
        contaminant_preset: args.contaminant_preset.clone(),
        min_length: args.min_length,
        quality_cutoff: args.quality_cutoff,
        n_policy: args.n_policy.clone(),
        adapter_policy: args.adapter_policy.clone(),
        polyx_policy: args.polyx_policy.clone(),
        contaminant_policy: args.contaminant_policy.clone(),
    })
}

/// # Errors
/// Returns an error if CLI arguments are invalid for benchmarking.
pub fn bench_args_from_validate(
    args: &FastqValidateArgs,
) -> Result<engine_args::BenchFastqValidateArgs> {
    let (strict, validation_mode) =
        normalize_validate_failure_flags(args.strict, args.validation_mode.as_deref())?;
    Ok(engine_args::BenchFastqValidateArgs {
        sample_id: args
            .sample_id
            .clone()
            .ok_or_else(|| anyhow::anyhow!("sample_id required for benchmark"))?,
        r1: args
            .r1
            .clone()
            .ok_or_else(|| anyhow::anyhow!("r1 required for benchmark"))?,
        r2: args.r2.clone(),
        out: args
            .out
            .clone()
            .ok_or_else(|| anyhow::anyhow!("out required for benchmark"))?,
        tools: args.tools.clone(),
        explain: false,
        strict,
        replicates: 1,
        jobs: 1,
        ci_bootstrap: None,
        threads: args.threads,
        validation_mode,
        pair_sync_policy: args.pair_sync_policy.clone(),
    })
}

fn normalize_validate_failure_flags(
    strict: bool,
    validation_mode: Option<&str>,
) -> Result<(bool, Option<String>)> {
    let Some(validation_mode) = validation_mode else {
        return Ok((strict, None));
    };
    let normalized = validation_mode.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "strict" => Ok((true, Some(normalized))),
        "report_only" => {
            if strict {
                Err(anyhow::anyhow!(
                    "--strict conflicts with --validation-mode report_only for fastq.validate_reads"
                ))
            } else {
                Ok((false, Some(normalized)))
            }
        }
        _ => Ok((strict, Some(validation_mode.to_string()))),
    }
}

/// # Errors
/// Returns an error if CLI arguments are invalid for preprocessing.
pub fn preprocess_args_from_cli(
    args: &FastqPreprocessArgs,
) -> Result<engine_args::BenchFastqPreprocessArgs> {
    let sample_id = args
        .sample_id
        .clone()
        .ok_or_else(|| anyhow::anyhow!("--sample-id is required"))?;
    let r1 = args
        .r1
        .clone()
        .ok_or_else(|| anyhow::anyhow!("--r1 is required"))?;
    let out = args
        .out
        .clone()
        .ok_or_else(|| anyhow::anyhow!("--out is required"))?;
    let mut out_args = engine_args::BenchFastqPreprocessArgs {
        sample_id,
        profile: args.pipeline_profile.clone(),
        r1,
        r2: args.r2.clone(),
        reference_fasta: args.alignment_reference.clone(),
        out,
        strict: args.strict,
        auto: args.auto,
        objective: args.objective.into(),
        bench_corpus: args.bench_corpus.map(Into::into),
        allow_partial: args.allow_partial,
        dry_run: args.common.dry_run,
        replicates: 1,
        jobs: args.jobs,
        ci_bootstrap: None,
        adapter_bank_preset: args.adapter_bank_preset.clone(),
        adapter_bank: args.adapter_bank.clone(),
        adapter_bank_file: args.adapter_bank_file.clone(),
        enable_adapters: args.enable_adapter.clone(),
        disable_adapters: args.disable_adapter.clone(),
        polyx_preset: args.polyx_preset.clone(),
        contaminant_preset: args.contaminant_preset.clone(),
        enable_contaminant_removal: args.enable_contaminant_removal,
        no_qc_post: args.no_qc_post,
        force_merge: args.force_merge,
        enable_correct: args.enable_correct,
        run_all_governed_tools: args.run_all_governed_tools,
        allow_planned: args.common.allow_planned,
        mode: engine_args::FastqPlannerMode::Shotgun,
    };
    if let Some(preset) = args.scientific_preset {
        apply_scientific_preset(preset, &mut out_args);
    }
    Ok(out_args)
}

#[must_use]
pub fn fastq_cross_args_from_cli(
    args: &FastqPreprocessArgs,
) -> bijux_dna_api::v1::api::plan::FastqCrossArgs {
    bijux_dna_api::v1::api::plan::FastqCrossArgs {
        sample_id: args.sample_id.clone(),
        r1: args.r1.clone(),
        r2: args.r2.clone(),
        alignment_bam: args.alignment_bam.clone(),
        alignment_bai: args.alignment_bai.clone(),
        alignment_reference: args.alignment_reference.clone(),
        alignment_rg_policy: args.alignment_rg_policy.clone(),
        alignment_meta: args.alignment_meta.clone(),
    }
}

fn apply_scientific_preset(
    preset: crate::commands::cli::parse::ScientificPresetArg,
    args: &mut engine_args::BenchFastqPreprocessArgs,
) {
    match preset {
        crate::commands::cli::parse::ScientificPresetArg::AncientDna => {
            if args.adapter_bank_preset.is_none() {
                args.adapter_bank_preset = Some("ssdna".to_string());
            }
            args.enable_contaminant_removal = true;
            args.force_merge = false;
        }
        crate::commands::cli::parse::ScientificPresetArg::Amplicon => {
            if args.adapter_bank_preset.is_none() {
                args.adapter_bank_preset = Some("illumina-default".to_string());
            }
            args.force_merge = true;
            args.mode = engine_args::FastqPlannerMode::EdnaAmplicon;
        }
        crate::commands::cli::parse::ScientificPresetArg::Metagenomic => {
            args.enable_contaminant_removal = true;
            args.force_merge = false;
        }
        crate::commands::cli::parse::ScientificPresetArg::WgsStandard => {}
    }
}
