//! Owner: bijux-analyze
//! External analyze contract.

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnalyzeContractV1 {
    pub schema_version: String,
    pub supported_inputs: Vec<String>,
    pub required_versions: Vec<String>,
    pub outputs: Vec<String>,
    pub compatibility: Vec<String>,
}

#[must_use]
pub fn analyze_contract_v1() -> AnalyzeContractV1 {
    AnalyzeContractV1 {
        schema_version: "bijux.analyze_contract.v1".to_string(),
        supported_inputs: vec![
            "facts.jsonl".to_string(),
            "run_index.jsonl".to_string(),
            "run_summary.json".to_string(),
        ],
        required_versions: vec![
            "bijux.facts.v1".to_string(),
            "bijux.run_summary.v1".to_string(),
            "run_index.jsonl@schema_version=1".to_string(),
        ],
        outputs: vec![
            "analysis.json".to_string(),
            "compare.json".to_string(),
            "ranking.json".to_string(),
            "report.json".to_string(),
            "report.html(optional)".to_string(),
            "report.md(optional)".to_string(),
        ],
        compatibility: vec![
            "minor additions must be optional".to_string(),
            "breaking changes require new schema version".to_string(),
            "deterministic ordering is required".to_string(),
        ],
    }
}
