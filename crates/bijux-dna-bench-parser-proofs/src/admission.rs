use crate::{catalog::FamilyProbeSpec, reports::require_available};
use anyhow::{anyhow, Result};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) fn select_families(
    active: Vec<(String, String)>,
    catalog: &BTreeMap<&str, FamilyProbeSpec>,
) -> Result<Vec<(String, String, FamilyProbeSpec)>> {
    if active.is_empty() {
        return Err(anyhow!("parser proof requires at least one active family"));
    }
    let mut seen = BTreeSet::new();
    active
        .into_iter()
        .map(|(id, summary)| {
            let spec = catalog.get(id.as_str()).copied().ok_or_else(|| {
                anyhow!("real-output parser smoke has no governed probe spec for family `{id}`")
            })?;
            if !seen.insert(id.clone()) {
                return Err(anyhow!("duplicate active parser family `{id}`"));
            }
            require_available(spec.parse_kind)?;
            Ok((id, summary, spec))
        })
        .collect()
}
