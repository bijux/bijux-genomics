use std::collections::BTreeMap;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ToolAdapterEntry {
    pub tool_id: &'static str,
    pub adapter_id: &'static str,
}

#[must_use]
#[allow(dead_code)]
pub fn tool_registry() -> BTreeMap<&'static str, ToolAdapterEntry> {
    let mut map = BTreeMap::new();
    for (tool_id, adapter_id) in [
        ("bwa", "bam.align"),
        ("bowtie2", "bam.align"),
        ("samtools", "bam.validate"),
        ("picard", "bam.markdup"),
        ("gatk", "bam.recalibration"),
        ("mosdepth", "bam.coverage"),
        ("pydamage", "bam.damage"),
        ("mapdamage2", "bam.damage"),
        ("preseq", "bam.complexity"),
        ("authenticct", "bam.authenticity"),
        ("yleaf", "bam.haplogroups"),
        ("king", "bam.kinship"),
        ("angsd", "bam.contamination"),
        ("rxy", "bam.sex"),
    ] {
        map.insert(
            tool_id,
            ToolAdapterEntry {
                tool_id,
                adapter_id,
            },
        );
    }
    map
}
