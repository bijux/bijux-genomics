use anyhow::{Context, Result};
use bijux_dna_domain_compiler::{
    compile_domain_configs, CompileOptions, DEFAULT_COMPILE_SCOPE, DEFAULT_DOMAIN_DIR,
};
use std::collections::BTreeMap;

#[path = "support/mod.rs"]
mod support;

#[test]
fn generated_tool_registries_use_container_catalog_metadata() -> Result<()> {
    let root = support::repo_root();
    let out_dir = support::artifact_output_dir("container-catalog-authority-")?;
    compile_domain_configs(&CompileOptions {
        domain_dir: root.join(DEFAULT_DOMAIN_DIR),
        configs_dir: out_dir.path().to_path_buf(),
        scope: DEFAULT_COMPILE_SCOPE.to_string(),
    })?;

    let catalog = read_toml(&root.join("containers/contracts/catalog.toml"))?;
    let contracts = rows_by_id(&catalog, "tools")?;
    for relative in [
        "ci/registry/tool_registry.toml",
        "ci/registry/tool_registry_experimental.toml",
        "ci/registry/tool_registry_vcf.toml",
    ] {
        let registry = read_toml(&out_dir.path().join(relative))?;
        for row in registry.get("tools").and_then(toml::Value::as_array).into_iter().flatten() {
            let id = text(row, "id")?;
            let contract = contracts
                .get(id)
                .with_context(|| format!("generated tool {id} is absent from the catalog"))?;
            for (generated, canonical) in [
                ("version", "version"),
                ("upstream", "upstream"),
                ("license", "license"),
                ("version_cmd", "version_command"),
                ("help_cmd", "help_command"),
                ("expected_version_regex", "expected_version_regex"),
                ("expected_bin", "expected_bin"),
                ("dockerfile", "dockerfile"),
                ("apptainer_def", "apptainer_definition"),
            ] {
                assert_eq!(
                    text(row, generated)?,
                    text(contract, canonical)?,
                    "{id}: generated {generated} diverges from catalog {canonical}"
                );
            }
            let expected_runtimes =
                [("docker_arm64_supported", "docker"), ("apptainer_supported", "apptainer")]
                    .into_iter()
                    .filter_map(|(field, runtime)| {
                        contract
                            .get(field)
                            .and_then(toml::Value::as_bool)
                            .unwrap_or(false)
                            .then_some(runtime)
                    })
                    .collect::<Vec<_>>();
            let actual_runtimes = row
                .get("runtimes")
                .and_then(toml::Value::as_array)
                .context("generated tool has no runtimes")?
                .iter()
                .map(|value| value.as_str().context("runtime is not text"))
                .collect::<Result<Vec<_>>>()?;
            assert_eq!(actual_runtimes, expected_runtimes, "{id}: runtime authority drift");
            let disposition = text(contract, "support_disposition")?;
            let expected_container_ref = if expected_runtimes.is_empty() {
                "external@licensed-installation".to_string()
            } else {
                format!("bijuxdna/{id}:{}", text(contract, "version")?)
            };
            assert_eq!(
                text(row, "container_ref")?,
                expected_container_ref,
                "{id}: container reference does not identify the catalog version"
            );
            if disposition != "retained" {
                assert_eq!(
                    text(row, "status")?,
                    disposition,
                    "{id}: unsupported disposition was promoted by domain metadata"
                );
            }
        }
        if relative == "ci/registry/tool_registry.toml" {
            let tool_stage_ids = registry
                .get("tools")
                .and_then(toml::Value::as_array)
                .context("production registry has no tools")?
                .iter()
                .map(|row| {
                    let id = text(row, "id")?;
                    let stages = row
                        .get("stage_ids")
                        .and_then(toml::Value::as_array)
                        .context("production tool has no stage_ids")?
                        .iter()
                        .map(|value| value.as_str().context("stage id is not text"))
                        .collect::<Result<Vec<_>>>()?;
                    Ok((id, stages))
                })
                .collect::<Result<BTreeMap<_, _>>>()?;
            for stage in registry
                .get("stages")
                .and_then(toml::Value::as_array)
                .context("production registry has no stages")?
            {
                let stage_id = text(stage, "id")?;
                for field in ["primary_tools", "optional_alternatives"] {
                    for tool_id in stage
                        .get(field)
                        .and_then(toml::Value::as_array)
                        .context("production stage has no tool binding array")?
                        .iter()
                        .map(|value| value.as_str().context("tool id is not text"))
                    {
                        assert!(
                            tool_stage_ids
                                .get(tool_id?)
                                .is_some_and(|stage_ids| stage_ids.contains(&stage_id)),
                            "{stage_id}: {field} contains a tool without an authored stage binding"
                        );
                    }
                }
            }
        }
    }
    Ok(())
}

fn read_toml(path: &std::path::Path) -> Result<toml::Value> {
    let raw = std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))
}

fn rows_by_id<'a>(
    document: &'a toml::Value,
    key: &str,
) -> Result<BTreeMap<&'a str, &'a toml::Value>> {
    document
        .get(key)
        .and_then(toml::Value::as_array)
        .context("catalog has no tool rows")?
        .iter()
        .map(|row| Ok((text(row, "id")?, row)))
        .collect()
}

fn text<'a>(row: &'a toml::Value, key: &str) -> Result<&'a str> {
    row.get(key)
        .and_then(toml::Value::as_str)
        .with_context(|| format!("row has no text field {key}"))
}
