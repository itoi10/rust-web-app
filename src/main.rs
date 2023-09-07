use std::net::TcpListener;
use web_prod::startup::run;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // ?はエラーが起きたらリターンするという意味。この場合はエラーステータスでプログラムが終了する
    let address = TcpListener::bind("127.0.0.1:8000")?;
    println!("Listening on http://{}", address.local_addr()?);
    // Actix-webサーバを非同期に移動して、awaitで待機する
    run(address)?.await
}
