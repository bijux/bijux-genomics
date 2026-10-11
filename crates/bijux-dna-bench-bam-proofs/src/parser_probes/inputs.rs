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

pub(super) enum AlignmentBackend {
    Bwa,
    Bowtie2,
}

pub(super) fn alignment_backend(tool_id: &str) -> anyhow::Result<AlignmentBackend> {
    use bijux_dna_core::id_catalog::{TOOL_BOWTIE2, TOOL_BWA};
    match tool_id {
        TOOL_BWA => Ok(AlignmentBackend::Bwa),
        TOOL_BOWTIE2 => Ok(AlignmentBackend::Bowtie2),
        _ => Err(anyhow::anyhow!("unsupported alignment parser proof tool: {tool_id}")),
    }
}
