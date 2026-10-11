//! Write the governed QC input fixture.
use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use std::path::Path;

pub(crate) fn write_governed_qc_input_vcf(output_path: &Path) -> Result<()> {
    let parent =
        output_path.parent().ok_or_else(|| anyhow!("QC input VCF path has no parent directory"))?;
    bijux_dna_infra::ensure_dir(parent).with_context(|| format!("create {}", parent.display()))?;
    let payload = "##fileformat=VCFv4.2\n\
##contig=<ID=chr1,length=1000>\n\
##INFO=<ID=DP,Number=1,Type=Integer,Description=\"Read depth\">\n\
##INFO=<ID=INFO,Number=1,Type=Float,Description=\"Imputation info\">\n\
##INFO=<ID=R2,Number=1,Type=Float,Description=\"Imputation R2\">\n\
##INFO=<ID=AF,Number=1,Type=Float,Description=\"Allele frequency\">\n\
##FORMAT=<ID=GT,Number=1,Type=String,Description=\"Genotype\">\n\
#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\tqc_ref\tqc_sparse\tqc_balanced\n\
chr1\t10\t.\tA\tG\t60\tPASS\tDP=20;INFO=0.95;R2=0.90;AF=0.10\tGT\t0/1\t./.\t0/0\n\
chr1\t20\t.\tC\tT\t62\tPASS\tDP=18;INFO=0.90;R2=0.88;AF=0.25\tGT\t1/1\t./.\t0/1\n\
chr1\t30\t.\tG\tA\t59\tLOWQUAL\tDP=16;INFO=0.82;R2=0.80;AF=0.05\tGT\t0/0\t./.\t./.\n\
chr1\t40\t.\tT\tG\t65\tPASS\tDP=22;INFO=0.93;R2=0.91;AF=0.40\tGT\t0/1\t1/1\t0/1\n";
    bijux_dna_infra::atomic_write_bytes(output_path, payload.as_bytes())?;
    Ok(())
}
