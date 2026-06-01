use std::path::Path;

use bijux_dna_domain_bam::params::{BqsrEffectiveParams, MarkDupEffectiveParams};

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
    let remove =
        matches!(params.duplicate_action, bijux_dna_domain_bam::params::DuplicateAction::Remove);
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

#[must_use]
pub fn recalibration_args_with_outputs(
    bam: &Path,
    reference: Option<&Path>,
    out_bam: &Path,
    out_bai: &Path,
    recal_report: &Path,
    summary: &Path,
    params: &BqsrEffectiveParams,
) -> Vec<String> {
    let known_sites = params
        .known_sites
        .iter()
        .map(|path| format!("--known-sites {path}"))
        .collect::<Vec<_>>()
        .join(" ");
    let mode = match params.mode {
        bijux_dna_domain_bam::params::BqsrMode::Standard => "standard",
        bijux_dna_domain_bam::params::BqsrMode::Skip => "skip",
        bijux_dna_domain_bam::params::BqsrMode::EmitOnly => "emit_only",
    };
    let known_sites_json = serde_json::to_string(
        &params.known_sites.iter().map(std::clone::Clone::clone).collect::<Vec<_>>(),
    )
    .unwrap_or_else(|_| "[]".to_string());
    let reference_json = serde_json::to_string(&reference.map(|path| path.display().to_string()))
        .unwrap_or_else(|_| "null".to_string());
    let command = match params.mode {
        bijux_dna_domain_bam::params::BqsrMode::Skip => format!(
            "cp {bam} {out} && \
printf 'tiny-index\\n' > {bai} && \
cat <<'EOF' > {report}\nstatus=skipped\nreason=requested_skip_mode\nEOF\n\
python - <<'PY' > {summary}\nimport json\nprint(json.dumps({{\"mode\": \"{mode}\", \"status\": \"skipped\", \"reason\": \"requested_skip_mode\", \"known_sites\": {known_sites_json}, \"reference\": {reference_json}, \"recalibration_report\": \"{report}\", \"output_bam\": \"{out}\", \"output_bai\": \"{bai}\"}}, indent=2))\nPY",
            bam = bam.display(),
            out = out_bam.display(),
            bai = out_bai.display(),
            report = recal_report.display(),
            summary = summary.display(),
            mode = mode,
            known_sites_json = known_sites_json,
            reference_json = reference_json,
        ),
        bijux_dna_domain_bam::params::BqsrMode::EmitOnly => format!(
            "cp {bam} {out} && \
printf 'tiny-index\\n' > {bai} && \
cat <<'EOF' > {report}\nstatus=emit_only\nreason=emit_only_requested\nEOF\n\
python - <<'PY' > {summary}\nimport json\nprint(json.dumps({{\"mode\": \"{mode}\", \"status\": \"emitted_only\", \"reason\": \"emit_only_requested\", \"known_sites\": {known_sites_json}, \"reference\": {reference_json}, \"recalibration_report\": \"{report}\", \"output_bam\": \"{out}\", \"output_bai\": \"{bai}\"}}, indent=2))\nPY",
            bam = bam.display(),
            out = out_bam.display(),
            bai = out_bai.display(),
            report = recal_report.display(),
            summary = summary.display(),
            mode = mode,
            known_sites_json = known_sites_json,
            reference_json = reference_json,
        ),
        bijux_dna_domain_bam::params::BqsrMode::Standard => {
            if reference.is_none() {
                format!(
                    "python - <<'PY' > {summary}\nimport json\nprint(json.dumps({{\"mode\": \"{mode}\", \"status\": \"refused\", \"reason\": \"missing_reference_context\", \"known_sites\": {known_sites_json}, \"reference\": null, \"recalibration_report\": \"{report}\", \"output_bam\": \"{out}\", \"output_bai\": \"{bai}\"}}, indent=2))\nPY\n\
echo 'bam.recalibration requires reference context for standard mode' >&2\nexit 2",
                    out = out_bam.display(),
                    bai = out_bai.display(),
                    report = recal_report.display(),
                    summary = summary.display(),
                    mode = mode,
                    known_sites_json = known_sites_json,
                )
            } else if params.known_sites.is_empty() {
                format!(
                    "python - <<'PY' > {summary}\nimport json\nprint(json.dumps({{\"mode\": \"{mode}\", \"status\": \"refused\", \"reason\": \"missing_known_sites\", \"known_sites\": [], \"reference\": {reference_json}, \"recalibration_report\": \"{report}\", \"output_bam\": \"{out}\", \"output_bai\": \"{bai}\"}}, indent=2))\nPY\n\
echo 'bam.recalibration requires at least one known-sites resource for standard mode' >&2\nexit 2",
                    out = out_bam.display(),
                    bai = out_bai.display(),
                    report = recal_report.display(),
                    summary = summary.display(),
                    mode = mode,
                    reference_json = reference_json,
                )
            } else {
                let reference = reference.expect("reference already checked");
                format!(
                    "gatk BaseRecalibrator -I {bam} -R {reference} {known_sites} -O {report} && \
gatk ApplyBQSR -I {bam} -R {reference} --bqsr-recal-file {report} -O {out} && \
samtools index {out} {bai} && \
python - <<'PY' > {summary}\nimport json\nprint(json.dumps({{\"mode\": \"{mode}\", \"status\": \"ran\", \"reason\": \"standard_mode_requested\", \"known_sites\": {known_sites_json}, \"reference\": {reference_json}, \"recalibration_report\": \"{report}\", \"output_bam\": \"{out}\", \"output_bai\": \"{bai}\"}}, indent=2))\nPY",
                    bam = bam.display(),
                    reference = reference.display(),
                    known_sites = known_sites,
                    report = recal_report.display(),
                    out = out_bam.display(),
                    bai = out_bai.display(),
                    summary = summary.display(),
                    mode = mode,
                    known_sites_json = known_sites_json,
                    reference_json = reference_json,
                )
            }
        }
    };
    vec!["/bin/sh".to_string(), "-c".to_string(), command]
}
