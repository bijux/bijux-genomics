# BIJUX CLI

Canonical DNA command surfaces are:

- `bijux-dna <command> [args...]` for the direct product binary
- `bijux dna <command> [args...]` when routed through the sibling `bijux-cli` host

## Rules

- `bijux-dna` is the canonical direct binary owned by this workspace.
- `bijux dna ...` must remain behaviorally equivalent when DNA is installed through `bijux-cli`.
- Local workspace docs, Cargo examples, and repo scripts should use `bijux-dna ...`.
- Global output flags (for example `--json`) are accepted at the root and apply to app subcommands.

## Command Snapshot

The normalized command tree snapshot for `bijux-dna --help` is stored in:

- `docs/cli/command_snapshot.txt`
