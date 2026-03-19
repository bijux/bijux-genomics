#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

usage() {
  cat <<'EOF'
Usage: scripts/tooling/certify-domains.sh <fastq|bam|vcf|all>

Env:
  BIJUX_CERT_PRODUCTION_MODE=1  Fail on warnings (default: 0).
  BIJUX_TRUTH_VCF=<path>         Optional truth VCF hook for vcf certification.
EOF
}

[[ $# -eq 1 ]] || {
  usage >&2
  exit 2
}

mode="$1"
case "$mode" in
  fastq|bam|vcf|all) ;;
  *)
    usage >&2
    exit 2
    ;;
esac

cert_root="${ARTIFACT_DIR:-${ISO_ROOT:-$ROOT_DIR/artifacts}}/certification"
mkdir -p "$cert_root"

prod_mode="${BIJUX_CERT_PRODUCTION_MODE:-0}"
if [[ "$prod_mode" == "1" || "$prod_mode" == "true" || "$prod_mode" == "TRUE" ]]; then
  production_mode=1
else
  production_mode=0
fi

if [[ "$mode" == "fastq" || "$mode" == "all" ]]; then
  "$ROOT_DIR/scripts/run.sh" examples run --allow-non-isolate fastq_edna_mini
fi
if [[ "$mode" == "vcf" || "$mode" == "all" ]]; then
  for ex in \
    vcf_damage_aware_genotype_mini \
    vcf_downstream_vcf_full_mini \
    vcf_downstream_demography_mini \
    vcf_imputation_mini; do
    "$ROOT_DIR/scripts/run.sh" examples run --allow-non-isolate "$ex"
  done
fi
if [[ "$mode" == "bam" || "$mode" == "all" ]]; then
  if [[ -f "$ROOT_DIR/assets/golden/smoke-inputs-v1/bam/sample.bam" ]]; then
    "$ROOT_DIR/scripts/run.sh" smoke run bam
  else
    echo "certify-domains: BAM smoke input missing; continuing with fixture-backed BAM certification"
  fi
fi

python3 - "$ROOT_DIR" "$cert_root" "$mode" "$production_mode" "${BIJUX_TRUTH_VCF:-}" <<'PY'
import json
import os
import sys
from pathlib import Path
from datetime import datetime, timezone

root = Path(sys.argv[1])
cert_root = Path(sys.argv[2])
mode = sys.argv[3]
production_mode = sys.argv[4] == "1"
truth_vcf = sys.argv[5].strip()

doc = (root / "docs/50-reference/MANIFEST_MIGRATION.md").read_text(encoding="utf-8")
errors: list[str] = []
warnings: list[str] = []
domains: dict[str, dict] = {}
seen_schema_versions: set[str] = set()

def load_json(path: Path):
    return json.loads(path.read_text(encoding="utf-8"))

def ensure(path: Path, desc: str):
    if not path.exists():
        errors.append(f"{desc} missing: {path}")
        return False
    return True

def flatten_keys(value, prefix=""):
    out = set()
    if isinstance(value, dict):
        for k, v in value.items():
            key = f"{prefix}.{k}" if prefix else str(k)
            out.add(key)
            out |= flatten_keys(v, key)
    elif isinstance(value, list) and value and isinstance(value[0], dict):
        out |= flatten_keys(value[0], f"{prefix}[]")
    return out

def check_schema_doc(schema_version: str):
    seen_schema_versions.add(schema_version)
    if schema_version not in doc:
        errors.append(
            f"schema version `{schema_version}` not documented in docs/50-reference/MANIFEST_MIGRATION.md"
        )

def compare_golden_key_drift(current_path: Path, golden_path: Path, label: str):
    if not ensure(current_path, f"{label} current") or not ensure(golden_path, f"{label} golden"):
        return
    current = load_json(current_path)
    golden = load_json(golden_path)
    current_keys = flatten_keys(current)
    golden_keys = flatten_keys(golden)
    missing = sorted(golden_keys - current_keys)
    if missing:
        errors.append(f"{label}: missing golden keys (key-drift): {missing[:12]}")

def collect_warning_strings(value):
    out = []
    if isinstance(value, dict):
        for k, v in value.items():
            if k.lower().startswith("warn"):
                if isinstance(v, list):
                    out.extend(str(x) for x in v)
                elif v:
                    out.append(str(v))
            out.extend(collect_warning_strings(v))
    elif isinstance(value, list):
        for v in value:
            out.extend(collect_warning_strings(v))
    return out

if mode in ("fastq", "all"):
    ex = root / "examples/fastq/edna-mini"
    art = root / "artifacts/examples/fastq_edna_mini"
    manifest = art / "manifest.json"
    metrics = art / "metrics.json"
    report = art / "report.json"
    ensure(manifest, "fastq manifest")
    ensure(metrics, "fastq metrics")
    ensure(report, "fastq report")
    if manifest.exists():
        m = load_json(manifest)
        check_schema_doc(str(m.get("schema_version", "")))
        for key in ("schema_version", "example_id", "files"):
            if key not in m:
                errors.append(f"fastq manifest missing key `{key}`")
    if metrics.exists():
        m = load_json(metrics)
        for key in ("example_id", "collected_at", "status"):
            if key not in m:
                errors.append(f"fastq metrics missing key `{key}`")
    compare_golden_key_drift(art / "report.json", ex / "golden/report.json", "fastq report")
    fastq_warn = []
    if report.exists():
        fastq_warn.extend(collect_warning_strings(load_json(report)))
    domains["fastq"] = {
        "status": "ok" if not errors else "failed",
        "warnings": sorted(set(fastq_warn)),
        "artifacts_dir": str(art),
    }
    warnings.extend(domains["fastq"]["warnings"])

if mode in ("bam", "all"):
    fixture = root / "crates/bijux-dna-analyze/tests/fixtures/golden_spine/bam-to-bam__adna_shotgun__v1/runs/bam-to-bam__adna_shotgun__v1/artifacts"
    run_manifest = fixture / "run_manifest.json"
    report = fixture / "report.json"
    facts = fixture / "facts.jsonl"
    ensure(run_manifest, "bam run_manifest")
    ensure(report, "bam report")
    ensure(facts, "bam facts")
    if run_manifest.exists():
        m = load_json(run_manifest)
        check_schema_doc(str(m.get("schema_version", "")))
        for key in ("schema_version", "run_id"):
            if key not in m:
                errors.append(f"bam run_manifest missing key `{key}`")
    if report.exists():
        r = load_json(report)
        for key in ("schema_version", "stages"):
            if key not in r:
                errors.append(f"bam report missing key `{key}`")
        check_schema_doc(str(r.get("schema_version", "")))
    if facts.exists():
        line = facts.read_text(encoding="utf-8").splitlines()[0]
        obj = json.loads(line)
        check_schema_doc(str(obj.get("schema_version", "")))
        if "metrics" not in obj:
            errors.append("bam facts.jsonl missing metrics object")
    domains["bam"] = {
        "status": "ok" if not errors else "failed",
        "warnings": [],
        "artifacts_dir": str(fixture),
    }

if mode in ("vcf", "all"):
    vcf_examples = [
        ("vcf_damage_aware_genotype_mini", root / "examples/vcf/damage-aware-genotype-mini"),
        ("vcf_downstream_vcf_full_mini", root / "examples/vcf/downstream-vcf-full-mini"),
        ("vcf_downstream_demography_mini", root / "examples/vcf/downstream-demography-mini"),
        ("vcf_imputation_mini", root / "examples/vcf/imputation-mini"),
    ]
    vcf_warn = []
    for ex_id, ex_dir in vcf_examples:
        art = root / "artifacts/examples" / ex_id
        report = art / "report.json"
        explain = art / "explain.json"
        metrics = art / "metrics.json"
        manifest = art / "manifest.json"
        ensure(report, f"{ex_id} report")
        ensure(explain, f"{ex_id} explain")
        ensure(metrics, f"{ex_id} metrics")
        ensure(manifest, f"{ex_id} manifest")
        compare_golden_key_drift(report, ex_dir / "golden/report.json", f"{ex_id} report")
        compare_golden_key_drift(explain, ex_dir / "golden/explain.json", f"{ex_id} explain")
        if report.exists():
            payload = load_json(report)
            schema_version = str(payload.get("schema_version", "")).strip()
            if schema_version:
                check_schema_doc(schema_version)
            elif manifest.exists():
                ms = str(load_json(manifest).get("schema_version", "")).strip()
                if ms:
                    check_schema_doc(ms)
                else:
                    errors.append(f"{ex_id}: neither report nor manifest declares schema_version")
            else:
                errors.append(f"{ex_id}: neither report nor manifest declares schema_version")
            vcf_warn.extend(collect_warning_strings(payload))
    truth_hook = {
        "enabled": bool(truth_vcf),
        "truth_vcf": truth_vcf or None,
        "status": "skipped",
        "details": "no truth VCF provided",
    }
    if truth_vcf:
        truth_path = Path(truth_vcf)
        if not truth_path.exists():
            errors.append(f"truth VCF path does not exist: {truth_vcf}")
            truth_hook["status"] = "failed"
            truth_hook["details"] = "path missing"
        else:
            truth_hook["status"] = "ok"
            truth_hook["details"] = "hook enabled; downstream concordance metrics must be consumed from imputation outputs"
    domains["vcf"] = {
        "status": "ok" if not errors else "failed",
        "warnings": sorted(set(vcf_warn)),
        "truth_concordance_hook": truth_hook,
        "artifacts_dir": str(root / "artifacts/examples"),
    }
    warnings.extend(domains["vcf"]["warnings"])

if production_mode and warnings:
    errors.append(f"production mode forbids warnings; found {len(warnings)} warning entries")

stamp = {
    "schema_version": "bijux.certification_run_stamp.v1",
    "mode": "production" if production_mode else "non_production",
    "relaxed_thresholds": not production_mode,
    "generated_at_utc": datetime.now(timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z"),
}

bundle = {
    "schema_version": "bijux.certification_bundle.v2",
    "generated_at_utc": stamp["generated_at_utc"],
    "mode": stamp["mode"],
    "relaxed_thresholds": stamp["relaxed_thresholds"],
    "domains": domains,
    "golden_drift_policy": {
        "mode": "schema_and_required_keys_only",
        "exact_metric_values_compared": False,
    },
    "artifact_schema_versions_seen": sorted(v for v in seen_schema_versions if v),
    "errors": errors,
    "warnings": sorted(set(warnings)),
    "status": "ok" if not errors else "failed",
}

cert_root.mkdir(parents=True, exist_ok=True)
(cert_root / "run_stamp.json").write_text(json.dumps(stamp, indent=2, sort_keys=True) + "\n", encoding="utf-8")
(cert_root / "certification_bundle.json").write_text(json.dumps(bundle, indent=2, sort_keys=True) + "\n", encoding="utf-8")

if errors:
    print("certification: FAILED", file=sys.stderr)
    for err in errors:
        print(f"- {err}", file=sys.stderr)
    raise SystemExit(1)
print("certification: OK")
PY

echo "certify-domains: OK (${cert_root}/certification_bundle.json)"
