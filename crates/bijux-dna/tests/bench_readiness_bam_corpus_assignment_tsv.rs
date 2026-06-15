#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_readiness_bam_corpus_assignment_writes_governed_tsv_columns() {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
    let _env_guard = support::EnvGuard::new().expect("capture env");
    let _crate_root = support::crate_root("bijux-dna").expect("crate root");
    let repo_root = support::repo_root().expect("repo root");
    let home = tempfile::tempdir().expect("tempdir");

    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dna"))
        .current_dir(&repo_root)
        .env("HOME", home.path())
        .env("BIJUX_SKIP_QA", "1")
        .env("BIJUX_ALLOW_SILVER", "1")
        .env("BIJUX_SKIP_IMAGE_CHECK", "1")
        .args(["bench", "readiness", "render-bam-corpus-assignment"])
        .output()
        .expect("run cli");

    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let tsv_path = repo_root.join("benchmarks/readiness/bam-corpus-assignment.tsv");
    assert!(tsv_path.is_file(), "BAM corpus assignment TSV must exist");

    let tsv = std::fs::read_to_string(&tsv_path).expect("read BAM corpus assignment");
    let mut lines = tsv.lines();
    assert_eq!(
        lines.next(),
        Some(
            "tool_id\tstage_id\tbenchmark_status\tsupport_status\tadapter_status\tparser_status\tcorpus_family_id\tfixture_id\tsample_id\tinput_contract\tbenchmark_limits\trequired_assets\texpected_outputs\tskip_behavior\treason"
        )
    );
    let rows = lines.map(|line| line.split('\t').collect::<Vec<_>>()).collect::<Vec<_>>();
    assert_eq!(rows.len(), 49, "TSV must retain the governed BAM row count");
    assert!(rows.iter().any(|row| {
        row[0] == "authenticct"
            && row[1] == "bam.authenticity"
            && row[2] == "benchmark_ready"
            && row[3] == "supported"
            && row[4] == "runnable"
            && row[5] == "parser_fixture_validated"
            && row[6] == "corpus-01-adna-bam"
            && row[7] == "corpus-01-adna-damage-mini"
            && row[8] == "adna_damage_non_udg"
            && row[9]
                == "ct5p=0.18;ga3p=0.11;short_frag=1;signal=moderate;strict_profile_upgraded=false;terminal=ct5p_dominant;udg=non_udg"
            && row[10] == "complexity_min_reads=3;coverage_depth_thresholds=1,5,10"
            && row[11]
                == "expected_damage=expected_damage.json;reference_fasta=adna_damage_reference.fasta"
            && row[12] == "authenticity_report,summary,stage_metrics"
            && row[13] == "not_applicable"
    }));
    assert!(rows.iter().any(|row| {
        row[0] == "mapdamage2"
            && row[1] == "bam.damage"
            && row[6] == "corpus-01-adna-bam"
            && row[7] == "corpus-01-adna-damage-mini"
            && row[8] == "adna_damage_non_udg"
            && row[9]
                == "ct5p=0.18;ga3p=0.11;short_frag=1;signal=moderate;strict_profile_upgraded=false;terminal=ct5p_dominant;udg=non_udg"
            && row[10] == "not_applicable"
            && row[11]
                == "expected_damage=expected_damage.json;reference_fasta=adna_damage_reference.fasta"
            && row[12] == "damage_report,terminal_position_metrics,stage_metrics"
            && row[13] == "not_applicable"
    }));
    assert!(rows.iter().any(|row| {
        row[0] == "verifybamid2"
            && row[1] == "bam.contamination"
            && row[6] == "corpus-01-adna-bam"
            && row[7] == "corpus-01-adna-bam-mini"
            && row[8] == "adna_contamination_panel_screen"
            && row[9] == "signal=moderate;terminal=ct5p_dominant;udg=non_udg"
            && row[10] == "minimum_mean_coverage=0.5"
            && row[11]
                == "reference_fasta=adna_bam_reference;reference_panel=adna_contamination_panel"
            && row[12] == "contamination_report,summary,stage_metrics"
            && row[13] == "not_applicable"
    }));
    assert!(rows.iter().any(|row| {
        row[0] == "rxy"
            && row[1] == "bam.sex"
            && row[6] == "corpus-01-adna-bam"
            && row[7] == "corpus-01-adna-bam-mini"
            && row[8] == "adna_xy_autosome_coverage"
            && row[9] == "signal=moderate;terminal=ct5p_dominant;udg=non_udg"
            && row[10]
                == "expected_autosomal_coverage=1;expected_x_coverage=0.5;expected_y_coverage=0.5;minimum_y_sites=5"
            && row[11] == "reference_fasta=adna_bam_reference"
            && row[12] == "sex_report,summary,stage_metrics"
            && row[13] == "not_applicable"
    }));
    assert!(rows.iter().any(|row| {
        row[0] == "yleaf"
            && row[1] == "bam.haplogroups"
            && row[6] == "corpus-01-adna-bam"
            && row[7] == "corpus-01-adna-bam-mini"
            && row[8] == "adna_y_haplogroup_panel"
            && row[9] == "signal=moderate;terminal=ct5p_dominant;udg=non_udg"
            && row[10] == "min_coverage=2"
            && row[11] == "reference_fasta=adna_bam_reference;reference_panel=adna-y-hg38-mini"
            && row[12] == "haplogroups,summary,stage_metrics"
            && row[13] == "not_applicable"
    }));
    assert!(rows.iter().any(|row| {
        row[0] == "angsd"
            && row[1] == "bam.genotyping"
            && row[2] == "benchmark_ready"
            && row[3] == "supported"
            && row[4] == "runnable"
            && row[5] == "parser_fixture_validated"
            && row[6] == "corpus-01-genotyping"
            && row[7] == "corpus-01-genotyping-mini"
            && row[8] == "human_like_genotyping_candidate_panel"
            && row[9]
                == "reference=corpus_01_bam_reference;regions=human_like_genotyping_target_regions;sites=human_like_genotyping_candidate_sites"
            && row[10] == "min_call_rate=0.5;min_posterior=0.9"
            && row[11]
                == "reference_fasta=corpus_01_bam_reference;regions=human_like_genotyping_target_regions;sites_vcf=human_like_genotyping_candidate_sites"
            && row[12]
                == "genotyping_bcf,genotyping_vcf,genotyping_vcf_tbi,genotyping_gl,summary,stage_metrics"
            && row[13] == "not_applicable"
    }));
    assert!(rows.iter().any(|row| {
        row[0] == "king"
            && row[1] == "bam.kinship"
            && row[2] == "benchmark_ready"
            && row[3] == "supported"
            && row[4] == "runnable"
            && row[5] == "parser_fixture_validated"
            && row[6] == "corpus-01-kinship"
            && row[7] == "corpus-01-kinship-mini"
            && row[8] == "human_like_kinship_low_overlap_pair,human_like_kinship_related_pair"
            && row[9]
                == "population_scope=human_diploid_panel;reference=corpus_01_bam_reference;reference_build=grch38;reference_panel=human_like_relatedness_panel"
            && row[10]
                == "human_like_kinship_low_overlap_pair.min_overlap_snps=5;human_like_kinship_low_overlap_pair.observed_max_overlap_snps=4;human_like_kinship_related_pair.min_overlap_snps=6;human_like_kinship_related_pair.observed_max_overlap_snps=6"
            && row[11]
                == "reference_fasta=corpus_01_bam_reference;reference_panel=human_like_relatedness_panel"
            && row[12] == "kinship_report,summary,kinship_segments,stage_metrics"
            && row[13]
                == "human_like_kinship_low_overlap_pair=stop_without_pairwise_results;human_like_kinship_related_pair=emit_pairwise_results"
    }));
    assert!(rows.iter().any(|row| {
        row[0] == "angsd"
            && row[1] == "bam.kinship"
            && row[6] == "corpus-01-kinship"
            && row[7] == "corpus-01-kinship-mini"
            && row[8] == "human_like_kinship_low_overlap_pair,human_like_kinship_related_pair"
            && row[12] == "kinship_report,summary,kinship_segments,stage_metrics"
            && row[13]
                == "human_like_kinship_low_overlap_pair=stop_without_pairwise_results;human_like_kinship_related_pair=emit_pairwise_results"
    }));
    assert!(rows.iter().any(|row| {
        row[0] == "samtools"
            && row[1] == "bam.qc_pre"
            && row[2] == "benchmark_ready"
            && row[3] == "supported"
            && row[4] == "runnable"
            && row[5] == "parser_fixture_validated"
            && row[6] == "corpus-01-bam"
            && row[7] == "corpus-01-bam-mini"
            && row[8] == "not_applicable"
            && row[9] == "not_applicable"
            && row[10] == "not_applicable"
            && row[11] == "not_applicable"
            && row[12] == "not_applicable"
            && row[13] == "not_applicable"
    }));
    assert!(rows.iter().any(|row| {
        row[0] == "bwa"
            && row[1] == "bam.align"
            && row[2] == "benchmark_ready"
            && row[3] == "supported"
            && row[4] == "runnable"
            && row[5] == "parser_fixture_validated"
            && row[6] == "corpus-01"
            && row[7] == "corpus-01-mini"
            && row[8] == "not_applicable"
            && row[9] == "not_applicable"
            && row[10] == "not_applicable"
            && row[11] == "not_applicable"
            && row[12] == "not_applicable"
            && row[13] == "not_applicable"
    }));
}
