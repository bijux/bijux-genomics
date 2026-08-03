# Docker Entrypoint and Default-Command Exceptions

Purpose: Record tool interfaces whose native help contract cannot use the standard
JSON `ENTRYPOINT` plus `CMD ["--help"]` form.

Policy:
- Preferred contract: JSON `ENTRYPOINT` to tool binary and JSON `CMD` defaulting to `--help` (or `--version`).
- Exception path: tool ID listed here with rationale.

## Exceptions

| tool_id | rationale | owner |
|---|---|---|
| `diamond` | Upstream DIAMOND exposes its command overview through the positional `help` subcommand rather than a `--help` option. | `bijux-genomics` |
| `preseq` | Upstream Preseq prints its version, command list, and usage only when invoked without a subcommand; `--help` is parsed as an unknown subcommand. | `bijux-genomics` |
