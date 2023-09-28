use crate::domain::{NewSubscriber, SubscriberEmail, SubscriberName};
use actix_web::{web, HttpResponse};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

// FormDataからNewSubscriberに変換を試みるトレイトを実装
impl TryFrom<FormData> for NewSubscriber {
    type Error = String;

    // TryFromトレイトを実装するとtry_into()メソッドが使えるようになる
    fn try_from(value: FormData) -> Result<Self, Self::Error> {
        let name = SubscriberName::parse(value.name)?;
        let email = SubscriberEmail::parse(value.email)?;
        // 成功したらNewSubscriberを返す
        Ok(Self { email, name })
    }
}

// tracing::instrumentを使うと関数の呼び出しをトレースできる
#[tracing::instrument(
    // ログのトレース名
    name = "Adding a new subscriber",
    // ログから除外するフィールド
    skip(form, pool),
    // ログに追加するフィールド
    fields(
        subscriber_email = %form.email,
        subscriber_name = %form.name
    )
)]
pub async fn subscribe(form: web::Form<FormData>, pool: web::Data<PgPool>) -> HttpResponse {
    // フォームをパースしてNewSubscriberを取得する。パースに失敗した場合は400を返す
    let new_subscriber = match form.0.try_into() {
        Ok(subscriber) => subscriber,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };

    match insert_subscriber(&pool, &new_subscriber).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[tracing::instrument(
    // ログのトレース名
    name = "Saving new subscriber details in the database",
    // ログから除外するフィールド
    skip(new_subscriber, pool)
)]
pub async fn insert_subscriber(
    pool: &PgPool,
    new_subscriber: &NewSubscriber,
) -> Result<(), sqlx::Error> {
    // クエリ実行
    sqlx::query!(
        r#"INSERT INTO subscriptions (id, email, name, subscribed_at, status)
           VALUES ($1, $2, $3, $4, 'confirmed')"#,
        Uuid::new_v4(),
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
        Utc::now()
    )
    .execute(pool)
    .await
    // map_errはErrのときに処理を行う。?をつけてeを返却する
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;
    Ok(())
}
