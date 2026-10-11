//! postprocess fixtures inputs.
use anyhow::Result;
use std::path::Path;

pub(crate) fn write_governed_postprocess_input_vcf(path: &Path) -> Result<()> {
    let payload = concat!(
        "##fileformat=VCFv4.2\n",
        "##contig=<ID=chr1,length=1000000>\n",
        "#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\ts2\ts1\n",
        "chr1\t100\t.\ta\tg,t\t60\t.\tMQ=50\tGT\t0/1\t1/2\n",
        "chr1\t101\t.\tAA\tA\t60\t.\tMQ=45\tGT\t0/1\t0/0\n",
        "chr1\t102\trskeep\tG\tC\t60\t.\tMQ=40\tGT\t0/0\t0/1\n"
    );
    bijux_dna_infra::atomic_write_bytes(path, payload.as_bytes())?;
    Ok(())
}
