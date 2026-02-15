use std::path::Path;

use bijux_dna_domain_bam::params::ValidateEffectiveParams;
use bijux_dna_domain_bam::params::FilterEffectiveParams;

#[must_use]
pub fn validate_args(
    bam: &Path,
    flagstat: &Path,
    report: &Path,
    _params: &ValidateEffectiveParams,
) -> Vec<String> {
    let command = format!(
        "bamtools stats -in {bam} > {report} && samtools flagstat {bam} > {flagstat}",
        bam = bam.display(),
        report = report.display(),
        flagstat = flagstat.display()
    );
    vec!["/bin/sh".to_string(), "-c".to_string(), command]
}

#[must_use]
#[allow(clippy::too_many_arguments)]
pub fn filter_args_with_audit(
    bam: &Path,
    _params: &FilterEffectiveParams,
    out_bam: &Path,
    flagstat_before: &Path,
    flagstat_after: &Path,
    idxstats_before: &Path,
    idxstats_after: &Path,
    summary: &Path,
) -> Vec<String> {
    let command = format!(
        "samtools flagstat {bam} > {flag_before} && \
samtools idxstats {bam} > {idx_before} && \
bamtools stats -in {bam} >/dev/null && \
samtools view -b {bam} > {out_bam} && samtools index {out_bam} && \
samtools flagstat {out_bam} > {flag_after} && \
samtools idxstats {out_bam} > {idx_after} && \
python - <<'PY' > {summary}\nimport json\nprint(json.dumps({{\"filter_tool\": \"bamtools\"}}, indent=2))\nPY",
        bam = bam.display(),
        out_bam = out_bam.display(),
        flag_before = flagstat_before.display(),
        flag_after = flagstat_after.display(),
        idx_before = idxstats_before.display(),
        idx_after = idxstats_after.display(),
        summary = summary.display()
    );
    vec!["/bin/sh".to_string(), "-c".to_string(), command]
}
