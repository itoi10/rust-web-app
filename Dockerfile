# Chefステージ
# cargo-chefがインストールされたイメージをベースにする
FROM lukemathwalker/cargo-chef:latest-rust-1.72.0 as chef
WORKDIR /app
# lld(リンカー)とclang(Cのコンパイラ)をインストールする
RUN apt update && apt install lld clang -y

# Plannerステージ
# Chefステージのイメージをベースにする
FROM chef as planner
COPY . .
# プロジェクトの依存関係に関する情報をrecipe.jsonに書き出す
RUN cargo chef prepare  --recipe-path recipe.json

# Builderステージ
FROM chef as builder
# Plannerステージで書き出したrecipe.jsonをコピーする
COPY --from=planner /app/recipe.json recipe.json
# 依存関係をビルドしてキャッシュに保存する
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
# SQLxのオフラインモードを有効にする
ENV SQLX_OFFLINE true
# プロジェクトをビルドする
RUN cargo build --release --bin web_prod

# Runtimeステージ
# 軽量なイメージをベースにする
FROM debian:bookworm-slim AS runtime
WORKDIR /app
# opensslとca-certificatesをインストールする
RUN apt-get update -y \
  && apt-get install -y --no-install-recommends openssl ca-certificates \
  # 不要なファイルを削除する
  && apt-get autoremove -y \
  && apt-get clean -y \
  && rm -rf /var/lib/apt/lists/*
# Builderステージでビルドしたバイナリをコピーする
COPY --from=builder /app/target/release/web_prod web_prod
# 設定ファイルをコピーする
COPY configuration configuration
# 本番環境であることを示す環境変数を設定する
ENV APP_ENVIRONMENT production
# バイナリを実行する
ENTRYPOINT ["./web_prod"]
