# STAGE_CONTRACTS

This document is strictly maintained. Update it alongside `tests/contracts/contract_snapshots.rs`.
Manual drift is treated as a contract break.

## Canonical examples
### bam.damage
metrics.json
```json
{"damage_rate":0.12}
```

stage_report.json
```json
{"stage_id":"bam.damage","metrics_path":"metrics.json"}
```
