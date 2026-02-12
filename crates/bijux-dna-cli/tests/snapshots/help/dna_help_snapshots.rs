use bijux_dna::commands::cli::Cli;
use clap::CommandFactory;

fn help_for(args: &[&str]) -> String {
    let cmd = Cli::command();
    let full_args = std::iter::once("bijux").chain(args.iter().copied());
    let err = cmd
        .try_get_matches_from(full_args)
        .expect_err("help invocation should return clap help error");
    assert_eq!(err.kind(), clap::error::ErrorKind::DisplayHelp);
    err.to_string()
}

#[test]
fn cli_dna_help_snapshot() {
    let help = help_for(&["dna", "--help"]);
    assert!(help.contains("Usage: bijux dna [OPTIONS] <COMMAND>"));
}

#[test]
fn cli_dna_fastq_help_snapshot() {
    let help = help_for(&["dna", "fastq", "--help"]);
    assert!(help.contains("Usage: bijux dna fastq [OPTIONS] <COMMAND>"));
}

#[test]
fn cli_dna_bam_help_snapshot() {
    let help = help_for(&["dna", "bam", "--help"]);
    assert!(help.contains("Usage: bijux dna bam [OPTIONS] <COMMAND>"));
}

#[test]
fn cli_dna_vcf_help_snapshot() {
    let help = help_for(&["dna", "vcf", "--help"]);
    assert!(help.contains("Usage: bijux dna vcf [OPTIONS] <COMMAND>"));
}
