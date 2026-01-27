# seqpurge Dockerfile (ARM64)
# License: Apache-2.0

# Stage 1: Build ngs-bits core + SeqPurge
FROM ubuntu:24.04 AS builder

ENV DEBIAN_FRONTEND=noninteractive
ARG SEQPURGE_VERSION=2025_05

RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    qtbase5-dev \
    qt5-qmake \
    libqt5xmlpatterns5-dev \
    libqt5charts5-dev \
    libqt5svg5-dev \
    zlib1g-dev \
    libbz2-dev \
    liblzma-dev \
    libhts-dev \
    libxml2-dev \
    pkg-config \
    git \
    ca-certificates && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /opt

RUN git clone --recursive --depth 1 --branch ${SEQPURGE_VERSION} \
        https://github.com/imgag/ngs-bits.git

WORKDIR /opt/ngs-bits

ENV SKIP_TESTS=1
RUN make build_libs_release -j$(nproc) SKIP_TESTS=1 && \
    make build_tools_release -j$(nproc) TOOLS=SeqPurge SKIP_TESTS=1

# sanity check
RUN test -x bin/SeqPurge

# Stage 2: Runtime image
FROM ubuntu:24.04

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update && apt-get install -y --no-install-recommends \
    libqt5core5a \
    libqt5xmlpatterns5 \
    libqt5charts5 \
    libqt5svg5 \
    libqt5sql5 \
    libqt5sql5-sqlite \
    libxml2 \
    zlib1g \
    libbz2-1.0 \
    liblzma5 \
    libhts3 \
    libgomp1 \
    ca-certificates && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /opt/ngs-bits/bin/SeqPurge /usr/local/bin/seqpurge
COPY --from=builder /opt/ngs-bits/bin/*.so* /usr/local/lib/
RUN ldconfig

WORKDIR /data

ENTRYPOINT ["/usr/local/bin/seqpurge"]
CMD ["--help"]
