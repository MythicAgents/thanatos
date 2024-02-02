### Real Dockerfile for the thanatos payload
# Note: Invoke this Dockerfile from the `Payload_Type/thanatos` directory using the command
#   `docker build -t docker.io/mehrn00/thanatos:v0.1.3 -f .docker/Dockerfile .`
FROM docker.io/library/fedora:35

# Install packages
RUN dnf install -y \
        python3.11 \
        python3-pip \
        python3.11-devel \
        mingw64-gcc \
        mingw32-gcc \
        mingw64-gcc-c++ \
        mingw32-gcc-c++ \
        mingw64-winpthreads-static.noarch \
        mingw32-winpthreads-static.noarch \
        libgcc.i686 \
        glibc-devel.i686 \
        openssl-devel \
        openssl-devel.i686 \
        openssl-static.i686 \
        perl-FindBin \
        perl-File-Compare \
        perl-IPC-Cmd \
        perl-File-Copy \
        perl-lib \
        clang \
        curl \
        make \
        musl-gcc \
        musl-libc-static \
        && dnf clean all

RUN python3.11 -m ensurepip

WORKDIR /tmp

# Install the python dependencies
COPY requirements.txt .
RUN python3.11 -m pip install -r requirements.txt

RUN rm requirements.txt

# Install rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs -o rustup.sh \
    && sh rustup.sh -y \
        --profile minimal \
        --default-toolchain stable \
        -t i686-unknown-linux-gnu \
        -t x86_64-unknown-linux-musl \
        -t i686-unknown-linux-musl \
        -t x86_64-pc-windows-gnu \
        -t i686-pc-windows-gnu

RUN rm -f rustup.sh

ENV PATH=$PATH:/root/.cargo/bin

# Fetch all the dependencies for each toolchain
RUN mkdir /tmp/thanatos
WORKDIR /tmp/thanatos

COPY thanatos/agent_code/ .

RUN cargo fetch --target x86_64-unknown-linux-gnu
RUN cargo fetch --target x86_64-unknown-linux-musl
RUN cargo fetch --target x86_64-pc-windows-gnu
RUN cargo fetch --target i686-unknown-linux-gnu
RUN cargo fetch --target i686-unknown-linux-musl
RUN cargo fetch --target i686-pc-windows-gnu

WORKDIR /
RUN rm -rf /tmp/thanatos

WORKDIR /tmp
RUN curl -L 'https://github.com/mozilla/sccache/releases/download/v0.4.0/sccache-v0.4.0-x86_64-unknown-linux-musl.tar.gz' -o sccache.tar.gz
RUN tar xf sccache.tar.gz
RUN mv sccache-v0.4.0-x86_64-unknown-linux-musl/sccache /usr/bin/sccache
RUN chmod +x /usr/bin/sccache
RUN rm -rf sccache-v0.4.0-x86_64-unknown-linux-musl
RUN rm -f sccache.tar.gz

COPY .docker/config.toml /root/.cargo/config.toml

WORKDIR /Mythic/

COPY main.py main.py
COPY thanatos thanatos
ENV SCCACHE_DIR /Mythic/.cache/sccache
CMD ["python3.11", "main.py"]
