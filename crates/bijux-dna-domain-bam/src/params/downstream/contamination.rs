use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::params::ContaminationScope;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ContaminationEffectiveParams {
    pub reference_panels: Vec<String>,
    pub scope: ContaminationScope,
    #[serde(default)]
    pub prior: Option<f64>,
    pub sex_specific: bool,
    #[serde(default)]
    pub assumptions: Option<String>,
    #[serde(default)]
    pub required_reference_digest: Option<String>,
    #[serde(default)]
    pub chromosome_system: Option<String>,
    #[serde(default)]
    pub minimum_mean_coverage: Option<f64>,
    #[serde(default = "default_emit_confidence_caveats")]
    pub emit_confidence_caveats: bool,
}

fn default_emit_confidence_caveats() -> bool {
    true
}

/// Admit the contamination backend's supported biological scope.
///
/// # Errors
/// Rejects unknown tools and incompatible nuclear or mitochondrial scopes.
pub fn validate_contamination_tool_scope(
    tool_id: &str,
    scope: ContaminationScope,
) -> anyhow::Result<()> {
    use bijux_dna_core::id_catalog::{
        BAM_CONTAMINATION, TOOL_CONTAMMIX, TOOL_SCHMUTZI, TOOL_VERIFYBAMID2,
    };
    match tool_id {
        TOOL_SCHMUTZI if !matches!(scope, ContaminationScope::Mito | ContaminationScope::Both) => {
            Err(anyhow::anyhow!("{} tool schmutzi requires scope mito/both", BAM_CONTAMINATION))
        }
        TOOL_VERIFYBAMID2 | TOOL_CONTAMMIX
            if !matches!(scope, ContaminationScope::Nuclear | ContaminationScope::Both) =>
        {
            Err(anyhow::anyhow!("{} tool {tool_id} requires scope nuclear/both", BAM_CONTAMINATION))
        }
        TOOL_SCHMUTZI | TOOL_VERIFYBAMID2 | TOOL_CONTAMMIX => Ok(()),
        _ => Err(anyhow::anyhow!("unsupported contamination tool: {tool_id}")),
    }
}

#[cfg(test)]
mod contracts {
    use super::{validate_contamination_tool_scope, ContaminationScope};
    #[test]
    fn contamination_backends_require_their_declared_scope() {
        for (tool, allowed, refused) in [
            ("schmutzi", ContaminationScope::Mito, ContaminationScope::Nuclear),
            ("verifybamid2", ContaminationScope::Nuclear, ContaminationScope::Mito),
            ("contammix", ContaminationScope::Nuclear, ContaminationScope::Mito),
        ] {
            assert!(validate_contamination_tool_scope(tool, allowed).is_ok());
            assert!(validate_contamination_tool_scope(tool, ContaminationScope::Both).is_ok());
            assert!(validate_contamination_tool_scope(tool, refused).is_err());
        }
        for tool in ["", "unknown", "schmutzi-extra"] {
            assert!(validate_contamination_tool_scope(tool, ContaminationScope::Both).is_err());
        }
    }
}
