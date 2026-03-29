use super::*;

pub(super) fn assets_refresh_golden(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
    ensure_help_only("refresh-golden", args)?;
    let out_dir = workspace.path("artifacts/assets-refresh/golden/toy-runs-v1");
    let target_dir = workspace.path("assets/golden/toy-runs-v1");
    let report_path = workspace.path("artifacts/assets-refresh/golden/report.json");

    if out_dir.exists() {
        fs::remove_dir_all(&out_dir).with_context(|| format!("remove {}", out_dir.display()))?;
    }
    if let Some(parent) = out_dir.parent() {
        bijux_dna_infra::ensure_dir(parent)
            .with_context(|| format!("create {}", parent.display()))?;
    }
    if let Some(parent) = report_path.parent() {
        bijux_dna_infra::ensure_dir(parent)
            .with_context(|| format!("create {}", parent.display()))?;
    }

    let outcome = test_toy_runs(
        workspace,
        &[
            "refresh".to_string(),
            "--accept".to_string(),
            "--profile".to_string(),
            "all".to_string(),
            "--out".to_string(),
            out_dir.display().to_string(),
        ],
    )?;
    if !outcome.is_success() {
        return Ok(outcome);
    }

    for entry in fs::read_dir(&out_dir).with_context(|| format!("read {}", out_dir.display()))? {
        let bundle = entry?.path();
        if !bundle.is_dir() {
            continue;
        }
        write_utf8(
            &bundle.join("GENERATE.md"),
            r"# GENERATE

## Command(s)
Generated via `cargo run -p bijux-dna-dev -- assets run refresh-golden`.

## Tool versions
- `bijux-dna-dev`, `cargo`, and `rustc` versions are recorded in `artifacts/assets-refresh/golden/report.json`.

## Input origins
- Derived from repository mini reference toy runs (`cargo run -p bijux-dna-dev -- test run toy-runs -- refresh --accept --profile all`).

## Expected outputs
- `manifest.json`
- `metrics.json`
- `artifact_checksums.json`
- `report.html`
- `CHECKSUMS.sha256`
",
        )?;
        write_checksum_manifest(
            &bundle.join("CHECKSUMS.sha256"),
            &[
                "artifact_checksums.json",
                "manifest.json",
                "metrics.json",
                "report.html",
                "GENERATE.md",
            ],
        )?;
    }

    write_refresh_report(
        &out_dir,
        &report_path,
        "golden/toy-runs-v1",
        "cargo run -p bijux-dna-dev -- assets run refresh-golden",
    )?;
    replace_dir(&out_dir, &target_dir)?;
    success_line(format!("golden refresh: wrote {}", target_dir.display()))
}

pub(super) fn assets_refresh_toy(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
    ensure_help_only("refresh-toy", args)?;
    let stage_dir = workspace.path("artifacts/assets-refresh/toy/core-v1");
    let target_dir = workspace.path("assets/toy/core-v1");
    let report_path = workspace.path("artifacts/assets-refresh/toy/report.json");

    if stage_dir.exists() {
        fs::remove_dir_all(&stage_dir)
            .with_context(|| format!("remove {}", stage_dir.display()))?;
    }
    bijux_dna_infra::ensure_dir(stage_dir.join("fastq"))
        .with_context(|| format!("create {}", stage_dir.join("fastq").display()))?;
    bijux_dna_infra::ensure_dir(stage_dir.join("bam"))
        .with_context(|| format!("create {}", stage_dir.join("bam").display()))?;
    bijux_dna_infra::ensure_dir(stage_dir.join("vcf"))
        .with_context(|| format!("create {}", stage_dir.join("vcf").display()))?;
    if let Some(parent) = report_path.parent() {
        bijux_dna_infra::ensure_dir(parent)
            .with_context(|| format!("create {}", parent.display()))?;
    }

    write_utf8(
        &stage_dir.join("fastq/reads_1.fastq"),
        "@read1/1\nACGTTGCAACGT\n+\nFFFFFFFFFFFF\n@read2/1\nTGCATGCATGCA\n+\nFFFFFFFFFFFF\n",
    )?;
    write_utf8(
        &stage_dir.join("fastq/reads_2.fastq"),
        "@read1/2\nACGTTGCAACGT\n+\nFFFFFFFFFFFF\n@read2/2\nTGCATGCATGCA\n+\nFFFFFFFFFFFF\n",
    )?;
    write_utf8(
        &stage_dir.join("bam/toy.sam"),
        "@HD\tVN:1.6\tSO:coordinate\n@SQ\tSN:chr1\tLN:1000\nread1\t0\tchr1\t1\t60\t12M\t*\t0\t0\tACGTTGCAACGT\tFFFFFFFFFFFF\nread2\t0\tchr1\t50\t60\t12M\t*\t0\t0\tTGCATGCATGCA\tFFFFFFFFFFFF\n",
    )?;
    write_utf8(
        &stage_dir.join("vcf/toy.vcf"),
        "##fileformat=VCFv4.2\n##contig=<ID=chr1,length=1000>\n#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\nchr1\t10\t.\tA\tG\t60\tPASS\t.\n",
    )?;

    write_checksum_manifest(
        &stage_dir.join("CHECKSUMS.sha256"),
        &[
            "bam/toy.sam",
            "fastq/reads_1.fastq",
            "fastq/reads_2.fastq",
            "vcf/toy.vcf",
        ],
    )?;
    write_checksum_manifest(&stage_dir.join("bam/CHECKSUMS.sha256"), &["toy.sam"])?;
    write_checksum_manifest(
        &stage_dir.join("fastq/CHECKSUMS.sha256"),
        &["reads_1.fastq", "reads_2.fastq"],
    )?;
    write_checksum_manifest(&stage_dir.join("vcf/CHECKSUMS.sha256"), &["toy.vcf"])?;

    write_utf8(
        &stage_dir.join("GENERATE.md"),
        r"# GENERATE

## Command(s)
Generated via `cargo run -p bijux-dna-dev -- assets run refresh-toy`.

## Tool versions
- `bijux-dna-dev`, `cargo`, and `rustc` versions are recorded in `artifacts/assets-refresh/toy/report.json`.

## Input origins
- Synthetic deterministic toy records authored in `bijux-dna-dev` assets control-plane commands.

## Expected outputs
- `fastq/reads_1.fastq`
- `fastq/reads_2.fastq`
- `bam/toy.sam`
- `vcf/toy.vcf`
- `CHECKSUMS.sha256`
",
    )?;

    write_refresh_report(
        &stage_dir,
        &report_path,
        "toy/core-v1",
        "cargo run -p bijux-dna-dev -- assets run refresh-toy",
    )?;
    replace_dir(&stage_dir, &target_dir)?;
    success_line(format!("toy refresh: wrote {}", target_dir.display()))
}

pub(super) fn assets_validate_reference(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
    ensure_help_only("validate-reference", args)?;
    let ref_root = workspace.path("assets/reference");
    if !ref_root.exists() {
        return Ok(OpsCommandOutcome::failure(
            "assets-reference-schema: assets/reference missing\n",
        ));
    }

    let mut errors = Vec::new();
    if !ref_root.join("SCHEMAS.md").is_file() {
        errors.push(
            "assets/reference/SCHEMAS.md missing (reference schema authority doc)".to_string(),
        );
    }

    let schema_re = Regex::new(r"(?m)^schema_version:\s*\S+")?;
    let id_re = Regex::new(r"(?m)^\s*-\s*id:\s*([A-Za-z0-9_.-]+)\s*$")?;
    let section_re = Regex::new(r"^\s*[A-Za-z0-9_]+:\s*")?;

    let mut yaml_files = WalkDir::new(&ref_root)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .map(|entry| entry.path().to_path_buf())
        .filter(|path| {
            matches!(
                path.extension().and_then(|ext| ext.to_str()),
                Some("yaml" | "yml")
            )
        })
        .collect::<Vec<_>>();
    yaml_files.sort();

    for path in &yaml_files {
        let text = read_utf8(path)?;
        let rel = workspace.rel(path).to_string_lossy().to_string();
        if !schema_re.is_match(&text) {
            errors.push(format!("{rel}: missing schema_version"));
        }

        let non_comment_keys = text
            .lines()
            .filter(|line| {
                let trimmed = line.trim();
                !trimmed.is_empty() && !trimmed.starts_with('#') && trimmed.contains(':')
            })
            .count();
        if non_comment_keys < 2 {
            errors.push(format!(
                "{rel}: expected schema_version plus at least one additional key"
            ));
        }

        let mut counts = BTreeMap::new();
        for capture in id_re.captures_iter(&text) {
            let Some(id) = capture.get(1).map(|value| value.as_str().to_string()) else {
                continue;
            };
            *counts.entry(id).or_insert(0usize) += 1;
        }
        let duplicates = counts
            .into_iter()
            .filter_map(|(id, count)| (count > 1).then_some(id))
            .collect::<Vec<_>>();
        if !duplicates.is_empty() {
            errors.push(format!("{rel}: duplicated ids: {}", duplicates.join(", ")));
        }
    }

    let mut banks = fs::read_dir(&ref_root)
        .with_context(|| format!("read {}", ref_root.display()))?
        .filter_map(std::result::Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .collect::<Vec<_>>();
    banks.sort();

    for bank_dir in banks {
        let mut bank_files = fs::read_dir(&bank_dir)
            .with_context(|| format!("read {}", bank_dir.display()))?
            .filter_map(std::result::Result::ok)
            .map(|entry| entry.path())
            .filter(|path| {
                matches!(
                    path.extension().and_then(|ext| ext.to_str()),
                    Some("yaml" | "yml")
                ) && !path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or_default()
                    .contains("presets")
            })
            .collect::<Vec<_>>();
        bank_files.sort();
        let mut preset_files = fs::read_dir(&bank_dir)
            .with_context(|| format!("read {}", bank_dir.display()))?
            .filter_map(std::result::Result::ok)
            .map(|entry| entry.path())
            .filter(|path| {
                matches!(
                    path.extension().and_then(|ext| ext.to_str()),
                    Some("yaml" | "yml")
                ) && path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or_default()
                    .contains("presets")
            })
            .collect::<Vec<_>>();
        preset_files.sort();
        if preset_files.is_empty() {
            continue;
        }

        let mut bank_ids = BTreeSet::new();
        for bank_file in bank_files {
            for capture in id_re.captures_iter(&read_utf8(&bank_file)?) {
                if let Some(id) = capture.get(1).map(|value| value.as_str().to_string()) {
                    bank_ids.insert(id);
                }
            }
        }

        for preset_file in preset_files {
            let rel = workspace.rel(&preset_file).to_string_lossy().to_string();
            let text = read_utf8(&preset_file)?;
            let mut lines = text.lines().peekable();
            while let Some(line) = lines.next() {
                let trimmed = line.trim_start();
                if !(trimmed.ends_with("_ids:") && trimmed.contains(':')) {
                    continue;
                }
                while let Some(next_line) = lines.peek().copied() {
                    let next_trimmed = next_line.trim();
                    if next_trimmed.is_empty() {
                        lines.next();
                        continue;
                    }
                    if section_re.is_match(next_line) && !next_trimmed.starts_with('-') {
                        break;
                    }
                    let candidate = next_trimmed.trim_start_matches('-').trim();
                    if !candidate.is_empty() && !bank_ids.contains(candidate) {
                        errors.push(format!(
                            "{rel}: unresolved preset reference id: {candidate}"
                        ));
                    }
                    lines.next();
                }
            }
        }
    }

    if errors.is_empty() {
        return success_line("assets-reference-schema: OK");
    }
    failure_lines("assets-reference-schema: FAILED", &errors)
}

