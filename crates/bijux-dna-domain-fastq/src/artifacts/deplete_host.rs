use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::params::{
    screen::{
        MappingReportFormat, ReadRetentionPolicy, ReferenceDecoyPolicy, ReferenceMaskingPolicy,
        ReferenceScope,
    },
    PairedMode,
};

pub const DEPLETE_HOST_REPORT_SCHEMA_VERSION: &str = "bijux.fastq.deplete_host.report.v2";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct DepleteHostReportV1 {
    pub schema_version: String,
    pub stage: String,
    pub stage_id: String,
    pub tool_id: String,
    pub paired_mode: PairedMode,
    pub threads: u32,
    pub reference_scope: ReferenceScope,
    pub reference_catalog_id: String,
    pub reference_index_artifact_id: String,
    pub reference_index_backend: String,
    pub reference_build_id: Option<String>,
    pub reference_digest: Option<String>,
    pub masking_policy: ReferenceMaskingPolicy,
    pub decoy_policy: ReferenceDecoyPolicy,
    pub decoy_catalog_id: Option<String>,
    pub identity_threshold: f64,
    pub retained_read_policy: ReadRetentionPolicy,
    pub emit_removed_reads: bool,
    pub report_format: MappingReportFormat,
    pub retain_unmapped_pairs: bool,
    pub input_r1: String,
    pub input_r2: Option<String>,
    pub output_r1: String,
    pub output_r2: Option<String>,
    pub removed_host_r1: String,
    pub removed_host_r2: Option<String>,
    pub report_json: String,
    pub reads_in: u64,
    pub reads_out: u64,
    pub reads_removed: u64,
    pub bases_in: u64,
    pub bases_out: u64,
    pub bases_removed: u64,
    pub pairs_in: Option<u64>,
    pub pairs_out: Option<u64>,
    pub host_fraction_removed: f64,
    pub runtime_s: Option<f64>,
    pub memory_mb: Option<f64>,
    pub exit_code: Option<i32>,
    pub raw_backend_report: Option<String>,
    pub raw_backend_report_format: Option<String>,
    pub backend_metrics: Option<serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::{DepleteHostReportV1, DEPLETE_HOST_REPORT_SCHEMA_VERSION};
    use crate::params::{
        screen::{
            MappingReportFormat, ReadRetentionPolicy, ReferenceDecoyPolicy, ReferenceMaskingPolicy,
            ReferenceScope,
        },
        PairedMode,
    };

    #[test]
    fn deplete_host_report_contract_round_trips() {
        let report = DepleteHostReportV1 {
            schema_version: DEPLETE_HOST_REPORT_SCHEMA_VERSION.to_string(),
            stage: "fastq.deplete_host".to_string(),
            stage_id: "fastq.deplete_host".to_string(),
            tool_id: "bowtie2".to_string(),
            paired_mode: PairedMode::PairedEnd,
            threads: 8,
            reference_scope: ReferenceScope::Host,
            reference_catalog_id: "host_reference".to_string(),
            reference_index_artifact_id: "reference_index".to_string(),
            reference_index_backend: "bowtie2_build".to_string(),
            reference_build_id: Some("2026.03".to_string()),
            reference_digest: Some("sha256:host".to_string()),
            masking_policy: ReferenceMaskingPolicy::Unmasked,
            decoy_policy: ReferenceDecoyPolicy::None,
            decoy_catalog_id: None,
            identity_threshold: 0.95,
            retained_read_policy: ReadRetentionPolicy::KeepNonHostReads,
            emit_removed_reads: true,
            report_format: MappingReportFormat::Bowtie2MetricsFile,
            retain_unmapped_pairs: true,
            input_r1: "reads_R1.fastq.gz".to_string(),
            input_r2: Some("reads_R2.fastq.gz".to_string()),
            output_r1: "host_depleted_R1.fastq.gz".to_string(),
            output_r2: Some("host_depleted_R2.fastq.gz".to_string()),
            removed_host_r1: "removed_host_R1.fastq.gz".to_string(),
            removed_host_r2: Some("removed_host_R2.fastq.gz".to_string()),
            report_json: "host_depletion_report.json".to_string(),
            reads_in: 200,
            reads_out: 150,
            reads_removed: 50,
            bases_in: 20_000,
            bases_out: 15_000,
            bases_removed: 5_000,
            pairs_in: Some(100),
            pairs_out: Some(75),
            host_fraction_removed: 0.25,
            runtime_s: Some(10.5),
            memory_mb: Some(512.0),
            exit_code: Some(0),
            raw_backend_report: Some("bowtie2.host.metrics.txt".to_string()),
            raw_backend_report_format: Some("bowtie2_met_file".to_string()),
            backend_metrics: Some(serde_json::json!({
                "reads_removed": 50_u64,
            })),
        };

        let encoded =
            serde_json::to_string(&report).unwrap_or_else(|err| panic!("serialize failed: {err}"));
        let decoded: DepleteHostReportV1 = serde_json::from_str(&encoded)
            .unwrap_or_else(|err| panic!("deserialize failed: {err}"));
        assert_eq!(decoded.tool_id, "bowtie2");
        assert_eq!(decoded.reads_removed, 50);
        assert_eq!(decoded.raw_backend_report_format.as_deref(), Some("bowtie2_met_file"));
    }
}
