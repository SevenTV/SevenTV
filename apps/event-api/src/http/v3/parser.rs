use std::collections::HashMap;

use hyper::body::Incoming;
use shared::event_api::payload;

const QUERY_HEADER_KEY: &str = "x-7tv-query";

/// Parses the query from the request, either from the `x-7tv-query` header or
/// the `q` query parameter.
pub fn parse_query(req: &hyper::Request<Incoming>) -> Result<Option<Vec<payload::Subscribe>>, serde_json::Error> {
	parse_query_header(req)
		.or_else(|| parse_query_uri(req))
		.map(|query| parse_json_subscriptions(&query))
		.transpose()
}

/// Try parse the query from the header `x-7tv-query`.
fn parse_query_header(req: &hyper::Request<Incoming>) -> Option<String> {
	req.headers()
		.get(QUERY_HEADER_KEY)
		.and_then(|v| v.to_str().ok().map(ToOwned::to_owned))
}

/// Try parse the query from the query parameter `q`.
fn parse_query_uri(req: &hyper::Request<Incoming>) -> Option<String> {
	req.uri()
		.query()
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

			Some(payload::Subscribe {
				ty,
				condition,
				ttl: None,
			})
		})
		.collect()
}
