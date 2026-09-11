use std::path::Path;

pub(super) fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).display().to_string()
}

pub(super) fn admit_damage(input: &super::model::DamageProbe<'_>) -> anyhow::Result<()> {
    if !input.c_to_t_5p.is_finite()
        || !input.g_to_a_3p.is_finite()
        || !(0.0..=1.0).contains(&input.c_to_t_5p)
        || !(0.0..=1.0).contains(&input.g_to_a_3p)
    {
        return Err(anyhow::anyhow!(
            "terminal damage probabilities must be finite and within [0, 1]"
        ));
    }
    Ok(())
}
