# bijux-dna-science Commands

## Commands
- `validate` compiles authored specs without writing generated outputs.
- `build` compiles authored specs and refreshes governed generated outputs.
- `trace` prints filtered FASTQ stage-tool evidence rows.
- `closure` prints filtered FASTQ closure status rows.
- `release --release-id <id>` writes a science release bundle under `artifacts/science-releases/`.

## Inputs
All commands resolve workspace-relative authored specs from `science/specs/**`.

## Outputs
Generated build outputs are written under `science/generated/**`. Release outputs are written under
`artifacts/science-releases/**`.
