# SCHEMA

## Canonical JSON example
```json
{
  "schema_version": "bijux.report.v1",
  "run_id": "run-123",
  "stages": [],
  "sections": {}
}
```

## Field semantics
- run_id: unique run identifier
- stages: ordered stage summaries
- sections: report sections keyed by name

## Final artifact bundle
- report.json
- report.html
- summary.tsv
