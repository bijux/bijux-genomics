//! damage filter fixtures inputs.
use crate::damage_filter::model::GOVERNED_VCF_DAMAGE_FILTER_SAMPLE_NAME;
use anyhow::Context;
use anyhow::Result;
use std::path::Path;

pub(crate) fn write_governed_damage_filter_input_vcf(path: &Path) -> Result<()> {
    let payload = format!(
        "##fileformat=VCFv4.2\n\
##reference=bijux-damage-filter-smoke\n\
##contig=<ID=chr1,length=12>\n\
##contig=<ID=chr2,length=12>\n\
##INFO=<ID=CT_GA_DAMAGE_RATIO,Number=1,Type=Float,Description=\"C>T or G>A damage ratio\">\n\
##INFO=<ID=DEAM5P,Number=1,Type=Float,Description=\"5 prime terminal deamination signal\">\n\
##INFO=<ID=DEAM3P,Number=1,Type=Float,Description=\"3 prime terminal deamination signal\">\n\
##INFO=<ID=PMD_SCORE,Number=1,Type=Float,Description=\"postmortem damage score\">\n\
##FORMAT=<ID=GT,Number=1,Type=String,Description=\"Genotype\">\n\
##FORMAT=<ID=PL,Number=G,Type=Integer,Description=\"Phred-scaled genotype likelihoods\">\n\
#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\t{GOVERNED_VCF_DAMAGE_FILTER_SAMPLE_NAME}\n\
chr1\t3\trs_keep\tA\tG\t60\tPASS\tCT_GA_DAMAGE_RATIO=0.02;DEAM5P=0.01;DEAM3P=0.01;PMD_SCORE=5\tGT:PL\t0/1:0,18,36\n\
chr1\t5\trs_damage\tC\tT\t72\tPASS\tCT_GA_DAMAGE_RATIO=0.80;DEAM5P=0.90;DEAM3P=0.10;PMD_SCORE=5\tGT:PL\t0/1:0,12,24\n\
chr2\t7\trs_transition_keep\tG\tA\t68\tPASS\tCT_GA_DAMAGE_RATIO=0.25;DEAM5P=0.05;DEAM3P=0.20;PMD_SCORE=5\tGT:PL\t0/1:0,14,28\n\
chr2\t9\trs_lowqual\tC\tT\t25\tPASS\tCT_GA_DAMAGE_RATIO=0.10;DEAM5P=0.10;DEAM3P=0.05;PMD_SCORE=5\tGT:PL\t0/1:0,20,40\n",
    );
    if let Some(parent) = path.parent() {
        bijux_dna_infra::ensure_dir(parent)
            .with_context(|| format!("create {}", parent.display()))?;
    }
    bijux_dna_infra::atomic_write_bytes(path, payload.as_bytes())?;
    Ok(())
}
