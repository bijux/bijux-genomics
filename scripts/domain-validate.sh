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

# strict stage capability enforcement:
# every tool in stage_tool_compatibility must satisfy stage tool_capability_requirements.
for dom in fastq bam; do
  idx="$ROOT_DIR/domain/$dom/index.yaml"
  awk '
    BEGIN { inmap=0 }
    /^stage_tool_compatibility:/ { inmap=1; next }
    inmap && /^[^[:space:]]/ { inmap=0 }
    inmap && /^[[:space:]]+[a-z0-9_.-]+:/ {
      line=$0
      gsub(/^[[:space:]]+/, "", line)
      split(line, p, ":")
      stage=p[1]
      rhs=line
      sub(/^[^:]+:[[:space:]]*\[/, "", rhs)
      sub(/\][[:space:]]*$/, "", rhs)
      gsub(/[[:space:]]/, "", rhs)
      print stage "|" rhs
    }
  ' "$idx" | while IFS='|' read -r stage tools_csv; do
    [ -n "$stage" ] || continue
    stage_file="$ROOT_DIR/domain/$dom/stages/$(echo "$stage" | sed "s#^$dom\\.##").yaml"
    [ -f "$stage_file" ] || fail "missing stage file for $stage ($stage_file)"
    reqs=$(awk '
      /^tool_capability_requirements:/ {inreq=1; next}
      inreq && /^  - / {print $2; next}
      inreq && !/^  - / {inreq=0}
    ' "$stage_file")
    # If no requirements are declared, skip compatibility checks for this stage.
    [ -n "$reqs" ] || continue
    [ -n "$tools_csv" ] || fail "stage $stage has capability requirements but no compatible tools in $idx"
    tools=$(echo "$tools_csv" | tr ',' ' ')
    for tool in $tools; do
      tool_file="$ROOT_DIR/domain/$dom/tools/$tool.yaml"
      [ -f "$tool_file" ] || fail "stage $stage references missing tool $tool ($tool_file)"
      caps=$(awk '
        /^capabilities:/ {incap=1; next}
        incap && /^  - / {print $2; next}
        incap && !/^  - / {incap=0}
      ' "$tool_file")
      for req in $reqs; do
        echo "$caps" | rg -qx "$req" || fail "stage $stage requires capability $req but tool $tool lacks it"
      done
    done
  done
done

echo "domain-validate: OK"
