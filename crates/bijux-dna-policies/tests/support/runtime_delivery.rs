use std::collections::BTreeSet;
use std::path::Path;

pub fn redistributable_runtime_contract_violations(
    root: &Path,
    subject: &str,
    tool: &toml::Value,
) -> Vec<String> {
    let mut violations = Vec::new();
    if tool.get("artifact_delivery").and_then(toml::Value::as_str) != Some("redistributable_image")
    {
        violations.push(format!("{subject} must declare redistributable_image delivery"));
        return violations;
    }
    if tool.get("product_support").and_then(toml::Value::as_str) != Some("supported")
        || tool.get("redistribution_permitted").and_then(toml::Value::as_bool) != Some(true)
        || tool.get("external_artifact_supported").and_then(toml::Value::as_bool) != Some(false)
        || tool.get("container").and_then(toml::Value::as_bool) != Some(true)
    {
        violations.push(format!("{subject} redistributable delivery dimensions are inconsistent"));
    }
    let runtimes = tool
        .get("runtimes")
        .and_then(toml::Value::as_array)
        .map(|values| values.iter().filter_map(toml::Value::as_str).collect::<BTreeSet<_>>())
        .unwrap_or_default();
    if runtimes.is_empty() {
        violations.push(format!("{subject} must declare at least one supported container runtime"));
    }
    for runtime in &runtimes {
        let recipe_key = match *runtime {
            "docker" => "dockerfile",
            "apptainer" => "apptainer_def",
            unknown => {
                violations.push(format!("{subject} declares unsupported runtime {unknown}"));
                continue;
            }
        };
        let recipe = tool.get(recipe_key).and_then(toml::Value::as_str).unwrap_or("");
        if recipe.is_empty() || !root.join(recipe).is_file() {
            violations.push(format!(
                "{subject} runtime {runtime} missing repository recipe at `{recipe}`"
            ));
        }
    }
    for (runtime, recipe_key) in [("docker", "dockerfile"), ("apptainer", "apptainer_def")] {
        let recipe = tool.get(recipe_key).and_then(toml::Value::as_str).unwrap_or("");
        if !runtimes.contains(runtime) && !recipe.is_empty() {
            violations.push(format!("{subject} has undeclared {runtime} recipe `{recipe}`"));
        }
    }
    violations
}
