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

[`portfolio.toml`](portfolio.toml) is the scientific admission authority. It
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
python3 -B benchmarks/benchmark_portfolio.py \
  --portfolio benchmarks/portfolio.toml \
  --registry benchmarks/campaigns.toml \
  --catalog containers/contracts/catalog.toml \
  --repository-root . \
  --json-output benchmarks/reports/scientific-benchmark-readiness.json \
  --markdown-output benchmarks/reports/scientific-benchmark-readiness.md
```

The generated readiness report is deliberately strict. It is the work queue
for corpus admission and stage campaign execution, not a promotional
completion counter.

Resolve the public ENA identities in `corpus-01` before downloading any bytes:

```console
python3 -B benchmarks/corpus_acquisition.py \
  --spec benchmarks/corpora/human-short-read-diversity.toml \
  --output "$STUDY/data/evidence/acquisition-plans/corpus-01.json"
```

The resolver uses ENA's run file report and records the exact HTTPS location,
source MD5, byte count, publication dates, layout, study accession, retrieval
time, and source-spec SHA-256 for every file. It refuses layout, study, file
count, size, or digest ambiguity. The resulting plan is acquisition authority,
not admission: downloaded files still require digest verification and a
governed data-catalog record before a comparative campaign can consume them.

Materialize the resolved plan under the governed data root:

```console
python3 -B benchmarks/corpus_materialization.py \
  --plan "$STUDY/data/evidence/acquisition-plans/$commit/human-short-read-diversity.json" \
  --data-root "$STUDY/data" \
  --jobs 4
```

For a queued materialization, first freeze only the benchmark-owned source at
the exact clean commit:

```console
python3 -B containers/contracts/container_validation.py freeze-repository \
  --governance containers/contracts/sites/lunarc.yml \
  --commit "$commit" \
  --scope sources \
  --path benchmarks
```

The materializer streams each file into a private partial path, verifies the
declared compressed byte count and ENA MD5, and atomically publishes it. It
then validates every gzip stream and FASTQ record, records read and base
counts, computes SHA-256, makes the admitted raw files read-only, and writes
the scientific catalog only after the complete cohort passes. Existing bytes
are never trusted by path alone, and an existing catalog is never replaced.

After materialization and format validation, build the path-stable scientific
data index from the study catalog:

```console
commit=$(git rev-parse HEAD)
evidence="$STUDY/data/evidence/indexes/$commit"
python3 -B benchmarks/data_catalog.py \
  --catalog-root "$STUDY/data/catalog" \
  --data-root "$STUDY/data" \
  --output "$evidence/data-index.json" \
  --report "$evidence/data-index.md"
```

The index re-hashes every admitted file and publishes only `study://data/...`
identities. A record remains ineligible when file integrity, source identity,
retrieval or admission time, access classification, redistribution policy, or
known limitations are missing.

[`campaigns.toml`](campaigns.toml) assigns stable `run-2001+` campaign
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
python3 -B benchmarks/benchmark_registry.py \
  --registry benchmarks/campaigns.toml \
  --catalog containers/contracts/catalog.toml \
  --evidence-root containers/reports \
  --json-output benchmarks/reports/stage-benchmark-coverage.json \
  --markdown-output benchmarks/reports/stage-benchmark-coverage.md
```

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

The mapping-summary campaign has a tracked specification generator rather than
an operator-maintained YAML copy. On LUNARC, first build the verified data
index for the exact clean commit, freeze that commit, and then prepare both
tool contracts from the site policy, data index, and canonical image inventory:

```console
commit=$(git rev-parse HEAD)
index="$STUDY/data/evidence/indexes/$commit/data-index.json"

python3 -B containers/contracts/container_validation.py freeze-repository \
  --governance containers/contracts/sites/lunarc.yml \
  --commit "$commit"

python3 -B benchmarks/mapping_summary_specs.py \
  --governance containers/contracts/sites/lunarc.yml \
  --data-index "$index" \
  --image-inventory containers/reports/image-inventory.json \
  --catalog containers/contracts/catalog.toml
```

The generator refuses a dirty repository, an ineligible corpus, a non-current
image, an incomplete tool closure, or an existing destination. It writes
`config-pipeline.yml` and `config-credential.yml` under the governed
`$STUDY/containers/runs/run-2004-picard/` and `run-2004-samtools/` manifest
directories. Validate each pair with `container_validation.py validate`, then submit with
`container_validation.py submit`. Submission is asynchronous; accounting and
benchmark admission happen only after both jobs reach a terminal state.

Measured Apptainer images include GNU `time` as a non-scientific runtime
dependency. The campaign records wall, user, system, and peak-resident-memory
values inside the same container boundary as the tool; a missing measurement
binary fails evidence instead of silently dropping resource metrics.

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

After accounting collection, validate every archived contract, output digest,
input lock, resource row, repetition, and Slurm identity before publishing the
small report:

```console
python3 -B benchmarks/benchmark_evidence.py \
  --registry benchmarks/campaigns.toml \
  --catalog containers/contracts/catalog.toml \
  --run-id run-2001 \
  --stage-result-root "$STUDY/results/benchmarks/run-2001-bam-genotyping" \
  --json-output benchmarks/reports/run-2001-bam-genotyping.json \
  --markdown-output benchmarks/reports/run-2001-bam-genotyping.md

python3 -B benchmarks/benchmark_evidence.py \
  --registry benchmarks/campaigns.toml \
  --catalog containers/contracts/catalog.toml \
  --run-id run-2002 \
  --stage-result-root "$STUDY/results/benchmarks/run-2002-bam-coverage" \
  --json-output benchmarks/reports/run-2002-bam-coverage.json \
  --markdown-output benchmarks/reports/run-2002-bam-coverage.md

python3 -B benchmarks/benchmark_evidence.py \
  --registry benchmarks/campaigns.toml \
  --catalog containers/contracts/catalog.toml \
  --run-id run-2003 \
  --stage-result-root "$STUDY/results/benchmarks/run-2003-bam-endogenous-content" \
  --json-output benchmarks/reports/run-2003-bam-endogenous-content.json \
  --markdown-output benchmarks/reports/run-2003-bam-endogenous-content.md

python3 -B benchmarks/benchmark_evidence.py \
  --registry benchmarks/campaigns.toml \
  --catalog containers/contracts/catalog.toml \
  --run-id run-2004 \
  --stage-result-root "$STUDY/results/benchmarks/run-2004-bam-mapping-summary" \
  --json-output benchmarks/reports/run-2004-bam-mapping-summary.json \
  --markdown-output benchmarks/reports/run-2004-bam-mapping-summary.md
```

The reviewed [BAM-genotyping report](reports/run-2001-bam-genotyping.md)
records performance and candidate-site observations. The [BAM-coverage
report](reports/run-2002-bam-coverage.md) compares the complete retained
tool closure under their honestly declared concurrency. The [endogenous-content
report](reports/run-2003-bam-endogenous-content.md) publishes the mapping
proxy with its biological claim boundary. All reports keep large run products
on the governed study filesystem.
