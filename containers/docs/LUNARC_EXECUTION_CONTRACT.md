# LUNARC execution contract

Container qualification runs archive three independent YAML contracts. Keeping
them separate prevents site topology, scientific commands, and credential
references from becoming accidental source-code constants.

- `config-governance.yml` identifies the execution site, governed study root,
  active repository, commit-addressed worktree root, and allowed Slurm account
  and partitions.
- `config-pipeline.yml` freezes one tool, image digest, repository commit,
  input digests, commands, outputs, resources, and acceptance rules.
- `config-credential.yml` records credential sources and redaction behavior.
  It never stores credential values.

The reviewed LUNARC site contract is
[`contracts/sites/lunarc.yml`](../contracts/sites/lunarc.yml). Copy it into a
run manifest as `config-governance.yml`; do not rewrite the executor when a
site path changes. The corresponding JSON Schema is
[`contracts/config-governance.schema.json`](../contracts/config-governance.schema.json).

Every submitted manifest contains all three contracts, the rendered submission
script, and the numeric Slurm job ID. The result directory receives the same
contracts, immutable image and input identities, command evidence, outputs,
and terminal Slurm accounting. A run must execute from the commit-addressed
worktree named in its pipeline contract, so later commits cannot change a
queued job.

Qualification runs use the flat
`results/container-validation/run-<id>-<tool>` boundary. A benchmark pipeline
sets `run.kind: benchmark` and writes to
`results/benchmarks/run-<id>-<domain>-<stage>/<tool>`. This keeps each tool's
raw evidence independent while giving one stage campaign a shared comparison
root. The validator derives that path from the declared run, stage, and tool;
changing a path alone cannot turn qualification output into benchmark evidence.

Use the executor explicitly:

```console
cargo run -q -p bijux-dna-dev -- containers run repository-freeze -- \
  --governance containers/contracts/sites/lunarc.yml \
  --commit COMMIT_SHA
cargo build --locked --release -p bijux-dna-dev
operator_program="$(pwd)/artifacts/rust/target/release/bijux-dna-dev"
test -x "${operator_program}"

cargo run -q -p bijux-dna-dev -- containers run qualification-slurm -- prepare \
  GOVERNANCE MANIFEST/config-pipeline.yml MANIFEST/config-credential.yml \
  "${operator_program}" "$(date -u +%Y-%m-%dT%H:%M:%SZ)"

cargo run -q -p bijux-dna-dev -- containers run qualification-slurm -- submit \
  QUALIFICATION_SPEC

cargo run -q -p bijux-dna-dev -- containers run qualification-slurm -- collect \
  QUALIFICATION_SPEC "$(date -u +%Y-%m-%dT%H:%M:%SZ)"

cargo run -q -p bijux-dna-dev -- containers run qualification-slurm -- archive-failure \
  QUALIFICATION_SPEC "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
```

The Rust preparation and runner boundary validates the current hostname against the site contract, rejects
paths outside the configured study roots, restricts Slurm resources to the
configured account and partitions, checks every immutable digest, and refuses
to overwrite an existing scheduler or accounting record. Submission returns
immediately; observation and collection perform one scheduler query per call.

`archive-failure` is the only supported way to free canonical submission and
result paths for a corrected attempt with the same stable run identity. It
requires failed terminal evidence and matching accounting, hashes the complete
attempt, and performs same-filesystem moves into the governed failure root.

A scheduler-passed attempt whose evidence is later found scientifically or
structurally invalid must not be relabelled as failed or deleted. Archive it
with a substantive reason before correcting and resubmitting the stable run:

```console
python3 containers/contracts/container_validation.py archive-superseded \
  --manifest MANIFEST \
  --reason "resource measurements did not satisfy the declared TSV evidence format"
```

This preserves the immutable submission and result trees under the governed
owner root, records their hashes and disposition, and frees only their canonical
execution paths. Qualification attempts remain under
`$STUDY/containers/evidence/{failures,superseded}`. Benchmark attempts are
isolated under `$STUDY/results/benchmark-attempts/{failed,superseded}` and use
the benchmark-owned attempt archive contract.

Historical owner correction must use `hpc run benchmark-attempt-migration`.
Direct `mv`, copy-and-delete, and path-name classification are not admissible:
the governed workflow derives ownership from the retained pipeline contract,
binds its plan to a SHA-256 ledger, verifies every source and target file, and
retains an exact rollback path.

Current pipeline contracts separate host storage from paths visible inside the
contained runtime. Governed input files mount as unique, safe filenames beneath
`/input`; the declared filename retains format and compression suffixes such as
`.bam` or `.fastq.gz`. The run result directory mounts at `/result`. Commands
must use those stable mount paths. This prevents a successful host-side
preflight followed by a missing file inside `--containall`, and makes commands
independent of LUNARC backend mount aliases.

Slurm scratch is also site policy. LUNARC supplies node-local storage through
`SNIC_TMP` (and sets `TMPDIR` to the same filesystem), so the site contract
declares `slurm.scratch_environment: SNIC_TMP` instead of encoding a frontend
`/tmp` path. The renderer requires an absolute existing value on the allocated
node, creates a job-unique child, verifies the requested capacity, binds it at
`/scratch`, exports container `TMPDIR=/scratch`, and removes only that exact
`mktemp` child when the job exits. A different site may instead declare one
fixed absolute `slurm.scratch_root`; declaring both authorities is invalid.
See the official [LUNARC local-disk guidance](https://lunarc-documentation.readthedocs.io/en/latest/manual/submitting_jobs/manual_local_disk/).

Frontend image builds use the Rust environment service. It derives a build
identity from the clean repository commit, catalog, recipe, source pin,
architecture, and interface; installs one embedded tool contract with matching
labels; and verifies that identity before the SIF can be reused:

```console
cargo run -p bijux-dna -- env ensure-images \
  --hpc-root /home/bijan/lu2024-12-24/Dardel/STUDY \
  --tool bijux_dna \
  --json
```

The transaction publishes only after exact-commit source revalidation,
embedded-contract inspection, and digest verification. An identity mismatch,
occupied staging path, or immutable destination conflict fails closed. The
retired Python image campaign and cache-oriented compatibility commands must
not create current image state.
