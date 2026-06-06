# bijux-dna-domain-vcf Tests

The test suite locks VCF domain contracts, generated registry parity, parser fixture banks,
guardrails, and crate layout.

## Test Map

| Surface | Test file or directory | Contract |
| --- | --- | --- |
| Contracts | `tests/contracts.rs`, `tests/contracts/*` | Stage taxonomy, transitions, params, metrics, invariants, registry output, committed config parity, and parser fixture normalization. |
| Guardrails | `tests/guardrails.rs` | Repository policy guardrails for the crate. |
| Boundaries | `tests/boundaries.rs`, `tests/boundaries/*` | Docs layout, command-free surface, dependency graph, and source/test tree shape. |

## Fixture Banks

- `tests/fixtures/bench/parsers/vcf/bcftools/<stage>/` stores the governed raw artifact bank for
  retained `bcftools` VCF stages.
- `tests/fixtures/bench/parsers/vcf/angsd/<stage>/` stores the governed raw artifact bank for
  retained `angsd` low-coverage VCF stages.
- `tests/fixtures/bench/parsers/vcf/plink/<stage>/` stores the governed raw artifact bank for the
  retained classic `plink` VCF rows.
- `tests/fixtures/bench/parsers/vcf/plink2/<stage>/` stores the governed raw artifact bank for the
  retained `plink2` VCF rows.
- `tests/fixtures/bench/parsers/vcf/eigensoft/pca/` and
  `tests/fixtures/bench/parsers/vcf/eigensoft/population_structure/` store the governed raw
  artifact bank for the retained `eigensoft` population-analysis rows.
- `tests/fixtures/bench/parsers/vcf/phasing/<tool_id>/` stores the governed raw artifact bank for
  the retained phasing backends.
- Every stage directory must contain the raw parser inputs required by `src/parsers/bcftools.rs`
  `src/parsers/angsd.rs`, `src/parsers/eigensoft.rs`, `src/parsers/phasing.rs`, or
  `src/parsers/plink_family.rs` plus `expected.normalized.json`.
- `tests/contracts/parsers/bcftools_fixture_bank.rs` is the SSOT that proves the checked-in raw
  artifacts still normalize to the committed expected payloads.
- `tests/contracts/parsers/angsd_fixture_bank.rs` is the SSOT for the retained ANGSD low-coverage
  parser bank.
- `tests/contracts/parsers/plink_fixture_bank.rs` is the SSOT for the retained classic `plink`
  parser bank.
- `tests/contracts/parsers/plink2_fixture_bank.rs` is the SSOT for the retained `plink2`
  population-analysis parser bank.
- `tests/contracts/parsers/eigensoft_fixture_bank.rs` is the SSOT for the retained `eigensoft`
  PCA and population-structure parser bank.
- `tests/contracts/parsers/phasing_fixture_bank.rs` is the SSOT for the retained phasing parser
  bank, including all-unphased rejection checks.

## Commands

```bash
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-domain-vcf --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-domain-vcf --no-default-features --test contracts
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-domain-vcf --no-default-features --test guardrails
CARGO_TARGET_DIR=artifacts/cargo-target cargo clippy -p bijux-dna-domain-vcf --all-targets --no-default-features -- -D warnings
```

## Artifact Discipline

Rust build products must use `CARGO_TARGET_DIR=artifacts/cargo-target`. Generated config artifacts
under `configs/ci/` should change only when registry rendering intentionally changes.
