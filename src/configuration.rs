use crate::domain::SubscriberEmail;
use secrecy::{ExposeSecret, Secret};
use serde_aux::field_attributes::deserialize_number_from_string;
use sqlx::postgres::{PgConnectOptions, PgSslMode};
use sqlx::ConnectOptions;
use std::convert::{TryFrom, TryInto};

// 構造体やメンバにpubを付けることで他のモジュールからもアクセスできるようになる
#[derive(serde::Deserialize)]
pub struct Settings {
    // データベースの設定
    pub database: DatabaseSettings,
    // アプリケーションの設定
    pub application: ApplicationSettings,
    // Email送信用の設定
    pub email_client: EmailClientSettings,
}

#[derive(serde::Deserialize)]
pub struct EmailClientSettings {
    // メール送信用のAPIのベースURL
    pub base_url: String,
    // メールの送信者として設定するアドレス
    pub sender_email: String,
    // Postmarkの認証トークン
    pub authorization_token: Secret<String>,
    // リクエストのタイムアウト時間
    pub timeout_milliseconds: u64,
}

impl EmailClientSettings {
    pub fn sender(&self) -> Result<SubscriberEmail, String> {
        SubscriberEmail::parse(self.sender_email.clone())
    }

    pub fn timeout(&self) -> std::time::Duration {
        std::time::Duration::from_millis(self.timeout_milliseconds)
    }
}

#[derive(serde::Deserialize)]
pub struct ApplicationSettings {
    // アプリケーションのポート番号 serdeのdeserialize_with属性を使って文字列から数値に変換する
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    // アプリケーションのホスト名
    pub host: String,
}

#[derive(serde::Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    // Secretから値を取り出すにはexpose_secret()を使う。誤ってログに出力しないようにするため
    pub password: Secret<String>,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
    pub database_name: String,
    pub require_ssl: bool,
}

impl DatabaseSettings {
    // データベース名の除いた接続オプションを返す
    pub fn without_db(&self) -> PgConnectOptions {
        let ssl_mode = if self.require_ssl {
            PgSslMode::Require // SSL接続を必須にする
        } else {
            PgSslMode::Prefer // SSL接続を任意にする
        };
        PgConnectOptions::new()
            .host(&self.host)
            .username(&self.username)
            .password(self.password.expose_secret())
            .port(self.port)
            .ssl_mode(ssl_mode)
    }

    // データベース名を含めた接続オプションを返す
    pub fn with_db(&self) -> PgConnectOptions {
        let options = self.without_db().database(&self.database_name);
        options.log_statements(tracing::log::LevelFilter::Trace)
    }
}

/// 設定をyamlファイルと環境変数から読み込む
/// 環境変数はAPP_プレフィックスを付ける。環境変数はyamlより優先される
pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    // カレントディレクトリを取得
    let base_path = std::env::current_dir().expect("Failed to determine the current directory");
    // 設定ファイルのディレクトリを組立
    let configuration_directory = base_path.join("configuration");

    // 環境変数からAPP_ENVIRONMENTを取得する。デフォルトはlocal
    let environment: Environment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "local".into())
        .try_into()
        .expect("Failed to parse APP_ENVIRONMENT.");
    // 環境に応じた設定ファイル名を組立
    let environment_filename = format!("{}.yaml", environment.as_str());
    // base.yamlと環境に応じた設定ファイルを読み込む
    let settings = config::Config::builder()
        .add_source(config::File::from(
            configuration_directory.join("base.yaml"),
        ))
        .add_source(config::File::from(
            configuration_directory.join(environment_filename),
        ))
        // APP_プレフィックスを付けた環境変数を設定として読み込む(これはyamlより後に追加しているので、yamlの設定より優先される)
        .add_source(
            config::Environment::with_prefix("APP")
                .prefix_separator("_")
                .separator("__"),
        )
        .build()?;
    // 設定をSettings構造体にデシリアライズする
    settings.try_deserialize::<Settings>()
}

// 環境の種類を表す列挙型 (ローカル環境か本番環境か)
pub enum Environment {
    Local,
    Production,
}

impl Environment {
    // 列挙型を文字列に変換する
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Local => "local",
            Environment::Production => "production",
        }
    }
}

// 文字列からEnvironment列挙型に変換する
impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "production" => Ok(Self::Production),
            other => Err(format!(
                "{} is not a supported environment. Use either `local` or `production`.",
                other
            )),
        }
    }
}
