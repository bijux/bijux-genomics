# Governed container runs

This directory is the version-controlled replay and evidence layer for LUNARC
container executions. A run ID is a permanent audit identity assigned at
submission, not a delivery milestone or a claim of scientific validity.

Each admitted directory uses `run-<id>-<tool>` and contains:

- the exact non-secret pipeline and credential-reference contracts;
- the execution and Slurm-accounting records when they exist; and
- `evidence.lock.json`, which authenticates the tracked files and inventories
  every file retained at the execution site without copying large results into
  Git.

The lock identifies its source result relative to the governed study root.
Absolute paths remain site configuration or immutable historical pipeline
evidence; generated indexes do not expose LUNARC backend mount aliases.

Execution success, scientific acceptance, benchmark qualification, and release
qualification are separate claims. This directory publishes one final passing
execution per represented tool. Superseded and failed attempts remain in the
checksummed [`attempt-ledger.json`](attempt-ledger.json) and in raw LUNARC
results; they are not presented as current evidence.

The site-specific filesystem and Slurm settings remain in
[`../contracts/sites/lunarc.yml`](../contracts/sites/lunarc.yml). Secret values
are forbidden. `config-credential.yml` may name a protected credential source,
but it must never contain a credential value.

## Validate tracked evidence

From the repository root:

```console
python3 containers/runs/manage.py validate \
  --runs-root containers/runs
```

Render the compact website/report index after validation:

```console
python3 containers/runs/manage.py render-index \
  --runs-root containers/runs \
  --output containers/reports/run-evidence.json

python3 containers/runs/manage.py render-report \
  --runs-root containers/runs \
  --output containers/reports/run-evidence.md
```

## Admit final execution evidence

Run the importer only against the governed LUNARC result root. Use a bounded
range so one reviewable scientific or operational group is admitted per
commit:

```console
python3 containers/runs/manage.py import-evidence \
  --result-root /home/bijan/lu2024-12-24/Dardel/STUDY/results/container-validation \
  --output-root containers/runs \
  --minimum-run 1001 \
  --maximum-run 1018
```

The importer refuses to replace an existing tracked run, rejects embedded
credential-value fields, checks run/tool identity across paths and contracts,
and records hashes for the complete remote result. Before rendering reports,
preview and then publish the latest successful execution per tool:

```console
python3 containers/runs/manage.py publish-final-evidence \
  --runs-root containers/runs \
  --ledger containers/runs/attempt-ledger.json

python3 containers/runs/manage.py publish-final-evidence \
  --runs-root containers/runs \
  --ledger containers/runs/attempt-ledger.json \
  --apply
```

Preview reports the exact final and superseded counts without changing files.
Apply writes the ledger before removing superseded tracked copies. Raw outputs,
SIFs, and logs remain in the study tree. Repeated publication extends the
ledger and repoints a tool's earlier attempts to its newly admitted final run.

The historical nested result layout (`<tool>/run-<id>`) is accepted for
admission only. New results use the flat `run-<id>-<tool>` identity so a run is
one independently movable and inspectable result directory.

## Submit a current run

An unexecuted specification belongs under the governed execution-site path
`STUDY/containers/evidence/specifications/run-<id>-<tool>`, not in the Git
checkout. This avoids an impossible self-reference in which a tracked pipeline
would need to contain the commit that contains itself. The specification pins
an already frozen repository commit, immutable images and inputs, stable
container mount names, predefined metrics, and non-secret credential
references.

Validate and submit it on LUNARC:

```console
specification=/home/bijan/lu2024-12-24/Dardel/STUDY/containers/evidence/specifications/run-1122-trim_galore

python3 containers/contracts/container_validation.py validate \
  --governance containers/contracts/sites/lunarc.yml \
  --pipeline "${specification}/config-pipeline.yml" \
  --credentials "${specification}/config-credential.yml"

python3 containers/contracts/container_validation.py submit \
  --manifest "${specification}" \
  --governance containers/contracts/sites/lunarc.yml
```

Submission copies the resolved contracts and generated Slurm script to the
immutable operational evidence directory. After the job becomes terminal,
collect scheduler accounting and import the completed flat result into this
directory. Only completed or failed evidence is tracked; provisional files do
not masquerade as a replay record.

## Migrate historical result paths

The layout migration verifies every result file against the tracked evidence
lock before changing a path. It writes a durable journal after every atomic
rename and supports verification and rollback. It never copies or deletes a
result. On LUNARC:

```console
python3 containers/runs/result_layout.py plan \
  --runs-root containers/runs \
  --result-root /home/bijan/lu2024-12-24/Dardel/STUDY/results/container-validation \
  --plan /home/bijan/lu2024-12-24/Dardel/STUDY/results/container-validation/.governance/result-layout.json

python3 containers/runs/result_layout.py apply \
  --runs-root containers/runs \
  --result-root /home/bijan/lu2024-12-24/Dardel/STUDY/results/container-validation \
  --plan /home/bijan/lu2024-12-24/Dardel/STUDY/results/container-validation/.governance/result-layout.json

python3 containers/runs/result_layout.py verify \
  --runs-root containers/runs \
  --result-root /home/bijan/lu2024-12-24/Dardel/STUDY/results/container-validation \
  --plan /home/bijan/lu2024-12-24/Dardel/STUDY/results/container-validation/.governance/result-layout.json
```

`rollback` accepts the same arguments and restores the recorded nested paths.
The journal is execution-site evidence and remains outside Git.
