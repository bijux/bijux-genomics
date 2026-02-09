# seqkit Dockerfile (ARM64)
# License: Apache-2.0
FROM ubuntu:24.04
ENV DEBIAN_FRONTEND=noninteractive

ARG VERSION_SEQKIT=2.8.2
ENV VERSION_SEQKIT=${VERSION_SEQKIT}

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        wget \
        ca-certificates && \
    wget -q https://github.com/shenwei356/seqkit/releases/download/v${VERSION_SEQKIT}/seqkit_linux_arm64.tar.gz -O /tmp/seqkit.tar.gz && \
    tar -xzf /tmp/seqkit.tar.gz -C /usr/local/bin && \
    mv /usr/local/bin/seqkit /usr/local/bin/seqkit-bin && \
    rm -rf /tmp/seqkit.tar.gz /var/lib/apt/lists/* && \
    apt-get purge -y wget ca-certificates && \
    apt-get autoremove -y

# Create a wrapper script for unified SeqKit execution
RUN echo '#!/bin/sh' > /usr/local/bin/seqkit && \
    echo 'if [ "$1" = "--version" ]; then' >> /usr/local/bin/seqkit && \
    echo '    echo "$VERSION_SEQKIT"' >> /usr/local/bin/seqkit && \
    echo '    exit 0' >> /usr/local/bin/seqkit && \
    echo 'elif [ "$1" = "--help" ]; then' >> /usr/local/bin/seqkit && \
    echo '    /usr/local/bin/seqkit-bin --help' >> /usr/local/bin/seqkit && \
    echo '    exit 0' >> /usr/local/bin/seqkit && \
    echo 'else' >> /usr/local/bin/seqkit && \
    echo '    /usr/local/bin/seqkit-bin "$@"' >> /usr/local/bin/seqkit && \
    echo 'fi' >> /usr/local/bin/seqkit && \
    chmod +x /usr/local/bin/seqkit

WORKDIR /data
ENTRYPOINT ["/bin/sh", "-c", "seqkit \"$@\""]
CMD ["--help"]
