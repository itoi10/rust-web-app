use std::net::TcpListener;
use web_prod::run;

// Webサーバーを起動して、そのアドレスを返す
fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    // 0を指定するとOSが空いているポートを割り当てる
    let port = listener.local_addr().unwrap().port();
    let server = run(listener).expect("Failed to bind address");
    // サーバーをバックグランドで起動。戻り値を使えばシャットダウンなどの操作もできるが、今回はプログラムが終了するまで動く
    let _ = tokio::spawn(server);
    // アドレスを返す
    format!("http://127.0.0.1:{}", port)
}

// GET /health_check が成功することを確認する
#[tokio::test]
async fn health_check_works() {
    // AAAパターン  Arrange(準備) -> Act(実行) -> Assert(検証)

    // Arrange
    let address = spawn_app();
    // クライアントを作成
    let client = reqwest::Client::new();

    // Act
    let response = client
        .get(&format!("{}/health_check", &address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}
