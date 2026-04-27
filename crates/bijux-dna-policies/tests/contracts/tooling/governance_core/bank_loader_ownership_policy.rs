#![allow(non_snake_case)]
use std::path::PathBuf;

use walkdir::WalkDir;

fn repo_root() -> PathBuf {
    bijux_dna_testkit::workspace_root_from_manifest(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn policy__contracts__bank_loader_ownership_policy__bank_loader_definitions_owned_by_domain_fastq()
{
    let root = repo_root();
    let crates_dir = root.join("crates");
    let mut offenders = Vec::new();
    let allowed_prefix = root
        .join("crates")
        .join("bijux-dna-domain-fastq")
        .join("src")
        .to_string_lossy()
        .to_string();
    let symbols = [
        "fn load_adapter_bank(",
        "fn load_adapter_presets(",
        "fn load_polyx_bank(",
        "fn load_polyx_presets(",
        "fn load_contaminant_motifs(",
        "fn load_contaminant_presets(",
    ];
    for entry in WalkDir::new(&crates_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
    {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
            continue;
        }
        let path_s = path.to_string_lossy();
        if path_s.contains("/tests/") {
            continue;
        }
        let content = std::fs::read_to_string(path).expect("read source");
        if symbols.iter().any(|symbol| content.contains(symbol))
            && !path_s.starts_with(&allowed_prefix)
        {
            offenders.push(path.display().to_string());
        }
    }
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "bank loader definitions must be centralized in bijux-dna-domain-fastq:\n{}",
        offenders.join("\n")
    );
}
