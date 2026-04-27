#![allow(non_snake_case)]
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    bijux_dna_testkit::workspace_root_from_manifest(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn policy__contracts__vcf_gl_propagation_policy__stage_contract_requires_gl_field_retention() {
    let root = repo_root();
    let stage = root.join("domain/vcf/stages/gl_propagation.yaml");
    let content = std::fs::read_to_string(&stage)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", stage.display()));

    bijux_dna_policies::policy_assert!(
        content.contains("gl_fields_present"),
        "gl_propagation stage must require gl_fields_present invariant: {}",
        stage.display()
    );
    bijux_dna_policies::policy_assert!(
        content.contains("gl_propagated_vcf") && content.contains("gl_propagation_report"),
        "gl_propagation stage must require gl_propagated_vcf and gl_propagation_report outputs: {}",
        stage.display()
    );
}

#[test]
fn policy__contracts__vcf_gl_propagation_policy__fixture_contract_asserts_gl_pl_survival() {
    let root = repo_root();
    let fx = root.join("domain/vcf/fixtures/vcf.gl_propagation/bcftools.txt");
    let content = std::fs::read_to_string(&fx)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", fx.display()));

    bijux_dna_policies::policy_assert!(
        content.contains("expected_outputs=gl_propagated.vcf,gl_propagation_report.json"),
        "vcf.gl_propagation fixture must emit gl_propagation_report.json: {}",
        fx.display()
    );
    bijux_dna_policies::policy_assert!(
        content.contains("expected_stdout_patterns=GL,PL,propagation_pass"),
        "vcf.gl_propagation fixture must assert GL/PL propagation pass markers: {}",
        fx.display()
    );
}
