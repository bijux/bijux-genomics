#[must_use]
pub fn param_registry_toml() -> String {
    const HEADER: &str = "# schema_version = 1\n\
# owner = bijux-dna-infra\n\
# purpose = Contract config for configs/ci/params/param_registry_vcf.toml\n\
# authority = bijux-dna-infra\n\
# stability = stable\n\
# last_updated = 2026-02-13\n\
\n\
# GENERATED - DO NOT EDIT - source: crates/bijux-dna-domain-vcf\n\n";
    const PARAMS: &[(&str, &str)] = &[
        ("vcf.call", "bijux.vcf.call.params"),
        ("vcf.filter", "bijux.vcf.filter.params"),
        ("vcf.stats", "bijux.vcf.stats.params"),
        ("vcf.call_gl", "bijux.vcf.call_gl.params"),
        ("vcf.call_diploid", "bijux.vcf.call_diploid.params"),
        ("vcf.call_pseudohaploid", "bijux.vcf.call_pseudohaploid.params"),
        ("vcf.damage_filter", "bijux.vcf.damage_filter.params"),
        ("vcf.gl_propagation", "bijux.vcf.gl_propagation.params"),
    ];

    let mut out = String::new();
    out.push_str(HEADER);
    for (index, (stage_id, param_type_id)) in PARAMS.iter().enumerate() {
        if index > 0 {
            out.push('\n');
        }
        out.push_str("[[params]]\n");
        out.push_str(&format!("stage_id = \"{stage_id}\"\n"));
        out.push_str(&format!("param_type_id = \"{param_type_id}\"\n"));
        out.push_str("schema_version = \"bijux.vcf.params.v1\"\n");
        out.push_str("params = []\n");
    }
    out
}

#[must_use]
pub fn required_tools_toml() -> String {
    let mut out = String::new();
    out.push_str("# GENERATED - DO NOT EDIT - source: crates/bijux-dna-domain-vcf\n");
    out.push_str("# source_commit: 53b050a6d117e40e0122777655e9d8cc428be9ad\n");
    out.push_str("# domain_schema_version: bijux.domain.v1\n\n");
    out.push_str("schema_version = \"bijux.required_tools.v1\"\n");
    out.push_str("required_tools = [\"bcftools\"]\n");
    out
}
