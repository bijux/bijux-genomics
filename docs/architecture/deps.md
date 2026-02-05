# Dependency Contract

This is the single source of truth for crate dependency direction. Every crate
must follow these rules; CI enforces them.

## Allowed dependency map

- `bijux-core` is the base layer.
- `bijux-domain-*` may depend on `bijux-core` and `bijux-infra` only.
- `bijux-stages-*` may depend on `bijux-domain-*`, `bijux-core`, and `bijux-infra` only.
- `bijux-pipelines` may depend on `bijux-domain-*`, `bijux-stages-*`, and `bijux-core`.
- `bijux-api` may depend on `bijux-core`, `bijux-domain-*`, `bijux-stages-*`,
  `bijux-pipelines`, `bijux-engine`, `bijux-analyze`, and `bijux-infra`.
- `bijux-cli` may depend on `bijux-api` only (no direct domain/stages/pipelines).

## Prohibited edges

- `bijux-domain-*` must not depend on `bijux-stages-*`, `bijux-engine`,
  `bijux-api`, `bijux-cli`, or `bijux-pipelines`.
- `bijux-stages-*` must not depend on `bijux-cli`, `bijux-api`, `bijux-analyze`,
  `bijux-benchmark`, or `bijux-pipelines`.
- `bijux-pipelines` must not depend on `bijux-engine` or `bijux-cli`.
- `bijux-cli` must not depend on `bijux-domain-*`, `bijux-stages-*`,
  `bijux-pipelines`, or `bijux-engine`.

## Rationale

This keeps domain/stage logic pure, makes pipelines the only scenario layer,
and makes `bijux-api` the single orchestration surface.
