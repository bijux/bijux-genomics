#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
ROOT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
cd "$ROOT_DIR"

if ! ./bin/require-isolate >/dev/null 2>&1; then
  exec ./bin/isolate "$0" "$@"
fi

CONFIG_PATH="${CONFIG_PATH:-configs/lab/config.toml}"
if [ ! -f "$CONFIG_PATH" ]; then
  echo "config not found: $CONFIG_PATH"
  echo "copy configs/lab/config_example.toml to configs/lab/config.toml"
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

echo "Runner: ${RUNNER_KIND}"
echo "Corpus: ${CORPUS_ROOT}"
echo "Output: ${OUTPUT_DIR}"

echo "→ run FASTQ benchmark"
cargo run --bin bijux-dna -- bench fastq \
  --runner "${RUNNER_KIND}" \
  --corpus-root "${CORPUS_ROOT}" \
  --out "${OUTPUT_DIR}"

echo "→ run BAM benchmark"
cargo run --bin bijux-dna -- bench bam \
  --runner "${RUNNER_KIND}" \
  --corpus-root "${CORPUS_ROOT}" \
  --out "${OUTPUT_DIR}"
