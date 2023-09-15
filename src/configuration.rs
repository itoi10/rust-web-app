use secrecy::{ExposeSecret, Secret};

// 構造体やメンバにpubを付けることで他のモジュールからもアクセスできるようになる
#[derive(serde::Deserialize)]
pub struct Settings {
    // データベースの設定
    pub database: DatabaseSettings,
    // アプリケーションの設定
    pub application: ApplicationSettings,
}

#[derive(serde::Deserialize)]
pub struct ApplicationSettings {
    // アプリケーションのポート番号
    pub port: u16,
    // アプリケーションのホスト名
    pub host: String,
}

#[derive(serde::Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    // Secretから値を取り出すにはexpose_secret()を使う。誤ってログに出力しないようにするため
    pub password: Secret<String>,
    pub port: u16,
    pub host: String,
    pub database_name: String,
}

impl DatabaseSettings {
    /// PostgreSQLの接続文字列を返す
    pub fn connection_string(&self) -> Secret<String> {
        Secret::new(format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port,
            self.database_name
        ))
    }
    /// PostgreSQLの接続文字列をDB名なしで返す
    pub fn connection_string_without_db(&self) -> Secret<String> {
        Secret::new(format!(
            "postgres://{}:{}@{}:{}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port
        ))
    }
}

/// 設定ファイルを読み込む
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
            configuration_directory.join(&environment_filename),
        ))
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
