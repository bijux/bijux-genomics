use std::path::Path;

use bijux_dna_domain_bam::params::KinshipEffectiveParams;

#[must_use]
pub fn args_with_outputs(
    tool_id: &str,
    bam: &Path,
    report: &Path,
    summary: &Path,
    segments_tsv: &Path,
    params: &KinshipEffectiveParams,
) -> Vec<String> {
    let command = match tool_id {
        "king" => format!(
            "king -b {bam} --kinship --related --prefix {prefix} && \
if [ -f {prefix}.kin0 ]; then cp {prefix}.kin0 {segments}; else : > {segments}; fi && \
python - <<'PY' > {summary}\nimport json\nprint(json.dumps({{\"method\": \"king\", \"reference_panel\": \"{panel}\", \"min_overlap_snps\": {min_overlap}}}, indent=2))\nPY && \
python - <<'PY' > {report}\nimport json\nprint(json.dumps({{\"status\": \"ok\", \"tool\": \"king\", \"segments\": \"{segments}\"}}, indent=2))\nPY",
            bam = bam.display(),
            prefix = report.with_extension("").display(),
            segments = segments_tsv.display(),
            summary = summary.display(),
            report = report.display(),
            panel = params.reference_panel,
            min_overlap = params.min_overlap_snps
        ),
        _ => format!(
            "angsd -i {bam} -doIBS 1 -out {prefix} && \
if [ -f {prefix}.ibs ]; then cp {prefix}.ibs {segments}; else : > {segments}; fi && \
python - <<'PY' > {summary}\nimport json\nprint(json.dumps({{\"method\": \"angsd\", \"reference_panel\": \"{panel}\", \"min_overlap_snps\": {min_overlap}}}, indent=2))\nPY && \
python - <<'PY' > {report}\nimport json\nprint(json.dumps({{\"status\": \"ok\", \"tool\": \"angsd\", \"segments\": \"{segments}\"}}, indent=2))\nPY",
            bam = bam.display(),
            prefix = report.with_extension("").display(),
            segments = segments_tsv.display(),
            summary = summary.display(),
            report = report.display(),
            panel = params.reference_panel,
            min_overlap = params.min_overlap_snps
        ),
    };
    vec!["/bin/sh".to_string(), "-c".to_string(), command]
}
