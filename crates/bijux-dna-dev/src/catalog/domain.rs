use crate::model::domain::{DomainCommandDefinition, DomainCommandSpec, NativeDomainCommandKey};

#[allow(clippy::too_many_lines)]
pub fn domain_registry() -> Vec<DomainCommandDefinition> {
    vec![
        native(
            "check-default-settings-docs",
            "Validate default-settings documentation coverage for every domain.",
            NativeDomainCommandKey::CheckDefaultSettingsDocs,
        ),
        native(
            "check-doc-links",
            "Validate relative links in domain documentation.",
            NativeDomainCommandKey::CheckDocLinks,
        ),
        native(
            "check-domain-index",
            "Validate generated domain indexes and their completeness contracts.",
            NativeDomainCommandKey::CheckDomainIndex,
        ),
        native(
            "check-domain-layout",
            "Validate the governed domain tree layout.",
            NativeDomainCommandKey::CheckDomainLayout,
        ),
        native(
            "check-domain-schema",
            "Validate domain schemas, payload contracts, and production fixtures.",
            NativeDomainCommandKey::CheckDomainSchema,
        ),
        native(
            "check-domain-tool-metadata",
            "Validate required metadata for domain tool declarations.",
            NativeDomainCommandKey::CheckDomainToolMetadata,
        ),
        native(
            "check-external-tool-policy",
            "Validate the external tool allowlist against domain fixtures.",
            NativeDomainCommandKey::CheckExternalToolPolicy,
        ),
        native(
            "check-fixture-contracts",
            "Validate fixture file contracts and fixture README coverage.",
            NativeDomainCommandKey::CheckFixtureContracts,
        ),
        native(
            "check-inventory",
            "Validate deterministic domain inventory generation.",
            NativeDomainCommandKey::CheckInventory,
        ),
        native(
            "check-orphan-files",
            "Validate that domain stage and tool files stay referenced.",
            NativeDomainCommandKey::CheckOrphanFiles,
        ),
        native(
            "check-planner-fixture-coverage",
            "Validate planner stages have matching domain fixtures.",
            NativeDomainCommandKey::CheckPlannerFixtureCoverage,
        ),
        native(
            "check-planner-stage-coverage",
            "Validate supported domain stages stay represented in planner configs.",
            NativeDomainCommandKey::CheckPlannerStageCoverage,
        ),
        native(
            "check-reference-bundle-lock",
            "Validate reference bundle lock hashes.",
            NativeDomainCommandKey::CheckReferenceBundleLock,
        ),
        native(
            "check-rust-stage-catalog-parity",
            "Validate Rust stage catalogs against domain indexes.",
            NativeDomainCommandKey::CheckRustStageCatalogParity,
        ),
        native(
            "check-shared-tools",
            "Validate shared-tool declarations across domains.",
            NativeDomainCommandKey::CheckSharedTools,
        ),
        native(
            "check-ssot-authority",
            "Validate SSOT documentation and domain version markers.",
            NativeDomainCommandKey::CheckSsotAuthority,
        ),
        native(
            "check-tool-container-parity",
            "Validate domain tool declarations against container definitions.",
            NativeDomainCommandKey::CheckToolContainerParity,
        ),
        native(
            "generate-index",
            "Regenerate domain indexes from stage and tool declarations.",
            NativeDomainCommandKey::GenerateIndex,
        ),
        native(
            "generate-inventory",
            "Generate domain inventory JSON and Markdown reports.",
            NativeDomainCommandKey::GenerateInventory,
        ),
        native(
            "inventory-drift",
            "Compare domain inventory against registry and code references.",
            NativeDomainCommandKey::InventoryDrift,
        ),
        native(
            "lock-registry",
            "Regenerate the registry lock hash and marker.",
            NativeDomainCommandKey::LockRegistry,
        ),
        native(
            "validate",
            "Run the governed domain validation surface.",
            NativeDomainCommandKey::Validate,
        ),
    ]
}

fn native(
    id: &'static str,
    summary: &'static str,
    key: NativeDomainCommandKey,
) -> DomainCommandDefinition {
    DomainCommandDefinition {
        id: id.to_string(),
        summary: summary.to_string(),
        command: DomainCommandSpec::Native { key },
    }
}
