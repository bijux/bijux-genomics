# Change Rules

Use these rules when changing `bijux-dna-stages-vcf`.

## Stage Surface

- Add new VCF stage IDs in the VCF domain crate first.
- Update `stage_specs.rs`, `docs/STAGE_CONTRACTS.md`, and
  `docs/COMMANDS.md` when a stage becomes managed here.
- Keep `implemented_stages()` and the catalog tests aligned with the supported
  stage set, not the full planned domain catalog.

## Execution Behavior

- Keep stage outputs deterministic for fixed fixtures, params, and local tool
  behavior.
- Record refusal reasons through typed errors or existing refusal codes.
- Keep writes under caller-provided output directories.
- Document every new environment override in `docs/EFFECTS.md`.

## Documentation

- Keep one root `README.md`.
- Keep all other crate docs in `docs/`.
- Keep `docs/COMMANDS.md` as the SSOT for managed operations.
- Keep `docs/` at 10 Markdown files or fewer.

## Dependencies

- Do not add API, planner, runtime, or environment dependencies here. Runner is
  the sole process-execution edge, and stage-contract is the shared plugin edge.
- Prefer domain, stage-contract, reference-db, core, and infra dependencies
  already used by the crate.
- Add dependency-boundary tests with any dependency graph change.
