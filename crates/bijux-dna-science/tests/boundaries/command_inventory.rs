use std::fs;
use std::path::Path;

#[test]
fn command_inventory_matches_science_cli_surface() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let commands_doc = read(root.join("docs/COMMANDS.md"));
    let cli_rs = read(root.join("src/cli.rs"));

    for variant in ["Validate", "Build", "Trace", "Closure", "Release"] {
        assert!(cli_rs.contains(variant), "src/cli.rs must define ScienceCommand::{variant}");
    }

    for command in [
        "`validate`",
        "`build`",
        "`trace [--stage <stage_id>] [--tool <tool_id>]`",
        "`closure [--stage <stage_id>] [--tool <tool_id>]`",
        "`release --release-id <release_id>`",
    ] {
        assert!(commands_doc.contains(command), "docs/COMMANDS.md must document command {command}");
    }

    for non_owned in [
        "workflow execution",
        "pipeline planning",
        "stage execution",
        "container launching",
        "runtime replay",
    ] {
        assert!(
            commands_doc.contains(non_owned),
            "docs/COMMANDS.md must document non-owned command surface: {non_owned}"
        );
    }
}

fn read(path: impl AsRef<Path>) -> String {
    let path = path.as_ref();
    fs::read_to_string(path).unwrap_or_else(|err| panic!("read {}: {err}", path.display()))
}
