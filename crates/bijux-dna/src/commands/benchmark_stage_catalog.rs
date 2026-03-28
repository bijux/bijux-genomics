use anyhow::{anyhow, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CorpusFastqStageCatalogEntry {
    pub(crate) stage_id: &'static str,
    pub(crate) bench_subcommand: &'static str,
    pub(crate) report_dir: &'static str,
    pub(crate) make_target_stem: &'static str,
    pub(crate) strict_resume_report: bool,
}

const CORPUS_FASTQ_STAGE_CATALOG: &[CorpusFastqStageCatalogEntry] = &[
    CorpusFastqStageCatalogEntry {
        stage_id: "fastq.validate_reads",
        bench_subcommand: "validate-reads",
        report_dir: "validate_reads",
        make_target_stem: "validate",
        strict_resume_report: false,
    },
    CorpusFastqStageCatalogEntry {
        stage_id: "fastq.trim_polyg_tails",
        bench_subcommand: "trim-polyg-tails",
        report_dir: "trim_polyg_tails",
        make_target_stem: "trim-polyg",
        strict_resume_report: false,
    },
    CorpusFastqStageCatalogEntry {
        stage_id: "fastq.trim_reads",
        bench_subcommand: "trim-reads",
        report_dir: "trim_reads",
        make_target_stem: "trim-reads",
        strict_resume_report: true,
    },
    CorpusFastqStageCatalogEntry {
        stage_id: "fastq.trim_terminal_damage",
        bench_subcommand: "trim-terminal-damage",
        report_dir: "trim_terminal_damage",
        make_target_stem: "trim-terminal-damage",
        strict_resume_report: false,
    },
    CorpusFastqStageCatalogEntry {
        stage_id: "fastq.detect_adapters",
        bench_subcommand: "detect-adapters",
        report_dir: "detect_adapters",
        make_target_stem: "detect-adapters",
        strict_resume_report: false,
    },
    CorpusFastqStageCatalogEntry {
        stage_id: "fastq.profile_reads",
        bench_subcommand: "profile-reads",
        report_dir: "profile_reads",
        make_target_stem: "profile-reads",
        strict_resume_report: false,
    },
    CorpusFastqStageCatalogEntry {
        stage_id: "fastq.profile_read_lengths",
        bench_subcommand: "profile-read-lengths",
        report_dir: "profile_read_lengths",
        make_target_stem: "profile-read-lengths",
        strict_resume_report: false,
    },
    CorpusFastqStageCatalogEntry {
        stage_id: "fastq.profile_overrepresented_sequences",
        bench_subcommand: "profile-overrepresented-sequences",
        report_dir: "profile_overrepresented_sequences",
        make_target_stem: "profile-overrepresented",
        strict_resume_report: false,
    },
    CorpusFastqStageCatalogEntry {
        stage_id: "fastq.filter_low_complexity",
        bench_subcommand: "filter-low-complexity",
        report_dir: "filter_low_complexity",
        make_target_stem: "filter-low-complexity",
        strict_resume_report: false,
    },
    CorpusFastqStageCatalogEntry {
        stage_id: "fastq.filter_reads",
        bench_subcommand: "filter",
        report_dir: "filter",
        make_target_stem: "filter-reads",
        strict_resume_report: true,
    },
    CorpusFastqStageCatalogEntry {
        stage_id: "fastq.merge_pairs",
        bench_subcommand: "merge",
        report_dir: "merge_pairs",
        make_target_stem: "merge",
        strict_resume_report: false,
    },
    CorpusFastqStageCatalogEntry {
        stage_id: "fastq.report_qc",
        bench_subcommand: "report-qc",
        report_dir: "report_qc",
        make_target_stem: "report-qc",
        strict_resume_report: false,
    },
    CorpusFastqStageCatalogEntry {
        stage_id: "fastq.remove_duplicates",
        bench_subcommand: "remove-duplicates",
        report_dir: "remove_duplicates",
        make_target_stem: "remove-duplicates",
        strict_resume_report: false,
    },
    CorpusFastqStageCatalogEntry {
        stage_id: "fastq.normalize_primers",
        bench_subcommand: "normalize-primers",
        report_dir: "normalize_primers",
        make_target_stem: "normalize-primers",
        strict_resume_report: false,
    },
    CorpusFastqStageCatalogEntry {
        stage_id: "fastq.deplete_rrna",
        bench_subcommand: "deplete-rrna",
        report_dir: "deplete_rrna",
        make_target_stem: "deplete-rrna",
        strict_resume_report: true,
    },
    CorpusFastqStageCatalogEntry {
        stage_id: "fastq.deplete_host",
        bench_subcommand: "deplete-host",
        report_dir: "deplete_host",
        make_target_stem: "deplete-host",
        strict_resume_report: true,
    },
    CorpusFastqStageCatalogEntry {
        stage_id: "fastq.deplete_reference_contaminants",
        bench_subcommand: "deplete-reference-contaminants",
        report_dir: "deplete_reference_contaminants",
        make_target_stem: "deplete-reference-contaminants",
        strict_resume_report: true,
    },
    CorpusFastqStageCatalogEntry {
        stage_id: "fastq.screen_taxonomy",
        bench_subcommand: "screen-taxonomy",
        report_dir: "screen_taxonomy",
        make_target_stem: "screen-taxonomy",
        strict_resume_report: true,
    },
    CorpusFastqStageCatalogEntry {
        stage_id: "fastq.correct_errors",
        bench_subcommand: "correct",
        report_dir: "correct_errors",
        make_target_stem: "correct-errors",
        strict_resume_report: true,
    },
    CorpusFastqStageCatalogEntry {
        stage_id: "fastq.extract_umis",
        bench_subcommand: "umi",
        report_dir: "extract_umis",
        make_target_stem: "extract-umis",
        strict_resume_report: true,
    },
];

pub(crate) fn corpus_fastq_stage_catalog_entry(
    stage_id: &str,
) -> Result<CorpusFastqStageCatalogEntry> {
    CORPUS_FASTQ_STAGE_CATALOG
        .iter()
        .copied()
        .find(|entry| entry.stage_id == stage_id)
        .ok_or_else(|| anyhow!("unsupported corpus benchmark stage `{stage_id}`"))
}

pub(crate) fn corpus_fastq_make_target(stage_id: &str, kind: &str) -> Result<String> {
    let entry = corpus_fastq_stage_catalog_entry(stage_id)?;
    match kind {
        "run" => Ok(format!("_benchmark-{}-corpus-01", entry.make_target_stem)),
        "report" => Ok(format!(
            "_benchmark-{}-corpus-01-report",
            entry.make_target_stem
        )),
        other => Err(anyhow!(
            "unsupported benchmark publication target kind: {other}"
        )),
    }
}
