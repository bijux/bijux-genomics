use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

fn repository_root() -> Result<PathBuf> {
    Ok(Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize()?)
}

fn current_catalog(root: &Path) -> PathBuf {
    root.join("benchmarks/contracts/stage-interfaces.toml")
}

fn catalog_value(path: &Path) -> Result<toml::Value> {
    let raw = std::fs::read_to_string(path)?;
    Ok(toml::from_str(&raw)?)
}

fn interfaces_mut(value: &mut toml::Value) -> Result<&mut Vec<toml::Value>> {
    value
        .get_mut("interfaces")
        .and_then(toml::Value::as_array_mut)
        .context("catalog omits interfaces")
}

fn remove_presentations(value: &mut toml::Value) -> Result<()> {
    for interface in interfaces_mut(value)? {
        interface.as_table_mut().context("interface is not a table")?.remove("report_presentation");
    }
    Ok(())
}

fn write_catalog(root: &Path, name: &str, value: &toml::Value) -> Result<PathBuf> {
    let path = root.join(name);
    std::fs::write(&path, toml::to_string(value)?)?;
    Ok(path)
}

#[test]
fn publication_completes_only_missing_historical_presentation_metadata() -> Result<()> {
    let root = repository_root()?;
    let current_path = current_catalog(&root);
    let current = bijux_dna_bench::load_stage_interface_catalog_from_repo(&current_path, &root)?;
    let temporary = bijux_dna_infra::temp_dir("historical-stage-interface-publication")?;
    let mut historical = catalog_value(&current_path)?;
    remove_presentations(&mut historical)?;
    let historical_path = write_catalog(temporary.path(), "historical.toml", &historical)?;

    assert!(
        bijux_dna_bench::load_stage_interface_catalog_from_repo(&historical_path, &root).is_err()
    );
    let loaded = bijux_dna_bench::load_stage_interface_catalog_for_publication(
        &historical_path,
        &root,
        &root,
    )?;

    assert_eq!(loaded.catalog, current);
    assert_eq!(
        loaded.presentation.catalog_sha256,
        bijux_dna_infra::hash_file_sha256(&current_path)?
    );
    assert_eq!(
        loaded.presentation.repository_commit,
        bijux_dna_containers::api::repository_head_commit(&root)?
    );
    assert_eq!(loaded.presentation.overlaid_stage_ids.len(), current.interfaces.len());
    assert!(loaded.presentation.overlaid_stage_ids.windows(2).all(|pair| pair[0] < pair[1]));
    Ok(())
}

#[test]
fn publication_rejects_non_presentation_schema_damage() -> Result<()> {
    let root = repository_root()?;
    let current_path = current_catalog(&root);
    let temporary = bijux_dna_infra::temp_dir("damaged-historical-stage-interface")?;
    let mut historical = catalog_value(&current_path)?;
    remove_presentations(&mut historical)?;
    interfaces_mut(&mut historical)?[0]
        .as_table_mut()
        .context("interface is not a table")?
        .remove("output_contract");
    let historical_path = write_catalog(temporary.path(), "damaged.toml", &historical)?;

    let error = bijux_dna_bench::load_stage_interface_catalog_for_publication(
        &historical_path,
        &root,
        &root,
    )
    .expect_err("non-presentation schema damage must remain invalid");
    assert!(format!("{error:#}").contains("output_contract"));
    Ok(())
}

#[test]
fn publication_rejects_an_unknown_historical_stage() -> Result<()> {
    let root = repository_root()?;
    let current_path = current_catalog(&root);
    let temporary = bijux_dna_infra::temp_dir("unknown-historical-stage-interface")?;
    let mut historical = catalog_value(&current_path)?;
    remove_presentations(&mut historical)?;
    interfaces_mut(&mut historical)?[0]
        .as_table_mut()
        .context("interface is not a table")?
        .insert("stage_id".to_string(), toml::Value::String("bam.unknown".to_string()));
    let historical_path = write_catalog(temporary.path(), "unknown-stage.toml", &historical)?;

    let error = bijux_dna_bench::load_stage_interface_catalog_for_publication(
        &historical_path,
        &root,
        &root,
    )
    .expect_err("presentation authority must cover the historical stage");
    assert!(
        format!("{error:#}").contains("presentation authority omits historical benchmark stage")
    );
    Ok(())
}

#[test]
fn publication_does_not_mask_malformed_present_presentation_metadata() -> Result<()> {
    let root = repository_root()?;
    let current_path = current_catalog(&root);
    let temporary = bijux_dna_infra::temp_dir("malformed-stage-presentation")?;
    let mut malformed = catalog_value(&current_path)?;
    let first =
        interfaces_mut(&mut malformed)?[0].as_table_mut().context("interface is not a table")?;
    first
        .get_mut("report_presentation")
        .and_then(toml::Value::as_table_mut)
        .context("interface omits report presentation")?
        .insert("comparison".to_string(), toml::Value::String("unknown".to_string()));
    let malformed_path = write_catalog(temporary.path(), "malformed.toml", &malformed)?;

    let error = bijux_dna_bench::load_stage_interface_catalog_for_publication(
        &malformed_path,
        &root,
        &root,
    )
    .expect_err("present malformed metadata must not be replaced");
    assert!(format!("{error:#}").contains("invalid for reasons other than absent presentation"));
    Ok(())
}
