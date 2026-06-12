#![allow(clippy::expect_used)]

use std::collections::BTreeSet;
use std::path::Path;

use bijux_dna_domain_fastq::STAGES;

#[test]
fn command_inventory_documents_fastq_stage_commands_and_synthetic_steps() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let content =
        std::fs::read_to_string(root.join("docs/COMMANDS.md")).expect("read docs/COMMANDS.md");

    assert!(
        content.contains("## Runtime Commands\nNone."),
        "FASTQ planner must document that it exposes no runtime commands"
    );
    for forbidden in [
        "No Cargo binary targets or `src/bin` command modules.",
        "No CLI parser ownership.",
        "No process spawning or runtime command execution.",
    ] {
        assert!(content.contains(forbidden), "COMMANDS.md must document `{forbidden}`");
    }
    assert!(
        !root.join("src/bin").exists(),
        "FASTQ planner must not grow Cargo binary command entrypoints"
    );

    let documented = documented_ids_with_prefix(&content, "fastq.");
    let expected = STAGES
        .iter()
        .map(|stage| stage.as_str().to_string())
        .chain(["fastq.preprocess".to_string()])
        .collect::<BTreeSet<_>>();
    assert_eq!(
        documented, expected,
        "COMMANDS.md must document every FASTQ stage command and planner-local FASTQ step"
    );

    let documented_synthetic = documented_ids_with_prefix(&content, "benchmark.")
        .into_iter()
        .chain(documented_ids_with_prefix(&content, "report."))
        .collect::<BTreeSet<_>>();
    assert_eq!(
        documented_synthetic,
        entries([
            "benchmark.compare_stage_tools",
            "benchmark.select_stage_tool",
            "report.aggregate",
        ]),
        "COMMANDS.md must document every non-FASTQ planner-local graph step"
    );
}

fn documented_ids_with_prefix(content: &str, prefix: &str) -> BTreeSet<String> {
    content.split('`').filter(|segment| segment.starts_with(prefix)).map(str::to_string).collect()
}

fn entries<const N: usize>(items: [&str; N]) -> BTreeSet<String> {
    items.into_iter().map(str::to_string).collect()
}
