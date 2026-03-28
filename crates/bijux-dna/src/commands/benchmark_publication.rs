use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::{anyhow, Context, Result};

use crate::commands::benchmark_workspace::{
    benchmark_config_path, load_benchmark_publication_config, BENCHMARK_CONFIG_ENV,
};
use crate::commands::cli::{
    BenchCorpusFastqPublicationStatusArgs, BenchCorpusFastqPublishedDossiersArgs,
    BenchCorpusFastqReportArgs, BenchPublicationTargetsArgs,
};

pub(crate) fn print_benchmark_publication_targets(
    cwd: &Path,
    args: &BenchPublicationTargetsArgs,
) -> Result<()> {
    let publication = load_benchmark_publication_config(cwd, args.config.as_deref())?;
    let Some(corpus_01) = publication.corpus_01 else {
        println!();
        return Ok(());
    };
    let targets = corpus_01
        .contracts
        .into_iter()
        .map(|contract| benchmark_make_target(&contract.stage_id, &args.kind))
        .collect::<Vec<_>>();
    println!("{}", targets.join(" "));
    Ok(())
}

pub(crate) fn run_corpus_fastq_publication_status(
    cwd: &Path,
    args: &BenchCorpusFastqPublicationStatusArgs,
) -> Result<()> {
    let config_path = benchmark_config_path(cwd, args.config.as_deref());
    let docs_root = absolutize(cwd, &args.docs_root);
    for spec in publication_status_steps(cwd, &docs_root) {
        run_subprocess(cwd, &config_path, &spec)?;
    }
    Ok(())
}

pub(crate) fn run_corpus_fastq_published_dossiers(
    cwd: &Path,
    args: &BenchCorpusFastqPublishedDossiersArgs,
) -> Result<()> {
    let publication = load_benchmark_publication_config(cwd, args.config.as_deref())?;
    if let Some(corpus_01) = publication.corpus_01 {
        for contract in corpus_01.contracts {
            run_corpus_fastq_report(
                cwd,
                &BenchCorpusFastqReportArgs {
                    stage: contract.stage_id,
                    config: args.config.clone(),
                    docs_root: args.docs_root.clone(),
                    run_root: args.run_root.clone(),
                },
            )?;
        }
    }
    run_corpus_fastq_publication_status(
        cwd,
        &BenchCorpusFastqPublicationStatusArgs {
            config: args.config.clone(),
            docs_root: args.docs_root.clone(),
        },
    )?;
    Ok(())
}

pub(crate) fn run_corpus_fastq_report(cwd: &Path, args: &BenchCorpusFastqReportArgs) -> Result<()> {
    let config_path = benchmark_config_path(cwd, args.config.as_deref());
    let stage_docs_root = absolutize(cwd, &args.docs_root)
        .join(&args.stage)
        .join("corpus-01");
    let report_spec = corpus_fastq_stage_render_step(
        cwd,
        &args.stage,
        &stage_docs_root,
        args.run_root.as_deref(),
    )?;
    run_subprocess(cwd, &config_path, &report_spec)?;
    let briefing_spec = corpus_fastq_stage_briefing_step(cwd, &args.stage, &stage_docs_root)?;
    run_subprocess(cwd, &config_path, &briefing_spec)?;
    Ok(())
}

fn benchmark_make_target(stage_id: &str, kind: &str) -> String {
    let stage_suffix = match stage_id {
        "fastq.validate_reads" => "validate",
        "fastq.detect_adapters" => "detect-adapters",
        "fastq.profile_reads" => "profile-reads",
        "fastq.profile_read_lengths" => "profile-read-lengths",
        "fastq.profile_overrepresented_sequences" => "profile-overrepresented",
        "fastq.normalize_primers" => "normalize-primers",
        "fastq.trim_polyg_tails" => "trim-polyg",
        "fastq.trim_reads" => "trim-reads",
        "fastq.filter_reads" => "filter-reads",
        "fastq.filter_low_complexity" => "filter-low-complexity",
        "fastq.deplete_rrna" => "deplete-rrna",
        "fastq.merge_pairs" => "merge",
        "fastq.remove_duplicates" => "remove-duplicates",
        "fastq.deplete_host" => "deplete-host",
        "fastq.deplete_reference_contaminants" => "deplete-reference-contaminants",
        "fastq.correct_errors" => "correct-errors",
        "fastq.extract_umis" => "extract-umis",
        "fastq.screen_taxonomy" => "screen-taxonomy",
        "fastq.trim_terminal_damage" => "trim-terminal-damage",
        "fastq.report_qc" => "report-qc",
        other => panic!("unsupported corpus benchmark publication stage: {other}"),
    };
    match kind {
        "run" => format!("_benchmark-{stage_suffix}-corpus-01"),
        "report" => format!("_benchmark-{stage_suffix}-corpus-01-report"),
        other => panic!("unsupported benchmark publication target kind: {other}"),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SubprocessSpec {
    program: &'static str,
    args: Vec<String>,
}

fn publication_status_steps(repo_root: &Path, docs_root: &Path) -> Vec<SubprocessSpec> {
    let repo_root_string = repo_root.display().to_string();
    let docs_root_string = docs_root.display().to_string();
    vec![
        SubprocessSpec {
            program: "python3",
            args: vec![
                repo_root
                    .join("makes/bin/benchmark_tooling_repo_checks.py")
                    .display()
                    .to_string(),
                "--repo-root".to_string(),
                repo_root_string.clone(),
            ],
        },
        SubprocessSpec {
            program: "python3",
            args: vec![
                repo_root
                    .join("makes/bin/audit_corpus_01_fastq_benchmark_docs.py")
                    .display()
                    .to_string(),
                "--repo-root".to_string(),
                repo_root_string.clone(),
                "--docs-root".to_string(),
                docs_root_string.clone(),
                "--json-out".to_string(),
                docs_root
                    .join("corpus-01-status.json")
                    .display()
                    .to_string(),
                "--markdown-out".to_string(),
                docs_root.join("corpus-01-status.md").display().to_string(),
            ],
        },
        SubprocessSpec {
            program: "python3",
            args: vec![
                repo_root
                    .join("makes/bin/build_corpus_01_benchmark_dossier_index.py")
                    .display()
                    .to_string(),
                "--docs-root".to_string(),
                docs_root_string.clone(),
                "--json-out".to_string(),
                docs_root
                    .join("corpus-01-dossier-index.json")
                    .display()
                    .to_string(),
                "--markdown-out".to_string(),
                docs_root
                    .join("corpus-01-dossier-index.md")
                    .display()
                    .to_string(),
            ],
        },
        SubprocessSpec {
            program: "python3",
            args: vec![
                repo_root
                    .join("makes/bin/audit_benchmark_workspace_layout.py")
                    .display()
                    .to_string(),
                "--json-out".to_string(),
                docs_root
                    .join("workspace-layout-status.json")
                    .display()
                    .to_string(),
                "--markdown-out".to_string(),
                docs_root
                    .join("workspace-layout-status.md")
                    .display()
                    .to_string(),
            ],
        },
        SubprocessSpec {
            program: "python3",
            args: vec![
                repo_root
                    .join("makes/bin/audit_published_corpus_01_fastq_results.py")
                    .display()
                    .to_string(),
                "--repo-root".to_string(),
                repo_root_string,
                "--json-out".to_string(),
                docs_root
                    .join("corpus-01-results-status.json")
                    .display()
                    .to_string(),
                "--markdown-out".to_string(),
                docs_root
                    .join("corpus-01-results-status.md")
                    .display()
                    .to_string(),
            ],
        },
        SubprocessSpec {
            program: "python3",
            args: vec![
                repo_root
                    .join("makes/bin/build_corpus_01_benchmark_remediation_queue.py")
                    .display()
                    .to_string(),
                "--status-json".to_string(),
                docs_root
                    .join("corpus-01-status.json")
                    .display()
                    .to_string(),
                "--results-json".to_string(),
                docs_root
                    .join("corpus-01-results-status.json")
                    .display()
                    .to_string(),
                "--findings-json".to_string(),
                docs_root
                    .join("corpus-01-publication-findings.json")
                    .display()
                    .to_string(),
                "--dossier-index-json".to_string(),
                docs_root
                    .join("corpus-01-dossier-index.json")
                    .display()
                    .to_string(),
                "--json-out".to_string(),
                docs_root
                    .join("corpus-01-remediation-queue.json")
                    .display()
                    .to_string(),
                "--markdown-out".to_string(),
                docs_root
                    .join("corpus-01-remediation-queue.md")
                    .display()
                    .to_string(),
            ],
        },
    ]
}

fn corpus_fastq_stage_render_step(
    repo_root: &Path,
    stage_id: &str,
    stage_docs_root: &Path,
    run_root: Option<&Path>,
) -> Result<SubprocessSpec> {
    let script_path = corpus_fastq_script_path(repo_root, "report");
    if !script_path.is_file() {
        return Err(anyhow!(
            "missing corpus benchmark render script for {stage_id}: {}",
            script_path.display()
        ));
    }
    let mut args = vec![
        script_path.display().to_string(),
        "--stage".to_string(),
        stage_id.to_string(),
        "--repo-root".to_string(),
        repo_root.display().to_string(),
        "--docs-root".to_string(),
        stage_docs_root.display().to_string(),
    ];
    if let Some(run_root) = run_root {
        args.push("--run-root".to_string());
        args.push(absolutize(repo_root, run_root).display().to_string());
    }
    Ok(SubprocessSpec {
        program: "python3",
        args,
    })
}

fn corpus_fastq_stage_briefing_step(
    repo_root: &Path,
    stage_id: &str,
    stage_docs_root: &Path,
) -> Result<SubprocessSpec> {
    let script_path = corpus_fastq_script_path(repo_root, "briefing");
    if !script_path.is_file() {
        return Err(anyhow!(
            "missing corpus benchmark briefing script for {stage_id}: {}",
            script_path.display()
        ));
    }
    Ok(SubprocessSpec {
        program: "python3",
        args: vec![
            script_path.display().to_string(),
            "--stage".to_string(),
            stage_id.to_string(),
            "--docs-root".to_string(),
            stage_docs_root.display().to_string(),
        ],
    })
}

fn corpus_fastq_script_path(repo_root: &Path, kind: &str) -> PathBuf {
    repo_root.join(format!("makes/bin/render_corpus_01_fastq_{kind}.py"))
}

fn run_subprocess(repo_root: &Path, config_path: &Path, spec: &SubprocessSpec) -> Result<()> {
    let status = Command::new(spec.program)
        .args(&spec.args)
        .current_dir(repo_root)
        .env(BENCHMARK_CONFIG_ENV, config_path)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .with_context(|| format!("run {}", format_command(spec)))?;
    if status.success() {
        return Ok(());
    }
    Err(anyhow!(
        "{} exited with {}",
        format_command(spec),
        status
            .code()
            .map(|code| code.to_string())
            .unwrap_or_else(|| "signal".to_string())
    ))
}

fn format_command(spec: &SubprocessSpec) -> String {
    std::iter::once(spec.program.to_string())
        .chain(spec.args.iter().cloned())
        .collect::<Vec<_>>()
        .join(" ")
}

fn absolutize(cwd: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        cwd.join(path)
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    #[test]
    fn publication_target_maps_profile_overrepresented_stage() {
        assert_eq!(
            super::benchmark_make_target("fastq.profile_overrepresented_sequences", "report"),
            "_benchmark-profile-overrepresented-corpus-01-report"
        );
    }

    #[test]
    fn publication_target_maps_merge_pairs_stage() {
        assert_eq!(
            super::benchmark_make_target("fastq.merge_pairs", "run"),
            "_benchmark-merge-corpus-01"
        );
    }

    #[test]
    fn publication_target_maps_filter_reads_stage() {
        assert_eq!(
            super::benchmark_make_target("fastq.filter_reads", "report"),
            "_benchmark-filter-reads-corpus-01-report"
        );
    }

    #[test]
    fn corpus_fastq_report_script_path_matches_stage_contract() {
        assert_eq!(
            super::corpus_fastq_script_path(Path::new("/repo"), "report"),
            Path::new("/repo/makes/bin/render_corpus_01_fastq_report.py")
        );
    }

    #[test]
    fn corpus_fastq_report_docs_root_tracks_stage_contract() {
        let docs_root = super::absolutize(Path::new("/repo"), Path::new("docs/benchmark"))
            .join("fastq.validate_reads")
            .join("corpus-01");
        assert_eq!(
            docs_root,
            Path::new("/repo/docs/benchmark/fastq.validate_reads/corpus-01")
        );
    }

    #[test]
    fn corpus_fastq_render_step_passes_stage_to_dispatcher() {
        let step = super::corpus_fastq_stage_render_step(
            Path::new("/repo"),
            "fastq.validate_reads",
            Path::new("/repo/docs/benchmark/fastq.validate_reads/corpus-01"),
            None,
        );
        let err = step.expect_err("dispatcher should not exist in unit test cwd");
        assert!(err.to_string().contains("render_corpus_01_fastq_report.py"));
    }

    #[test]
    fn publication_status_steps_write_to_docs_root_outputs() {
        let steps =
            super::publication_status_steps(Path::new("/repo"), Path::new("/repo/docs/benchmark"));
        let last = steps.last().expect("status steps");
        assert_eq!(last.program, "python3");
        assert!(last
            .args
            .contains(&"/repo/docs/benchmark/corpus-01-remediation-queue.json".to_string()));
        assert!(last
            .args
            .contains(&"/repo/docs/benchmark/corpus-01-results-status.json".to_string()));
    }
}
