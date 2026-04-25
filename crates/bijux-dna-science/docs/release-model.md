# Release Model

Science releases are manifest-driven and immutable after cut.

A release manifest selects authored claims and bindings by typed ID. The release command writes:

- release metadata JSON
- compiled evidence outputs
- index JSON

under `artifacts/science-releases/<release_id>/`.
