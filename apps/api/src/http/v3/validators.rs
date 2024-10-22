use async_graphql::CustomValidator;

#[derive(Debug, Copy, Clone)]
pub struct EmoteNameValidator;

impl CustomValidator<String> for EmoteNameValidator {
	fn check(&self, value: &String) -> Result<(), async_graphql::InputValueError<String>> {
		if check_emote_name(value) {
			Ok(())
		} else {
			Err(async_graphql::InputValueError::custom("invalid emote name"))
		}
	}
}

pub fn check_emote_name(value: impl AsRef<str>) -> bool {
	static REGEX: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();

	REGEX
		.get_or_init(|| regex::Regex::new(r"^[\w\-():!+|.'?><\p{Emoji_Presentation}*#]{1,100}$").unwrap())
		.is_match(value.as_ref())
}

#[derive(Debug, Copy, Clone)]
pub struct NameValidator;

impl CustomValidator<String> for NameValidator {
	fn check(&self, value: &String) -> Result<(), async_graphql::InputValueError<String>> {
		if check_name(value) {
			Ok(())
		} else {
			Err(async_graphql::InputValueError::custom("invalid name"))
		}
	}
}

pub fn check_name(value: impl AsRef<str>) -> bool {
	static REGEX: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();

	REGEX
		.get_or_init(|| regex::Regex::new(r"^[\w\-():!+|.'?><\p{Emoji_Presentation}*# ]{1,100}$").unwrap())
		.is_match(value.as_ref())
}

#[derive(Debug, Copy, Clone)]
pub struct TagsValidator;

impl CustomValidator<Vec<String>> for TagsValidator {
	fn check(&self, value: &Vec<String>) -> Result<(), async_graphql::InputValueError<Vec<String>>> {
		if check_tags(value) {
			Ok(())
		} else {
			Err(async_graphql::InputValueError::custom("invalid tags"))
		}
	}
}

pub fn check_tag(value: impl AsRef<str>) -> bool {
	static REGEX: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();

	REGEX
		.get_or_init(|| regex::Regex::new(r"^\w{3,30}$").unwrap())
		.is_match(value.as_ref())
}

pub fn check_tags<S: AsRef<str>, I: ExactSizeIterator<Item = S>>(tags: impl IntoIterator<Item = S, IntoIter = I>) -> bool {
	let mut iter = tags.into_iter();
	iter.len() <= 6 && iter.all(check_tag)
}
