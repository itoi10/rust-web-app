use actix_web::dev::Server;
use actix_web::{web, App, HttpResponse, HttpServer};
use serde::{Deserialize, Serialize};
use std::net::TcpListener;

async fn health_check() -> HttpResponse {
    // finish()することでHttpResponseBuildeからHttpResponseに変換する
    // finish()しない場合はimpl Responderを返すことになる
    // impl Responder for HttpResponseBuilderはfinish()を呼び出しているので結果的に同じこと
    HttpResponse::Ok().finish()
}

#[derive(serde::Deserialize)]
struct FormData {
    email: String,
    name: String,
}

// FormData構造体へのデシリアライズが無効の場合は自動的に400 Bad Requestを返す
async fn subscribe(_form: web::Form<FormData>) -> HttpResponse {
    HttpResponse::Ok().finish()
}

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
