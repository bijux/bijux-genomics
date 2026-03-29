use super::effect::EffectSize;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DecisionMetricTrace {
    pub metric_id: String,
    pub value: Option<f64>,
    pub weight: f64,
    pub contribution: f64,
    pub effect: Option<EffectSize>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DecisionTrace {
    pub per_metric: Vec<DecisionMetricTrace>,
    pub penalties: Vec<String>,
    pub missing: Vec<String>,
    pub tie_breaks: Vec<String>,
}

impl DecisionTrace {
    #[must_use]
    pub fn empty() -> Self {
        Self {
            per_metric: Vec::new(),
            penalties: Vec::new(),
            missing: Vec::new(),
            tie_breaks: Vec::new(),
        }
    }
}
