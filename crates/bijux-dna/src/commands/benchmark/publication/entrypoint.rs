use std::path::Path;

use anyhow::{anyhow, Result};

use super::{
    absolutize, render_corpus_fastq_dossier, write_corpus_fastq_docs_status,
    write_corpus_fastq_dossier_index, write_corpus_fastq_remediation_queue,
    write_corpus_fastq_results_status,
};
use crate::commands::benchmark_repo_checks::{audit_repo_checks, fail_on_repo_check_violations};
use crate::commands::benchmark_workspace::{
    benchmark_publication_contracts, load_benchmark_config, write_workspace_layout_status,
};
use crate::commands::cli::{
    BenchCorpusFastqPublicationStatusArgs, BenchCorpusFastqPublishedDossiersArgs,
    BenchCorpusFastqReportArgs, BenchPublicationTargetsArgs,
};

pub(crate) fn print_benchmark_publication_targets(
    cwd: &Path,
    args: &BenchPublicationTargetsArgs,
) -> Result<()> {
    let contracts = benchmark_publication_contracts(cwd, args.config.as_deref(), &args.corpus_id)?;
    if contracts.is_empty() {
        println!();
        return Ok(());
    }
    let targets = contracts
        .into_iter()
        .map(|contract| {
            corpus_fastq_publication_command(
                &contract.stage_id,
                &args.corpus_id,
                &args.kind,
                args.config.as_deref(),
            )
        })
        .collect::<Result<Vec<_>>>()?;
    println!("{}", targets.join("\n"));
    Ok(())
}

pub(super) fn corpus_fastq_publication_command(
    stage_id: &str,
    corpus_id: &str,
    kind: &str,
    config: Option<&Path>,
) -> Result<String> {
    let mut command = vec!["bijux-dna".to_string(), "bench".to_string()];
    match kind {
        "run" => {
            command.push("corpus-fastq".to_string());
            command.push("--corpus-id".to_string());
            command.push(corpus_id.to_string());
            command.push("--stage".to_string());
            command.push(stage_id.to_string());
        }
        "report" => {
            command.push("corpus-fastq-report".to_string());
            command.push("--stage".to_string());
            command.push(stage_id.to_string());
            command.push("--corpus-id".to_string());
            command.push(corpus_id.to_string());
        }
        other => {
            return Err(anyhow!("unsupported benchmark publication target kind: {other}"));
        }
    }
    if let Some(path) = config {
        command.push("--config".to_string());
        command.push(path.display().to_string());
    }
    Ok(command.join(" "))
}

pub(crate) fn run_corpus_fastq_publication_status(
    cwd: &Path,
    args: &BenchCorpusFastqPublicationStatusArgs,
) -> Result<()> {
    let docs_root = absolutize(cwd, &args.docs_root);
    write_corpus_fastq_dossier_index(cwd, args.config.as_deref(), &docs_root, &args.corpus_id)?;
    write_workspace_layout_status(cwd, args.config.as_deref(), &docs_root)?;
    write_corpus_fastq_results_status(cwd, args.config.as_deref(), &docs_root, &args.corpus_id)?;
    fail_on_repo_check_violations(&audit_repo_checks(cwd)?)?;
    write_corpus_fastq_docs_status(cwd, args.config.as_deref(), &docs_root, &args.corpus_id)?;
    write_corpus_fastq_remediation_queue(cwd, args.config.as_deref(), &docs_root, &args.corpus_id)?;
    Ok(())
}

pub(crate) fn run_corpus_fastq_published_dossiers(
    cwd: &Path,
    args: &BenchCorpusFastqPublishedDossiersArgs,
) -> Result<()> {
    for contract in benchmark_publication_contracts(cwd, args.config.as_deref(), &args.corpus_id)? {
        run_corpus_fastq_report(
            cwd,
            &BenchCorpusFastqReportArgs {
                stage: contract.stage_id,
                corpus_id: args.corpus_id.clone(),
                config: args.config.clone(),
                docs_root: args.docs_root.clone(),
                run_root: args.run_root.clone(),
            },
        )?;
    }
    run_corpus_fastq_publication_status(
        cwd,
        &BenchCorpusFastqPublicationStatusArgs {
            corpus_id: args.corpus_id.clone(),
            config: args.config.clone(),
            docs_root: args.docs_root.clone(),
        },
    )?;
    Ok(())
}

pub(crate) fn run_corpus_fastq_report(cwd: &Path, args: &BenchCorpusFastqReportArgs) -> Result<()> {
    let stage_docs_root = absolutize(cwd, &args.docs_root).join(&args.stage).join(&args.corpus_id);
    let benchmark_config = load_benchmark_config(cwd, args.config.as_deref())?;
    render_corpus_fastq_dossier(
        cwd,
        &benchmark_config,
        args.config.as_deref(),
        &args.corpus_id,
        &args.stage,
        args.run_root.as_deref(),
        &stage_docs_root,
    )
}
