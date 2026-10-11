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

Use the executor explicitly:

```console
python3 containers/contracts/container_validation.py freeze-repository \
  --governance containers/contracts/sites/lunarc.yml \
  --commit COMMIT_SHA

python3 containers/contracts/container_validation.py submit \
  --manifest MANIFEST

python3 containers/contracts/container_validation.py collect-accounting \
  --manifest MANIFEST

python3 containers/contracts/container_validation.py archive-failure \
  --manifest MANIFEST
```

The executor validates the current hostname against the site contract, rejects
paths outside the configured study roots, restricts Slurm resources to the
configured account and partitions, checks every immutable digest, and refuses
to overwrite an existing result or accounting record.

`archive-failure` is the only supported way to free canonical submission and
result paths for a corrected attempt with the same stable run identity. It
requires failed terminal evidence and matching accounting, hashes the complete
attempt, and performs same-filesystem moves into the governed failure root.

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

Frontend builds remain in `containers/apptainer/.building` until the admission
command verifies the recipe at the declared Git commit, the completed build
log, and machine-readable Apptainer inspection. It then moves the SIF to an
immutable architecture-and-content-addressed name and writes build evidence:

```console
python3 containers/contracts/container_build.py \
  --governance containers/contracts/sites/lunarc.yml \
  --tool TOOL --version VERSION --repository-commit COMMIT_SHA \
  --definition containers/apptainer/TOOL.def \
  --staged-image STAGED_SIF --build-log BUILD_LOG
```

For an immutable image built before admission automation was available, use
`--immutable-image` instead. Admission then verifies that the existing name
matches its content identity and that any recorded inspection is identical;
it never renames or overwrites the image.
