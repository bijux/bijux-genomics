use super::*;

fn metadata() -> Value {
    let binaries = ["owner::a", "owner::b", "owner::empty"]
        .into_iter()
        .map(|id| {
            (
                id.to_string(),
                json!({
                    "binary-id":id, "binary-name":id, "package-id":"owner-package", "kind":"test",
                    "binary-path":format!("/compiled/{id}"), "build-platform":"target"
                }),
            )
        })
        .collect::<BTreeMap<_, _>>();
    json!({"rust-build-meta":{"target-directory":"/compiled"},"rust-binaries":binaries})
}

fn observations(plan: &BinaryDiscoveryPlan) -> Result<Vec<(String, Value)>> {
    plan.groups.iter().map(|group| {
        let mut suites = BTreeMap::new();
        let mut count = 0usize;
        let metadata: Value = serde_json::from_slice(&group.metadata)?;
        for (id, binary) in metadata["rust-binaries"].as_object().into_iter().flatten() {
            let mut suite = binary.clone();
            suite["package-name"] = json!("owner");
            suite["status"] = json!("listed");
            suite["testcases"] = if id.ends_with("empty") {json!({})} else {
                count += 1;
                json!({"case":{"ignored":id.ends_with('b'), "filter-match":{"status":"matches"}}})
            };
            suites.insert(id.clone(),suite);
        }
        Ok((group.label.clone(),json!({"rust-build-meta":plan.metadata["rust-build-meta"],"test-count":count,"rust-suites":suites})))
    }).collect()
}

fn merge(plan: &BinaryDiscoveryPlan, values: &[(String, Value)]) -> Result<Vec<u8>> {
    let bytes = values
        .iter()
        .map(|(name, value)| Ok((name.clone(), serde_json::to_vec(value)?)))
        .collect::<Result<Vec<_>>>()?;
    plan.merge(bytes.iter().map(|(name, bytes)| (name.as_str(), bytes.as_slice())))
}

#[test]
fn binary_discovery_preserves_empty_ignored_and_ordinary_suites() -> Result<()> {
    for native_budget in [2, 4, 10] {
        let plan = BinaryDiscoveryPlan::new(&serde_json::to_vec(&metadata())?, native_budget)?;
        let bytes = merge(&plan, &observations(&plan)?)?;
        let result = ExpectedTestInventory::parse(&bytes)?;
        assert_eq!(result.case_count(), 2);
        assert_eq!(result.ignored_case_count(), 1);
        assert_eq!(result.suites.len(), 3);
        assert_eq!(plan.groups.len(), 3usize.div_ceil(native_budget / 2));
        assert!(plan.groups.iter().all(|group| group.worker_demand(0, native_budget).is_ok()));
    }
    Ok(())
}

#[test]
fn binary_discovery_rejects_missing_repeated_or_unknown_groups() -> Result<()> {
    let plan = BinaryDiscoveryPlan::new(&serde_json::to_vec(&metadata())?, 2)?;
    let valid = observations(&plan)?;
    assert!(merge(&plan, &valid[..1]).is_err());
    assert!(merge(&plan, &[valid[0].clone(), valid[0].clone()]).is_err());
    let mut changed = valid.clone();
    changed[0].0 = "unexpected".to_string();
    assert!(merge(&plan, &changed).is_err());
    assert!(BinaryDiscoveryPlan::new(&serde_json::to_vec(&metadata())?, 0).is_err());
    assert!(BinaryDiscoveryPlan::new(&serde_json::to_vec(&metadata())?, 1).is_err());
    Ok(())
}

#[test]
fn binary_discovery_rejects_changed_metadata_coverage_counts_and_execution_scope() -> Result<()> {
    let plan = BinaryDiscoveryPlan::new(&serde_json::to_vec(&metadata())?, 2)?;
    let valid = observations(&plan)?;
    for (pointer, replacement) in [
        ("/rust-build-meta/target-directory", json!("/other")),
        ("/test-count", json!(900)),
        ("/rust-suites/owner::a/binary-path", json!("/other/binary")),
        ("/rust-suites/owner::a/status", json!("skipped")),
        ("/rust-suites/owner::a/testcases/case/filter-match/status", json!("mismatch")),
        ("/rust-suites/owner::a/testcases/case/ignored", json!(null)),
        ("/rust-suites/owner::empty", json!(null)),
    ] {
        let mut changed = valid.clone();
        let selected = changed
            .iter_mut()
            .find(|(_, value)| value.pointer(pointer).is_some())
            .context("fixture pointer owner")?;
        *selected.1.pointer_mut(pointer).context("fixture pointer")? = replacement;
        assert!(merge(&plan, &changed).is_err(), "admitted mutation {pointer}");
    }
    let mut wrong_group = valid.clone();
    wrong_group[0].1["rust-suites"]["owner::b"] = valid[1].1["rust-suites"]["owner::b"].clone();
    assert!(
        merge(&plan, &wrong_group).is_err(),
        "admitted a real binary assigned to another group"
    );
    let mut changed = valid;
    changed[0].1["rust-suites"]["unrecognized"] = json!({});
    assert!(merge(&plan, &changed).is_err());
    assert!(plan.merge([(plan.groups[0].label.as_str(), b"{".as_slice())]).is_err());
    Ok(())
}

#[test]
fn discovery_metadata_partitions_preserve_native_records_and_cover_each_binary_once() -> Result<()>
{
    let original = metadata();
    let mut authority = original.clone();
    authority["rust-build-meta"]["non-test-binaries"] =
        json!({"owner": [{"name":"runner","path":"runner"}]});
    authority["producer-extension"] = json!({"retained":true});
    let plan = BinaryDiscoveryPlan::new(&serde_json::to_vec(&authority)?, 2)?;
    let mut union = BTreeSet::new();
    for group in &plan.groups {
        let selected: Value = serde_json::from_slice(&group.metadata)?;
        assert_eq!(selected["rust-build-meta"], authority["rust-build-meta"]);
        assert_eq!(selected["producer-extension"], authority["producer-extension"]);
        for (id, record) in selected["rust-binaries"].as_object().context("binary map")? {
            assert!(union.insert(id.clone()), "binary was assigned twice");
            assert_eq!(record, &authority["rust-binaries"][id]);
        }
    }
    assert_eq!(
        union,
        authority["rust-binaries"].as_object().context("binary map")?.keys().cloned().collect()
    );
    Ok(())
}

#[test]
fn bounded_native_subsets_preserve_exact_selection_and_stable_binary_namespaces() -> Result<()> {
    let bytes = serde_json::to_vec(&metadata())?;
    let selected = ["owner::a".to_string(), "owner::b".to_string()].into_iter().collect();
    let plan = BinaryDiscoveryPlan::selected(&bytes, Some(&selected), 4)?;
    assert_eq!(plan.groups.len(), 1);
    assert_eq!(plan.groups[0].binaries, selected);
    assert_eq!(plan.groups[0].worker_demand(3, 4)?, 4);
    assert!(plan.groups[0].worker_demand(5, 4).is_err());
    let same = BinaryDiscoveryPlan::selected(&bytes, Some(&selected), 40)?;
    assert_eq!(same.groups[0].label, plan.groups[0].label);
    let missing = ["owner::absent".to_string()].into_iter().collect();
    assert!(BinaryDiscoveryPlan::selected(&bytes, Some(&missing), 4).is_err());
    assert!(BinaryDiscoveryPlan::selected(&bytes, Some(&BTreeSet::new()), 4).is_err());
    let actual = ExpectedTestInventory::parse(&merge(&plan, &observations(&plan)?)?)?;
    assert_eq!((actual.case_count(), actual.ignored_case_count(), actual.suites.len()), (2, 1, 2));
    Ok(())
}

#[test]
fn archive_discovery_accepts_only_declared_target_remapping_and_preserves_producer_inputs(
) -> Result<()> {
    let original = serde_json::to_vec(&metadata())?;
    let producer = BinaryDiscoveryPlan::new(&original, 4)?;
    let inputs = producer.groups.iter().map(|group| group.metadata.clone()).collect::<Vec<_>>();
    let admitted = producer.admit_target_remapping(Path::new("/extracted/target"))?;
    assert_eq!(
        admitted.groups.iter().map(|group| group.metadata.clone()).collect::<Vec<_>>(),
        inputs
    );
    let mut observed = observations(&admitted)?;
    for (_, value) in &mut observed {
        for suite in value["rust-suites"].as_object_mut().context("fixture suites")?.values_mut() {
            let path = suite["binary-path"]
                .as_str()
                .context("fixture producer path")?
                .strip_prefix("/compiled/")
                .context("fixture relative binary")?
                .to_owned();
            suite["binary-path"] = json!(format!("/extracted/target/{path}"));
        }
    }
    ExpectedTestInventory::parse(&merge(&admitted, &observed)?)?;
    observed[0].1["rust-suites"]["owner::a"]["binary-path"] = json!("/foreign/executable");
    assert!(merge(&admitted, &observed).is_err());
    let mut escaped = metadata();
    escaped["rust-binaries"]["owner::a"]["binary-path"] = json!("/foreign/executable");
    assert!(BinaryDiscoveryPlan::new(&serde_json::to_vec(&escaped)?, 4)?
        .admit_target_remapping(Path::new("/extracted/target"))
        .is_err());
    Ok(())
}
