use std::path::Path;

use bijux_dna_domain_bam::params::HaplogroupEffectiveParams;

fn python_json_literal(payload: serde_json::Value) -> String {
    serde_json::to_string(&payload)
        .unwrap_or_else(|error| panic!("haplogroups payload must serialize: {error}"))
}

#[must_use]
pub fn args_with_outputs(
    tool_id: &str,
    bam: &Path,
    bam_index: Option<&Path>,
    report: &Path,
    summary: &Path,
    params: &HaplogroupEffectiveParams,
) -> Vec<String> {
    let output_prefix = report.with_extension("");
    let bam_index_check =
        bam_index.map_or_else(String::new, |path| format!("test -f {} && ", path.display()));
    let panel_check = format!("test -f {} && ", params.reference_panel);
    let population_scope = params.population_scope.as_deref().unwrap_or("unspecified");
    let min_coverage = params.min_coverage.unwrap_or(0.0);
    let summary_payload = python_json_literal(serde_json::json!({
        "method": tool_id,
        "reference_panel": params.reference_panel,
        "reference_build": params.reference_build,
        "population_scope": population_scope,
        "min_coverage": params.min_coverage,
        "refuse_without_population_context": params.refuse_without_population_context,
    }));
    let report_payload = python_json_literal(serde_json::json!({
        "schema_version": "bijux.bam.haplogroups.v1",
        "tool": tool_id,
        "classification_scope": "y_haplogroup_inference",
        "reference_panel": params.reference_panel,
        "reference_build": params.reference_build,
        "population_scope": population_scope,
        "coverage_gate": {
            "min_coverage": min_coverage,
        },
        "refuse_without_population_context": params.refuse_without_population_context,
        "summary_output": summary.display().to_string(),
        "assignment_output_prefix": output_prefix.display().to_string(),
    }));
    let command = match tool_id {
        "yleaf" => format!(
            "{bam_index_check}{panel_check}yleaf -bam {bam} -o {output_prefix} --reference_genome {reference_build} && \
python - <<'PY' > {summary}\nimport json\nprint(json.dumps(json.loads({summary_payload}), indent=2))\nPY && \
python - <<'PY' > {report}\nimport json\nprint(json.dumps(json.loads({report_payload}), indent=2))\nPY",
            bam_index_check = bam_index_check,
            panel_check = panel_check,
            bam = bam.display(),
            output_prefix = output_prefix.display(),
            reference_build = params.reference_build,
            summary = summary.display(),
            report = report.display(),
            summary_payload = summary_payload,
            report_payload = report_payload
        ),
        _ => format!(
            "printf '%s\\n' 'bam.haplogroups tool {tool_id} is not implemented for governed planning' >&2; exit 1"
        ),
    };
    vec!["/bin/sh".to_string(), "-c".to_string(), command]
}
