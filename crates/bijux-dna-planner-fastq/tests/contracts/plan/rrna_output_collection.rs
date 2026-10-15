use std::fs;
use std::io::{Read, Write};
use std::path::Path;
use std::process::{Command, Output};

use anyhow::{Context, Result};
use bijux_dna_planner_fastq::tool_adapters::fastq::deplete_rrna::plan_rrna;
use bijux_dna_stage_contract::StagePlanV1;
use flate2::{read::GzDecoder, write::GzEncoder, Compression};

const RETAINED_R1: &[u8] = b"@retained-r1\nACGT\n+\nIIII\n";
const RETAINED_R2: &[u8] = b"@retained-r2\nTGCA\n+\nIIII\n";
const REMOVED_R1: &[u8] = b"@removed-r1\nAAAA\n+\nIIII\n";
const REMOVED_R2: &[u8] = b"@removed-r2\nTTTT\n+\nIIII\n";

// Only the external tool is replaced. The planner's entire Bash body follows this
// function unchanged, so its collection, compression and publication really run.
const OWNED_TOOL: &str = r#"sortmerna() {
    printf '%s\0' "$@" > "$BIJUX_SORTMERNA_ARGV"
    local workdir='' readb='' file
    while [ "$#" -gt 0 ]; do
        case "$1" in
            --workdir) workdir="$2"; shift ;;
            --readb) readb="$2"; shift ;;
        esac
        shift
    done
    for file in "$BIJUX_SORTMERNA_PAYLOAD/out/"*; do
        if [ -f "$file" ]; then cp -- "$file" "$workdir/out/"; fi
    done
    for file in "$BIJUX_SORTMERNA_PAYLOAD/readb/"*; do
        if [ -f "$file" ]; then cp -- "$file" "$readb/"; fi
    done
    printf 'owned rRNA collection report\n' > "$workdir/out/aligned.log"
    printf 'SORTMERNA_COLLECTION_STUB_COMPLETED\n' >&2
}
"#;

#[derive(Clone, Copy)]
enum Layout {
    Single,
    Paired,
    Readb,
    Overlapping,
    MissingR2,
}

struct Observation {
    plan: StagePlanV1,
    output: Output,
}

fn payload(path: &Path, bytes: &[u8]) -> Result<()> {
    if path.extension().is_some_and(|extension| extension == "gz") {
        let mut encoder = GzEncoder::new(fs::File::create(path)?, Compression::default());
        encoder.write_all(bytes)?;
        encoder.finish()?;
    } else {
        fs::write(path, bytes)?;
    }
    Ok(())
}

fn observe(root: &Path, directory: &str, layout: Layout) -> Result<Observation> {
    let case = root.join(directory);
    let source = case.join("payload");
    for name in ["out", "readb", "home", "scratch", "artifacts"] {
        fs::create_dir_all(source.join(name))?;
    }
    let paired = !matches!(layout, Layout::Single);
    let (folder, names) = match layout {
        Layout::Single => ("out", ["other.fq.gz", "", "aligned.fastq", ""]),
        Layout::Readb => {
            ("readb", ["fwd_0.fq.gz", "rev_0.fastq", "aligned_fwd_0.fastq.gz", "aligned_rev_0.fq"])
        }
        Layout::Overlapping => (
            "out",
            [
                "other_paired_fwd.fastq.gz",
                "other_paired_rev.fq",
                "aligned_paired_aligned_fwd.fq.gz",
                "aligned_paired_aligned_rev.fastq",
            ],
        ),
        Layout::Paired | Layout::MissingR2 => (
            "out",
            [
                "other_fwd.fastq.gz",
                "other_rev.fq",
                "aligned_aligned_fwd.fq.gz",
                "aligned_aligned_rev.fastq",
            ],
        ),
    };
    for (index, (name, bytes)) in
        names.into_iter().zip([RETAINED_R1, RETAINED_R2, REMOVED_R1, REMOVED_R2]).enumerate()
    {
        if !(name.is_empty() || matches!(layout, Layout::MissingR2) && index == 1) {
            payload(&source.join(folder).join(name), bytes)?;
        }
    }
    let r1 = case.join("input R1.fastq");
    let r2 = case.join("input R2.fastq");
    fs::write(&r1, RETAINED_R1)?;
    fs::write(&r2, RETAINED_R2)?;
    let out = case.join("result");
    let plan = plan_rrna(
        &super::domain_tool("fastq.deplete_rrna", "sortmerna"),
        &r1,
        paired.then_some(r2.as_path()),
        &out,
    )?;
    let [program, flag, body] = plan.command.template.as_slice() else {
        anyhow::bail!("rRNA command must expose its actual shell argv");
    };
    assert_eq!(program, "bash");
    let argv_path = source.join("tool-argv.bin");
    let inherited_path = std::env::var_os("PATH").context("declare inherited executable PATH")?;
    let output = Command::new("bash")
        .env_clear()
        .env("PATH", inherited_path)
        .env("LC_ALL", "C")
        .env("PS1", "")
        .env("TZ", "UTC")
        .arg(flag)
        .arg(format!("{OWNED_TOOL}\n{body}"))
        .current_dir(&case)
        .env("HOME", source.join("home"))
        .env("TMPDIR", source.join("scratch"))
        .env("TEST_TMP_DIR", source.join("scratch"))
        .env("ARTIFACT_ROOT", source.join("artifacts"))
        .env("ISO_ROOT", source.join("artifacts"))
        .env("ARTIFACT_DIR", source.join("artifacts"))
        .env("BIJUX_SORTMERNA_PAYLOAD", &source)
        .env("BIJUX_SORTMERNA_ARGV", &argv_path)
        .env_remove("BASH_ENV")
        .env_remove("ENV")
        .output()?;
    let diagnostic = String::from_utf8_lossy(&output.stderr);
    assert_eq!(
        diagnostic.matches("SORTMERNA_COLLECTION_STUB_COMPLETED").count(),
        1,
        "tool fixture completion differs: {diagnostic}"
    );
    assert_tool_argv(&argv_path, &r1, &r2, &out, paired)?;
    Ok(Observation { plan, output })
}

fn assert_tool_argv(
    argv_path: &Path,
    r1: &Path,
    r2: &Path,
    out: &Path,
    paired: bool,
) -> Result<()> {
    let raw = fs::read(argv_path)?;
    assert_eq!(raw.last(), Some(&0), "tool argv must end with its actual NUL separator");
    let argv = raw
        .split(|byte| *byte == 0)
        .filter(|part| !part.is_empty())
        .map(|part| String::from_utf8(part.to_vec()))
        .collect::<std::result::Result<Vec<_>, _>>()?;
    assert_eq!(
        argv.iter().filter(|arg| arg.as_str() == "--reads").count(),
        if paired { 2 } else { 1 }
    );
    for (key, value) in [
        ("--reads", r1.to_path_buf()),
        ("--workdir", out.join("sortmerna_workdir")),
        ("--readb", out.join("sortmerna_workdir/readb")),
    ] {
        let expected = if key == "--workdir" {
            format!("{}/", value.display())
        } else {
            value.display().to_string()
        };
        assert!(
            argv.windows(2).any(|pair| pair[0] == key && pair[1] == expected),
            "tool argv lost {key}"
        );
    }
    if paired {
        assert!(argv
            .windows(2)
            .any(|pair| pair[0] == "--reads" && pair[1] == r2.to_string_lossy()));
    }
    Ok(())
}

fn artifact<'a>(plan: &'a StagePlanV1, name: &str) -> &'a Path {
    &plan
        .io
        .outputs
        .iter()
        .find(|item| item.name.as_str() == name)
        .unwrap_or_else(|| panic!("missing {name}"))
        .path
}

fn decoded(path: &Path) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    GzDecoder::new(fs::File::open(path)?).read_to_end(&mut bytes)?;
    Ok(bytes)
}

fn collected(observed: &Observation, paired: bool) -> Result<()> {
    assert!(
        observed.output.status.success(),
        "collector failed: {}",
        String::from_utf8_lossy(&observed.output.stderr)
    );
    for (name, expected) in
        [("rrna_filtered_reads_r1", RETAINED_R1), ("rrna_removed_reads_r1", REMOVED_R1)]
    {
        assert_eq!(
            decoded(artifact(&observed.plan, name))?,
            expected,
            "{name} multiplicity or bytes changed"
        );
    }
    if paired {
        for (name, expected) in
            [("rrna_filtered_reads_r2", RETAINED_R2), ("rrna_removed_reads_r2", REMOVED_R2)]
        {
            assert_eq!(
                decoded(artifact(&observed.plan, name))?,
                expected,
                "{name} multiplicity or bytes changed"
            );
        }
    }
    assert_eq!(
        fs::read(artifact(&observed.plan, "rrna_report_tsv"))?,
        b"owned rRNA collection report\n"
    );
    assert_eq!(fs::read(artifact(&observed.plan, "rrna_report_json"))?, b"{}\n");
    Ok(())
}

#[test]
fn sortmerna_collection_preserves_literal_parent_paths() -> Result<()> {
    let owned = bijux_dna_testkit::tempdir_for("sortmerna-literal-parent");
    collected(&observe(owned.path(), "ordinary", Layout::Single)?, false)?;
    collected(&observe(owned.path(), "readb", Layout::Readb)?, true)?;
    let missing = observe(owned.path(), "missing-r2", Layout::MissingR2)?;
    assert!(!missing.output.status.success());
    let diagnostic = String::from_utf8_lossy(&missing.output.stderr);
    assert!(diagnostic.contains("missing expected SortMeRNA output"));
    assert!(diagnostic.contains(
        artifact(&missing.plan, "rrna_filtered_reads_r2").to_str().context("owned output path")?
    ));
    assert_eq!(decoded(artifact(&missing.plan, "rrna_filtered_reads_r1"))?, RETAINED_R1);
    assert!(!artifact(&missing.plan, "rrna_filtered_reads_r2").exists());
    assert!(!artifact(&missing.plan, "rrna_report_json").exists());
    eprintln!("SORTMERNA_LITERAL_PARENT_CONTROLS_PASSED");
    for directory in ["space in directory", "literal [x]*?' parent"] {
        eprintln!("SORTMERNA_LITERAL_PARENT_VECTOR {directory}");
        collected(&observe(owned.path(), directory, Layout::Paired)?, true)
            .with_context(|| format!("literal parent {directory}"))?;
    }
    Ok(())
}

#[test]
fn sortmerna_collection_emits_each_matched_file_once() -> Result<()> {
    let owned = bijux_dna_testkit::tempdir_for("sortmerna-overlapping-patterns");
    collected(&observe(owned.path(), "ordinary", Layout::Paired)?, true)?;
    eprintln!("SORTMERNA_OVERLAP_CONTROL_PASSED");
    let overlapping = observe(owned.path(), "overlap", Layout::Overlapping)?;
    eprintln!("SORTMERNA_OVERLAP_VECTOR status={}", overlapping.output.status);
    collected(&overlapping, true)?;
    Ok(())
}
