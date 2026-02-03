# seqpurge Dockerfile (ARM64)
# License: Apache-2.0
FROM ubuntu:24.04
ENV DEBIAN_FRONTEND=noninteractive

ARG SEQPURGE_VERSION=2025_05
ENV SEQPURGE_VERSION=${SEQPURGE_VERSION}

RUN apt-get update && apt-get install -y --no-install-recommends \
    g++ qtbase5-dev qt5-qmake libqt5xmlpatterns5-dev libqt5charts5-dev libqt5sql5-mysql \
    libqt5sql5-psql libqt5sql5-sqlite libqt5xml5 libqt5sql5 zlib1g-dev libbz2-dev \
    liblzma-dev libhts-dev libxml2-dev pkg-config git && \
    git clone --recursive https://github.com/imgag/ngs-bits.git /opt/ngs-bits && \
    cd /opt/ngs-bits && \
    git checkout ${SEQPURGE_VERSION} && \
    make build_libs_release -j$(nproc) && \
    make build_tools_release -j$(nproc) && \
    cp bin/SeqPurge /usr/local/bin/seqpurge-bin && \
    cp bin/*.so* /usr/local/lib/ && \
    ldconfig && \
    apt-get purge -y g++ qt5-qmake git pkg-config && \
    apt-get autoremove -y && \
    rm -rf /var/lib/apt/lists/* /opt/ngs-bits

# Create a wrapper script for unified SeqPurge execution
RUN echo '#!/bin/sh' > /usr/local/bin/seqpurge && \
    echo 'if [ "$1" = "--version" ]; then' >> /usr/local/bin/seqpurge && \
    echo '    echo "$SEQPURGE_VERSION"' >> /usr/local/bin/seqpurge && \
    echo '    exit 0' >> /usr/local/bin/seqpurge && \
    echo 'elif [ "$1" = "--help" ]; then' >> /usr/local/bin/seqpurge && \
    echo '    /usr/local/bin/seqpurge-bin --help' >> /usr/local/bin/seqpurge && \
    echo '    exit 0' >> /usr/local/bin/seqpurge && \
    echo 'else' >> /usr/local/bin/seqpurge && \
    echo '    /usr/local/bin/seqpurge-bin "$@"' >> /usr/local/bin/seqpurge && \
    echo 'fi' >> /usr/local/bin/seqpurge && \
    chmod +x /usr/local/bin/seqpurge

WORKDIR /data
ENTRYPOINT ["/bin/sh", "-c", "seqpurge \"$@\""]
CMD ["--help"]
