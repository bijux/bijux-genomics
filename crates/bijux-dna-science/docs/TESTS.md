# bijux-dna-science Tests

## Test Files
- `tests/contracts.rs` checks typed science identifier validation.
- `tests/fastq_environment_slice.rs` verifies compiled FASTQ evidence and generated output parity.
- `tests/tsv_shape.rs` verifies governed TSV files remain rectangular.
- `tests/guardrails.rs` runs shared workspace guardrails for this crate.

## Local Commands
- `cargo test -p bijux-dna-science --test contracts`
- `cargo test -p bijux-dna-science --test fastq_environment_slice`
- `cargo test -p bijux-dna-science --test tsv_shape`
- `cargo test -p bijux-dna-science --test guardrails`
