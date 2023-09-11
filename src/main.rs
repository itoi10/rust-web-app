use sqlx::PgPool;
use std::net::TcpListener;
use tracing::subscriber::set_global_default;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};
use web_prod::configuration::get_configuration;
use web_prod::startup::run;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // すべてのログをsubscribersにリダイレクトする設定 (これでactix-webのログも出力される)
    LogTracer::init().expect("Failed to set logger");
    // 環境変数RUST_LOGに設定されたログレベルを読み込む
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    // Bunyan形式(JSON形式のログ)で標準出力に出力する
    let formatting_layer = BunyanFormattingLayer::new("zero2prod".into(), std::io::stdout);
    let subscriber = Registry::default()
        // ログレベルを設定
        .with(env_filter)
        // JSON形式でログ出力するための設定
        .with(JsonStorageLayer)
        // Bunyan形式でログを標準出力に出力する
        .with(formatting_layer);
    // アプリケーション全体でこのログ設定を使う
    set_global_default(subscriber).expect("Failed to set subscriber");

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
