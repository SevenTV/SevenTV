use std::str::FromStr;

use shared::database::entitlement::{EntitlementEdge, EntitlementEdgeId, EntitlementEdgeKind};
use shared::database::paint::PaintId;
use shared::database::product::{
	SubscriptionBenefit, SubscriptionBenefitCondition, SubscriptionBenefitId, SubscriptionProduct, SubscriptionProductId,
	TimePeriod,
};
use shared::database::queries::{filter, update};
use shared::database::MongoCollection;

#[tokio::main]
async fn main() {
	let db = mongodb::Client::with_uri_str(std::env::var("MONGO_URI").unwrap().as_str())
		.await
		.unwrap()
		.default_database()
		.expect("no default database");

	let paints = [
		PaintId::from_str("01JDMG16RAXNB7GX3899J3JJTZ").unwrap(),
		PaintId::from_str("01JDMG54FXCJ617Q8NG9QVY2JV").unwrap(),
		PaintId::from_str("01JDMG71M0AR032H7JS8TKPM6B").unwrap(),
		PaintId::from_str("01JDMGP7HXT3TVB29MSYPTYCZ4").unwrap(),
		PaintId::from_str("01JD0YJQ1Y0BSH1WF9CGGJYG1H").unwrap(),
	];

	let subscription_benefit = SubscriptionBenefitId::from_str("01JDMH6XKJTFRQTXA0147RGB8W").unwrap();

	let subscription_product_id = SubscriptionProductId::from_str("01FEVKBBTGRAT7FCY276TNTJ4A").unwrap();

	EntitlementEdge::collection(&db)
		.insert_many(paints.into_iter().map(|paint_id| EntitlementEdge {
			id: EntitlementEdgeId {
				from: EntitlementEdgeKind::SubscriptionBenefit {
					subscription_benefit_id: subscription_benefit,
				},
				to: EntitlementEdgeKind::Paint { paint_id },
				managed_by: None,
			},
		}))
		.await
		.unwrap();

	SubscriptionProduct::collection(&db)
		.update_one(
			filter::filter! {
				SubscriptionProduct {
					#[query(rename = "_id")]
					id: subscription_product_id,
				}
			},
			update::update! {
				#[query(push)]
				SubscriptionProduct {
					#[query(serde)]
					benefits: SubscriptionBenefit {
						id: subscription_benefit,
						name: "December 2024 Paint Bundle".to_owned(),
						condition: SubscriptionBenefitCondition::TimePeriod(TimePeriod {
							start: "2024-12-01T00:00:00Z".parse().unwrap(),
							end: "2025-01-01T00:00:00Z".parse().unwrap(),
						})
					},
				}
			},
		)
		.await
		.unwrap();
}
