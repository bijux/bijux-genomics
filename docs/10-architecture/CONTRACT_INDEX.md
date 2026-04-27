# Contract Index

Owner: Architecture
Scope: Repository-wide contract authority index
Last reviewed: 2026-04-26
Contract version: v1

## Purpose
List the authoritative document for each architecture contract category.

## Index
| Contract category | Authority |
| --- | --- |
| root architecture map | [docs/10-architecture/ARCHITECTURE_CONTRACT.md](ARCHITECTURE_CONTRACT.md) |
| crate boundaries | [docs/10-architecture/CRATE_BOUNDARY_CONTRACTS.md](CRATE_BOUNDARY_CONTRACTS.md) |
| dependency boundaries | [docs/10-architecture/BOUNDARY_MAP.md](BOUNDARY_MAP.md) |
| crate responsibility map | [docs/10-architecture/CRATE_AUTHORITY_MAP.md](CRATE_AUTHORITY_MAP.md) |
| contract artifact spine | [docs/10-architecture/CONTRACT_SPINE.md](CONTRACT_SPINE.md) |
| domain transition boundary | [docs/10-architecture/SSOT.md](SSOT.md) |
| generated files boundary | [docs/10-architecture/GENERATED_FILES_CONTRACT.md](GENERATED_FILES_CONTRACT.md) |
| dry-run side-effect boundary | [docs/10-architecture/DRY_RUN_EFFECTS_CONTRACT.md](DRY_RUN_EFFECTS_CONTRACT.md) |
| snapshot/golden boundary | [docs/10-architecture/SNAPSHOT_GOLDEN_CONTRACT.md](SNAPSHOT_GOLDEN_CONTRACT.md) |
| fixture boundary | [docs/40-policies/TESTS_STYLE.md](../40-policies/TESTS_STYLE.md) |
| stage-id boundary | [docs/10-architecture/SSOT.md](SSOT.md) |
| tool-id boundary | [containers/docs/TOOL_IDS_CONTRACT.md](../../containers/docs/TOOL_IDS_CONTRACT.md) |
| profile-id boundary | [docs/10-architecture/CONTRACT_AUTHORITY.md](CONTRACT_AUTHORITY.md) |
| metrics envelope boundary | [docs/10-architecture/CONTRACT_SPINE.md](CONTRACT_SPINE.md) |
| run manifest schema boundary | [docs/10-architecture/CONTRACT_SPINE.md](CONTRACT_SPINE.md) |
| stage report schema boundary | [docs/10-architecture/CONTRACT_SPINE.md](CONTRACT_SPINE.md) |
| report schema boundary | [docs/30-operations/REPORT_CONTRACT.md](../30-operations/REPORT_CONTRACT.md) |
| tool invocation boundary | [docs/10-architecture/CONTRACT_SPINE.md](CONTRACT_SPINE.md) |
| effective config boundary | [docs/10-architecture/CONTRACT_AUTHORITY.md](CONTRACT_AUTHORITY.md) |
| asset contract boundary | [assets/CONTRACT.md](../../assets/CONTRACT.md) |
| container contract boundary | [containers/docs/SMOKE_CONTRACT.md](../../containers/docs/SMOKE_CONTRACT.md) |
| science evidence boundary | [containers/docs/SCIENCE_EVIDENCE_BOUNDARY.md](../../containers/docs/SCIENCE_EVIDENCE_BOUNDARY.md) |
| HPC boundary | [docs/30-operations/benchmark/workspace-contract.md](../30-operations/benchmark/workspace-contract.md) |
| network boundary | [docs/10-architecture/ARCHITECTURE_CONTRACT.md](ARCHITECTURE_CONTRACT.md) |
| privacy boundary | [docs/40-policies/TESTS_STYLE.md](../40-policies/TESTS_STYLE.md) |
| error taxonomy boundary | [docs/10-architecture/CONTRACT_SPINE.md](CONTRACT_SPINE.md) |
| waiver boundary | [containers/docs/SMOKE_CONTRACT.md](../../containers/docs/SMOKE_CONTRACT.md) |
| threshold boundary | [docs/10-architecture/CONTRACT_AUTHORITY.md](CONTRACT_AUTHORITY.md) |
| resource estimate boundary | [docs/10-architecture/CONTRACT_AUTHORITY.md](CONTRACT_AUTHORITY.md) |
| comparability boundary | [docs/20-science/SCIENTIFIC_DECISIONS.md](../20-science/SCIENTIFIC_DECISIONS.md) |
| license boundary | [docs/50-reference/LICENSING.md](../50-reference/LICENSING.md) |
| deprecation boundary | [docs/50-reference/CONTRACT_COMPATIBILITY.md](../50-reference/CONTRACT_COMPATIBILITY.md) |
| release artifact boundary | [.github/release.env](../../.github/release.env) |
| GHCR publication boundary | [.github/workflows/publish-ghcr-container-images.yml](../../.github/workflows/publish-ghcr-container-images.yml) |
| reference hydration boundary | [assets/reference/LOCK.md](../../assets/reference/LOCK.md) |
| cache boundary | [docs/30-operations/benchmark/workspace-contract.md](../30-operations/benchmark/workspace-contract.md) |
| schema migration boundary | [docs/50-reference/CONTRACT_VERSIONING.md](../50-reference/CONTRACT_VERSIONING.md) |
| status promotion boundary | [docs/10-architecture/CONTRACT_AUTHORITY.md](CONTRACT_AUTHORITY.md) |
| review ownership boundary | [docs/40-policies/POLICY_OWNERSHIP.md](../40-policies/POLICY_OWNERSHIP.md) |

## Validation commands
- `CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-policies --test contracts contract_index_policy`
