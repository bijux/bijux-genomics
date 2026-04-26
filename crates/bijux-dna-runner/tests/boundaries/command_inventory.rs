use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

#[test]
fn command_inventory_documents_runner_backend_commands() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let commands_doc = root.join("docs").join("COMMANDS.md");
    let content = fs::read_to_string(&commands_doc)
        .unwrap_or_else(|err| panic!("read {}: {err}", commands_doc.display()));

    assert!(
        content.contains("## Runtime Commands\nNone."),
        "COMMANDS.md must make the runtime command ownership boundary explicit"
    );
    assert!(
        !root.join("src").join("bin").exists(),
        "bijux-dna-runner must not define src/bin CLI entrypoints"
    );
    assert_eq!(
        documented_commands(&content),
        entries([
            "docker run",
            "apptainer exec",
            "execute_observer_command",
            "run_command",
            "run_command_with_context",
            "replay_run",
        ]),
        "COMMANDS.md must document each command family this crate can manage"
    );
}

fn documented_commands(content: &str) -> BTreeSet<String> {
    content
        .split('`')
        .filter(|segment| {
            matches!(
                *segment,
                "docker run"
                    | "apptainer exec"
                    | "execute_observer_command"
                    | "run_command"
                    | "run_command_with_context"
                    | "replay_run"
            )
        })
        .map(str::to_string)
        .collect()
}

fn entries<const N: usize>(items: [&str; N]) -> BTreeSet<String> {
    items.into_iter().map(str::to_string).collect()
}
