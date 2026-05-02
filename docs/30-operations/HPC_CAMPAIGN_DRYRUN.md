# HPC Campaign Dry Run and Preflight

This runbook describes the campaign planning layer for HPC benchmark execution.

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
  --user-overrides configs/hpc/campaign/user.override.toml
```

Run campaign dry-run:

```bash
bijux-dna config campaign-dry-run \
  --config configs/hpc/campaign/lunarc-small.toml \
  --env-file configs/hpc/.env \
  --user-overrides configs/hpc/campaign/user.override.toml
```

Print JSON reports:

```bash
bijux-dna config campaign-preflight --json --config configs/hpc/campaign/lunarc-small.toml
bijux-dna config campaign-dry-run --json --config configs/hpc/campaign/lunarc-small.toml
```

## Security notes

- Do not commit Slurm account/project values in campaign config files.
- Keep env files private (`0600` on Unix-like hosts).
- Use `security.env_file` plus local override files for user-specific settings.

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
