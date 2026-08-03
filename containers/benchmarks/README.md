# Real-data stage benchmarks

This directory defines publishable container benchmarks. It does not contain
raw outputs or pretend that fixture timing is scientific evidence.

[`campaigns.toml`](campaigns.toml) assigns the stable `run-2001+` campaign
identities. Every campaign names its governed corpus, reference, helper
databases, complete retained-tool set, cache policy, repetitions, scientific
question, quality metrics, resource metrics, and limitations. The validator
derives each complete tool set from the canonical container catalog; omitting a
retained tool makes the campaign invalid.

Validate the definitions and render the honest coverage report:

```console
python3 -B containers/benchmarks/benchmark_registry.py \
  --registry containers/benchmarks/campaigns.toml \
  --catalog containers/contracts/catalog.toml \
  --json-output containers/reports/stage-benchmark-coverage.json \
  --markdown-output containers/reports/stage-benchmark-coverage.md
```

Execution specifications remain under the governed LUNARC evidence root
because they pin an already committed repository SHA. A benchmark pipeline
sets `run.kind: benchmark`; the execution validator then confines results to
`STUDY/results/benchmarks/run-<id>-<stage>/<tool>`. Each command runs only the
admitted Apptainer image. LUNARC modules or host-installed scientific binaries
are not valid benchmark participants.

The current BAM-genotyping campaign uses one warmup and three measured warm
cache repetitions. Candidate-site recovery is reported honestly: the Ensembl
panel is independent of the ancient horse sample, but it is not a truth
genotype. A later truth-backed campaign must use masked or independently
validated genotypes before making an accuracy or concordance claim.
