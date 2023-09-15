FROM rust:1.72.0

WORKDIR /app

RUN apt update && apt install lld clang -y

COPY . .

# ビルド時にSQLxがオフラインモードで動作するようにする
ENV SQLX_OFFLINE true

# リリースモード(最適化されたバイナリ)でビルド
RUN cargo build --release

# 本番環境で動作するように環境変数を設定
ENV APP_ENVIRONMENT production

ENTRYPOINT ["./target/release/web_prod"]
