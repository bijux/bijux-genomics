# bijux-dna-environment Public API

## Public Modules

- `build`
- `public_api`
- `resolve`
- `runtime_spec`

## Major Export Groups

- `build`: `DockerToolSpec`, `EnvironmentBuilder`, `default_docker_tools`, and
  `extract_version_from_dockerfile`.
- `resolve`: environment errors, image and platform models, catalog loading, image resolution,
  cache helpers, local command probes, shell capture, smoke helpers, and reference registration.
- `runtime_spec`: pure compatibility checks between a platform and selected runtime.
- `public_api::api`: facade re-export for consumers that prefer one stable import path.

## Stability Rules

- `src/lib.rs` is the public module source of truth.
- New public modules require this file and `tests/public_api_docs.rs` to change together.
- New process-running APIs require `COMMANDS.md` and command boundary tests.
- Public model field changes must update `CONTRACTS.md` and schema tests.

## Stability Tiers

- Stable: the Public Modules and Major Export Groups documented in this file.
- Experimental: new runtime/build adapters are experimental until they are listed here and covered by the corresponding docs/tests.
- Internal: module-local helpers and any item not exported through the documented public modules or facade.
