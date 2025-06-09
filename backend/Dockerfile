# Stage 1: build a static, musl-linked binary
-FROM rust:nightly AS builder
+FROM rust:nightly-slim AS builder
WORKDIR /usr/src/app
RUN rustup target add x86_64-unknown-linux-musl
# Copy only manifest files from the backend subfolder
COPY backend/Cargo.toml backend/Cargo.lock ./
# Fetch dependencies with a dummy main
RUN mkdir src && echo "fn main(){}" > src/main.rs && cargo fetch
# Pull in actual source code
COPY backend/src ./src
RUN cargo build --release --target x86_64-unknown-linux-musl

# Stage 2: distroless runtime
FROM gcr.io/distroless/cc
WORKDIR /app
COPY --from=builder /usr/src/app/target/x86_64-unknown-linux-musl/release/backend .
COPY frontend ./frontend
COPY static  ./static
EXPOSE 8080
CMD ["./backend"]