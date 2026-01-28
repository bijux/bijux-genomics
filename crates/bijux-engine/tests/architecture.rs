use std::fs;
use std::path::{Path, PathBuf};

fn collect_rs_files(root: &Path, files: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_rs_files(&path, files);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            files.push(path);
        }
    }
}

fn assert_no_imports(dir: &str, forbidden: &[&str]) {
    let mut files = Vec::new();
    collect_rs_files(Path::new(dir), &mut files);
    for file in files {
        let Ok(contents) = fs::read_to_string(&file) else {
            continue;
        };
        for needle in forbidden {
            assert!(
                !contents.contains(needle),
                "forbidden import in {}: {}",
                file.display(),
                needle
            );
        }
    }
}

fn assert_no_fastq_terms(dir: &str) {
    let mut files = Vec::new();
    collect_rs_files(Path::new(dir), &mut files);
    for file in files {
        let Ok(contents) = fs::read_to_string(&file) else {
            continue;
        };
        let has_stage_id = contents.contains("\"fastq.") || contents.contains("'fastq.");
        assert!(
            !has_stage_id,
            "fastq term in engine core: {}",
            file.display()
        );
    }
}

fn assert_no_tool_names(dir: &str, tool_ids: &[&str]) {
    let mut files = Vec::new();
    collect_rs_files(Path::new(dir), &mut files);
    for file in files {
        if file.file_name().and_then(|name| name.to_str()) == Some("tools.rs") {
            continue;
        }
        let Ok(contents) = fs::read_to_string(&file) else {
            continue;
        };
        for tool in tool_ids {
            assert!(
                !contents.contains(tool),
                "tool id leaked into engine core: {} -> {}",
                file.display(),
                tool
            );
        }
    }
}

#[test]
fn executor_does_not_import_composer_observer_validator() {
    assert_no_imports(
        "crates/bijux-engine/src/services/executor",
        &[
            "crate::core::composer::",
            "crate::services::observer::",
            "crate::core::validator::",
        ],
    );
}

#[test]
fn observer_does_not_import_executor() {
    assert_no_imports(
        "crates/bijux-engine/src/services/observer",
        &["crate::services::executor::"],
    );
}

#[test]
fn validator_does_not_import_executor() {
    assert_no_imports(
        "crates/bijux-engine/src/core/validator",
        &["crate::services::executor::"],
    );
}

#[test]
fn engine_core_is_fastq_agnostic() {
    assert_no_fastq_terms("crates/bijux-engine/src/core/composer");
    assert_no_fastq_terms("crates/bijux-engine/src/services/executor");
    assert_no_fastq_terms("crates/bijux-engine/src/services/observer");
    assert_no_fastq_terms("crates/bijux-engine/src/core/validator");
    assert_no_fastq_terms("crates/bijux-engine/src/core/types");
    assert_no_fastq_terms("crates/bijux-engine/src/core/errors");
}

#[test]
fn engine_does_not_import_domain_crates() {
    assert_no_imports(
        "crates/bijux-engine/src",
        &["bijux_domain", "bijux-domain", "domain::"],
    );
}

#[test]
fn engine_core_does_not_embed_tool_ids() {
    let tool_ids = [
        "adapterremoval",
        "atropos",
        "bbduk",
        "bbmerge",
        "centrifuge",
        "cutadapt",
        "fastp",
        "fastqc",
        "fastq_screen",
        "fastqvalidator",
        "fastqvalidator_official",
        "flash2",
        "fqtools",
        "kaiju",
        "kraken2",
        "metaphlan",
        "multiqc",
        "pear",
        "prinseq",
        "qualimap",
        "rcorrector",
        "samtools",
        "seqkit",
        "seqkit_stats",
        "seqpurge",
        "spades",
        "trimmomatic",
        "umi_tools",
        "vsearch",
    ];
    assert_no_tool_names("crates/bijux-engine/src/core", &tool_ids);
}
