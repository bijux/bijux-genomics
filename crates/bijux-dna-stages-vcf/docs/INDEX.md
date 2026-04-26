# Documentation Index

## Core

- [ARCHITECTURE.md](ARCHITECTURE.md): source layout and module responsibilities.
- [BOUNDARY.md](BOUNDARY.md): ownership and forbidden surfaces.
- [CHANGE_RULES.md](CHANGE_RULES.md): rules for changing this crate.
- [COMMANDS.md](COMMANDS.md): SSOT for managed operations.
- [DEPENDENCIES.md](DEPENDENCIES.md): dependency graph contract.
- [EFFECTS.md](EFFECTS.md): allowed and forbidden production effects.
- [PUBLIC_API.md](PUBLIC_API.md): public module and export contract.
- [STAGE_CONTRACTS.md](STAGE_CONTRACTS.md): VCF stage coverage and artifact duties.
- [TESTS.md](TESTS.md): local verification commands.

## Documentation Shape

This crate keeps only this root `README.md` outside `docs/`. The `docs/`
directory is capped at 10 Markdown files so command, boundary, dependency,
stage, and verification contracts stay discoverable.
