# Real-data stage benchmarks

This directory defines publishable container benchmarks. It does not contain
raw outputs or pretend that fixture timing is scientific evidence.

[`campaigns.toml`](campaigns.toml) assigns the stable `run-2001+` campaign
identities. Every campaign names its governed corpus, reference, helper
databases, complete retained-tool set, cache policy, repetitions, scientific
question, interpretation, quality metrics, resource metrics, repeated result
artifact, measured input-size fields, and limitations. These declarations keep
the evidence validator stage-neutral: a coverage campaign cannot silently
inherit genotyping outputs or claims. The validator derives each complete tool
set from the canonical container catalog; omitting a retained tool makes the
campaign invalid.

Validate the definitions and render the honest coverage report:

```console
python3 -B containers/benchmarks/benchmark_registry.py \
  --registry containers/benchmarks/campaigns.toml \
  --catalog containers/contracts/catalog.toml \
  --evidence-root containers/reports \
  --json-output containers/reports/stage-benchmark-coverage.json \
  --markdown-output containers/reports/stage-benchmark-coverage.md
```

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

python3 -B containers/benchmarks/mapping_summary_specs.py \
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
python3 -B containers/benchmarks/benchmark_evidence.py \
  --registry containers/benchmarks/campaigns.toml \
  --catalog containers/contracts/catalog.toml \
  --run-id run-2001 \
  --stage-result-root "$STUDY/results/benchmarks/run-2001-bam-genotyping" \
  --json-output containers/reports/run-2001-bam-genotyping.json \
  --markdown-output containers/reports/run-2001-bam-genotyping.md

python3 -B containers/benchmarks/benchmark_evidence.py \
  --registry containers/benchmarks/campaigns.toml \
  --catalog containers/contracts/catalog.toml \
  --run-id run-2002 \
  --stage-result-root "$STUDY/results/benchmarks/run-2002-bam-coverage" \
  --json-output containers/reports/run-2002-bam-coverage.json \
  --markdown-output containers/reports/run-2002-bam-coverage.md

python3 -B containers/benchmarks/benchmark_evidence.py \
  --registry containers/benchmarks/campaigns.toml \
  --catalog containers/contracts/catalog.toml \
  --run-id run-2003 \
  --stage-result-root "$STUDY/results/benchmarks/run-2003-bam-endogenous-content" \
  --json-output containers/reports/run-2003-bam-endogenous-content.json \
  --markdown-output containers/reports/run-2003-bam-endogenous-content.md

python3 -B containers/benchmarks/benchmark_evidence.py \
  --registry containers/benchmarks/campaigns.toml \
  --catalog containers/contracts/catalog.toml \
  --run-id run-2004 \
  --stage-result-root "$STUDY/results/benchmarks/run-2004-bam-mapping-summary" \
  --json-output containers/reports/run-2004-bam-mapping-summary.json \
  --markdown-output containers/reports/run-2004-bam-mapping-summary.md
```

The reviewed [BAM-genotyping report](../reports/run-2001-bam-genotyping.md)
records performance and candidate-site observations. The [BAM-coverage
report](../reports/run-2002-bam-coverage.md) compares the complete retained
tool closure under their honestly declared concurrency. The [endogenous-content
report](../reports/run-2003-bam-endogenous-content.md) publishes the mapping
proxy with its biological claim boundary. All reports keep large run products
on the governed study filesystem.
