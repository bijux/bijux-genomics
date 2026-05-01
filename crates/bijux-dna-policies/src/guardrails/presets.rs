use super::configuration::GuardrailConfig;

pub(super) fn for_crate(name: &str) -> GuardrailConfig {
    let mut config = GuardrailConfig::default();
    if name == "bijux-dna-domain-bam" {
        config.max_pub_items_per_file = 120;
        config.allow_stage_id_paths = vec!["/src/stage_specs/mod.rs".to_string()];
    }
    if name == "bijux-dna-domain-fastq" {
        config.max_pub_items_per_file = 110;
        config.allow_stage_id_paths = vec!["/src/id_catalog.rs".to_string()];
    }
    if name == "bijux-dna-runtime" {
        config.max_pub_items_per_file = 90;
    }
    if name == "bijux-dna-pipelines" {
        config.allow_mod_only_dirs = vec!["/src/vcf".to_string()];
    }
    if name == "bijux-dna" {
        config.allow_mod_only_dirs = vec![
            "/src/commands/corpus".to_string(),
            "/src/commands/planning".to_string(),
            "/src/commands/status".to_string(),
            "/src/public_api".to_string(),
        ];
    }
    if name == "bijux-dna-api" {
        config.allow_mod_only_dirs = vec!["/src/internal/fastq".to_string()];
    }
    config
}
