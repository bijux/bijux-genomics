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
qualification are separate claims. Importing an old run never upgrades its
claim: historical records default to `scientific_status: not_assessed`, and
failed attempts remain visible.

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
```

## Admit execution-site history

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
and records hashes for the complete remote result. Raw outputs, SIFs, and logs
remain in the study tree.

The historical nested result layout (`<tool>/run-<id>`) is accepted for
admission only. New results use the flat `run-<id>-<tool>` identity so a run is
one independently movable and inspectable result directory.
