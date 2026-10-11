use crate::bam::assignment::declarations::NOT_APPLICABLE;
use crate::bam::assignment::models::BamCorpusAssignmentRow;
use anyhow::anyhow;
use anyhow::Result;
use bijux_dna_core::id_catalog::BAM_AUTHENTICITY;
use bijux_dna_core::id_catalog::BAM_CONTAMINATION;
use bijux_dna_core::id_catalog::BAM_DAMAGE;
use bijux_dna_core::id_catalog::BAM_GENOTYPING;
use bijux_dna_core::id_catalog::BAM_HAPLOGROUPS;
use bijux_dna_core::id_catalog::BAM_KINSHIP;
use bijux_dna_core::id_catalog::BAM_SEX;
pub(in crate::bam::assignment) fn ensure_bam_fixture_assignment_coverage(
    rows: &[BamCorpusAssignmentRow],
) -> Result<()> {
    let expected_rows = [
        BamFixtureAssignmentExpectation {
            stage_id: BAM_AUTHENTICITY,
            fixture_id: "corpus-01-adna-damage-mini",
            sample_id: "adna_damage_non_udg",
            input_contract: "ct5p=0.18;ga3p=0.11;short_frag=1;signal=moderate;strict_profile_upgraded=false;terminal=ct5p_dominant;udg=non_udg",
            benchmark_limits: "complexity_min_reads=3;coverage_depth_thresholds=1,5,10",
            required_assets: "expected_damage=expected_damage.json;reference_fasta=adna_damage_reference.fasta",
            expected_outputs: "authenticity_report,summary,stage_metrics",
            skip_behavior: NOT_APPLICABLE,
        },
        BamFixtureAssignmentExpectation {
            stage_id: BAM_DAMAGE,
            fixture_id: "corpus-01-adna-damage-mini",
            sample_id: "adna_damage_non_udg",
            input_contract: "ct5p=0.18;ga3p=0.11;short_frag=1;signal=moderate;strict_profile_upgraded=false;terminal=ct5p_dominant;udg=non_udg",
            benchmark_limits: NOT_APPLICABLE,
            required_assets: "expected_damage=expected_damage.json;reference_fasta=adna_damage_reference.fasta",
            expected_outputs: "damage_report,terminal_position_metrics,stage_metrics",
            skip_behavior: NOT_APPLICABLE,
        },
        BamFixtureAssignmentExpectation {
            stage_id: BAM_CONTAMINATION,
            fixture_id: "corpus-01-adna-bam-mini",
            sample_id: "adna_contamination_panel_screen",
            input_contract: "signal=moderate;terminal=ct5p_dominant;udg=non_udg",
            benchmark_limits: "minimum_mean_coverage=0.5",
            required_assets: "reference_fasta=adna_bam_reference;reference_panel=adna_contamination_panel",
            expected_outputs: "contamination_report,summary,stage_metrics",
            skip_behavior: NOT_APPLICABLE,
        },
        BamFixtureAssignmentExpectation {
            stage_id: BAM_SEX,
            fixture_id: "corpus-01-adna-bam-mini",
            sample_id: "adna_xy_autosome_coverage",
            input_contract: "signal=moderate;terminal=ct5p_dominant;udg=non_udg",
            benchmark_limits: "expected_autosomal_coverage=1;expected_x_coverage=0.5;expected_y_coverage=0.5;minimum_y_sites=5",
            required_assets: "reference_fasta=adna_bam_reference",
            expected_outputs: "sex_report,summary,stage_metrics",
            skip_behavior: NOT_APPLICABLE,
        },
        BamFixtureAssignmentExpectation {
            stage_id: BAM_HAPLOGROUPS,
            fixture_id: "corpus-01-adna-bam-mini",
            sample_id: "adna_y_haplogroup_panel",
            input_contract: "signal=moderate;terminal=ct5p_dominant;udg=non_udg",
            benchmark_limits: "min_coverage=2",
            required_assets: "reference_fasta=adna_bam_reference;reference_panel=adna-y-hg38-mini",
            expected_outputs: "haplogroups,summary,stage_metrics",
            skip_behavior: NOT_APPLICABLE,
        },
        BamFixtureAssignmentExpectation {
            stage_id: BAM_GENOTYPING,
            fixture_id: "corpus-01-genotyping-mini",
            sample_id: "human_like_genotyping_candidate_panel",
            input_contract: "reference=corpus_01_bam_reference;regions=human_like_genotyping_target_regions;sites=human_like_genotyping_candidate_sites",
            benchmark_limits: "min_call_rate=0.5;min_posterior=0.9",
            required_assets: "reference_fasta=corpus_01_bam_reference;regions=human_like_genotyping_target_regions;sites_vcf=human_like_genotyping_candidate_sites",
            expected_outputs: "genotyping_bcf,genotyping_vcf,genotyping_vcf_tbi,genotyping_gl,summary,stage_metrics",
            skip_behavior: NOT_APPLICABLE,
        },
        BamFixtureAssignmentExpectation {
            stage_id: BAM_KINSHIP,
            fixture_id: "corpus-01-kinship-mini",
            sample_id: "human_like_kinship_low_overlap_pair,human_like_kinship_related_pair",
            input_contract: "population_scope=human_diploid_panel;reference=corpus_01_bam_reference;reference_build=grch38;reference_panel=human_like_relatedness_panel",
            benchmark_limits: "human_like_kinship_low_overlap_pair.min_overlap_snps=5;human_like_kinship_low_overlap_pair.observed_max_overlap_snps=4;human_like_kinship_related_pair.min_overlap_snps=6;human_like_kinship_related_pair.observed_max_overlap_snps=6",
            required_assets: "reference_fasta=corpus_01_bam_reference;reference_panel=human_like_relatedness_panel",
            expected_outputs: "kinship_report,summary,kinship_segments,stage_metrics",
            skip_behavior: "human_like_kinship_low_overlap_pair=stop_without_pairwise_results;human_like_kinship_related_pair=emit_pairwise_results",
        },
    ];

    for expectation in expected_rows {
        let stage_rows =
            rows.iter().filter(|row| row.stage_id == expectation.stage_id).collect::<Vec<_>>();
        if stage_rows.is_empty() {
            return Err(anyhow!(
                "BAM corpus assignment coverage is missing governed rows for stage `{}`",
                expectation.stage_id
            ));
        }
        for row in stage_rows {
            if row.fixture_id != expectation.fixture_id
                || row.sample_id != expectation.sample_id
                || row.input_contract != expectation.input_contract
                || row.benchmark_limits != expectation.benchmark_limits
                || row.required_assets != expectation.required_assets
                || row.expected_outputs != expectation.expected_outputs
                || row.skip_behavior != expectation.skip_behavior
            {
                return Err(anyhow!(
                    "BAM corpus assignment row `{}` / `{}` drifted away from the governed stage evidence contract",
                    row.stage_id,
                    row.tool_id
                ));
            }
        }
    }

    Ok(())
}
pub(in crate::bam::assignment) struct BamFixtureAssignmentExpectation {
    pub(in crate::bam::assignment) stage_id: &'static str,
    pub(in crate::bam::assignment) fixture_id: &'static str,
    pub(in crate::bam::assignment) sample_id: &'static str,
    pub(in crate::bam::assignment) input_contract: &'static str,
    pub(in crate::bam::assignment) benchmark_limits: &'static str,
    pub(in crate::bam::assignment) required_assets: &'static str,
    pub(in crate::bam::assignment) expected_outputs: &'static str,
    pub(in crate::bam::assignment) skip_behavior: &'static str,
}
