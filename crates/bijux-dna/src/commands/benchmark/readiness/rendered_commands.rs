use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Serialize;

use super::benchmark_command_rows::{collect_benchmark_command_rows, render_shell_command};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_RENDERED_COMMANDS_PATH: &str =
    "artifacts/benchmark-smoke/readiness/rendered-commands.sh";
const RENDERED_COMMANDS_SCHEMA_VERSION: &str = "bijux.bench.readiness.rendered_commands.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct RenderedCommandRow {
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) readiness_kind: String,
    pub(crate) argv: Vec<String>,
    pub(crate) command: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct RenderedCommandsReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) row_count: usize,
    pub(crate) rows: Vec<RenderedCommandRow>,
}

pub(crate) fn run_render_commands(args: &parse::BenchReadinessRenderCommandsArgs) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_commands(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_RENDERED_COMMANDS_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_commands(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<RenderedCommandsReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let rows = collect_rendered_command_rows(repo_root)?;

    if let Some(parent) = output_path.parent() {
        bijux_dna_infra::ensure_dir(parent)
            .with_context(|| format!("create {}", parent.display()))?;
    }
    let rendered = render_commands_shell_script(repo_root, &rows);
    bijux_dna_infra::atomic_write_bytes(&output_path, rendered.as_bytes())?;

    Ok(RenderedCommandsReport {
        schema_version: RENDERED_COMMANDS_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &output_path),
        row_count: rows.len(),
        rows,
    })
}

pub(crate) fn collect_rendered_command_rows(repo_root: &Path) -> Result<Vec<RenderedCommandRow>> {
    collect_benchmark_command_rows(repo_root)?
        .into_iter()
        .map(|entry| {
            let command = render_shell_command(&entry.argv);
            Ok(RenderedCommandRow {
                stage_id: entry.stage_id,
                tool_id: entry.tool_id,
                readiness_kind: entry.readiness_kind,
                argv: entry.argv,
                command,
            })
        })
        .collect()
}

fn render_commands_shell_script(repo_root: &Path, rows: &[RenderedCommandRow]) -> String {
    let mut rendered = String::from("#!/usr/bin/env bash\nset -euo pipefail\n");
    rendered.push_str(&format!(
        "repo_root={}\n",
        render_shell_command(&[repo_root.to_string_lossy().into_owned()])
    ));
    rendered.push_str("cd \"$repo_root\"\n\n");
    for row in rows {
        rendered.push_str(&format!("# {} / {}\n{}\n", row.stage_id, row.tool_id, row.command));
    }
    rendered
}

fn repo_relative_path(repo_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo_root.join(path)
    }
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).display().to_string()
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    struct CurrentDirGuard(PathBuf);

    impl CurrentDirGuard {
        fn enter(path: &std::path::Path) -> Self {
            let original = std::env::current_dir().expect("capture current dir");
            std::env::set_current_dir(path).expect("set current dir");
            Self(original)
        }
    }

    impl Drop for CurrentDirGuard {
        fn drop(&mut self) {
            std::env::set_current_dir(&self.0).expect("restore current dir");
        }
    }

    fn repo_root() -> PathBuf {
        crate::commands::support::workspace_root::resolve_repo_root().expect("repo root")
    }

    #[cfg(feature = "bam_downstream")]
    #[test]
    fn rendered_commands_report_governed_benchmark_ready_row_slice() {
        use super::{render_commands, DEFAULT_RENDERED_COMMANDS_PATH};
        use crate::commands::benchmark::repo_locking::{acquire_cwd_lock, RepoProcessLock};

        let root = repo_root();
        let _cwd_lock = acquire_cwd_lock();
        let _repo_lock =
            RepoProcessLock::acquire(&root, "benchmark-readiness-mutators").expect("repo lock");
        let _cwd = CurrentDirGuard::enter(&root);
        let report = render_commands(&root, PathBuf::from(DEFAULT_RENDERED_COMMANDS_PATH))
            .expect("render commands");

        assert_eq!(report.schema_version, "bijux.bench.readiness.rendered_commands.v1");
        assert_eq!(report.output_path, "artifacts/benchmark-smoke/readiness/rendered-commands.sh");
        assert!(report.row_count > 0);
        assert_eq!(report.rows.len(), report.row_count);
        assert!(report.rows.iter().all(|row| {
            !row.argv.is_empty()
                && !row.command.is_empty()
                && row.command == crate::commands::benchmark::readiness::benchmark_command_rows::render_shell_command(&row.argv)
        }));
    }

    fn observe_rendered_command(
        root: &std::path::Path,
        relative: &std::path::Path,
        outside: &std::path::Path,
        arguments: &[&str],
    ) -> anyhow::Result<Vec<String>> {
        use anyhow::{ensure, Context};
        let script = super::repo_relative_path(root, relative);
        let record = script.with_extension("observed");
        std::fs::create_dir_all(script.parent().context("script parent")?)?;
        let mut argv = vec![
            "bash".to_owned(),
            "-c".to_owned(),
            r#"record=$1; shift; printf '%s\0' "$PWD" "$@" > "$record""#.to_owned(),
            "rendered-command-recorder".to_owned(),
            record.to_str().context("UTF-8 record path")?.to_owned(),
        ];
        argv.extend(arguments.iter().map(|value| (*value).to_owned()));
        let row = super::RenderedCommandRow {
            stage_id: "root-observation".to_owned(),
            tool_id: "bash".to_owned(),
            readiness_kind: "argv-recorder".to_owned(),
            command: super::render_shell_command(&argv),
            argv,
        };
        std::fs::write(&script, super::render_commands_shell_script(root, &[row]))?;
        let output =
            std::process::Command::new("bash").arg(&script).current_dir(outside).output()?;
        ensure!(
            output.status.success(),
            "rendered script {} exited {}: stdout={} stderr={}",
            script.display(),
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let bytes = std::fs::read(&record)?;
        let bytes = bytes.strip_suffix(&[0]).context("NUL-terminated observation")?;
        bytes
            .split(|byte| *byte == 0)
            .map(|field| String::from_utf8(field.to_vec()).context("UTF-8 observation"))
            .collect()
    }

    #[test]
    fn rendered_command_scripts_execute_in_the_admitted_root_and_preserve_argv(
    ) -> anyhow::Result<()> {
        let fixture = bijux_dna_testkit::tempdir_for("rendered-command-repository-binding").keep();
        let root = fixture.join("repository's root $literal");
        std::fs::create_dir(&root)?;
        let root = root.canonicalize()?;
        let expected_root = root.to_string_lossy().into_owned();
        let arguments =
            ["two words", "quote'argument", "$literal; $(not-a-command)", "", "line\nbreak"];
        let control = observe_rendered_command(
            &root,
            std::path::Path::new("artifacts/control/commands.sh"),
            &fixture,
            &arguments,
        )?;
        assert_eq!(control[0], expected_root, "plain admitted-root control");
        assert_eq!(&control[1..], arguments, "control argv is literal");
        println!("RENDERED_COMMAND_ROOT_CONTROL_PASS");
        let mut observations = Vec::new();
        for (kind, relative) in [
            ("default", super::DEFAULT_RENDERED_COMMANDS_PATH),
            ("custom", "artifacts/custom/deep/readiness/scripts/commands.sh"),
        ] {
            let fields = observe_rendered_command(
                &root,
                std::path::Path::new(relative),
                &fixture,
                &arguments,
            )?;
            observations.push((kind, fields));
        }
        println!(
            "RENDERED_COMMAND_ROOT_OBSERVATIONS {}",
            serde_json::to_string(&serde_json::json!({
                "fixture_root": fixture,
                "control": control,
                "expected_root": expected_root,
                "expected_arguments": arguments,
                "observations": observations,
            }))?
        );
        for (kind, fields) in &observations {
            assert_eq!(&fields[1..], arguments, "{kind} argv is literal");
        }
        for (kind, fields) in &observations {
            assert_eq!(
                fields[0], expected_root,
                "{kind} script must execute at the admitted repository"
            );
        }
        Ok(())
    }
}
