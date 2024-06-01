use async_graphql::SimpleObject;

#[derive(SimpleObject, Default)]
pub struct Mutation {
	pub hello: String,
}
