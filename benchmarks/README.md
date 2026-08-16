# Benchmarks

This root owns both deterministic benchmark harness assets and publishable
scientific comparisons. Benchmark-owned configuration, schemas, fixtures, and
readiness consumers resolve through the shared benchmark path contract. The
default root is `benchmarks/`; supported commands may override it with
`--benchmark-root` or `BIJUX_BENCHMARK_ROOT`.

The established subroots remain:

- `configs/` for local and HPC benchmark pipeline inputs
- `schemas/` for normalized metric contracts
- `tests/` for benchmark-owned focused checks and fixtures
- `readiness/` for deterministic harness and adapter proof surfaces

Readiness files prove that code paths and contracts exist. They are not real
scientific observations. Disposable local-smoke and scheduler-dry-run products
remain outside this tracked authority under the governed artifact or study
result roots.

This root also governs scientific comparisons across every retained FASTQ,
BAM, and VCF stage. It does not treat a successful container invocation,
one-sample timing, fixture execution, or agreement between two programs as
scientific completion.

[`contracts/portfolio.toml`](contracts/portfolio.toml) is the scientific admission authority. It
defines the real-data corpus portfolio, minimum cohort sizes, truth strategy,
resource measurements, canonical output contract, and downstream substitution
probe required for each of the 71 retained stages. The assignment closure is
machine validated: a newly retained stage cannot appear without an explicit
scientific profile.

The same file reserves one stable `run-2001`–`run-2071` identity for every
retained stage. A reservation is not evidence: it gives the corpus preparation,
Slurm manifests, tool/sample executions, aggregate report, and replay record one
durable campaign identity before any result exists. An active campaign whose
stage and reserved identity disagree is rejected.

The portfolio separates three facts that were previously blurred:

- `source_defined` means accessions and selection rationale exist, but the data
  are not yet checksum-admitted beneath the governed study root.
- `admitted` means files, source identity, retrieval time, size, and digest have
  been verified. A one-sample admitted corpus may qualify execution but cannot
  support comparative claims.
- `comparative` evidence requires the complete applicable tool set, at least ten
  real samples, a stage-appropriate truth strategy, repeated resource
  measurements, and a real downstream consumer for every tool output.

Validate the portfolio and render the exact blockers for every stage:

```console
cargo run -p bijux-dna-dev -- \
  tooling run benchmark-scientific-readiness --
```

The generated readiness report is deliberately strict. It is the work queue
for corpus admission and stage campaign execution, not a promotional
completion counter.

Resolve the public ENA identities in `corpus-01` before downloading any bytes:

```console
bijux-dna bench acquire-fastq-corpus \
  --spec benchmarks/corpora/human-short-read-diversity.toml \
  --output "$STUDY/data/evidence/acquisition-plans/corpus-01.json" \
  --retrieved-at-utc "$retrieved_at_utc"
```

The resolver uses ENA's run file report and records the exact HTTPS location,
source MD5, byte count, publication dates, layout, study accession, retrieval
time, and source-spec SHA-256 for every file. It refuses layout, study, file
count, size, or digest ambiguity. The resulting plan is acquisition authority,
not admission: downloaded files still require digest verification and a
governed data-catalog record before a comparative campaign can consume them.

For a domain-neutral stage portfolio, resolve each frozen FASTQ collection independently so
distinct species or metagenome labels remain distinct governed catalogs:

```console
bijux-dna bench acquire-selected-fastq-collection \
  --spec benchmarks/corpora/environmental-taxonomy-challenge.toml \
  --collection-id ena-tara-oceans \
  --output "$STUDY/data/evidence/acquisition-plans/$commit/environmental-taxonomy-challenge/ena-tara-oceans.json" \
  --retrieved-at-utc "$retrieved_at_utc"
```

The selected collection must declare FASTQ data and a complete, exact accession list. The
resolver rejects missing or extra ENA rows, mixed scientific names within the collection, unsafe
file identities, and incomplete checksum, size, layout, or archive context.

Materialize the resolved plan under the governed data root:

```console
bijux-dna bench materialize-fastq-corpus \
  --plan "$STUDY/data/evidence/acquisition-plans/$commit/human-short-read-diversity.json" \
  --data-root "$STUDY/data" \
  --repository-commit "$commit" \
  --jobs 4 \
  --retries 2 \
  --timeout-seconds 120 \
  --admitted-at-utc "$admitted_at_utc"
```

For a queued materialization, author one
`bijux.runner.corpus_materialization_slurm_spec.v1` JSON specification. The
typed contract binds the permanent run and attempt identities, clean frozen
repository path and commit, Linux `bijux-dna` executable path and SHA-256,
the exact product-owned `command_arguments`, acquisition plan path and SHA-256,
governed data and evidence roots, scheduler resources, transfer limits, and
admission timestamp. The runner shell-quotes and executes those arguments but
does not construct a domain command. For an idempotent replay
of an admitted corpus, `existing_catalog_sha256` binds the exact governed
`data/catalog/snapshots/<repository-commit>/<portfolio-id>.json` catalog that
must already exist; omit it only for first admission. Legacy flat catalog
authorities remain immutable and are never replaced by materialization.

Prepare immutable scheduler material from that specification, then submit it
exactly once without waiting:

```console
specification="$STUDY/data/evidence/materializations/run-$run_id/specification.json"
bijux-dna bench corpus-materialization prepare \
  --specification "$specification" \
  --submitted-at-utc "$submitted_at_utc"
bijux-dna bench corpus-materialization submit \
  --specification "$specification"
```

Observe one scheduler snapshot without polling:

```console
bijux-dna bench corpus-materialization status \
  --specification "$specification" \
  --observed-at-utc "$observed_at_utc"
```

After the job is terminal, collect scheduler accounting, re-hash the admitted
FASTQ files and catalog, and publish immutable materialization evidence:

```console
bijux-dna bench corpus-materialization collect \
  --specification "$specification" \
  --collected-at-utc "$collected_at_utc"
```

Collection refuses active jobs and existing evidence. A completed Slurm state
is insufficient by itself: catalog identity, file count, byte total, per-file
SHA-256, and positive read counts must all pass before the corpus receives a
passing materialization evidence record.

The generated transfer job invokes only the exact checksum-bound Rust
executable; it runs no Python and no scientific application. FASTQ, BAM, and
VCF stage campaigns remain strictly container-only.

`benchmarks/adapters/materialize_fastq_corpus.py` and
`benchmarks/adapters/corpus_materialization_slurm.py` are argument-only
compatibility adapters. They replace their process with the same `bijux-dna`
commands, use `BIJUX_DNA_BIN` when set, and own no transfer, validation,
scheduler, evidence, or publication decisions.

The materializer streams each file into a private partial path, verifies the
declared compressed byte count and ENA MD5, and atomically publishes it. It
then validates every gzip stream and FASTQ record, records read and base
counts, computes SHA-256, makes the admitted raw files read-only, and writes
the scientific catalog only after the complete cohort passes. Existing bytes
are never trusted by path alone, and an existing catalog is never replaced.

After materialization and format validation, publish source-addressed catalog
registry authority from
`$STUDY/data/evidence/catalog-registries/$commit/registry.json`:

```console
bijux-dna bench catalog-registry publish \
  --data-root "$STUDY/data" \
  --source-commit "$commit"
```

The Rust publisher re-hashes every declared snapshot and verifies its exact
source-addressed path, catalog ID, schema, and digest before publishing
`data/catalog/registries/<source-commit>.json`. Existing equivalent authority
is revalidated without mutation; conflicting authority is rejected.

After materialization and registry publication, publish source-addressed catalog
snapshots and a checked registry that identifies the current v2 snapshot for
each corpus. Build the path-stable scientific data index from that registry:

```console
commit=$(git rev-parse HEAD)
evidence="$STUDY/data/evidence/indexes/$commit"
cargo run -q -p bijux-dna -- bench data-index \
  --catalog-registry "$STUDY/data/catalog/registries/$commit.json" \
  --data-root "$STUDY/data" \
  --output "$evidence/data-index.json" \
  --report "$evidence/data-index.md" \
  --created-at-utc "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
```

The registry retains historical v1 snapshot paths and hashes for replay, but
only a source-addressed v2 snapshot may be current. The `bijux-dna-bench`
authority verifies the registry, re-hashes each selected catalog and every
admitted file, and publishes only `study://data/...` identities. A record remains ineligible when file
integrity, source identity, retrieval or admission time, access classification,
redistribution policy, or known limitations are missing.

[`contracts/stage-campaigns.toml`](contracts/stage-campaigns.toml) assigns stable `run-2001+` campaign
identities and labels each execution as `qualification` or `comparative`.
The numeric identity never upgrades the evidence tier. Every campaign names
its governed corpus, reference, helper
databases, complete retained-tool set, cache policy, repetitions, scientific
question, interpretation, quality metrics, resource metrics, repeated result
artifact, measured input-size fields, and limitations. These declarations keep
the evidence validator stage-neutral: a coverage campaign cannot silently
inherit genotyping outputs or claims. The validator derives each complete tool
set from the canonical container catalog; omitting a retained tool makes the
campaign invalid.

Validate the execution definitions and render their coverage report:

```console
cargo run -q -p bijux-dna-dev -- tooling run benchmark-stage-coverage
```

Use the same command with `-- --check` in gates. Coverage retains the source
commit and checksum of every historical stage report and marks it `stale`
instead of counting it as a current admission.

## Scientific comparison contract

A comparative campaign operates on a stage × corpus × sample × tool matrix.
Every applicable retained tool must run for every eligible sample. An exclusion
is allowed only when it is declared before execution with a scientific or input
compatibility reason; a failed or slow tool is not an exclusion reason.

The common resource surface records wall time, CPU time and utilization, peak
resident memory, filesystem reads and writes, input and output sizes, and input
throughput. One warmup and at least three measured repetitions expose run
variability. Reports preserve per-sample observations before aggregation, so
large files, low-depth libraries, and difficult biological cohorts cannot be
hidden by a global mean.

Scientific correctness is stage-specific. Read transformations require known
signals and retention truth; taxonomy requires real eDNA or mock-community
composition plus immutable databases and negative controls; BAM stages require
diverse real alignments and orthogonal authentication; VCF calling, filtering,
phasing, and imputation require independently curated and masked genotype
truth. Cross-tool consistency is useful integration evidence but is never
renamed accuracy.

Interchangeability is an executable property. Every tool must emit the same
canonical stage contract, and its real output must pass the profile's declared
downstream probe. For example, every aligner output must survive BAM validation,
coverage, filtering, and variant calling; every trimmer output must survive
FASTQ validation, QC, and alignment. A tool can rank well on speed only after
scientific correctness and output compatibility pass.

Execution specifications remain under the governed LUNARC evidence root
because they pin an already committed repository SHA. A benchmark pipeline
sets `run.kind: benchmark`; the execution validator then confines results to
`STUDY/results/benchmarks/run-<id>-<stage>/<tool>`. Each command runs only the
admitted Apptainer image. LUNARC modules or host-installed scientific binaries
are not valid benchmark participants.

The mapping-summary campaign is prepared by `bijux-dna-bench` through the
generic campaign lifecycle rather than an operator-maintained YAML or Python
generator. On LUNARC, create a source-locked run-2004 plan and initialize it,
then use `benchmark-campaign-cycle prepare` or `submit` with the exact frozen
repository, verified data index, canonical image inventory, STUDY root,
account, and partition authorities:

```console
cargo run -p bijux-dna-dev -- tooling run benchmark-campaign-cycle -- prepare \
  --account "$SLURM_ACCOUNT" \
  --benchmark-root "$STUDY/results/benchmarks" \
  --data-index "$STUDY/data/evidence/indexes/$commit/data-index.json" \
  --inventory "$image_inventory" \
  --operator "$USER" \
  --partition "$SLURM_PARTITION" \
  --plan "$campaign_plan" \
  --prepared-at-utc "$prepared_at_utc" \
  --repository-root "$frozen_repository" \
  --study-root "$STUDY"
```

Preparation refuses source-lock drift, an ineligible or digest-mismatched
corpus, a non-current image, incomplete Picard/Samtools closure, or conflicting
attempt evidence. It writes immutable execution specifications and Slurm
scripts under
`$STUDY/results/benchmarks/run-2004-bam-mapping-summary/<tool>/attempts/`.
`submit` returns without polling; later `reconcile` observations and `admit`
calls preserve scheduler accounting and validate the exact output closure.

Measured Apptainer invocations are wrapped by GNU `time` on the Slurm host. The campaign records
wall, user, system, and peak-resident-memory values for the complete isolated container invocation;
tool images do not carry an undeclared timing-package dependency. A missing host measurement binary
fails the job preflight instead of silently dropping resource metrics.

The current BAM-genotyping campaign uses one warmup and three measured warm
cache repetitions. Candidate-site recovery is reported honestly: the Ensembl
panel is independent of the ancient horse sample, but it is not a truth
genotype. A later truth-backed campaign must use masked or independently
validated genotypes before making an accuracy or concordance claim.

The BAM-coverage campaign applies the same repetition and cache policy to the
complete retained closure: Bedtools, Mosdepth, and Samtools. Each tool receives
the same coordinate-sorted BAM and immutable indexes and emits the same compact
coverage-summary contract. Thread counts are declared per tool: Bedtools and
Samtools use their single-threaded coverage implementations, while Mosdepth
uses four threads. The report therefore does not imply concurrency a tool did
not consume. Breadth and mean-depth agreement are integration checks;
predeclared absolute and relative tolerances make disagreement visible instead
of averaging it away. A comparison outside those tolerances fails benchmark
admission while preserving the observations in the report. Without an
independent truth surface the comparison is not an accuracy claim.

The BAM endogenous-content campaign measures primary mapped reads before
duplicate removal with explicit BAM-flag masks. It calls that fraction an
operational proxy: mapping to EquCab3 depends on the reference and upstream
alignment, and contamination can map too. The report must not relabel this
reproducible mapping statistic as biological ground truth.

The BAM mapping-summary campaign compares Picard and Samtools with one
explicit primary-record contract. Picard's `PAIR` and `UNPAIRED` categories
are summed; its `FIRST_OF_PAIR` and `SECOND_OF_PAIR` rows are excluded to avoid
double counting. Samtools supplies the corresponding primary, primary-mapped,
and paired-in-sequencing counts. Exact count agreement is required. The
campaign deliberately allocates one CPU to both tools because these summary
operations do not expose equivalent useful parallelism. Agreement proves that
the integrations interpret BAM flags consistently for this corpus, not that
the upstream alignments are true.

The FASTQ profile campaign is the first comparative real-cohort execution. It
uses all 20 admitted human runs (30 FASTQ files and 3.64 GB compressed input),
all three retained profilers, immutable Apptainer images, one warmup, and three
measured repetitions. Each profiler output is normalized to the same versioned
benchmark schema and must agree exactly with independently recorded admission
counts before a common comparison consumer accepts it. This establishes
replayable comparison interoperability. It does not yet establish operational
pipeline substitution against the full `fastq.profile_reads` stage report, and
the report states that boundary explicitly.

After accounting collection, validate every archived contract, output digest,
input lock, resource row, repetition, and Slurm identity before publishing the
small report:

Terminal benchmark attempts are owned by the benchmark result tree. Failed
attempts are retained under
`$STUDY/results/benchmark-attempts/failed/RUN-TOOL-slurm-JOB/`; reviewed but
invalid attempts are retained under the adjacent `superseded/` root. Each
bundle carries a `bijux.benchmark.attempt_archive.v1` record with the stable run,
attempt, tool, and Slurm identities, original STUDY URIs, and a size/SHA-256
inventory. Container qualification attempts remain under
`$STUDY/containers/evidence/` and must not be mixed into these roots.

Historical benchmark bundles below container evidence are moved only through
the Rust-owned reversible migration workflow. The plan classifies ownership
from each retained pipeline's `run.kind`, inventories every byte, and records
qualification or incomplete material as explicit exclusions. Apply writes an
interruption-safe ledger before moving anything; verify recomputes every size
and SHA-256; rollback verifies each target before restoring its exact source
path. The same plan normalizes the historical benchmark-owned `failures/` root
to the governed singular `failed/` disposition without rewriting any bundle.

```console
migration_root="$STUDY/evidence/migrations/benchmark-attempt-owner-routing"
mkdir -p "$migration_root"

cargo run -p bijux-dna-dev -- hpc run benchmark-attempt-migration -- \
  plan --study-root "$STUDY" \
  --plan "$migration_root/migration-plan.json"

cargo run -p bijux-dna-dev -- hpc run benchmark-attempt-migration -- \
  apply --study-root "$STUDY" \
  --plan "$migration_root/migration-plan.json" \
  --ledger "$migration_root/migration-ledger.json"

cargo run -p bijux-dna-dev -- hpc run benchmark-attempt-migration -- \
  verify --study-root "$STUDY" \
  --plan "$migration_root/migration-plan.json" \
  --ledger "$migration_root/migration-ledger.json" \
  --inventory "$migration_root/after-inventory.json"

# Rollback is available until the migration is accepted.
cargo run -p bijux-dna-dev -- hpc run benchmark-attempt-migration -- \
  rollback --study-root "$STUDY" \
  --plan "$migration_root/migration-plan.json" \
  --ledger "$migration_root/migration-ledger.json"
```

Current campaign execution is owned by the Rust `benchmark-campaign-cycle`
workflow. It plans immutable `run-<id>-<domain>-<stage>` attempt trees,
prepares crate-owned execution material, submits Slurm jobs, reconciles
scheduler observations, and admits only complete result closures. Run
`cargo run -p bijux-dna-dev -- tooling run benchmark-campaign-cycle --` for
the exact command contract. `run-2002` and `run-2003` have executable material
but intentionally have no current report until fresh Slurm attempts pass
reconciliation and admission.

Benchmark execution admission and report publication are owned exclusively by
`bijux-dna-bench`, with operator composition in `bijux-dna-dev`. Historical
reports retain their original source comments as provenance, but no Python
benchmark evidence control plane remains. The superseded BAM coverage and
endogenous-content reports are retained byte-for-byte as
[historical run-2002 evidence](reports/historical/run-2002-bam-coverage.md)
and [historical run-2003 evidence](reports/historical/run-2003-bam-endogenous-content.md).

The checked-in [BAM mapping-summary report](reports/run-2004-bam-mapping-summary.md),
[FASTQ read-length report](reports/run-2042-fastq-profile-read-lengths.md), and
[FASTQ profile report](reports/run-2043-fastq-profile-reads.md) are published
from admitted campaign attempts by crate-owned report workflows. BAM
genotyping uses the same Rust admission and publication authority. A report is
current only when its source, data, image, scheduler, output, science, and
downstream-probe locks validate together. All large run products remain on the
governed study filesystem.
