# bijux-dna-science Tests

## Test Files

- `tests/contracts.rs` checks typed science identifier validation and external evidence acquisition.
  `tests/contracts/archive_sync.py` uses local Git fixtures to exercise clone/fetch,
  export, storage overrides, and rejected destinations without network access.
- `tests/fastq_environment_slice.rs` verifies compiled FASTQ evidence and generated output parity.
- `tests/tsv_shape.rs` verifies governed TSV files remain rectangular.
- `tests/guardrails.rs` runs shared workspace guardrails for this crate.

## Test Taxonomy

- Contract tests protect public identifiers, docs layout, command inventory, public API docs,
  dependency boundaries, and architecture shape.
- Evidence parity tests protect deterministic compiler output against committed generated files.
- Shape tests protect governed TSV inputs and outputs.
- Guardrail tests keep the crate wired into repository policy.

## Local Commands

- `CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-science --no-default-features`
- `CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-science --test contracts --no-default-features`
- `CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-science --test fastq_environment_slice --no-default-features`
- `CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-science --test tsv_shape --no-default-features`
- `CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-science --test guardrails --no-default-features`

Private compilation cases in `tests/unit/compilation.rs` prove that an admitted observation keeps environment and container views consistent after registry changes and source removal, while a new acquisition observes the new registry. Shared reads never excuse a missing source relationship on another binding. The existing FASTQ slice case remains the generated-output compatibility authority.
