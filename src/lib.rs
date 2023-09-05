use actix_web::dev::Server;
use actix_web::{web, App, HttpResponse, HttpServer};
use std::net::TcpListener;

async fn health_check() -> HttpResponse {
    // finish()することでHttpResponseBuildeからHttpResponseに変換する
    // finish()しない場合はimpl Responderを返すことになる
    // impl Responder for HttpResponseBuilderはfinish()を呼び出しているので結果的に同じこと
    HttpResponse::Ok().finish()
}

pub fn run(listener: TcpListener) -> Result<Server, std::io::Error> {
    // 新しいHttpServerオブジェクトを作成する
    let server = HttpServer::new(|| App::new().route("/health_check", web::get().to(health_check)))
        .listen(listener)?
        .run();
    // Resultでラップして返す。これで呼び出し元でサーバーのライフタイムを管理できる
    Ok(server)
}
