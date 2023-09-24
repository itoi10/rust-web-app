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
        // todo!()
        // とりあえずOKを返す
        Ok(())
    }
}

// 単体テスト
#[cfg(test)]
mod tests {
    use crate::domain::SubscriberEmail;
    use crate::email_client::EmailClient;
    use fake::faker::internet::en::SafeEmail;
    // 英語のランダムな文章を生成する
    use fake::faker::lorem::en::{Paragraph, Sentence};
    use fake::{Fake, Faker};
    // wiremockを使ってHTTPリクエストをモックする
    use wiremock::matchers::any;
    use wiremock::{Mock, MockServer, ResponseTemplate};

    // #[tokio::test]をつけると非同期テストになる
    // send_emailメソッドがbase_urlへリクエストを行うテスト
    #[tokio::test]
    async fn send_email_fires_a_request_to_base_url() {
        // [Arrange]
        // モックのHTTPサーバを起動
        let mock_server = MockServer::start().await;
        // 送信元メールアドレスを生成
        let sender = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        // EmailClientを初期化 (モックサーバのURLと送信元メールアドレスを渡す)
        let email_client = EmailClient::new(mock_server.uri(), sender);

        // モックサーバにリクエストが来たら200を返すように設定
        // any()はどんなリクエストにもマッチする
        Mock::given(any())
            .respond_with(ResponseTemplate::new(200))
            // リクエストが1回来ることを期待する。この回数より多くても少なくてもテスト失敗になる
            .expect(1)
            .mount(&mock_server)
            .await;

        // 送信先メールアドレスと件名、本文を生成
        let subscriber_email = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let subject: String = Sentence(1..2).fake();
        let content: String = Paragraph(1..10).fake();

        // [Act]
        // メールを送信する
        let _ = email_client
            .send_email(subscriber_email, &subject, &content, &content)
            .await;

        // [Assert]
        // モックサーバにリクエストが1回くれば成功
    }
}
