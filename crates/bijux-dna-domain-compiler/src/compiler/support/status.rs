use super::*;

pub(crate) fn ensure_status(status: &str, path: &Path) -> Result<()> {
    match status {
        "supported" | "planned" | "out_of_scope" => Ok(()),
        _ => Err(anyhow!(
            "{} invalid status `{status}` (expected supported|planned|out_of_scope)",
            path.display()
        )),
    }
}

pub(crate) fn scope_active(entry_scope: &str, active_scope: &str) -> bool {
    entry_scope == active_scope
}

pub(crate) fn is_tool_meaningful_in_domain(domain: &str, tool_id: &str) -> bool {
    const FASTQ_FORBIDDEN: &[&str] = &[
        "bcftools",
        "picard",
        "gatk",
        "preseq",
        "schmutzi",
        "verifybamid2",
        "contammix",
    ];
    const BAM_FORBIDDEN: &[&str] = &[
        "cutadapt",
        "fastp",
        "trimmomatic",
        "adapterremoval",
        "fastqc",
        "kraken2",
        "bracken",
        "krakenuniq",
    ];
    match domain {
        "fastq" => !FASTQ_FORBIDDEN.contains(&tool_id),
        "bam" => !BAM_FORBIDDEN.contains(&tool_id),
        _ => true,
    }
}

pub(crate) fn is_umbrella_stage(stage_id: &str) -> bool {
    matches!(stage_id, "fastq.preprocess" | "bam.preprocess")
}
