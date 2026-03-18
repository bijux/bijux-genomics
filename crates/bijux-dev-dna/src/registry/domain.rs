use anyhow::{anyhow, Result};

use crate::infrastructure::script_catalog::load_supported_scripts;
use crate::infrastructure::workspace::Workspace;
use crate::model::domain::DomainCommandDefinition;

pub fn domain_registry(workspace: &Workspace) -> Result<Vec<DomainCommandDefinition>> {
    let mut commands = load_supported_scripts(workspace)?
        .into_iter()
        .filter(|entry| entry.path.starts_with("scripts/domain/"))
        .filter(|entry| entry.path.ends_with(".sh"))
        .filter(|entry| entry.path != "scripts/domain/make.sh")
        .map(|entry| {
            let id = entry
                .path
                .rsplit('/')
                .next()
                .and_then(|name| name.strip_suffix(".sh"))
                .ok_or_else(|| anyhow!("unsupported domain script path `{}`", entry.path))?;
            Ok(DomainCommandDefinition {
                id: id.to_string(),
                summary: format!("Run `{}`.", entry.path),
                rel_path: entry.path,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    commands.sort_by(|left, right| left.id.cmp(&right.id));
    for pair in commands.windows(2) {
        if pair[0].id == pair[1].id {
            return Err(anyhow!("duplicate domain command id `{}`", pair[0].id));
        }
    }
    Ok(commands)
}
