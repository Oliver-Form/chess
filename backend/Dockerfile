# Stage 1: build the Rust binary
FROM rust:1.71 AS builder
WORKDIR /usr/src/app
# copy only manifest, fetch deps
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main(){}" > src/main.rs \
    && cargo fetch

# copy your code and build in release
COPY . .
RUN cargo build --release

# Stage 2: skinny runtime image
FROM debian:buster-slim
RUN apt-get update \
  && apt-get install -y ca-certificates \
  && rm -rf /var/lib/apt/lists/*
WORKDIR /app
# copy the compiled binary
COPY --from=builder /usr/src/app/target/release/backend .
# copy your frontend & static folders so Warp can serve them
COPY ../frontend ./frontend
COPY ../static  ./static

EXPOSE 8080
CMD ["./backend"]

# 