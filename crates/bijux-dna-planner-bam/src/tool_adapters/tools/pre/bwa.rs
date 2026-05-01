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

#[must_use]
#[allow(clippy::too_many_lines)]
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
    let build_index = if params.build_indices {
        format!(
            "if [ ! -f {ref}.fai ]; then samtools faidx {ref}; fi; \
        if [ ! -f {ref}.dict ]; then gatk CreateSequenceDictionary -R {ref} -O {ref}.dict; fi; \
        if [ ! -f {ref}.bwt ]; then bwa index {ref}; fi;",
            ref = reference.display()
        )
    } else {
        String::new()
    };
    let command = if params.preset == "adna_short" {
        let sai_r1 = out_dir.join("r1.sai");
        let r1_cmd = format!(
            "bwa aln -l 1024 -n 0.01 -t {threads} {ref} {r1} > {sai}",
            threads = params.threads,
            ref = reference.display(),
            r1 = r1.display(),
            sai = sai_r1.display()
        );
        if let Some(r2) = r2 {
            let sai_r2 = out_dir.join("r2.sai");
            format!(
                "{build}{r1_cmd} && \
            bwa aln -l 1024 -n 0.01 -t {threads} {ref} {r2} > {sai_r2} && \
            bwa sampe -r '{rg}' {ref} {sai_r1} {sai_r2} {r1} {r2} | samtools sort -o {out} && \
            samtools index {out} && \
            samtools flagstat {out} > {flagstat} && samtools idxstats {out} > {idxstats} && \
            samtools stats {out} > {stats} && \
            python - <<'PY' > {metrics}\nimport json\npayload={{\"tool\":\"bwa_aln\",\"preset\":\"adna_short\",\"reference\":\"{ref}\",\"bam\":\"{out}\",\"read_group\":\"{rg}\"}}\nprint(json.dumps(payload, indent=2))\nPY",
                build = build_index,
                r1_cmd = r1_cmd,
                threads = params.threads,
                ref = reference.display(),
                r2 = r2.display(),
                sai_r1 = sai_r1.display(),
                sai_r2 = sai_r2.display(),
                r1 = r1.display(),
                out = out_bam.display(),
                flagstat = flagstat.display(),
                idxstats = idxstats.display(),
                stats = stats.display(),
                metrics = metrics.display(),
                rg = rg
            )
        } else {
            format!(
                "{build}{r1_cmd} && \
            bwa samse -r '{rg}' {ref} {sai_r1} {r1} | samtools sort -o {out} && \
            samtools index {out} && \
            samtools flagstat {out} > {flagstat} && samtools idxstats {out} > {idxstats} && \
            samtools stats {out} > {stats} && \
            python - <<'PY' > {metrics}\nimport json\npayload={{\"tool\":\"bwa_aln\",\"preset\":\"adna_short\",\"reference\":\"{ref}\",\"bam\":\"{out}\",\"read_group\":\"{rg}\"}}\nprint(json.dumps(payload, indent=2))\nPY",
                build = build_index,
                r1_cmd = r1_cmd,
                ref = reference.display(),
                sai_r1 = sai_r1.display(),
                r1 = r1.display(),
                out = out_bam.display(),
                flagstat = flagstat.display(),
                idxstats = idxstats.display(),
                stats = stats.display(),
                metrics = metrics.display(),
                rg = rg
            )
        }
    } else if let Some(r2) = r2 {
        let seed_flag = params.seed_length.map_or_else(String::new, |seed| format!(" -k {seed}"));
        format!(
            "{build}bwa mem -t {threads}{seed_flag} -R '{rg}' {ref} {r1} {r2} | samtools sort -o {out} && \
        samtools index {out} && \
        samtools flagstat {out} > {flagstat} && samtools idxstats {out} > {idxstats} && \
        samtools stats {out} > {stats} && \
        python - <<'PY' > {metrics}\nimport json\npayload={{\"tool\":\"bwa_mem\",\"preset\":\"{preset}\",\"sensitivity_profile\":{sensitivity_profile},\"seed_length\":{seed_length},\"reference\":\"{ref}\",\"bam\":\"{out}\",\"read_group\":\"{rg}\"}}\nprint(json.dumps(payload, indent=2))\nPY",
            build = build_index,
            threads = params.threads,
            seed_flag = seed_flag,
            rg = rg,
            ref = reference.display(),
            r1 = r1.display(),
            r2 = r2.display(),
            out = out_bam.display(),
            flagstat = flagstat.display(),
            idxstats = idxstats.display(),
            stats = stats.display(),
            metrics = metrics.display(),
            preset = params.preset,
            sensitivity_profile = serde_json::to_string(&params.sensitivity_profile).unwrap_or_else(|_| "null".to_string()),
            seed_length = params.seed_length.map_or_else(|| "null".to_string(), |value| value.to_string())
        )
    } else {
        let seed_flag = params.seed_length.map_or_else(String::new, |seed| format!(" -k {seed}"));
        format!(
            "{build}bwa mem -t {threads}{seed_flag} -R '{rg}' {ref} {r1} | samtools sort -o {out} && \
        samtools index {out} && \
        samtools flagstat {out} > {flagstat} && samtools idxstats {out} > {idxstats} && \
        samtools stats {out} > {stats} && \
        python - <<'PY' > {metrics}\nimport json\npayload={{\"tool\":\"bwa_mem\",\"preset\":\"{preset}\",\"sensitivity_profile\":{sensitivity_profile},\"seed_length\":{seed_length},\"reference\":\"{ref}\",\"bam\":\"{out}\",\"read_group\":\"{rg}\"}}\nprint(json.dumps(payload, indent=2))\nPY",
            build = build_index,
            threads = params.threads,
            seed_flag = seed_flag,
            rg = rg,
            ref = reference.display(),
            r1 = r1.display(),
            out = out_bam.display(),
            flagstat = flagstat.display(),
            idxstats = idxstats.display(),
            stats = stats.display(),
            metrics = metrics.display(),
            preset = params.preset,
            sensitivity_profile = serde_json::to_string(&params.sensitivity_profile).unwrap_or_else(|_| "null".to_string()),
            seed_length = params.seed_length.map_or_else(|| "null".to_string(), |value| value.to_string())
        )
    };

    vec!["/bin/sh".to_string(), "-c".to_string(), command]
}
