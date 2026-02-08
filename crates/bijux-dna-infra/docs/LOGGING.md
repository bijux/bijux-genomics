# LOGGING

## Stable formatting guarantees
- JSON output with deterministic field ordering per `tracing_subscriber` JSON.
- Required fields: `event`, `component`, `step_id` when applicable.
- Span names are short, action-oriented verbs.

## Intentionally flexible
- Additional contextual fields may be added without breaking changes.
- Event message wording may change if structured fields remain stable.

## Never log
- secrets
- PII
