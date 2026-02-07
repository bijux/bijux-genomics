# STAGE_CONTRACTS

## Symmetry
Symmetry is enforced at the contract level (observable inputs/outputs), not file naming.

## Canonical examples
### fastq.trim
metrics.json
```json
{"reads_in":100,"reads_out":95,"retention":0.95}
```

stage_report.json
```json
{"stage_id":"fastq.trim","metrics_path":"metrics.json"}
```
