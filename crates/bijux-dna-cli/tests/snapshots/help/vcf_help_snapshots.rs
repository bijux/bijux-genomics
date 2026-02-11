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
fn cli_vcf_help_snapshot() {
    let help = help_for(&["dna", "vcf", "--help"]);
    assert!(help.contains("Usage: bijux dna vcf <COMMAND>"));
    assert!(help.contains("plan"));
    assert!(help.contains("explain"));
    assert!(help.contains("run"));
}

#[test]
fn cli_vcf_run_help_snapshot() {
    let help = help_for(&["dna", "vcf", "run", "--help"]);
    assert!(help.contains("Usage: bijux dna vcf run [OPTIONS] --vcf <VCF> --out <OUT>"));
    assert!(help.contains("--profile"));
    assert!(help.contains("--tool"));
}

#[test]
fn cli_vcf_explain_help_snapshot() {
    let help = help_for(&["dna", "vcf", "explain", "--help"]);
    assert!(help.contains("Usage: bijux dna vcf explain [OPTIONS]"));
    assert!(help.contains("--profile"));
}
