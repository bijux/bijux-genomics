use std::path::Path;

use bijux_dna_domain_bam::types::BedRegions;

use bijux_dna_domain_bam::params::{
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
    let flagstat_before = out_bam.with_file_name("flagstat.before.txt");
    let flagstat_after = out_bam.with_file_name("flagstat.after.txt");
    let idxstats_before = out_bam.with_file_name("idxstats.before.txt");
    let idxstats_after = out_bam.with_file_name("idxstats.after.txt");
    let summary = out_bam.with_file_name("filter.summary.json");
    filter_args_with_audit(
        bam,
        params,
        out_bam,
        &flagstat_before,
        &flagstat_after,
        &idxstats_before,
        &idxstats_after,
        &summary,
    )
}

#[must_use]
#[allow(clippy::too_many_arguments)]
pub fn filter_args_with_audit(
    bam: &Path,
    params: &FilterEffectiveParams,
    out_bam: &Path,
    flagstat_before: &Path,
    flagstat_after: &Path,
    idxstats_before: &Path,
    idxstats_after: &Path,
    summary: &Path,
) -> Vec<String> {
    filter_args_with_audit_and_summary_name(
        bam,
        params,
        out_bam,
        flagstat_before,
        flagstat_after,
        idxstats_before,
        idxstats_after,
        summary,
        "filter",
    )
}

#[must_use]
#[allow(clippy::too_many_arguments)]
pub fn filter_args_with_audit_and_summary_name(
    bam: &Path,
    params: &FilterEffectiveParams,
    out_bam: &Path,
    flagstat_before: &Path,
    flagstat_after: &Path,
    idxstats_before: &Path,
    idxstats_after: &Path,
    summary: &Path,
    action: &str,
) -> Vec<String> {
    let mut exclude_flags = params.exclude_flags.clone();
    if params.remove_duplicates && !exclude_flags.contains(&0x400u16) {
        exclude_flags.push(0x400u16);
    }
    let mut view_args = vec![
        "samtools".to_string(),
        "view".to_string(),
        "-h".to_string(),
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
    if !exclude_flags.is_empty() {
        view_args.push("-F".to_string());
        view_args.push(
            exclude_flags
                .iter()
                .map(std::string::ToString::to_string)
                .collect::<Vec<_>>()
                .join(","),
        );
    }
    view_args.push(bam.display().to_string());
    let bai_path = format!("{}.bai", out_bam.display());
    let length_filter = if params.min_length > 0 {
        format!("awk 'BEGIN{{OFS=\"\\t\"}} /^@/{{print; next}} length($10)>={}'", params.min_length)
    } else {
        "cat".to_string()
    };
    let command = format!(
        "samtools flagstat {bam} > {flagstat_before} && \
samtools idxstats {bam} > {idxstats_before} && \
{view} | {length_filter} | samtools view -b - | samtools sort -@ 1 -l 6 -o {out} && samtools index -@ 1 {out} {bai} && \
samtools flagstat {out} > {flagstat_after} && \
samtools idxstats {out} > {idxstats_after} && \
python - <<'PY' > {summary}\nimport json\npayload = {{\"action\": \"{action}\", \"input_bam\": \"{bam}\", \"output_bam\": \"{out}\", \"params\": {{\"mapq_threshold\": {mapq}, \"min_length\": {min_len}, \"remove_duplicates\": {remove_dup}}}, \"artifacts\": {{\"flagstat_before\": \"{flagstat_before}\", \"flagstat_after\": \"{flagstat_after}\", \"idxstats_before\": \"{idxstats_before}\", \"idxstats_after\": \"{idxstats_after}\"}}}}\nprint(json.dumps(payload, indent=2))\nPY",
        view = view_args.join(" "),
        length_filter = length_filter,
        out = out_bam.display(),
        bai = bai_path,
        bam = bam.display(),
        flagstat_before = flagstat_before.display(),
        flagstat_after = flagstat_after.display(),
        idxstats_before = idxstats_before.display(),
        idxstats_after = idxstats_after.display(),
        summary = summary.display(),
        action = action,
        mapq = params.mapq_threshold,
        min_len = params.min_length,
        remove_dup = if params.remove_duplicates { "true" } else { "false" }
    );
    vec!["/bin/sh".to_string(), "-c".to_string(), command]
}

#[must_use]
pub fn mapping_summary_args(
    bam: &Path,
    flagstat: &Path,
    idxstats: &Path,
    stats: &Path,
    summary: &Path,
) -> Vec<String> {
    let command = format!(
        "samtools flagstat {bam} > {flagstat} && \
samtools idxstats {bam} > {idxstats} && \
samtools stats {bam} > {stats} && \
python - <<'PY' > {summary}\nimport json\nprint(json.dumps({{\"stage\":\"bam.mapping_summary\",\"flagstat\":\"{flagstat}\",\"idxstats\":\"{idxstats}\",\"stats\":\"{stats}\"}}, indent=2))\nPY",
        bam = bam.display(),
        flagstat = flagstat.display(),
        idxstats = idxstats.display(),
        stats = stats.display(),
        summary = summary.display(),
    );
    vec!["/bin/sh".to_string(), "-c".to_string(), command]
}

#[must_use]
pub fn markdup_args(
    bam: &Path,
    out_bam: &Path,
    _flagstat: &Path,
    _idxstats: &Path,
    params: &bijux_dna_domain_bam::params::MarkDupEffectiveParams,
) -> Vec<String> {
    let flagstat_before = out_bam.with_file_name("flagstat.before.txt");
    let flagstat_after = out_bam.with_file_name("flagstat.after.txt");
    let idxstats_before = out_bam.with_file_name("idxstats.before.txt");
    let idxstats_after = out_bam.with_file_name("idxstats.after.txt");
    let summary = out_bam.with_file_name("markdup.summary.json");
    markdup_args_with_audit(
        bam,
        out_bam,
        &flagstat_before,
        &flagstat_after,
        &idxstats_before,
        &idxstats_after,
        &summary,
        params,
    )
}

#[must_use]
#[allow(clippy::too_many_arguments)]
pub fn markdup_args_with_audit(
    bam: &Path,
    out_bam: &Path,
    flagstat_before: &Path,
    flagstat_after: &Path,
    idxstats_before: &Path,
    idxstats_after: &Path,
    summary: &Path,
    params: &bijux_dna_domain_bam::params::MarkDupEffectiveParams,
) -> Vec<String> {
    let remove =
        matches!(params.duplicate_action, bijux_dna_domain_bam::params::DuplicateAction::Remove);
    let mut args = vec!["samtools markdup".to_string()];
    if remove {
        args.push("-r".to_string());
    }
    args.push(bam.display().to_string());
    args.push(out_bam.display().to_string());
    let command = format!(
        "samtools flagstat {bam} > {flagstat_before} && \
samtools idxstats {bam} > {idxstats_before} && \
{markdup} && samtools index {out} {out}.bai && \
samtools flagstat {out} > {flagstat_after} && \
samtools idxstats {out} > {idxstats_after} && \
python - <<'PY' > {summary}\nimport json\npayload = {{\"input_bam\": \"{bam}\", \"output_bam\": \"{out}\", \"remove_duplicates\": {remove}, \"artifacts\": {{\"flagstat_before\": \"{flagstat_before}\", \"flagstat_after\": \"{flagstat_after}\", \"idxstats_before\": \"{idxstats_before}\", \"idxstats_after\": \"{idxstats_after}\"}}}}\nprint(json.dumps(payload, indent=2))\nPY",
        markdup = args.join(" "),
        out = out_bam.display(),
        bam = bam.display(),
        flagstat_before = flagstat_before.display(),
        flagstat_after = flagstat_after.display(),
        idxstats_before = idxstats_before.display(),
        idxstats_after = idxstats_after.display(),
        summary = summary.display(),
        remove = if remove { "true" } else { "false" }
    );
    vec!["/bin/sh".to_string(), "-c".to_string(), command]
}

#[must_use]
pub fn depth_args(
    bam: &Path,
    depth: &Path,
    summary: &Path,
    regions: Option<&BedRegions>,
) -> Vec<String> {
    let regions_arg = regions
        .map_or_else(String::new, |regions| format!("-b {} ", regions.as_path().display()));
    let command = format!(
        "samtools depth -a {regions_arg}{bam} > {depth} && \
awk '{{sum+=$3; if($3>0) cov++}} END {{mean=(NR>0)?sum/NR:0; print \"total\", NR, cov, mean}}' {depth} > {summary}",
        regions_arg = regions_arg,
        bam = bam.display(),
        depth = depth.display(),
        summary = summary.display()
    );
    vec!["/bin/sh".to_string(), "-c".to_string(), command]
}
