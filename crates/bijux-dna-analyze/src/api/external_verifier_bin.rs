use std::path::PathBuf;

use anyhow::{anyhow, Result};
use bijux_dna_analyze::exports::{
    verify_evidence_bundle, verify_profile_bundle, write_methods_summary_json,
    write_profile_bundle_json, EvidenceBundleProfileV1,
};

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.is_empty() {
        print_usage();
        return Err(anyhow!("missing command"));
    }
    let command = args.remove(0);
    match command.as_str() {
        "verify-evidence" => {
            let bundle_path = parse_required_path(&args, 0, "bundle_path")?;
            let verification = verify_evidence_bundle(&bundle_path)?;
            println!("{}", serde_json::to_string_pretty(&verification)?);
        }
        "verify-profile" => {
            let bundle_path = parse_required_path(&args, 0, "profile_bundle_path")?;
            let verification = verify_profile_bundle(&bundle_path)?;
            println!("{}", serde_json::to_string_pretty(&verification)?);
        }
        "write-methods" => {
            let run_dir = parse_required_path(&args, 0, "run_dir")?;
            let facts_path = args.get(1).map(PathBuf::from);
            let output = write_methods_summary_json(&run_dir, facts_path.as_deref())?;
            println!("{}", output.display());
        }
        "write-profile" => {
            let run_dir = parse_required_path(&args, 0, "run_dir")?;
            let profile = parse_profile(args.get(1).map(String::as_str).unwrap_or("publication_strict"))?;
            let facts_path = args.get(2).map(PathBuf::from);
            let output = write_profile_bundle_json(&run_dir, facts_path.as_deref(), profile)?;
            println!("{}", output.display());
        }
        other => {
            print_usage();
            return Err(anyhow!("unsupported command `{other}`"));
        }
    }
    Ok(())
}

fn parse_required_path(args: &[String], index: usize, label: &str) -> Result<PathBuf> {
    let value = args
        .get(index)
        .ok_or_else(|| anyhow!("missing required argument `{label}`"))?;
    let path = PathBuf::from(value);
    if !path.exists() {
        return Err(anyhow!("path does not exist: {}", path.display()));
    }
    Ok(path)
}

fn parse_profile(value: &str) -> Result<EvidenceBundleProfileV1> {
    match value {
        "draft" => Ok(EvidenceBundleProfileV1::Draft),
        "operational" => Ok(EvidenceBundleProfileV1::Operational),
        "certification" => Ok(EvidenceBundleProfileV1::Certification),
        "publication" => Ok(EvidenceBundleProfileV1::Publication),
        "publication_strict" => Ok(EvidenceBundleProfileV1::PublicationStrict),
        "collaborator_redacted" => Ok(EvidenceBundleProfileV1::CollaboratorRedacted),
        "archive_retention" => Ok(EvidenceBundleProfileV1::ArchiveRetention),
        other => Err(anyhow!(
            "unsupported profile `{other}`; expected one of: draft, operational, certification, publication, publication_strict, collaborator_redacted, archive_retention"
        )),
    }
}

fn print_usage() {
    eprintln!(
        "usage:\n  bijux-dna-verify verify-evidence <evidence_bundle.json>\n  bijux-dna-verify verify-profile <profile_bundle.json>\n  bijux-dna-verify write-methods <run_dir> [facts.jsonl]\n  bijux-dna-verify write-profile <run_dir> [profile] [facts.jsonl]"
    );
}
