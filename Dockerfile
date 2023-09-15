FROM rust:1.72.0

WORKDIR /app

RUN apt update && apt install lld clang -y

COPY . .

# ビルド時にSQLxがオフラインモードで動作するようにする
ENV SQLX_OFFLINE true

# リリースモード(最適化されたバイナリ)でビルド
RUN cargo build --release

ENTRYPOINT ["./target/release/web_prod"]
