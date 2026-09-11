#![cfg(test)]
use anyhow::Result;
use serde_json::Value;
use std::path::Path;

fn copy_tree(source: &Path, destination: &Path) -> Result<()> {
    if source.is_dir() {
        bijux_dna_infra::ensure_dir(destination)?;
        for entry in std::fs::read_dir(source)? {
            let entry = entry?;
            copy_tree(&entry.path(), &destination.join(entry.file_name()))?;
        }
    } else {
        bijux_dna_infra::ensure_dir(destination.parent().expect("fixture parent"))?;
        std::fs::copy(source, destination)?;
    }
    Ok(())
}

fn copy_bam_fixture(repository: &Path) -> Result<()> {
    let source = crate::test_support::repo_root();
    let corpus = "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini";
    copy_tree(&source.join(corpus), &repository.join(corpus))?;
    let manifest: toml::Value =
        toml::from_str(&std::fs::read_to_string(source.join(corpus).join("manifest.toml"))?)?;
    for sample in manifest["samples"].as_array().expect("sample declarations") {
        for path in sample["source_paths"].as_array().expect("sample evidence") {
            let path = path.as_str().expect("source path");
            if !repository.join(path).exists() {
                copy_tree(&source.join(path), &repository.join(path))?;
            }
        }
    }
    Ok(())
}

fn snapshot_files(root: &Path) -> Result<Vec<(std::path::PathBuf, Vec<u8>)>> {
    let mut files = Vec::new();
    for entry in std::fs::read_dir(root)? {
        let entry = entry?;
        if entry.path().is_dir() {
            files.extend(snapshot_files(&entry.path())?);
        } else {
            files.push((entry.path(), std::fs::read(entry.path())?));
        }
    }
    Ok(files)
}

fn retains_native_evidence(stage: &str) -> Result<()> {
    let repository = tempfile::tempdir()?;
    if stage != "gl_propagation" {
        copy_bam_fixture(repository.path())?;
    }
    let inputs = crate::test_support::inputs();
    let execute = || -> Result<Value> {
        let root = repository.path();
        Ok(match stage {
            "call" => serde_json::to_value(crate::call::run_local_vcf_call_smoke(
                &inputs, root, "bcftools",
            )?)?,
            "call_gl" => serde_json::to_value(crate::call_gl::run_local_vcf_call_gl_smoke(
                &inputs, root, "bcftools",
            )?)?,
            "call_diploid" => serde_json::to_value(
                crate::call_diploid::run_local_vcf_call_diploid_smoke(&inputs, root, "bcftools")?,
            )?,
            "call_pseudohaploid" => serde_json::to_value(
                crate::call_pseudohaploid::run_local_vcf_call_pseudohaploid_smoke(
                    &inputs, root, "bcftools",
                )?,
            )?,
            _ => serde_json::to_value(crate::gl_propagation::run_local_vcf_gl_propagation_smoke(
                &inputs, root, "bcftools",
            )?)?,
        })
    };
    let first = execute()?;
    let root = repository.path().join(first["output_root"].as_str().expect("report root"));
    let executions = root.join("executions");
    let retained = snapshot_files(&executions)?;
    assert!(!retained.is_empty());
    for (path, bytes) in &retained {
        if path.file_name().is_some_and(|name| name == "call_manifest.json") {
            let manifest: Value = serde_json::from_slice(bytes)?;
            for field in ["reference_fasta", "output", "metrics"] {
                let native = Path::new(manifest[field].as_str().expect("native path"));
                assert!(native.starts_with(&executions), "{field}: {}", native.display());
                assert!(native.is_file());
            }
        }
    }
    std::fs::write(root.join("user-evidence.txt"), b"retain")?;
    let repeated = execute()?;
    assert_eq!(repeated["output_root"], first["output_root"]);
    assert_eq!(std::fs::read(root.join("user-evidence.txt"))?, b"retain");
    assert_eq!(std::fs::read_dir(&executions)?.count(), 2);
    for (path, bytes) in retained {
        assert_eq!(std::fs::read(path)?, bytes);
    }
    let index: Value = serde_json::from_slice(&std::fs::read(root.join("stage-result.json"))?)?;
    for artifact in index["outputs"].as_array().expect("output index") {
        let output =
            repository.path().join(artifact["realized_path"].as_str().expect("output path"));
        assert!(output.starts_with(&root) && !output.starts_with(&executions));
        assert!(output.is_file());
    }
    if stage == "call_pseudohaploid" {
        assert_eq!(first["deterministic_replay_match"], true);
        assert_eq!(first["canonical_output_sha256"], first["canonical_replay_output_sha256"]);
    }
    Ok(())
}

#[test]
fn call_retains_native_evidence_across_publication() -> Result<()> {
    retains_native_evidence("call")
}
#[test]
fn call_gl_retains_native_evidence_across_publication() -> Result<()> {
    retains_native_evidence("call_gl")
}
#[test]
fn diploid_retains_native_evidence_across_publication() -> Result<()> {
    retains_native_evidence("call_diploid")
}
#[test]
fn pseudohaploid_retains_native_replay_across_publication() -> Result<()> {
    retains_native_evidence("call_pseudohaploid")
}
#[test]
fn likelihood_propagation_retains_native_evidence_across_publication() -> Result<()> {
    retains_native_evidence("gl_propagation")
}
