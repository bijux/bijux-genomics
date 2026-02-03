# trimmomatic Dockerfile (ARM64)
# License: Apache-2.0
FROM ubuntu:24.04
ENV DEBIAN_FRONTEND=noninteractive

ARG VERSION_TRIMMOMATIC=0.39
ENV VERSION_TRIMMOMATIC=${VERSION_TRIMMOMATIC}

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        wget \
        ca-certificates \
        unzip \
        openjdk-11-jre-headless && \
    wget -q http://www.usadellab.org/cms/uploads/supplementary/Trimmomatic/Trimmomatic-${VERSION_TRIMMOMATIC}.zip -O /tmp/trimmomatic.zip && \
    unzip -q /tmp/trimmomatic.zip -d /opt && \
    mv /opt/Trimmomatic-${VERSION_TRIMMOMATIC}/trimmomatic-${VERSION_TRIMMOMATIC}.jar /opt/trimmomatic.jar && \
    rm -rf /tmp/trimmomatic.zip /var/lib/apt/lists/* && \
    apt-get purge -y wget unzip && \
    apt-get autoremove -y

# Create a wrapper script for unified Trimmomatic execution
# Maps --version to -version and normalizes --help exit status for AxiomFlow tools
RUN echo '#!/bin/sh' > /usr/local/bin/trimmomatic && \
    echo 'if [ "$1" = "--version" ]; then' >> /usr/local/bin/trimmomatic && \
    echo '    java -jar /opt/trimmomatic.jar -version' >> /usr/local/bin/trimmomatic && \
    echo '    exit 0' >> /usr/local/bin/trimmomatic && \
    echo 'elif [ "$1" = "--help" ]; then' >> /usr/local/bin/trimmomatic && \
    echo '    java -jar /opt/trimmomatic.jar --help' >> /usr/local/bin/trimmomatic && \
    echo '    exit 0' >> /usr/local/bin/trimmomatic && \
    echo 'else' >> /usr/local/bin/trimmomatic && \
    echo '    java -jar /opt/trimmomatic.jar "$@"' >> /usr/local/bin/trimmomatic && \
    echo 'fi' >> /usr/local/bin/trimmomatic && \
    chmod +x /usr/local/bin/trimmomatic

WORKDIR /data
ENTRYPOINT ["/bin/sh", "-c", "trimmomatic \"$@\""]
CMD ["--help"]
