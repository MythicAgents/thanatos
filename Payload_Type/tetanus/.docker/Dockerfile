### Real Dockerfile for the tetanus payload
# Note: Invoke this Dockerfile from the `Payload_Type/tetanus` directory using the command
#   `docker build -t docker.io/mehrn00/tetanus:v0.1.1 -f .docker/Dockerfile .`
FROM fedora:32

# Install packages
RUN dnf install -y @development-tools \
        glibc.i686 \
        glibc-devel.i686 \
        python3-pip \
        mingw64-gcc \
        mingw64-gcc-c++ \
        mingw32-gcc-c++ \
        mingw64-winpthreads-static.noarch \
        mingw32-winpthreads-static.noarch \
        openssl-static.i686 \
        openssl-static.x86_64 \
        musl-libc.x86_64 \
        musl-libc.i686 \
        musl-gcc \
        zlib-static.i686 \
        zlib-static.x86_64 \
        python3-devel \
        && dnf clean all


# Install the python dependencies
COPY requirements.txt .
RUN pip3 install -r requirements.txt
RUN rm requirements.txt

# Install rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs -o rustup.sh && sh rustup.sh -y
RUN rm -f rustup.sh

# Add cargo and rustup to path
RUN ln -sf /root/.cargo/bin/cargo /usr/local/bin/cargo
RUN ln -sf /root/.cargo/bin/rustup /usr/local/bin/rustup

# Add 32 bit linux toolchain
RUN rustup target install i686-unknown-linux-gnu

# Add musl toolchains for static linking
RUN rustup target install x86_64-unknown-linux-musl
RUN rustup target install i686-unknown-linux-musl

# Add windows toolchains
RUN rustup target install x86_64-pc-windows-gnu
RUN rustup target install i686-pc-windows-gnu

# Fetch dependencies
RUN mkdir -p /tmp/tetanus
COPY agent_code /tmp/tetanus
WORKDIR /tmp/tetanus
RUN cargo fetch
WORKDIR /
RUN rm -rf /tmp/tetanus

ENTRYPOINT ["/Mythic/mythic/payload_service.sh"]
