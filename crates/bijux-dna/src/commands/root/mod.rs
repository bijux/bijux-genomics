use std::path::Path;

use anyhow::Result;

use crate::commands::{cli, corpus, ena};

pub(crate) fn handle_ena_root(command: &cli::EnaCommand, cwd: &Path) -> Result<()> {
    match command {
        cli::EnaCommand::Select(args) => ena::select_snapshot(cwd, args)?,
        cli::EnaCommand::Fetch(args) => ena::fetch_from_snapshot(cwd, args)?,
    }
    Ok(())
}

pub(crate) fn handle_corpus_root(command: &cli::CorpusCommand, cwd: &Path) -> Result<()> {
    match command {
        cli::CorpusCommand::Materialize(args) => corpus::materialize_corpus(cwd, args)?,
        cli::CorpusCommand::Normalize { corpus } => corpus::normalize_corpus(cwd, corpus)?,
        cli::CorpusCommand::Validate { corpus } => corpus::validate_corpus(cwd, corpus)?,
        cli::CorpusCommand::List(args) => {
            if args.json {
                corpus::list_corpus_json(cwd, args.corpus.as_deref())?;
            } else {
                corpus::list_corpus_text(cwd, args.corpus.as_deref())?;
            }
        }
        cli::CorpusCommand::Diff { left, right, json } => {
            if *json {
                corpus::diff_manifests_json(cwd, left, right)?;
            } else {
                corpus::diff_manifests_text(cwd, left, right)?;
            }
        }
    }
    Ok(())
}
