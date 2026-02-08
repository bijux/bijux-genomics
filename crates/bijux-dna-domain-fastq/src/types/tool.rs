//! Owner: bijux-dna-domain-fastq
//! Tool reference identity for domain reports.

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolReferenceV1 {
    pub id: String,
    pub stage: String,
    pub version: String,
    pub params: serde_json::Value,
}
