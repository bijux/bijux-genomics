//! Owner: benchmark local campaign planning.
//! Campaign templates owned by the workflow boundary.
use anyhow::Context;
use anyhow::{anyhow, Result};
use bijux_dna_bench_model::OutputTemplates;
use regex::Regex;
use std::collections::{BTreeMap, BTreeSet};

pub(crate) fn template_tokens(template: &str) -> Result<BTreeSet<String>> {
    let re = Regex::new(r"\{([a-z_]+)\}").context("compile template regex")?;
    let mut out = BTreeSet::new();
    for cap in re.captures_iter(template) {
        if let Some(token) = cap.get(1) {
            out.insert(token.as_str().to_string());
        }
    }
    Ok(out)
}

pub(crate) fn validate_templates(templates: &OutputTemplates) -> Result<()> {
    let allowed = BTreeSet::from([
        "job_id".to_string(),
        "timestamp".to_string(),
        "campaign".to_string(),
        "domain".to_string(),
        "stage".to_string(),
        "tool".to_string(),
        "sample".to_string(),
        "array_task".to_string(),
    ]);
    let required = BTreeSet::from([
        "job_id".to_string(),
        "timestamp".to_string(),
        "campaign".to_string(),
        "domain".to_string(),
        "stage".to_string(),
        "tool".to_string(),
        "sample".to_string(),
    ]);
    for (name, template) in [
        ("log", templates.log.as_str()),
        ("out", templates.out.as_str()),
        ("err", templates.err.as_str()),
        ("results", templates.results.as_str()),
        ("code", templates.code.as_str()),
    ] {
        let tokens = template_tokens(template)?;
        for token in &tokens {
            if !allowed.contains(token.as_str()) {
                return Err(anyhow!(
                    "output template `{name}` references unsupported token `{token}`"
                ));
            }
        }
        for required_token in &required {
            if !tokens.contains(required_token.as_str()) {
                return Err(anyhow!(
                    "output template `{name}` must include required token `{required_token}`"
                ));
            }
        }
    }
    Ok(())
}

pub(crate) fn fill_template(template: &str, values: &BTreeMap<String, String>) -> Result<String> {
    let mut out = template.to_string();
    for token in template_tokens(template)? {
        let value = values
            .get(&token)
            .ok_or_else(|| anyhow!("template token `{token}` has no resolved value"))?;
        out = out.replace(&format!("{{{token}}}"), value);
    }
    Ok(out)
}
