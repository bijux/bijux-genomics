use std::path::PathBuf;

use anyhow::{anyhow, Result};
use bijux_dna_analyze::exports::{
    list_reviewer_challenges, submit_reviewer_challenge, verify_evidence_bundle,
    verify_profile_bundle, write_methods_summary_json, write_profile_bundle_json,
    EvidenceBundleProfileV1, ReviewerChallengeRequestV1,
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
            let profile = parse_profile(args.get(1).map_or("publication_strict", String::as_str))?;
            let facts_path = args.get(2).map(PathBuf::from);
            let output = write_profile_bundle_json(&run_dir, facts_path.as_deref(), profile)?;
            println!("{}", output.display());
        }
        "challenge-submit" => {
            let run_dir = parse_required_path(&args, 0, "run_dir")?;
            let artifact_id = parse_required_arg(&args, 1, "artifact_id")?;
            let evidence_path = parse_required_arg(&args, 2, "evidence_path")?;
            let report_field = parse_required_arg(&args, 3, "report_field")?;
            let caveat = parse_required_arg(&args, 4, "caveat")?;
            let question = parse_required_arg(&args, 5, "question")?;
            let requested_by = parse_required_arg(&args, 6, "requested_by")?;
            let request = ReviewerChallengeRequestV1 {
                artifact_id,
                evidence_path,
                report_field,
                caveat,
                question,
                requested_by,
            };
            let challenge = submit_reviewer_challenge(&run_dir, &request)?;
            println!("{}", serde_json::to_string_pretty(&challenge)?);
        }
        "challenge-list" => {
            let run_dir = parse_required_path(&args, 0, "run_dir")?;
            let rows = list_reviewer_challenges(&run_dir)?;
            println!("{}", serde_json::to_string_pretty(&rows)?);
        }
        other => {
            print_usage();
            return Err(anyhow!("unsupported command `{other}`"));
        }
    }
    Ok(())
}

fn parse_required_path(args: &[String], index: usize, label: &str) -> Result<PathBuf> {
    let value = args.get(index).ok_or_else(|| anyhow!("missing required argument `{label}`"))?;
    let path = PathBuf::from(value);
    if !path.exists() {
        return Err(anyhow!("path does not exist: {}", path.display()));
    }
    Ok(path)
}

fn parse_required_arg(args: &[String], index: usize, label: &str) -> Result<String> {
    let value =
        args.get(index).ok_or_else(|| anyhow!("missing required argument `{label}`"))?.trim();
    if value.is_empty() {
        return Err(anyhow!("argument `{label}` cannot be empty"));
    }
    Ok(value.to_string())
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
        "usage:\n  bijux-dna-verify verify-evidence <evidence_bundle.json>\n  bijux-dna-verify verify-profile <profile_bundle.json>\n  bijux-dna-verify write-methods <run_dir> [facts.jsonl]\n  bijux-dna-verify write-profile <run_dir> [profile] [facts.jsonl]\n  bijux-dna-verify challenge-submit <run_dir> <artifact_id> <evidence_path> <report_field> <caveat> <question> <requested_by>\n  bijux-dna-verify challenge-list <run_dir>"
    );
}
