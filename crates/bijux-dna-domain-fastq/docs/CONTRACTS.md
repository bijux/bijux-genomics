# bijux-dna-domain-fastq Contracts

This crate is the FASTQ domain contract source for the workspace. Contract changes must be explicit,
reviewable, and covered by tests.

## Owned Contracts

- Stage IDs, stage semantics, IO contracts, and canonical stage ordering.
- Tool IDs, stage-tool compatibility, maturity, normalization, and benchmark readiness.
- Parameter descriptors, typed effective parameter models, canonical JSON, and defaults.
- Metric schemas, metric classes, retention semantics, and invariant evaluation rules.
- Adapter, contaminant, and polyX bank models, preset resolution, provenance, and deterministic
  bank hashing.
- Observer parser contracts and normalization surfaces for stage/tool outputs.
- Benchmark query context metadata and benchmark corpus descriptors.

## Change Rules

- Adding a new public contract field is breaking unless the type documents a default or fallback.
- Removing or renaming a public export, stage ID, tool ID, artifact ID, metric field, or parameter
  field is breaking.
- Bank changes require provenance updates and refreshed deterministic fixtures or snapshots.
- Metric and invariant changes require updated semantic tests and contract snapshots.
- Stage ordering or dependency changes require pipeline contract tests and domain manifest parity.

## Failure Patterns

- Adapter failures: elevated adapter k-mer rates or adapter trimming report mismatches.
- PolyX failures: low-complexity or polyG/polyN metrics outside stage thresholds.
- rRNA contamination: screen rates or database provenance outside governed expectations.
- Low complexity: entropy/complexity metrics outside invariant thresholds.
- Retention ambiguity: any naked percentage without numerator, denominator, unit, and stage boundary.

## Verification

Contract updates should run the full crate tests plus the focused contract suite:

```bash
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-domain-fastq --no-default-features --test contracts
```

Reference-index compatibility is admitted by `validate_reference_index_backend` from the authored integration matrix. Host and reference-contaminant depletion planners call it directly, so standalone and composed planning reject unsupported index backends consistently. Matching an index artifact role or recording a backend name does not establish compatibility.
