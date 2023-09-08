// 構造体やメンバにpubを付けることで他のモジュールからもアクセスできるようになる
#[derive(serde::Deserialize)]
pub struct Settings {
    // データベースの設定
    pub database: DatabaseSettings,
    // アプリケーションのポート番号
    pub application_port: u16,
}

#[derive(serde::Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: String,
    pub port: u16,
    pub host: String,
    pub database_name: String,
}

// PostgreSQLの接続文字列を返す
impl DatabaseSettings {
    pub fn connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.database_name
        )
    }
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    // configuration.yamlファイルから設定を読み込む
    let settings = config::Config::builder()
        .add_source(config::File::new(
            "configuration.yaml",
            config::FileFormat::Yaml,
        ))
        .build()?;
    // 設定をSettings構造体にデシリアライズする
    settings.try_deserialize::<Settings>()
}
