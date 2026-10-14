use anyhow::{ensure, Context as _, Result};
use bijux_dna_domain_compiler::{
    compile_domain_configs, domain_deprecation_catalogs, load_domain_registry_bundle,
    validate_domain, CompileOptions, DomainDeprecationCatalog, DomainRegistryReleaseBundle,
    ValidateOptions, DEFAULT_COMPILE_SCOPE,
};
use std::path::Path;

const UNKNOWN: &str = "unknown_replacement_tool";
const KNOWN: &str = r#"[[deprecations]]
tool_id = "bwa"
deprecated_since = "2026-01-01"
removal_after = "2027-01-01"
rationale = "Replacement inventory admission contract"
replacement = "bowtie2"
"#;

fn compile(root: &Path, output: &Path) -> Result<()> {
    compile_domain_configs(&CompileOptions {
        domain_dir: root.join("domain"),
        configs_dir: output.to_path_buf(),
        scope: DEFAULT_COMPILE_SCOPE.to_string(),
    })
}

fn generated_bundle(output: &Path) -> Result<DomainRegistryReleaseBundle> {
    load_domain_registry_bundle(&output.join("ci/registry/domain_registry_release_bundle.json"))
}

fn replacement(bundle: &DomainRegistryReleaseBundle) -> Result<&str> {
    let bam = bundle
        .domains
        .iter()
        .find(|domain| domain.domain_id == "bam")
        .context("real BAM inventory")?;
    Ok(bam
        .deprecations
        .iter()
        .find(|row| row.tool_id.as_deref() == Some("bwa"))
        .context("actual generated tool deprecation")?
        .replacement
        .as_str())
}

struct UnknownObservation {
    validation: Result<()>,
    compilation: Result<()>,
    published_target: Option<String>,
    outputs_exist: bool,
}

fn observe_unknown(
    root: &Path,
    path: &Path,
    validation_options: &ValidateOptions,
) -> Result<UnknownObservation> {
    std::fs::write(path, KNOWN)?;
    validate_domain(validation_options).context("known replacement public validation")?;
    let known_output = root.join("artifacts/known-generated");
    compile(root, &known_output).context("known replacement public compilation")?;
    let known = generated_bundle(&known_output)?;
    let bam =
        known.domains.iter().find(|domain| domain.domain_id == "bam").context("BAM domain")?;
    ensure!(
        bam.tools.iter().any(|tool| tool.tool_id == "bwa")
            && bam.tools.iter().any(|tool| tool.tool_id == "bowtie2"),
        "known IDs must exist in real source inventory"
    );
    ensure!(
        !known.domains.iter().flat_map(|domain| &domain.tools).any(|tool| tool.tool_id == UNKNOWN),
        "negative target must be absent from inventory"
    );
    ensure!(replacement(&known)? == "bowtie2", "known generated replacement wire differs");
    let snapshot =
        std::fs::read(known_output.join("ci/registry/domain_deprecations_snapshot.json"))?;
    let projection: Vec<DomainDeprecationCatalog> = serde_json::from_slice(&snapshot)?;
    ensure!(
        projection == domain_deprecation_catalogs(&known),
        "compiled projection differs from real bundle"
    );
    eprintln!("COMPILER_DEPRECATION_COMPILE_CONTROLS_PASS");

    let unknown =
        KNOWN.replace("replacement = \"bowtie2\"", "replacement = \"unknown_replacement_tool\"");
    std::fs::write(path, unknown)?;
    let validation = validate_domain(validation_options);
    let unknown_output = root.join("artifacts/unknown-generated");
    let compilation = compile(root, &unknown_output);
    eprintln!("unknown replacement validate_domain={validation:?}; compile_domain_configs={compilation:?}");
    let published_target = if compilation.is_ok() {
        let generated = generated_bundle(&unknown_output)?;
        let target = replacement(&generated)?.to_string();
        ensure!(target == UNKNOWN, "successful compile must be observed against changed source");
        let raw =
            std::fs::read(unknown_output.join("ci/registry/domain_deprecations_snapshot.json"))?;
        let projection: Vec<DomainDeprecationCatalog> = serde_json::from_slice(&raw)?;
        ensure!(
            projection == domain_deprecation_catalogs(&generated),
            "unknown compiled projection differs"
        );
        eprintln!("UNKNOWN_COMPILED_REPLACEMENT_OBSERVED: {target}");
        Some(target)
    } else {
        None
    };
    std::fs::write(path, KNOWN)?;
    validate_domain(validation_options).context("restored known validation")?;
    let restored_output = root.join("artifacts/restored-known-generated");
    compile(root, &restored_output).context("restored known compilation")?;
    ensure!(
        generated_bundle(&restored_output)? == known,
        "restored compiled bundle identity differs"
    );
    ensure!(
        std::fs::read(restored_output.join("ci/registry/domain_deprecations_snapshot.json"))?
            == snapshot,
        "restored projection bytes differ"
    );
    Ok(UnknownObservation {
        validation,
        compilation,
        published_target,
        outputs_exist: unknown_output.exists(),
    })
}

fn require_refusals(observation: UnknownObservation, path: &Path) -> Result<()> {
    let mut failures = Vec::new();
    for (label, result) in
        [("validation", observation.validation), ("compilation", observation.compilation)]
    {
        if let Err(error) = result {
            let message = format!("{error:#}");
            ensure!(
                message.contains("replacement")
                    && message.contains(UNKNOWN)
                    && message.contains(&path.display().to_string()),
                "wrong {label} refusal: {message}"
            );
        } else {
            failures.push(label);
        }
    }
    ensure!(
        failures.is_empty(),
        "unknown bare replacement was admitted by {failures:?}; actual compiled replacement: {:?}; output directory exists: {}",
        observation.published_target,
        observation.outputs_exist,
    );
    Ok(())
}

#[test]
fn compilation_rejects_tool_replacements_outside_inventory() -> Result<()> {
    let fixture = super::support::artifact_output_dir("deprecation-source-")?;
    let root = fixture.path();
    super::domain_workspace::materialize(&super::support::repo_root(), root)?;
    let path = root.join("configs/ci/registry/deprecations.toml");
    let original = std::fs::read(&path)?;
    let permissions = std::fs::metadata(&path)?.permissions();
    let options = ValidateOptions { domain_dir: root.join("domain") };
    validate_domain(&options).context("unmodified public validation positive")?;
    compile(root, &root.join("artifacts/unmodified-generated"))
        .context("unmodified public compilation positive")?;
    let observation = observe_unknown(root, &path, &options);
    std::fs::write(&path, &original)?;
    ensure!(std::fs::read(&path)? == original, "original governed bytes must be restored");
    ensure!(std::fs::metadata(&path)?.permissions() == permissions, "input permissions changed");
    validate_domain(&options).context("restored original validation")?;
    require_refusals(observation?, &path)
}
