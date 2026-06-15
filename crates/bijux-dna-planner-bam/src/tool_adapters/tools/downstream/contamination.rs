use std::path::Path;

use bijux_dna_domain_bam::params::ContaminationEffectiveParams;

fn summary_payload(
    method: &str,
    tool_scope: &str,
    params: &ContaminationEffectiveParams,
    reference: Option<&Path>,
    panels: &[&Path],
) -> String {
    serde_json::to_string_pretty(&serde_json::json!({
        "method": method,
        "scope": params.scope,
        "tool_scope": tool_scope,
        "assumptions": params.assumptions,
        "reference": reference.map(|path| path.display().to_string()),
        "reference_panels": panels
            .iter()
            .map(|path| path.display().to_string())
            .collect::<Vec<_>>(),
        "minimum_mean_coverage": params.minimum_mean_coverage,
        "emit_confidence_caveats": params.emit_confidence_caveats,
    }))
    .unwrap_or_else(|error| panic!("contamination summary payload must serialize: {error}"))
}

fn python_json_literal(payload: String) -> String {
    serde_json::to_string(&payload)
        .unwrap_or_else(|error| panic!("contamination summary literal must serialize: {error}"))
}

pub mod contammix {
    use super::*;

    #[must_use]
    pub fn args_with_outputs(
        bam: &Path,
        bam_index: Option<&Path>,
        reference: Option<&Path>,
        panels: &[&Path],
        report: &Path,
        summary: &Path,
        params: &ContaminationEffectiveParams,
    ) -> Vec<String> {
        let panel = panels
            .first()
            .map_or_else(String::new, |path| format!(" --reference-panel {}", path.display()));
        let reference_flag =
            reference.map_or_else(String::new, |path| format!(" --reference {}", path.display()));
        let bam_index =
            bam_index.map_or_else(String::new, |path| format!("test -f {} && ", path.display()));
        let summary_payload =
            python_json_literal(summary_payload("contammix", "nuclear", params, reference, panels));
        let command = format!(
            "{bam_index}contammix --bam {bam}{reference_flag}{panel} > {report} && \
python - <<'PY' > {summary}\nimport json\nprint(json.dumps(json.loads({summary_payload}), indent=2))\nPY",
            bam_index = bam_index,
            bam = bam.display(),
            reference_flag = reference_flag,
            panel = panel,
            report = report.display(),
            summary = summary.display(),
            summary_payload = summary_payload
        );
        vec!["/bin/sh".to_string(), "-c".to_string(), command]
    }
}

pub mod schmutzi {
    use super::*;

    #[must_use]
    pub fn args_with_outputs(
        bam: &Path,
        reference: Option<&Path>,
        report: &Path,
        summary: &Path,
        params: &ContaminationEffectiveParams,
    ) -> Vec<String> {
        let reference_flag =
            reference.map_or_else(String::new, |path| format!(" --reference {}", path.display()));
        let summary_payload =
            python_json_literal(summary_payload("schmutzi", "mt", params, reference, &[]));
        let command = format!(
            "schmutzi --bam {bam}{reference_flag} --outdir {out_dir} && \
if [ -f {out_dir}/contamination.txt ]; then cp {out_dir}/contamination.txt {report}; else : > {report}; fi && \
python - <<'PY' > {summary}\nimport json\nprint(json.dumps(json.loads({summary_payload}), indent=2))\nPY",
            bam = bam.display(),
            reference_flag = reference_flag,
            out_dir = report
                .parent()
                .map_or_else(|| ".".to_string(), |p| p.display().to_string()),
            report = report.display(),
            summary = summary.display(),
            summary_payload = summary_payload
        );
        vec!["/bin/sh".to_string(), "-c".to_string(), command]
    }
}

pub mod verifybamid2 {
    use super::*;

    #[must_use]
    pub fn args_with_outputs(
        bam: &Path,
        bam_index: Option<&Path>,
        reference: Option<&Path>,
        panels: &[&Path],
        report: &Path,
        summary: &Path,
        params: &ContaminationEffectiveParams,
    ) -> Vec<String> {
        let output_prefix = report.with_extension("");
        let panel = panels
            .first()
            .map_or_else(String::new, |path| format!(" --SVDPrefix {}", path.display()));
        let reference_flag =
            reference.map_or_else(String::new, |path| format!(" --Reference {}", path.display()));
        let bam_index =
            bam_index.map_or_else(String::new, |path| format!("test -f {} && ", path.display()));
        let summary_payload = python_json_literal(summary_payload(
            "verifybamid2",
            "nuclear",
            params,
            reference,
            panels,
        ));
        let command = format!(
            "{bam_index}verifybamid2 --BamFile {bam}{reference_flag}{panel} --Output {output_prefix} && \
if [ -f {output_prefix}.selfSM ]; then cp {output_prefix}.selfSM {report}; else : > {report}; fi && \
python - <<'PY' > {summary}\nimport json\nprint(json.dumps(json.loads({summary_payload}), indent=2))\nPY",
            bam_index = bam_index,
            bam = bam.display(),
            reference_flag = reference_flag,
            panel = panel,
            output_prefix = output_prefix.display(),
            report = report.display(),
            summary = summary.display(),
            summary_payload = summary_payload
        );
        vec!["/bin/sh".to_string(), "-c".to_string(), command]
    }
}
