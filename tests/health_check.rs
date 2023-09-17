use once_cell::sync::Lazy;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::net::TcpListener;
use uuid::Uuid;
use web_prod::configuration::{get_configuration, DatabaseSettings};
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
async fn spawn_app() -> TestApp {
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
    // HTTPサーバ起動
    let server = run(listener, connection_pool.clone()).expect("Failed to bind address");
    let _ = tokio::spawn(server);
    // TestApp構造体を返す
    TestApp {
        address,
        db_pool: connection_pool,
    }
}

/// テスト用のデータベースを作成し、接続プールを返す
pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
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

/// GET /health_check 正常系テスト
#[tokio::test]
async fn health_check_works() {
    // [Arrange]
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    // [Act]
    let response = client
        // Use the returned application address
        .get(&format!("{}/health_check", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    // [Assert]
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

/// POST /subscriptions 正常系テスト
#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    // [Arrange]
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    // [Act]
    let response = client
        .post(&format!("{}/subscriptions", &app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    //[Assert]
    assert_eq!(200, response.status().as_u16());
    // データベースに保存されたデータを取得
    let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
}

/// POST /subscriptions 異常系テスト
#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    // [Arrange]
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        // [Act]
        let response = client
            .post(&format!("{}/subscriptions", &app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request.");

        // [Assert]
        assert_eq!(
            400,
            response.status().as_u16(),
            // Additional customised error message on test failure
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        );
    }
}
