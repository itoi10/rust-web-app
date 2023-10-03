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

    let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription.");

    // DBに保存されたデータを検証する
    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
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
