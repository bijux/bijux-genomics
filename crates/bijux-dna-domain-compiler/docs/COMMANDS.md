# bijux-dna-domain-compiler Commands

`bijux-dna-domain-compiler` owns the compile, validation, release-bundle, and registry-query
command surface for generated CI config views.

## Managed command inventory

| Command | Binary | Purpose | Owned options |
| --- | --- | --- | --- |
| `compile_domain_configs` | `src/bin/compile_domain_configs.rs` | Compile authored `domain/` YAML into governed generated config files under `configs/ci/`. | `--domain-dir <path>`, `--configs-dir <path>`, `--scope <scope>` |
| `domain_registry_bundle` | `src/bin/domain_registry_bundle.rs` | Materialize the typed domain release bundle from authored YAML or print an existing bundle file. | `--domain-dir <path>`, `--configs-dir <path>`, `--bundle <path>`, `--write-generated` |
| `domain_registry_query` | `src/bin/domain_registry_query.rs` | Query domains, stages, tools, defaults, metrics, artifacts, deprecations, evidence, or fixtures from a typed registry bundle. | `--bundle <path>`, `--domain-dir <path>`, `--kind <domains|stages|tools|metrics|artifacts|defaults|deprecations|evidence|fixtures>`, `--domain <id>`, `--stage-id <id>`, `--tool-id <id>` |
| `domain_validate` | `src/bin/domain_validate.rs` | Validate authored domain YAML, reference catalogs, stage/tool indexes, and cross-domain coverage. | `--domain-dir <path>` |

The default compile scope is `pre_hpc_pre_vcf`. That scope emits FASTQ/BAM runtime config
views and keeps VCF config views separated into explicit generated VCF files. The bundle command
can also write the governed JSON release surfaces under `configs/ci/registry/`.

## Boundary

- These commands may read authored domain files and write declared generated config outputs.
- These commands may read a previously generated bundle file when `--bundle` is supplied.
- These commands must not execute bioinformatics tools, launch containers, call network services,
  schedule pipelines, or mutate runtime state.
- New command flags must be documented here in the same change that adds the CLI behavior.

## Forbidden Command Surfaces

- No bioinformatics tool execution.
- No container, scheduler, or runtime orchestration.
- No network clients.
- No writes outside declared generated config outputs.

## Verification

Use `CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-domain-compiler --no-default-features --test boundaries`
to verify that the command inventory matches the binary tree.
