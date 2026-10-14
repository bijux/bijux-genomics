use anyhow::{ensure, Context as _, Result};
use bijux_dna_domain_compiler::{
    compile_domain_configs, validate_domain, CompileOptions, ValidateOptions, DEFAULT_COMPILE_SCOPE,
};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

const STAGE: &str = "fastq.validate_reads";

struct Observation {
    validation: Result<()>,
    compilation: Result<()>,
    index: PathBuf,
    outputs: PathBuf,
    quantity: f64,
}

fn compile(root: &Path, outputs: &Path) -> Result<()> {
    compile_domain_configs(&CompileOptions {
        domain_dir: root.join("domain"),
        configs_dir: outputs.to_path_buf(),
        scope: DEFAULT_COMPILE_SCOPE.to_string(),
    })
}

fn generated_files(root: &Path, directory: &Path) -> Result<BTreeMap<PathBuf, Vec<u8>>> {
    let mut files = BTreeMap::new();
    for entry in std::fs::read_dir(directory)? {
        let path = entry?.path();
        ensure!(!path.is_symlink(), "generated file must remain owned");
        if path.is_dir() {
            files.extend(generated_files(root, &path)?);
        } else {
            ensure!(path.is_file(), "generated output must be regular");
            files.insert(path.strip_prefix(root)?.to_path_buf(), std::fs::read(path)?);
        }
    }
    Ok(files)
}

fn mapping_body(source: &str, key: &str, indent: usize) -> Result<(usize, usize)> {
    let header = format!("{}{key}:", " ".repeat(indent));
    let mut start = None;
    let mut end = source.len();
    let mut offset = 0;
    for line in source.split_inclusive('\n') {
        let content = line.trim_end_matches(['\r', '\n']);
        if let Some(rest) = content.strip_prefix(&header) {
            ensure!(rest.trim().is_empty(), "fixture mapping {key} must use its own line");
            ensure!(start.is_none(), "fixture mapping {key} must be unique");
            start = Some(offset + line.len());
        } else if start.is_some() && end == source.len() {
            let trimmed = content.trim_start();
            if !trimmed.is_empty()
                && !trimmed.starts_with('#')
                && content.len() - trimmed.len() <= indent
            {
                end = offset;
            }
        }
        offset += line.len();
    }
    Ok((start.with_context(|| format!("missing fixture mapping {key}"))?, end))
}

// Preserve unrelated YAML syntax, especially empty mapping values used by the custom loader.
// Only the unique numeric scalar in the governed mapping may change.
fn replace_quantity(original: &[u8], section: &str, field: &str, token: &str) -> Result<Vec<u8>> {
    let source = std::str::from_utf8(original)?;
    let (section_start, section_end) = mapping_body(source, section, 0)?;
    let (stage_start, stage_end) = mapping_body(&source[section_start..section_end], STAGE, 2)?;
    let start = section_start + stage_start;
    let end = section_start + stage_end;
    let prefix = format!("    {field}:");
    let mut scalar = None;
    let mut offset = start;
    for line in source[start..end].split_inclusive('\n') {
        let content = line.trim_end_matches(['\r', '\n']);
        if let Some(rest) = content.strip_prefix(&prefix) {
            ensure!(scalar.is_none(), "fixture scalar {field} must be unique");
            let value = rest.trim_start();
            let length =
                value.find(|ch: char| ch.is_whitespace() || ch == '#').unwrap_or(value.len());
            value[..length].parse::<f64>().context("fixture scalar must be numeric")?;
            let suffix = value[length..].trim_start();
            ensure!(
                suffix.is_empty() || suffix.starts_with('#'),
                "fixture scalar has extra content"
            );
            let position = offset + prefix.len() + rest.len() - value.len();
            scalar = Some(position..position + length);
        }
        offset += line.len();
    }
    let scalar = scalar.with_context(|| format!("missing fixture scalar {field}"))?;
    let mut changed = source.to_string();
    changed.replace_range(scalar, token);
    Ok(changed.into_bytes())
}

fn fixture_quantity(token: &str) -> Result<f64> {
    match token {
        ".nan" => Ok(f64::NAN),
        ".inf" => Ok(f64::INFINITY),
        "-.inf" => Ok(f64::NEG_INFINITY),
        _ => Ok(token.parse()?),
    }
}

fn observe(
    root: &Path,
    original: &[u8],
    section: &str,
    field: &str,
    token: &str,
    label: &str,
) -> Result<Observation> {
    let index = root.join("domain/fastq/index.yaml");
    let quantity = fixture_quantity(token)?;
    std::fs::write(&index, replace_quantity(original, section, field, token)?)?;
    let outputs = root.join("artifacts/generated").join(label);
    ensure!(!outputs.exists(), "each observation requires fresh generated output");
    let validation = validate_domain(&ValidateOptions { domain_dir: root.join("domain") });
    let compilation = compile(root, &outputs);
    std::fs::write(&index, original)?;
    ensure!(std::fs::read(&index)? == original, "original input bytes must be restored");
    eprintln!("{label}: validate_domain={validation:?}; compile_domain_configs={compilation:?}");
    Ok(Observation { validation, compilation, index, outputs, quantity })
}

fn emitted_quantity(outputs: &Path, field: &str) -> Result<f64> {
    let document: toml::Value =
        toml::from_str(&std::fs::read_to_string(outputs.join("ci/stages/stages.toml"))?)?;
    let row = document
        .get("stages")
        .and_then(toml::Value::as_array)
        .context("generated stage array")?
        .iter()
        .find(|row| row.get("id").and_then(toml::Value::as_str) == Some(STAGE))
        .context("selected stage must be emitted")?;
    if field == "memory_gb" {
        let value = row.get("resource_memory_gb").context("generated memory")?;
        ensure!(
            matches!(value, toml::Value::Float(_) | toml::Value::Integer(_)),
            "numeric memory required"
        );
        Ok(value.to_string().parse()?)
    } else {
        let encoded = row
            .get("output_size_estimates_mb")
            .and_then(toml::Value::as_array)
            .context("generated output estimates")?
            .iter()
            .filter_map(toml::Value::as_str)
            .find_map(|value| value.strip_prefix("validation_report:"))
            .context("selected output estimate must be emitted")?;
        Ok(encoded.parse()?)
    }
}

fn require_disposition(
    observation: Observation,
    section: &str,
    field: &str,
    allowed: bool,
) -> Result<()> {
    if allowed {
        observation.validation.context("finite validation compatibility")?;
        observation.compilation.context("finite compilation compatibility")?;
        let emitted = emitted_quantity(&observation.outputs, field)?;
        ensure!(
            emitted.to_bits() == observation.quantity.to_bits(),
            "generated quantity must preserve its admitted value"
        );
    } else {
        let validation =
            observation.validation.err().context("matrix path admitted refused quantity")?;
        let compilation =
            observation.compilation.err().context("loading path admitted refused quantity")?;
        let validation = format!("{validation:#}");
        let compilation = format!("{compilation:#}");
        ensure!(
            validation.contains(STAGE)
                && validation.contains(section)
                && validation.contains(&observation.index.display().to_string()),
            "wrong matrix refusal: {validation}"
        );
        ensure!(
            compilation.contains(STAGE) && compilation.contains(section),
            "wrong loading refusal: {compilation}"
        );
        ensure!(
            !observation.outputs.exists(),
            "quantity admission must precede output-directory creation"
        );
    }
    Ok(())
}

fn exercise_quantity(section: &str, field: &str, controls: &[(&str, bool)]) -> Result<()> {
    let fixture = super::support::artifact_output_dir("resource-quantity-")?;
    super::domain_workspace::materialize(&super::support::repo_root(), fixture.path())?;
    let root = fixture.path();
    let index = root.join("domain/fastq/index.yaml");
    let original = std::fs::read(&index)?;
    let original_mode = std::fs::metadata(&index)?.permissions();
    validate_domain(&ValidateOptions { domain_dir: root.join("domain") })
        .context("unmodified private-workspace validation positive")?;
    let baseline = root.join("artifacts/base-generated");
    compile(root, &baseline).context("unmodified private-workspace compilation positive")?;
    let baseline_files = generated_files(&baseline, &baseline)?;
    ensure!(!baseline_files.is_empty(), "baseline must produce actual outputs");
    for (number, &(token, allowed)) in controls.iter().enumerate() {
        require_disposition(
            observe(root, &original, section, field, token, &format!("control-{number}"))?,
            section,
            field,
            allowed,
        )?;
    }
    if field == "memory_gb" {
        for integer in ["time_minutes", "threads"] {
            require_disposition(
                observe(root, &original, section, integer, "0", integer)?,
                section,
                integer,
                false,
            )?;
        }
    }
    eprintln!("COMPILER_QUANTITY_CONTROLS_PASS: {field}");
    let mut failures = Vec::new();
    for (number, token) in [".nan", ".inf"].into_iter().enumerate() {
        let observation =
            observe(root, &original, section, field, token, &format!("nonfinite-{number}"))?;
        ensure!(!observation.quantity.is_finite(), "causal wire must decode as nonfinite");
        if let Err(error) = require_disposition(observation, section, field, false) {
            failures.push(format!("{token}: {error:#}"));
        }
    }
    ensure!(std::fs::read(&index)? == original, "final source input must be restored");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        ensure!(
            std::fs::metadata(&index)?.permissions().mode() == original_mode.mode(),
            "source mode changed"
        );
    }
    #[cfg(not(unix))]
    ensure!(
        std::fs::metadata(&index)?.permissions().readonly() == original_mode.readonly(),
        "source permissions changed"
    );
    validate_domain(&ValidateOptions { domain_dir: root.join("domain") })?;
    let restored = root.join("artifacts/restored-generated");
    compile(root, &restored)?;
    ensure!(
        baseline_files == generated_files(&restored, &restored)?,
        "restored generated path/byte map differs"
    );
    ensure!(failures.is_empty(), "nonfinite admission failures: {failures:?}");
    Ok(())
}

#[test]
fn memory_requires_finite_positive_quantity() -> Result<()> {
    exercise_quantity(
        "stage_resource_hints",
        "memory_gb",
        &[("4.25", true), ("1.0e-12", true), ("0.0", false), ("-1.0", false), ("-.inf", false)],
    )
}

#[test]
fn output_estimates_require_finite_nonnegative_quantities() -> Result<()> {
    exercise_quantity(
        "stage_output_size_estimates_mb",
        "validation_report",
        &[("0.0", true), ("-0.0", true), ("1.0e200", true), ("-1.0", false), ("-.inf", false)],
    )
}
