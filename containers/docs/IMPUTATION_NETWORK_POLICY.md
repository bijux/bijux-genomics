# Imputation Network Policy

Purpose: explicit "no implicit network at runtime" contract for VCF downstream imputation/phasing stack.

Scope:
- `glimpse`, `impute5`, `minimac4`, `shapeit5`, `beagle`, `eagle`, `bcftools`, `plink2`.

Authority surfaces:
- [../README.md](../README.md)
- [../index.md](../index.md)
- [NETWORK_USAGE.md](NETWORK_USAGE.md)
- [IMPUTATION_RUNTIME_CONSTRAINTS.md](IMPUTATION_RUNTIME_CONSTRAINTS.md)
- [SECURITY_BOUNDARY.md](SECURITY_BOUNDARY.md)

Runtime policy:
- Runtime network access is prohibited for all tools in this set.
- Any required downloads must be handled by explicit acquisition workflows before execution.
- Runtime wrappers must not fetch references/panels/maps implicitly.

Build policy:
- Build-time network access is allowed only for pinned, checksummed upstream assets.
- Upstream source, version, and checksums must be recorded in registry/license metadata.

Enforcement:
- `cargo run -p bijux-dna-dev -- containers run check-network-disclosure`
- `cargo run -p bijux-dna-dev -- containers run check-imputation-network-policy`
