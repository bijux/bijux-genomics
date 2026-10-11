use super::{ContainerDeliveryCatalog, ExternalArtifactToolContract};
use anyhow::{anyhow, Result};
use std::collections::BTreeSet;
impl ContainerDeliveryCatalog {
    /// Select all supported tools, including supported external deliveries.
    ///
    /// # Errors
    /// Rejects a catalog without supported tools.
    pub fn supported_tool_ids(&self) -> Result<BTreeSet<String>> {
        let integrated_tool_ids = self
            .catalog
            .tools
            .iter()
            .filter(|tool| tool.is_product_supported())
            .map(|tool| tool.id.clone())
            .collect::<BTreeSet<_>>();
        if integrated_tool_ids.is_empty() {
            return Err(anyhow!("container catalog does not integrate any executable tools"));
        }
        Ok(integrated_tool_ids)
    }
    /// Project external runtime verification contracts with explicit stage/domain admission.
    ///
    /// # Errors
    /// Rejects redistributable/unsupported external tools, absent runtime metadata,
    /// missing domains or stages, and stages outside their declared domains.
    pub fn external_artifact_contracts(&self) -> Result<Vec<ExternalArtifactToolContract>> {
        let mut contracts = self.catalog.tools.iter()
        .filter(|tool| tool.external_artifact_supported).cloned()
        .map(|mut tool| {
            if !tool.is_product_supported()
                || tool.is_redistributable_image()
                || tool.redistribution_permitted
            {
                return Err(anyhow!(
                    "external artifact tool `{}` must remain supported with non-redistributable external delivery",
                    tool.id
                ));
            }
            tool.domains.sort();
            tool.domains.dedup();
            tool.stage_ids.sort();
            tool.stage_ids.dedup();
            if tool.domains.is_empty() || tool.stage_ids.is_empty() {
                return Err(anyhow!(
                    "external artifact tool `{}` must declare domains and stage ids",
                    tool.id
                ));
            }
            if [
                tool.expected_bin.as_str(),
                tool.version_command.as_str(),
                tool.help_command.as_str(),
                tool.expected_version_regex.as_str(),
                tool.version.as_str(),
            ]
            .iter()
            .any(|value| value.trim().is_empty())
            {
                return Err(anyhow!(
                    "external artifact tool `{}` must declare complete runtime verification metadata",
                    tool.id
                ));
            }
            for stage_id in &tool.stage_ids {
                let domain = stage_id.split_once('.').map(|(domain, _)| domain).ok_or_else(|| {
                    anyhow!(
                        "external artifact tool `{}` has malformed stage id `{stage_id}`",
                        tool.id
                    )
                })?;
                if !tool.domains.iter().any(|candidate| candidate == domain) {
                    return Err(anyhow!(
                        "external artifact tool `{}` stage `{stage_id}` is outside its declared domains",
                        tool.id
                    ));
                }
            }
            Ok(ExternalArtifactToolContract {
                tool_id: tool.id,
                domains: tool.domains,
                stage_ids: tool.stage_ids,
                expected_bin: tool.expected_bin,
                version_command: tool.version_command,
                help_command: tool.help_command,
                expected_version_regex: tool.expected_version_regex,
                declared_version: tool.version,
            })
        })
        .collect::<Result<Vec<_>>>()?;
        contracts.sort_by(|left, right| left.tool_id.cmp(&right.tool_id));
        if contracts.is_empty() {
            return Err(anyhow!(
                "container catalog does not declare any operator-provided external artifacts"
            ));
        }
        Ok(contracts)
    }
    /// Project exact stage/tool bindings for admitted external artifacts.
    ///
    /// # Errors
    /// Returns external runtime-contract admission failures.
    pub fn external_artifact_binding_keys(&self) -> Result<BTreeSet<(String, String)>> {
        Ok(self
            .external_artifact_contracts()?
            .into_iter()
            .flat_map(|contract| {
                contract
                    .stage_ids
                    .into_iter()
                    .map(move |stage_id| (stage_id, contract.tool_id.clone()))
            })
            .collect())
    }
}
