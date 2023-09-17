use sqlx::postgres::PgPoolOptions;
use std::net::TcpListener;
use web_prod::configuration::get_configuration;
use web_prod::startup::run;
use web_prod::telemetry::{get_subscriber, init_subscriber};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // ログ設定を初期化
    let subscriber = get_subscriber("web_prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    // 設定ファイルを読み込む
    let configuration = get_configuration().expect("Failed to read configuration.");
    // Postgresに接続
    let connection_pool = PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2)) // 接続タイムアウトを2秒に設定
        .connect_lazy_with(configuration.database.with_db());

    let address = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    );
    let listener = TcpListener::bind(address)?;
    // Actix-webサーバを非同期に移動して、awaitで待機する
    run(listener, connection_pool)?.await
}
