# ADD_FIXTURE

## Goal
Add a minimal, representative fixture that supports deterministic tests.

## Steps
1. Pick the smallest artifact that still exercises the contract.
2. Place it under `tests/fixtures/<crate>/<purpose>/`.
3. Ensure the fixture name matches the artifact type (e.g., `report.json`).
4. Update the crate's `docs/TESTS.md` with the fixture → contract mapping.
5. Add or update the test that loads the fixture and asserts stability.

## Rules
- No empty directories under `tests/fixtures/**`.
- Fixtures must be deterministic; avoid timestamps or random fields.
- If the fixture must include timestamps, document the allowed variability.
