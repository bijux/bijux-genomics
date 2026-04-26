# Dry Run

## Contract
Dry-run commands must plan and render without executing tools. They may create declared planning
artifacts only when the command contract says so; they must not spawn processes or contact networks.

## Operator Surface
The main FASTQ dry-run surface is:

```text
bijux-dna run preprocess --dry-run --r1 reads.fastq.gz --out artifacts --sample-id SAMPLE
```

VCF and BAM commands also expose `--dry-run` where their parser contract supports it, but the CLI
must still delegate planning and execution semantics behind API/domain-owned boundaries.

## Stable Inputs
- CLI arguments
- repository configs under `configs/`
- governed domain/assets material needed for planning
- explicit environment variables documented by the selected command

## Stable Outputs
- deterministic terminal or JSON output
- declared plan/manifest/graph artifacts
- stable error categories when planning is refused

## Non-Goals
- Tool execution
- container selection side effects beyond API planning evidence
- hidden writes used only for debugging

## Verification
- `tests/contracts/dry_run.rs`
- `tests/contracts/dry_run/`
- help and output snapshots under `tests/snapshots/`
