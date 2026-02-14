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
  cat <<'USAGE'
Usage: scripts/tooling/simulate-coverage-regime.sh <mean_depth_x> [--profile <name>]

Profiles:
  adna_lowcov_capture
  adna_lowcov_shotgun
  modern_wgs_capture
  modern_wgs_shotgun
  default
USAGE
}

if [[ "${1:-}" == "--help" || "${1:-}" == "-h" ]]; then
  usage
  exit 0
fi

if [[ $# -lt 1 ]]; then
  usage >&2
  exit 2
fi

coverage="$1"
shift
profile="default"
while [[ $# -gt 0 ]]; do
  case "$1" in
    --profile)
      [[ $# -ge 2 ]] || { echo "missing value for --profile" >&2; exit 2; }
      profile="$2"
      shift 2
      ;;
    *)
      echo "unknown arg: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

python3 - "$ROOT_DIR" "$coverage" "$profile" <<'PY'
from pathlib import Path
import json
import sys
try:
    import tomllib
except ModuleNotFoundError:
    import tomli as tomllib

root = Path(sys.argv[1])
mean_depth = float(sys.argv[2])
profile = sys.argv[3]

cfg = tomllib.loads((root / "configs/runtime/coverage_regimes.toml").read_text(encoding="utf-8"))
dec = cfg["decision"]["coverage_regime"]
base = dec["thresholds"]
profiles = dec.get("profiles", {})

if profile == "default":
    th = base
else:
    if profile not in profiles:
        raise SystemExit(f"unknown profile: {profile}")
    th = profiles[profile]

gl_max = float(th["gl_max_depth"])
pseudo_max = float(th["pseudohaploid_max_depth"])
dip_min = float(th["diploid_min_depth"])

if mean_depth <= gl_max:
    selected = "gl"
    path = ["vcf.call_gl", "vcf.damage_filter", "vcf.gl_propagation", "vcf.impute", "vcf.postprocess"]
elif mean_depth <= pseudo_max:
    selected = "pseudohaploid"
    path = ["vcf.call_pseudohaploid", "vcf.damage_filter", "vcf.impute", "vcf.postprocess"]
elif mean_depth >= dip_min:
    selected = "diploid"
    path = ["vcf.call_diploid", "vcf.damage_filter", "vcf.impute", "vcf.postprocess"]
else:
    selected = "pseudohaploid"
    path = ["vcf.call_pseudohaploid", "vcf.damage_filter", "vcf.impute", "vcf.postprocess"]

out = {
    "decision": "decision.coverage_regime",
    "profile": profile,
    "coverage": {
        "mean_depth_x": mean_depth,
    },
    "thresholds_used": {
        "gl_max_depth": gl_max,
        "pseudohaploid_max_depth": pseudo_max,
        "diploid_min_depth": dip_min,
    },
    "selected_regime": selected,
    "pipeline_path": path,
}
print(json.dumps(out, indent=2, sort_keys=True))
PY
