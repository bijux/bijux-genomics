# DETERMINISM

## What is stable
- Report ordering, section naming, and numeric aggregations are deterministic for the same inputs.
- `report.json` and `report_bundle/index.html` are stable for identical inputs and schema versions.

## What may vary
- Wall-clock timestamps, durations, and similar timing metadata are intentionally unstable.

## Threat model
If input artifacts, schema versions, or metrics registries change, outputs are expected to change.
