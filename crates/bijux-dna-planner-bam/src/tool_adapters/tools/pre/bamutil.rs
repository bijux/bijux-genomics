use std::path::Path;

#[must_use]
pub fn overlap_correction_args_with_audit(
    bam: &Path,
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
bam clipOverlap --in {bam} --out {out_bam} --stats > {summary}.clipoverlap.log 2>&1 && \
samtools index -@ 1 {out_bam} {out_bam}.bai && \
samtools flagstat {out_bam} > {flag_after} && \
samtools idxstats {out_bam} > {idx_after} && \
python - <<'PY' > {summary}\nimport json\nprint(json.dumps({{\"method\": \"bamutil.clipOverlap\", \"input\": \"{bam}\", \"output\": \"{out_bam}\"}}, indent=2))\nPY",
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
