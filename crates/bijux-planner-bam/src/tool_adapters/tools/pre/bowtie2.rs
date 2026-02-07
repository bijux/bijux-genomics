use std::path::Path;

use bijux_domain_bam::params::AlignEffectiveParams;

fn rg_string(params: &AlignEffectiveParams) -> String {
    format!(
        "@RG\\tID:{}\\tSM:{}\\tPL:{}\\tLB:{}",
        params.read_group.id,
        params.read_group.sample,
        params.read_group.platform,
        params.read_group.library
    )
}

#[must_use]
pub fn align_args(
    reference: &Path,
    r1: &Path,
    r2: Option<&Path>,
    out_dir: &Path,
    params: &AlignEffectiveParams,
) -> Vec<String> {
    let out_bam = out_dir.join("align.bam");
    let flagstat = out_dir.join("flagstat.txt");
    let idxstats = out_dir.join("idxstats.txt");
    let stats = out_dir.join("samtools_stats.txt");
    let metrics = out_dir.join("align.metrics.json");
    let rg = rg_string(params);
    let index_prefix = reference.display();
    let build_index = if params.build_indices {
        format!(
            "if [ ! -f {ref}.fai ]; then samtools faidx {ref}; fi; \
        if [ ! -f {ref}.dict ]; then gatk CreateSequenceDictionary -R {ref} -O {ref}.dict; fi; \
        if [ ! -f {ref}.1.bt2 ]; then bowtie2-build {ref} {ref}; fi;",
            ref = reference.display()
        )
    } else {
        String::new()
    };
    let align = if let Some(r2) = r2 {
        format!(
            "bowtie2 -x {idx} -1 {r1} -2 {r2} --rg '{rg}' --rg-id {rgid} -p {threads}",
            idx = index_prefix,
            r1 = r1.display(),
            r2 = r2.display(),
            rg = rg,
            rgid = params.read_group.id,
            threads = params.threads
        )
    } else {
        format!(
            "bowtie2 -x {idx} -U {r1} --rg '{rg}' --rg-id {rgid} -p {threads}",
            idx = index_prefix,
            r1 = r1.display(),
            rg = rg,
            rgid = params.read_group.id,
            threads = params.threads
        )
    };
    let command = format!(
        "{build}{align} | samtools sort -o {out} && \
    samtools index {out} && \
    samtools flagstat {out} > {flagstat} && samtools idxstats {out} > {idxstats} && \
    samtools stats {out} > {stats} && \
    python - <<'PY' > {metrics}\nimport json\npayload={{\"tool\":\"bowtie2\",\"preset\":\"{preset}\",\"reference\":\"{ref}\",\"bam\":\"{out}\",\"read_group\":\"{rg}\"}}\nprint(json.dumps(payload, indent=2))\nPY",
        build = build_index,
        align = align,
        out = out_bam.display(),
        flagstat = flagstat.display(),
        idxstats = idxstats.display(),
        stats = stats.display(),
        metrics = metrics.display(),
        preset = params.preset,
        ref = reference.display(),
        rg = rg
    );
    vec!["/bin/sh".to_string(), "-c".to_string(), command]
}
