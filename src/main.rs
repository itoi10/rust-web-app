use web_prod::configuration::get_configuration;
use web_prod::startup::Application;
use web_prod::telemetry::{get_subscriber, init_subscriber};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // ログ設定を初期化
    let subscriber = get_subscriber("web_prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    // 設定ファイルを読み込む
    let configuration = get_configuration().expect("Failed to read configuration.");

    // アプリケーション初期化
    let application = Application::build(configuration).await?;

    // アプリケーション起動
    application.run_until_stopped().await?;

    Ok(())
}
