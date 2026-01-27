# sortmerna Dockerfile (ARM64)
# License: Apache-2.0
FROM ubuntu:24.04

ARG VERSION_SORTMERNA=4.3.7
ARG VERSION_ROCKSDB=6.20.3
ENV VERSION_SORTMERNA=${VERSION_SORTMERNA} \
    VERSION_ROCKSDB=${VERSION_ROCKSDB} \
    DEBIAN_FRONTEND=noninteractive

# 1) Build deps + pip‐installed CMake>=3.21
RUN set -e; \
    apt-get update && \
    apt-get install -y --no-install-recommends \
      wget \
      ca-certificates \
      build-essential \
      zlib1g-dev \
      libbz2-dev \
      python3 \
      python3-pip && \
    pip3 install --no-cache-dir --break-system-packages "cmake>=3.21" && \
    rm -rf /var/lib/apt/lists/*

# 2) concurrentqueue.h for SortMeRNA
RUN wget -qO /usr/local/include/concurrentqueue.h \
    https://raw.githubusercontent.com/cameron314/concurrentqueue/master/concurrentqueue.h

# 3) Build RocksDB from source
RUN set -e; \
    wget -qO /tmp/rocksdb.tar.gz \
      https://github.com/facebook/rocksdb/archive/refs/tags/v${VERSION_ROCKSDB}.tar.gz && \
    tar -xf /tmp/rocksdb.tar.gz -C /opt && \
    cd /opt/rocksdb-${VERSION_ROCKSDB} && \
    sed -i '1i#include <cstdint>' db/compaction/compaction_iteration_stats.h && \
    cmake -S . -B build \
      -DCMAKE_BUILD_TYPE=Release \
      -DCMAKE_CXX_FLAGS="-Wno-error=redundant-move -include cstdint -include system_error" \
      -DWITH_LZ4=OFF \
      -DWITH_ZSTD=OFF \
      -DWITH_GFLAGS=OFF \
      -DWITH_SNAPPY=OFF && \
    cmake --build build -j"$(nproc)" && \
    cmake --install build && \
    rm -rf /tmp/rocksdb.tar.gz

# 4) Build SortMeRNA from source (with -pthread flags)
RUN set -e; \
    wget -qO /tmp/sortmerna.tar.gz \
      https://github.com/biocore/sortmerna/archive/v${VERSION_SORTMERNA}.tar.gz && \
    tar -xf /tmp/sortmerna.tar.gz -C /opt && \
    cd /opt/sortmerna-${VERSION_SORTMERNA} && \
    cmake -S . -B build \
      -DCMAKE_BUILD_TYPE=Release \
      -DCMAKE_PREFIX_PATH=/usr/local \
      -DTHREADS_PREFER_PTHREAD_FLAG=ON \
      -DCMAKE_C_FLAGS="-pthread" \
      -DCMAKE_CXX_FLAGS="-pthread -include cstdint" \
      -DCMAKE_EXE_LINKER_FLAGS="-pthread" && \
    cmake --build build -j"$(nproc)" && \
    cmake --install build && \
    mv /usr/local/bin/sortmerna /usr/local/bin/sortmerna-bin && \
    rm -rf /tmp/sortmerna.tar.gz

# 5) Cleanup build-time tools
RUN set -e; \
    python3 -m pip uninstall -y --break-system-packages cmake && \
    apt-get purge -y --auto-remove \
      python3-pip \
      python3 \
      wget \
      ca-certificates \
      build-essential && \
    rm -rf /var/lib/apt/lists/*

# 6) Thin wrapper so “sortmerna --version” + “--help” work
RUN printf '%s\n' \
  '#!/bin/sh' \
  'case "$1" in' \
  '  --version) echo "${VERSION_SORTMERNA}"; exit 0;;' \
  '  --help)    /usr/local/bin/sortmerna-bin --help; exit 0;;' \
  '  *)         exec /usr/local/bin/sortmerna-bin "$@";; esac' \
  > /usr/local/bin/sortmerna && chmod +x /usr/local/bin/sortmerna

WORKDIR /data
ENTRYPOINT ["sortmerna"]
CMD ["--help"]
