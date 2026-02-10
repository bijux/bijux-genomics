## What
Defines scope for domain compilation and validation responsibilities.

## Why
Keeps SSOT ownership explicit: domain authored, configs generated.

## Non-goals
Running tools, planning execution, or benchmarking.

## Contracts
Inputs come from `domain/**`; outputs are generated config files under `configs/`.

## Examples
`cargo run -p bijux-dna-domain-compiler --bin compile_domain_configs -- --domain-dir domain --configs-dir configs`

## Failure modes
Invalid domain schema, missing fields, or incompatible stage/tool mappings.
