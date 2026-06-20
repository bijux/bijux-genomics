# HPC Campaign Dry Run and Preflight

This runbook describes the campaign planning layer for HPC benchmark execution.

## Purpose

- Define deterministic preflight and dry-run behavior before HPC submissions.
- Provide operator-safe commands for campaign planning, validation, and encrypted artifact handling.

## Scope

- Covers campaign profile generation, preflight checks, dry-run planning, submission entrypoints, and bundle operations.
- Applies to `benchmarks/configs/hpc/campaign/*.toml` profiles and optional policy/env inputs.

## Non-goals

- Does not replace scientific stage/tool quality contracts.
- Does not document cluster-specific scheduler administration outside campaign command inputs.
- Does not permit committing secret-bearing env files.

## Contracts

- `campaign-preflight` and `campaign-dry-run` outputs must be deterministic for fixed inputs.
- Security-sensitive values (account/project/env secrets) must remain redacted in reports.
- User policy overrides must remain explicit, file-backed, and opt-in.

## Goals covered

- Shared layout contract for corpora, databases, images, scratch, logs, and encrypted bundles.
- Confidential env-file loading for Slurm account/project resolution.
- Site profile resolution and user-local override support.
- Preflight checks before submission.
- Dry-run job expansion with deterministic output paths.

## Commands

Local asset staging dry-run before cluster transfer planning:

```bash
make bench-hpc-asset-staging-dry-run
```

Render only the governed manifest of staged benchmark inputs:

```bash
make bench-hpc-asset-staging-render
```

Validate an existing staged-input manifest against the current governed all-domain command slice:

```bash
make bench-hpc-asset-staging-validate
```

Render the governed scratch-path, input-link, output-root, and cleanup layout for the same future
HPC jobs:

```bash
make bench-hpc-scratch-layout-render
```

Validate an existing scratch layout against the current governed submit manifest and command
surfaces:

```bash
make bench-hpc-scratch-layout-validate
```

Render the governed execution-resolution surface for the selected future HPC jobs:

```bash
make bench-hpc-execution-resolver-render
```

Validate an existing execution resolver against the current submit manifest, runtime probes, and
Apptainer conversion map:

```bash
make bench-hpc-execution-resolver-validate
```

Render the governed dependency simulation report that proves failed jobs block only descendants
while sibling branches and unrelated benchmark work continue:

```bash
make bench-hpc-dependency-simulation-render
```

Validate an existing dependency simulation report against the current governed HPC job graph:

```bash
make bench-hpc-dependency-simulation-validate
```

Render the governed benchmark-result SLURM array and its per-index manifest:

```bash
make bench-hpc-stage-benchmark-array-render
```

Validate an existing benchmark-result SLURM array against the current selected jobs, scratch
layout, and execution resolver:

```bash
make bench-hpc-stage-benchmark-array-validate
```

Render the governed essential-pipeline-node SLURM array and its dependency manifest:

```bash
make bench-hpc-pipeline-node-array-render
```

Validate an existing essential-pipeline-node SLURM array against the current selected jobs,
validated pipeline DAGs, and scratch layout:

```bash
make bench-hpc-pipeline-node-array-validate
```

Call the underlying CLI directly when a non-default manifest path is needed:

```bash
bijux-dna bench local render-hpc-asset-staging-manifest \
  --output runs/bench/hpc-dry-run/asset-staging-manifest.json

bijux-dna bench local validate-hpc-asset-staging-manifest \
  --manifest runs/bench/hpc-dry-run/asset-staging-manifest.json

bijux-dna bench local render-hpc-scratch-layout \
  --output runs/bench/hpc-dry-run/scratch-layout.json

bijux-dna bench local validate-hpc-scratch-layout \
  --manifest runs/bench/hpc-dry-run/scratch-layout.json

bijux-dna bench local render-hpc-execution-resolver \
  --output runs/bench/hpc-dry-run/execution-resolver.tsv

bijux-dna bench local validate-hpc-execution-resolver \
  --manifest runs/bench/hpc-dry-run/execution-resolver.tsv

bijux-dna bench local render-hpc-dependency-simulation \
  --output runs/bench/hpc-dry-run/slurm-dependency-simulation.json

bijux-dna bench local validate-hpc-dependency-simulation \
  --manifest runs/bench/hpc-dry-run/slurm-dependency-simulation.json

bijux-dna bench local render-hpc-stage-benchmark-array \
  --output runs/bench/hpc-dry-run/slurm/stage-benchmark-array.sbatch

bijux-dna bench local validate-hpc-stage-benchmark-array \
  --script runs/bench/hpc-dry-run/slurm/stage-benchmark-array.sbatch

bijux-dna bench local render-hpc-pipeline-node-array \
  --output runs/bench/hpc-dry-run/slurm/pipeline-node-array.sbatch

bijux-dna bench local validate-hpc-pipeline-node-array \
  --script runs/bench/hpc-dry-run/slurm/pipeline-node-array.sbatch
```

Generate baseline campaign profiles:

```bash
bijux-dna config write-campaign-profiles --out-dir benchmarks/configs/hpc/campaign
```

Run campaign preflight:

```bash
bijux-dna config campaign-preflight \
  --config benchmarks/configs/hpc/campaign/lunarc-small.toml \
  --env-file configs/hpc/.env \
  --user-overrides benchmarks/configs/hpc/campaign/user.policy.toml
```

Run campaign dry-run:

```bash
bijux-dna config campaign-dry-run \
  --config benchmarks/configs/hpc/campaign/lunarc-small.toml \
  --env-file configs/hpc/.env \
  --user-overrides benchmarks/configs/hpc/campaign/user.policy.toml
```

Print JSON reports:

```bash
bijux-dna config campaign-preflight --json --config benchmarks/configs/hpc/campaign/lunarc-small.toml
bijux-dna config campaign-dry-run --json --config benchmarks/configs/hpc/campaign/lunarc-small.toml
```

Submit a single stage benchmark (mock mode):

```bash
bijux-dna slurm submit-stage-benchmark \
  --config benchmarks/configs/hpc/campaign/lunarc-small.toml \
  --stage fastq.validate_reads \
  --mock-submit
```

Submit one domain benchmark set:

```bash
bijux-dna slurm submit-domain-benchmark \
  --config benchmarks/configs/hpc/campaign/lunarc-small.toml \
  --domain fastq \
  --mock-submit
```

Submit a cross-domain subset:

```bash
bijux-dna slurm submit-cross-benchmark \
  --config benchmarks/configs/hpc/campaign/lunarc-small.toml \
  --domains fastq,bam \
  --mock-submit
```

Submit a whole campaign:

```bash
bijux-dna slurm submit-campaign \
  --config benchmarks/configs/hpc/campaign/lunarc-small.toml \
  --mock-submit
```

Write a copy-back manifest:

```bash
bijux-dna slurm copy-back-manifest \
  --config benchmarks/configs/hpc/campaign/lunarc-small.toml
```

Verify encrypted bundle integrity:

```bash
bijux-dna slurm verify-bundle \
  --bundle /shared/bijux/results/fastq-hpc-mini/fastq/fastq.validate_reads/seqkit_v2/sample_0001/dryrun-0001-1700000000.results
```

Decrypt one encrypted bundle into a private local directory:

```bash
bijux-dna slurm decrypt-bundle \
  --bundle /shared/bijux/results/fastq-hpc-mini/fastq/fastq.validate_reads/seqkit_v2/sample_0001/dryrun-0001-1700000000.results \
  --out-dir artifacts/investigation/decrypt
```

Re-encrypt an existing bundle for rotated recipients:

```bash
bijux-dna slurm rewrap-bundle \
  --bundle /shared/bijux/results/fastq-hpc-mini/.../dryrun-0001-1700000000.results \
  --recipient age1newrecipientxxxxxxxxxxxxxxxxxxxx \
  --out-bundle /shared/bijux/results/fastq-hpc-mini/.../dryrun-0001-1700000000.results.rotated
```

Import one encrypted results/code pair for replay validation:

```bash
bijux-dna slurm import-replay \
  --results-bundle /shared/bijux/results/fastq-hpc-mini/.../dryrun-0001-1700000000.results \
  --code-bundle /shared/bijux/code/fastq-hpc-mini/.../dryrun-0001-1700000000.code \
  --out-dir artifacts/investigation/replay
```

Import a copied campaign directory and decode all available pairs:

```bash
bijux-dna slurm import-campaign \
  --campaign-dir artifacts/investigation/campaign-copy \
  --out-dir artifacts/investigation/campaign-import
```

Export a minimal encrypted failure package for one benchmark row:

```bash
bijux-dna slurm export-failure-bundle \
  --config benchmarks/configs/hpc/campaign/cross-mini.toml \
  --stage fastq.validate_reads \
  --tool seqkit_v2 \
  --sample sample_0001 \
  --recipient age1collaboratorxxxxxxxxxxxxxxxxxxx \
  --out-dir artifacts/investigation/failure-export
```

Share an encrypted bundle with a collaborator profile:

```bash
bijux-dna slurm share-bundle \
  --bundle /shared/bijux/results/fastq-hpc-mini/.../dryrun-0001-1700000000.results \
  --profile benchmarks/configs/hpc/campaign/sharing/collaborator-a.toml \
  --out-dir artifacts/investigation/shared
```

Verify results/code completeness and appraiser-output encryption policy:

```bash
bijux-dna slurm verify-results-policy \
  --results-bundle /shared/bijux/results/fastq-hpc-mini/.../dryrun-0001-1700000000.results \
  --code-bundle /shared/bijux/code/fastq-hpc-mini/.../dryrun-0001-1700000000.code
```

## Security notes

- Do not commit Slurm account/project values in campaign config files.
- Keep env files private (`0600` on Unix-like hosts).
- Use `security.env_file` plus local override files for user-specific settings.
- Keep `security.encrypt_operator_outputs = false` unless log/out/err files must be encrypted.
- Use `security.encryption_backend = "age-cli"` with valid `age1...` recipients and
  configured `security.encryption_identity_files` for real key-based encryption.
- Decrypt/import commands refuse unsafe destination directories by default. Use
  `--allow-unsafe-destination` only for explicit audited exceptions.

## Output path tokens

Supported placeholders in output templates:

- `{job_id}`
- `{timestamp}`
- `{campaign}`
- `{domain}`
- `{stage}`
- `{tool}`
- `{sample}`
- `{array_task}`

Required placeholders for every template:

- `{job_id}`
- `{timestamp}`
- `{campaign}`
- `{domain}`
- `{stage}`
- `{tool}`
- `{sample}`

## Dependency model

- `[[jobs]]` can declare `name` and `depends_on = ["job_name"]`.
- When omitted, the scheduler layer still enforces in-sample ordering by default.
- Generated Slurm scripts include `--dependency=afterok:...` when dependencies resolve.
