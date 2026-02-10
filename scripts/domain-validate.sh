#!/bin/sh
set -eu

ROOT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)

fail() {
  echo "domain-validate: $*" >&2
  exit 1
}

require_file() {
  [ -f "$1" ] || fail "missing required file: $1"
}

require_key() {
  key="$1"
  file="$2"
  rg -q "^${key}:" "$file" || fail "missing key '${key}' in $file"
}

require_file "$ROOT_DIR/domain/fastq/stages/_schema.yaml"
require_file "$ROOT_DIR/domain/bam/stages/_schema.yaml"
require_file "$ROOT_DIR/domain/fastq/tools/_schema.yaml"
require_file "$ROOT_DIR/domain/bam/tools/_schema.yaml"
require_file "$ROOT_DIR/domain/fastq/index.yaml"
require_file "$ROOT_DIR/domain/bam/index.yaml"

for f in "$ROOT_DIR"/domain/fastq/stages/*.yaml "$ROOT_DIR"/domain/bam/stages/*.yaml; do
  b=$(basename "$f")
  [ "$b" = "_schema.yaml" ] && continue
  require_key stage_id "$f"
  require_key inputs "$f"
  require_key outputs "$f"
  require_key required_inputs "$f"
  require_key required_outputs "$f"
  require_key tool_capability_requirements "$f"
  require_key invariants "$f"
  require_key planned_out_of_scope "$f"
done

for f in "$ROOT_DIR"/domain/fastq/tools/*.yaml "$ROOT_DIR"/domain/bam/tools/*.yaml; do
  b=$(basename "$f")
  [ "$b" = "_schema.yaml" ] && continue
  for k in tool_id upstream versioning_strategy default_version install_kind license stage_ids expected_artifacts metrics_schema_id version_cmd help_cmd comparability; do
    require_key "$k" "$f"
  done
  # ensure tool references at least one known stage
  tool_stages=$(awk '/^stage_ids:/{flag=1; next} flag && /^  - /{print $2} flag && !/^  - /{flag=0}' "$f" || true)
  [ -n "$tool_stages" ] || true
done

# domain index should list all local stage and tool ids.
for dom in fastq bam; do
  stage_list=$(awk -F'"' '/^stage_id:/{print $2}' "$ROOT_DIR"/domain/$dom/stages/*.yaml | sort -u)
  tool_list=$(awk -F'"' '/^tool_id:/{print $2}' "$ROOT_DIR"/domain/$dom/tools/*.yaml | sort -u)
  idx="$ROOT_DIR/domain/$dom/index.yaml"
  for s in $stage_list; do
    rg -q "^  - ${s}$" "$idx" || fail "$idx missing stage id $s"
  done
  for t in $tool_list; do
    rg -q "^  - ${t}$" "$idx" || fail "$idx missing tool id $t"
  done
done

echo "domain-validate: OK"
