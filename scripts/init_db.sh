#!/usr/bin/env bash
set -x
set -eo pipefail

# psql コマンドがインストールされているか確認
if ! [ -x "$(command -v psql)" ]; then
  # psql がインストールされていない場合、エラーメッセージを出力
  echo >&2 "Error: psql is not installed."
  exit 1
fi

# sqlx コマンドがインストールされているか確認
if ! [ -x "$(command -v sqlx)" ]; then
  # sqlx がインストールされていない場合、エラーメッセージを出力
  echo >&2 "Error: sqlx is not installed."
  echo >&2 "Use:"
  echo >&2 "    cargo install sqlx-cli --no-default-features --features postgres"
  echo >&2 "to install it."
  exit 1
fi


# カスタムユーザーが設定されているか確認、デフォルトは 'postgres'
DB_USER=${POSTGRES_USER:=postgres}
# カスタムパスワードが設定されているか確認、デフォルトは 'password'
DB_PASSWORD="${POSTGRES_PASSWORD:=password}"
# カスタムデータベース名が設定されているか確認、デフォルトは 'newsletter'
DB_NAME="${POSTGRES_DB:=newsletter}"
# カスタムポートが設定されているか確認、デフォルトは '5432'
DB_PORT="${POSTGRES_PORT:=5432}"


# DockerのPostgreSQLを起動をスキップできるようにする
if [[ -z "${SKIP_DOCKER}" ]]
then
  # Dockerを使用してpostgresを起動
  docker run \
    -e "POSTGRES_USER=${DB_USER}" \
    -e "POSTGRES_PASSWORD=${DB_PASSWORD}" \
    -e "POSTGRES_DB=${DB_NAME}" \
    -p "${DB_PORT}":5432 \
    -d postgres \
    postgres -N 1000
    # テスト目的で最大接続数を増やす (デフォルトは100)
fi


# PostgreSQLがコマンドを受け付ける準備ができるまで繰り返し接続を試みる
until PGPASSWORD="${DB_PASSWORD}" psql -h "localhost" -U "${DB_USER}" -p "${DB_PORT}" -d "postgres" -c '\q'; do
  # Postgresがまだ利用できない場合はエラーメッセージを出力
  >&2 echo "Postgres is still unavailable - sleeping"
  # 1秒待ってから再度接続を試みる
  sleep 1
done

# Postgresが起動していることを確認したメッセージを出力
>&2 echo "Postgres is up and running on port ${DB_PORT}!"

# DATABASE_URL環境変数を設定
export DATABASE_URL=postgres://${DB_USER}:${DB_PASSWORD}@localhost:${DB_PORT}/${DB_NAME}
# sqlx CLIツールを使用してデータベースを作成
sqlx database create
sqlx migrate run

>&2 echo "Postgres has been migrated, ready to go!"
