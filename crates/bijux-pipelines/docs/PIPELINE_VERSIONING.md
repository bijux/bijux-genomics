# Pipeline Versioning and Deprecation Policy

Pipeline IDs follow `{input}-to-{output}__{profile}__v{n}`.

Rules:
1. `__v1` profiles are comparable within the same major version. Changes within a version must
   not invalidate existing metrics or report sections.
2. Any breaking change to stages, required outputs, or report sections requires a new version
   (`__v2`, `__v3`, ...).
3. Deprecations must keep the previous version registered for at least one release cycle.
4. Migration notes must be recorded in release notes and, when possible, in the defaults ledger
   provenance assumptions.
5. Experimental or beta profiles must be explicitly named and must not replace stable IDs.

## Blessed Pipelines (Canonical Entrypoints)

These IDs are the single-source canonical entrypoints for each scope target:

- `fastq-to-fastq__default__v1`
- `fastq-to-bam__default__v1`
- `bam-to-bam__adna_shotgun__v1`

Use these IDs when documenting commands, examples, and default run recipes.
