use crate::model::ops::{NativeOpsCommandKey, OpsCommandDefinition, OpsCommandSpec};

#[must_use]
pub fn assets_registry() -> Vec<OpsCommandDefinition> {
    vec![
        native(
            "refresh-golden",
            "Regenerate deterministic toy-run golden bundles under assets/golden.",
            NativeOpsCommandKey::AssetsRefreshGolden,
        ),
        native(
            "refresh-toy",
            "Regenerate deterministic toy fixtures under assets/toy.",
            NativeOpsCommandKey::AssetsRefreshToy,
        ),
        native(
            "validate-reference",
            "Validate governed reference bank schema contracts under assets/reference.",
            NativeOpsCommandKey::AssetsValidateReference,
        ),
    ]
}

#[must_use]
pub fn docs_registry() -> Vec<OpsCommandDefinition> {
    vec![
        native(
            "check-doc-assets",
            "Validate that docs image assets live under docs/assets.",
            NativeOpsCommandKey::DocsCheckDocAssets,
        ),
        native(
            "check-doc-depth",
            "Validate required narrative sections across docs markdown.",
            NativeOpsCommandKey::DocsCheckDocDepth,
        ),
        native(
            "check-doc-links",
            "Validate internal docs links and publication references.",
            NativeOpsCommandKey::DocsCheckDocLinks,
        ),
        native(
            "check-doc-root-layout",
            "Validate the governed docs root layout.",
            NativeOpsCommandKey::DocsCheckDocRootLayout,
        ),
        native(
            "check-docs-graph",
            "Validate docs reachability and the generated docs graph.",
            NativeOpsCommandKey::DocsCheckDocsGraph,
        ),
        native(
            "check-domain-doc-references",
            "Validate docs stage and tool references against governed registries.",
            NativeOpsCommandKey::DocsCheckDomainDocReferences,
        ),
        native(
            "check-generated-docs",
            "Validate generated docs headers and deterministic rendered output.",
            NativeOpsCommandKey::DocsCheckGeneratedDocs,
        ),
        native(
            "check-no-placeholder-language",
            "Reject placeholder language in governed docs.",
            NativeOpsCommandKey::DocsCheckNoPlaceholderLanguage,
        ),
        native(
            "check-root-pollution",
            "Reject forbidden repo-root outputs.",
            NativeOpsCommandKey::DocsCheckRootPollution,
        ),
        native(
            "check-doc-major-depth",
            "Validate required sections in major top-level docs.",
            NativeOpsCommandKey::DocsCheckDocMajorDepth,
        ),
    ]
}

#[must_use]
pub fn examples_registry() -> Vec<OpsCommandDefinition> {
    vec![
        native(
            "generate-index",
            "Generate examples/index.yaml from example contracts.",
            NativeOpsCommandKey::ExamplesGenerateIndex,
        ),
        native(
            "check-index",
            "Validate generated examples index output.",
            NativeOpsCommandKey::ExamplesCheckIndex,
        ),
        native(
            "run",
            "Run a deterministic example bundle flow.",
            NativeOpsCommandKey::ExamplesRun,
        ),
        native(
            "check-drift",
            "Validate that an example matches its golden outputs.",
            NativeOpsCommandKey::ExamplesCheckDrift,
        ),
    ]
}

#[must_use]
pub fn hpc_registry() -> Vec<OpsCommandDefinition> {
    vec![
        native(
            "validate-frontend-constraints",
            "Validate HPC frontend host and disk constraints.",
            NativeOpsCommandKey::HpcValidateFrontendConstraints,
        ),
        native(
            "run-frontend-mini-e2e",
            "Run the governed frontend mini end-to-end flow.",
            NativeOpsCommandKey::HpcRunFrontendMiniE2e,
        ),
        native(
            "lunarc-pull",
            "Pull governed Lunarc outputs into a local evidence directory.",
            NativeOpsCommandKey::HpcLunarcPull,
        ),
        native(
            "lunarc-push",
            "Push the governed repository context to Lunarc.",
            NativeOpsCommandKey::HpcLunarcPush,
        ),
    ]
}

#[must_use]
pub fn lab_registry() -> Vec<OpsCommandDefinition> {
    vec![
        native(
            "run-bench",
            "Run the governed FASTQ and BAM benchmark commands.",
            NativeOpsCommandKey::LabRunBench,
        ),
        native(
            "run-pipelines",
            "Run configured lab pipelines against the configured corpus.",
            NativeOpsCommandKey::LabRunPipelines,
        ),
    ]
}

#[must_use]
pub fn smoke_registry() -> Vec<OpsCommandDefinition> {
    vec![
        native(
            "run",
            "Dispatch a local smoke target.",
            NativeOpsCommandKey::SmokeRun,
        ),
        native(
            "smoke-bam",
            "Run the BAM smoke contract.",
            NativeOpsCommandKey::SmokeBam,
        ),
        native(
            "smoke-fastq",
            "Run the FASTQ smoke contract.",
            NativeOpsCommandKey::SmokeFastq,
        ),
    ]
}

#[must_use]
pub fn test_registry() -> Vec<OpsCommandDefinition> {
    vec![
        native(
            "test-scripts-smoke",
            "Probe the native control-plane command surface.",
            NativeOpsCommandKey::TestScriptsSmoke,
        ),
        native(
            "test-triage",
            "Bucket test failures from a text log.",
            NativeOpsCommandKey::TestTriage,
        ),
        native(
            "reproduce-failure",
            "Render reproduction commands from a nextest JSONL log.",
            NativeOpsCommandKey::TestReproduceFailure,
        ),
        native(
            "fastq-gold-repro",
            "Validate deterministic FASTQ toy-run outputs across repeated runs.",
            NativeOpsCommandKey::TestFastqGoldRepro,
        ),
        native(
            "toy-runs",
            "Run deterministic toy-run golden helpers.",
            NativeOpsCommandKey::TestToyRuns,
        ),
    ]
}

fn native(id: &str, summary: &str, key: NativeOpsCommandKey) -> OpsCommandDefinition {
    OpsCommandDefinition {
        id: id.to_string(),
        summary: summary.to_string(),
        command: OpsCommandSpec::Native { key },
    }
}
