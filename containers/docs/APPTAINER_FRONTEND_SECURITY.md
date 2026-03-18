# Apptainer Frontend Security & Supply Chain Enforcement

Purpose: enforce frontend-only SBOM, vulnerability, licensing, pinning, secret, and network disclosure controls for Apptainer SIF artifacts.

## Workflow
- Run:
  - `./scripts/containers/run-apptainer-frontend-security.sh`
- Validate gate:
  - `cargo run -p bijux-dev-dna -- containers run check-apptainer-frontend-security`

## Controls
- SBOM generation for every Apptainer runtime tool SIF.
- SBOM artifact linked to SIF digest and written in `security_summary.json`.
- Vulnerability scan on frontend host with `grype` or `trivy` when available.
- Critical CVE fail gate with allowlist:
  - `configs/ci/tools/vuln_allowlist.toml`
- License metadata contract:
  - `containers/licenses/<tool>.license.toml` must exist with non-empty SPDX.
- Base image and pinning checks:
  - `cargo run -p bijux-dev-dna -- containers run check-version-hash-pin`
  - `cargo run -p bijux-dev-dna -- containers run check-apptainer-hardening`
- Secret scanning:
  - `scripts/containers/check-no-secrets.sh`
- Network disclosure enforcement:
  - `cargo run -p bijux-dev-dna -- containers run check-network-disclosure`

## Outputs
- `artifacts/containers/hpc/frontend-security/<run_id>/security_summary.json`
- `artifacts/containers/hpc/frontend-security/<run_id>/sbom_index.json`
- `containers/docs/APPTAINER_FRONTEND_SECURITY_SUMMARY.md`
