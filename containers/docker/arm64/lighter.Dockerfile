# lighter Dockerfile (ARM64)
# License: Apache-2.0
FROM ubuntu:24.04

ARG  VERSION_LIGHTER=1.1.2
ENV  VERSION_LIGHTER=${VERSION_LIGHTER} \
     DEBIAN_FRONTEND=noninteractive \
     TZ=UTC

# ------------------------------------------------------------------
# 1. Build Lighter
# ------------------------------------------------------------------
RUN set -eux; \
    ### Define build-time vs runtime dependencies
    BUILD_DEPS="build-essential curl" && \
    RUNTIME_DEPS="zlib1g-dev ca-certificates" && \
    \
    ### Install all dependencies
    apt-get update && \
    apt-get install -y --no-install-recommends \
        $BUILD_DEPS \
        $RUNTIME_DEPS ; \
    \
    ### source + compile
    curl -L --retry 5 --retry-delay 3 --connect-timeout 20 --max-time 300 \
        -o /tmp/lighter.tar.gz \
        "https://github.com/mourisl/Lighter/archive/refs/tags/v${VERSION_LIGHTER}.tar.gz" && \
    tar -xzf /tmp/lighter.tar.gz -C /opt && \
    cd /opt/Lighter-${VERSION_LIGHTER} && \
    # fix GCC‑12 strictness
    sed -i '1i#include <string.h>' main.cpp && \
    sed -r -i '/char[[:space:]]+code\[128\]/,/};/d' main.cpp && \
    sed -i '/code\[65\]/i static char code[128]; memset(code,(signed char)-1,sizeof(code));' main.cpp && \
    make -j"$(nproc)" CXXFLAGS="-Wall -O3 -std=c++03 -Wno-narrowing" && \
    install -m 0755 lighter /usr/local/bin/lighter-bin && \
    \
    ### tidy up: ONLY purge the build-time dependencies
    cd / && rm -rf /opt/Lighter-${VERSION_LIGHTER} /tmp/lighter.tar.gz && \
    apt-get purge -y --auto-remove $BUILD_DEPS && \
    apt-get clean && rm -rf /var/lib/apt/lists/*

# ------------------------------------------------------------------
# 2. Thin wrapper (optional)
# ------------------------------------------------------------------
RUN printf '%s\n' \
  '#!/usr/bin/env bash' \
  'set -e' \
  'case "$1" in' \
  '  --version) echo "$VERSION_LIGHTER"; exit 0 ;;' \
  '  --help|-h) exec /usr/local/bin/lighter-bin --help ;;' \
  'esac' \
  'exec /usr/local/bin/lighter-bin "$@"' \
  > /usr/local/bin/lighter && chmod +x /usr/local/bin/lighter

WORKDIR /data
# **NO ENTRYPOINT** – the caller supplies the command
CMD ["lighter", "--help"]
