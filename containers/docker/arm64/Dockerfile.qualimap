# qualimap Dockerfile (ARM64)
# License: Apache-2.0
FROM ubuntu:24.04
ENV DEBIAN_FRONTEND=noninteractive

ARG VERSION_QUALIMAP=2.3
ENV VERSION_QUALIMAP=${VERSION_QUALIMAP}

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        curl \
        ca-certificates \
        unzip \
        openjdk-11-jre-headless && \
    curl -L --retry 5 --retry-delay 3 --connect-timeout 20 --max-time 300 \
        -o /tmp/qualimap.zip \
        https://bitbucket.org/kokonech/qualimap/downloads/qualimap_v${VERSION_QUALIMAP}.zip && \
    unzip -q /tmp/qualimap.zip -d /opt && \
    mv /opt/qualimap_v${VERSION_QUALIMAP} /opt/qualimap && \
    rm -rf /tmp/qualimap.zip /var/lib/apt/lists/* && \
    apt-get purge -y curl unzip && \
    apt-get autoremove -y

# Create a wrapper script for unified Qualimap execution
RUN echo '#!/bin/sh' > /usr/local/bin/qualimap && \
    echo 'if [ "$1" = "--version" ]; then' >> /usr/local/bin/qualimap && \
    echo '    echo "$VERSION_QUALIMAP"' >> /usr/local/bin/qualimap && \
    echo '    exit 0' >> /usr/local/bin/qualimap && \
    echo 'elif [ "$1" = "--help" ]; then' >> /usr/local/bin/qualimap && \
    echo '    java -jar /opt/qualimap/qualimap.jar --help' >> /usr/local/bin/qualimap && \
    echo '    exit 0' >> /usr/local/bin/qualimap && \
    echo 'else' >> /usr/local/bin/qualimap && \
    echo '    java -jar /opt/qualimap/qualimap.jar "$@"' >> /usr/local/bin/qualimap && \
    echo 'fi' >> /usr/local/bin/qualimap && \
    chmod +x /usr/local/bin/qualimap

WORKDIR /data
ENTRYPOINT ["/bin/sh", "-c", "qualimap \"$@\""]
CMD ["--help"]
