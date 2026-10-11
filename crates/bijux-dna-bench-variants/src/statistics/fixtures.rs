use anyhow::{Context, Result};
use std::path::Path;
pub(super) fn write_governed_stats_input_vcf(path: &Path) -> Result<()> {
    let payload = "\
##fileformat=VCFv4.2\n\
##reference=bijux-stats-smoke\n\
##contig=<ID=chr1,length=24>\n\
##INFO=<ID=DP,Number=1,Type=Integer,Description=\"Read depth\">\n\
##FORMAT=<ID=GT,Number=1,Type=String,Description=\"Genotype\">\n\
#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\tsample_a\tsample_b\n\
chr1\t3\trs_transition_1\tA\tG\t60\tPASS\tDP=12\tGT\t0/1\t0/0\n\
chr1\t5\trs_transition_2\tC\tT\t62\tPASS\tDP=14\tGT\t1/1\t0/1\n\
chr1\t7\trs_transversion\tA\tT\t64\tPASS\tDP=16\tGT\t0/1\t1/1\n\
chr1\t9\trs_indel\tAT\tA\t58\tPASS\tDP=18\tGT\t0/1\t0/0\n";
    if let Some(parent) = path.parent() {
        bijux_dna_infra::ensure_dir(parent)
            .with_context(|| format!("create {}", parent.display()))?;
    }
    bijux_dna_infra::atomic_write_bytes(path, payload.as_bytes())?;
    Ok(())
}
