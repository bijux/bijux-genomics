#![allow(non_snake_case)]
use std::path::PathBuf;

use walkdir::WalkDir;

fn repo_root() -> PathBuf {
    bijux_dna_testkit::workspace_root_from_manifest(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn policy__contracts__assets_scope_policy__assets_contains_data_only_no_code() {
    let root = repo_root();
    let assets = root.join("assets");
    let mut offenders = Vec::new();
    let allowed = [
        "yaml", "yml", "json", "jsonl", "toml", "txt", "md", "fasta", "fa", "fna", "sam", "bam",
        "bai", "bed", "bt2", "dat", "dict", "fai", "fastq", "fq", "tsv", "csv", "gz", "vcf",
        "html", "sha256",
    ];
    for entry in WalkDir::new(&assets)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
    {
        let path = entry.path();
        let ext = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
        if !allowed.contains(&ext) {
            offenders.push(path.display().to_string());
        }
    }
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "assets/ must contain data banks/references only (no code/executables):\n{}",
        offenders.join("\n")
    );
}
