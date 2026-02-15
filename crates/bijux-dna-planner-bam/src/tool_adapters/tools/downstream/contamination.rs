use std::path::Path;

use bijux_dna_domain_bam::params::ContaminationEffectiveParams;

pub mod contammix {
    use super::*;

    #[must_use]
    pub fn args_with_outputs(
        bam: &Path,
        report: &Path,
        summary: &Path,
        _params: &ContaminationEffectiveParams,
    ) -> Vec<String> {
        let command = format!(
            "contammix --bam {bam} > {report} && \
python - <<'PY' > {summary}\nimport json\nprint(json.dumps({{\"method\": \"contammix\", \"scope\": \"nuclear\"}}, indent=2))\nPY",
            bam = bam.display(),
            report = report.display(),
            summary = summary.display()
        );
        vec!["/bin/sh".to_string(), "-c".to_string(), command]
    }
}

pub mod schmutzi {
    use super::*;

    #[must_use]
    pub fn args_with_outputs(
        bam: &Path,
        report: &Path,
        summary: &Path,
        _params: &ContaminationEffectiveParams,
    ) -> Vec<String> {
        let command = format!(
            "schmutzi --bam {bam} --outdir {out_dir} && \
if [ -f {out_dir}/contamination.txt ]; then cp {out_dir}/contamination.txt {report}; else : > {report}; fi && \
python - <<'PY' > {summary}\nimport json\nprint(json.dumps({{\"method\": \"schmutzi\", \"scope\": \"mt\"}}, indent=2))\nPY",
            bam = bam.display(),
            out_dir = report
                .parent()
                .map_or_else(|| ".".to_string(), |p| p.display().to_string()),
            report = report.display(),
            summary = summary.display()
        );
        vec!["/bin/sh".to_string(), "-c".to_string(), command]
    }
}

pub mod verifybamid2 {
    use super::*;

    #[must_use]
    pub fn args_with_outputs(
        bam: &Path,
        report: &Path,
        summary: &Path,
        _params: &ContaminationEffectiveParams,
    ) -> Vec<String> {
        let command = format!(
            "verifybamid2 --BamFile {bam} --Output {report} && \
python - <<'PY' > {summary}\nimport json\nprint(json.dumps({{\"method\": \"verifybamid2\", \"scope\": \"nuclear\"}}, indent=2))\nPY",
            bam = bam.display(),
            report = report.display(),
            summary = summary.display()
        );
        vec!["/bin/sh".to_string(), "-c".to_string(), command]
    }
}
