#![cfg(test)]

use bijux_dna_containers::evidence::tool_contract::{
    inspect_apptainer_definition, ApptainerDefinitionIssue,
};

const VALID_DEFINITION: &str = r#"Bootstrap: docker
From: debian:bookworm-slim

%labels
    org.opencontainers.image.source https://example.invalid/tool
    org.opencontainers.image.revision aabbcc
    org.opencontainers.image.created 2000-01-01
    org.opencontainers.image.licenses MIT
    org.opencontainers.image.version 1.0
    org.opencontainers.image.tool tool
    org.opencontainers.image.title tool

%environment

%post
    true

%runscript
    exec tool "$@"

%help
    Tool usage
"#;

#[test]
fn definition_lint_preserves_required_sections_and_literal_runscript_forwarding() {
    assert!(inspect_apptainer_definition(VALID_DEFINITION, true).is_empty());

    let missing_section = VALID_DEFINITION.replace("%post\n", "%install\n");
    assert_eq!(
        inspect_apptainer_definition(&missing_section, true),
        [ApptainerDefinitionIssue::MissingRequiredSections]
    );
    let reordered_sections =
        VALID_DEFINITION.replace("%environment\n\n%post", "%post\n\n%environment");
    assert_eq!(
        inspect_apptainer_definition(&reordered_sections, true),
        [ApptainerDefinitionIssue::InvalidSectionOrder]
    );
    let no_exec = VALID_DEFINITION.replace("exec tool \"$@\"", "tool \"$@\"");
    assert_eq!(
        inspect_apptainer_definition(&no_exec, true),
        [ApptainerDefinitionIssue::RunscriptDoesNotExec]
    );
    let unquoted_arguments = VALID_DEFINITION.replace("exec tool \"$@\"", "exec tool $@");
    let issues = inspect_apptainer_definition(&unquoted_arguments, true);
    assert_eq!(issues, [ApptainerDefinitionIssue::RunscriptDoesNotForwardArguments]);
    assert_eq!(issues[0].to_string(), "%runscript must pass through \"$@\"");

    let missing_source = VALID_DEFINITION
        .replace("    org.opencontainers.image.source https://example.invalid/tool\n", "");
    assert!(inspect_apptainer_definition(&missing_source, false).is_empty());
    assert_eq!(
        inspect_apptainer_definition(&missing_source, true),
        [ApptainerDefinitionIssue::MissingOciLabel { label: "org.opencontainers.image.source" }]
    );
}

#[test]
fn definition_lint_preserves_multiple_diagnostics_in_declared_rule_order() {
    let raw = r"%labels
%post
    APT-GET PURGE obsolete
    apt autoremove
    apt purge obsolete
    apt-get autoremove
    /opt/bijux/VERSION.json
    bijux-tool-info
%environment
%runscript
    tool $@
%help
    Tool usage
";
    let issues = inspect_apptainer_definition(raw, true);
    assert_eq!(
        issues,
        [
            ApptainerDefinitionIssue::BannedPackageRemoval { command: "apt-get purge" },
            ApptainerDefinitionIssue::BannedPackageRemoval { command: "apt purge" },
            ApptainerDefinitionIssue::BannedPackageRemoval { command: "apt-get autoremove" },
            ApptainerDefinitionIssue::BannedPackageRemoval { command: "apt autoremove" },
            ApptainerDefinitionIssue::InvalidSectionOrder,
            ApptainerDefinitionIssue::RunscriptDoesNotExec,
            ApptainerDefinitionIssue::RunscriptDoesNotForwardArguments,
            ApptainerDefinitionIssue::MissingOciLabel { label: "org.opencontainers.image.source" },
            ApptainerDefinitionIssue::MissingOciLabel {
                label: "org.opencontainers.image.revision"
            },
            ApptainerDefinitionIssue::MissingOciLabel { label: "org.opencontainers.image.created" },
            ApptainerDefinitionIssue::MissingOciLabel {
                label: "org.opencontainers.image.licenses"
            },
            ApptainerDefinitionIssue::MissingOciLabel { label: "org.opencontainers.image.version" },
            ApptainerDefinitionIssue::MissingOciLabel { label: "org.opencontainers.image.tool" },
            ApptainerDefinitionIssue::MissingOciLabel { label: "org.opencontainers.image.title" },
            ApptainerDefinitionIssue::DuplicatedSelfReportMetadata,
            ApptainerDefinitionIssue::DuplicatedSelfReportCommand,
        ]
    );
    assert_eq!(issues[0].to_string(), "contains banned `apt-get purge`");
    assert_eq!(issues[7].to_string(), "missing OCI metadata label org.opencontainers.image.source");
}
