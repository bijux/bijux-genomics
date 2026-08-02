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

python3 containers/contracts/container_validation.py render-slurm \
  --governance MANIFEST/config-governance.yml \
  --pipeline MANIFEST/config-pipeline.yml \
  --credentials MANIFEST/config-credential.yml \
  --output MANIFEST/submit.sh

python3 containers/contracts/container_validation.py collect-accounting \
  --manifest MANIFEST
```

The executor validates the current hostname against the site contract, rejects
paths outside the configured study roots, restricts Slurm resources to the
configured account and partitions, checks every immutable digest, and refuses
to overwrite an existing result or accounting record.

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
