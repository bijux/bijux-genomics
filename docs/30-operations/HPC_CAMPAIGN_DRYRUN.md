# HPC Campaign Dry Run and Preflight

This runbook describes the campaign planning layer for HPC benchmark execution.

## Purpose

- Define deterministic preflight and dry-run behavior before HPC submissions.
- Provide operator-safe commands for campaign planning, validation, and encrypted artifact handling.

## Scope

- Covers campaign profile generation, preflight checks, dry-run planning, submission entrypoints, and bundle operations.
- Applies to `configs/hpc/campaign/*.toml` profiles and optional policy/env inputs.

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

Generate baseline campaign profiles:

```bash
bijux-dna config write-campaign-profiles --out-dir configs/hpc/campaign
```

Run campaign preflight:

```bash
bijux-dna config campaign-preflight \
  --config configs/hpc/campaign/lunarc-small.toml \
  --env-file configs/hpc/.env \
  --user-overrides configs/hpc/campaign/user.policy.toml
```

Run campaign dry-run:

```bash
bijux-dna config campaign-dry-run \
  --config configs/hpc/campaign/lunarc-small.toml \
  --env-file configs/hpc/.env \
  --user-overrides configs/hpc/campaign/user.policy.toml
```

Print JSON reports:

```bash
bijux-dna config campaign-preflight --json --config configs/hpc/campaign/lunarc-small.toml
bijux-dna config campaign-dry-run --json --config configs/hpc/campaign/lunarc-small.toml
```

Submit a single stage benchmark (mock mode):

```bash
bijux-dna slurm submit-stage-benchmark \
  --config configs/hpc/campaign/lunarc-small.toml \
  --stage fastq.validate_reads \
  --mock-submit
```

Submit one domain benchmark set:

```bash
bijux-dna slurm submit-domain-benchmark \
  --config configs/hpc/campaign/lunarc-small.toml \
  --domain fastq \
  --mock-submit
```

Submit a cross-domain subset:

```bash
bijux-dna slurm submit-cross-benchmark \
  --config configs/hpc/campaign/lunarc-small.toml \
  --domains fastq,bam \
  --mock-submit
```

Submit a whole campaign:

```bash
bijux-dna slurm submit-campaign \
  --config configs/hpc/campaign/lunarc-small.toml \
  --mock-submit
```

Write a copy-back manifest:

```bash
bijux-dna slurm copy-back-manifest \
  --config configs/hpc/campaign/lunarc-small.toml
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
  --config configs/hpc/campaign/cross-mini.toml \
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
  --profile configs/hpc/campaign/sharing/collaborator-a.toml \
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
