use actix_web::HttpResponse;

pub async fn health_check() -> HttpResponse {
    // finish()することでHttpResponseBuildeからHttpResponseに変換する
    // finish()しない場合はimpl Responderを返すことになる
    // impl Responder for HttpResponseBuilderはfinish()を呼び出しているので結果的に同じこと
    HttpResponse::Ok().finish()
}
