use std::collections::HashSet;
use std::str::FromStr;

use futures::TryStreamExt;
use shared::database::entitlement::{EntitlementEdge, EntitlementEdgeId, EntitlementEdgeKind, EntitlementEdgeManagedBy};
use shared::database::product::special_event::SpecialEventId;
use shared::database::product::subscription::SubscriptionId;
use shared::database::product::SubscriptionProductId;
use shared::database::queries::filter;
use shared::database::user::UserId;
use shared::database::MongoCollection;

#[tokio::main]
async fn main() {
	let db = mongodb::Client::with_uri_str(std::env::var("MONGO_URI").unwrap().as_str())
		.await
		.unwrap()
		.default_database()
		.expect("no default database");

	println!("Connected to database");

	let minion_special_event = SpecialEventId::from_str("01JD4622NH7F4NEEAW4FMHR9FC").unwrap();

	let subscription_product_id = SubscriptionProductId::from_str("01FEVKBBTGRAT7FCY276TNTJ4A").unwrap();

	let list = std::fs::read_to_string("./minion.txt").unwrap();

	let user_ids = list
		.lines()
		.map(|line| line.trim())
		.filter(|line| !line.is_empty())
		.map(|line| line.parse().unwrap())
		.collect::<HashSet<UserId>>();

	let entitlements = EntitlementEdge::collection(&db)
		.find(filter::filter!{
			EntitlementEdge {
				#[query(flatten, rename = "_id")]
				id: EntitlementEdgeId {
					#[query(serde)]
					to: EntitlementEdgeKind::SpecialEvent { special_event_id: minion_special_event },
				},
			}
		})
		.await
		.unwrap()
		.try_collect::<Vec<_>>()
		.await
		.unwrap();

	let users_that_have_id = entitlements
		.iter()
		.map(|entitlement| match entitlement.id.from {
			EntitlementEdgeKind::Subscription { subscription_id } => subscription_id.user_id,
			_ => panic!("expected subscription"),
		})
		.collect::<HashSet<_>>();

	let users_that_have_not_id = user_ids.difference(&users_that_have_id);

	println!("{} users missing", users_that_have_not_id.clone().count());

	EntitlementEdge::collection(&db)
		.insert_many(users_that_have_not_id.map(|user_id| EntitlementEdge {
			id: EntitlementEdgeId {
				from: EntitlementEdgeKind::Subscription {
					subscription_id: SubscriptionId {
						product_id: subscription_product_id,
						user_id: *user_id,
					},
				},
				to: EntitlementEdgeKind::SpecialEvent {
					special_event_id: minion_special_event,
				},
				managed_by: Some(EntitlementEdgeManagedBy::SpecialEvent { special_event_id: minion_special_event })
			},
		}))
		.await
		.unwrap();

	println!("Inserted missing entitlements");
}
