use anyhow::Result;

pub fn assert_stage_contracts<'a, I, F>(
    domain: &str,
    id_catalog: I,
    mut contract_for_stage: F,
) -> Result<()>
where
    I: IntoIterator<Item = &'a str>,
    F: FnMut(&str) -> bool,
{
    for stage_id in id_catalog {
        if !contract_for_stage(stage_id) {
            anyhow::bail!("missing stage contract for {domain}:{stage_id}");
        }
    }
    Ok(())
}
