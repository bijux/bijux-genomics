use std::collections::BTreeMap;

use bijux_dna_core::ids::{ToolId, id_catalog};

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ToolAdapterEntry {
    pub tool_id: ToolId,
    pub adapter_id: &'static str,
}

#[must_use]
#[allow(dead_code)]
pub fn tool_registry() -> BTreeMap<ToolId, ToolAdapterEntry> {
    let mut map = BTreeMap::new();
    for (tool_id, adapter_id) in [
        (ToolId::from_static("bwa"), id_catalog::BAM_ALIGN),
        (ToolId::from_static("bowtie2"), id_catalog::BAM_ALIGN),
        (ToolId::from_static("samtools"), id_catalog::BAM_VALIDATE),
        (ToolId::from_static("picard"), id_catalog::BAM_MARKDUP),
        (ToolId::from_static("gatk"), id_catalog::BAM_RECALIBRATION),
        (ToolId::from_static("mosdepth"), id_catalog::BAM_COVERAGE),
        (ToolId::from_static("pydamage"), id_catalog::BAM_DAMAGE),
        (ToolId::from_static("mapdamage2"), id_catalog::BAM_DAMAGE),
        (ToolId::from_static("preseq"), id_catalog::BAM_COMPLEXITY),
        (ToolId::from_static("authenticct"), id_catalog::BAM_AUTHENTICITY),
        (ToolId::from_static("yleaf"), id_catalog::BAM_HAPLOGROUPS),
        (ToolId::from_static("king"), id_catalog::BAM_KINSHIP),
        (ToolId::from_static("angsd"), id_catalog::BAM_CONTAMINATION),
        (ToolId::from_static("rxy"), id_catalog::BAM_SEX),
    ] {
        let key = tool_id.clone();
        map.insert(
            key,
            ToolAdapterEntry {
                tool_id,
                adapter_id,
            },
        );
    }
    map
}
