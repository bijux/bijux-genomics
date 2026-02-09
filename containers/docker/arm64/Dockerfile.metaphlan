# metaphlan Dockerfile (ARM64)
# License: Apache-2.0
FROM ubuntu:24.04

ARG VERSION_METAPHLAN=4.1.1
ENV VERSION_METAPHLAN=${VERSION_METAPHLAN} \
    DEBIAN_FRONTEND=noninteractive

# 1) OS‐level build deps for pysam, h5py, biopython, etc.
RUN set -eux; \
    apt-get update && \
    apt-get install -y --no-install-recommends \
      python3 \
      python3-pip \
      python3-dev \
      build-essential \
      pkg-config \
      libbz2-dev \
      liblzma-dev \
      libcurl4-gnutls-dev \
      libssl-dev \
      zlib1g-dev \
      libhdf5-dev && \
    rm -rf /var/lib/apt/lists/*

# 2) Install MetaPhlAn and its Python deps from PyPI
RUN set -eux; \
    pip3 install --no-cache-dir --break-system-packages metaphlan==${VERSION_METAPHLAN}

# 3) Rename the real script, leave a thin wrapper at `metaphlan`
RUN set -eux; \
    mv /usr/local/bin/metaphlan /usr/local/bin/metaphlan-bin && \
    printf '%s\n' \
      '#!/bin/sh' \
      'case "$1" in' \
      '  --version) echo "${VERSION_METAPHLAN}"; exit 0;;' \
      '  --help)    exec /usr/local/bin/metaphlan-bin --help;;' \
      '  *)         exec /usr/local/bin/metaphlan-bin "$@";; esac' \
      > /usr/local/bin/metaphlan && \
    chmod +x /usr/local/bin/metaphlan

WORKDIR /data
ENTRYPOINT ["metaphlan"]
CMD ["--help"]
