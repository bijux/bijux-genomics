use crate::{
    command_coverage::BamCommandAdapterCoverageRow, output_contract::BamAdapterOutputContractRow,
};
use bijux_dna_bench_bindings::{expected::ExpectedBenchmarkResultRow, serving::ToolServingMapRow};
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(in crate::readiness) struct BindingKey {
    pub(in crate::readiness) stage_id: String,
    pub(in crate::readiness) tool_id: String,
}
pub(in crate::readiness) trait BindingRow {
    fn stage_id(&self) -> &str;
    fn tool_id(&self) -> &str;
}
impl BindingRow for ToolServingMapRow {
    fn stage_id(&self) -> &str {
        &self.stage_id
    }

    fn tool_id(&self) -> &str {
        &self.tool_id
    }
}
impl BindingRow for BamCommandAdapterCoverageRow {
    fn stage_id(&self) -> &str {
        &self.stage_id
    }

    fn tool_id(&self) -> &str {
        &self.tool_id
    }
}
impl BindingRow for BamAdapterOutputContractRow {
    fn stage_id(&self) -> &str {
        &self.stage_id
    }

    fn tool_id(&self) -> &str {
        &self.tool_id
    }
}
impl BindingRow for crate::parser_coverage::BamParserFixtureCoverageRow {
    fn stage_id(&self) -> &str {
        &self.stage_id
    }

    fn tool_id(&self) -> &str {
        &self.tool_id
    }
}
impl BindingRow for ExpectedBenchmarkResultRow {
    fn stage_id(&self) -> &str {
        &self.stage_id
    }

    fn tool_id(&self) -> &str {
        &self.tool_id
    }
}
pub(in crate::readiness) fn binding_key(stage_id: &str, tool_id: &str) -> BindingKey {
    BindingKey { stage_id: stage_id.to_string(), tool_id: tool_id.to_string() }
}

impl BindingRow for crate::report_map::BamReportMapRow {
    fn stage_id(&self) -> &str {
        &self.stage_id
    }
    fn tool_id(&self) -> &str {
        &self.tool_id
    }
}
