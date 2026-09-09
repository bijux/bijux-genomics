use super::*;

fn seed(output: &Path, existing: bool) -> Result<()> {
    if existing {
        fs::write(output, b"previous ciphertext")?;
        fs::write(sidecar_path_for(output), b"previous sidecar")?;
    }
    Ok(())
}
fn prepare(path: &Path) -> Result<((), Vec<u8>)> {
    fs::write(path, b"next ciphertext")?;
    Ok(((), b"next sidecar".to_vec()))
}
fn assert_pair(output: &Path, existing: bool, committed: bool) -> Result<()> {
    if committed {
        assert_eq!(fs::read(output)?, b"next ciphertext");
        assert_eq!(fs::read(sidecar_path_for(output))?, b"next sidecar");
    } else if existing {
        assert_eq!(fs::read(output)?, b"previous ciphertext");
        assert_eq!(fs::read(sidecar_path_for(output))?, b"previous sidecar");
    } else {
        assert!(!output.exists());
        assert!(!sidecar_path_for(output).exists());
    }
    assert!(!locations(output)?.1.exists());
    Ok(())
}

#[test]
fn publication_errors_restore_both_members_and_reject_competing_readers() -> Result<()> {
    for existing in [false, true] {
        for point in [
            PublicationPoint::Prepared,
            PublicationPoint::Ciphertext,
            PublicationPoint::Sidecar,
            PublicationPoint::Committed,
        ] {
            let root = tempfile::tempdir()?;
            let output = root.path().join("bundle");
            seed(&output, existing)?;
            let pair = BundlePair::acquire(&output)?;
            let error = pair.publish(prepare, |observed| {
                assert!(BundlePair::acquire_reader(&output).is_err());
                assert!(BundlePair::acquire(&output).is_err());
                if observed == point {
                    anyhow::bail!("injected publication failure");
                }
                Ok(())
            });
            assert!(error.is_err());
            assert_pair(&output, existing, point == PublicationPoint::Committed)?;
            drop(pair);
            assert!(BundlePair::acquire_reader(&output)?.is_some());
        }
    }
    Ok(())
}

#[test]
fn publication_recovers_after_process_exit() -> Result<()> {
    if let Some(point) = std::env::var_os("BIJUX_BUNDLE_PUBLICATION_EXIT_POINT") {
        let output = PathBuf::from(
            std::env::var_os("BIJUX_BUNDLE_PUBLICATION_EXIT_OUTPUT").context("child output")?,
        );
        let pair = BundlePair::acquire(&output)?;
        pair.publish(prepare, |observed| {
            if point == format!("{observed:?}").as_str() {
                std::process::exit(23);
            }
            Ok(())
        })?;
        anyhow::bail!("requested exit point was not reached");
    }
    let test = format!(
        "{}::publication_recovers_after_process_exit",
        module_path!().split_once("::").context("test module")?.1
    );
    for existing in [false, true] {
        for point in [
            PublicationPoint::Prepared,
            PublicationPoint::Ciphertext,
            PublicationPoint::Sidecar,
            PublicationPoint::Committed,
        ] {
            let root = tempfile::tempdir()?;
            let output = root.path().join("bundle");
            seed(&output, existing)?;
            let child = std::process::Command::new(std::env::current_exe()?)
                .args(["--exact", &test, "--nocapture"])
                .env("BIJUX_BUNDLE_PUBLICATION_EXIT_POINT", format!("{point:?}"))
                .env("BIJUX_BUNDLE_PUBLICATION_EXIT_OUTPUT", &output)
                .output()?;
            assert_eq!(child.status.code(), Some(23), "{}", String::from_utf8_lossy(&child.stderr));
            let _reader = BundlePair::acquire_reader(&output)?.context("published pair lease")?;
            assert_pair(&output, existing, point == PublicationPoint::Committed)?;
        }
    }
    Ok(())
}

#[test]
fn recovery_preserves_corrupt_backup_evidence_until_it_can_restore_safely() -> Result<()> {
    let root = tempfile::tempdir()?;
    let output = root.path().join("bundle");
    seed(&output, true)?;
    let pair = BundlePair::acquire(&output)?;
    assert!(pair
        .publish(prepare, |point| {
            if point == PublicationPoint::Ciphertext {
                fs::write(pair.journal.join("old-sidecar"), b"corrupt")?;
                anyhow::bail!("sidecar publication failed");
            }
            Ok(())
        })
        .is_err());
    assert_eq!(fs::read(&output)?, b"next ciphertext");
    assert_eq!(fs::read(sidecar_path_for(&output))?, b"previous sidecar");
    drop(pair);
    assert!(BundlePair::acquire_reader(&output).is_err());
    fs::write(locations(&output)?.1.join("old-sidecar"), b"previous sidecar")?;
    let _reader = BundlePair::acquire_reader(&output)?.context("recover restored backup")?;
    assert_pair(&output, true, false)?;
    Ok(())
}

#[test]
fn legacy_readers_do_not_create_leases_and_foreign_journals_are_preserved() -> Result<()> {
    let root = tempfile::tempdir()?;
    let output = root.path().join("bundle");
    seed(&output, true)?;
    assert!(BundlePair::acquire_reader(&output)?.is_none());
    assert!(!locations(&output)?.0.exists());
    let journal = locations(&output)?.1;
    fs::create_dir(&journal)?;
    fs::write(journal.join("foreign-evidence"), b"preserve")?;
    assert!(BundlePair::acquire(&output).is_err());
    assert_eq!(fs::read(journal.join("foreign-evidence"))?, b"preserve");
    assert_eq!(fs::read(&output)?, b"previous ciphertext");
    Ok(())
}

#[cfg(unix)]
#[test]
fn redirected_publication_members_are_not_output_owners() -> Result<()> {
    let root = tempfile::tempdir()?;
    let external = root.path().join("external");
    fs::write(&external, b"preserve")?;
    for sidecar in [false, true] {
        let output = root.path().join(if sidecar { "sidecar-bundle" } else { "cipher-bundle" });
        let redirected = if sidecar { sidecar_path_for(&output) } else { output.clone() };
        std::os::unix::fs::symlink(&external, &redirected)?;
        assert!(BundlePair::acquire(&output).is_err());
        assert_eq!(fs::read(&external)?, b"preserve");
    }
    Ok(())
}
