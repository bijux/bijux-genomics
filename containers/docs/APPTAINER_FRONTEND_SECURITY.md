# Apptainer Frontend Security & Supply Chain Enforcement

Purpose: enforce frontend-only SBOM, vulnerability, licensing, pinning, secret, and network disclosure controls for Apptainer SIF artifacts.

[FRONTEND_BUILD_AUTHORITY.md](FRONTEND_BUILD_AUTHORITY.md),
[../licenses/README.md](../licenses/README.md), and
[../../docs/50-reference/LICENSING.md](../../docs/50-reference/LICENSING.md)
define the adjacent control surfaces this frontend security gate depends on.

## Workflow
- Run:
  - `cargo run -p bijux-dna-dev -- containers run run-apptainer-frontend-security`
- Validate gate:
  - `cargo run -p bijux-dna-dev -- containers run check-apptainer-frontend-security`

## Controls
- SBOM generation for every Apptainer runtime tool SIF.
- SBOM artifact linked to SIF digest and written in `security_summary.json`.
- Vulnerability scan on frontend host with `grype` or `trivy` when available.
- Critical CVE fail gate with allowlist:
  - [configs/ci/tools/vuln_allowlist.toml](../../configs/ci/tools/vuln_allowlist.toml)
- License metadata contract:
  - [containers/licenses/README.md](../licenses/README.md) governs the
    per-tool `*.license.toml` files, which must exist with non-empty SPDX.
- Base image and pinning checks:
  - `cargo run -p bijux-dna-dev -- containers run check-version-hash-pin`
  - `cargo run -p bijux-dna-dev -- containers run check-apptainer-hardening`
- Secret scanning:
  - `cargo run -p bijux-dna-dev -- containers run check-no-secrets`
- Network disclosure enforcement:
  - `cargo run -p bijux-dna-dev -- containers run check-network-disclosure`

## Outputs
- `artifacts/containers/hpc/frontend-security/<run_id>/security_summary.json`
- `artifacts/containers/hpc/frontend-security/<run_id>/sbom_index.json`
- [containers/docs/APPTAINER_FRONTEND_SECURITY_SUMMARY.md](APPTAINER_FRONTEND_SECURITY_SUMMARY.md)
