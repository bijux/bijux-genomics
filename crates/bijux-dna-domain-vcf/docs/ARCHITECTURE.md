# bijux-dna-domain-vcf Architecture

`bijux-dna-domain-vcf` is a pure VCF domain library. It exposes typed contracts and deterministic
registry-rendering helpers; callers decide if and where generated TOML is written.

## Layout

```text
src/
  artifacts/             # schema fragments and typed output-truth rows
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
- `artifacts.rs` and `artifacts/` own output-truth assembly plus the supporting typed rows and
  schema fragments for VCF downstream reports.
- `params/` and `metrics.rs` own schema-versioned public payloads.
- `parsers/` owns normalization of governed raw artifact banks into shared stage metrics payloads,
  including retained `bcftools`, `angsd`, `plink`, `plink2`, and descent-family VCF surfaces.
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

`artifacts/genotype_calls.rs` owns phase-insensitive genotype comparison. Exact
strings retain equality; otherwise complete calls with at least two alleles are
compared without allele order or phase separators. This operation does not
validate genotype syntax. Science evidence and benchmark workflows share this
domain contract without depending on one another.

`artifacts/genotype_truth.rs` owns pure genotype, ploidy, phase and likelihood
projection over an internal typed request. Declared input paths and caller labels
are report metadata; the kernel does not read them. The public Path facade retains
its existing document acquisition followed by statistics acquisition, including
error order and supplied missingness. Native decoding and filesystem acquisition
remain in that compatibility path. This boundary does not change the summary
schema or claim that every domain artifact service is free of acquisition effects.

`artifacts/fixture_genotypes/` owns strict mini-fixture GT observations over borrowed
VCF text: allele dosage, zygosity, phase, per-contig homozygosity and last-seen site
frequencies. The typed request's source path is diagnostic metadata. The caller
retains text acquisition, expected-truth document construction, validation and
publication. Strict fixture syntax intentionally differs from the aggregate
`genotype_truth` projection; neither capability substitutes the other's semantics.

`artifacts/fixture_genotypes/homozygosity.rs` owns runs of at least two adjacent
homozygous observations and paired runs over the same per-contig flags. Contig
boundaries, existing record order and paired-vector truncation are retained.
The expected-truth caller keeps ROH/IBD document schemas, state admission,
independent equal-dosage site counts, acquisition and publication. These fixture
run counts do not claim physical interval alignment or matching genotype alleles.
