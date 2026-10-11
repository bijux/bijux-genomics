//! gl propagation fixtures inputs.
use crate::gl_propagation::model::GOVERNED_VCF_GL_PROPAGATION_SAMPLE_NAME;
use anyhow::Context;
use anyhow::Result;
use std::path::Path;

pub(crate) fn write_governed_gl_propagation_input_vcf(path: &Path) -> Result<()> {
    let payload = format!(
        "##fileformat=VCFv4.2\n\
##reference=bijux-gl-propagation-smoke\n\
##contig=<ID=chr1,length=12>\n\
##contig=<ID=chr2,length=12>\n\
##FORMAT=<ID=GT,Number=1,Type=String,Description=\"Genotype\">\n\
##FORMAT=<ID=GL,Number=G,Type=Float,Description=\"Genotype Likelihood\">\n\
##FORMAT=<ID=PL,Number=G,Type=Integer,Description=\"Phred-scaled Genotype Likelihood\">\n\
##FORMAT=<ID=GP,Number=G,Type=Float,Description=\"Genotype Posterior\">\n\
#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\t{GOVERNED_VCF_GL_PROPAGATION_SAMPLE_NAME}\n\
chr1\t3\trs1\tA\tG\t60\tPASS\t.\tGT:GL:PL:GP\t0/1:0.0,-1.0,-2.0:10,0,20:0.05,0.90,0.05\n\
chr1\t7\trs2\tC\tT\t55\tPASS\t.\tGT:GL:PL:GP\t1/1:-2.2,-1.0,0.0:22,10,0:0.01,0.09,0.90\n\
chr2\t9\trs3\tG\tA\t70\tPASS\t.\tGT:GL:PL:GP\t0/0:0.0,-1.6,-3.2:0,16,32:0.92,0.07,0.01\n",
    );
    if let Some(parent) = path.parent() {
        bijux_dna_infra::ensure_dir(parent)
            .with_context(|| format!("create {}", parent.display()))?;
    }
    bijux_dna_infra::atomic_write_bytes(path, payload.as_bytes())?;
    Ok(())
}
