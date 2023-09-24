// crateはプロジェクトのルートを指すキーワード
use crate::email_client::EmailClient;
use crate::routes::{health_check, subscribe};
use actix_web::dev::Server;
use actix_web::web::Data;
use actix_web::{web, App, HttpServer};
use sqlx::PgPool;
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;

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
            .app_data(db_pool.clone()) // cloneは新しい参照を作成しているだけで実体を複製しているわけではない
            .app_data(email_client.clone())
    })
    .listen(listener)?
    .run();
    // Resultでラップして返す。これで呼び出し元でサーバーのライフタイムを管理できる
    Ok(server)
}
