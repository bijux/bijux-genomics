use std::path::Path;

use bijux_domain_bam::params::{
    ContaminationEffectiveParams, CoverageEffectiveParams, DamageEffectiveParams,
    FilterEffectiveParams, QcPreEffectiveParams, ValidateEffectiveParams,
};

pub fn samtools_validate_args(bam: &Path, _params: &ValidateEffectiveParams) -> Vec<String> {
    vec![
        "samtools".to_string(),
        "quickcheck".to_string(),
        "-v".to_string(),
        bam.display().to_string(),
    ]
}

pub fn samtools_qc_pre_args(bam: &Path, _params: &QcPreEffectiveParams) -> Vec<String> {
    vec![
        "samtools".to_string(),
        "flagstat".to_string(),
        bam.display().to_string(),
    ]
}

pub fn samtools_filter_args(
    bam: &Path,
    params: &FilterEffectiveParams,
    out_bam: &Path,
) -> Vec<String> {
    let mut args = vec![
        "samtools".to_string(),
        "view".to_string(),
        "-b".to_string(),
        "-q".to_string(),
        params.mapq_threshold.to_string(),
    ];
    if !params.include_flags.is_empty() {
        args.push("-f".to_string());
        args.push(
            params
                .include_flags
                .iter()
                .map(std::string::ToString::to_string)
                .collect::<Vec<_>>()
                .join(","),
        );
    }
    if !params.exclude_flags.is_empty() {
        args.push("-F".to_string());
        args.push(
            params
                .exclude_flags
                .iter()
                .map(std::string::ToString::to_string)
                .collect::<Vec<_>>()
                .join(","),
        );
    }
    args.push("-o".to_string());
    args.push(out_bam.display().to_string());
    args.push(bam.display().to_string());
    args
}

pub fn mosdepth_args(
    bam: &Path,
    out_prefix: &Path,
    _params: &CoverageEffectiveParams,
) -> Vec<String> {
    vec![
        "mosdepth".to_string(),
        "-n".to_string(),
        out_prefix.display().to_string(),
        bam.display().to_string(),
    ]
}

pub fn pydamage_args(bam: &Path, out_json: &Path, params: &DamageEffectiveParams) -> Vec<String> {
    vec![
        "pydamage".to_string(),
        "analyze".to_string(),
        "--input".to_string(),
        bam.display().to_string(),
        "--output".to_string(),
        out_json.display().to_string(),
        "--min-mapq".to_string(),
        params.pmd_threshold_5p.to_string(),
    ]
}

pub fn contamination_args(bam: &Path, _params: &ContaminationEffectiveParams) -> Vec<String> {
    vec![
        "contamination_tool".to_string(),
        "--input".to_string(),
        bam.display().to_string(),
    ]
}
