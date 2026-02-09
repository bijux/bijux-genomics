# bbduk Dockerfile (ARM64)
# License: Apache-2.0
FROM ubuntu:24.04
ENV DEBIAN_FRONTEND=noninteractive

ARG VERSION_BBDUK=39.08
ENV VERSION_BBDUK=${VERSION_BBDUK}

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        wget ca-certificates openjdk-17-jre-headless && \
    wget -q https://sourceforge.net/projects/bbmap/files/BBMap_${VERSION_BBDUK}.tar.gz -O /tmp/bbmap.tar.gz && \
    tar -xzf /tmp/bbmap.tar.gz -C /opt && \
    rm /tmp/bbmap.tar.gz && \
    apt-get purge -y wget && \
    apt-get autoremove -y && \
    rm -rf /var/lib/apt/lists/*

# Create a wrapper script for unified BBDuk execution
RUN echo '#!/bin/sh' > /usr/local/bin/bbduk && \
    echo '/opt/bbmap/bbduk.sh "$@"' >> /usr/local/bin/bbduk && \
    chmod +x /usr/local/bin/bbduk

WORKDIR /data
ENTRYPOINT ["bbduk"]
CMD ["--help"]
