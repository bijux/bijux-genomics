use super::super::real_output_parser_probe::{
    render_bam_genotyping_parser_smoke_row, render_real_output_parser_smoke,
    RealOutputParserSmokeRow, DEFAULT_REAL_OUTPUT_PARSER_SMOKE_PATH,
};
use anyhow::Result;
use bijux_dna_bench_bam_proofs::downstream::{
    observations::ParserObservation, ports::ParserEvidence,
};
use std::path::Path;
pub(super) struct ParserAdapter;
impl ParserEvidence for ParserAdapter {
    fn genotyping(&self, root: &Path) -> Result<ParserObservation> {
        Ok(observe(render_bam_genotyping_parser_smoke_row(root)?))
    }
    fn all(&self, root: &Path) -> Result<Vec<ParserObservation>> {
        Ok(render_real_output_parser_smoke(root, DEFAULT_REAL_OUTPUT_PARSER_SMOKE_PATH.into())?
            .rows
            .into_iter()
            .map(observe)
            .collect())
    }
}
fn observe(row: RealOutputParserSmokeRow) -> ParserObservation {
    ParserObservation {
        representative_stage_id: row.representative_stage_id,
        representative_tool_id: row.representative_tool_id,
        proof_path: row.proof_path,
        parsed_schema_version: row.parsed_schema_version,
        normalized_snapshot: row.normalized_snapshot,
        passed: row.passed,
    }
}
