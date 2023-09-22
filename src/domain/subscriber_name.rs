use unicode_segmentation::UnicodeSegmentation;

// サブスクライバーの名前を保持する構造体
#[derive(Debug)]
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

// #[cfg(test)]をつけるとcargo testのときのみコンパイルされる
// Rustでは特定のファイルやモジュール内にmod testsを定義してテストを書ける
#[cfg(test)]
mod tests {
    // 別の名前空間となるので、SubscriberNameをインポートする
    use crate::domain::SubscriberName;
    use claim::{assert_err, assert_ok};

    #[test]
    fn a_256_grapheme_long_name_is_valid() {
        let name = "ё".repeat(256);
        assert_ok!(SubscriberName::parse(name));
    }

    #[test]
    fn a_name_longer_than_256_graphemes_is_rejected() {
        let name = "a".repeat(257);
        assert_err!(SubscriberName::parse(name));
    }

    // 日本語の場合のテストも追加
    #[test]
    fn a_256_grapheme_long_japanese_name_is_valid() {
        let name = "あ".repeat(256);
        assert_ok!(SubscriberName::parse(name));
    }

    // 日本語の場合のテストも追加
    #[test]
    fn a_japanese_name_longer_than_256_graphemes_is_rejected() {
        let name = "あ".repeat(257);
        assert_err!(SubscriberName::parse(name));
    }

    #[test]
    fn whitespace_only_names_are_rejected() {
        let name = " ".to_string();
        assert_err!(SubscriberName::parse(name));
    }

    #[test]
    fn empty_string_is_rejected() {
        let name = "".to_string();
        assert_err!(SubscriberName::parse(name));
    }

    #[test]
    fn names_containing_an_invalid_character_are_rejected() {
        for name in &['/', '(', ')', '"', '<', '>', '\\', '{', '}'] {
            let name = name.to_string();
            assert_err!(SubscriberName::parse(name));
        }
    }

    #[test]
    fn a_valid_name_is_parsed_successfully() {
        let name = "Ursula Le Guin".to_string();
        assert_ok!(SubscriberName::parse(name));
    }
}
