use crate::domain::SubscriberEmail;
use reqwest::Client;
use secrecy::{ExposeSecret, Secret};

pub struct EmailClient {
    // Clientのインスタンスを保持する
    http_client: Client,
    // リクエストを行うAPIのURL
    base_url: String,
    // メールの送信者として設定するアドレス
    sender: SubscriberEmail,
    // Postmarkの認証トークン
    authorization_token: Secret<String>,
}

impl EmailClient {
    pub fn new(
        base_url: String,
        sender: SubscriberEmail,
        authorization_token: Secret<String>,
        timeout: std::time::Duration,
    ) -> Self {
        let http_client = Client::builder().timeout(timeout).build().unwrap();
        Self {
            http_client,
            base_url,
            sender,
            authorization_token,
        }
    }

    // メールを送信する
    pub async fn send_email(
        &self,
        recipient: SubscriberEmail,
        subject: &str,
        html_content: &str,
        text_content: &str,
    ) -> Result<(), reqwest::Error> {
        // リクエストURL
        let url = format!("{}/email", self.base_url);
        // リクエストボディ
        let request_body = SendEmailRequest {
            from: self.sender.as_ref(),
            to: recipient.as_ref(),
            subject,
            html_body: html_content,
            text_body: text_content,
        };
        // リクエスト送信
        self.http_client
            .post(&url)
            // Postmarkの認証トークンをヘッダに設定
            .header(
                "X-Postmark-Server-Token",
                self.authorization_token.expose_secret(),
            )
            .json(&request_body)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }
}

// リクエストボディの構造体
// jsonシリアライズ可能にするためにSerializeをつける
// PascalCaseはfromをFromに、toをToなどに変換する
// {"From": "xxx", "To": "xxx", "Subject": "xxx", "HtmlBody": "xxx", "TextBody": "xxx"}
#[derive(serde::Serialize)]
#[serde(rename_all = "PascalCase")]
struct SendEmailRequest<'a> {
    from: &'a str,
    to: &'a str,
    subject: &'a str,
    html_body: &'a str,
    text_body: &'a str,
}

// 単体テスト
#[cfg(test)]
mod tests {
    use crate::domain::SubscriberEmail;
    use crate::email_client::EmailClient;
    use claim::assert_err;
    use claim::assert_ok;
    use fake::faker::internet::en::SafeEmail;
    use secrecy::Secret;
    use wiremock::matchers::any;
    // 英語のランダムな文章を生成する
    use fake::faker::lorem::en::{Paragraph, Sentence};
    use fake::{Fake, Faker};
    // wiremockを使ってHTTPリクエストをモックする
    use wiremock::matchers::{header, header_exists, method, path};
    use wiremock::Request;
    use wiremock::{Mock, MockServer, ResponseTemplate};

    // リクエストボディを検証するカスタムマッチャー
    struct SendEmailBodyMatcher;

    // wiremock::Matchトレイトを実装することで、リクエストボディを検証できるようになる
    impl wiremock::Match for SendEmailBodyMatcher {
        // 与えられたリクエストが期待する条件を満たすかどうかを判断し真偽値を返す
        fn matches(&self, request: &Request) -> bool {
            // リクエストボディをJSONとしてパースを試みる
            let result: Result<serde_json::Value, _> = serde_json::from_slice(&request.body);
            if let Ok(body) = result {
                // リクエストボディに期待するフィールドが含まれているかどうかを判断する
                // 全てののフィールドが含まれていればtrueを返し、そうでなければfalseを返す
                body.get("From").is_some()
                    && body.get("To").is_some()
                    && body.get("Subject").is_some()
                    && body.get("HtmlBody").is_some()
                    && body.get("TextBody").is_some()
            } else {
                // JSONパース失敗時はfalseを返す
                false
            }
        }
    }

    /// ランダムなメールの件名を生成
    fn subject() -> String {
        Sentence(1..2).fake()
    }

    /// ランダムなメールの内容を生成
    fn content() -> String {
        Paragraph(1..10).fake()
    }

    /// ランダムな購読者のメールアドレスを生成
    fn email() -> SubscriberEmail {
        SubscriberEmail::parse(SafeEmail().fake()).unwrap()
    }

    /// EmailClientのテスト用インスタンスを取得
    fn email_client(base_url: String) -> EmailClient {
        EmailClient::new(
            base_url,
            email(),
            Secret::new(Faker.fake()),
            // テスト時間を短くするためにタイムアウト時間を短くする
            std::time::Duration::from_millis(200),
        )
    }

    // send_emailメソッドがbase_urlへリクエストを行うテスト
    // #[tokio::test]をつけると非同期テストになる
    #[tokio::test]
    async fn send_email_fires_a_request_to_base_url() {
        // [Arrange]
        // モックのHTTPサーバを起動
        let mock_server = MockServer::start().await;
        // EmailClientを初期化 (モックサーバのURLと送信元メールアドレス、ランダムなトークンを渡す)
        let email_client = email_client(mock_server.uri());

        // モックサーバに期待のリクエストが来たら200を返すように設定
        // ヘッダー、パス、メソッドを指定してリクエストをモックする
        Mock::given(header_exists("X-Postmark-Server-Token"))
            .and(header("Content-Type", "application/json"))
            .and(path("/email"))
            .and(method("POST"))
            // リクエストボディを検証するカスタムマッチャーを指定
            .and(SendEmailBodyMatcher)
            .respond_with(ResponseTemplate::new(200))
            // 期待するリクエスト回数。これより多くても少なくてもテスト失敗
            .expect(1)
            .mount(&mock_server)
            .await;

        // [Act]
        // メールを送信する
        let _ = email_client
            .send_email(email(), &subject(), &content(), &content())
            .await;

        // [Assert]
        // モックサーバにリクエストが1回くれば成功
    }

    // サーバが200を返したらメール送信成功
    #[tokio::test]
    async fn send_email_succeeds_if_the_server_returns_200() {
        // [Arrange]
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        Mock::given(any())
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        // [Act]
        let outcome = email_client
            .send_email(email(), &subject(), &content(), &content())
            .await;

        // [Assert]
        // send_emailメソッドがOkを返せば成功
        assert_ok!(outcome);
    }

    // サーバが500を返したらメール送信失敗
    #[tokio::test]
    async fn send_email_fails_if_the_server_returns_500() {
        // [Arrange]
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        // サーバが500を返すように設定
        Mock::given(any())
            .respond_with(ResponseTemplate::new(500))
            .expect(1)
            .mount(&mock_server)
            .await;

        // [Act]
        let outcome = email_client
            .send_email(email(), &subject(), &content(), &content())
            .await;

        // [Assert]
        // send_emailメソッドがErrを返せばテスト成功
        assert_err!(outcome);
    }

    // サーバの応答に時間がかかる場合はタイムアウト
    #[tokio::test]
    async fn send_email_times_out_if_the_server_takes_too_long() {
        // [Arrange]
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        // サーバの応答に3分かかるように設定
        let response = ResponseTemplate::new(200).set_delay(std::time::Duration::from_secs(180));
        Mock::given(any())
            .respond_with(response)
            .expect(1)
            .mount(&mock_server)
            .await;

        // [Act]
        let outcome = email_client
            .send_email(email(), &subject(), &content(), &content())
            .await;

        // [Assert]
        // send_emailメソッドがErrを返せばテスト成功
        assert_err!(outcome);
    }
}
