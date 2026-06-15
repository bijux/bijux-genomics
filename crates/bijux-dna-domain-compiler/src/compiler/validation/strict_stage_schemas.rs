use super::{anyhow, bail, collect_yaml_files, read_yaml, Context, Result, ValidateOptions};

#[derive(Debug, serde::Deserialize)]
struct StageSchemaDoc {
    #[serde(default)]
    required_fields: Vec<String>,
    #[serde(default)]
    allowed_status: Vec<String>,
    #[serde(default)]
    required_scope: String,
    #[serde(default)]
    domain: String,
}

pub(super) fn validate_stage_schema_contracts(options: &ValidateOptions, dom: &str) -> Result<()> {
    let schema_path = options.domain_dir.join(dom).join("stages").join("_schema.yaml");
    let schema: StageSchemaDoc = read_yaml(&schema_path)?;
    let stages_dir = options.domain_dir.join(dom).join("stages");
    for path in collect_yaml_files(&stages_dir)? {
        if path
            .file_name()
            .and_then(|value| value.to_str())
            .is_some_and(|name| name.starts_with('_'))
        {
            continue;
        }
        validate_single_stage_schema(&schema, &schema_path, &path)?;
    }
    Ok(())
}

fn validate_single_stage_schema(
    schema: &StageSchemaDoc,
    schema_path: &std::path::Path,
    path: &std::path::Path,
) -> Result<()> {
    let raw = std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let value: serde_yaml::Value = bijux_dna_infra::formats::parse_yaml(&raw)
        .with_context(|| format!("parse {}", path.display()))?;
    let mapping = value
        .as_mapping()
        .ok_or_else(|| anyhow!("{} must deserialize to a YAML mapping", path.display()))?;

    for field in &schema.required_fields {
        let key = serde_yaml::Value::String(field.clone());
        let Some(entry) = mapping.get(&key) else {
            bail!(
                "{} missing required field `{}` declared by {}",
                path.display(),
                field,
                schema_path.display()
            );
        };
        if matches!(entry, serde_yaml::Value::Null) {
            bail!("{} field `{}` must not be null", path.display(), field);
        }
        if let Some(text) = entry.as_str() {
            if text.trim().is_empty() {
                bail!("{} field `{}` must not be blank", path.display(), field);
            }
        }
    }

    let status = mapping
        .get(serde_yaml::Value::String("status".to_string()))
        .and_then(serde_yaml::Value::as_str)
        .unwrap_or_default();
    if !schema.allowed_status.is_empty()
        && !schema.allowed_status.iter().any(|allowed| allowed == status)
    {
        bail!(
            "{} status `{}` is outside schema-allowed values {:?}",
            path.display(),
            status,
            schema.allowed_status
        );
    }

    let scope = mapping
        .get(serde_yaml::Value::String("scope".to_string()))
        .and_then(serde_yaml::Value::as_str)
        .unwrap_or_default();
    if !schema.required_scope.trim().is_empty() && scope != schema.required_scope {
        bail!(
            "{} scope `{}` must match schema scope `{}`",
            path.display(),
            scope,
            schema.required_scope
        );
    }

    let domain = mapping
        .get(serde_yaml::Value::String("domain".to_string()))
        .and_then(serde_yaml::Value::as_str)
        .unwrap_or_default();
    if !schema.domain.trim().is_empty() && domain != schema.domain {
        bail!(
            "{} domain `{}` must match schema domain `{}`",
            path.display(),
            domain,
            schema.domain
        );
    }

    validate_ports(path, mapping, "inputs")?;
    validate_ports(path, mapping, "outputs")?;
    validate_metrics(path, mapping)?;

    if !mapping.contains_key(serde_yaml::Value::String("description".to_string())) {
        bail!(
            "{} missing description; strict stage schemas require explicit stage descriptions",
            path.display()
        );
    }
    if !mapping.contains_key(serde_yaml::Value::String("tool_capability_requirements".to_string()))
    {
        bail!(
            "{} missing tool_capability_requirements; strict stage schemas require backend capability declarations",
            path.display()
        );
    }

    Ok(())
}

fn validate_ports(path: &std::path::Path, mapping: &serde_yaml::Mapping, key: &str) -> Result<()> {
    let ports = mapping
        .get(serde_yaml::Value::String(key.to_string()))
        .and_then(serde_yaml::Value::as_sequence)
        .ok_or_else(|| anyhow!("{} field `{}` must be a YAML sequence", path.display(), key))?;
    for (index, port) in ports.iter().enumerate() {
        let Some(port_map) = port.as_mapping() else {
            bail!(
                "{} {}[{index}] must be a mapping with role/type/cardinality",
                path.display(),
                key
            );
        };
        for field in ["name", "data_type", "cardinality"] {
            let value = port_map
                .get(serde_yaml::Value::String(field.to_string()))
                .and_then(serde_yaml::Value::as_str)
                .unwrap_or_default();
            if value.trim().is_empty() {
                bail!("{} {}[{index}] missing {} role metadata", path.display(), key, field);
            }
        }
    }
    Ok(())
}

fn validate_metrics(path: &std::path::Path, mapping: &serde_yaml::Mapping) -> Result<()> {
    let metrics = mapping
        .get(serde_yaml::Value::String("metrics".to_string()))
        .and_then(serde_yaml::Value::as_sequence)
        .ok_or_else(|| anyhow!("{} field `metrics` must be a YAML sequence", path.display()))?;
    for (index, metric) in metrics.iter().enumerate() {
        let Some(metric_map) = metric.as_mapping() else {
            bail!(
                "{} metrics[{index}] must be an object so the metric stays typed and versionable",
                path.display()
            );
        };
        let name = metric_map
            .get(serde_yaml::Value::String("name".to_string()))
            .and_then(serde_yaml::Value::as_str)
            .unwrap_or_default();
        if name.trim().is_empty() {
            bail!("{} metrics[{index}] missing metric name", path.display());
        }
    }
    Ok(())
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::validate_stage_schema_contracts;
    use crate::ValidateOptions;

    #[test]
    fn strict_stage_schema_reports_missing_description() -> anyhow::Result<()> {
        let temp = tempfile::tempdir()?;
        let domain_dir = temp.path().join("fastq");
        std::fs::create_dir_all(domain_dir.join("stages"))?;
        std::fs::write(
            domain_dir.join("stages/_schema.yaml"),
            r#"
required_fields:
  - stage_id
  - status
  - scope
  - domain
  - inputs
  - outputs
  - description
  - metrics
  - compatible_tools
  - assumptions
  - defaults_source
  - bank_hooks
  - metrics_schema
  - tool_capability_requirements
  - required_inputs
  - required_outputs
  - allowed_missingness
allowed_status:
  - supported
required_scope: "pre_hpc_pre_vcf"
domain: "fastq"
"#,
        )?;
        std::fs::write(
            domain_dir.join("stages/example.yaml"),
            r#"
stage_id: "fastq.example"
status: "supported"
scope: "pre_hpc_pre_vcf"
domain: "fastq"
inputs:
  - name: "reads"
    data_type: "fastq"
    cardinality: "One"
outputs:
  - name: "report_json"
    data_type: "json"
    cardinality: "One"
tool_capability_requirements: []
metrics:
  - name: "read_count"
compatible_tools:
  - "fastp"
assumptions:
  - "test"
defaults_source: "doc_ref:test"
bank_hooks:
  - "none"
metrics_schema: "bijux.stage.metrics.v1"
required_inputs:
  - "reads"
required_outputs:
  - "report_json"
allowed_missingness:
  - "none"
"#,
        )?;
        let error = validate_stage_schema_contracts(
            &ValidateOptions { domain_dir: temp.path().to_path_buf() },
            "fastq",
        )
        .expect_err("schema should reject missing description");
        assert!(error.to_string().contains("missing required field `description`"));
        Ok(())
    }
}
