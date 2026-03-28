use anyhow::Result;

use crate::commands::benchmark_workspace::load_benchmark_publication_config;
use crate::commands::cli::BenchPublicationTargetsArgs;

pub(crate) fn print_benchmark_publication_targets(
    cwd: &std::path::Path,
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

#[cfg(test)]
mod tests {
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
}
