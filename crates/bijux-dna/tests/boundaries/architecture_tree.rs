use std::collections::BTreeSet;

#[test]
fn dna_tree_matches_architecture_contract() {
    let root = crate::support::crate_root("bijux-dna")
        .unwrap_or_else(|err| panic!("resolve crate root: {err}"));

    let root_entries = dir_entries(&root);
    let expected_root: BTreeSet<_> = [
        "BOUNDARY.md",
        "Cargo.toml",
        "PUBLIC_API.md",
        "README.md",
        "docs/",
        "src/",
        "tests/",
    ]
    .into_iter()
    .map(str::to_string)
    .collect();
    assert_eq!(
        root_entries, expected_root,
        "dna crate root must stay minimal and intentional"
    );

    let src_entries = dir_entries(&root.join("src"));
    let expected_src: BTreeSet<_> = [
        "bin/",
        "cli_entrypoint.rs",
        "commands/",
        "lib.rs",
        "process_exit.rs",
        "public_api/",
    ]
    .into_iter()
    .map(str::to_string)
    .collect();
    assert_eq!(
        src_entries, expected_src,
        "dna src tree must match the documented CLI layout"
    );

    let command_entries = dir_entries(&root.join("src/commands"));
    let expected_commands: BTreeSet<_> = [
        "bam/",
        "benchmark/",
        "cli/",
        "command_prelude.rs",
        "corpus.rs",
        "ena/",
        "example/",
        "example.rs",
        "fastq/",
        "hpc/",
        "mod.rs",
        "policies.rs",
        "profile_runtime.rs",
        "repo_root.rs",
        "report_inputs.rs",
        "router/",
        "run_plan.rs",
        "status.rs",
        "vcf/",
    ]
    .into_iter()
    .map(str::to_string)
    .collect();
    assert_eq!(
        command_entries, expected_commands,
        "dna commands tree must stay partitioned by enduring concern"
    );

    let router_entries = dir_entries(&root.join("src/commands/router"));
    let expected_router: BTreeSet<_> = [
        "argv.rs",
        "entrypoint.rs",
        "mod.rs",
        "root.rs",
        "root_commands.rs",
        "runtime.rs",
    ]
    .into_iter()
    .map(str::to_string)
    .collect();
    assert_eq!(
        router_entries, expected_router,
        "router tree must stay focused on CLI entry and routing"
    );

    let benchmark_entries = dir_entries(&root.join("src/commands/benchmark"));
    let expected_benchmark: BTreeSet<_> = [
        "config.rs",
        "corpus_fastq/",
        "corpus_fastq.rs",
        "corpus_metadata.rs",
        "fastq_bench.rs",
        "mod.rs",
        "publication/",
        "repo_checks.rs",
        "stage_catalog.rs",
        "suite/",
        "taxonomy_database.rs",
        "workspace/",
        "workspace.rs",
    ]
    .into_iter()
    .map(str::to_string)
    .collect();
    assert_eq!(
        benchmark_entries, expected_benchmark,
        "benchmark tree must keep benchmark-specific workflows together"
    );

    let public_api_entries = dir_entries(&root.join("src/public_api"));
    let expected_public_api: BTreeSet<_> = ["mod.rs"].into_iter().map(str::to_string).collect();
    assert_eq!(
        public_api_entries, expected_public_api,
        "public api tree must stay curated"
    );
}

fn dir_entries(path: &std::path::Path) -> BTreeSet<String> {
    std::fs::read_dir(path)
        .unwrap_or_else(|err| panic!("read {}: {err}", path.display()))
        .map(|entry| entry.unwrap_or_else(|err| panic!("read entry in {}: {err}", path.display())))
        .map(|entry| {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            if path.is_dir() {
                format!("{name}/")
            } else {
                name
            }
        })
        .collect()
}
