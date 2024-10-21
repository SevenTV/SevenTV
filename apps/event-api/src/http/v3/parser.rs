use std::collections::HashMap;

use shared::event_api::payload;

/// Try parse the query from the query parameter `q`.
pub fn parse_query_uri(query: Option<String>) -> Option<String> {
	query
		.as_ref()
		.map(|q| url::form_urlencoded::parse(q.as_bytes()))
		.and_then(|mut uri| uri.find(|(key, _)| key == "q").map(|(_, value)| value.to_string()))
}

/// Parse the query as JSON.
pub fn parse_json_subscriptions(json: &str) -> Result<Vec<payload::Subscribe>, serde_json::Error> {
	serde_json::from_slice(json.as_bytes())
}

/// Parse the query as a path.
pub fn parse_path_subscriptions(uri: &str) -> Vec<payload::Subscribe> {
	let path = uri.split_once('@').map(|(_, path)| path).unwrap_or("");

	// The query is a very annoying format.
	// v3@entitlement.*<id=121903137;platform=TWITCH;ctx=channel>,cosmetic.*
	// <id=121903137;platform=TWITCH;ctx=channel>,emote_set.*
	// <object_id=65b0039ae6deb9b914c5caa8> We need to split it by `,`, then by `<`,
	// then by `;`. We do not support values that contain a `;` or `,` or `<` in
	// them. However none of those are used in the current query format.
	// This should be deprecated in favor of the JSON format above.

	path.split(',')
		.filter_map(|topic| {
			let topic = topic.trim();

			if topic.is_empty() {
				return None;
			}

			let (topic, condition) = topic
				.split_once('<')
				.map(|(topic, condition)| (topic, condition.strip_suffix('>')))
				.unwrap_or((topic, None));

			let ty = topic.parse().ok()?;

			let condition = condition
				.map(|condition| {
					let condition = condition.split(';');

					let mut conds = HashMap::new();

					for cond in condition {
						let (key, value) = cond.split_once('=').unwrap_or((cond, ""));

						conds.insert(key.to_owned(), value.to_owned());
					}

					conds
				})
				.unwrap_or_default();

			payload::Subscribe::new_from_hash(ty, &condition)
		})
		.collect()
}
