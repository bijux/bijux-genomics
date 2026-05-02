# Release Runnable Examples

Release-facing runnable examples with governed manifests, expected outputs, caveats, and command paths.

## FASTQ essential QC

| Field | Value |
|---|---|
| Example ID | `fastq_essential_qc` |
| Command path | `cargo run -q -p bijux-dna-dev -- examples run run fastq_essential_qc` |
| Example manifest | `examples/fastq/essential-qc/example.toml` |
| Stage manifest | `examples/fastq/essential-qc/workflow-manifest.json` |
| Expected outputs | `examples/fastq/essential-qc/expected-evidence.json` |
| Caveat surface | `examples/fastq/essential-qc/README.md` (smoke-only bundle expectations and governed stage order) |

## BAM essential alignment QC

| Field | Value |
|---|---|
| Example ID | `bam_essential_alignment_qc` |
| Command path | `cargo run -q -p bijux-dna-dev -- examples run run bam_essential_alignment_qc` |
| Example manifest | `examples/bam/essential-alignment-qc/example.toml` |
| Stage manifest | `examples/bam/essential-alignment-qc/workflow-manifest.json` |
| Expected outputs | `examples/bam/essential-alignment-qc/expected-evidence.json` |
| Caveat surface | `examples/bam/essential-alignment-qc/README.md` (reference preflight and damage/authenticity caveats) |

## VCF essential QC

| Field | Value |
|---|---|
| Example ID | `vcf_essential_qc` |
| Command path | `cargo run -q -p bijux-dna-dev -- examples run run -- vcf_essential_qc` |
| Example manifest | `examples/vcf/essential-qc/example.toml` |
| Stage manifest | `examples/vcf/essential-qc/workflow-manifest.json` |
| Expected outputs | `examples/vcf/essential-qc/expected-evidence.json` |
| Caveat surface | `examples/vcf/essential-qc/README.md` (mini-corpus scope and preflight refusal boundaries) |

## Release bundle and verifier path

After any example run writes a run directory, materialize and verify release-facing evidence:

```sh
cargo run -q -p bijux-dna-analyze --bin bijux-dna-verify -- write-methods <run-dir> [facts.jsonl]
cargo run -q -p bijux-dna-analyze --bin bijux-dna-verify -- write-profile <run-dir> publication_strict [facts.jsonl]
cargo run -q -p bijux-dna-analyze --bin bijux-dna-verify -- verify-profile <run-dir>/profile_bundle_publication_strict.json
cargo run -q -p bijux-dna-analyze --bin bijux-dna-verify -- verify-evidence <run-dir>/evidence_bundle.json
```

Reviewer challenge workflow:

```sh
cargo run -q -p bijux-dna-analyze --bin bijux-dna-verify -- challenge-submit <run-dir> <artifact_id> <evidence_path> <report_field> <caveat> <question> <requested_by>
cargo run -q -p bijux-dna-analyze --bin bijux-dna-verify -- challenge-list <run-dir>
```

## Container publication smoke proof

Promoted GHCR publication workflows attach command-IO smoke evidence artifacts per tool:

- Docker publish workflow artifact path: `artifacts/containers/ghcr/workflow/docker-arm64/<tool>.smoke_proof.json`
- Apptainer publish workflow artifact path: `artifacts/containers/ghcr/workflow/apptainer/<tool>.smoke_proof.json`

These records capture executed smoke commands, expected and actual exit codes, output first lines, and output SHA-256 digests.

## Purpose
This document describes the governed intent and operator-facing meaning of this surface.

## Scope
The scope is limited to repository-owned behavior, contracts, and evidence paths for this topic.

## Non-goals
This document does not redefine source-of-truth schemas, code ownership boundaries, or release policy outside this surface.

## Contracts
Claims here are valid only when they remain consistent with governed configs, domain authorities, and policy checks.

