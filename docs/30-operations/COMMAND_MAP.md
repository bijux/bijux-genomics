# Command Map

Operator command map for the currently shipped CLI surface.

Authority:
- [docs/cli/command_snapshot.txt](../cli/command_snapshot.txt)
- [docs/cli/release_help_snapshot.txt](../cli/release_help_snapshot.txt)

## Validate
- Manifest contract check:
  `cargo run -q -p bijux-dna -- validate-manifests`
- Example contract/index check:
  `cargo run -q -p bijux-dna-dev -- examples run check-index`

## Plan
- List governed profiles:
  `cargo run -q -p bijux-dna -- plan list`
- Explain a governed profile:
  `cargo run -q -p bijux-dna -- plan explain-profile`
- Validate profile invariants:
  `cargo run -q -p bijux-dna -- plan validate-profile`
- Explain a cross-domain template-backed profile:
  `cargo run -q -p bijux-dna -- plan explain-profile fastq-to-vcf__minimal__v1`
- Validate cross-domain template registry alignment:
  `cargo run -q -p bijux-dna -- plan validate-profile bam-to-vcf__default__v1`

## Explain
- Summarize governed runs:
  `cargo run -q -p bijux-dna -- explain summary`
- Render governed run facts:
  `cargo run -q -p bijux-dna -- explain report`

## Execute
- Deterministic canonical example bundle:
  `cargo run -q -p bijux-dna-dev -- examples run run -- <example-id>`
- FASTQ workflow execution surface:
  `cargo run -q -p bijux-dna -- run run`

## Inspect
- Current governed status view:
  `cargo run -q -p bijux-dna -- status --contracts`
- Search analyzed runs:
  `cargo run -q -p bijux-dna -- analyze runs`
- Run operations evidence paths and recovery steps:
  [RUN_OPERATIONS.md](RUN_OPERATIONS.md)

## Verify
- Verify an evidence bundle:
  `cargo run -q -p bijux-dna -- analyze evidence verify --run-id <run-id>`

## Replay
- Validate replayability from a recorded manifest:
  `cargo run -q -p bijux-dna -- replay <run-id> --validate-only`

## Diff
- Diff governed profiles:
  `cargo run -q -p bijux-dna -- plan profile-diff`
- Diff governed runs:
  `cargo run -q -p bijux-dna -- compare <run-a> <run-b>`
- Diff evidence bundles:
  `cargo run -q -p bijux-dna -- analyze evidence compare <left-bundle> <right-bundle>`

## Report
- Render a governed run report:
  `cargo run -q -p bijux-dna -- analyze report <run-id>`

Notes:
- Use [examples/index.yaml](../../examples/index.yaml) to discover governed canonical example ids.
- Use [artifacts/planning/scoreboard.yaml](../../artifacts/planning/scoreboard.yaml) and
  [artifacts/planning/cards.yaml](../../artifacts/planning/cards.yaml) for Level 1 closure scope instead of manually copying goal tables here.
