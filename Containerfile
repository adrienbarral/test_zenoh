# syntax=docker/dockerfile:1

################################################################################
# Create a stage for building the application.

ARG RUST_VERSION=1.82.0
FROM --platform=linux/arm64 rust:${RUST_VERSION}-bullseye AS build
RUN apt-get update && apt-get upgrade -y
RUN apt-get install -y pkg-config openssl libssl-dev ca-certificates
RUN \
    apt-get install ca-certificates && \
    apt-get clean

WORKDIR /app
COPY Cargo.toml Cargo.lock /app/
COPY src /app/src

RUN --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    <<EOF
set -e
ls -all /app
rustup target add aarch64-unknown-linux-gnu
cargo build --locked --release --target aarch64-unknown-linux-gnu
cp ./target/aarch64-unknown-linux-gnu/release/publisher /bin/publisher
cp ./target/aarch64-unknown-linux-gnu/release/subscriber /bin/subscriber
EOF

################################################################################
# Create a new stage for running the application that contains the minimal
# runtime dependencies for the application. This often uses a different base
# image from the build stage where the necessary files are copied from the build
# stage.
#
# The example below uses the debian bullseye image as the foundation for running the app.
# By specifying the "bullseye-slim" tag, it will also use whatever happens to be the
# most recent version of that tag when you build your Dockerfile. If
# reproducibility is important, consider using a digest
# (e.g., debian@sha256:ac707220fbd7b67fc19b112cee8170b41a9e97f703f588b2cdbbcdcecdd8af57).
FROM --platform=linux/arm64 debian:bullseye AS final
RUN apt-get update && apt-get upgrade -y
RUN apt-get install -y pkg-config openssl libssl-dev ca-certificates
RUN apt-get install ca-certificates
RUN apt-get install -y iputils-ping

RUN apt-get clean

# Create a non-privileged user that the app will run under.
# See https://docs.docker.com/develop/develop-images/dockerfile_best-practices/#user
#ARG UID=10001
#RUN adduser \
#    --disabled-password \
#    --gecos "" \
#    --home "/nonexistent" \
#    --shell "/sbin/nologin" \
#    --no-create-home \
#    --uid "${UID}" \
#    appuser
#USER appuser

# Copy the executable from the "build" stage.
COPY --from=build /bin/publisher /bin/
COPY --from=build /bin/subscriber /bin/
