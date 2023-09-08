use std::net::TcpListener;
use web_prod::configuration::get_configuration;
use web_prod::startup::run;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let configuration = get_configuration().expect("Failed to read configuration.");

    let address = format!("127.0.0.1:{}", configuration.application_port);
    println!("Listening on http://{}", address);
    let listener = TcpListener::bind(address)?;
    // Actix-webサーバを非同期に移動して、awaitで待機する
    run(listener)?.await
}
