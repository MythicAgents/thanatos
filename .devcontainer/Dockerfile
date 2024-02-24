FROM ubuntu:latest

ARG username=user
ARG userUid=1000
ARG userGid=1000
ARG upgradePackages=false
ARG goversion=1.22.0
ARG golangcilintversion=1.56.2
ARG shell=fish
ARG extraPackages
ARG protocversion=25.3

# Update apt cache
RUN apt-get update -y

# Create user
RUN groupadd -g ${userGid} ${username}
RUN useradd -m \
    -u ${userUid} \
    -g ${userGid} \
    ${username}

# Install base packages
RUN DEBIAN_FRONTEND=noninteractive apt-get install -y \
    curl \
    sudo \
    git \
    gcc \
    gnupg2 \
    mingw-w64 \
    pkg-config \
    llvm \
    unzip \
    lcov \
    libssl-dev

# Install Golang
WORKDIR /tmp
RUN curl -L "https://go.dev/dl/go${goversion}.linux-amd64.tar.gz" -o go${goversion}.linux-amd64.tar.gz
RUN rm -rf /usr/local/go
RUN tar -C /usr/local -xzf go${goversion}.linux-amd64.tar.gz

RUN grep -qxF 'export PATH=$PATH:/usr/local/go/bin' /home/${username}/.profile \
    || echo 'export PATH=$PATH:/usr/local/go/bin' >> /home/${username}/.profile

RUN rm -f go${goversion}.linux-amd64.tar.gz

# Install golangci-lint
USER ${username}
RUN mkdir -p /home/${username}/go/bin

RUN curl -sSfL https://raw.githubusercontent.com/golangci/golangci-lint/master/install.sh \
    | sh -s -- -b /home/${username}/go/bin v${golangcilintversion}

# Install go cover
RUN /usr/local/go/bin/go install golang.org/x/tools/cmd/cover@latest
RUN /usr/local/go/bin/go install github.com/jandelgado/gcov2lcov@latest

RUN grep -qxF 'export PATH=$PATH:/home/${username}/go/bin' /home/${username}/.profile \
    || echo 'export PATH=$PATH:/home/${username}/go/bin' >> /home/${username}/.profile

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs -o rustup.sh \
    && sh rustup.sh -y \
    --profile minimal \
    --default-toolchain stable \
    -t x86_64-unknown-linux-gnu \
    -t x86_64-pc-windows-gnu \
    -t i686-unknown-linux-gnu \
    -t i686-pc-windows-gnu

RUN grep -qxF '. "$HOME/.cargo/env"' /home/${username}/.profile \
    || echo '. "$HOME/.cargo/env"' >> /home/${username}/.profile

RUN rm -f rustup.sh

# Install nightly
RUN $HOME/.cargo/bin/rustup install nightly

# Install rustfmt, clippy and llvm tools
RUN $HOME/.cargo/bin/rustup component add rustfmt clippy llvm-tools-preview

# Install rustfilt and grcov
RUN $HOME/.cargo/bin/cargo install grcov rustfilt

USER 0

# Install protoc
RUN mkdir -p /tmp/protoc
WORKDIR /tmp/protoc
RUN curl -L https://github.com/protocolbuffers/protobuf/releases/download/v${protocversion}/protoc-${protocversion}-linux-x86_64.zip \
    -o protoc.zip
RUN unzip protoc.zip
RUN mv bin/protoc /usr/local/bin
RUN chmod 755 /usr/local/bin/protoc
WORKDIR /tmp
RUN rm -rf /tmp/protoc

# Ensure zsh is installed if the shell is zsh
RUN if [ "${shell}" = "zsh" ]; then apt-get install -y zsh; fi

# Ensure fish is installed if the shell is fish
RUN if [ "${shell}" = "fish" ]; then apt-get install -y fish; fi

# Set the shell
RUN chsh -s /usr/bin/${shell} ${username}

# Add the user to sudoers
RUN echo "${username} ALL=(ALL) NOPASSWD:ALL" >> /etc/sudoers

# Install any extra packages
RUN if [ "${extraPackages}" != "" ]; then DEBIAN_FRONTEND=noninteractive apt-get install -y ${extraPackages}; fi

USER ${username}

# Setup PATH if fish is the shell. This needs to be in the context of the user
RUN if [ "${shell}" = "fish" ]; then \
    /usr/bin/fish -c 'set -U fish_user_paths /usr/local/go/bin $fish_user_paths' \
    && /usr/bin/fish -c 'set -U fish_user_paths $HOME/.cargo/bin $fish_user_paths' \
    && /usr/bin/fish -c 'set -U fish_user_paths $HOME/go/bin $fish_user_paths' \
    ; fi
