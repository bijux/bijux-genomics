#![allow(non_snake_case)]
#[path = "../../../support/fs.rs"]
mod support;

use walkdir::WalkDir;

use support::workspace_root;

type SectionPositions = (Option<usize>, Option<usize>, Option<usize>, Option<usize>, Option<usize>);

fn section_positions(content: &str) -> SectionPositions {
    let mut labels = None;
    let mut environment = None;
    let mut post = None;
    let mut runscript = None;
    let mut help = None;
    for (idx, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed == "%labels" && labels.is_none() {
            labels = Some(idx);
        } else if trimmed == "%environment" && environment.is_none() {
            environment = Some(idx);
        } else if trimmed == "%post" && post.is_none() {
            post = Some(idx);
        } else if trimmed == "%runscript" && runscript.is_none() {
            runscript = Some(idx);
        } else if trimmed == "%help" && help.is_none() {
            help = Some(idx);
        }
    }
    (labels, environment, post, runscript, help)
}

#[test]
fn policy__contracts__apptainer_structure_policy__section_order_and_minimal_post_contract() {
    let root = workspace_root().join("containers").join("apptainer");
    let mut offenders = Vec::new();

    for entry in WalkDir::new(&root) {
        let Ok(entry) = entry else { continue };
        let path = entry.path();
        if !entry.file_type().is_file()
            || path.extension().and_then(|ext| ext.to_str()) != Some("def")
        {
            continue;
        }

        let Ok(content) = std::fs::read_to_string(path) else {
            continue;
        };
        let lowered = content.to_ascii_lowercase();

        let (labels, environment, post, runscript, help) = section_positions(&content);
        if labels.is_none()
            || environment.is_none()
            || post.is_none()
            || runscript.is_none()
            || help.is_none()
        {
            offenders.push(format!("{}: missing required section(s)", path.display()));
            continue;
        }
        let labels = labels.unwrap_or_default();
        let environment = environment.unwrap_or_default();
        let post = post.unwrap_or_default();
        let runscript = runscript.unwrap_or_default();
        let help = help.unwrap_or_default();
        if !(labels < environment && environment < post && post < runscript && runscript < help) {
            offenders.push(format!(
                "{}: sections must appear in order %labels -> %environment -> %post -> %runscript -> %help",
                path.display()
            ));
        }

        if lowered.contains("rm -rf /usr/bin")
            || lowered.contains("rm -rf /bin/")
            || lowered.contains("rm -rf /sbin/")
        {
            offenders.push(format!(
                "{}: must not remove base utilities from root filesystem paths",
                path.display()
            ));
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "Apptainer structure contract violations:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__contracts__apptainer_structure_policy__runscript_execs_tool_passthrough() {
    let root = workspace_root().join("containers").join("apptainer");
    let mut offenders = Vec::new();

    for entry in WalkDir::new(&root) {
        let Ok(entry) = entry else { continue };
        let path = entry.path();
        if !entry.file_type().is_file()
            || path.extension().and_then(|ext| ext.to_str()) != Some("def")
        {
            continue;
        }
        let Ok(content) = std::fs::read_to_string(path) else {
            continue;
        };
        let runscript = content
            .split("%runscript")
            .nth(1)
            .and_then(|chunk| chunk.split("\n%").next())
            .unwrap_or("");
        let lowered = runscript.to_ascii_lowercase();
        if !lowered.contains("exec ") || !runscript.contains("\"$@\"") {
            offenders.push(path.display().to_string());
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "Apptainer runscript must exec tool with \"$@\" passthrough:\n{}",
        offenders.join("\n")
    );
}
