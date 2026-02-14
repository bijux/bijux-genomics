#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../../" && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env
LC_ALL=C
export LC_ALL
./bin/isolate cargo run -p bijux-dna-domain-compiler --bin compile_domain_configs -- --domain-dir domain --configs-dir configs

# Keep legacy root-level CI config paths in sync for policy/tests that still
# assert compatibility with configs/ci/*.toml paths.
cp -f "${ROOT_DIR}/configs/ci/registry/tool_registry.toml" "${ROOT_DIR}/configs/ci/tool_registry.toml"
cp -f "${ROOT_DIR}/configs/ci/registry/tool_registry_vcf.toml" "${ROOT_DIR}/configs/ci/tool_registry_vcf.toml"
cp -f "${ROOT_DIR}/configs/ci/registry/domains.toml" "${ROOT_DIR}/configs/ci/domains.toml"
cp -f "${ROOT_DIR}/configs/ci/stages/stages.toml" "${ROOT_DIR}/configs/ci/stages.toml"
cp -f "${ROOT_DIR}/configs/ci/stages/stages_vcf.toml" "${ROOT_DIR}/configs/ci/stages_vcf.toml"
cp -f "${ROOT_DIR}/configs/ci/params/param_registry.toml" "${ROOT_DIR}/configs/ci/param_registry.toml"
cp -f "${ROOT_DIR}/configs/ci/params/param_registry_vcf.toml" "${ROOT_DIR}/configs/ci/param_registry_vcf.toml"
cp -f "${ROOT_DIR}/configs/ci/tools/images.toml" "${ROOT_DIR}/configs/ci/images.toml"
cp -f "${ROOT_DIR}/configs/ci/tools/required_tools.toml" "${ROOT_DIR}/configs/ci/required_tools.toml"
cp -f "${ROOT_DIR}/configs/ci/tools/required_tools_vcf.toml" "${ROOT_DIR}/configs/ci/required_tools_vcf.toml"
