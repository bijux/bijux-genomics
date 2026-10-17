use std::fmt::Write as _;
use std::io::Write as _;
use std::path::{Path, PathBuf};

use anyhow::{ensure, Context as _, Result};
use bijux_dna_domain_fastq::inspect_headers;

#[test]
fn header_style_is_independent_of_record_identity_and_preserves_pair_refusal() -> Result<()> {
    let repository = bijux_dna_testkit::workspace_root_from_manifest(env!("CARGO_MANIFEST_DIR"));
    let scratch = bijux_dna_infra::temp_dir_in(
        &repository.join("artifacts/testing/fastq-header-style"),
        "header-contract-",
    )?;
    let first = write_reads(scratch.path(), "left", &["read-a/1"], false)?;
    let other = write_reads(scratch.path(), "other", &["foreign/2"], false)?;
    let error = inspect_headers(&first, Some(&other), true)
        .err()
        .context("different mate identities must retain strict pairing refusal")?;
    ensure!(error.to_string().contains("pairing mismatch"), "{error:#}");
    ensure!(
        inspect_headers(&first, Some(&other), false)?
            .warnings
            .iter()
            .any(|warning| warning.contains("pairing mismatch")),
        "non-strict pairing warning must remain"
    );
    let mut observations = Vec::new();
    for (label, names) in [
        ("suffix", ["read-a/1", "read-b/1"]),
        ("casava", ["read-a 1:N:0:ACGT", "read-b 1:N:0:ACGT"]),
        ("plain", ["read-a", "read-b"]),
    ] {
        for gzip in [false, true] {
            let path = write_reads(scratch.path(), label, &names, gzip)?;
            observations.push((
                label,
                gzip,
                inspect_headers(&path, None, false),
                inspect_headers(&path, None, true),
            ));
        }
    }
    for (label, gzip, advisory, strict) in observations {
        ensure!(
            advisory?.warnings.is_empty(),
            "homogeneous {label} gzip={gzip} has false style warning"
        );
        ensure!(strict.is_ok(), "homogeneous {label} gzip={gzip} refused: {strict:?}");
    }
    Ok(())
}

fn write_reads(root: &Path, label: &str, names: &[&str], gzip: bool) -> Result<PathBuf> {
    let path = root.join(format!("{label}.fastq{}", if gzip { ".gz" } else { "" }));
    let mut payload = String::new();
    for name in names {
        write!(&mut payload, "@{name}\nACGT\n+\nIIII\n")?;
    }
    let bytes = if gzip {
        let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        encoder.write_all(payload.as_bytes())?;
        encoder.finish()?
    } else {
        payload.into_bytes()
    };
    std::fs::write(&path, bytes)?;
    Ok(path)
}

#[test]
fn mixed_header_families_retain_advisory_and_strict_style_refusals() -> Result<()> {
    let repository = bijux_dna_testkit::workspace_root_from_manifest(env!("CARGO_MANIFEST_DIR"));
    let scratch = bijux_dna_infra::temp_dir_in(
        &repository.join("artifacts/testing/fastq-header-style"),
        "mixed-header-contract-",
    )?;
    for names in
        [["read-a/1", "read-b"], ["read-a 1:N:0:ACGT", "read-b/1"], ["read-a", "read-b 2:N:0:ACGT"]]
    {
        for gzip in [false, true] {
            let path = write_reads(scratch.path(), "mixed", &names, gzip)?;
            ensure!(
                inspect_headers(&path, None, false)?.warnings == ["header style drift detected"],
                "mixed families must retain the style warning"
            );
            let error = inspect_headers(&path, None, true)
                .err()
                .context("strict mixed families must refuse")?;
            ensure!(error.to_string() == "header style drift detected", "{error:#}");
        }
    }
    Ok(())
}
