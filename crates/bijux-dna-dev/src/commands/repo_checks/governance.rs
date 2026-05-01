use std::collections::BTreeSet;

use anyhow::{Context, Result};

use crate::commands::command_support::{fail, pass, read, regex};
use crate::model::check::{CheckDefinition, CheckOutcome};
use crate::runtime::workspace::Workspace;

pub(crate) fn check_audit_allowlist(
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    let path = workspace.path("audit-allowlist.toml");
    if !path.is_file() {
        return fail(check, "missing audit-allowlist.toml");
    }
    let document: toml::Value = toml::from_str(&read(&path)?)?;
    let rows =
        document.get("advisory").and_then(toml::Value::as_array).cloned().unwrap_or_default();
    let today = chrono::Utc::now().date_naive();
    let id_re = regex(r"^RUSTSEC-\d{4}-\d{4}$")?;
    let mut errors = Vec::new();
    for (index, row) in rows.iter().enumerate() {
        let tag = format!("entry[{index}]");
        let advisory = row.get("id").and_then(toml::Value::as_str).unwrap_or("");
        let why = row.get("why").and_then(toml::Value::as_str).unwrap_or("");
        let expiry = row.get("expiry").and_then(toml::Value::as_str).unwrap_or("");
        let owner = row.get("owner").and_then(toml::Value::as_str).unwrap_or("");
        let link = row.get("link").and_then(toml::Value::as_str).unwrap_or("");
        if !id_re.is_match(advisory) {
            errors.push(format!("{tag}: id must match RUSTSEC-YYYY-NNNN"));
        }
        if why.trim().is_empty() {
            errors.push(format!("{tag}: missing why"));
        }
        if owner.trim().is_empty() {
            errors.push(format!("{tag}: missing owner"));
        }
        if !(link.starts_with("http://") || link.starts_with("https://")) {
            errors.push(format!("{tag}: link must be http(s)"));
        }
        match chrono::NaiveDate::parse_from_str(expiry, "%Y-%m-%d") {
            Ok(expiry_date) if expiry_date >= today => {}
            Ok(_) => errors.push(format!("{tag}: expiry has passed ({expiry})")),
            Err(_) => errors.push(format!("{tag}: expiry must be YYYY-MM-DD")),
        }
    }
    if errors.is_empty() {
        return pass(check, "audit allowlist entries are well-formed and non-expired");
    }
    fail(check, errors.join("\n"))
}

pub(crate) fn check_deny_policy_deviations(
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    let path = workspace.path("configs/rust/deny.deviations.toml");
    if !path.is_file() {
        return fail(check, "missing configs/rust/deny.deviations.toml");
    }
    let document: toml::Value = toml::from_str(&read(&path)?)?;
    let rows =
        document.get("deviation").and_then(toml::Value::as_array).cloned().unwrap_or_default();
    if rows.is_empty() {
        return pass(check, "deny policy deviations governance contract is satisfied");
    }
    let today = chrono::Utc::now().date_naive();
    let mut errors = Vec::new();
    for (index, row) in rows.iter().enumerate() {
        let tag = format!("entry[{index}]");
        let id = row.get("id").and_then(toml::Value::as_str).unwrap_or("").trim();
        let owner = row.get("owner").and_then(toml::Value::as_str).unwrap_or("").trim();
        let reason = row.get("reason").and_then(toml::Value::as_str).unwrap_or("").trim();
        let expiry = row.get("expiry").and_then(toml::Value::as_str).unwrap_or("").trim();
        let review = row.get("review").and_then(toml::Value::as_str).unwrap_or("").trim();
        if id.is_empty() {
            errors.push(format!("{tag}: missing id"));
        }
        if owner.is_empty() {
            errors.push(format!("{tag}: missing owner"));
        }
        if reason.is_empty() {
            errors.push(format!("{tag}: missing reason"));
        }
        match chrono::NaiveDate::parse_from_str(expiry, "%Y-%m-%d") {
            Ok(expiry_date) if expiry_date >= today => {}
            Ok(_) => errors.push(format!("{tag}: expiry has passed ({expiry})")),
            Err(_) => errors.push(format!("{tag}: expiry must be YYYY-MM-DD")),
        }
        if !(review.starts_with("http://") || review.starts_with("https://")) {
            errors.push(format!("{tag}: review must be an http(s) link"));
        } else if !review.contains("bijux-std") {
            errors.push(format!("{tag}: review must reference bijux-std"));
        }
    }
    if errors.is_empty() {
        return pass(check, "deny policy deviations governance contract is satisfied");
    }
    fail(check, errors.join("\n"))
}

pub(crate) fn check_bench_knob_discipline_downstream(
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    let knobs_path = workspace.path("configs/bench/knobs.toml");
    if !knobs_path.is_file() {
        return fail(check, "missing configs/bench/knobs.toml");
    }
    let suites = [
        "examples/vcf/downstream-vcf-full-mini/bench-suite.toml",
        "examples/vcf/downstream-demography-mini/bench-suite.toml",
        "examples/vcf/essential-qc/bench-suite.toml",
        "examples/vcf/imputation-mini/bench-suite.toml",
    ];
    let mut errors = Vec::new();
    for rel in suites {
        let path = workspace.path(rel);
        if !path.is_file() {
            errors.push(format!("missing bench suite: {rel}"));
            continue;
        }
        let data: toml::Value = toml::from_str(&read(&path)?)?;
        if data.get("suite_id").and_then(toml::Value::as_str).unwrap_or("").trim().is_empty() {
            errors.push(format!("{rel}: missing suite_id"));
        }
        let Some(stages) = data.get("stages").and_then(toml::Value::as_array) else {
            errors.push(format!("{rel}: stages must be non-empty list"));
            continue;
        };
        if stages.is_empty()
            || !stages.iter().all(|value| value.as_str().unwrap_or("").starts_with("vcf."))
        {
            errors.push(format!("{rel}: all stages must be vcf.*"));
        }
    }
    let knobs: toml::Value = toml::from_str(&read(&knobs_path)?)?;
    let defaults =
        knobs.get("defaults").and_then(toml::Value::as_table).cloned().unwrap_or_default();
    for key in ["warmup_policy", "repetitions", "capture_cpu", "capture_memory", "capture_io"] {
        if !defaults.contains_key(key) {
            errors.push(format!("configs/bench/knobs.toml defaults missing {key}"));
        }
    }
    if errors.is_empty() {
        return pass(check, "downstream benchmark suites stay bound to governed knobs");
    }
    fail(check, errors.join("\n"))
}

pub(crate) fn check_bench_knobs(
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    let cfg: toml::Value = toml::from_str(&read(&workspace.path("configs/bench/knobs.toml"))?)?;
    let defaults = cfg
        .get("defaults")
        .and_then(toml::Value::as_table)
        .context("configs/bench/knobs.toml missing [defaults]")?;
    let warmup = defaults.get("warmup_policy").and_then(toml::Value::as_str).unwrap_or("");
    if !["none", "once", "per-benchmark"].contains(&warmup) {
        return fail(check, "warmup_policy must be one of none|once|per-benchmark");
    }
    let repetitions =
        defaults.get("repetitions").and_then(toml::Value::as_integer).unwrap_or_default();
    if !(1..=100).contains(&repetitions) {
        return fail(check, "repetitions must be within [1, 100]");
    }
    for key in ["capture_cpu", "capture_memory", "capture_io"] {
        if defaults.get(key).and_then(toml::Value::as_bool).is_none() {
            return fail(check, format!("{key} must be a boolean"));
        }
    }
    pass(check, "bench knobs keep the expected default contract")
}

pub(crate) fn check_benchmark_integrity_policy(
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    let native_ops =
        read(&workspace.path("crates/bijux-dna-dev/src/commands/ops/tooling/acquisition.rs"))?;
    let mut errors = Vec::new();
    if !native_ops.contains("tooling_benchmark_integrity_mini") {
        errors.push("benchmark integrity mini workflow must exist".to_string());
    }
    if !native_ops.contains("benchmarks/integrity-mini") {
        errors.push(
            "benchmark integrity mini workflow must default outputs under benchmarks/integrity-mini/"
                .to_string(),
        );
    }
    if !native_ops.contains("containers/smoke") {
        errors.push(
            "benchmark integrity mini workflow must guard against smoke/benchmark log mixing"
                .to_string(),
        );
    }
    if !native_ops.contains("\"bench\"") || !native_ops.contains("\"fastq\"") {
        errors.push(
            "benchmark integrity mini workflow must invoke bijux-dna bench fastq directly"
                .to_string(),
        );
    }
    let knobs: toml::Value = toml::from_str(&read(&workspace.path("configs/bench/knobs.toml"))?)?;
    let variance =
        knobs.get("variance").and_then(toml::Value::as_table).cloned().unwrap_or_default();
    for key in ["runtime_relative_max", "memory_relative_max", "report_structure_match"] {
        if !variance.contains_key(key) {
            errors.push(format!("configs/bench/knobs.toml [variance] missing `{key}`"));
        }
    }
    let doc = read(&workspace.path("docs/30-operations/BENCHMARK_VARIANCE.md"))?;
    for phrase in ["runtime relative variance", "memory relative variance", "report.html"] {
        if !doc.to_lowercase().contains(&phrase.to_lowercase()) {
            errors.push(format!(
                "docs/30-operations/BENCHMARK_VARIANCE.md missing phrase `{phrase}`"
            ));
        }
    }
    if errors.is_empty() {
        return pass(check, "benchmark integrity rules stay documented and enforced");
    }
    fail(check, errors.join("\n"))
}

pub(crate) fn check_certification_schema_docs(
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    let doc = read(&workspace.path("docs/50-reference/MANIFEST_MIGRATION.md"))?;
    let required = [
        "bijux.certification_bundle.v2",
        "bijux.certification_run_stamp.v1",
        "bijux.frontend.mini_domain_validation.v1",
    ];
    let missing = required.into_iter().filter(|needle| !doc.contains(needle)).collect::<Vec<_>>();
    if missing.is_empty() {
        return pass(check, "certification schema versions stay documented");
    }
    fail(check, format!("missing schema versions in MANIFEST_MIGRATION.md: {}", missing.join(", ")))
}

pub(crate) fn check_clippy_allowlist_expiry(
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    let cfg: toml::Value =
        toml::from_str(&read(&workspace.path("configs/ci/lints/clippy_allowlist.toml"))?)?;
    let entries = cfg.get("allow").and_then(toml::Value::as_array).cloned().unwrap_or_default();
    let today = chrono::Utc::now().date_naive();
    let mut errors = Vec::new();
    for (index, entry) in entries.iter().enumerate() {
        let path = entry.get("path").and_then(toml::Value::as_str).unwrap_or("");
        let lint = entry.get("lint").and_then(toml::Value::as_str).unwrap_or("");
        let expiry = entry.get("expires_on").and_then(toml::Value::as_str).unwrap_or("");
        let reason = entry.get("reason").and_then(toml::Value::as_str).unwrap_or("");
        if [path, lint, expiry, reason].iter().any(|value| value.trim().is_empty()) {
            errors.push(format!("entry #{}: path/lint/expires_on/reason are required", index + 1));
            continue;
        }
        let Ok(expiry_date) = chrono::NaiveDate::parse_from_str(expiry, "%Y-%m-%d") else {
            errors.push(format!("entry #{}: invalid expires_on {}", index + 1, expiry));
            continue;
        };
        if expiry_date < today {
            errors.push(format!(
                "entry #{}: expired allow entry for {} ({})",
                index + 1,
                path,
                lint
            ));
            continue;
        }
        let file = workspace.path(path);
        if !file.is_file() {
            errors.push(format!("entry #{}: missing {}", index + 1, path));
            continue;
        }
        let content = read(&file)?;
        if !content.contains(&format!("#[allow(clippy::{lint})]")) {
            errors.push(format!(
                "entry #{}: allow(clippy::{lint}) not found in {}",
                index + 1,
                path
            ));
        }
    }
    if errors.is_empty() {
        return pass(check, "clippy allowlist expiry contract is satisfied");
    }
    fail(check, errors.join("\n"))
}

pub(crate) fn check_clippy_allowlist_growth(
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    let cfg: toml::Value =
        toml::from_str(&read(&workspace.path("configs/ci/lints/clippy_allowlist.toml"))?)?;
    let baseline: toml::Value =
        toml::from_str(&read(&workspace.path("configs/ci/lints/clippy_allowlist_baseline.toml"))?)?;
    let allow = cfg.get("allow").and_then(toml::Value::as_array).cloned().unwrap_or_default();
    let base_entries =
        baseline.get("entry").and_then(toml::Value::as_array).cloned().unwrap_or_default();
    let max_entries = baseline
        .get("max_entries")
        .and_then(toml::Value::as_integer)
        .unwrap_or_else(|| i64::try_from(base_entries.len()).unwrap_or(i64::MAX));
    let allow_keys = allow
        .iter()
        .filter_map(|entry| {
            Some((
                entry.get("path")?.as_str()?.to_string(),
                entry.get("lint")?.as_str()?.to_string(),
            ))
        })
        .collect::<BTreeSet<_>>();
    let base_keys = base_entries
        .iter()
        .filter_map(|entry| {
            Some((
                entry.get("path")?.as_str()?.to_string(),
                entry.get("lint")?.as_str()?.to_string(),
            ))
        })
        .collect::<BTreeSet<_>>();
    let mut errors = Vec::new();
    let allow_len = i64::try_from(allow.len()).unwrap_or(i64::MAX);
    if allow_len > max_entries {
        errors.push(format!("allowlist grew: {} > max_entries={max_entries}", allow.len()));
    }
    for (path, lint) in allow_keys.difference(&base_keys) {
        errors.push(format!("new allowlist entries are forbidden: {path} :: {lint}"));
    }
    if errors.is_empty() {
        return pass(check, "clippy allowlist did not grow");
    }
    fail(check, errors.join("\n"))
}
