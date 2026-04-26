# Effects

## Allowed Effects
- Read CLI arguments, current working directory, explicit environment variables, and repository
  configuration.
- Read governed inputs required by the selected command, such as registry/config/domain files.
- Write declared command outputs, including dry-run manifests, reports, status files, and
  observability log packs.
- Print deterministic terminal or JSON output.
- Contact remote services only from commands whose purpose explicitly requires it, such as ENA
  materialization or fetch workflows.

## Forbidden Effects
- No direct process spawning for tools or containers.
- No undeclared network access or background fetches.
- No writes outside paths requested by command flags or documented command defaults.
- No mutation of source, registry, or generated config files unless the command explicitly owns that
  maintenance workflow.
- No direct stage execution ownership in CLI adapters.

## Determinism
For the same inputs and repository state, CLI-visible output must be stable. If a command needs
time, host, or runtime data, it must either normalize it at the boundary or mark the output as
runtime evidence rather than a stable contract.

## Operator Errors
Operator errors are categorized in `src/process_exit.rs` and rendered without leaking hidden
implementation state. Contract, parse, tool, and infrastructure failures must stay distinguishable.

## Enforcement
- `tests/boundaries/guardrails/no_process_spawn.rs`
- `tests/boundaries/guardrails/deps.rs`
- `tests/contracts/cli_behavior.rs`
- Repository policy tests for ad-hoc filesystem writes and command spawning
