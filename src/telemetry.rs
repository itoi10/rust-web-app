use tracing::subscriber::set_global_default;
use tracing::Subscriber;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};

pub fn get_subscriber<Sink>(
    name: String,
    env_filter: String,
    sink: Sink,
) -> impl Subscriber + Sync + Send
where
    Sink: for<'a> MakeWriter<'a> + Send + Sync + 'static,
{
    // 環境変数RUST_LOGに設定されたログレベルを読み込む
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(env_filter));
    // Bunyan形式(JSON形式のログ)で出力する
    let formatting_layer = BunyanFormattingLayer::new(name, sink);
    Registry::default()
        // ログレベルを設定
        .with(env_filter)
        // JSON形式でログ出力するための設定
        .with(JsonStorageLayer)
        // Bunyan形式でログを標準出力に出力する
        .with(formatting_layer)
}

pub fn init_subscriber(subscriber: impl Subscriber + Send + Sync) {
    // すべてのログをsubscribersにリダイレクトする設定 (これでactix-webのログも出力される)
    LogTracer::init().expect("Failed to set logger");
    // アプリケーション全体でこのログ設定を使う
    set_global_default(subscriber).expect("Failed to set subscriber");
}
