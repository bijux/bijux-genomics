use crate::request_args::{OperatorDiagnosisResponseV1, RedactionProfileV1, RunBrowserResponseV1};
use std::path::PathBuf;

pub fn redact_run_browser(
    mut response: RunBrowserResponseV1,
    profile: Option<RedactionProfileV1>,
) -> RunBrowserResponseV1 {
    let Some(profile) = profile else {
        return response;
    };
    response.runs_root = redact_path(&response.runs_root, profile);
    for row in &mut response.rows {
        row.run_dir = redact_path(&row.run_dir, profile);
        if matches!(profile, RedactionProfileV1::External) {
            row.correlation_id = None;
        }
    }
    response
}

pub fn redact_operator_diagnosis(
    mut response: OperatorDiagnosisResponseV1,
    profile: Option<RedactionProfileV1>,
) -> OperatorDiagnosisResponseV1 {
    let Some(profile) = profile else {
        return response;
    };
    response.run_dir = redact_path(&response.run_dir, profile);
    for command in &mut response.commands {
        command.evidence_path =
            command.evidence_path.as_ref().map(|path| redact_path(path, profile));
        command.argv = redact_argv(&command.argv, profile);
    }
    if matches!(profile, RedactionProfileV1::External) {
        response.run_id = "redacted-run".to_string();
    }
    response
}

fn redact_path(path: &std::path::Path, profile: RedactionProfileV1) -> PathBuf {
    match profile {
        RedactionProfileV1::Collaborator => {
            path.file_name().map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."))
        }
        RedactionProfileV1::External => PathBuf::from("<redacted-path>"),
    }
}

fn redact_argv(argv: &[String], profile: RedactionProfileV1) -> Vec<String> {
    if argv.is_empty() {
        return Vec::new();
    }
    match profile {
        RedactionProfileV1::Collaborator => argv
            .iter()
            .map(|value| {
                if value.starts_with('/') {
                    PathBuf::from(value)
                        .file_name()
                        .and_then(|name| name.to_str())
                        .unwrap_or("redacted")
                        .to_string()
                } else {
                    value.clone()
                }
            })
            .collect(),
        RedactionProfileV1::External => {
            let mut redacted = vec![argv[0].clone()];
            if argv.len() > 1 {
                redacted.push("<redacted-args>".to_string());
            }
            redacted
        }
    }
}
