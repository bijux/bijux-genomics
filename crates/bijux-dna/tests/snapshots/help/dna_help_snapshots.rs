#![allow(clippy::expect_used)]

use bijux_dna::public_api::cli::Cli;
use clap::CommandFactory;

fn help_for(args: &[&str]) -> String {
    let cmd = Cli::command();
    let full_args = std::iter::once("bijux-dna").chain(args.iter().copied());
    let err = cmd
        .try_get_matches_from(full_args)
        .expect_err("help invocation should return clap help error");
    assert_eq!(err.kind(), clap::error::ErrorKind::DisplayHelp);
    err.to_string()
}

#[test]
fn cli_dna_help_snapshot() {
    let help = help_for(&["--help"]);
    assert!(help.contains("Usage: bijux-dna [OPTIONS] <COMMAND>"));
}

#[test]
fn cli_dna_fastq_help_snapshot() {
    let cmd = Cli::command();
    let err = cmd
        .try_get_matches_from(["bijux-dna", "fastq", "--help"])
        .expect_err("fastq should be removed");
    assert_eq!(err.kind(), clap::error::ErrorKind::InvalidSubcommand);
}

#[test]
fn cli_dna_run_help_examples_use_public_command_prefix() {
    let help = help_for(&["run", "filter", "--help"]);

    assert!(help.contains("bijux-dna run filter"));
    assert!(!help.contains("bijux-dna fastq filter"));
}

#[test]
fn cli_dna_bam_help_snapshot() {
    let help = help_for(&["bam", "--help"]);
    assert!(help.contains("Usage: bijux-dna bam [OPTIONS] <COMMAND>"));
}

#[test]
fn cli_dna_vcf_help_snapshot() {
    let help = help_for(&["vcf", "--help"]);
    assert!(help.contains("Usage: bijux-dna vcf [OPTIONS] <COMMAND>"));
}
