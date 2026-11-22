FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends \
    gcc-arm-linux-gnueabi binutils-arm-linux-gnueabi gcc make libc6-dev libc6-dev-armel-cross \
    bc bison flex libssl-dev libelf-dev device-tree-compiler python3 ca-certificates \
    && rm -rf /var/lib/apt/lists/*
