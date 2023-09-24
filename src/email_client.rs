use crate::domain::SubscriberEmail;
use reqwest::Client;

pub struct EmailClient {
    // Clientのインスタンスを保持する
    http_client: Client,
    // リクエストを行うAPIのURL
    base_url: String,
    // メールの送信者として設定するアドレス
    sender: SubscriberEmail,
}

impl EmailClient {
    pub fn new(base_url: String, sender: SubscriberEmail) -> Self {
        Self {
            http_client: Client::new(),
            base_url,
            sender,
        }
    }

    // !TODO メールを送信する
    pub async fn send_email(
        &self,
        recipient: SubscriberEmail,
        subject: &str,
        html_content: &str,
        text_content: &str,
    ) -> Result<(), String> {
        todo!()
    }
}
