# Architecture

`bijux-dna-planner-vcf` is organized as a planning library with private implementation modules and a small root public API.

## Layout
- `src/lib.rs` declares private modules and re-exports the public planner surface.
- `src/api.rs` defines caller inputs and panel lock structures.
- `src/planner.rs` coordinates input validation, reference resolution, stage selection, tool selection, stage planning, and graph construction.
- `src/stage_sequence.rs` resolves default and requested VCF stage order.
- `src/reference_context.rs` resolves panel, map, and reference bundle context through `bijux-dna-db-ref`.
- `src/tool_catalog.rs` and `src/tool_selection.rs` map stages and coverage regimes to governed tools.
- `src/stage_plan.rs`, `src/stage_io.rs`, `src/params.rs`, and `src/chunk_plan.rs` build deterministic stage plan payloads.
- `src/execution_graph.rs` turns stage plans into a planner-level execution graph.
- `src/explain.rs` and `src/explain_model.rs` expose deterministic explain output.
- `src/workspace_config.rs` reads repository-owned registry files used to validate stages, tools, and params.

## Dependency Direction
The planner depends on core graph contracts, VCF domain contracts, DB reference catalog views, and stage contract payload types. Runtime, runner, CLI, API, environment, benchmark, FASTQ, and BAM crates stay outside this dependency graph.

## Runtime Boundary
The crate emits planned command specs. It does not spawn processes, route commands, parse tool outputs, or execute products.

## Review Focus
Changes to stage ordering, tool defaults, reference resolution, graph edges, explain output, or public exports should include boundary or snapshot coverage.
