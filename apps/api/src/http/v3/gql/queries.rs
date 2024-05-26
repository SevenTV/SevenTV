use async_graphql::SimpleObject;

#[derive(SimpleObject, Default)]
pub struct Query {
    pub hello: String,
}
