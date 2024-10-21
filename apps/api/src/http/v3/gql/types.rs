use async_graphql::Enum;

#[derive(Enum, Copy, Clone, PartialEq, Eq, Debug)]
#[graphql(rename_items = "SCREAMING_SNAKE_CASE")]
pub enum ListItemAction {
	Add,
	Update,
	Remove,
}
