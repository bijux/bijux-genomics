fn resolve_profile_alias(id: &str) -> &str {
    match id {
        "fastq-adna" => "fastq-to-fastq__adna__v1",
        "fastq-reference-adna" => "fastq-to-fastq__reference_adna__v1",
        "fastq-default" => "fastq-to-fastq__default__v1",
        "bam-adna" => "bam-to-bam__adna_shotgun__v1",
        "bam-default" => "bam-to-bam__default__v1",
        "vcf-minimal" => "vcf-to-vcf__minimal__v1",
        other => other,
    }
}

fn bench_bam_stage_args_to_api(
    args: &crate::commands::cli::parse::BenchBamStageArgs,
) -> bijux_dna_api::v1::api::bench::BenchBamStageArgs {
    bijux_dna_api::v1::api::bench::BenchBamStageArgs {
        sample_id: args.sample_id.clone(),
        stage: args.stage.stage(),
        bam: args.bam.clone(),
        out: args.out.clone(),
        tools: args.tools.clone(),
        explain: args.explain,
        allow_silver: args.allow_silver,
        allow_experimental: args.allow_experimental,
        replicates: args.replicates,
        jobs: args.jobs,
        dry_run: args.dry_run,
        allow_planned: args.allow_planned,
    }
}

fn bench_bam_pipeline_args_to_api(
    args: &crate::commands::cli::parse::BenchBamPipelineArgs,
) -> bijux_dna_api::v1::api::bench::BenchBamPipelineArgs {
    bijux_dna_api::v1::api::bench::BenchBamPipelineArgs {
        profile: args.profile.clone(),
        sample_id: args.sample_id.clone(),
        bam: args.bam.clone(),
        out: args.out.clone(),
        tools: args.tools.clone(),
        explain: args.explain,
        allow_silver: args.allow_silver,
        allow_experimental: args.allow_experimental,
        replicates: args.replicates,
        jobs: args.jobs,
        dry_run: args.dry_run,
        allow_planned: args.allow_planned,
    }
}
