# RUNBOOK

## Repro steps (local docker)
```
make image-qa PLATFORM=linux/amd64
```

## Expected outputs
- artifacts under `target/qa/`
- `manifest.json`
- `report.json`

## Offline defaults
Offline mode is default. Network-enabled QA must be explicitly enabled.
See `OFFLINE_POLICY.md`.

## Expected runtime
< 5 minutes on local docker for the default fixture set.
