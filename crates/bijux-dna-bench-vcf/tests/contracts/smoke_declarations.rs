use anyhow::{Context, Result};
use bijux_dna_bench_vcf::{
    adapters::observations::VcfAdapterObservations,
    evidence::{
        expectations::collect_vcf_expected_benchmark_result_rows,
        scope::collect_vcf_active_stage_tool_matrix_rows_from_expected,
        smoke::{
            render_vcf_local_container_smoke, VcfSmokeInputs,
            DEFAULT_VCF_LOCAL_CONTAINER_SMOKE_PATH, VCF_LOCAL_CONTAINER_SMOKE_SCHEMA_VERSION,
        },
    },
};
use bijux_dna_bench_vcf_evidence::snapshot::VcfStageReadinessBindingKey;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};
fn host_support(root: &Path, stage: &str) -> Result<Option<String>> {
    let suffix = stage.strip_prefix("vcf.").context("VCF stage")?;
    let path = format!("crates/bijux-dna/src/commands/benchmark/local_vcf_{suffix}_smoke.rs");
    Ok(root.join(&path).is_file().then_some(path))
}
fn with_governed<T>(run: impl FnOnce(&Path, &VcfSmokeInputs<'_>) -> Result<T>) -> Result<T> {
    let root = crate::support::repo_root();
    let bindings = crate::support::inputs(&root);
    let observations = VcfAdapterObservations::new(&bindings, &root);
    let expected = collect_vcf_expected_benchmark_result_rows(&observations)?;
    let active = expected
        .iter()
        .map(|row| VcfStageReadinessBindingKey {
            stage_id: row.stage_id.clone(),
            tool_id: row.tool_id.clone(),
            corpus_id: row.corpus_id.clone(),
            asset_profile_id: row.asset_profile_id.clone(),
        })
        .collect::<Vec<_>>();
    let retained = collect_vcf_active_stage_tool_matrix_rows_from_expected(
        &bindings, &root, &active, &expected,
    )?;
    let matrix: bijux_dna_bench_model::LocalVcfStageMatrixConfig = toml::from_str(
        &std::fs::read_to_string(root.join("configs/bench/local/vcf-stage-matrix.toml"))?,
    )?;
    let defaults =
        matrix.rows.into_iter().map(|row| (row.stage_id, row.tool_id)).collect::<BTreeMap<_, _>>();
    // Test composition supplies external identities; canonical delivery schema admission is container-owned.
    let catalog: toml::Value =
        toml::from_str(&std::fs::read_to_string(root.join("containers/contracts/catalog.toml"))?)?;
    let external = catalog
        .get("tools")
        .and_then(toml::Value::as_array)
        .context("tools")?
        .iter()
        .filter(|tool| {
            tool.get("external_artifact_supported").and_then(toml::Value::as_bool) == Some(true)
        })
        .map(|tool| {
            tool.get("id").and_then(toml::Value::as_str).map(str::to_owned).context("tool identity")
        })
        .collect::<Result<BTreeSet<_>>>()?;
    run(
        &root,
        &VcfSmokeInputs {
            bindings: &bindings,
            retained: &retained,
            stage_defaults: &defaults,
            external_tool_ids: &external,
            host_support,
        },
    )
}
#[test]
fn vcf_local_container_smoke_tracks_retained_bindings() -> Result<()> {
    let report = with_governed(|root, inputs| {
        render_vcf_local_container_smoke(
            root,
            Path::new(DEFAULT_VCF_LOCAL_CONTAINER_SMOKE_PATH),
            inputs,
        )
    })?;
    assert_eq!(report.schema_version, VCF_LOCAL_CONTAINER_SMOKE_SCHEMA_VERSION);
    assert_eq!(
        report.output_path,
        "artifacts/benchmark-smoke/readiness/vcf/vcf-local-container-smoke.tsv"
    );
    assert_eq!(report.row_count, 39);
    assert_eq!(report.stage_count, 20);
    assert_eq!(report.tool_count, 16);
    assert_eq!(report.host_stage_smoke_row_count, 20);
    assert_eq!(report.container_smoke_row_count, 15);
    assert_eq!(report.external_artifact_row_count, 4);
    assert_eq!(report.runtime_counts.get("host").copied(), Some(20));
    assert_eq!(report.runtime_counts.get("docker-arm64").copied(), Some(11));
    assert_eq!(report.runtime_counts.get("apptainer").copied(), Some(4));
    assert_eq!(report.runtime_counts.get("external").copied(), Some(4));

    assert!(report.rows.iter().any(|row| {
        row.stage_id == "vcf.call"
            && row.tool_id == "bcftools"
            && row.smoke_path_kind == "host_stage_smoke"
            && row.smoke_command == "bijux-dna bench local run-vcf-call-smoke --tool-id bcftools"
    }));
    for (stage_id, tool_id) in [
        ("vcf.call_gl", "angsd"),
        ("vcf.call_pseudohaploid", "angsd"),
        ("vcf.imputation_metrics", "impute5"),
        ("vcf.impute", "impute5"),
    ] {
        assert!(report.rows.iter().any(|row| {
            row.stage_id == stage_id
                && row.tool_id == tool_id
                && row.smoke_path_kind == "external_artifact_admission"
                && row.smoke_runtime == "external"
                && row.smoke_support_path == "containers/contracts/catalog.toml"
        }));
    }
    assert!(report.rows.iter().any(|row| {
        row.stage_id == "vcf.impute"
            && row.tool_id == "glimpse"
            && row.smoke_path_kind == "docker_container_smoke"
            && row.smoke_command == "bijux-dna env smoke docker-arm64 glimpse"
    }));
    assert!(report.rows.iter().any(|row| {
        row.stage_id == "vcf.postprocess"
            && row.tool_id == "bcftools"
            && row.smoke_path_kind == "host_stage_smoke"
            && row.smoke_command
                == "bijux-dna bench local run-vcf-postprocess-smoke --tool-id bcftools"
    }));
    assert!(report.rows.iter().any(|row| {
        row.stage_id == "vcf.phasing"
            && row.tool_id == "shapeit"
            && row.smoke_path_kind == "apptainer_container_smoke"
            && row.smoke_command == "bijux-dna env smoke apptainer shapeit"
    }));
    Ok(())
}
#[test]
fn malformed_smoke_scope_preserves_existing_report_before_publication() -> Result<()> {
    with_governed(|root, inputs| {
        let directory = tempfile::tempdir()?;
        let output = directory.path().join("smoke.tsv");
        std::fs::write(&output, b"prior evidence")?;
        let mut duplicate = inputs.retained.to_vec();
        duplicate.push(inputs.retained[0].clone());
        let mut foreign = inputs.retained.to_vec();
        foreign[0].corpus_id = "foreign".to_owned();
        let mut absent = inputs.retained.to_vec();
        absent.pop();
        for retained in [duplicate, foreign, absent] {
            let invalid = VcfSmokeInputs {
                retained: &retained,
                bindings: inputs.bindings,
                stage_defaults: inputs.stage_defaults,
                external_tool_ids: inputs.external_tool_ids,
                host_support,
            };
            assert!(render_vcf_local_container_smoke(root, &output, &invalid).is_err());
            assert_eq!(std::fs::read(&output)?, b"prior evidence");
        }
        let mut defaults = inputs.stage_defaults.clone();
        defaults.pop_first();
        let invalid = VcfSmokeInputs {
            retained: inputs.retained,
            bindings: inputs.bindings,
            stage_defaults: &defaults,
            external_tool_ids: inputs.external_tool_ids,
            host_support,
        };
        assert!(render_vcf_local_container_smoke(root, &output, &invalid).is_err());
        assert_eq!(std::fs::read(output)?, b"prior evidence");
        Ok(())
    })
}
