use std::path::Path;

use bijux_dna_domain_bam::params::MarkDupEffectiveParams;

#[must_use]
pub fn markdup_args(
    bam: &Path,
    out_bam: &Path,
    _flagstat: &Path,
    _idxstats: &Path,
    params: &MarkDupEffectiveParams,
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
    params: &MarkDupEffectiveParams,
) -> Vec<String> {
    let remove = matches!(
        params.duplicate_action,
        bijux_dna_domain_bam::params::DuplicateAction::Remove
    );
    let command = format!(
        "samtools flagstat {bam} > {flagstat_before} && \
samtools idxstats {bam} > {idxstats_before} && \
gatk MarkDuplicatesSpark -I {bam} -O {out} --REMOVE_DUPLICATES {remove} --CREATE_INDEX true && \
samtools flagstat {out} > {flagstat_after} && samtools idxstats {out} > {idxstats_after} && \
python - <<'PY' > {summary}\nimport json\npayload = {{\"input_bam\": \"{bam}\", \"output_bam\": \"{out}\", \"remove_duplicates\": {remove}, \"artifacts\": {{\"flagstat_before\": \"{flagstat_before}\", \"flagstat_after\": \"{flagstat_after}\", \"idxstats_before\": \"{idxstats_before}\", \"idxstats_after\": \"{idxstats_after}\"}}}}\nprint(json.dumps(payload, indent=2))\nPY",
        bam = bam.display(),
        out = out_bam.display(),
        remove = if remove { "true" } else { "false" },
        flagstat_before = flagstat_before.display(),
        flagstat_after = flagstat_after.display(),
        idxstats_before = idxstats_before.display(),
        idxstats_after = idxstats_after.display(),
        summary = summary.display()
    );
    vec!["/bin/sh".to_string(), "-c".to_string(), command]
}
