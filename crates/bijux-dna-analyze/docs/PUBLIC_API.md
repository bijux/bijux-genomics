# Public API

## Crate Root
The crate root is intentionally small:

- `analyze_run`: the canonical analysis entrypoint
- `pub use public_api::*;`: the curated stable surface

`src/lib.rs` must not grow direct compatibility shims for every helper. New stable exports belong
in `src/public_api/` so the public surface remains reviewable.

## Stable Surface
`src/public_api/mod.rs` re-exports:

- request and response models from `src/api/`
- versioned contract helpers from `src/contracts/`
- report, load, aggregate, failure, and export helpers intended for reuse
- decision comparison and ranking helpers under explicit namespaces
- metric semantics helpers shared with downstream consumers

## Internal Modules
Some modules remain public for existing tests and downstream integration points. Treat those as
compatibility surfaces, not permission to bypass ownership boundaries:

- `aggregate` owns metric schemas and aggregation helpers.
- `decision` owns comparison, scoring, ranking, and traces.
- `exports` owns summary and dashboard artifact writers, evidence composition and
  verification, profile bundles, and reviewer challenge journals.
- `failure` owns failure classification and hints.
- `load` owns artifact loading and optional SQLite/parquet readers.
- `model` owns typed internal analysis records.
- `report` owns report construction and rendering.

## Evidence Export Surface

The existing `exports` namespace exposes `build_evidence_bundle`,
`write_evidence_bundle_json`, `verify_evidence_bundle`, and `compare_evidence_bundles`
with their versioned bundle, artifact, source, health, timeline, provenance, and
comparison DTOs. Manifest-declared artifact names, paths, optional hashes, sorted
order, and duplicate handling belong to this analysis projection. An earlier
`evidence_bundle.json` publication is not inferred as a source artifact when the
manifest declares no artifacts. Explicit manifest declarations keep their existing
projection and verification semantics.

`validate_evidence_bundle_profile` accepts a typed bundle and
`EvidenceBundleProfileV1` without acquisition. `write_methods_summary_json`,
`write_profile_bundle_json`, and `verify_profile_bundle` compose the existing
methods/profile contracts with source acquisition, file digests, and publication.
`submit_reviewer_challenge` and `list_reviewer_challenges` use
`ReviewerChallengeRequestV1` and `ReviewerChallengeRecordV1` to maintain a separate
review journal tied to inventory, report, evidence, and caveat references. These
exports interpret completed artifacts; they do not own workflow execution or
scheduler state.

## Change Rule
Adding or removing a stable item requires:

- updating this document when the intended surface changes
- updating `tests/snapshots/bijux-dna-analyze__schemas__public_api.txt`
- running the public API snapshot through `docs/COMMANDS.md`
