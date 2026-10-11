pub(crate) use bijux_dna_api::v1::api::bench::{
    load_benchmark_local_stage_inventory as load_local_stage_inventory,
    render_benchmark_all_domain_stage_inventory as render_all_domain_stage_inventory,
    BenchmarkLocalStageInventoryDomain as BenchLocalDomain,
    BenchmarkLocalStageReadinessKind as LocalStageReadinessKind,
};

pub(crate) const DEFAULT_ALL_DOMAIN_STAGE_LIST_PATH: &str =
    "artifacts/benchmark-smoke/readiness/all-domain-stage-list.json";

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::commands::benchmark::local_vcf_stage_matrix::DEFAULT_VCF_STAGE_MATRIX_PATH;

    use super::{
        load_local_stage_inventory, render_all_domain_stage_inventory, BenchLocalDomain,
        LocalStageReadinessKind, DEFAULT_ALL_DOMAIN_STAGE_LIST_PATH,
    };

    fn repo_root() -> PathBuf {
        crate::commands::support::workspace_root::resolve_repo_root().expect("repo root")
    }

    #[test]
    fn fastq_local_stage_inventory_matches_governed_count() {
        let inventory = load_local_stage_inventory(&repo_root(), BenchLocalDomain::Fastq)
            .expect("load FASTQ local stage inventory");

        assert_eq!(inventory.domain, "fastq");
        assert_eq!(inventory.stage_matrix_path, "configs/bench/local/fastq-stage-matrix.toml");
        assert_eq!(inventory.stage_count, 27);
        assert_eq!(inventory.stages.len(), 27);
        assert!(
            inventory.stages.iter().any(|stage| stage.stage_id == "fastq.screen_taxonomy"
                && stage.readiness_kind == LocalStageReadinessKind::DryOrSmoke),
            "FASTQ local inventory must retain governed mixed readiness coverage"
        );
    }

    #[test]
    fn bam_local_stage_inventory_matches_governed_count() {
        let inventory = load_local_stage_inventory(&repo_root(), BenchLocalDomain::Bam)
            .expect("load BAM local stage inventory");

        assert_eq!(inventory.domain, "bam");
        assert_eq!(inventory.stage_matrix_path, "configs/bench/local/bam-stage-matrix.toml");
        assert_eq!(inventory.stage_count, 24);
        assert_eq!(inventory.stages.len(), 24);
        assert!(
            inventory.stages.iter().any(|stage| stage.stage_id == "bam.align"
                && stage.readiness_kind == LocalStageReadinessKind::DryOrSmoke),
            "BAM local inventory must retain governed mixed readiness coverage"
        );
    }

    #[test]
    fn vcf_local_stage_inventory_matches_governed_count() {
        let inventory = load_local_stage_inventory(&repo_root(), BenchLocalDomain::Vcf)
            .expect("load VCF local stage inventory");

        assert_eq!(inventory.domain, "vcf");
        assert_eq!(inventory.stage_matrix_path, DEFAULT_VCF_STAGE_MATRIX_PATH);
        assert_eq!(inventory.stage_count, 20);
        assert_eq!(inventory.stages.len(), 20);
        assert!(
            inventory.stages.iter().any(|stage| stage.stage_id == "vcf.call"
                && stage.readiness_kind == LocalStageReadinessKind::Smoke),
            "VCF local inventory must retain governed smoke readiness coverage"
        );
    }

    #[test]
    fn all_domain_stage_inventory_aggregates_governed_counts() {
        let root = repo_root();
        let report = render_all_domain_stage_inventory(
            &root,
            &[BenchLocalDomain::Fastq, BenchLocalDomain::Bam, BenchLocalDomain::Vcf],
            PathBuf::from(DEFAULT_ALL_DOMAIN_STAGE_LIST_PATH),
        )
        .expect("render all-domain local stage inventory");

        assert_eq!(
            report.output_path,
            "artifacts/benchmark-smoke/readiness/all-domain-stage-list.json"
        );
        assert_eq!(
            report.selected_domains,
            vec!["fastq".to_string(), "bam".to_string(), "vcf".to_string()]
        );
        assert_eq!(report.domain_counts.get("fastq"), Some(&27));
        assert_eq!(report.domain_counts.get("bam"), Some(&24));
        assert_eq!(report.domain_counts.get("vcf"), Some(&20));
        assert_eq!(report.total_stage_count, 71);
        assert_eq!(report.inventories.len(), 3);
    }
}
