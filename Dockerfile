FROM rust:1.58 AS builder

RUN rustup target add x86_64-unknown-linux-musl
RUN apt update && apt install -y musl-tools musl-dev
RUN update-ca-certificates

# Create appuser
ENV USER=openapi-gateway
ENV UID=10001

RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    "${USER}"


WORKDIR /openapi-gateway

COPY ./Cargo.toml .
COPY ./Cargo.lock .
COPY ./src ./src

RUN cargo build --target x86_64-unknown-linux-musl --release

FROM scratch

# Import from builder.
COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group

WORKDIR /openapi-gateway

# Copy our build
COPY --from=builder /openapi-gateway/target/x86_64-unknown-linux-musl/release/openapi-gateway ./
COPY swagger-ui ./swagger-ui
# Use an unprivileged user.
USER openapi-gateway:openapi-gateway

CMD ["/openapi-gateway/openapi-gateway"]