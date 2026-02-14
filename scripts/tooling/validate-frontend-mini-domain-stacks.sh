#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

OUT_DIR="${OUT_DIR:-${ISO_ROOT:-$ROOT_DIR/artifacts}/domain/frontend-mini-validation}"
mkdir -p "$OUT_DIR"

examples=(
  "fastq_edna_mini"
  "vcf_damage_aware_genotype_mini"
  "vcf_downstream_vcf_full_mini"
  "vcf_downstream_demography_mini"
  "vcf_imputation_mini"
)

for ex in "${examples[@]}"; do
  "$ROOT_DIR/scripts/examples/run.sh" --allow-non-isolate "$ex"
done

python3 - "$ROOT_DIR" "$OUT_DIR" <<'PY'
from pathlib import Path
import hashlib
import json
import subprocess
import sys
try:
    import tomllib
except ModuleNotFoundError:
    import tomli as tomllib

root = Path(sys.argv[1])
out_dir = Path(sys.argv[2])
errors = []
checks = []

examples = [
    ("fastq_edna_mini", root / "examples/fastq/edna-mini"),
    ("vcf_damage_aware_genotype_mini", root / "examples/vcf/damage-aware-genotype-mini"),
    ("vcf_downstream_vcf_full_mini", root / "examples/vcf/downstream-vcf-full-mini"),
    ("vcf_downstream_demography_mini", root / "examples/vcf/downstream-demography-mini"),
    ("vcf_imputation_mini", root / "examples/vcf/imputation-mini"),
]

def load_json(path: Path):
    return json.loads(path.read_text(encoding="utf-8"))

def sha(path: Path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

for ex_id, ex_dir in examples:
    art = root / "artifacts/examples" / ex_id
    required = [
        "plan.json", "explain.json", "report.json",
        "golden_report.json", "run_report.json", "metrics.json", "logs.txt"
    ]
    for name in required:
        if not (art / name).exists():
            errors.append(f"{ex_id}: missing {name}")

    for jf in ("plan.json", "explain.json", "report.json"):
        p = art / jf
        g = ex_dir / "golden" / jf
        if p.exists() and g.exists() and p.read_text(encoding="utf-8") != g.read_text(encoding="utf-8"):
            errors.append(f"{ex_id}: {jf} differs from golden")

    suite = tomllib.loads((ex_dir / "bench-suite.toml").read_text(encoding="utf-8"))
    stages = [str(s).strip() for s in suite.get("stages", [])]
    plan = load_json(art / "plan.json")
    got_stages = [str(s).strip() for s in plan.get("stages", [])]
    for st in stages:
        if st not in got_stages:
            errors.append(f"{ex_id}: stage {st} missing in plan.json stages")

    logs = (art / "logs.txt").read_text(encoding="utf-8") if (art / "logs.txt").exists() else ""
    for key in ("example_id=", "corpus_id=", "mini_supported=", "step1=", "step2=", "step3=", "step4="):
        if key not in logs:
            errors.append(f"{ex_id}: logs.txt missing {key}")

    metrics = load_json(art / "metrics.json")
    for key in ("example_id", "collected_at", "status"):
        if key not in metrics:
            errors.append(f"{ex_id}: metrics.json missing {key}")

    if ex_id.startswith("vcf_"):
        explain = load_json(art / "explain.json")
        report = load_json(art / "report.json")
        for doc_name, payload in (("explain.json", explain), ("report.json", report)):
            cov = payload.get("coverage_regime", {})
            if str(cov.get("selected", "")).strip() not in {"gl", "pseudohaploid", "diploid"}:
                errors.append(f"{ex_id}: {doc_name} coverage_regime.selected invalid")
            for key in ("thresholds_used", "observed_coverage_stats"):
                if key not in cov:
                    errors.append(f"{ex_id}: {doc_name} coverage_regime missing {key}")

    checks.append({
        "example_id": ex_id,
        "artifact_dir": str(art),
        "plan_sha256": sha(art / "plan.json"),
        "explain_sha256": sha(art / "explain.json"),
        "report_sha256": sha(art / "report.json"),
    })

# Coverage regime branch logic behavior (GL + pseudohaploid + diploid)
sim_cases = [
    ("adna_lowcov_capture", "1", "gl"),
    ("adna_lowcov_capture", "6", "pseudohaploid"),
    ("modern_wgs_shotgun", "20", "diploid"),
]
for profile, depth, want in sim_cases:
    proc = subprocess.run(
        [str(root / "scripts/tooling/simulate-coverage-regime.sh"), depth, "--profile", profile],
        text=True,
        capture_output=True,
        check=False,
    )
    if proc.returncode != 0:
        errors.append(f"coverage_regime simulate failed: profile={profile} depth={depth}")
        continue
    payload = json.loads(proc.stdout)
    got = str(payload.get("selected_regime", ""))
    if got != want:
        errors.append(f"coverage_regime mismatch: profile={profile} depth={depth} expected={want} got={got}")

# BAM authenticity consistency (mini/test stack contract-level)
auth_text = (root / "domain/bam/stages/authenticity.yaml").read_text(encoding="utf-8")
tools = []
in_tools = False
for raw in auth_text.splitlines():
    line = raw.rstrip()
    if line.strip().startswith("compatible_tools:"):
        in_tools = True
        continue
    if in_tools:
        if line.startswith("  - "):
            tools.append(line.split("-", 1)[1].strip())
            continue
        if line and not line.startswith(" "):
            break
tools = sorted(str(t).strip() for t in tools if t.strip())
if tools != ["authenticct", "damageprofiler", "pmdtools"]:
    errors.append(f"bam.authenticity compatible_tools mismatch: {tools}")

for fixture in sorted((root / "domain/bam/fixtures/bam.authenticity").glob("*.txt")):
    kv = {}
    for line in fixture.read_text(encoding="utf-8").splitlines():
        if "=" not in line:
            continue
        k, v = line.split("=", 1)
        kv[k.strip()] = v.strip()
    if kv.get("stage") != "bam.authenticity":
        errors.append(f"{fixture}: stage must be bam.authenticity")
    if kv.get("domain") != "bam":
        errors.append(f"{fixture}: domain must be bam")
    if kv.get("expected_outputs") != "contract_artifacts":
        errors.append(f"{fixture}: expected_outputs must be contract_artifacts")
    if kv.get("expected_stdout_patterns") != "contract_ok":
        errors.append(f"{fixture}: expected_stdout_patterns must be contract_ok")

summary = {
    "schema_version": "bijux.frontend.mini_domain_validation.v1",
    "ok": not errors,
    "checks": checks,
    "errors": errors,
}
out = out_dir / "summary.json"
out.write_text(json.dumps(summary, indent=2, sort_keys=True) + "\n", encoding="utf-8")
print(out)
if errors:
    print("frontend mini domain validation: FAILED", file=sys.stderr)
    for e in errors:
        print(f"- {e}", file=sys.stderr)
    raise SystemExit(1)
print("frontend mini domain validation: OK")
PY
