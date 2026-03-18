# CLI_CONVENTIONS

## Determinism
- Output is deterministic for identical inputs.
- Secrets are redacted.
- Error messages are concise and actionable.

## CLI output stability
- JSON outputs are stable and ordered; schema changes require compatibility review.
- Human-readable text is allowed to change wording, but must preserve meaning and exit codes.
