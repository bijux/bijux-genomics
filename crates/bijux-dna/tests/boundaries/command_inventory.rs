use std::collections::BTreeSet;

use bijux_dna::public_api::cli::Cli;
use clap::{Command, CommandFactory};

const REQUIRED_SECTIONS: &[&str] = &[
    "## Stable Operator Commands",
    "### Environment",
    "### Registry",
    "### Corpus",
    "### Status",
    "### FASTQ Run Surface",
    "### Pipeline Profiles",
    "### Analysis And Explanation",
    "### Benchmarking",
    "## Debug And Repository-Control Commands",
    "## Ownership Rules",
    "## Verification",
];

#[test]
fn command_doc_is_complete_cli_inventory() {
    let root = crate::support::crate_root("bijux-dna")
        .unwrap_or_else(|err| panic!("resolve crate root: {err}"));
    let commands_doc = root.join("docs/COMMANDS.md");
    let commands = std::fs::read_to_string(&commands_doc)
        .unwrap_or_else(|err| panic!("read {}: {err}", commands_doc.display()));

    assert!(
        commands.contains("This file is the single source of truth for commands managed by the `bijux-dna` crate."),
        "COMMANDS.md must identify itself as the CLI command SSOT"
    );
    for section in REQUIRED_SECTIONS {
        assert!(commands.contains(section), "COMMANDS.md must include `{section}`");
    }
    let mut cli = without_generated_help(Cli::command());
    cli.build();
    let mut declared = BTreeSet::new();
    collect_leaf_commands(&cli, cli.get_name(), &mut declared);
    assert!(!declared.is_empty(), "the public parser has no declared command leaves");
    assert_eq!(
        resolve_documented_command(&cli, "bijux-dna bench active-scope validate --fast"),
        Ok("bijux-dna benchmark active-scope validate".to_string())
    );
    assert_eq!(
        resolve_documented_command(&cli, "bijux-dna environment export-containers --json"),
        Ok("bijux-dna env export-containers".to_string())
    );
    assert_eq!(
        resolve_documented_command(&cli, "bijux-dna run validate"),
        Ok("bijux-dna run validate-pre".to_string())
    );
    assert!(resolve_documented_command(&cli, "bijux-dna env undeclared-command").is_err());
    assert!(resolve_documented_command(&cli, "bijux-dna env info --undeclared-option").is_err());
    let entries = documented_cli_commands(&commands);
    for required in [
        "bijux-dna env export-containers --json",
        "bijux-dna registry export-containers --json",
        "bijux-dna bench active-scope validate --fast",
    ] {
        assert!(entries.contains(required), "COMMANDS.md must retain option example `{required}`");
    }
    let documented = entries
        .iter()
        .map(|entry| {
            resolve_documented_command(&cli, entry)
                .unwrap_or_else(|error| panic!("invalid documented command `{entry}`: {error}"))
        })
        .collect::<BTreeSet<_>>();
    let missing = declared.difference(&documented).collect::<Vec<_>>();
    let foreign = documented.difference(&declared).collect::<Vec<_>>();
    eprintln!(
        "CLI_COMMAND_TREE_CONTROLS_PASS: declared={} documented={} missing={missing:?} foreign={foreign:?}",
        declared.len(), documented.len()
    );
    assert_eq!(documented, declared, "COMMANDS.md must cover every public parser leaf");
    assert!(
        commands.contains("- `bijux-dna run validate` aliases `bijux-dna run validate-pre`."),
        "COMMANDS.md must document the visible validate alias"
    );
    assert!(
        commands.contains("- `bijux-dna run stats` aliases `bijux-dna run stats-neutral`."),
        "COMMANDS.md must document the visible stats alias"
    );
    assert!(
        commands.contains(
            "CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna --test boundaries --no-default-features"
        ),
        "COMMANDS.md must list the focused command-inventory verification command"
    );
}

// Disable only generated help commands before building the real declaration tree.
// Hidden application commands and explicitly declared commands remain governed.
fn without_generated_help(command: Command) -> Command {
    command.disable_help_subcommand(true).mut_subcommands(without_generated_help)
}

fn collect_leaf_commands(command: &Command, path: &str, leaves: &mut BTreeSet<String>) {
    if command.get_subcommands().next().is_none() {
        assert!(leaves.insert(path.to_string()), "duplicate parser leaf `{path}`");
        return;
    }
    for child in command.get_subcommands() {
        collect_leaf_commands(child, &format!("{path} {}", child.get_name()), leaves);
    }
}

fn resolve_documented_command(cli: &Command, entry: &str) -> Result<String, String> {
    let mut words = entry.split_whitespace().peekable();
    if words.next() != Some(cli.get_name()) {
        return Err("entry must start with the public executable name".to_string());
    }
    let mut command = cli;
    let mut canonical = cli.get_name().to_string();
    while words.peek().is_some_and(|word| !word.starts_with('-')) {
        let name = words.next().ok_or("missing command token")?;
        command = command
            .get_subcommands()
            .find(|child| {
                child.get_name() == name || child.get_all_aliases().any(|alias| alias == name)
            })
            .ok_or_else(|| format!("undeclared subcommand `{name}` below `{canonical}`"))?;
        canonical.push(' ');
        canonical.push_str(command.get_name());
    }
    if command.get_subcommands().next().is_some() {
        return Err(format!("`{canonical}` names a command group rather than a leaf"));
    }
    // Inventory entries name commands and optional flag markers, not complete invocations.
    // Required positional inputs and option values belong to the command's usage documentation.
    for option in words {
        let (flag, value) =
            option.split_once('=').map_or((option, None), |(flag, value)| (flag, Some(value)));
        let argument = command
            .get_arguments()
            .find(|argument| {
                if let Some(long) = flag.strip_prefix("--") {
                    argument.get_long() == Some(long)
                        || argument.get_all_aliases().is_some_and(|aliases| aliases.contains(&long))
                } else if let Some(short) =
                    flag.strip_prefix('-').filter(|short| short.chars().count() == 1)
                {
                    short.chars().next().is_some_and(|short| {
                        argument.get_short() == Some(short)
                            || argument
                                .get_all_short_aliases()
                                .is_some_and(|aliases| aliases.contains(&short))
                    })
                } else {
                    false
                }
            })
            .ok_or_else(|| format!("undeclared option `{option}` for `{canonical}`"))?;
        if value.is_some() && !argument.get_action().takes_values() {
            return Err(format!("flag `{flag}` does not accept an attached value"));
        }
    }
    Ok(canonical)
}

fn documented_cli_commands(commands: &str) -> BTreeSet<String> {
    commands
        .lines()
        .filter_map(|line| line.strip_prefix("- `"))
        .filter_map(|line| line.split_once('`').map(|(command, _)| command.to_string()))
        .filter(|command| command.starts_with("bijux-dna "))
        .collect()
}
