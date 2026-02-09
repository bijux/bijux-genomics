# kraken2 Dockerfile (ARM64)
# License: Apache-2.0
FROM ubuntu:24.04
ENV DEBIAN_FRONTEND=noninteractive

ARG VERSION_KRAKEN2=2.1.3
ENV VERSION_KRAKEN2=${VERSION_KRAKEN2}

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        wget ca-certificates build-essential zlib1g-dev && \
    wget -q https://github.com/DerrickWood/kraken2/archive/v${VERSION_KRAKEN2}.tar.gz -O /tmp/kraken2.tar.gz && \
    tar -xzf /tmp/kraken2.tar.gz -C /opt && \
    cd /opt/kraken2-${VERSION_KRAKEN2} && \
    ./install_kraken2.sh /usr/local/bin && \
    rm -rf /tmp/kraken2.tar.gz /opt/kraken2-${VERSION_KRAKEN2} && \
    apt-get purge -y wget build-essential && \
    apt-get autoremove -y && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /data
ENTRYPOINT ["/bin/sh", "-c", "kraken2 \"$@\""]
CMD []
