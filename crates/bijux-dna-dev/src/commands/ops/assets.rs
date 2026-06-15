use super::{
    ensure_help_only, failure_lines, fs, read_utf8, replace_dir, success_line, test_toy_runs,
    write_checksum_manifest, write_refresh_report, write_utf8, BTreeMap, BTreeSet, Context,
    OpsCommandOutcome, Regex, Result, WalkDir, Workspace,
};

use super::set_assets_readonly;

const VALIDATION_PASS_BAM_BYTES: &[u8] = &[
    31, 139, 8, 4, 0, 0, 0, 0, 0, 255, 6, 0, 66, 67, 2, 0, 235, 0, 141, 80, 177, 110, 194, 48, 16,
    13, 3, 131, 243, 5, 140, 30, 97, 112, 140, 169, 232, 224, 41, 45, 72, 45, 18, 4, 104, 68, 87,
    116, 9, 46, 53, 34, 49, 242, 153, 0, 127, 143, 3, 8, 6, 22, 134, 123, 122, 186, 187, 247, 244,
    238, 62, 63, 38, 141, 86, 35, 8, 226, 239, 33, 249, 77, 164, 136, 222, 73, 58, 149, 185, 49,
    118, 165, 75, 112, 42, 140, 211, 57, 73, 19, 153, 255, 91, 65, 198, 137, 236, 119, 195, 248,
    231, 139, 140, 134, 210, 174, 5, 73, 39, 126, 213, 42, 86, 9, 182, 3, 196, 48, 158, 93, 70, 8,
    133, 51, 102, 139, 100, 150, 60, 248, 197, 189, 247, 22, 9, 218, 222, 157, 124, 183, 67, 6,
    227, 251, 148, 86, 90, 29, 40, 203, 40, 51, 148, 47, 80, 89, 228, 153, 222, 64, 89, 227, 254,
    120, 69, 182, 86, 165, 41, 116, 142, 28, 172, 211, 127, 144, 59, 228, 21, 108, 245, 10, 156,
    54, 229, 178, 14, 16, 101, 80, 188, 162, 71, 84, 94, 236, 204, 137, 223, 226, 115, 47, 124, 50,
    243, 225, 66, 255, 154, 160, 233, 171, 190, 63, 232, 121, 114, 6, 166, 14, 77, 99, 49, 1, 0, 0,
    31, 139, 8, 4, 0, 0, 0, 0, 0, 255, 6, 0, 66, 67, 2, 0, 79, 0, 179, 100, 64, 0, 86, 27, 79, 33,
    70, 134, 100, 6, 54, 40, 31, 70, 23, 25, 24, 24, 50, 36, 0, 25, 66, 30, 66, 170, 96, 16, 228,
    30, 85, 148, 110, 200, 96, 137, 164, 16, 162, 121, 50, 92, 19, 3, 186, 230, 14, 65, 13, 20,
    205, 0, 47, 32, 160, 28, 122, 0, 0, 0, 31, 139, 8, 4, 0, 0, 0, 0, 0, 255, 6, 0, 66, 67, 2, 0,
    27, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

const VALIDATION_PASS_BAI_BYTES: &[u8] = &[
    66, 65, 73, 1, 1, 0, 0, 0, 2, 0, 0, 0, 73, 18, 0, 0, 1, 0, 0, 0, 0, 0, 236, 0, 0, 0, 0, 0, 0,
    0, 60, 1, 0, 0, 0, 0, 74, 146, 0, 0, 2, 0, 0, 0, 0, 0, 236, 0, 0, 0, 0, 0, 0, 0, 60, 1, 0, 0,
    0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 236, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0,
];

fn with_assets_writable<F>(workspace: &Workspace, action: F) -> Result<OpsCommandOutcome>
where
    F: FnOnce() -> Result<OpsCommandOutcome>,
{
    set_assets_readonly(workspace, false)?;
    let result = action();
    let restore = set_assets_readonly(workspace, true);
    match (result, restore) {
        (Ok(outcome), Ok(())) => Ok(outcome),
        (Err(error), Ok(())) => Err(error),
        (Ok(_), Err(error)) => Err(error),
        (Err(error), Err(restore_error)) => Err(error.context(format!(
            "also failed to restore assets read-only permissions: {restore_error}"
        ))),
    }
}

pub(super) fn assets_refresh_golden(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    ensure_help_only("refresh-golden", args)?;
    with_assets_writable(workspace, || {
        let out_dir = workspace.path("artifacts/assets-refresh/golden/toy-runs-v1");
        let target_dir = workspace.path("assets/golden/toy-runs-v1");
        let report_path = workspace.path("artifacts/assets-refresh/golden/report.json");

        if out_dir.exists() {
            fs::remove_dir_all(&out_dir)
                .with_context(|| format!("remove {}", out_dir.display()))?;
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

        for entry in
            fs::read_dir(&out_dir).with_context(|| format!("read {}", out_dir.display()))?
        {
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
    })
}

pub(super) fn assets_refresh_reference(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    ensure_help_only("refresh-reference", args)?;
    with_assets_writable(workspace, || {
        write_reference_asset_docs(workspace)?;
        success_line("reference refresh: wrote assets/reference docs")
    })
}

pub(super) fn assets_refresh_toy(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    ensure_help_only("refresh-toy", args)?;
    with_assets_writable(workspace, || {
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
        bijux_dna_infra::ensure_dir(stage_dir.join("tables"))
            .with_context(|| format!("create {}", stage_dir.join("tables").display()))?;
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
            &stage_dir.join("bam/qc_pre_core_metrics.sam"),
            "@HD\tVN:1.6\tSO:coordinate\n@SQ\tSN:chr1\tLN:100\n@SQ\tSN:chr2\tLN:80\n@RG\tID:rg1\tSM:core-v1-qc-pre\nr001\t0\tchr1\t5\t60\t8M\t*\t0\t0\tACGTACGT\tFFFFFFFF\tRG:Z:rg1\nr002\t1024\tchr1\t20\t25\t8M\t*\t0\t0\tTGCATGCA\tFFFFFFFF\tRG:Z:rg1\nr003\t0\tchr2\t10\t10\t8M\t*\t0\t0\tCCGGAATT\tFFFFFFFF\tRG:Z:rg1\n",
        )?;
        write_utf8(
            &stage_dir.join("bam/mapping_summary_partial_mapping.sam"),
            "@HD\tVN:1.6\tSO:coordinate\n@SQ\tSN:chr1\tLN:50\n@RG\tID:rg1\tSM:core-v1-mapping-summary\nr001\t0\tchr1\t1\t45\t6M\t*\t0\t0\tACGTAC\tFFFFFF\tRG:Z:rg1\nr002\t0\tchr1\t10\t20\t6M\t*\t0\t0\tTGCATG\tFFFFFF\tRG:Z:rg1\nr003\t4\t*\t0\t0\t*\t*\t0\t0\tNNNNNN\tFFFFFF\tRG:Z:rg1\n",
        )?;
        write_utf8(
            &stage_dir.join("bam/filter_mixed_constraints.sam"),
            "@HD\tVN:1.6\tSO:coordinate\n@SQ\tSN:chr1\tLN:100\n@RG\tID:rg1\tSM:core-v1-filter\ngood001\t0\tchr1\t1\t60\t8M\t*\t0\t0\tACGTACGT\tFFFFFFFF\tRG:Z:rg1\nlowq001\t0\tchr1\t10\t10\t8M\t*\t0\t0\tTGCATGCA\tFFFFFFFF\tRG:Z:rg1\nshort001\t0\tchr1\t20\t60\t6M\t*\t0\t0\tGATTAC\tFFFFFF\tRG:Z:rg1\ndup001\t1024\tchr1\t30\t60\t8M\t*\t0\t0\tCCCCGGGG\tFFFFFFFF\tRG:Z:rg1\nunmap001\t4\t*\t0\t0\t*\t*\t0\t0\tNNNNNNNN\tFFFFFFFF\tRG:Z:rg1\n",
        )?;
        write_utf8(
            &stage_dir.join("bam/mapq_threshold_ladder.sam"),
            "@HD\tVN:1.6\tSO:coordinate\n@SQ\tSN:chr1\tLN:100\n@RG\tID:rg1\tSM:core-v1-mapq-filter\nmapq60\t0\tchr1\t1\t60\t8M\t*\t0\t0\tACGTACGT\tFFFFFFFF\tRG:Z:rg1\nmapq30\t0\tchr1\t12\t30\t8M\t*\t0\t0\tTGCATGCA\tFFFFFFFF\tRG:Z:rg1\nmapq10\t0\tchr1\t24\t10\t8M\t*\t0\t0\tGGGGTTTT\tFFFFFFFF\tRG:Z:rg1\nunmapped\t4\t*\t0\t0\t*\t*\t0\t0\tNNNNNNNN\tFFFFFFFF\tRG:Z:rg1\n",
        )?;
        write_utf8(
            &stage_dir.join("bam/length_threshold_ladder.sam"),
            "@HD\tVN:1.6\tSO:coordinate\n@SQ\tSN:chr1\tLN:100\n@RG\tID:rg1\tSM:core-v1-length-filter\nlen12\t0\tchr1\t1\t60\t12M\t*\t0\t0\tACGTACGTACGT\tFFFFFFFFFFFF\tRG:Z:rg1\nlen8\t0\tchr1\t20\t60\t8M\t*\t0\t0\tTGCATGCA\tFFFFFFFF\tRG:Z:rg1\nlen5\t0\tchr1\t35\t60\t5M\t*\t0\t0\tGATTA\tFFFFF\tRG:Z:rg1\nunmapped10\t4\t*\t0\t0\t*\t*\t0\t0\tNNNNNNNNNN\tFFFFFFFFFF\tRG:Z:rg1\n",
        )?;
        write_utf8(
            &stage_dir.join("bam/validation_reference.fasta"),
            ">chr1\nACGTACGTACGTACGTACGT\n",
        )?;
        fs::write(stage_dir.join("bam/validation_pass.bam"), VALIDATION_PASS_BAM_BYTES)?;
        fs::write(stage_dir.join("bam/validation_pass.bam.bai"), VALIDATION_PASS_BAI_BYTES)?;
        fs::write(stage_dir.join("bam/validation_malformed.bam"), b"not-bam\n")?;
        write_utf8(
            &stage_dir.join("vcf/toy.vcf"),
            "##fileformat=VCFv4.2\n##contig=<ID=chr1,length=1000>\n#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\nchr1\t10\t.\tA\tG\t60\tPASS\t.\n",
        )?;
        write_utf8(
            &stage_dir.join("tables/otu_abundance_small.tsv"),
            "sample_id\tfeature_id\tabundance\nsample_a\totu_001\t10\nsample_a\totu_002\t30\nsample_b\totu_001\t5\nsample_b\totu_003\t5\n",
        )?;

        write_checksum_manifest(
            &stage_dir.join("CHECKSUMS.sha256"),
            &[
                "bam/toy.sam",
                "bam/qc_pre_core_metrics.sam",
                "bam/mapping_summary_partial_mapping.sam",
                "bam/filter_mixed_constraints.sam",
                "bam/mapq_threshold_ladder.sam",
                "bam/length_threshold_ladder.sam",
                "bam/validation_malformed.bam",
                "bam/validation_pass.bam",
                "bam/validation_pass.bam.bai",
                "bam/validation_reference.fasta",
                "fastq/reads_1.fastq",
                "fastq/reads_2.fastq",
                "tables/otu_abundance_small.tsv",
                "vcf/toy.vcf",
            ],
        )?;
        write_checksum_manifest(
            &stage_dir.join("bam/CHECKSUMS.sha256"),
            &[
                "toy.sam",
                "qc_pre_core_metrics.sam",
                "mapping_summary_partial_mapping.sam",
                "filter_mixed_constraints.sam",
                "mapq_threshold_ladder.sam",
                "length_threshold_ladder.sam",
                "validation_malformed.bam",
                "validation_pass.bam",
                "validation_pass.bam.bai",
                "validation_reference.fasta",
            ],
        )?;
        write_checksum_manifest(
            &stage_dir.join("fastq/CHECKSUMS.sha256"),
            &["reads_1.fastq", "reads_2.fastq"],
        )?;
        write_checksum_manifest(
            &stage_dir.join("tables/CHECKSUMS.sha256"),
            &["otu_abundance_small.tsv"],
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
- `bam/qc_pre_core_metrics.sam`
- `bam/mapping_summary_partial_mapping.sam`
- `bam/filter_mixed_constraints.sam`
- `bam/mapq_threshold_ladder.sam`
- `bam/length_threshold_ladder.sam`
- `bam/validation_reference.fasta`
- `bam/validation_pass.bam`
- `bam/validation_pass.bam.bai`
- `bam/validation_malformed.bam`
- `tables/otu_abundance_small.tsv`
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
    })
}

fn write_reference_asset_docs(workspace: &Workspace) -> Result<()> {
    let ref_root = workspace.path("assets/reference");
    write_utf8(&ref_root.join("EVIDENCE.md"), REFERENCE_EVIDENCE_MD)?;
    write_utf8(&ref_root.join("LOCK.md"), REFERENCE_LOCK_MD)?;
    write_utf8(&ref_root.join("index.md"), REFERENCE_INDEX_MD)?;
    Ok(())
}

const REFERENCE_EVIDENCE_MD: &str = r"# Reference Asset Evidence

## What
Evidence and provenance notes for governed FASTQ reference assets under `assets/reference/`.

## Why
Reference assets affect scientific interpretation as much as tool choice. Adapter banks, primer banks, contaminant references, and QC thresholds must be explicit about whether they are production references, sentinel records, or synthetic test motifs.

## Asset Status
| Asset group | Current files | Evidence status | Review rule |
| --- | --- | --- | --- |
| Adapter bank | `adapters/bank.v1.yaml`, `adapters/presets.v1.yaml` | Mixed vendor-derived motifs and synthetic fallback motifs | Treat vendor-derived motifs as supported only when a matching source note is present; treat synthetic motifs as test coverage, not kit truth. |
| Primer bank | `primers/*.fasta` | Literature-derived marker primers | Keep marker, sequence, and citation together when adding or changing primers. |
| Contaminant motifs | `contaminants/contaminant_motifs.v1.yaml` | Small deterministic motif set | Use for deterministic detection tests and policy wiring, not as a complete contamination database. |
| Contaminant references | `contaminants/references/phix174.fasta`, `contaminants/references/univec.fasta` | Sentinel FASTA records in the current repository | Do not describe these as complete PhiX174 or UniVec references until replaced by pinned upstream snapshots. |
| PolyX bank | `polyx/bank.v1.yaml`, `polyx/presets.v1.yaml` | Deterministic sequence-tail policy assets | Use as policy inputs for trimming behavior, not as external biological references. |
| QC thresholds | `qc_thresholds.yaml` | Governed thresholds | Interpret only with the matching FASTQ stage assumptions and profile defaults. |

## Primer References
| Primer set | Marker | Current sequences | Primary evidence |
| --- | --- | --- | --- |
| `COI_folmer_v1` | mitochondrial COI barcode | LCO1490, HCO2198 | Folmer et al. 1994, published COI primer pair for invertebrate mitochondrial cytochrome c oxidase I. |
| `16S_universal_v1` | bacterial 16S rRNA | 27F, 1492R | Weisburg et al. 1991, broad bacterial 16S amplification primers. |
| `ITS2_plant_v1` | plant ITS2 | S2F, S3R | Chen et al. 2010, ITS2 plant DNA barcode primer set. |

## Sentinel Contaminant References
The current PhiX174 and UniVec FASTA payloads are deliberately tiny records:

- `phix174.fasta` is 48 bytes in the current checkout.
- `univec.fasta` is 43 bytes in the current checkout.

Those sizes are incompatible with complete upstream PhiX174 or UniVec reference payloads. They are suitable for path, checksum, parser, and policy tests only. Production contaminant depletion or screening requires pinned upstream replacement payloads, checksum review, and an updated lock note.

## Update Requirements
- Stage candidate updates under `artifacts/assets-refresh/reference/`.
- Record upstream URL, retrieval date, checksum, sequence count, and total bases.
- Diff sequence headers and lengths before replacing a tracked reference.
- Recompute affected `CHECKSUMS.sha256` files in the same change.
- Add uncertain source or missing production payload questions to the workspace-level `NEEDED.md` handoff.
";

const REFERENCE_LOCK_MD: &str = r"# Reference Source Lock

## Purpose
Define pinned upstream sources for reference assets and a safe update workflow.

## Pinned Sources
- `assets/reference/contaminants/references/univec.fasta`
  - upstream: NCBI UniVec snapshot
  - current tracked payload: sentinel record, not a complete UniVec snapshot
  - update method: explicit download to staging + checksum review
- `assets/reference/contaminants/references/phix174.fasta`
  - upstream: PhiX174 reference sequence snapshot
  - current tracked payload: sentinel record, not a complete PhiX174 snapshot
  - update method: explicit download to staging + checksum review

## Update Workflow
1. Stage candidate updates under `artifacts/assets-refresh/reference/`.
2. Diff old/new sequence headers and lengths.
3. Recompute package checksums.
4. Update provenance notes and commit with rationale.

## Safety Diff Rules
- Always diff by checksums and sequence statistics.
- Do not replace references silently.
- Any sequence-content change requires review notes in commit message.
";

const REFERENCE_INDEX_MD: &str = r"# Reference Assets

## What
Production reference data, banks, and presets that are not toy or golden fixtures.

## Rules
- Keep only deterministic data artifacts.
- Domain crates should reference these paths via stable relative paths.
- Source update and pin policy is defined in `assets/reference/LOCK.md`.
- Evidence and sentinel-vs-production status are defined in `assets/reference/EVIDENCE.md`.

---
Asset Provenance Footer
Last regenerated: 2026-02-13
Regenerate command: `cargo run -p bijux-dna-dev -- assets run refresh-reference`
";

pub(super) fn assets_validate_reference(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
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
    if !ref_root.join("EVIDENCE.md").is_file() {
        errors.push(
            "assets/reference/EVIDENCE.md missing (reference evidence authority doc)".to_string(),
        );
    }
    if !ref_root.join("LOCK.md").is_file() {
        errors.push("assets/reference/LOCK.md missing (reference source lock doc)".to_string());
    }
    if !ref_root.join("index.md").is_file() {
        errors.push("assets/reference/index.md missing (reference assets index doc)".to_string());
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
            matches!(path.extension().and_then(|ext| ext.to_str()), Some("yaml" | "yml"))
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
            errors.push(format!("{rel}: expected schema_version plus at least one additional key"));
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
                matches!(path.extension().and_then(|ext| ext.to_str()), Some("yaml" | "yml"))
                    && !path
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
                matches!(path.extension().and_then(|ext| ext.to_str()), Some("yaml" | "yml"))
                    && path
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
                        errors.push(format!("{rel}: unresolved preset reference id: {candidate}"));
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
