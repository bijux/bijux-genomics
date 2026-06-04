use super::super::*;

fn tool_supports_stage_domain(tools: &ToolMap, tool_id: &str, stage_domain: &str) -> bool {
    tools.get(tool_id).is_some_and(|tool| {
        tool.domain == stage_domain || tool.domains.iter().any(|domain| domain == stage_domain)
    })
}

pub(super) fn build_stages_toml(
    tools: &ToolMap,
    stage_to_tools: &StageToolMap,
    stage_statuses: &StageStatusMap,
    stage_output_kinds: &StageOutputKindsMap,
    production_tool_ids: &BTreeSet<String>,
    domain_dir: &Path,
    source_commit: &str,
) -> String {
    let mut ordering_map = BTreeMap::<String, Vec<String>>::new();
    let mut prereq_map = BTreeMap::<String, Vec<String>>::new();
    let mut resource_map = BTreeMap::<String, StageResourceHint>::new();
    let mut output_size_map = BTreeMap::<String, BTreeMap<String, f64>>::new();
    let mut sanity_map = BTreeMap::<String, Vec<String>>::new();
    let mut qc_thresholds_map = BTreeMap::<String, BTreeMap<String, ThresholdBand>>::new();
    let mut contamination_thresholds_map =
        BTreeMap::<String, BTreeMap<String, ThresholdBand>>::new();
    let mut authenticity_thresholds_map =
        BTreeMap::<String, BTreeMap<String, ThresholdBand>>::new();
    let mut duplication_thresholds_map = BTreeMap::<String, BTreeMap<String, ThresholdBand>>::new();
    let mut coverage_sufficiency_map = BTreeMap::<String, Vec<String>>::new();
    let mut sex_kinship_sufficiency_map = BTreeMap::<String, Vec<String>>::new();
    let mut pipelines = Vec::<(String, Vec<String>)>::new();
    let mut benchmark_scenarios = Vec::<(String, BenchmarkScenario)>::new();
    for dom in ["fastq", "bam"] {
        let index_path = domain_dir.join(dom).join("index.yaml");
        if !index_path.exists() {
            continue;
        }
        if let Ok(index) = read_yaml::<DomainIndex>(&index_path) {
            for (k, v) in index.stage_ordering_constraints {
                ordering_map.insert(k, v);
            }
            for (k, v) in index.stage_prerequisites {
                prereq_map.insert(k, v);
            }
            for (k, v) in index.stage_resource_hints {
                resource_map.insert(k, v);
            }
            for (k, v) in index.stage_output_size_estimates_mb {
                output_size_map.insert(k, v);
            }
            for (k, v) in index.stage_sanity_metrics {
                sanity_map.insert(k, v);
            }
            for (k, v) in index.stage_qc_thresholds {
                qc_thresholds_map.insert(k, v);
            }
            for (k, v) in index.stage_contamination_thresholds {
                contamination_thresholds_map.insert(k, v);
            }
            for (k, v) in index.stage_authenticity_thresholds {
                authenticity_thresholds_map.insert(k, v);
            }
            for (k, v) in index.stage_duplication_thresholds {
                duplication_thresholds_map.insert(k, v);
            }
            for (k, v) in index.stage_coverage_sufficiency {
                coverage_sufficiency_map.insert(k, v);
            }
            for (k, v) in index.stage_sex_kinship_sufficiency {
                sex_kinship_sufficiency_map.insert(k, v);
            }
            for (pipeline, stages) in index.pipeline_compositions {
                pipelines.push((format!("{dom}.{pipeline}"), stages));
            }
            for (scenario_id, scenario) in index.benchmark_scenarios {
                benchmark_scenarios.push((format!("{dom}.{scenario_id}"), scenario));
            }
        }
    }
    let mut stages_toml = generated_header("domain/**", source_commit);
    for (stage_id, tools_set) in stage_to_tools {
        let status =
            stage_statuses.get(stage_id.as_str()).map_or("planned", std::string::String::as_str);
        if status != "supported" {
            continue;
        }
        let _ = writeln!(stages_toml, "[[stages]]");
        let _ = writeln!(stages_toml, "id = \"{stage_id}\"");
        let _ = writeln!(stages_toml, "status = \"{status}\"");
        let stage_domain = stage_id.split('.').next().unwrap_or_default();
        let mut v = tools_set.iter().cloned().collect::<Vec<_>>();
        v.retain(|tool_id| {
            production_tool_ids.contains(tool_id)
                && tool_supports_stage_domain(tools, tool_id, stage_domain)
        });
        v.sort();
        let output_kinds = stage_output_kinds.get(stage_id).cloned().unwrap_or_default();
        let _ = writeln!(stages_toml, "output_kinds = {}", toml_array(&output_kinds));
        let _ = writeln!(
            stages_toml,
            "ordering_after = {}",
            toml_array(ordering_map.get(stage_id).map_or(&[], Vec::as_slice))
        );
        let _ = writeln!(
            stages_toml,
            "prerequisites = {}",
            toml_array(prereq_map.get(stage_id).map_or(&[], Vec::as_slice))
        );
        if let Some(resources) = resource_map.get(stage_id) {
            let _ = writeln!(stages_toml, "resource_memory_gb = {}", resources.memory_gb);
            let _ = writeln!(stages_toml, "resource_time_minutes = {}", resources.time_minutes);
            let _ = writeln!(stages_toml, "resource_threads = {}", resources.threads);
        }
        if let Some(sanity) = sanity_map.get(stage_id) {
            let _ = writeln!(stages_toml, "sanity_metrics = {}", toml_array(sanity));
        }
        if let Some(size_estimates) = output_size_map.get(stage_id) {
            let _ = writeln!(
                stages_toml,
                "output_size_estimates_mb = {}",
                encode_f64_map(size_estimates)
            );
        }
        if let Some(qc) = qc_thresholds_map.get(stage_id) {
            let _ = writeln!(stages_toml, "qc_thresholds = {}", encode_threshold_map(qc));
        }
        if let Some(contam) = contamination_thresholds_map.get(stage_id) {
            let _ = writeln!(
                stages_toml,
                "contamination_thresholds = {}",
                encode_threshold_map(contam)
            );
        }
        if let Some(auth) = authenticity_thresholds_map.get(stage_id) {
            let _ =
                writeln!(stages_toml, "authenticity_thresholds = {}", encode_threshold_map(auth));
        }
        if let Some(dup) = duplication_thresholds_map.get(stage_id) {
            let _ = writeln!(stages_toml, "duplication_thresholds = {}", encode_threshold_map(dup));
        }
        if let Some(coverage_logic) = coverage_sufficiency_map.get(stage_id) {
            let _ = writeln!(stages_toml, "coverage_sufficiency = {}", toml_array(coverage_logic));
        }
        if let Some(sex_kinship_logic) = sex_kinship_sufficiency_map.get(stage_id) {
            let _ = writeln!(
                stages_toml,
                "sex_kinship_sufficiency = {}",
                toml_array(sex_kinship_logic)
            );
        }
        let _ = writeln!(stages_toml, "tools = {}\n", toml_array(&v));
    }
    pipelines.sort_by(|a, b| a.0.cmp(&b.0));
    for (pipeline_id, stages) in pipelines {
        let _ = writeln!(stages_toml, "[[pipelines]]");
        let _ = writeln!(stages_toml, "id = \"{pipeline_id}\"");
        let _ = writeln!(stages_toml, "stages = {}", toml_array(&stages));
        stages_toml.push('\n');
    }
    benchmark_scenarios.sort_by(|a, b| a.0.cmp(&b.0));
    for (scenario_id, scenario) in benchmark_scenarios {
        let _ = writeln!(stages_toml, "[[benchmark_scenarios]]");
        let _ = writeln!(stages_toml, "id = \"{scenario_id}\"");
        let _ = writeln!(stages_toml, "stage_id = \"{}\"", scenario.stage_id);
        let _ =
            writeln!(stages_toml, "description = \"{}\"", scenario.description.replace('"', "'"));
        let _ = writeln!(stages_toml, "fairness_rules = {}", toml_array(&scenario.fairness_rules));
        stages_toml.push('\n');
    }
    stages_toml
}
