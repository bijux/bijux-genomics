use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

#[test]
fn command_inventory_documents_no_runtime_commands() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let commands_doc = root.join("docs").join("COMMANDS.md");
    let content = fs::read_to_string(&commands_doc)
        .unwrap_or_else(|err| panic!("read {}: {err}", commands_doc.display()));

    assert!(
        content.contains("## Runtime Commands\nNone."),
        "COMMANDS.md must make runtime command ownership explicit"
    );
    assert!(
        content.contains("## Managed Command Families\nNone."),
        "COMMANDS.md must make the managed command set empty"
    );
    assert!(
        !root.join("src").join("bin").exists(),
        "bijux-dna-runtime must not define src/bin command entrypoints"
    );
    assert_eq!(
        documented_entrypoints(&content),
        entries([
            "build_telemetry_adapter",
            "create_run_layout",
            "prepare_tool_run_dirs",
            "write_canonical_json",
            "write_manifest",
            "write_run_manifest",
        ]),
        "COMMANDS.md must list runtime entrypoints without treating them as shell commands"
    );
}

fn documented_entrypoints(content: &str) -> BTreeSet<String> {
    content
        .split('`')
        .filter(|segment| {
            matches!(
                *segment,
                "build_telemetry_adapter"
                    | "create_run_layout"
                    | "prepare_tool_run_dirs"
                    | "write_canonical_json"
                    | "write_manifest"
                    | "write_run_manifest"
            )
        })
        .map(str::to_string)
        .collect()
}

fn entries<const N: usize>(items: [&str; N]) -> BTreeSet<String> {
    items.into_iter().map(str::to_string).collect()
}
