use once_cell::sync::Lazy;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::net::TcpListener;
use uuid::Uuid;
use web_prod::configuration::{get_configuration, DatabaseSettings};
use web_prod::email_client::EmailClient;
use web_prod::startup::run;
use web_prod::telemetry::{get_subscriber, init_subscriber};

// ログ設定を一度だけ初期化する
static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();
    if std::env::var("TEST_LOG").is_ok() {
        // ログを標準出力に出力する
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        // ログ出力を無効にする
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    };
});

pub struct TestApp {
    // HTTPアドレス
    pub address: String,
    // データベース接続プール
    pub db_pool: PgPool,
}

/// テスト用のHTTPサーバを起動する
pub async fn spawn_app() -> TestApp {
    // 最初だけログ設定を初期化する
    Lazy::force(&TRACING);

    // :0を指定するとOSが空いているポートを自動的に割り当てる
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    // 設定ファイルを読み込む
    let mut configuration = get_configuration().expect("Failed to read configuration.");

    // テスト用のデータベース名を生成
    configuration.database.database_name = Uuid::new_v4().to_string();

    // データベース作成
    let connection_pool = configure_database(&configuration.database).await;

    // EmailClientを初期化
    let sender_email = configuration
        .email_client
        .sender()
        .expect("Invalid sender email address.");
    let timeout = configuration.email_client.timeout();
    let email_client = EmailClient::new(
        configuration.email_client.base_url,
        sender_email,
        configuration.email_client.authorization_token,
        timeout,
    );

    // HTTPサーバ起動
    let server =
        run(listener, connection_pool.clone(), email_client).expect("Failed to bind address");
    let _ = tokio::spawn(server);
    // TestApp構造体を返す
    TestApp {
        address,
        db_pool: connection_pool,
    }
}

/// テスト用のデータベースを作成し、接続プールを返す
async fn configure_database(config: &DatabaseSettings) -> PgPool {
    // PostgreSQLに接僕
    let mut connection = PgConnection::connect_with(&config.without_db())
        .await
        .expect("Failed to connect to Postgres");

    // データベースを作成
    connection
        .execute(&*format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database.");

    // データベース接続プールを作成
    let connection_pool = PgPool::connect_with(config.with_db())
        .await
        .expect("Failed to connect to Postgres.");

    // マイグレーションを実行
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");

    // データベース接続プールを返す
    connection_pool
}
