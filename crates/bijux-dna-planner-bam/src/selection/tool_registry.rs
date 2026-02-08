use std::collections::BTreeMap;

use bijux_dna_core::ids::id_catalog;

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
        ("bwa", id_catalog::BAM_ALIGN),
        ("bowtie2", id_catalog::BAM_ALIGN),
        ("samtools", id_catalog::BAM_VALIDATE),
        ("picard", id_catalog::BAM_MARKDUP),
        ("gatk", id_catalog::BAM_RECALIBRATION),
        ("mosdepth", id_catalog::BAM_COVERAGE),
        ("pydamage", id_catalog::BAM_DAMAGE),
        ("mapdamage2", id_catalog::BAM_DAMAGE),
        ("preseq", id_catalog::BAM_COMPLEXITY),
        ("authenticct", id_catalog::BAM_AUTHENTICITY),
        ("yleaf", id_catalog::BAM_HAPLOGROUPS),
        ("king", id_catalog::BAM_KINSHIP),
        ("angsd", id_catalog::BAM_CONTAMINATION),
        ("rxy", id_catalog::BAM_SEX),
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
