<!-- GENERATED FILE - DO NOT EDIT -->
<!-- Regenerate with: cargo run -p bijux-dna-dev -- tooling run generate-docs -->

# UPGRADE_GUIDE

Release: `feat-deep-foundation-compatibility`

Title: Compatibility and upgrade discipline

Source schema: `bijux.release_changes.v1`

## Area Status
- Schemas changed: `true`
- Defaults changed: `false`
- Tools changed: `false`
- Containers changed: `false`
- Evidence expectations changed: `true`
- API changed: `true`
- Error registry changed: `true`

## Changes

### Schemas
- `workflow_manifest`: Legacy v0 workflow manifests upgrade deterministically to bijux.workflow_manifest.v1.
  Migration: Use bijux-dna-core manifest migration helpers before persisting or diffing imported workflow manifests.
  Test: `workflow_manifest_v0_upgrades_deterministically`
- `plan_manifest`: Legacy v0 plan manifests upgrade to bijux.plan_manifest.v1 and recompute the plan fingerprint.
  Migration: Equivalent payloads preserve the v1 fingerprint; frozen legacy fixtures with stale workflow_fingerprint values must explain the expected drift.
  Test: `plan_manifest_v0_upgrade_preserves_equivalent_v1_fingerprint`
- `artifact_inventory`: Runtime readers accept bijux.artifact_inventory.v0 and normalize it to bijux.artifact_inventory.v1.
  Migration: Use read_supported_artifact_inventory for compatibility reads and keep v1 as the write format.
  Test: `artifact_inventory_reader_accepts_supported_legacy_fixture`

### Defaults
No governed changes declared in this release.

### Tools
No governed changes declared in this release.

### Containers
No governed changes declared in this release.

### Evidence Expectations
- `evidence_bundle_profiles`: Draft, operational, certification, and publication evidence lanes now declare required paths and tolerated gaps.
  Migration: Validate bundles against the target lane before certification or publication review.
  Test: `draft_profile_tolerates_missing_publication_material`

### API
- `v1_route_inventory`: The v1 plan, dry-run, execute, and status routes now publish a governed route-version inventory tied to workflow, plan, runtime, and evidence schemas.
  Migration: Review route_version_inventory and refresh the snapshot if a route changes its governed model families.
  Test: `route_version_inventory_schema_is_stable`

### Errors
- `durable_error_registry`: Contract, scientific, runtime, infrastructure, API, and cache errors now have durable ids with reviewed remediation text.
  Migration: Use the governed wire codes and remediation text in operator triage and release notes.
  Test: `error_registry_snapshot_is_stable`
