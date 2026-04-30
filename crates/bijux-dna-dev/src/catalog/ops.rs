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
            "refresh-reference",
            "Regenerate governed reference asset documentation under assets/reference.",
            NativeOpsCommandKey::AssetsRefreshReference,
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
        native("run", "Run a deterministic example bundle flow.", NativeOpsCommandKey::ExamplesRun),
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
            "benchmark-sync-pull",
            "Pull governed benchmark-environment outputs into the local mirror.",
            NativeOpsCommandKey::HpcBenchmarkSyncPull,
        ),
        native(
            "benchmark-sync-push",
            "Push the governed repository context for the benchmark environment.",
            NativeOpsCommandKey::HpcBenchmarkSyncPush,
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
        native("run", "Dispatch a local smoke target.", NativeOpsCommandKey::SmokeRun),
        native("smoke-bam", "Run the BAM smoke contract.", NativeOpsCommandKey::SmokeBam),
        native("smoke-fastq", "Run the FASTQ smoke contract.", NativeOpsCommandKey::SmokeFastq),
    ]
}

#[must_use]
pub fn test_registry() -> Vec<OpsCommandDefinition> {
    vec![
        native(
            "test-control-plane-smoke",
            "Probe the native control-plane command surface.",
            NativeOpsCommandKey::TestControlPlaneSmoke,
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

#[must_use]
#[allow(clippy::too_many_lines)]
pub fn tooling_registry() -> Vec<OpsCommandDefinition> {
    vec![
        native(
            "cargo-targets",
            "Run governed cargo test and nextest target bundles.",
            NativeOpsCommandKey::ToolingCargoTargets,
        ),
        native(
            "check-config-snapshot",
            "Validate configs/schema/config_tree.snapshot against the governed config tree.",
            NativeOpsCommandKey::ToolingCheckConfigSnapshot,
        ),
        native(
            "check-config-paths",
            "Validate config path literals referenced across governed source roots.",
            NativeOpsCommandKey::ToolingCheckConfigPaths,
        ),
        native(
            "ci-audit",
            "Run the governed advisory audit gate.",
            NativeOpsCommandKey::ToolingCiAudit,
        ),
        native(
            "ci-clippy",
            "Run workspace clippy through the native control plane.",
            NativeOpsCommandKey::ToolingCiClippy,
        ),
        native(
            "ci-clippy-executors",
            "Run clippy on runner and executor crates through the native control plane.",
            NativeOpsCommandKey::ToolingCiClippyExecutors,
        ),
        native(
            "ci-coverage",
            "Run the governed coverage workflow through the native control plane.",
            NativeOpsCommandKey::ToolingCiCoverage,
        ),
        native(
            "ci-fast",
            "Run the fast CI make profile through the native control plane.",
            NativeOpsCommandKey::ToolingCiFast,
        ),
        native(
            "ci-fmt",
            "Run rustfmt through the native control plane.",
            NativeOpsCommandKey::ToolingCiFmt,
        ),
        native(
            "ci-install-tools",
            "Install required CI cargo tools.",
            NativeOpsCommandKey::ToolingCiInstallTools,
        ),
        native(
            "ci-slow",
            "Run the slow CI make profile through the native control plane.",
            NativeOpsCommandKey::ToolingCiSlow,
        ),
        native(
            "ci-test",
            "Run governed nextest suites through the native control plane.",
            NativeOpsCommandKey::ToolingCiTest,
        ),
        native(
            "ci-test-slow",
            "Run governed slow nextest suites through the native control plane.",
            NativeOpsCommandKey::ToolingCiTestSlow,
        ),
        native(
            "clean-docs",
            "Remove generated docs artifacts under the governed docs artifact root.",
            NativeOpsCommandKey::ToolingCleanDocs,
        ),
        native(
            "certification-gate",
            "Run the governed local certification gate bundle.",
            NativeOpsCommandKey::ToolingCertificationGate,
        ),
        native(
            "certify-level1",
            "Generate the Level 1 completion certificate after the essential release gate passes.",
            NativeOpsCommandKey::ToolingCertifyLevel1,
        ),
        native(
            "certify-all",
            "Generate the cross-domain certification bundle.",
            NativeOpsCommandKey::ToolingCertifyAll,
        ),
        native(
            "certify-bam",
            "Run the BAM certification bundle slice.",
            NativeOpsCommandKey::ToolingCertifyBam,
        ),
        native(
            "certify-domains",
            "Run governed domain certification and emit the certification bundle.",
            NativeOpsCommandKey::ToolingCertifyDomains,
        ),
        native(
            "certify-fastq",
            "Run the FASTQ certification bundle slice.",
            NativeOpsCommandKey::ToolingCertifyFastq,
        ),
        native(
            "certify-vcf",
            "Run the VCF certification bundle slice.",
            NativeOpsCommandKey::ToolingCertifyVcf,
        ),
        native(
            "acquire-maps",
            "Materialize or lock governed recombination map assets.",
            NativeOpsCommandKey::ToolingAcquireMaps,
        ),
        native(
            "acquire-panels",
            "Materialize or lock governed panel assets.",
            NativeOpsCommandKey::ToolingAcquirePanels,
        ),
        native(
            "acquire-reference",
            "Materialize or lock governed reference assets.",
            NativeOpsCommandKey::ToolingAcquireReference,
        ),
        native(
            "reference-external-data",
            "Run governed reference and external-data scenario suite for goals G171-G180.",
            NativeOpsCommandKey::ToolingReferenceExternalData,
        ),
        native(
            "scientific-caveat-propagation",
            "Run governed scientific-caveat propagation scenario suite for goals G181-G190.",
            NativeOpsCommandKey::ToolingScientificCaveatPropagation,
        ),
        native(
            "architecture-report",
            "Generate a compact workspace architecture drift report under artifacts/architecture.",
            NativeOpsCommandKey::ToolingArchitectureReport,
        ),
        native(
            "benchmark-smoke-level1",
            "Measure smoke-only duration and artifact sizes for the canonical Level 1 examples.",
            NativeOpsCommandKey::ToolingBenchmarkSmokeLevel1,
        ),
        native(
            "benchmark-integrity-mini",
            "Run the optional frontend mini benchmark integrity helper over bijux-dna bench fastq.",
            NativeOpsCommandKey::ToolingBenchmarkIntegrityMini,
        ),
        native(
            "config-inventory",
            "Generate governed config inventory artifacts under artifacts/.",
            NativeOpsCommandKey::ToolingConfigInventory,
        ),
        native(
            "coverage-summary",
            "Summarize llvm-cov JSON output with optional baseline and threshold checks.",
            NativeOpsCommandKey::ToolingCoverageSummary,
        ),
        native(
            "crash-triage",
            "Print likely crash causes from a crash provenance JSON artifact.",
            NativeOpsCommandKey::ToolingCrashTriage,
        ),
        native(
            "deprecate-vcf-knob",
            "Append a governed VCF knob deprecation entry.",
            NativeOpsCommandKey::ToolingDeprecateVcfKnob,
        ),
        native(
            "deprecate-vcf-panel",
            "Append a governed VCF panel deprecation entry.",
            NativeOpsCommandKey::ToolingDeprecateVcfPanel,
        ),
        native(
            "docs-build",
            "Build, lint, or serve the governed MkDocs site using the native control plane.",
            NativeOpsCommandKey::ToolingDocsBuild,
        ),
        native(
            "flake-hunt",
            "Run repeated nextest executions for a flake candidate expression.",
            NativeOpsCommandKey::ToolingFlakeHunt,
        ),
        native(
            "generate-configs",
            "Compile generated domain config artifacts from governed domain sources.",
            NativeOpsCommandKey::ToolingGenerateConfigs,
        ),
        native(
            "generate-compatibility-matrix",
            "Generate docs/50-reference/COMPATIBILITY_MATRIX.md from governed registries.",
            NativeOpsCommandKey::ToolingGenerateCompatibilityMatrix,
        ),
        native(
            "generate-config-tree-snapshot",
            "Generate configs/schema/config_tree.snapshot and its marker contract.",
            NativeOpsCommandKey::ToolingGenerateConfigTreeSnapshot,
        ),
        native(
            "generate-panel-compatibility-matrix",
            "Generate docs/50-reference/PANEL_COMPATIBILITY_MATRIX.md from panel and map catalogs.",
            NativeOpsCommandKey::ToolingGeneratePanelCompatibilityMatrix,
        ),
        native(
            "generate-policy-index",
            "Generate artifacts/policies/index.md from policy test sources.",
            NativeOpsCommandKey::ToolingGeneratePolicyIndex,
        ),
        native(
            "generate-docs",
            "Generate the governed documentation outputs backed by native generators.",
            NativeOpsCommandKey::ToolingGenerateDocs,
        ),
        native(
            "generate-docs-graph",
            "Generate docs/DOCS_GRAPH.toml from the governed docs tree.",
            NativeOpsCommandKey::ToolingGenerateDocsGraph,
        ),
        native(
            "generate-domain-coverage-doc",
            "Generate docs/20-science/DOMAIN_COVERAGE.generated.md from domain contracts.",
            NativeOpsCommandKey::ToolingGenerateDomainCoverageDoc,
        ),
        native(
            "generate-repo-root-map",
            "Generate docs/00-intro/REPO_ROOT_MAP.generated.md from root intent metadata.",
            NativeOpsCommandKey::ToolingGenerateRepoRootMap,
        ),
        native(
            "generate-tool-index",
            "Generate docs/20-science/TOOL_INDEX.md from governed tool registries.",
            NativeOpsCommandKey::ToolingGenerateToolIndex,
        ),
        native(
            "image-qa",
            "Run the image QA binary through the native control plane.",
            NativeOpsCommandKey::ToolingImageQa,
        ),
        native(
            "inventory",
            "Generate governed inventory artifacts under artifacts/inventory.",
            NativeOpsCommandKey::ToolingInventory,
        ),
        native(
            "lint-fast",
            "Run changed-path lint gates through the native control plane.",
            NativeOpsCommandKey::ToolingLintFast,
        ),
        native(
            "make-help",
            "Print public and optional internal make target help.",
            NativeOpsCommandKey::ToolingMakeHelp,
        ),
        native(
            "repo-doctor",
            "Run the governed repository doctor bundle.",
            NativeOpsCommandKey::ToolingRepoDoctor,
        ),
        native(
            "bijux",
            "Run the bijux-dna binary inside the governed artifact environment.",
            NativeOpsCommandKey::ToolingRunBijux,
        ),
        native(
            "setup-docs-venv",
            "Create or refresh the governed docs virtual environment.",
            NativeOpsCommandKey::ToolingSetupDocsVenv,
        ),
        native(
            "simulate-coverage-regime",
            "Simulate deterministic VCF coverage regime selection from configured thresholds.",
            NativeOpsCommandKey::ToolingSimulateCoverageRegime,
        ),
        native(
            "validate-frontend-mini-domain-stacks",
            "Validate governed mini frontend stacks and their artifact contracts.",
            NativeOpsCommandKey::ToolingValidateFrontendMiniDomainStacks,
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
