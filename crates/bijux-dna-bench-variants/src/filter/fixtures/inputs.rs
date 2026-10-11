//! filter fixtures inputs.
use crate::filter::model::GOVERNED_VCF_FILTER_SAMPLE_NAME;
use anyhow::Context;
use anyhow::Result;
use std::path::Path;

pub(crate) fn write_governed_filter_input_vcf(path: &Path) -> Result<()> {
    let payload = format!(
        "##fileformat=VCFv4.2\n\
##reference=bijux-filter-smoke\n\
##contig=<ID=chr1,length=24>\n\
##INFO=<ID=DP,Number=1,Type=Integer,Description=\"Read depth\">\n\
##INFO=<ID=MQ,Number=1,Type=Float,Description=\"Mapping quality\">\n\
##INFO=<ID=FS,Number=1,Type=Float,Description=\"Strand bias\">\n\
##INFO=<ID=AF,Number=A,Type=Float,Description=\"Alternate allele frequency\">\n\
##FORMAT=<ID=GT,Number=1,Type=String,Description=\"Genotype\">\n\
#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\t{GOVERNED_VCF_FILTER_SAMPLE_NAME}\n\
chr1\t3\trs_pass\tA\tG\t60\tPASS\tDP=12;MQ=55;FS=10;AF=0.40\tGT\t0/1\n\
chr1\t5\trs_lowqual\tC\tT\t20\tPASS\tDP=14;MQ=50;FS=9;AF=0.45\tGT\t0/1\n\
chr1\t7\trs_lowdp\tG\tA\t62\tPASS\tDP=4;MQ=52;FS=8;AF=0.35\tGT\t0/1\n\
chr1\t9\trs_lowmq\tT\tC\t64\tPASS\tDP=13;MQ=20;FS=7;AF=0.30\tGT\t0/1\n\
chr1\t11\trs_missing\tA\tC\t68\tPASS\tDP=18;MQ=60;FS=6;AF=0.25\tGT\t./.\n",
    );
    if let Some(parent) = path.parent() {
        bijux_dna_infra::ensure_dir(parent)
            .with_context(|| format!("create {}", parent.display()))?;
    }
    bijux_dna_infra::atomic_write_bytes(path, payload.as_bytes())?;
    Ok(())
}
