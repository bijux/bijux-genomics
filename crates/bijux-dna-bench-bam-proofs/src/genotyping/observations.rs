use serde::Deserialize;
#[derive(Debug, Clone, Deserialize)]
pub(super) struct LocalGenotypingPlan {
    pub(super) stage_id: String,
    pub(super) tool_id: String,
    pub(super) out_dir: String,
    pub(super) io: LocalGenotypingPlanIo,
    pub(super) params: LocalGenotypingPlanParams,
    pub(super) effective_params: LocalGenotypingEffectiveParams,
    pub(super) command: LocalGenotypingCommand,
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct LocalGenotypingPlanIo {
    pub(super) outputs: Vec<LocalGenotypingArtifact>,
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct LocalGenotypingArtifact {
    pub(super) name: String,
    pub(super) path: String,
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct LocalGenotypingPlanParams {
    pub(super) sample_id: String,
    pub(super) reference: String,
    pub(super) sites: String,
    pub(super) regions: String,
    pub(super) producer_contract: LocalGenotypingProducerContract,
    pub(super) caller: String,
    pub(super) tool: String,
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct LocalGenotypingProducerContract {
    pub(super) bcf: String,
    pub(super) vcf: String,
    pub(super) tbi: String,
    pub(super) gl: String,
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct LocalGenotypingEffectiveParams {
    pub(super) caller: String,
    pub(super) min_posterior: f64,
    pub(super) min_call_rate: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct LocalGenotypingCommand {
    pub(super) template: Vec<String>,
}
