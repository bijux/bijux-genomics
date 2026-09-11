use anyhow::Context;
use anyhow::Result;
use std::path::Path;
use std::path::PathBuf;

pub(in crate::readiness) fn prepare_local_hpc_profile_support(
    root: &Path,
) -> Result<HpcProfileSupportPaths> {
    for name in [
        "corpora",
        "databases",
        "images",
        "scratch",
        "logs",
        "results",
        "code",
        "imports",
        "baselines",
    ] {
        bijux_dna_infra::ensure_dir(root.join(name))
            .with_context(|| format!("create {}", root.join(name).display()))?;
    }
    let env_file_path = root.join("campaign.env");
    bijux_dna_infra::atomic_write_bytes(&env_file_path, &[])?;

    let policy_path = root.join("layout.policy.toml");
    let layout = [
        ("corpora_root", "corpora"),
        ("databases_root", "databases"),
        ("images_root", "images"),
        ("scratch_root", "scratch"),
        ("logs_root", "logs"),
        ("encrypted_results_root", "results"),
        ("encrypted_code_root", "code"),
        ("appraiser_imports_root", "imports"),
        ("baselines_root", "baselines"),
    ]
    .into_iter()
    .map(|(key, child)| (key, root.join(child).to_string_lossy().into_owned()))
    .collect::<std::collections::BTreeMap<_, _>>();
    let rendered = toml::to_string(&LocalProfilePolicy { layout })?;
    bijux_dna_infra::atomic_write_bytes(&policy_path, rendered.as_bytes())?;

    Ok(HpcProfileSupportPaths { env_file_path, policy_path })
}

#[derive(Debug, Clone)]
pub(in crate::readiness) struct HpcProfileSupportPaths {
    pub env_file_path: PathBuf,
    pub policy_path: PathBuf,
}

#[derive(serde::Serialize)]
struct LocalProfilePolicy<'a> {
    layout: std::collections::BTreeMap<&'a str, String>,
}
