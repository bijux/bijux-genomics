use std::fs;

use anyhow::{Context, Result};
use bijux_dna_bench::{inspect_benchmark_source_asset, BenchmarkSourceKind};

#[test]
fn file_inspection_preserves_content_identity_and_missing_input_semantics() -> Result<()> {
    let root = tempfile::tempdir()?;
    fs::write(root.path().join("reads with spaces.fastq"), b"abc")?;
    let source = inspect_benchmark_source_asset(root.path(), "reads with spaces.fastq")?
        .context("file source")?;
    assert_eq!(source.kind, BenchmarkSourceKind::File);
    assert_eq!(source.source_path, "reads with spaces.fastq");
    assert_eq!(
        source.checksum_sha256,
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    assert_eq!((source.size_bytes, source.member_count), (3, 1));
    assert!(inspect_benchmark_source_asset(root.path(), "absent/input")?.is_none());
    Ok(())
}

#[test]
fn compound_identity_is_order_independent_and_sensitive_to_names_and_content() -> Result<()> {
    let root = tempfile::tempdir()?;
    for (directory, reverse) in [("first", false), ("second", true)] {
        let path = root.path().join(directory);
        fs::create_dir_all(path.join("nested"))?;
        let files = if reverse {
            [("nested/b.txt", "xyz"), ("a.txt", "abc")]
        } else {
            [("a.txt", "abc"), ("nested/b.txt", "xyz")]
        };
        for (name, content) in files {
            fs::write(path.join(name), content)?;
        }
    }
    let first = inspect_benchmark_source_asset(root.path(), "first")?.context("first")?;
    let second = inspect_benchmark_source_asset(root.path(), "second")?.context("second")?;
    assert_eq!(first.kind, BenchmarkSourceKind::Directory);
    assert_eq!((first.size_bytes, first.member_count), (6, 2));
    assert_eq!(first.checksum_sha256, second.checksum_sha256);
    fs::write(root.path().join("second/a.txt"), b"abd")?;
    let changed = inspect_benchmark_source_asset(root.path(), "second")?.context("changed")?;
    assert_ne!(first.checksum_sha256, changed.checksum_sha256);
    fs::rename(root.path().join("second/a.txt"), root.path().join("second/c.txt"))?;
    let renamed = inspect_benchmark_source_asset(root.path(), "second")?.context("renamed")?;
    assert_ne!(changed.checksum_sha256, renamed.checksum_sha256);
    Ok(())
}

#[test]
fn index_bundle_includes_only_files_with_the_exact_prefix_separator() -> Result<()> {
    let root = tempfile::tempdir()?;
    fs::write(root.path().join("reference.1.bt2"), b"one")?;
    fs::write(root.path().join("reference.2.bt2"), b"two")?;
    fs::write(root.path().join("reference-other.1.bt2"), b"unrelated")?;
    fs::create_dir(root.path().join("reference.directory"))?;
    let source = inspect_benchmark_source_asset(root.path(), "reference")?.context("bundle")?;
    assert_eq!(source.kind, BenchmarkSourceKind::PrefixBundle);
    assert_eq!((source.size_bytes, source.member_count), (6, 2));
    fs::write(root.path().join("reference-other.1.bt2"), b"changed unrelated")?;
    let repeated = inspect_benchmark_source_asset(root.path(), "reference")?.context("bundle")?;
    assert_eq!(source, repeated);
    Ok(())
}

#[cfg(unix)]
#[test]
fn directory_links_preserve_logical_names_and_reject_cycles() -> Result<()> {
    let root = tempfile::tempdir()?;
    fs::create_dir(root.path().join("source"))?;
    fs::write(root.path().join("source/reads"), b"abc")?;
    std::os::unix::fs::symlink("source", root.path().join("alias"))?;
    let source = inspect_benchmark_source_asset(root.path(), "source")?.context("source")?;
    let alias = inspect_benchmark_source_asset(root.path(), "alias")?.context("alias")?;
    assert_eq!(source.checksum_sha256, alias.checksum_sha256);
    assert_eq!(alias.source_path, "alias");
    std::os::unix::fs::symlink(".", root.path().join("source/cycle"))?;
    let error = inspect_benchmark_source_asset(root.path(), "source").expect_err("cyclic input");
    assert!(error.to_string().contains("cyclic source directory"));
    Ok(())
}
