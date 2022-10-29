# FROM rust:1.62.0 as builder
# WORKDIR /home/rust/src
# COPY . .
# RUN rustup target add x86_64-unknown-linux-musl
# RUN apt update && apt install musl-tools -y
# RUN cargo build --release --target x86_64-unknown-linux-musl
# RUN pwd
# RUN ls -al .
# RUN ls -al ./target/x86_64-unknown-linux-musl/release

# FROM alpine:latest
# WORKDIR /todo-app-server
# COPY --from=builder /home/rust/src/target/x86_64-unknown-linux-musl/release/todo-app-server .
# RUN ls -al
# EXPOSE 8080
# ENTRYPOINT [ "./todo-app-server" ]

FROM rust:1.62.0-alpine as builder
WORKDIR /app
COPY . .
