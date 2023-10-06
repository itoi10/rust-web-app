// crateはプロジェクトのルートを指すキーワード
use crate::configuration::DatabaseSettings;
use crate::configuration::Settings;
use crate::email_client::EmailClient;
use crate::routes::{confirm, health_check, subscribe};
use actix_web::dev::Server;
use actix_web::web::Data;
use actix_web::{web, App, HttpServer};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;

// サーバとポート番号を保持する構造体
pub struct Application {
    port: u16,
    server: Server,
}

impl Application {
    // コンストラクタ的なメソッド
    pub async fn build(configuration: Settings) -> Result<Self, std::io::Error> {
        let connection_pool = get_connection_pool(&configuration.database);

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

        let address = format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        );
        let listener = TcpListener::bind(address)?;
        let port = listener.local_addr().unwrap().port();
        let server = run(listener, connection_pool, email_client)?;

        // Self { port, server} で新しいインスタンスが作成され、Okバリアントでラップされて返される
        Ok(Self { port, server })
    }

    /// ポート番号を返す
    pub fn port(&self) -> u16 {
        self.port
    }

    /// サーバ実行
    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

pub fn get_connection_pool(configuration: &DatabaseSettings) -> PgPool {
    PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(configuration.with_db())
}

pub fn run(
    listener: TcpListener,
    db_pool: PgPool,
    email_client: EmailClient,
) -> Result<Server, std::io::Error> {
    // connectionをActixWebアプリ全体で共有するために、web::Data::newとapp_dataを使う
    let db_pool = Data::new(db_pool);
    // EmailClientも同様に共有する
    let email_client = Data::new(email_client);

    // 新しいHttpServerオブジェクトを作成する
    let server = HttpServer::new(move || {
        // moveでconnectionをクロージャに封じ込めて、複数のスレッドから安全にアクセスできるようにする
        App::new()
            .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            .route("/subscriptions/confirm", web::get().to(confirm))
            .app_data(db_pool.clone()) // cloneは新しい参照を作成しているだけで実体を複製しているわけではない
            .app_data(email_client.clone())
    })
    .listen(listener)?
    .run();
    // Resultでラップして返す。これで呼び出し元でサーバーのライフタイムを管理できる
    Ok(server)
}
