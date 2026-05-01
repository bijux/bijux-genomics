# bijux-dna-domain-compiler Architecture

`bijux-dna-domain-compiler` is a boundary crate between authored domain metadata and generated
configuration views. It validates source-of-truth YAML and emits deterministic TOML files consumed
by CI/runtime-facing crates, plus typed JSON release bundles consumed by governance and API layers.

## Layout

```text
src/
  bin/
    compile_domain_configs.rs  # CLI wrapper for config generation
    domain_registry_bundle.rs  # CLI wrapper for typed release-bundle emission
    domain_registry_query.rs   # CLI wrapper for typed release-bundle queries
    domain_validate.rs         # CLI wrapper for domain validation
  compiler/
    bundle.rs                  # typed registry bundle builders and query projection
    compile.rs                 # generation orchestration and output writes
    coverage.rs                # validation coverage reporting
    loading/                   # source loading and TOML rendering builders
      image_registries.rs      # image/source registry readers
      index_catalogs.rs        # reference index catalog materialization
      index_defaults.rs        # generated reference-index defaults
      load_and_collect.rs      # shared load/collect orchestration
      stage_loading.rs         # authored stage YAML readers
      stage_registries.rs      # stage registry render builders
      tool_loading.rs          # authored tool YAML readers
      tool_registries.rs       # tool registry render builders
    models.rs                  # internal domain/config data shapes and public options
    support/                   # repository, rendering, placeholder, status, and tool helpers
      placeholders.rs          # placeholder and planned-status policy helpers
      render.rs                # deterministic rendering helpers
      repository.rs            # workspace and source path helpers
      status.rs                # status validation helpers
      tooling.rs               # tool-role and domain-meaning helpers
    validation/                # schema, catalog, index, stage, and tool validation
      catalog_coverage.rs      # stage coverage validation
      catalog_validation.rs    # generated catalog consistency checks
      deprecations.rs          # deprecation replacement and release policy checks
      fixture_consistency.rs   # tool/stage fixture claim cross-checks
      index_rules/             # reference-index compatibility, inventory, and version rules
      stage_files.rs           # stage YAML validation
      strict_stage_schemas.rs  # required schema-field and vocabulary checks
      tool_files.rs            # tool YAML validation
    vcf_emit.rs                # separate generated VCF config views
  lib.rs                       # public crate surface
```

## Data Flow

1. `validate_domain` verifies required domain files, reference catalogs, vocabularies, stage files,
   tool files, indexes, shared-tool ownership, and canonical stage coverage.
2. `compile_domain_configs` loads the selected active scope from domain YAML.
3. The compiler builds tool registries, image versions, stage registries, VCF-specific views, and
   the typed domain registry release bundle.
4. Generated files are written beneath `CompileOptions::configs_dir`.
5. Query helpers and CLI wrappers project domains, stages, tools, defaults, metrics, artifacts,
   deprecations, evidence, and fixtures from the release bundle without mutating state.
6. Determinism tests compare the generated output set across repeated compiles.

## Boundaries

- Binaries are thin argument parsers. Compiler behavior and bundle-query logic live in library code.
- `loading/` may parse authored source and build deterministic render buffers.
- `validation/` owns correctness checks before generated configs or bundle surfaces are trusted.
- `support/` may provide filesystem and rendering helpers, but must not become runtime execution.
- Runtime, runner, stage, planner, benchmark, API, database, and developer-control-plane behavior
  belongs outside this crate.

## Naming Rules

- CLI binaries use the managed command names listed in [COMMANDS.md](COMMANDS.md).
- Test modules use behavior names such as `boundaries`, `determinism_generated_outputs`,
  `domain_registry_bundle`, `domain_registry_cli`, and `planned_tool_registry_boundaries`.
- Generated output paths must stay documented in [CONTRACTS.md](CONTRACTS.md) and covered by tests.
