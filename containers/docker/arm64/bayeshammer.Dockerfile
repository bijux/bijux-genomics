# bayeshammer Dockerfile (ARM64)
# License: Apache-2.0
FROM ubuntu:24.04

ARG  VERSION_SPADES=4.2.0
ENV  VERSION_SPADES=${VERSION_SPADES} \
     DEBIAN_FRONTEND=noninteractive \
     TZ=UTC

# ------------------------------------------------------------------
# 1. Build‑time deps and compilation
# ------------------------------------------------------------------
RUN set -eux; \
    ### Define build-time vs runtime dependencies
    BUILD_DEPS="build-essential cmake wget" && \
    RUNTIME_DEPS="libgomp1 libbz2-dev zlib1g-dev libboost-all-dev python3 python3-pip python3-yaml" && \
    \
    ### Install all dependencies
    apt-get update && \
    apt-get install -y --no-install-recommends \
        $BUILD_DEPS \
        $RUNTIME_DEPS ; \
    \
    ### source + compile
    wget -qO /tmp/spades.tar.gz \
        "https://github.com/ablab/spades/archive/refs/tags/v${VERSION_SPADES}.tar.gz" && \
    tar -xzf /tmp/spades.tar.gz -C /opt && \
    cd /opt/spades-${VERSION_SPADES} && \
    ./spades_compile.sh && \
    cp -r bin/* /usr/local/bin/ && \
    cp -r share/spades /usr/local/share/ && \
    \
    ### tidy up: ONLY purge the build-time dependencies
    cd / && rm -rf /opt/spades-${VERSION_SPADES} /tmp/spades.tar.gz && \
    apt-get purge -y --auto-remove $BUILD_DEPS && \
    apt-get clean && rm -rf /var/lib/apt/lists/*

# ------------------------------------------------------------------
# 2. Thin helper for convenience (optional)
# ------------------------------------------------------------------
RUN printf '%s\n' \
  '#!/usr/bin/env bash' \
  'set -e' \
  'case "$1" in' \
  '  --version) echo "$VERSION_SPADES"; exit 0 ;;' \
  '  --help|-h) exec spades.py --only-error-correction --help ;;' \
  'esac' \
  'exec spades.py --only-error-correction "$@"' \
  > /usr/local/bin/bayeshammer && chmod +x /usr/local/bin/bayeshammer

# ------------------------------------------------------------------
# 3. Default behaviour
# ------------------------------------------------------------------
WORKDIR /data
# **No ENTRYPOINT** – the caller decides whether to run `spades.py`
#   directly or use the helper.
CMD ["bayeshammer", "--help"]
