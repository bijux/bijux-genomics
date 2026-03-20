# ADD_TOOL

## Checklist
- Declare the tool in `domain/fastq/tools/*.yaml` with `execution_contract`, `expected_artifacts`, and `stage_contracts`.
- Close the stage/tool support state in `domain/fastq/execution_support.yaml` only when planning, runtime, parsing, and comparison are genuinely governed.
- Add or extend planner adapters only for closed runtime bindings that need stage-family command planning.
- Keep parsing and normalization in `bijux-dna-stages-fastq`; do not hide runtime interpretation in planner glue.
- Add contract tests and benchmark suite coverage for the new governed surface.
- Do not maintain manual stage-to-tool tables in docs; manifests, execution support, and generated registries are the source of truth.
