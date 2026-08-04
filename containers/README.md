# Containers

Authoritative entrypoint:
- [containers/index.md](index.md)

Operational docs index:
- [containers/docs/index.md](docs/index.md)
- [containers/docs/LUNARC_EXECUTION_CONTRACT.md](docs/LUNARC_EXECUTION_CONTRACT.md)
- [containers/reports/README.md](reports/README.md)
- [scientific benchmark authority](../benchmarks/README.md)
- [containers/reports/lunarc-fastq-bam.md](reports/lunarc-fastq-bam.md)
- [containers/reports/lunarc-vcf-phasing.md](reports/lunarc-vcf-phasing.md)
- [containers/reports/lunarc-vcf-relatedness.md](reports/lunarc-vcf-relatedness.md)
- [containers/reports/lunarc-reference-complexity.md](reports/lunarc-reference-complexity.md)
- [containers/reports/lunarc-amplicon-asv.md](reports/lunarc-amplicon-asv.md)
- [containers/docs/EVIDENCE_QUALIFICATION.md](docs/EVIDENCE_QUALIFICATION.md)
- [containers/docs/VERSION_AUTHORITY.md](docs/VERSION_AUTHORITY.md)
- [containers/docs/SCIENCE_EVIDENCE_BOUNDARY.md](docs/SCIENCE_EVIDENCE_BOUNDARY.md)
- [containers/licenses/README.md](licenses/README.md)
- [containers/docs/GHCR_PUBLISH.md](docs/GHCR_PUBLISH.md) (GHCR publication surface)
- `https://github.com/bijux?tab=packages&repo_name=bijux-genomics` (repository packages view)
- [containers/docs/SMOKE_CONTRACT.md](docs/SMOKE_CONTRACT.md) (smoke contract)

Site paths and scheduler policy are configuration, not source-code constants.
The reviewed LUNARC authority is
[`containers/contracts/sites/lunarc.yml`](contracts/sites/lunarc.yml); every
submitted run archives its resolved copy alongside the pipeline and redacted
credential-reference contracts.

## Build identity-governed LUNARC images

Build or reuse Apptainer images only through the Rust environment service. It
derives each build identity from the clean repository commit, catalog, recipe,
source pin, architecture, and declared interface. The generated definition
installs exactly one `/usr/share/bijux/tool-contract.json` and matching labels;
the resulting SIF is rejected before reuse when its digest, embedded contract,
or labels differ.

Run from a clean checkout at the commit being qualified. `--hpc-root` is the
exact root that will own the generated `containers`, corpus, and result paths.
Build the complete retained roster with bounded frontend concurrency:

```console
cargo run -p bijux-dna -- env ensure-images \
  --hpc-root /home/bijan/lu2024-12-24/Dardel/STUDY \
  --all-retained \
  --jobs 4 \
  --force-smoke \
  --repair-mismatch \
  --output /home/bijan/lu2024-12-24/Dardel/STUDY/evidence/image-roster.json \
  --json
```

The command is resumable: an interrupted rerun verifies and reuses every
identity-correct SIF before building missing tools. It emits one report row for
all 85 retained catalog entries and exits nonzero if any row fails. For a
narrow stage-scoped operation, select one stage or a comma-separated set of
stages from the same domain:

```console
cargo run -p bijux-dna -- env ensure-images \
  --hpc-root /home/bijan/lu2024-12-24/Dardel/STUDY \
  --domain bam \
  --stage bam.endogenous_content \
  --force-smoke \
  --json
```

Cached images are verified against the current expected contract before use.
An invalid cached image fails closed; `--repair-mismatch` moves it into the
governed quarantine root before rebuilding. Do not invoke the retired Python
image campaign or admission commands to create or admit images. Reports from
those retired build paths are historical records, not current qualification
evidence.

Image construction runs on the approved frontend. Scientific tool execution
remains a Slurm workload with separate pipeline, input, scheduler, runtime, and
scientific evidence contracts.

## Verify version and help contracts

Image admission proves that a recipe produced an inspectable SIF; it does not
prove that the packaged entry points start correctly. After inventory and
quarantine are complete, run the catalog-wide runtime campaign on the approved
frontend. It executes version and help checks with a disabled runtime network,
an isolated resolvable hostname, bounded command time, and one immutable
stdout/stderr pair per check. The same hostname policy applies to governed
Slurm execution so hostname-sensitive runtimes behave consistently without
restoring network access:

```console
runtime_root="/home/bijan/lu2024-12-24/Dardel/STUDY/containers/evidence/runtime/${image_commit}"
mkdir -p "${runtime_root}"

python3 -B containers/contracts/runtime_verification.py prepare \
  --inventory "${canonical_inventory}" \
  --catalog containers/contracts/catalog.toml \
  --governance containers/contracts/sites/lunarc.yml \
  --repository "${frozen_repository}" \
  --output "${runtime_root}/execution.json"

python3 -B containers/contracts/runtime_verification.py execute \
  --execution "${runtime_root}/execution.json" \
  --governance containers/contracts/sites/lunarc.yml \
  --jobs 8 \
  --timeout-seconds 30
```

The execution record retains the exact frozen repository, catalog, inventory,
image, command, output, and log digests. A passing runtime campaign qualifies
only the version and help dimensions. Real one-sample execution remains a
governed Slurm run, and scientific, benchmark, security, and release claims
remain independent. `--repository` must identify a clean checkout beneath the
governed commit-addressed worktree root whose HEAD matches the inventory
commit; later movement of the active checkout cannot invalidate or silently
change this campaign.

Raw execution JSON keeps the absolute paths needed to audit files at LUNARC;
it is not suitable for a public repository. Publish portable companions before
reviewing or importing tracked reports:

```console
publication_root="${runtime_root}/publication"
mkdir -p "${publication_root}"

python3 -B containers/contracts/image_inventory.py publish \
  --inventory "${canonical_inventory}" \
  --study-root /home/bijan/lu2024-12-24/Dardel/STUDY \
  --repository "$(pwd)" \
  --output "${publication_root}/image-inventory.json"

python3 -B containers/contracts/runtime_verification.py publish \
  --execution "${runtime_root}/execution.json" \
  --study-root /home/bijan/lu2024-12-24/Dardel/STUDY \
  --repository "$(pwd)" \
  --output "${publication_root}/runtime-verification.json"
```

Publication replaces governed absolute roots with `study://` and
`repository://` identifiers and records the SHA-256 of the untouched raw JSON.
It fails when any absolute path lies outside those roots. Qualification accepts
only these portable publication schemas; execution and replay continue to use
the raw site records.

## Prepare governed input data

Container validation does not download or silently replace scientific input.
Admit a file beneath the configured study `data/` root, verify its format with
the domain-appropriate reader, and record its SHA-256 in `config-pipeline.yml`.
For the paired FASTQ corpus used by the current real-run examples:

```console
data_root="/home/bijan/lu2024-12-24/Dardel/STUDY/data/corpora/ena-prjeb10854-err1021817"
gzip -t "${data_root}/ERR1021817.read-prefix-100000_1.fastq.gz"
gzip -t "${data_root}/ERR1021817.read-prefix-100000_2.fastq.gz"
sha256sum "${data_root}/ERR1021817.read-prefix-100000_1.fastq.gz" \
  "${data_root}/ERR1021817.read-prefix-100000_2.fastq.gz"
```

The validator re-hashes every declared input, image, and frozen repository
before submission. A missing file or checksum mismatch is a hard failure; the
executor never substitutes a fixture.

Scientific data selection and admission belong to the root benchmark authority,
not to container runtime contracts. Container runs consume its verified index
instead of encoding knowledge of individual catalog shapes:

```console
study=/home/bijan/lu2024-12-24/Dardel/STUDY
evidence="$study/data/evidence/indexes/$(git rev-parse HEAD)"
python3 -B benchmarks/data_catalog.py \
  --catalog-root "$study/data/catalog" \
  --data-root "$study/data" \
  --output "$evidence/data-index.json" \
  --report "$evidence/data-index.md"
```

The benchmark-owned index recalculates every governed file SHA-256, replaces execution-site
absolute paths with `study://data/...` identities, and reports missing source
or temporal provenance without inventing values. A record is benchmark-eligible
only when its governed files, source identity, retrieval or admission time,
access classification, and limitations are explicit.

## Prepare and submit a real run

Draft specifications are staged under the governed study evidence root. The
repository commit named by the specification must first be frozen into a clean
execution checkout:

```console
run_commit="$(git rev-parse HEAD)"
python3 -B containers/contracts/container_validation.py freeze-repository \
  --governance containers/contracts/sites/lunarc.yml \
  --commit "${run_commit}"

run_spec="/home/bijan/lu2024-12-24/Dardel/STUDY/containers/evidence/specifications/run-1125-trimmomatic"
python3 -B containers/contracts/container_validation.py validate \
  --governance containers/contracts/sites/lunarc.yml \
  --pipeline "${run_spec}/config-pipeline.yml" \
  --credentials "${run_spec}/config-credential.yml"
python3 -B containers/contracts/container_validation.py submit \
  --manifest "${run_spec}" \
  --governance containers/contracts/sites/lunarc.yml
```

To validate every staged specification before submitting any of them, run:

```console
spec_root="/home/bijan/lu2024-12-24/Dardel/STUDY/containers/evidence/specifications"
for run_spec in "${spec_root}"/run-*-*; do
  python3 -B containers/contracts/container_validation.py validate \
    --governance containers/contracts/sites/lunarc.yml \
    --pipeline "${run_spec}/config-pipeline.yml" \
    --credentials "${run_spec}/config-credential.yml"
done
```

Submission remains an explicit per-run action so a malformed or scientifically
inappropriate batch cannot enter the queue merely because it shares a folder.

## Collect results and publish run evidence

After Slurm reaches a terminal state, collection adds immutable scheduler
accounting and preserves failures even when the executor never started:

```console
python3 -B containers/contracts/container_validation.py collect-accounting \
  --manifest "${run_spec}" \
  --governance containers/contracts/sites/lunarc.yml
```

If the collected `evidence.json` is failed, archive that attempt before
correcting and resubmitting the same stable run specification:

```console
python3 -B containers/contracts/container_validation.py archive-failure \
  --manifest "${run_spec}" \
  --governance containers/contracts/sites/lunarc.yml
```

Archival accepts only failed terminal evidence with matching Slurm accounting.
It hashes every submission and result file, verifies that both trees and the
owner-specific archive root share one filesystem, then moves them through a
recoverable partial directory. Qualification failures remain under
`containers/evidence/failures/RUN-TOOL-slurm-JOB/`; benchmark failures use
`results/benchmark-attempts/failed/RUN-TOOL-slurm-JOB/` and the governed
`bijux.benchmark.attempt_archive.v1` record. This frees the canonical submission
and result paths without deleting the failed attempt or allowing it into the
final run index. A passed result cannot be archived by this command.

If a passed attempt later proves scientifically or structurally invalid,
preserve it with an explicit disposition rather than deleting or relabelling
it:

```console
python3 -B containers/contracts/container_validation.py archive-superseded \
  --manifest "${run_spec}" \
  --governance containers/contracts/sites/lunarc.yml \
  --reason "resource measurements did not satisfy the declared TSV evidence format"
```

This command requires passed terminal evidence, matching accounting, and a
substantive reason. It applies the same atomic, same-filesystem preservation
rules under `containers/evidence/superseded/` for qualification attempts and
`results/benchmark-attempts/superseded/` for benchmark attempts, then frees only
the canonical paths for a corrected submission.

Import one completed run by bounding the identifier. For a full replay, use an
empty artifact directory so existing tracked evidence is never overwritten:

```console
result_root="/home/bijan/lu2024-12-24/Dardel/STUDY/results/container-validation"

# One run
python3 -B containers/runs/manage.py import-evidence \
  --result-root "${result_root}" \
  --output-root containers/runs \
  --minimum-run 1125 \
  --maximum-run 1125

# Full replay into disposable review output
replay_root="artifacts/containers/run-evidence-replay"
python3 -B containers/runs/manage.py import-evidence \
  --result-root "${result_root}" \
  --output-root "${replay_root}"

python3 -B containers/runs/manage.py publish-final-evidence \
  --runs-root "${replay_root}" \
  --ledger "${replay_root}/attempt-ledger.json" \
  --apply

# Publish the newly imported tracked run only after previewing the selection
python3 -B containers/runs/manage.py publish-final-evidence \
  --runs-root containers/runs \
  --ledger containers/runs/attempt-ledger.json
python3 -B containers/runs/manage.py publish-final-evidence \
  --runs-root containers/runs \
  --ledger containers/runs/attempt-ledger.json \
  --apply

python3 -B containers/runs/manage.py validate --runs-root containers/runs
python3 -B containers/runs/manage.py render-index \
  --runs-root containers/runs \
  --output containers/reports/run-evidence.json
python3 -B containers/runs/manage.py render-report \
  --runs-root containers/runs \
  --output containers/reports/run-evidence.md
```

The all-runs importer is intentionally non-overwriting. Use an empty artifact
root for complete replay and comparison; use run bounds when adding final
evidence to the repository incrementally.
