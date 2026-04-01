# Tests

## Intent
The test tree is organized by what each suite protects.

## Suite map
- `tests/boundaries.rs`: effect-boundary enforcement and architecture-tree checks
- `tests/contracts.rs`: orchestration behavior, params hashing, recording truth-set contracts, and
  helper naming rules
- `tests/determinism.rs`: replay determinism and manifest layout stability

## Important directories
- `tests/contracts/recording/`: execution truth-set documentation and recording completeness checks
- `tests/support/`: shared engine fixtures, graph builders, and runner stubs
- `tests/schemas/`: reserved for future engine-owned schema assertions; the engine currently does
  not keep a standalone schema test target

## Regenerating affected coverage
- boundaries: `cargo test -p bijux-dna-engine --test boundaries -j 1`
- contracts: `cargo test -p bijux-dna-engine --test contracts -j 1`
- determinism: `cargo test -p bijux-dna-engine --test determinism -j 1`

## Failure interpretation
- boundary failures mean the source tree or effect boundary drifted
- contract failures mean execution behavior or recorded artifacts changed
- determinism failures mean stable ordering or repeatable output changed
