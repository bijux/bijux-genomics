#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CallStageKind {
    Alias,
    Gl,
    Diploid,
    Pseudohaploid,
}

#[derive(Debug, Clone, Serialize)]
pub struct CallStageOutputs {
    pub called_vcf: PathBuf,
    pub called_tbi: PathBuf,
    pub call_metrics_json: PathBuf,
    pub call_metrics_tsv: PathBuf,
    pub call_manifest_json: PathBuf,
}
