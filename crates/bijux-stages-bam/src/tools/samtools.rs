use std::path::Path;

use bijux_domain_bam::params::{
    FilterEffectiveParams, QcPreEffectiveParams, ValidateEffectiveParams,
};

#[must_use]
pub fn validate_args(
    bam: &Path,
    flagstat: &Path,
    report: &Path,
    _params: &ValidateEffectiveParams,
) -> Vec<String> {
    let command = format!(
        "samtools quickcheck -v {bam} > {report} && samtools flagstat {bam} > {flagstat}",
        bam = bam.display(),
        report = report.display(),
        flagstat = flagstat.display()
    );
    vec!["/bin/sh".to_string(), "-c".to_string(), command]
}

#[must_use]
pub fn qc_pre_args(
    bam: &Path,
    flagstat: &Path,
    idxstats: &Path,
    stats: &Path,
    _params: &QcPreEffectiveParams,
) -> Vec<String> {
    let command = format!(
        "samtools flagstat {bam} > {flagstat} && samtools idxstats {bam} > {idxstats} && samtools stats {bam} > {stats}",
        bam = bam.display(),
        flagstat = flagstat.display(),
        idxstats = idxstats.display(),
        stats = stats.display()
    );
    vec!["/bin/sh".to_string(), "-c".to_string(), command]
}

#[must_use]
pub fn filter_args(bam: &Path, params: &FilterEffectiveParams, out_bam: &Path) -> Vec<String> {
    let mut view_args = vec![
        "samtools".to_string(),
        "view".to_string(),
        "-b".to_string(),
        "-q".to_string(),
        params.mapq_threshold.to_string(),
    ];
    if !params.include_flags.is_empty() {
        view_args.push("-f".to_string());
        view_args.push(
            params
                .include_flags
                .iter()
                .map(std::string::ToString::to_string)
                .collect::<Vec<_>>()
                .join(","),
        );
    }
    if !params.exclude_flags.is_empty() {
        view_args.push("-F".to_string());
        view_args.push(
            params
                .exclude_flags
                .iter()
                .map(std::string::ToString::to_string)
                .collect::<Vec<_>>()
                .join(","),
        );
    }
    view_args.push(bam.display().to_string());
    let bai_path = format!("{}.bai", out_bam.display());
    let command = format!(
        "{view} | samtools sort -o {out} && samtools index {out} {bai}",
        view = view_args.join(" "),
        out = out_bam.display(),
        bai = bai_path
    );
    vec!["/bin/sh".to_string(), "-c".to_string(), command]
}

#[must_use]
pub fn markdup_args(
    bam: &Path,
    out_bam: &Path,
    flagstat: &Path,
    idxstats: &Path,
    params: &bijux_domain_bam::params::MarkDupEffectiveParams,
) -> Vec<String> {
    let remove = matches!(params.duplicate_action, bijux_domain_bam::params::DuplicateAction::Remove);
    let mut args = vec!["samtools markdup".to_string()];
    if remove {
        args.push("-r".to_string());
    }
    args.push(bam.display().to_string());
    args.push(out_bam.display().to_string());
    let command = format!(
        "{markdup} && samtools index {out} {out}.bai && samtools flagstat {out} > {flagstat} && samtools idxstats {out} > {idxstats}",
        markdup = args.join(" "),
        out = out_bam.display(),
        flagstat = flagstat.display(),
        idxstats = idxstats.display()
    );
    vec!["/bin/sh".to_string(), "-c".to_string(), command]
}

#[must_use]
pub fn depth_args(bam: &Path, depth: &Path) -> Vec<String> {
    let command = format!(
        "samtools depth -a {bam} > {depth}",
        bam = bam.display(),
        depth = depth.display()
    );
    vec!["/bin/sh".to_string(), "-c".to_string(), command]
}
