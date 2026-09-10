//! imputation fixtures inputs.
use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use bijux_dna_stages_vcf::pipeline::ImputeBackend;
use std::path::Path;

pub(crate) fn write_governed_impute_input_vcf(
    output_path: &Path,
    backend: ImputeBackend,
) -> Result<()> {
    let parent = output_path
        .parent()
        .ok_or_else(|| anyhow!("VCF impute smoke input path has no parent directory"))?;
    bijux_dna_infra::ensure_dir(parent).with_context(|| format!("create {}", parent.display()))?;
    let format_headers = if matches!(backend, ImputeBackend::Glimpse) {
        "##FORMAT=<ID=GT,Number=1,Type=String,Description=\"Genotype\">\n##FORMAT=<ID=GL,Number=G,Type=Float,Description=\"Genotype likelihoods\">\n"
    } else {
        "##FORMAT=<ID=GT,Number=1,Type=String,Description=\"Genotype\">\n"
    };
    let (format, masked_first, donor_first, masked_second, donor_second) = match backend {
        ImputeBackend::Glimpse => (
            "GT:GL",
            "./.:-1.0,-0.1,-2.0",
            "0/1:-2.0,-0.1,-1.0",
            "0/0:-0.1,-1.0,-2.0",
            "0/0:-0.1,-1.0,-2.0",
        ),
        ImputeBackend::Minimac4 => ("GT", ".|.", "0|1", "0|0", "0|0"),
        ImputeBackend::Beagle | ImputeBackend::Impute5 => ("GT", "./.", "0/1", "0/0", "0/0"),
    };
    let payload = format!(
        "##fileformat=VCFv4.2\n\
##reference=GRCh38\n\
##contig=<ID=1,length=248956422>\n\
{format_headers}\
#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\tmasked_sample\tdonor_sample\n\
1\t100\t.\tA\tG\t60\tPASS\t.\t{format}\t{masked_first}\t{donor_first}\n\
1\t140\t.\tC\tT\t60\tPASS\t.\t{format}\t{masked_second}\t{donor_second}\n"
    );
    bijux_dna_infra::atomic_write_bytes(output_path, payload.as_bytes())?;
    Ok(())
}

pub(crate) fn write_governed_impute_truth_vcf(output_path: &Path) -> Result<()> {
    let parent = output_path
        .parent()
        .ok_or_else(|| anyhow!("VCF impute smoke truth path has no parent directory"))?;
    bijux_dna_infra::ensure_dir(parent).with_context(|| format!("create {}", parent.display()))?;
    let payload = "##fileformat=VCFv4.2\n\
##reference=GRCh38\n\
##contig=<ID=1,length=248956422>\n\
##FORMAT=<ID=GT,Number=1,Type=String,Description=\"Genotype\">\n\
#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\tmasked_sample\tdonor_sample\n\
1\t100\t.\tA\tG\t60\tPASS\t.\tGT\t0/1\t0/1\n\
1\t140\t.\tC\tT\t60\tPASS\t.\tGT\t0/0\t0/0\n";
    bijux_dna_infra::atomic_write_bytes(output_path, payload.as_bytes())?;
    Ok(())
}
