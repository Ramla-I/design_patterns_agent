# Build stage
FROM rust:1.85-slim-bookworm AS builder

RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /build
COPY Cargo.toml Cargo.lock ./
COPY src/ src/

# Build release binary without the translation feature (not needed for analysis)
RUN cargo build --release --no-default-features

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates git && \
    rm -rf /var/lib/apt/lists/*

# Clone the rust stdlib once at image build time so it's baked in.
# --depth 1 --filter=blob:none keeps it small (~250 MB vs multi-GB full clone).
# To analyze a fresher version, mount over /data/rust or rebuild the image.
RUN git clone --depth 1 --filter=blob:none --sparse https://github.com/rust-lang/rust.git /data/rust && \
    cd /data/rust && git sparse-checkout set library && \
    rm -rf /data/rust/.git

COPY --from=builder /build/target/release/design_patterns_agent /usr/local/bin/design_patterns_agent

WORKDIR /workspace
# Output (progress.jsonl, invariants.jsonl, report) goes here.
# Mount a host volume to persist across runs: -v ./runs:/workspace/runs
VOLUME ["/workspace/runs"]

# API key must be provided at runtime:
#   docker run -e ANTHROPIC_API_KEY=sk-... <image>
#   or: docker run <image> --api-key sk-...

ENTRYPOINT ["design_patterns_agent", "analyze", "/data/rust/library"]

# Sensible defaults for the stdlib run.  Override any flag with:
#   docker run <image> --concurrency 10 --token-budget 2000000
CMD [ \
    "--multi-crate", \
    "--concurrency", "5", \
    "--priority-modules", "sync,io,fs,net,cell,collections,thread,process", \
    "--provider", "anthropic", \
    "--model", "claude-sonnet-4-20250514", \
    "--token-budget", "1000000" \
]
