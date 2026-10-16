use super::*;

fn write_input(root: &Path, relative: &str, payload: &str) -> Result<()> {
    let path = root.join(relative);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, payload)?;
    Ok(())
}

fn fixture(root: &Path) -> Result<()> {
    write_input(root, "domain/fastq/metrics.yaml", "metric_ids: [z, a]\n")?;
    write_input(root, "domain/bam/metrics.yaml", "metrics: [{id: b}]\n")?;
    write_input(
        root,
        "domain/fastq/stages/nested/reads.yaml",
        "stage_id: shared\nmetrics: [{name: z, meaning: \"a\\tb\\nc\\rd\"}, {name: a}]\n",
    )?;
    write_input(root, "domain/bam/stages/align.yaml", "stage_id: shared\nmetrics: [{name: b}]\n")?;
    let metric_ids = VcfDomainStage::all()
        .iter()
        .flat_map(|stage| stage_metrics_contract(*stage).required_metrics.iter().copied())
        .collect::<BTreeSet<_>>();
    let vocabulary = serde_yaml::to_string(&serde_json::json!({"metric_ids": metric_ids}))?;
    write_input(root, "domain/vcf/metrics.yaml", &vocabulary)?;
    Ok(())
}

fn rejection<T>(result: Result<T>) -> Result<anyhow::Error> {
    match result {
        Err(error) => Ok(error),
        Ok(_) => bail!("expected metric registry refusal"),
    }
}

#[test]
fn metric_registry_preserves_vocabulary_forms_and_yaml_error_chain() -> Result<()> {
    let temp = bijux_dna_testkit::tempdir_for(
        "metric_registry_preserves_vocabulary_forms_and_yaml_error_chain",
    );
    let path = temp.path().join("metrics.yaml");
    std::fs::write(
        &path,
        "metric_ids: [a, ' ', ' a ']\nmetrics: [{id: a}, {id: b}, {}]\nunknown: retained\n",
    )?;
    assert_eq!(
        load_domain_metric_vocabulary(&path)?,
        BTreeSet::from([" a ".to_string(), "a".to_string(), "b".to_string(),])
    );
    std::fs::write(&path, "metric_ids: [\n")?;
    let error = rejection(load_domain_metric_vocabulary(&path))?;
    let Err(parser_error) = serde_yaml::from_str::<MetricRegistryVocabularyDoc>("metric_ids: [\n")
    else {
        bail!("malformed YAML unexpectedly parsed");
    };
    assert_eq!(error.to_string(), format!("parse {}", path.display()));
    assert_eq!(error.root_cause().to_string(), parser_error.to_string());
    std::fs::write(&path, "metric_ids: [' ']\nmetrics: [{}]\n")?;
    let error = rejection(load_domain_metric_vocabulary(&path))?;
    assert_eq!(error.to_string(), format!("metric vocabulary `{}` is empty", path.display()));
    Ok(())
}

#[test]
fn metric_registry_stage_acquisition_preserves_order_and_refusal_priority() -> Result<()> {
    let temp = bijux_dna_testkit::tempdir_for(
        "metric_registry_stage_acquisition_preserves_order_and_refusal_priority",
    );
    let root = temp.path();
    let vocabulary = root.join("domain/fastq/metrics.yaml");
    write_input(root, "domain/fastq/stages/a.yaml", "[invalid")?;
    let error = rejection(collect_stage_yaml_metric_registry_rows(root, "fastq"))?;
    assert_eq!(error.to_string(), format!("read {}", vocabulary.display()));
    write_input(root, "domain/fastq/metrics.yaml", "metric_ids: [a, b]\n")?;
    write_input(root, "domain/fastq/stages/a.yaml", "stage_id: first\nmetrics: [{name: b}]\n")?;
    write_input(
        root,
        "domain/fastq/stages/nested/b.yaml",
        "stage_id: second\nmetrics: [{name: a}]\n",
    )?;
    write_input(root, "domain/fastq/stages/_schema.yaml", "[invalid")?;
    write_input(root, "domain/fastq/stages/blank.yaml", "metrics: [{}]\n")?;
    write_input(root, "domain/fastq/stages/not_yaml.yml", "[invalid")?;
    let rows = collect_stage_yaml_metric_registry_rows(root, "fastq")?;
    assert_eq!(
        rows.iter().map(|row| row.stage_id.as_str()).collect::<Vec<_>>(),
        ["first", "second"]
    );
    let first = root.join("domain/fastq/stages/a.yaml");
    write_input(root, "domain/fastq/stages/a.yaml", "stage_id: first\nmetrics: [{}]\n")?;
    assert_eq!(
        rejection(collect_stage_yaml_metric_registry_rows(root, "fastq"))?.to_string(),
        format!("stage metric declaration in `{}` is missing a metric name", first.display())
    );
    write_input(
        root,
        "domain/fastq/stages/a.yaml",
        "stage_id: first\nmetrics: [{name: a}, {name: a}, {name: unknown}]\n",
    )?;
    assert_eq!(
        rejection(collect_stage_yaml_metric_registry_rows(root, "fastq"))?.to_string(),
        format!("stage `first` declares duplicate metric `a` in `{}`", first.display())
    );
    write_input(
        root,
        "domain/fastq/stages/a.yaml",
        "stage_id: first\nmetrics: [{name: unknown}, {name: a}, {name: a}]\n",
    )?;
    assert_eq!(
        rejection(collect_stage_yaml_metric_registry_rows(root, "fastq"))?.to_string(),
        format!("stage `first` declares unregistered metric `unknown` in `{}`", first.display())
    );
    Ok(())
}

#[test]
fn metric_registry_publication_preserves_wire_order_counts_and_tsv() -> Result<()> {
    let temp = bijux_dna_testkit::tempdir_for(
        "metric_registry_publication_preserves_wire_order_counts_and_tsv",
    );
    let root = temp.path();
    fixture(root)?;
    let source = root.join("domain/fastq/stages/nested/reads.yaml");
    let before = std::fs::read(&source)?;
    let output = root.join("published/metrics.tsv");
    let report: crate::v1::api::MetricRegistryReport =
        crate::v1::api::write_metric_registry_report(root, &output)?;
    let rows: &[crate::v1::api::MetricRegistryRow] = &report.rows;
    assert_eq!(report.schema_version, "bijux.crates.metric_registry.v1");
    assert_eq!(report.workspace_manifest, "Cargo.toml");
    assert_eq!(report.output_path, "published/metrics.tsv");
    assert_eq!(report.domain_count, 3);
    let expected_vcf = VcfDomainStage::all()
        .iter()
        .flat_map(|stage| {
            stage_metrics_contract(*stage)
                .required_metrics
                .iter()
                .map(move |id| (stage.as_str(), *id))
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(
        rows.iter()
            .filter(|row| row.domain == "vcf")
            .map(|row| (row.stage_id.as_str(), row.metric_id.as_str()))
            .collect::<BTreeSet<_>>(),
        expected_vcf
    );
    assert_eq!(report.row_count, 3 + expected_vcf.len());
    assert_eq!(
        report.stage_count,
        1 + expected_vcf.iter().map(|(stage, _)| *stage).collect::<BTreeSet<_>>().len()
    );
    assert_eq!(
        rows.iter().map(|row| (&row.domain, &row.stage_id, &row.metric_id)).collect::<Vec<_>>(),
        rows.iter()
            .map(|row| (&row.domain, &row.stage_id, &row.metric_id))
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>()
    );
    let tsv = std::fs::read_to_string(&output)?;
    assert_eq!(tsv.lines().next(), Some("domain\tstage_id\tmetric_id\tmeaning\tcontract_kind\tstage_contract_surface\tdomain_registry_surface"));
    assert!(tsv.contains("fastq\tshared\tz\ta b c d\tyaml_stage_metrics\tdomain/fastq/stages/nested/reads.yaml\tdomain/fastq/metrics.yaml\n"));
    assert_eq!(tsv.lines().count(), report.row_count + 1);
    let wire = serde_json::to_value(&report)?;
    assert_eq!(wire.as_object().map(serde_json::Map::len), Some(7));
    assert_eq!(wire["rows"][0].as_object().map(serde_json::Map::len), Some(7));
    assert_eq!(
        wire["rows"][0],
        serde_json::json!({
            "domain": "bam", "stage_id": "shared", "metric_id": "b", "meaning": "",
            "contract_kind": "yaml_stage_metrics", "stage_contract_surface": "domain/bam/stages/align.yaml",
            "domain_registry_surface": "domain/bam/metrics.yaml",
        })
    );
    assert_eq!(std::fs::read(&source)?, before);
    Ok(())
}

#[test]
fn metric_registry_refusal_preserves_output_and_vcf_contract_authority() -> Result<()> {
    let temp = bijux_dna_testkit::tempdir_for(
        "metric_registry_refusal_preserves_output_and_vcf_contract_authority",
    );
    let root = temp.path();
    fixture(root)?;
    let output = root.join("metrics.tsv");
    std::fs::write(&output, b"retained publication\n")?;
    write_input(
        root,
        "domain/fastq/stages/duplicate.yaml",
        "stage_id: shared\nmetrics: [{name: a}]\n",
    )?;
    assert_eq!(
        rejection(write_metric_registry_report(root, &output))?.to_string(),
        "metric registry produced duplicate row for `fastq` / `shared` / `a`"
    );
    assert_eq!(std::fs::read(&output)?, b"retained publication\n");
    std::fs::remove_file(root.join("domain/fastq/stages/duplicate.yaml"))?;
    let (stage, metric) = VcfDomainStage::all()
        .iter()
        .find_map(|stage| {
            stage_metrics_contract(*stage).required_metrics.first().map(|metric| (*stage, *metric))
        })
        .ok_or_else(|| anyhow::anyhow!("VCF contracts must declare a metric"))?;
    write_input(root, "domain/vcf/metrics.yaml", "metric_ids: [unregistered-control]\n")?;
    assert_eq!(
        rejection(write_metric_registry_report(root, &output))?.to_string(),
        format!(
            "VCF stage `{}` references unregistered metric `{metric}` in `domain/vcf/metrics.yaml`",
            stage.as_str()
        )
    );
    assert_eq!(std::fs::read(&output)?, b"retained publication\n");
    Ok(())
}
