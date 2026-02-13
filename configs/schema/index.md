# configs/schema

## What
Configuration files for the schema domain.

## Philosophy
Keep schema configuration scoped to this directory so ownership is explicit and drift is easy to detect.

## Rules
- Schema evolution policy lives in `configs/schema/CONFIG_SCHEMA_RULES.md`.

## Notes
Schema-oriented descriptors or schema governance inputs should be placed here rather than mixed into CI/runtime directories.
