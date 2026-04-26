# bijux-dna-infra Effects

`bijux-dna-infra` is allowed to provide generic filesystem effects only when the caller explicitly
invokes them. It must stay offline by default and must not gain process or network authority.

## Allowed Reads

- Caller-provided files and directories.
- Config-compatible JSON, TOML, and optional YAML payloads.
- Existing files when hashing, copying, removing, locking, or preserving atomic-write permissions.

## Allowed Writes

- Caller-provided output paths through IO helpers.
- Temporary files created beside atomic-write targets.
- Temporary directories through `temp_dir` and `temp_dir_in`.
- Caller-provided log files when `init_logging` is used with the `tracing` feature.

## Allowed Processes

None. The full command inventory is intentionally empty; see `COMMANDS.md`.

## Logging Contract

- Logging setup writes to the caller-provided path and returns a `WorkerGuard`.
- Structured fields such as `event`, `component`, and `step_id` should remain stable when callers
  emit them.
- Logs must not include secrets or PII.

## Forbidden Effects

- Process spawning and shell command execution.
- Network access.
- Domain-specific writes or source-tree mutation in tests.
- Writing generated outputs outside caller-provided paths or repository `artifacts/` during local
  verification.
