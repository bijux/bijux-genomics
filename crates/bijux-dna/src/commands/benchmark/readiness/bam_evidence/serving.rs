use super::super::tool_serving_map::{
    collect_bam_tool_serving_map_rows, render_bam_tool_serving_map, ToolServingMapRow,
    DEFAULT_BAM_TOOL_SERVING_MAP_PATH,
};
use super::BamEvidenceAdapter;
use anyhow::Result;
use bijux_dna_bench_bam_evidence::{
    observations::{ServingBinding, ServingInventory},
    ports::ServingEvidence,
};
use std::path::{Path, PathBuf};
fn observe(row: ToolServingMapRow) -> ServingBinding {
    ServingBinding {
        tool_id: row.tool_id,
        stage_id: row.stage_id,
        support_status: row.support_status,
        adapter_status: row.adapter_status,
        parser_status: row.parser_status,
        corpus_status: row.corpus_status,
    }
}
impl ServingEvidence for BamEvidenceAdapter {
    fn serving(&self, repo_root: &Path) -> Result<ServingInventory> {
        let (stage_count, tool_count, rows) = collect_bam_tool_serving_map_rows(repo_root)?;
        Ok(ServingInventory {
            stage_count,
            tool_count,
            rows: rows.into_iter().map(observe).collect(),
        })
    }
    fn publish_serving(&self, repo_root: &Path) -> Result<ServingInventory> {
        let report = render_bam_tool_serving_map(
            repo_root,
            PathBuf::from(DEFAULT_BAM_TOOL_SERVING_MAP_PATH),
        )?;
        Ok(ServingInventory {
            stage_count: report.stage_count,
            tool_count: report.tool_count,
            rows: report.rows.into_iter().map(observe).collect(),
        })
    }
}
