# vsearch Dockerfile (ARM64)
# License: Apache-2.0
FROM ubuntu:24.04
ENV DEBIAN_FRONTEND=noninteractive

ARG VERSION_VSEARCH=2.28.1
ENV VERSION_VSEARCH=${VERSION_VSEARCH}

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        wget \
        ca-certificates \
        build-essential \
        git \
        autoconf \
        automake \
        libtool \
        pkg-config \
        zlib1g-dev \
        libbz2-dev \
        liblzma-dev && \
    # ── fetch source at the requested tag ────────────────────────────
    git clone --depth 1 --branch v${VERSION_VSEARCH} https://github.com/torognes/vsearch.git /tmp/vsearch-src && \
    cd /tmp/vsearch-src && \
    ./autogen.sh && \
    ./configure --quiet CFLAGS="-O3" && \
    make -j"$(nproc)" && \
    # ── install binary ───────────────────────────────────────────────
    cp bin/vsearch /usr/local/bin/vsearch-bin && \
    strip /usr/local/bin/vsearch-bin && \
    # ── cleanup ──────────────────────────────────────────────────────
    cd / && rm -rf /tmp/vsearch-src && \
    apt-get purge -y git wget ca-certificates build-essential autoconf automake libtool pkg-config && \
    apt-get autoremove -y && \
    rm -rf /var/lib/apt/lists/*

# Create a wrapper script for unified VSEARCH execution
RUN echo '#!/bin/sh' > /usr/local/bin/vsearch && \
    echo 'if [ "$1" = "--version" ]; then' >> /usr/local/bin/vsearch && \
    echo '    echo "$VERSION_VSEARCH"' >> /usr/local/bin/vsearch && \
    echo '    exit 0' >> /usr/local/bin/vsearch && \
    echo 'elif [ "$1" = "--help" ]; then' >> /usr/local/bin/vsearch && \
    echo '    /usr/local/bin/vsearch-bin --help' >> /usr/local/bin/vsearch && \
    echo '    exit 0' >> /usr/local/bin/vsearch && \
    echo 'else' >> /usr/local/bin/vsearch && \
    echo '    /usr/local/bin/vsearch-bin "$@"' >> /usr/local/bin/vsearch && \
    echo 'fi' >> /usr/local/bin/vsearch && \
    chmod +x /usr/local/bin/vsearch

WORKDIR /data
ENTRYPOINT ["/bin/sh", "-c", "vsearch \"$@\""]
CMD ["--help"]
