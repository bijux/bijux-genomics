use std::path::Path;

use bijux_dna_domain_bam::params::AuthenticityEffectiveParams;

#[must_use]
pub fn args_with_outputs(
    bam: &Path,
    report: &Path,
    summary: &Path,
    params: &AuthenticityEffectiveParams,
) -> Vec<String> {
    let command = format!(
        "samtools flagstat {bam} > {flagstat} && \
samtools stats {bam} > {stats} && \
python - <<'PY' {flagstat} {stats} > {report}\nimport json,sys\nflagstat,stats=sys.argv[1],sys.argv[2]\nprint(json.dumps({{\"method\":\"signal_aggregate\",\"flagstat\":flagstat,\"stats\":stats,\"mode\":\"{mode}\"}}, indent=2))\nPY && \
python - <<'PY' > {summary}\nimport json\nprint(json.dumps({{\"method\": \"signal_aggregate\", \"mode\": \"{mode}\", \"status\": \"ok\"}}, indent=2))\nPY",
        bam = bam.display(),
        mode = params.mode,
        flagstat = report.with_file_name("authenticity.flagstat.txt").display(),
        stats = report.with_file_name("authenticity.stats.txt").display(),
        report = report.display(),
        summary = summary.display(),
    );
    vec!["/bin/sh".to_string(), "-c".to_string(), command]
}
