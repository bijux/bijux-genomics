# seqtk Dockerfile (ARM64)
# License: Apache-2.0
FROM ubuntu:24.04
ENV DEBIAN_FRONTEND=noninteractive

ARG VERSION_SEQTK=1.5-r133
ENV VERSION_SEQTK=${VERSION_SEQTK}

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        git \
        ca-certificates \
        build-essential \
        zlib1g-dev && \
    git clone https://github.com/lh3/seqtk.git /opt/seqtk && \
    cd /opt/seqtk && \
    make && \
    mv seqtk /usr/local/bin/seqtk-bin && \
    rm -rf /opt/seqtk /var/lib/apt/lists/* && \
    apt-get purge -y git build-essential && \
    apt-get autoremove -y

# Create a wrapper script for unified Seqtk execution
RUN echo '#!/bin/sh' > /usr/local/bin/seqtk && \
    echo 'if [ "$1" = "--version" ]; then' >> /usr/local/bin/seqtk && \
    echo '    echo "$VERSION_SEQTK"' >> /usr/local/bin/seqtk && \
    echo '    exit 0' >> /usr/local/bin/seqtk && \
    echo 'elif [ "$1" = "--help" ]; then' >> /usr/local/bin/seqtk && \
    echo '    /usr/local/bin/seqtk-bin --help' >> /usr/local/bin/seqtk && \
    echo '    exit 0' >> /usr/local/bin/seqtk && \
    echo 'else' >> /usr/local/bin/seqtk && \
    echo '    /usr/local/bin/seqtk-bin "$@"' >> /usr/local/bin/seqtk && \
    echo 'fi' >> /usr/local/bin/seqtk && \
    chmod +x /usr/local/bin/seqtk

WORKDIR /data
CMD ["seqtk", "--help"]
