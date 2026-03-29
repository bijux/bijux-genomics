#[must_use]
pub fn param_registry_toml() -> String {
    let mut out = String::new();
    out.push_str("# GENERATED - DO NOT EDIT - source: crates/bijux-dna-domain-vcf\n\n");
    out.push_str("[[params]]\nstage_id = \"vcf.call\"\nparam_type_id = \"bijux.vcf.call.params\"\nschema_version = \"bijux.vcf.params.v1\"\n\n");
    out.push_str("[[params]]\nstage_id = \"vcf.filter\"\nparam_type_id = \"bijux.vcf.filter.params\"\nschema_version = \"bijux.vcf.params.v1\"\n\n");
    out.push_str("[[params]]\nstage_id = \"vcf.stats\"\nparam_type_id = \"bijux.vcf.stats.params\"\nschema_version = \"bijux.vcf.params.v1\"\n");
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
