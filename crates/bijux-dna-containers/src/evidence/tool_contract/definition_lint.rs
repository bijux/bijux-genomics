//! Pure lexical admission for acquired Apptainer definition text.

use std::fmt;

/// One lexical contract violation in an acquired Apptainer definition.
///
/// Issues retain declaration order and can be accumulated before command presentation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ApptainerDefinitionIssue {
    /// A package removal command is present.
    BannedPackageRemoval { command: &'static str },
    /// Required sections exist in an unsupported order.
    InvalidSectionOrder,
    /// At least one required section is absent.
    MissingRequiredSections,
    /// The runscript does not contain an exec invocation.
    RunscriptDoesNotExec,
    /// The runscript does not forward the quoted argument vector.
    RunscriptDoesNotForwardArguments,
    /// A governed OCI metadata label is absent.
    MissingOciLabel { label: &'static str },
    /// A separate self-report metadata file duplicates OCI labels.
    DuplicatedSelfReportMetadata,
    /// A separate self-report command duplicates OCI labels.
    DuplicatedSelfReportCommand,
}

impl fmt::Display for ApptainerDefinitionIssue {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BannedPackageRemoval { command } => {
                write!(formatter, "contains banned `{command}`")
            }
            Self::InvalidSectionOrder => formatter.write_str(
                "section order must be %labels -> %environment -> %post -> %runscript -> %help",
            ),
            Self::MissingRequiredSections => formatter.write_str(
                "missing required sections (%labels,%environment,%post,%runscript,%help)",
            ),
            Self::RunscriptDoesNotExec => formatter.write_str("%runscript must use exec"),
            Self::RunscriptDoesNotForwardArguments => {
                formatter.write_str("%runscript must pass through \"$@\"")
            }
            Self::MissingOciLabel { label } => {
                write!(formatter, "missing OCI metadata label {label}")
            }
            Self::DuplicatedSelfReportMetadata => formatter.write_str(
                "duplicated self-report metadata /opt/bijux/VERSION.json is forbidden; use OCI labels instead",
            ),
            Self::DuplicatedSelfReportCommand => formatter.write_str(
                "duplicated self-report command bijux-tool-info is forbidden; use OCI labels instead",
            ),
        }
    }
}

/// Inspect acquired definition text without filesystem or runtime access.
///
/// This preserves lexical substring and section-marker checks; it is not an Apptainer parser
/// or proof that an image can be built or executed. Governed metadata checks apply only when
/// requested by the caller's source scope.
#[must_use]
pub fn inspect_apptainer_definition(
    raw: &str,
    require_governed_metadata: bool,
) -> Vec<ApptainerDefinitionIssue> {
    let mut issues = Vec::new();
    let lowered = raw.to_ascii_lowercase();
    for banned in ["apt-get purge", "apt purge", "apt-get autoremove", "apt autoremove"] {
        if lowered.contains(banned) {
            issues.push(ApptainerDefinitionIssue::BannedPackageRemoval { command: banned });
        }
    }
    let labels = raw.lines().position(|line| line.trim() == "%labels");
    let env = raw.lines().position(|line| line.trim() == "%environment");
    let post = raw.lines().position(|line| line.trim() == "%post");
    let runscript = raw.lines().position(|line| line.trim() == "%runscript");
    let help = raw.lines().position(|line| line.trim() == "%help");
    match (labels, env, post, runscript, help) {
        (
            Some(labels_index),
            Some(env_index),
            Some(post_index),
            Some(runscript_index),
            Some(help_index),
        ) => {
            if !(labels_index < env_index
                && env_index < post_index
                && post_index < runscript_index
                && runscript_index < help_index)
            {
                issues.push(ApptainerDefinitionIssue::InvalidSectionOrder);
            }
        }
        _ => issues.push(ApptainerDefinitionIssue::MissingRequiredSections),
    }
    if let Some(runscript_chunk) = apptainer_section_body(raw, "runscript") {
        if !runscript_chunk.contains("exec ") {
            issues.push(ApptainerDefinitionIssue::RunscriptDoesNotExec);
        }
        if !runscript_chunk.contains("\"$@\"") {
            issues.push(ApptainerDefinitionIssue::RunscriptDoesNotForwardArguments);
        }
    }
    if require_governed_metadata {
        for label in [
            "org.opencontainers.image.source",
            "org.opencontainers.image.revision",
            "org.opencontainers.image.created",
            "org.opencontainers.image.licenses",
            "org.opencontainers.image.version",
            "org.opencontainers.image.tool",
            "org.opencontainers.image.title",
        ] {
            if !raw.contains(label) {
                issues.push(ApptainerDefinitionIssue::MissingOciLabel { label });
            }
        }
        if raw.contains("/opt/bijux/VERSION.json") {
            issues.push(ApptainerDefinitionIssue::DuplicatedSelfReportMetadata);
        }
        if raw.contains("bijux-tool-info") {
            issues.push(ApptainerDefinitionIssue::DuplicatedSelfReportCommand);
        }
    }
    issues
}

fn apptainer_section_body<'a>(raw: &'a str, section: &str) -> Option<&'a str> {
    raw.split(&format!("%{section}"))
        .nth(1)
        .and_then(|chunk| chunk.split("\n%").next())
        .map(str::trim)
        .filter(|chunk| !chunk.is_empty())
}
