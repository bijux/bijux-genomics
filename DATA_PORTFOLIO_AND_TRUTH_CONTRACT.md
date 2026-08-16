# Data Portfolio and Truth Contract

Owner: `bijux-dna-bench`, `bijux-dna-bench-model`, and `bijux-dna-science`
Scope: Production benchmark data, truth, materialization, admission, and claim eligibility
Contract version: v1
Status: Normative

## Purpose

This contract separates a data source declaration from benchmark evidence. A public accession,
large file, checked-in fixture, successful download, or successful tool invocation is not by itself
admissible scientific evidence. Production publication requires a checksum-closed, scientifically
fit corpus or truth set admitted under the governed STUDY root.

The machine-readable authorities are
[`benchmarks/contracts/portfolio.toml`](benchmarks/contracts/portfolio.toml),
[`benchmarks/contracts/data-fitness.toml`](benchmarks/contracts/data-fitness.toml), and the source
definitions in [`benchmarks/corpora/`](benchmarks/corpora/). This document defines how those
authorities must be interpreted by the Rust product owners.

## Ownership

| Surface | Exclusive responsibility |
| --- | --- |
| `bijux-dna-bench-model` | Persisted corpus, source, data-index, fitness, audit, truth, evidence, and publication records; validation; generated schemas. |
| `bijux-dna-bench` | Source loading, acquisition planning, materialization, checksum validation, admission, portfolio audit, case binding, evidence aggregation, publication, and replay. |
| `bijux-dna-science` | Truth comparison, uncertainty, bias assessment, scientific exclusions, claim eligibility, and scientific verdicts. |
| `bijux-dna-policies` | Access, license, redistribution, retention, and publication-policy verdicts. |
| `benchmarks/` | Reviewed declarations, generated schemas, permanent run definitions, citations, indexes, and small reports only. |

Shell and Python may implement a narrow source- or tool-specific transfer or conversion adapter.
They must not allocate data identities, decide eligibility, admit evidence, derive scientific
verdicts, select publications, or reconstruct policy outside the typed Rust lifecycle.

## Evidence Classes

Data used for code correctness and data used for scientific publication are different products:

- `toy`, `fixture`, `fake`, `micro`, `smoke`, and `synthetic_only` inputs may test parsing,
  scheduling, resource wiring, failure handling, and schema closure.
- `qualification_real` inputs may prove that an artifact executes a real stage contract and emits
  admissible formats. They do not establish comparative scientific performance unless the bound
  fitness profile explicitly permits the claim.
- `production_real` inputs may support publication only after materialization, admission, fitness,
  truth, control, and evidence closure all pass.
- Supplementary simulation or spike-ins may expose controlled failure modes, but synthetic-only
  evidence never replaces the required real-data cohort.

The portfolio audit must fail closed when a corpus marked publishable is bound to a suspected toy,
fixture, synthetic, or smoke asset.

## Corpus States

`missing`, `source_defined`, and `admitted` are distinct lifecycle states:

- `missing` means that no stable source selection exists.
- `source_defined` means that source authorities, access terms, scientific purpose, expected scale,
  and limitations are reviewed, but no production evidence has been admitted.
- `admitted` means that every selected file, catalog, sample count, access receipt, checksum,
  lineage record, and eligibility decision is present and mutually consistent in STUDY.

A declaration never becomes admitted merely because its bytes happen to exist. Admission is an
immutable Rust-owned transition. Historical admission records remain addressable and are never
overwritten to make a newer corpus current.

## Required Identity Closure

Every corpus, reference, database, panel, or truth set must retain:

- a stable corpus and catalog identity;
- source authority, accession or release identity, and acquisition timestamp;
- license or access terms, redistribution classification, allowed use, and attribution;
- organism, sample, library, assay, platform, layout, and study context where applicable;
- exact file size and SHA-256 for every materialized input;
- immutable reference build, contig policy, database release, and exclusion policy;
- sample and case counts with the authority used to derive them;
- selection criteria, cohort strata, controls, held-out split, and truth role;
- acquisition, transformation, validation, and derivation lineage;
- known limitations, bias risks, and permitted scientific claims;
- an identity-addressed STUDY URI for data, catalogs, receipts, and derived truth.

Mutable URLs, live database names, local absolute workspace paths, and scheduler job IDs are
provenance inputs, not stable data identities.

Planned biological samples and planned archive accessions are separate counts. A treatment pair,
technical replicate, or multi-run library may require multiple exact accessions for one sample;
the source authority must freeze both counts and every selected accession without inflating the
scientific sample count. An unfrozen sample-bearing collection is not acquisition-ready and must
remain an explicit portfolio-audit blocker.

## Scientific Fitness

The bound profile in
[`benchmarks/contracts/data-fitness.toml`](benchmarks/contracts/data-fitness.toml) is evaluated
before a case can support publication. Its minimum real samples, cases per tool, distinct studies,
input scale, representative strata, layouts, controls, truth requirement, bias assessments, and
required limitations are conjunctive requirements.

Ten samples are the normal comparative minimum, not an automatic certificate of fitness. A panel
can be large and still be scientifically unsuitable because it lacks relevant strata, independent
truth, negative controls, held-out observations, access identity, or a defensible sampling design.
An exception is valid only when the profile records the reduced dimension, original and permitted
minimum, scientific rationale, compensating evidence, review authority, and narrower allowed
claims.

Samples derived from the same biological material, lane, accession, or technical replicate must
not be counted as independent without an explicit dependence model. Case count, sample count, and
study count remain separate quantities.

## Truth and Claim Rules

Truth strategy determines what may be claimed:

| Truth requirement | Permitted interpretation |
| --- | --- |
| `curated_truth` | Accuracy against an independently curated and checksum-bound truth surface, within its declared regions and variant or signal classes. |
| `masked_truth` | Recovery or calibration against a frozen held-out split that was excluded from planning and parameter selection. |
| `orthogonal_expectation` | Accuracy or error-detection claims only for the independently observed signal and controls; tool agreement is not truth. |
| `observational_performance_only` | Runtime, resource use, robustness, repeatability, interoperability, and descriptive consistency; no biological accuracy claim. |

Cross-tool agreement can expose integration defects but cannot establish correctness. Mapping-derived
statistics are not independent alignment truth, a derived call set is not independent genotype
truth, and an observational population cohort does not make inferred ancestry or demography a
ground-truth label.

Held-out truth must be generated before comparative execution, stored separately from candidate
inputs, and bound by an immutable split manifest. Parameters, thresholds, tool selection, and
scientific exclusions must not inspect the held-out labels. Any post-hoc access invalidates the
affected accuracy claim and requires a new split identity and execution attempt.

## Controls and Strata

Positive, negative, no-op, blank, untreated-input, and technical-replicate controls are retained as
first-class cases when required by the fitness profile. Damage, contamination, ancestry, sex,
coverage, library preparation, read length, database composition, missingness, and other declared
strata must remain visible through aggregation. A portfolio-wide mean must not erase a known
scientific failure mode or substitute for per-stratum uncertainty.

Exclusions are allowed only when their rule is declared before aggregation or when immutable
failure evidence supports a scientifically reviewed exception. Tool failures, unavailable truth,
and missing strata remain visible; they are not removed to make a comparison green.

## Materialization and Admission

The production lifecycle is one-way and identity-addressed:

1. Load and validate the checked-in portfolio, fitness rubric, and source specification.
2. Resolve source metadata and access terms without treating live metadata as immutable evidence.
3. Freeze the acquisition plan, selected accessions, expected files, resource projection, and
   allowed use.
   A source portfolio that spans multiple archive scientific names is split into independently
   identified collection plans. Every selected FASTQ collection declares its sample era as
   `ancient`, `modern`, or `not_applicable`; the resolver never infers this scientific context
   from names or accession prefixes.
4. Transfer or derive bytes into a staging root and validate format, record closure, sample
   identity, size, and checksum.
5. Publish an immutable governed catalog and access receipt under STUDY.
6. Derive a typed data index from admitted catalogs; never reconstruct admission from directory
   contents.
7. Audit portfolio bindings, sample counts, source-spec hashes, file integrity, fitness, and
   suspected nonproduction contamination.
8. Bind each benchmark case to exact data-index, corpus, truth, reference, database, and held-out
   split identities. Every required `ToolBinding` input must resolve through its declared corpus,
   reference, database, or upstream-result source role before the case can be allocated. The case
   authority persists every selected catalog file's canonical STUDY URI, SHA-256 digest, byte size,
   and governed role; catalog identity alone is not an executable material closure.
9. Admit execution evidence only when the complete input and scientific contract remains current.

Materialization may run concurrently for independent assets. A blocked acquisition blocks only
the cases that require it. No campaign waits for every portfolio asset, and no foreground process
polls a transfer, scheduler job, or report file.

## Access and Redistribution

Access, product use, and redistribution are independent decisions. Controlled or restricted data
may be benchmark-eligible in an authorized private STUDY scope while remaining prohibited from
Git, public reports, bundles, or clean-room replay payloads. Publications expose only permitted
derived observations and stable source citations; they never expose restricted bytes, credentials,
private paths, or access tokens.

An expired, revoked, unresolved, or scope-incompatible access receipt makes the affected material
unavailable for new execution. Existing historical evidence remains immutable but cannot be
selected as current unless the publication policy explicitly validates its continuing use.

## Publication and Replay

A report is current only when its source, catalog, stage/interface, data index, corpus, truth,
reference/database, artifact, campaign, result, and report hashes match. Changing any dependency
creates a new report identity and invalidates only the affected closure. Historical reports are
never overwritten.

Every accuracy claim must identify its truth strategy, truth digest, held-out split when used,
eligible strata, uncertainty method, controls, exclusions, and limitations. Performance-only
evidence must say so explicitly. Replay verifies identities and deterministic derivations without
requiring redistribution of source bytes that policy forbids.

## Enforcement

At minimum, validate the checked-in data contract and generated surfaces with:

```sh
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-bench-model
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-bench
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-policies --test contracts
CARGO_TARGET_DIR=artifacts/cargo-target cargo run -q -p bijux-dna-dev -- tooling run benchmark-data-portfolio-audit -- schema --output artifacts/data-portfolio-audit.schema.json
```

Production readiness additionally requires a source-commit-bound portfolio audit derived from the
live governed data index. A green fixture suite, source declaration, source SIF, catalog row, or
successful scheduler job cannot substitute for that audit.
