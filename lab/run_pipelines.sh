#!/usr/bin/env sh
set -eu

CONFIG_PATH="${CONFIG_PATH:-lab/config.toml}"
if [ ! -f "$CONFIG_PATH" ]; then
  echo "config not found: $CONFIG_PATH"
  echo "copy lab/config.example.toml to lab/config.toml"
  exit 1
fi

get_value() {
  key="$1"
  grep -E "^${key}[[:space:]]*=" "$CONFIG_PATH" | head -n1 | sed -E 's/^[^=]+=//; s/[\" ]//g'
}

CORPUS_ROOT="${CORPUS_ROOT:-$(get_value corpus_root)}"
RUNNER_KIND="${RUNNER_KIND:-$(get_value runner_kind)}"
OUTPUT_DIR="${OUTPUT_DIR:-$(get_value output_dir)}"

if [ -z "${CORPUS_ROOT}" ]; then
  echo "CORPUS_ROOT is required"
  exit 1
fi
if [ -z "${OUTPUT_DIR}" ]; then
  echo "OUTPUT_DIR is required"
  exit 1
fi

PIPELINE_IDS="${PIPELINE_IDS:-$(get_value pipeline_ids)}"
if [ -z "${PIPELINE_IDS}" ]; then
  echo "PIPELINE_IDS is required"
  exit 1
fi

echo "Runner: ${RUNNER_KIND}"
echo "Corpus: ${CORPUS_ROOT}"
echo "Output: ${OUTPUT_DIR}"
echo "Pipelines: ${PIPELINE_IDS}"

for pipeline in $(echo "$PIPELINE_IDS" | tr "," " "); do
  echo "→ run pipeline ${pipeline}"
  cargo run --bin bijux -- run \
    --pipeline "${pipeline}" \
    --runner "${RUNNER_KIND}" \
    --corpus-root "${CORPUS_ROOT}" \
    --out "${OUTPUT_DIR}"
done
