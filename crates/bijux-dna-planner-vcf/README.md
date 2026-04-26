# bijux-dna-planner-vcf

`bijux-dna-planner-vcf` builds deterministic VCF stage plans, execution graphs, and explain payloads from VCF domain contracts, reference catalog context, tool registry declarations, and typed planner inputs.

## Ownership
This crate owns VCF planning only. It chooses stage order, validates requested stages and tools against VCF domain and registry contracts, resolves reference/panel context, materializes stage plans, builds execution graphs, and explains the plan.

It must not execute tools, parse runtime metrics, route CLI commands, discover tools from the environment, mutate generated configuration, or own runtime orchestration.

## Public Surface
The public API is the root export surface from `src/lib.rs`:

- `VcfPipelineInputs`, `VcfPanelLock`, and `ChunkPlanSettings`
- `RegionChunkPlan`
- `PlannerExplainV1` and `PlannerExplainStage`
- `plan_vcf_stage_plans`, `plan_vcf_pipeline`, `plan_vcf_minimal`
- `explain_vcf_plan`
- `PLANNER_VERSION`

## Documentation
- [docs/INDEX.md](docs/INDEX.md)
- [docs/COMMANDS.md](docs/COMMANDS.md)
- [docs/DEPENDENCIES.md](docs/DEPENDENCIES.md)
- [docs/PUBLIC_API.md](docs/PUBLIC_API.md)
- [docs/TESTS.md](docs/TESTS.md)

## Tests
Run:

```bash
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-planner-vcf --no-default-features
```
