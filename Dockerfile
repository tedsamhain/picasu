######################
# Frontend builder stage
#
# Must run before the backend build below: the `embed-frontend` feature
# embeds `../gallery-frontend/dist/` into the binary at *compile* time (see
# gallery-backend/src/public/embedded.rs), so the dist directory has to
# already exist when `cargo build` runs.
######################
FROM node:lts AS frontend-builder
WORKDIR /app/gallery-frontend
COPY gallery-frontend/package.json gallery-frontend/package-lock.json ./
RUN npm ci
COPY gallery-frontend ./
RUN npm run build:only

######################
# Backend builder stage
######################
FROM rust:bookworm AS builder

ARG BUILD_TYPE=release
ENV BUILD_TYPE=${BUILD_TYPE}

WORKDIR /app/gallery-backend

RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

COPY gallery-backend/Cargo.lock gallery-backend/Cargo.toml ./
COPY gallery-backend/src ./src

# embed-frontend reads this path relative to the crate root at compile time.
COPY --from=frontend-builder /app/gallery-frontend/dist /app/gallery-frontend/dist

# Single self-contained binary: no separate frontend assets to ship or
# locate at runtime, and no need to control the working directory the
# binary is launched from.
RUN if [ "${BUILD_TYPE}" = "release" ]; then \
    cargo build --release --features embed-frontend --bin urocissa; \
    elif [ "${BUILD_TYPE}" = "debug" ]; then \
    cargo build --features embed-frontend --bin urocissa; \
    else \
    cargo build --profile "${BUILD_TYPE}" --features embed-frontend --bin urocissa; \
    fi

RUN cp /app/gallery-backend/target/${BUILD_TYPE}/urocissa /app/gallery-backend/urocissa

######################
# Runtime stage
######################
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
    ffmpeg \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Fixed in-image storage roots. Bind-mount host directories onto these in
# `compose.yaml`/`docker run -v`, rather than relying on Urocissa's
# own portable/installed-mode autodetection or moving files into a
# user-supplied path at container startup. See docs/CONFIG.md.
ENV UROCISSA_CONFIG_HOME=/config
ENV UROCISSA_DATA_HOME=/data
ENV UROCISSA_IMAGE_HOME=/images
RUN mkdir -p /config /data /images

WORKDIR /app
COPY --from=builder /app/gallery-backend/urocissa ./urocissa

EXPOSE 5673
ENTRYPOINT ["/app/urocissa"]
