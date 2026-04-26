# bijux-dna-planner-fastq Docs Index

## Start Here
- `ARCHITECTURE.md` — source layout, planner subsystems, and stage adapter families.
- `BOUNDARY.md` — allowed inputs, forbidden dependencies, and forbidden effects.
- `PUBLIC_API.md` — public modules, root exports, and `stage_api` surface.
- `COMMANDS.md` — runtime command status and planned command-spec inventory.
- `DEPENDENCIES.md` — allowed dependency graph and forbidden dependency direction.
- `EFFECTS.md` — allowed effects and planner purity rules.
- `DETERMINISM.md` — stable ordering, graph, explain, and snapshot guarantees.
- `EXPLAIN_OUTPUT.md` — explain metadata and snapshot expectations.
- `TESTS.md` — test entry points, contract modules, fixtures, and standard command.

## Documentation Rules
- Root `README.md` is the only Markdown file at the crate root.
- All other crate docs live in `docs/`.
- This crate keeps exactly ten docs under `docs/`.
- Do not maintain manual stage-to-tool matrices in Markdown; domain manifests and tests are the source of truth.
