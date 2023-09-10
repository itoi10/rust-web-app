use sqlx::PgPool;
use std::net::TcpListener;
use web_prod::configuration::get_configuration;
use web_prod::startup::run;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // 設定ファイルを読み込む
    let configuration = get_configuration().expect("Failed to read configuration.");
    // Postgresに接続 (PgConnectionは単一のデータベース接続だが、PgPoolはコネクションプール)
    let connection_pool = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to Postgres.");

    let address = format!("127.0.0.1:{}", configuration.application_port);
    println!("Listening on http://{}", address);
    let listener = TcpListener::bind(address)?;
    // Actix-webサーバを非同期に移動して、awaitで待機する
    run(listener, connection_pool)?.await
}
