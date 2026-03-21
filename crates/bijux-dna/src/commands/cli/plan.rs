use anyhow::{anyhow, Result};
use bijux_dna_api::v1::api::bench::fastq_args as engine_args;
use bijux_dna_api::v1::api::run::{StageId, ToolId};

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

#[must_use]
pub fn resolve_stage_tool(command: &DnaCommand) -> (StageId, ToolId, CommonArgs) {
    match command {
        DnaCommand::Fastq { command } => match command {
            FastqCommand::ListStages
            | FastqCommand::Stages
            | FastqCommand::Doctor
            | FastqCommand::ListTools { .. }
            | FastqCommand::Explain { .. }
            | FastqCommand::Compare(_)
            | FastqCommand::Run(_) => (
                StageId::from_static("fastq.trim_reads"),
                ToolId::from_static("fastp"),
                CommonArgs::default(),
            ),
            FastqCommand::Trim(args) => (
                StageId::from_static("fastq.trim_reads"),
                ToolId::from_static("fastp"),
                args.common.clone(),
            ),
            FastqCommand::ValidateReads(args) => (
                StageId::from_static("fastq.validate_reads"),
                ToolId::from_static("fastqvalidator"),
                args.common.clone(),
            ),
            FastqCommand::ProfileReads(common) => (
                StageId::from_static("fastq.profile_reads"),
                ToolId::from_static("seqkit_stats"),
                common.clone(),
            ),
            FastqCommand::Filter(args) => (
                StageId::from_static("fastq.filter_reads"),
                ToolId::from_static("fastp"),
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
                ToolId::from_static("multiqc"),
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
        DnaCommand::Bam { command } => match command {
            BamCommand::ListStages | BamCommand::Explain { .. } => (
                StageId::from_static("bam.validate"),
                ToolId::from_static("samtools"),
                CommonArgs::default(),
            ),
            BamCommand::Run(args) => (
                StageId::new(args.stage.stage().as_str()),
                ToolId::new(args.tool.clone().unwrap_or_else(|| "samtools".to_string())),
                CommonArgs::default(),
            ),
        },
        DnaCommand::Vcf { command } => match command {
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
            ToolId::from_static("fastp"),
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
        damage_mode: args.damage_mode.clone(),
        trim_5p_bases: args.trim_5p_bases,
        trim_3p_bases: args.trim_3p_bases,
    })
}

/// # Errors
/// Returns an error if tool mode cannot be resolved for this stage.
pub fn bench_args_validate(
    args: &BenchFastqValidateArgs,
) -> Result<engine_args::BenchFastqValidateArgs> {
    Ok(engine_args::BenchFastqValidateArgs {
        sample_id: args.sample_id.clone(),
        r1: args.r1.clone(),
        r2: args.r2.clone(),
        out: args.out.clone(),
        tools: resolve_bench_tools("fastq.validate_reads", &args.tools)?,
        explain: args.explain,
        strict: args.strict,
        q_cutoff: args.q_cutoff,
        replicates: args.replicates,
        jobs: args.jobs,
        ci_bootstrap: args.ci_bootstrap,
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
        max_n: args.max_n,
        low_complexity_threshold: args.low_complexity_threshold,
        kmer_ref: args.kmer_ref.clone(),
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
        tools: resolve_bench_tools("fastq.screen_taxonomy", &args.tools)?,
        explain: args.explain,
        replicates: args.replicates,
        jobs: args.jobs,
        ci_bootstrap: args.ci_bootstrap,
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

    let cwd = std::env::current_dir().map_err(|err| anyhow!("resolve cwd: {err}"))?;
    let registry_path = bijux_dna_infra::configs_file(&cwd, "ci/registry/tool_registry.toml");
    let all_tools = registry_tools_for_stage(&registry_path, stage, "all")?;
    if all_tools.is_empty() {
        return Err(anyhow!("no compatible tools found for stage `{stage}`"));
    }
    let mut selected = match mode {
        "auto" => registry_tools_for_stage(&registry_path, stage, "primary")?,
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
mod tests {
    use super::resolve_stage_tool;
    use crate::commands::cli::parse::{CommonArgs, DnaCommand, FastqCommand};

    #[test]
    fn fastq_command_routing_uses_canonical_stage_ids() {
        let cases = [
            (
                DnaCommand::Fastq {
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
                },
                "fastq.filter_reads",
            ),
            (
                DnaCommand::Fastq {
                    command: FastqCommand::Merge(CommonArgs::default()),
                },
                "fastq.merge_pairs",
            ),
            (
                DnaCommand::Fastq {
                    command: FastqCommand::Contam(CommonArgs::default()),
                },
                "fastq.deplete_reference_contaminants",
            ),
            (
                DnaCommand::Fastq {
                    command: FastqCommand::Umi(CommonArgs::default()),
                },
                "fastq.extract_umis",
            ),
            (
                DnaCommand::Fastq {
                    command: FastqCommand::ErrorCorrect(CommonArgs::default()),
                },
                "fastq.correct_errors",
            ),
            (
                DnaCommand::Fastq {
                    command: FastqCommand::Qc(CommonArgs::default()),
                },
                "fastq.report_qc",
            ),
            (
                DnaCommand::Fastq {
                    command: FastqCommand::Align(CommonArgs::default()),
                },
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
        let (stage, tool, _common) = resolve_stage_tool(&DnaCommand::Fastq {
            command: FastqCommand::Qc(CommonArgs::default()),
        });
        assert_eq!(stage.as_str(), "fastq.report_qc");
        assert_eq!(tool.as_str(), "multiqc");
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
        strict: args.strict,
        q_cutoff: args.q_cutoff,
        replicates: 1,
        jobs: 1,
        ci_bootstrap: None,
    })
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
