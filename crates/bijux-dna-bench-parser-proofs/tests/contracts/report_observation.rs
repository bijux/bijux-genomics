use anyhow::{Context, Result};
use bijux_dna_bench_parser_proofs::reports::{parse_proof, ParseKind};
use serde_json::{json, Value};

fn reports() -> Vec<(ParseKind, Value, &'static str)> {
    vec![
        (
            ParseKind::VcfCallMetrics,
            json!({"schema_version": "fixture.v1", "stage_id": null, "tool_id": "declared-tool", "variant_count": null, "sample_count": null}),
            "stage_id",
        ),
        (
            ParseKind::VcfDemographyReport,
            json!({"schema_version": "fixture.v1", "stage_id": null, "tool_id": "declared-tool", "method": null, "inference_status": null, "time_bins": null, "ne_estimates": null, "status": null}),
            "stage_id",
        ),
        (
            ParseKind::VcfIbdReport,
            json!({"schema_version": "fixture.v1", "stage_id": null, "tool_id": "declared-tool", "execution_mode": null, "pair_count": null, "rows": null, "status": null}),
            "stage_id",
        ),
        (
            ParseKind::VcfImputeMetrics,
            json!({"schema_version": "fixture.v1", "stage_id": null, "tool_id": "declared-tool", "variant_count": null, "imputed_genotypes": null}),
            "stage_id",
        ),
        (
            ParseKind::VcfPhasingMetrics,
            json!({"schema_version": "fixture.v1", "stage_id": null, "tool_id": "declared-tool", "phased_genotypes": null, "phase_set_count": null}),
            "stage_id",
        ),
        (
            ParseKind::VcfPopulationStructureReport,
            json!({"schema_version": "fixture.v1", "tool_id": "declared-tool", "status": null, "sample_groups": null, "distance_summary": null}),
            "tool_id",
        ),
        (
            ParseKind::VcfQcMetrics,
            json!({"schema_version": "fixture.v1", "tool_id": "declared-tool", "sample_missingness": null, "heterozygosity": null, "hwe_summary": null}),
            "tool_id",
        ),
    ]
}

#[test]
fn vcf_kinds_return_the_admitted_value_and_project_exact_fields() -> Result<()> {
    let owner = tempfile::tempdir()?;
    let path = owner.path().join("report.json");
    for (kind, value, _) in reports() {
        std::fs::write(&path, serde_json::to_vec(&value)?)?;
        let (observed, snapshot) =
            parse_proof(&path, kind, &["schema_version", "tool_id", "absent"])?;
        assert_eq!(observed, value);
        assert_eq!(snapshot.len(), 2);
        assert_eq!(snapshot.get("schema_version"), value.get("schema_version"));
        assert_eq!(snapshot.get("tool_id"), value.get("tool_id"));
        assert!(!snapshot.contains_key("absent"));
    }
    Ok(())
}

#[test]
fn vcf_missing_key_priority_retains_the_path_and_kind_order() -> Result<()> {
    let owner = tempfile::tempdir()?;
    let path = owner.path().join("report.json");
    for (kind, mut value, first_missing) in reports() {
        let object = value.as_object_mut().context("object report fixture")?;
        let _ = object.remove("stage_id");
        assert!(object.remove("tool_id").is_some());
        std::fs::write(&path, serde_json::to_vec(&value)?)?;
        let error =
            parse_proof(&path, kind, &[]).err().context("missing governed keys admitted")?;
        assert_eq!(error.to_string(), format!("{} is missing `{first_missing}`", path.display()));
    }
    Ok(())
}

#[test]
fn vcf_public_facade_preserves_read_and_parse_diagnostics() -> Result<()> {
    let owner = tempfile::tempdir()?;
    let path = owner.path().join("report.json");
    let error = parse_proof(&path, ParseKind::VcfCallMetrics, &[])
        .err()
        .context("absent report admitted")?;
    assert_eq!(error.to_string(), format!("read {}", path.display()));
    std::fs::create_dir(&path)?;
    assert!(parse_proof(&path, ParseKind::VcfCallMetrics, &[]).is_err());
    std::fs::remove_dir(&path)?;
    std::fs::write(&path, [0xff])?;
    let error = parse_proof(&path, ParseKind::VcfCallMetrics, &[])
        .err()
        .context("non-UTF8 report admitted")?;
    assert_eq!(error.to_string(), format!("read {}", path.display()));
    std::fs::write(&path, "{")?;
    let error = parse_proof(&path, ParseKind::VcfCallMetrics, &[])
        .err()
        .context("malformed report admitted")?;
    assert_eq!(error.to_string(), format!("parse {}", path.display()));
    Ok(())
}
