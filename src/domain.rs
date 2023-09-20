use unicode_segmentation::UnicodeSegmentation;

pub struct NewSubscriber {
    pub email: String,
    pub name: SubscriberName,
}

// サブスクライバーの名前を保持する構造体
pub struct SubscriberName(String);

impl SubscriberName {
    /// 入力が名前として有効な文字列ならばSubscriberNameを返し、そうでなければエラーを返す
    pub fn parse(s: String) -> Result<SubscriberName, String> {
        // 空白か?
        let is_empty_or_whitespace = s.trim().is_empty();

        // 文字数が256文字超か?
        let is_too_long = s.graphemes(true).count() > 256;

        // 禁止文字を含むか?
        let forbidden_characters = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
        let contains_forbidden_characters = s.chars().any(|g| forbidden_characters.contains(&g));

        if is_empty_or_whitespace || is_too_long || contains_forbidden_characters {
            Err(format!("{} is not a valid subscriber name.", s))
        } else {
            Ok(Self(s))
        }
    }
}

// AsRefトレイトを実装することで、SubscriberNameを&strとして扱えるようにする
impl AsRef<str> for SubscriberName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
