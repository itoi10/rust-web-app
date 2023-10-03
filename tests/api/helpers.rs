use once_cell::sync::Lazy;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;
use web_prod::configuration::{get_configuration, DatabaseSettings};
use web_prod::startup::{get_connection_pool, Application};
use web_prod::telemetry::{get_subscriber, init_subscriber};
use wiremock::MockServer;

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
    // メールサーバ(PostmarkのAPIの代わり)
    pub email_server: MockServer,
}

impl TestApp {
    /// /subscriptinsにPOSTリクエストを送信する
    pub async fn post_subscriptions(&self, body: String) -> reqwest::Response {
        reqwest::Client::new()
            .post(&format!("{}/subscriptions", &self.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }
}

/// テスト用のHTTPサーバを起動する
pub async fn spawn_app() -> TestApp {
    // 最初だけログ設定を初期化する
    Lazy::force(&TRACING);

    // PostmarkのAPIの代わりにモックサーバを起動する
    let email_server = MockServer::start().await;

    // テストの際には設定をランダム化して、テスト間の独立性を確保する
    let configuration = {
        let mut c = get_configuration().expect("Failed to read configuration.");
        // Use a different database for each test case
        c.database.database_name = Uuid::new_v4().to_string();
        // Use a random OS port
        c.application.port = 0;
        // モックサーバをメールAPIとして使う
        c.email_client.base_url = email_server.uri();
        c
    };

    // データベースを作成しマイグレーションを実行する
    configure_database(&configuration.database).await;

    // アプリケーション初期化
    let application = Application::build(configuration.clone())
        .await
        .expect("Failed to build application.");
    // アプリケーション起動前にアドレスを取得
    let address = format!("http://127.0.0.1:{}", application.port());
    // アプリケーション実行
    let _ = tokio::spawn(application.run_until_stopped());

    TestApp {
        address,
        db_pool: get_connection_pool(&configuration.database),
        email_server,
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
