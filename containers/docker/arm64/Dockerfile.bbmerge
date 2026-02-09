# bbmerge Dockerfile (ARM64)
# License: Apache-2.0
FROM ubuntu:24.04
ENV DEBIAN_FRONTEND=noninteractive

# Define the version of BBTools to install
ARG VERSION_BBMERGE=39.01
ENV VERSION_BBMERGE=${VERSION_BBMERGE}

# Install Java, download and extract BBTools, then clean up
RUN apt-get update && apt-get install -y --no-install-recommends \
    openjdk-11-jre-headless \
    wget && \
    wget -q https://downloads.sourceforge.net/project/bbmap/BBMap_${VERSION_BBMERGE}.tar.gz -O bbmap.tar.gz && \
    tar -xzvf bbmap.tar.gz -C /opt && \
    rm bbmap.tar.gz && \
    apt-get purge -y wget && \
    apt-get autoremove -y && \
    rm -rf /var/lib/apt/lists/*

# Create a wrapper script for unified BBMerge execution compatible with the test script
RUN echo '#!/bin/sh' > /usr/local/bin/bbmerge && \
    echo 'if [ "$1" = "--version" ]; then' >> /usr/local/bin/bbmerge && \
    # BBTools prints version information to stderr, so redirect it to stdout
    echo '    /opt/bbmap/bbmerge.sh --version 2>&1' >> /usr/local/bin/bbmerge && \
    echo 'else' >> /usr/local/bin/bbmerge && \
    # Pass all other arguments to the original script
    echo '    /opt/bbmap/bbmerge.sh "$@"' >> /usr/local/bin/bbmerge && \
    echo 'fi' >> /usr/local/bin/bbmerge && \
    chmod +x /usr/local/bin/bbmerge

WORKDIR /data
ENTRYPOINT ["bbmerge"]
CMD ["--help"]
