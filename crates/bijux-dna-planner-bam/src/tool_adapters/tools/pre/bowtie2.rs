use std::path::Path;

use bijux_dna_domain_bam::params::AlignEffectiveParams;

fn rg_string(params: &AlignEffectiveParams) -> String {
    let mut rg = format!(
        "@RG\\tID:{}\\tSM:{}\\tPL:{}\\tLB:{}",
        params.read_group.id,
        params.read_group.sample,
        params.read_group.platform,
        params.read_group.library
    );
    if let Some(platform_unit) = params.read_group.platform_unit.as_deref() {
        rg.push_str(&format!("\\tPU:{platform_unit}"));
    }
    rg
}

fn preset_flags(preset: &str) -> &'static str {
    match preset {
        // aDNA-friendly: local alignment, shorter seed, allow one mismatch in seed.
        "adna_short" | "adna_sensitive" => "--very-sensitive-local -N 1 -L 20",
        // eDNA/pollen/metagenomic-like: local + very-sensitive to preserve reads.
        "edna_metagenomic" => "--very-sensitive-local -N 1 -L 22",
        _ => "--very-sensitive",
    }
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
    let mut preset_flags = preset_flags(&params.preset).to_string();
    if let Some(seed_length) = params.seed_length {
        preset_flags.push_str(&format!(" -L {seed_length}"));
    }
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
            "bowtie2 -x {idx} -1 {r1} -2 {r2} {preset_flags} --rg '{rg}' --rg-id {rgid} -p {threads}",
            idx = index_prefix,
            r1 = r1.display(),
            r2 = r2.display(),
            preset_flags = preset_flags,
            rg = rg,
            rgid = params.read_group.id,
            threads = params.threads
        )
    } else {
        format!(
            "bowtie2 -x {idx} -U {r1} {preset_flags} --rg '{rg}' --rg-id {rgid} -p {threads}",
            idx = index_prefix,
            r1 = r1.display(),
            preset_flags = preset_flags,
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
    python - <<'PY' > {metrics}\nimport json\npayload={{\"tool\":\"bowtie2\",\"preset\":\"{preset}\",\"sensitivity_profile\":{sensitivity_profile},\"seed_length\":{seed_length},\"reference\":\"{ref}\",\"bam\":\"{out}\",\"read_group\":\"{rg}\"}}\nprint(json.dumps(payload, indent=2))\nPY",
        build = build_index,
        align = align,
        out = out_bam.display(),
        flagstat = flagstat.display(),
        idxstats = idxstats.display(),
        stats = stats.display(),
        metrics = metrics.display(),
        preset = params.preset,
        sensitivity_profile = serde_json::to_string(&params.sensitivity_profile).unwrap_or_else(|_| "null".to_string()),
        seed_length = params.seed_length.map_or_else(|| "null".to_string(), |value| value.to_string()),
        ref = reference.display(),
        rg = rg
    );
    vec!["/bin/sh".to_string(), "-c".to_string(), command]
}
