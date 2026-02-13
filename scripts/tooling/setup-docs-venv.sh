#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../../" && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env
LC_ALL=C
export LC_ALL

DOCS_PY="${DOCS_PY:-python3}"
DOCS_VENV="${DOCS_VENV:-artifacts/docs/.venv}"
DOCS_REQ="${DOCS_REQ:-configs/docs/requirements.txt}"

"${DOCS_PY}" -m venv "${DOCS_VENV}"
"${DOCS_VENV}/bin/pip" install --upgrade pip
"${DOCS_VENV}/bin/pip" install -r "${DOCS_REQ}"
