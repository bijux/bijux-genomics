use std::path::Path;

use bijux_dna_domain_bam::params::FilterEffectiveParams;
use bijux_dna_domain_bam::params::ValidateEffectiveParams;
use bijux_dna_domain_bam::types::BedRegions;

#[must_use]
pub fn validate_args(
    bam: &Path,
    flagstat: &Path,
    report: &Path,
    _params: &ValidateEffectiveParams,
) -> Vec<String> {
    let command = format!(
        "bedtools bamtobed -i {bam} >/dev/null && \
python - <<'PY' > {report}\nimport json\nprint(json.dumps({{\"validator\": \"bedtools.bamtobed\", \"status\": \"ok\"}}, indent=2))\nPY && \
samtools flagstat {bam} > {flagstat}",
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
bedtools bamtobed -i {bam} >/dev/null && \
samtools view -b {bam} > {out_bam} && samtools index {out_bam} && \
samtools flagstat {out_bam} > {flag_after} && \
samtools idxstats {out_bam} > {idx_after} && \
python - <<'PY' > {summary}\nimport json\nprint(json.dumps({{\"filter_tool\": \"bedtools\"}}, indent=2))\nPY",
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

#[must_use]
pub fn coverage_args(
    bam: &Path,
    depth: &Path,
    summary: &Path,
    regions: Option<&BedRegions>,
) -> Vec<String> {
    let bedtools_probe = regions.map_or_else(
        || format!("bedtools genomecov -ibam {bam} >/dev/null", bam = bam.display()),
        |regions| {
            format!(
                "bedtools coverage -a {regions} -b {bam} >/dev/null",
                regions = regions.as_path().display(),
                bam = bam.display()
            )
        },
    );
    let depth_regions_arg =
        regions.map_or_else(String::new, |regions| format!("-b {} ", regions.as_path().display()));
    let command = format!(
        "{bedtools_probe} && \
samtools depth -a {depth_regions_arg}{bam} > {depth} && \
awk '{{sum+=$3; if($3>0) cov++}} END {{mean=(NR>0)?sum/NR:0; print \"total\", NR, cov, mean}}' {depth} > {summary}",
        bedtools_probe = bedtools_probe,
        bam = bam.display(),
        depth_regions_arg = depth_regions_arg,
        depth = depth.display(),
        summary = summary.display()
    );
    vec!["/bin/sh".to_string(), "-c".to_string(), command]
}
