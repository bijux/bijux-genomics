# bijux-dna-domain-vcf Architecture

`bijux-dna-domain-vcf` is a pure VCF domain library. It exposes typed contracts and deterministic
registry-rendering helpers; callers decide if and where generated TOML is written.

## Layout

```text
src/
  contracts/
    invariants.rs         # VCF, species, and panel-map invariant checks
    panel_governance.rs   # reference panel governance and selection policy
    stage_delivery.rs     # output format guarantees
    stage_io.rs           # required inputs, outputs, and indexes
    stage_metrics.rs      # metrics schemas and required fields
  params/                 # typed param contracts and effective params
  taxonomy/               # downstream stage taxonomy and ordering
  coverage.rs             # domain coverage report
  lib.rs                  # public facade and catalogs
  metrics.rs              # schema-versioned VCF metrics
  parsers/                # normalized parser contracts for retained raw VCF artifacts
  registry_emit.rs        # deterministic TOML string materialization
  stage_baseline.rs       # canonical call/filter/stats baseline
```

## Ownership Rules

- `contracts/` owns validation and stage contract truth.
- `params/` and `metrics.rs` own schema-versioned public payloads.
- `parsers/` owns normalization of governed raw artifact banks into shared stage metrics payloads.
- `taxonomy/` owns downstream stage order and forbidden transitions.
- `registry_emit.rs` may render strings only; filesystem writes stay outside this crate.
- Runtime, runner, planner, stage execution, API, database, and environment behavior belong
  outside this crate.

## Data Flow

1. Public catalogs in `lib.rs` expose stable IDs.
2. Typed params and metrics define serializable payload contracts.
3. Stage taxonomy and contract modules validate downstream VCF invariants.
4. Registry rendering functions produce deterministic TOML that tests compare to committed config
   artifacts.
