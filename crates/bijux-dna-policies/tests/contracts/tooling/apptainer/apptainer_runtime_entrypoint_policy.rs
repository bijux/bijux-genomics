#![allow(non_snake_case)]
#[path = "../../../support/fs.rs"]
mod support;

use walkdir::WalkDir;

#[test]
fn policy__contracts__apptainer_runtime_entrypoint_policy__python_java_defs_have_deterministic_exec_entrypoint(
) {
    let root = support::workspace_root().join("containers").join("apptainer");
    let mut offenders = Vec::new();

    for entry in WalkDir::new(&root) {
        let Ok(entry) = entry else { continue };
        let path = entry.path();
        if !entry.file_type().is_file()
            || path.extension().and_then(|ext| ext.to_str()) != Some("def")
        {
            continue;
        }

        let Ok(raw) = std::fs::read_to_string(path) else {
            continue;
        };
        let lowered = raw.to_ascii_lowercase();
        let runscript = raw
            .split("%runscript")
            .nth(1)
            .and_then(|chunk| chunk.split("\n%").next())
            .unwrap_or("");
        let runscript_lowered = runscript.to_ascii_lowercase();

        let references_python = lowered.contains("python3") || lowered.contains("python:");
        let references_java = lowered.contains("java -jar") || lowered.contains("openjdk");

        if references_python
            && !(runscript_lowered.contains("exec ") && runscript.contains("\"$@\""))
        {
            offenders.push(format!(
                "{}: python runtime present but runscript is not deterministic exec passthrough",
                path.display()
            ));
        }

        if references_java && !(runscript_lowered.contains("exec ") && runscript.contains("\"$@\""))
        {
            offenders.push(format!(
                "{}: java runtime present but runscript is not deterministic exec passthrough",
                path.display()
            ));
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "apptainer runtime/entrypoint policy violations:\n{}",
        offenders.join("\n")
    );
}
