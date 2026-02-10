## What
Describes the compilation flow from domain YAML into generated TOML configs.

## Why
Prevents drift between authored domain metadata and runtime-consumed registry/config views.

## Non-goals
No runtime execution, no side-effectful orchestration, and no planning decisions.

## Contracts
Compiler must emit deterministic generated files with generated headers and schema-consistent fields.

## Examples
Compile and validate in CI before lint/test gates.

## Failure modes
Generation drift, missing required keys, unknown IDs, or duplicate IDs.
