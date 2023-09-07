// crateはプロジェクトのルートを指すキーワード
use crate::routes::{health_check, subscribe};
use actix_web::dev::Server;
use actix_web::{web, App, HttpServer};
use std::net::TcpListener;

pub fn run(listener: TcpListener) -> Result<Server, std::io::Error> {
    // 新しいHttpServerオブジェクトを作成する
    let server = HttpServer::new(|| {
        App::new()
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
    })
    .listen(listener)?
    .run();
    // Resultでラップして返す。これで呼び出し元でサーバーのライフタイムを管理できる
    Ok(server)
}
