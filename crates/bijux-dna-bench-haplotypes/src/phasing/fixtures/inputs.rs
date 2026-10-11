//! phasing fixtures inputs.
use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use std::path::Path;

pub(crate) fn write_governed_phasing_input_vcf(output_path: &Path) -> Result<()> {
    let parent = output_path
        .parent()
        .ok_or_else(|| anyhow!("VCF phasing smoke input path has no parent directory"))?;
    bijux_dna_infra::ensure_dir(parent).with_context(|| format!("create {}", parent.display()))?;
    let payload = "##fileformat=VCFv4.2\n\
##reference=GRCh38\n\
##contig=<ID=1,length=248956422>\n\
##FORMAT=<ID=GT,Number=1,Type=String,Description=\"Genotype\">\n\
#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\tcohort_alpha\tcohort_beta\n\
1\t101\t.\tA\tG\t60\tPASS\t.\tGT\t0/1\t1/0\n\
1\t120\t.\tC\tT\t61\tPASS\t.\tGT\t0/1\t0/1\n\
1\t145\t.\tG\tA\t59\tPASS\t.\tGT\t1/0\t0/1\n\
1\t160\t.\tT\tC\t58\tPASS\t.\tGT\t0/1\t1/0\n";
    bijux_dna_infra::atomic_write_bytes(output_path, payload.as_bytes())?;
    Ok(())
}
