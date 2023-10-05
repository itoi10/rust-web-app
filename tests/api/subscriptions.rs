use crate::helpers::spawn_app;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

// POST /subscriptions 成功時のテスト
#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    // [Arrange]
    let app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    // [Act]
    let response = app.post_subscriptions(body.into()).await;

    // [Assert]
    assert_eq!(200, response.status().as_u16());
}

// 上の関数からDB保存のテストを分離
#[tokio::test]
async fn subscribe_persists_the_new_subscriber() {
    // [Arrange]
    let app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    // [Act]
    app.post_subscriptions(body.into()).await;

    // [Assert]
    let saved = sqlx::query!("SELECT email, name, status FROM subscriptions",)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
    assert_eq!(saved.status, "pending_confirmation")
}

// POST /subscriptions フィールドが不足している場合のテスト
#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    // [Arrange]
    let app = spawn_app().await;
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        // [Act]
        let response = app.post_subscriptions(invalid_body.into()).await;

        // [Assert]
        assert_eq!(
            400,
            response.status().as_u16(),
            // Additional customised error message on test failure
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        );
    }
}

// POST /subscriptions フィールドが不正な場合のテスト
#[tokio::test]
async fn subscribe_returns_a_400_when_fields_are_present_but_invalid() {
    // [Arrange]
    let app = spawn_app().await;
    let test_cases = vec![
        ("name=&email=ursula_le_guin%40gmail.com", "empty name"),
        ("name=Ursula&email=", "empty email"),
        ("name=Ursula&email=definitely-not-an-email", "invalid email"),
    ];

    for (body, description) in test_cases {
        // [Act]
        let response = app.post_subscriptions(body.into()).await;

        // [Assert]
        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not return a 400 Bad Request when the payload was {}.",
            description
        );
    }
}

// 有効なデータの場合に確認メールを送信するテスト
#[tokio::test]
async fn subscribe_sends_a_confirmation_email_for_valid_data() {
    // [Arrange]
    // アプリの起動と、有効な名前とメールアドレスの用意
    let app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    // POSTリクエストをモックサーバに送信する準備
    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    // [Act]
    app.post_subscriptions(body.into()).await; // intoは &str -> String

    // [Assert]
    // Mock::givenで設定したリクエストの検証される
}

// リンク付きの確認メールを送信するテスト
#[tokio::test]
async fn subscribe_sends_a_confirmation_email_with_a_link() {
    // [Arrange]
    let app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        // expect() はこのテストの観点ではない
        .mount(&app.email_server)
        .await;

    // [Act]
    app.post_subscriptions(body.into()).await;

    // [Assert]
    // モックサーバに送信されたリクエストを取得する
    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    let body: serde_json::Value = serde_json::from_slice(&email_request.body).unwrap();

    // リクエストフィールからリンクを取得する
    // Rustの | 引数 | { 処理 } はクロージャ
    let get_link = |s: &str| {
        // linkify::LinkFinder::new() はリンクを検出する
        let links: Vec<_> = linkify::LinkFinder::new()
            .links(s)
            // URLのみを取得する (除外しない場合はEmailアドレスも取得される)
            .filter(|l| *l.kind() == linkify::LinkKind::Url)
            .collect();
        assert_eq!(links.len(), 1);
        // &linkify::Link -> &str -> String
        // to_stringというメソッドもあるが、to_ownedはToOwnedトレイトを実装している型に対して使える
        links[0].as_str().to_owned()
    };

    let html_link = get_link(&body["HtmlBody"].as_str().unwrap());
    let text_link = get_link(&body["TextBody"].as_str().unwrap());
    // htmlとtextのリンクが同じであること
    assert_eq!(html_link, text_link);
}
