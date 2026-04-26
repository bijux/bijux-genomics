# FIXTURES

Reference fixtures live under `tests/fixtures/` and are used by contract and interpretation tests.
Each fixture is intentionally small but represents a real-world scenario.

## Reference fixtures
- `tests/fixtures/reference/default/authentic_adna.json`
  Scenario: expected aDNA damage/authenticity patterns.
  Purpose: baseline for authenticity interpretation.

- `tests/fixtures/reference/default/modern_contaminated.json`
  Scenario: modern contamination signal.
  Purpose: contamination interpretation and thresholds.

- `tests/fixtures/reference/default/low_complexity.json`
  Scenario: low library complexity.
  Purpose: complexity + coverage sufficiency interpretation.

- `tests/fixtures/reference/default/kinship_pair_a.json`
  Scenario: kinship candidate A.
  Purpose: kinship sufficiency + overlap requirements.

- `tests/fixtures/reference/default/kinship_pair_b.json`
  Scenario: kinship candidate B.
  Purpose: kinship sufficiency + overlap requirements.

## Tool output fixtures
- `tests/fixtures/bam/default/damageprofiler.json`
  Scenario: damage profiler tool output.
  Purpose: parser + schema stability.

- `tests/fixtures/bam/default/pydamage.json`
  Scenario: pydamage output.
  Purpose: parser + schema stability.

- `tests/fixtures/bam/default/contamination.json`
  Scenario: contamination tool output.
  Purpose: parser + schema stability.

- `tests/fixtures/bam/default/sex.json`
  Scenario: sex inference tool output.
  Purpose: parser + schema stability.

- `tests/fixtures/tool_metrics/default/*.json`
  Scenario: tool-specific downstream metrics.
  Purpose: parser + typed model stability.
