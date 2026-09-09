//! Owner: deterministic illustrative benchmark simulation artifact rendering.
use anyhow::{Context, Result};
use bijux_dna_bench_model::{
    all_domain_simulation_output_ids, AllDomainExpectedBenchmarkResultRow,
    AllDomainOutputDeclarationRow, AllDomainRenderedCommandRow,
};
use std::path::Path;
/// Render illustrative commands as a script artifact without executing them.
#[must_use]
pub fn render_all_domain_simulation_command_script(
    expected: &AllDomainExpectedBenchmarkResultRow,
    command: &AllDomainRenderedCommandRow,
) -> String {
    let mut rendered = String::from("#!/usr/bin/env bash\nset -euo pipefail\n");
    rendered.push_str(&format!(
        "# all-domain benchmark fake-run script\n# result_id={}\n# domain={}\n# stage_id={}\n# tool_id={}\n# corpus_id={}\n# asset_profile_id={}\n\n",
        expected.result_id,
        expected.domain,
        expected.stage_id,
        expected.tool_id,
        expected.corpus_id,
        expected.asset_profile_id
    ));
    for script_command in &command.script_commands {
        rendered.push_str(script_command);
        rendered.push('\n');
    }
    rendered
}
pub(crate) fn render_result_stdout(
    expected: &AllDomainExpectedBenchmarkResultRow,
    command: &AllDomainRenderedCommandRow,
    outputs: &AllDomainOutputDeclarationRow,
) -> String {
    let mut rendered = format!(
        "fake local all-domain benchmark run\nresult_id={}\ndomain={}\nstage_id={}\ntool_id={}\ncorpus_id={}\nasset_profile_id={}\ncommand_source={}\n",
        expected.result_id,
        expected.domain,
        expected.stage_id,
        expected.tool_id,
        expected.corpus_id,
        expected.asset_profile_id,
        command.command_source
    );
    rendered.push_str("declared_outputs=\n");
    for artifact_id in all_domain_simulation_output_ids(outputs) {
        rendered.push_str(&format!("  - {artifact_id}\n"));
    }
    rendered.push_str("expected_metrics=\n");
    for metric in &expected.expected_metrics {
        rendered.push_str(&format!("  - {metric}\n"));
    }
    rendered.push_str("commands=\n");
    for script_command in &command.script_commands {
        rendered.push_str(&format!("  - {script_command}\n"));
    }
    rendered
}
pub(crate) fn fake_output_bytes(
    expected: &AllDomainExpectedBenchmarkResultRow,
    artifact_id: &str,
    output_path: &Path,
) -> Result<Vec<u8>> {
    if binary_output_extension(output_path) {
        return Ok(Vec::new());
    }
    if output_path.extension().and_then(|ext| ext.to_str()) == Some("json") {
        return serde_json::to_vec_pretty(&serde_json::json!({
            "schema_version": "bijux.bench.local_all_domain_fake_output.v1",
            "result_id": expected.result_id,
            "domain": expected.domain,
            "stage_id": expected.stage_id,
            "tool_id": expected.tool_id,
            "artifact_id": artifact_id,
        }))
        .context("serialize all-domain fake JSON output");
    }
    if output_path.extension().and_then(|ext| ext.to_str()) == Some("tsv") {
        return Ok(format!(
            "result_id\tdomain\tstage_id\ttool_id\tartifact_id\n{}\t{}\t{}\t{}\t{artifact_id}\n",
            expected.result_id, expected.domain, expected.stage_id, expected.tool_id
        )
        .into_bytes());
    }
    if output_path.extension().and_then(|ext| ext.to_str()) == Some("html") {
        return Ok(format!(
            "<html><body><h1>fake local all-domain benchmark output</h1><p>{}</p><p>{artifact_id}</p></body></html>\n",
            expected.result_id
        )
        .into_bytes());
    }
    if output_path.extension().and_then(|ext| ext.to_str()) == Some("fastq") {
        return Ok(format!(
            "@{}_{}\nACGT\n+\nIIII\n",
            expected.stage_id.replace('.', "_"),
            artifact_id.replace('.', "_")
        )
        .into_bytes());
    }
    if output_path.extension().and_then(|ext| ext.to_str()) == Some("fasta") {
        return Ok(format!(
            ">{}_{}\nACGTACGT\n",
            expected.stage_id.replace('.', "_"),
            artifact_id.replace('.', "_")
        )
        .into_bytes());
    }
    if output_path.extension().and_then(|ext| ext.to_str()) == Some("vcf") {
        return Ok(format!(
            "##fileformat=VCFv4.3\n#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\nchr1\t1\t{}\tA\tG\t.\tPASS\tRESULT_ID={};STAGE_ID={};TOOL_ID={}\n",
            artifact_id.replace('.', "_"),
            expected.result_id,
            expected.stage_id,
            expected.tool_id
        )
        .into_bytes());
    }

    Ok(format!(
        "fake local all-domain benchmark output\nresult_id={}\ndomain={}\nstage_id={}\ntool_id={}\nartifact_id={artifact_id}\n",
        expected.result_id, expected.domain, expected.stage_id, expected.tool_id
    )
    .into_bytes())
}
pub(crate) fn output_path_is_directory(artifact_id: &str) -> bool {
    artifact_id.ends_with("_bundle") || artifact_id.ends_with("_dir")
}
pub(crate) fn binary_output_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| matches!(ext, "bam" | "bai" | "bcf" | "gz" | "pdf" | "tbi" | "zip"))
}
