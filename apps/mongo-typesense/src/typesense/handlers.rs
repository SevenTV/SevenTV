use std::sync::Arc;

use anyhow::Context;
use bson::Document;
use mongodb::change_stream::event::{ChangeStreamEvent, OperationType};
use shared::database::emote_set::EmoteSetId;
use shared::database::entitlement::{EntitlementEdgeId, EntitlementEdgeKind, EntitlementGroupId};
use shared::database::entitlement_edge::EntitlementEdgeGraphTraverse;
use shared::database::graph::{Direction, GraphTraverse};
use shared::database::product::promotion::PromotionId;
use shared::database::product::ProductId;
use shared::database::role::RoleId;
use shared::database::user::editor::UserEditorId;
use shared::database::user::relation::UserRelationId;
use shared::database::user::UserId;
use shared::database::MongoCollection;
use shared::typesense::types::{TypesenseCollection, TypesenseString};

use crate::batcher::updater::MongoReq;
use crate::global::Global;
use crate::types::{mongo, typesense};

pub async fn process<M: SupportedMongoCollection>(
	global: &Arc<Global>,
	message: ChangeStreamEvent<Document>,
) -> anyhow::Result<bool> {
	if M::skip(&message) {
		return Ok(false);
	}

	let id = parse_document_key::<M>(&message)?;

	match message.operation_type {
		OperationType::Delete => M::handle_delete(global, id, message).await,
		OperationType::Insert => M::handle_insert(global, id, message).await,
		OperationType::Replace => M::handle_replace(global, id, message).await,
		OperationType::Update => M::handle_update(global, id, message).await,
		_ => M::handle_any(global, id, message).await,
	}?;

	Ok(true)
}

pub trait SupportedMongoCollection: MongoCollection + Send + Sync {
	fn skip(message: &ChangeStreamEvent<Document>) -> bool {
		!matches!(
			message.operation_type,
			OperationType::Delete | OperationType::Insert | OperationType::Replace | OperationType::Update
		)
	}

	fn handle_delete(
		global: &Arc<Global>,
		id: Self::Id,
		message: ChangeStreamEvent<Document>,
	) -> impl std::future::Future<Output = anyhow::Result<()>> + Send {
		Self::handle_any(global, id, message)
	}

	fn handle_insert(
		global: &Arc<Global>,
		id: Self::Id,
		message: ChangeStreamEvent<Document>,
	) -> impl std::future::Future<Output = anyhow::Result<()>> + Send {
		Self::handle_any(global, id, message)
	}

	fn handle_replace(
		global: &Arc<Global>,
		id: Self::Id,
		message: ChangeStreamEvent<Document>,
	) -> impl std::future::Future<Output = anyhow::Result<()>> + Send {
		Self::handle_any(global, id, message)
	}

	fn handle_update(
		global: &Arc<Global>,
		id: Self::Id,
		message: ChangeStreamEvent<Document>,
	) -> impl std::future::Future<Output = anyhow::Result<()>> + Send {
		Self::handle_any(global, id, message)
	}

	fn handle_any(
		global: &Arc<Global>,
		id: Self::Id,
		message: ChangeStreamEvent<Document>,
	) -> impl std::future::Future<Output = anyhow::Result<()>> + Send;
}

fn parse_document_key<M: SupportedMongoCollection>(msg: &ChangeStreamEvent<Document>) -> anyhow::Result<M::Id> {
	let id = msg
		.document_key
		.as_ref()
		.context("missing document key")?
		.get("_id")
		.context("missing id")?;

	bson::from_bson(id.clone()).context("failed to deserialize document key")
}

macro_rules! default_impl {
	($batcher:ident, $typesense_collection:ty, $mongo_collection:ty) => {
		impl SupportedMongoCollection for $mongo_collection {
			#[tracing::instrument(skip_all, fields(id))]
			async fn handle_delete(
				global: &Arc<Global>,
				id: Self::Id,
				_: ChangeStreamEvent<Document>,
			) -> anyhow::Result<()> {
				typesense_codegen::apis::documents_api::delete_document(
					&global.typesense,
					<$typesense_collection as TypesenseCollection>::COLLECTION_NAME,
					&id.to_string(),
				)
				.await
				.context("failed to delete document")?;
				Ok(())
			}

			#[tracing::instrument(skip_all, fields(id))]
			async fn handle_any(global: &Arc<Global>, id: Self::Id, _: ChangeStreamEvent<Document>) -> anyhow::Result<()> {
				let Ok(Some(data)) = global.$batcher.loader.load(id.clone()).await else {
					anyhow::bail!("failed to load data");
				};

				if data.search_updated_at.is_some_and(|u| u > data.updated_at) {
					return Ok(());
				}

				let updated_at = bson::DateTime::from_chrono(data.updated_at);

				let Ok(data) = data.try_into() else {
					return Ok(());
				};

				global.$batcher.inserter.execute(data).await?;

				let now = bson::DateTime::from_chrono(chrono::Utc::now());

				// Perhaps this could be a batcher?
				global
					.updater
					.update::<$mongo_collection>(
						bson::doc! { "_id": id, "updated_at": updated_at },
						bson::doc! { "$set": { "search_updated_at": now } },
						false,
					)
					.await
					.with_context(|| {
						format!(
							"failed to update {}",
							<$mongo_collection as MongoCollection>::COLLECTION_NAME
						)
					})?;

				Ok(())
			}
		}
	};
}

default_impl!(discount_code_batcher, typesense::DiscountCode, mongo::DiscountCode);
default_impl!(gift_code_batcher, typesense::GiftCode, mongo::GiftCode);
default_impl!(redeem_code_batcher, typesense::RedeemCode, mongo::RedeemCode);
default_impl!(special_event_batcher, typesense::SpecialEvent, mongo::SpecialEvent);
default_impl!(invoice_batcher, typesense::Invoice, mongo::Invoice);
default_impl!(
	subscription_period_batcher,
	typesense::SubscriptionPeriod,
	mongo::SubscriptionPeriod
);
default_impl!(user_ban_template_batcher, typesense::UserBanTemplate, mongo::UserBanTemplate);
default_impl!(user_ban_batcher, typesense::UserBan, mongo::UserBan);
default_impl!(event_batcher, typesense::Event, mongo::StoredEvent);
default_impl!(automod_rule_batcher, typesense::AutomodRule, mongo::AutomodRule);
default_impl!(badge_batcher, typesense::Badge, mongo::Badge);
default_impl!(
	emote_moderation_request_batcher,
	typesense::EmoteModerationRequest,
	mongo::EmoteModerationRequest
);
default_impl!(emote_batcher, typesense::Emote, mongo::Emote);
default_impl!(page_batcher, typesense::Page, mongo::Page);
default_impl!(paint_batcher, typesense::Paint, mongo::Paint);
default_impl!(ticket_batcher, typesense::Ticket, mongo::Ticket);
default_impl!(ticket_message_batcher, typesense::TicketMessage, mongo::TicketMessage);

impl SupportedMongoCollection for mongo::UserEditor {
	#[tracing::instrument(skip_all, fields(user_id = %id.user_id, editor_id = %id.editor_id))]
	async fn handle_delete(global: &Arc<Global>, id: UserEditorId, _: ChangeStreamEvent<Document>) -> anyhow::Result<()> {
		typesense_codegen::apis::documents_api::delete_document(
			&global.typesense,
			typesense::UserEditor::COLLECTION_NAME,
			&TypesenseString(id).to_string(),
		)
		.await
		.context("failed to delete document")?;
		Ok(())
	}

	#[tracing::instrument(skip_all, fields(user_id = %id.user_id, editor_id = %id.editor_id))]
	async fn handle_any(global: &Arc<Global>, id: UserEditorId, _: ChangeStreamEvent<Document>) -> anyhow::Result<()> {
		let Ok(Some(data)) = global.user_editor_batcher.loader.load(id).await else {
			anyhow::bail!("failed to load data");
		};

		if data.search_updated_at.is_some_and(|u| u > data.updated_at) {
			return Ok(());
		}

		let updated_at = bson::DateTime::from_chrono(data.updated_at);

		global.user_editor_batcher.inserter.execute(data.into()).await?;
		let now = bson::DateTime::from_chrono(chrono::Utc::now());

		global
			.updater
			.update::<mongo::UserEditor>(
				bson::doc! { "_id": bson::to_bson(&id)?, "updated_at": updated_at },
				bson::doc! { "$set": { "search_updated_at": now } },
				false,
			)
			.await
			.context("failed to update user editor")?;

		Ok(())
	}
}

impl SupportedMongoCollection for mongo::UserRelation {
	async fn handle_delete(global: &Arc<Global>, id: UserRelationId, _: ChangeStreamEvent<Document>) -> anyhow::Result<()> {
		typesense_codegen::apis::documents_api::delete_document(
			&global.typesense,
			typesense::UserRelation::COLLECTION_NAME,
			&TypesenseString(id).to_string(),
		)
		.await
		.context("failed to delete document")?;
		Ok(())
	}

	#[tracing::instrument(skip_all, fields(user_id = %id.user_id, other_user_id = %id.other_user_id))]
	async fn handle_any(global: &Arc<Global>, id: UserRelationId, _: ChangeStreamEvent<Document>) -> anyhow::Result<()> {
		let Ok(Some(data)) = global.user_relation_batcher.loader.load(id).await else {
			anyhow::bail!("failed to load data");
		};

		if data.search_updated_at.is_some_and(|u| u > data.updated_at) {
			return Ok(());
		}

		let updated_at = bson::DateTime::from_chrono(data.updated_at);
		global.user_relation_batcher.inserter.execute(data.into()).await?;
		let now = bson::DateTime::from_chrono(chrono::Utc::now());

		global
			.updater
			.update::<mongo::UserRelation>(
				bson::doc! { "_id": bson::to_bson(&id)?, "updated_at": updated_at },
				bson::doc! { "$set": { "search_updated_at": now } },
				false,
			)
			.await
			.context("failed to update user relation")?;

		Ok(())
	}
}

impl SupportedMongoCollection for mongo::User {
	async fn handle_delete(global: &Arc<Global>, id: UserId, _: ChangeStreamEvent<Document>) -> anyhow::Result<()> {
		typesense_codegen::apis::documents_api::delete_document(
			&global.typesense,
			typesense::User::COLLECTION_NAME,
			&id.to_string(),
		)
		.await
		.context("failed to delete document")?;
		Ok(())
	}

	#[tracing::instrument(skip_all, fields(id))]
	async fn handle_any(global: &Arc<Global>, id: UserId, _: ChangeStreamEvent<Document>) -> anyhow::Result<()> {
		let Ok(Some(mut data)) = global.user_batcher.loader.load(id).await else {
			anyhow::bail!("failed to load data");
		};

		if data.search_updated_at.is_some_and(|u| u > data.updated_at) {
			return Ok(());
		}

		let traverse = EntitlementEdgeGraphTraverse {
			inbound_loader: &global.entitlement_inbound_loader,
			outbound_loader: &global.entitlement_outbound_loader,
		};

		let granted_entitlements = traverse
			.traversal(
				Direction::Outbound,
				[
					EntitlementEdgeKind::User { user_id: id },
					EntitlementEdgeKind::GlobalDefaultEntitlementGroup,
				],
			)
			.await
			.map_err(|()| anyhow::anyhow!("failed to load entitlements"))?;

		let emote_set = if let Some(active_emote_set_id) = data.style.active_emote_set_id {
			global
				.emote_set_batcher
				.loader
				.load(active_emote_set_id)
				.await
				.map_err(|()| anyhow::anyhow!("failed to load emote set"))?
		} else {
			None
		};

		let roles = global
			.role_batcher
			.loader
			.load_many(granted_entitlements.iter().filter_map(|edge| match edge.id.to {
				EntitlementEdgeKind::Role { role_id } => Some(role_id),
				_ => None,
			}))
			.await
			.map_err(|()| anyhow::anyhow!("failed to load roles"))?;

		let old_role_rank = data.cached_role_rank;
		data.cached_role_rank = roles.values().map(|role| role.rank).max().unwrap_or(0);

		let old_entitlements = data.cached_entitlements.clone();
		data.cached_entitlements = granted_entitlements.iter().map(|edge| edge.id.to.clone()).collect();
		data.cached_entitlements.sort();
		data.cached_entitlements.dedup();

		let old_emotes = data.cached_active_emotes.clone();
		data.cached_active_emotes = emote_set
			.into_iter()
			.flat_map(|emote_set| emote_set.emotes)
			.map(|emote| emote.id)
			.collect();
		data.cached_active_emotes.sort();
		data.cached_active_emotes.dedup();

		let now = bson::DateTime::from_chrono(chrono::Utc::now());
		let mut update = bson::doc! { "search_updated_at": now };

		if old_entitlements != data.cached_entitlements {
			update.insert("cached_entitlements", bson::to_bson(&data.cached_entitlements)?);
		}

		if old_emotes != data.cached_active_emotes {
			update.insert("cached_active_emotes", bson::to_bson(&data.cached_active_emotes)?);
		}

		if old_role_rank != data.cached_role_rank {
			update.insert("cached_role_rank", data.cached_role_rank);
		}

		let updated_at = bson::DateTime::from_chrono(data.updated_at);

		global
			.user_batcher
			.inserter
			.execute(typesense::User::from_db(
				data,
				// This field is specifically used to filter out the user's own entitlements (things which are directly
				// granted to them, not via some other entity)
				granted_entitlements
					.into_iter()
					.filter(|edge| edge.id.from == EntitlementEdgeKind::User { user_id: id })
					.map(|edge| edge.id.to),
			))
			.await?;

		if global
			.updater
			.update::<mongo::User>(
				bson::doc! { "_id": id, "updated_at": updated_at },
				bson::doc! { "$set": update },
				false,
			)
			.await
			.context("failed to update user")?
		{}

		Ok(())
	}
}

impl SupportedMongoCollection for mongo::EntitlementEdge {
	#[tracing::instrument(skip_all, fields(to = %id.to, from = %id.from, managed_by = %id.managed_by.as_ref().map(|m| m.to_string()).unwrap_or_default()))]
	async fn handle_any(global: &Arc<Global>, id: EntitlementEdgeId, _: ChangeStreamEvent<Document>) -> anyhow::Result<()> {
		let now = bson::DateTime::from_chrono(chrono::Utc::now());
		let update = bson::doc! { "$set": { "updated_at": now } };

		let updates = match &id.from {
			EntitlementEdgeKind::User { user_id } => {
				vec![MongoReq::update::<mongo::User>(bson::doc! { "_id": user_id }, update, false)]
			}
			EntitlementEdgeKind::Role { role_id } => vec![
				MongoReq::update::<mongo::Role>(bson::doc! { "_id": role_id }, update.clone(), false),
				MongoReq::update::<mongo::User>(
					bson::doc! { "cached_entitlements": bson::to_bson(&id.from)? },
					update,
					true,
				),
			],
			EntitlementEdgeKind::Product { product_id } => vec![
				MongoReq::update::<mongo::Product>(bson::doc! { "_id": product_id }, update.clone(), false),
				MongoReq::update::<mongo::User>(
					bson::doc! { "cached_entitlements": bson::to_bson(&id.from)? },
					update,
					true,
				),
			],
			EntitlementEdgeKind::SubscriptionProduct { product_id } => vec![
				MongoReq::update::<mongo::SubscriptionProduct>(bson::doc! { "_id": product_id }, update.clone(), false),
				MongoReq::update::<mongo::User>(
					bson::doc! { "cached_entitlements": bson::to_bson(&id.from)? },
					update,
					true,
				),
			],
			EntitlementEdgeKind::Promotion { promotion_id } => vec![
				MongoReq::update::<mongo::Promotion>(bson::doc! { "_id": promotion_id }, update.clone(), false),
				MongoReq::update::<mongo::User>(
					bson::doc! { "cached_entitlements": bson::to_bson(&id.from)? },
					update,
					true,
				),
			],
			EntitlementEdgeKind::EntitlementGroup { entitlement_group_id } => vec![
				MongoReq::update::<mongo::EntitlementGroup>(
					bson::doc! { "_id": entitlement_group_id },
					update.clone(),
					false,
				),
				MongoReq::update::<mongo::User>(
					bson::doc! { "cached_entitlements": bson::to_bson(&id.from)? },
					update,
					true,
				),
			],
			EntitlementEdgeKind::GlobalDefaultEntitlementGroup => {
				vec![MongoReq::update::<mongo::User>(bson::doc! {}, update, true)]
			}
			EntitlementEdgeKind::Badge { .. } => {
				tracing::warn!("badge has child entitlements");
				vec![]
			}
			EntitlementEdgeKind::Paint { .. } => {
				tracing::warn!("paint has child entitlements");
				vec![]
			}
			EntitlementEdgeKind::EmoteSet { .. } => {
				tracing::warn!("emote set has child entitlements");
				vec![]
			}
			EntitlementEdgeKind::Subscription { subscription_id } => {
				// TODO: subscription
				todo!()
			}
		};

		global
			.updater
			.bulk(updates)
			.await
			.into_iter()
			.collect::<Result<Vec<_>, _>>()
			.context("failed to update entitlements")?;

		Ok(())
	}
}

impl SupportedMongoCollection for mongo::Product {
	async fn handle_delete(global: &Arc<Global>, id: ProductId, _: ChangeStreamEvent<Document>) -> anyhow::Result<()> {
		typesense_codegen::apis::documents_api::delete_document(
			&global.typesense,
			typesense::Product::COLLECTION_NAME,
			&id.to_string(),
		)
		.await
		.context("failed to delete document")?;
		Ok(())
	}

	#[tracing::instrument(skip_all, fields(id))]
	async fn handle_any(global: &Arc<Global>, id: ProductId, _: ChangeStreamEvent<Document>) -> anyhow::Result<()> {
		let Ok(Some(data)) = global.product_batcher.loader.load(id.clone()).await else {
			anyhow::bail!("failed to load data");
		};

		if data.search_updated_at.is_some_and(|u| u > data.updated_at) {
			return Ok(());
		}

		let updated_at = bson::DateTime::from_chrono(data.updated_at);

		let granted_entitlements = global
			.entitlement_outbound_loader
			.load(EntitlementEdgeKind::Product { product_id: id.clone() })
			.await
			.map_err(|()| anyhow::anyhow!("failed to load entitlements"))?
			.unwrap_or_default();

		global
			.product_batcher
			.inserter
			.execute(typesense::Product::from_db(
				data,
				granted_entitlements.into_iter().map(|edge| edge.id.to),
			))
			.await?;

		let now = bson::DateTime::from_chrono(chrono::Utc::now());

		global
			.updater
			.update::<mongo::Product>(
				bson::doc! { "_id": id, "updated_at": updated_at },
				bson::doc! { "$set": { "search_updated_at": now } },
				false,
			)
			.await
			.context("failed to update product")?;

		Ok(())
	}
}

impl SupportedMongoCollection for mongo::Promotion {
	async fn handle_delete(global: &Arc<Global>, id: PromotionId, _: ChangeStreamEvent<Document>) -> anyhow::Result<()> {
		typesense_codegen::apis::documents_api::delete_document(
			&global.typesense,
			typesense::Promotion::COLLECTION_NAME,
			&id.to_string(),
		)
		.await
		.context("failed to delete document")?;
		Ok(())
	}

	#[tracing::instrument(skip_all, fields(id))]
	async fn handle_any(global: &Arc<Global>, id: PromotionId, _: ChangeStreamEvent<Document>) -> anyhow::Result<()> {
		let Ok(Some(data)) = global.promotion_batcher.loader.load(id.clone()).await else {
			anyhow::bail!("failed to load data");
		};

		if data.search_updated_at.is_some_and(|u| u > data.updated_at) {
			return Ok(());
		}

		let updated_at = bson::DateTime::from_chrono(data.updated_at);

		let granted_entitlements = global
			.entitlement_outbound_loader
			.load(EntitlementEdgeKind::Promotion {
				promotion_id: id.clone(),
			})
			.await
			.map_err(|()| anyhow::anyhow!("failed to load entitlements"))?
			.unwrap_or_default();

		global
			.promotion_batcher
			.inserter
			.execute(typesense::Promotion::from_db(
				data,
				granted_entitlements.into_iter().map(|edge| edge.id.to),
			))
			.await?;

		let now = bson::DateTime::from_chrono(chrono::Utc::now());

		global
			.updater
			.update::<mongo::Promotion>(
				bson::doc! { "_id": id, "updated_at": updated_at },
				bson::doc! { "$set": { "search_updated_at": now } },
				false,
			)
			.await
			.context("failed to update promotion")?;

		Ok(())
	}
}

impl SupportedMongoCollection for mongo::Role {
	async fn handle_delete(global: &Arc<Global>, id: RoleId, _: ChangeStreamEvent<Document>) -> anyhow::Result<()> {
		typesense_codegen::apis::documents_api::delete_document(
			&global.typesense,
			typesense::Role::COLLECTION_NAME,
			&id.to_string(),
		)
		.await
		.context("failed to delete document")?;
		Ok(())
	}

	#[tracing::instrument(skip_all, fields(id))]
	async fn handle_any(global: &Arc<Global>, id: RoleId, _: ChangeStreamEvent<Document>) -> anyhow::Result<()> {
		let Ok(Some(data)) = global.role_batcher.loader.load(id.clone()).await else {
			anyhow::bail!("failed to load data");
		};

		if data.search_updated_at.is_some_and(|u| u > data.updated_at) {
			return Ok(());
		}

		let updated_at = bson::DateTime::from_chrono(data.updated_at);
		let update_rank = if data.applied_rank.is_some_and(|r| r == data.rank) {
			None
		} else {
			Some(data.rank)
		};

		let granted_entitlements = global
			.entitlement_outbound_loader
			.load(EntitlementEdgeKind::Role { role_id: id.clone() })
			.await
			.map_err(|()| anyhow::anyhow!("failed to load entitlements"))?
			.unwrap_or_default();

		global
			.role_batcher
			.inserter
			.execute(typesense::Role::from_db(
				data,
				granted_entitlements.into_iter().map(|edge| edge.id.to),
			))
			.await?;

		let now = bson::DateTime::from_chrono(chrono::Utc::now());

		let updates = if let Some(rank) = update_rank {
			vec![
				MongoReq::update::<mongo::Role>(
					bson::doc! { "_id": id, "updated_at": updated_at },
					bson::doc! { "$set": { "applied_rank": rank, "search_updated_at": now } },
					false,
				),
				MongoReq::update::<mongo::User>(
					bson::doc! { "cached_entitlements": bson::to_bson(&EntitlementEdgeKind::Role { role_id: id })? },
					bson::doc! { "$set": { "updated_at": now } },
					true,
				),
			]
		} else {
			vec![MongoReq::update::<mongo::Role>(
				bson::doc! { "_id": id, "updated_at": updated_at },
				bson::doc! { "$set": { "search_updated_at": now } },
				false,
			)]
		};

		global
			.updater
			.bulk(updates)
			.await
			.into_iter()
			.collect::<Result<Vec<_>, _>>()
			.context("failed to update role")?;

		Ok(())
	}
}

impl SupportedMongoCollection for mongo::EntitlementGroup {
	async fn handle_delete(
		global: &Arc<Global>,
		id: EntitlementGroupId,
		_: ChangeStreamEvent<Document>,
	) -> anyhow::Result<()> {
		typesense_codegen::apis::documents_api::delete_document(
			&global.typesense,
			typesense::EntitlementGroup::COLLECTION_NAME,
			&id.to_string(),
		)
		.await
		.context("failed to delete document")?;
		Ok(())
	}

	#[tracing::instrument(skip_all, fields(id))]
	async fn handle_any(global: &Arc<Global>, id: EntitlementGroupId, _: ChangeStreamEvent<Document>) -> anyhow::Result<()> {
		let Ok(Some(data)) = global.entitlement_group_batcher.loader.load(id).await else {
			anyhow::bail!("failed to load data");
		};

		if data.search_updated_at.is_some_and(|u| u > data.updated_at) {
			return Ok(());
		}

		let updated_at = bson::DateTime::from_chrono(data.updated_at);

		let granted_entitlements = global
			.entitlement_outbound_loader
			.load(EntitlementEdgeKind::EntitlementGroup {
				entitlement_group_id: id,
			})
			.await
			.map_err(|()| anyhow::anyhow!("failed to load entitlements"))?
			.unwrap_or_default();

		global
			.entitlement_group_batcher
			.inserter
			.execute(typesense::EntitlementGroup::from_db(
				data,
				granted_entitlements.into_iter().map(|edge| edge.id.to),
			))
			.await?;

		let now = bson::DateTime::from_chrono(chrono::Utc::now());

		global
			.updater
			.update::<mongo::EntitlementGroup>(
				bson::doc! { "_id": id, "updated_at": updated_at },
				bson::doc! { "$set": { "search_updated_at": now } },
				false,
			)
			.await
			.context("failed to update entitlement group")?;

		Ok(())
	}
}

impl SupportedMongoCollection for mongo::EmoteSet {
	async fn handle_delete(global: &Arc<Global>, id: EmoteSetId, _: ChangeStreamEvent<Document>) -> anyhow::Result<()> {
		typesense_codegen::apis::documents_api::delete_document(
			&global.typesense,
			typesense::EmoteSet::COLLECTION_NAME,
			&id.to_string(),
		)
		.await
		.context("failed to delete document")?;
		Ok(())
	}

	#[tracing::instrument(skip_all, fields(id))]
	async fn handle_any(global: &Arc<Global>, id: EmoteSetId, _: ChangeStreamEvent<Document>) -> anyhow::Result<()> {
		let Ok(Some(data)) = global.emote_set_batcher.loader.load(id.clone()).await else {
			anyhow::bail!("failed to load data");
		};

		if data.search_updated_at.is_some_and(|u| u > data.updated_at) {
			return Ok(());
		}

		let emotes_changed = data.emotes_changed_since_reindex;
		let updated_at = bson::DateTime::from_chrono(data.updated_at);

		global.emote_set_batcher.inserter.execute(data.into()).await?;

		let now = bson::DateTime::from_chrono(chrono::Utc::now());

		let updates = if emotes_changed {
			vec![
				MongoReq::update::<mongo::EmoteSet>(
					bson::doc! { "_id": id, "updated_at": updated_at },
					bson::doc! { "$set": { "emotes_changed_since_reindex": false, "search_updated_at": now } },
					false,
				),
				MongoReq::update::<mongo::User>(
					bson::doc! { "style.active_emote_set_id": id },
					bson::doc! { "$set": { "updated_at": now } },
					true,
				),
				MongoReq::update::<mongo::EmoteSet>(
					bson::doc! { "origin_config.origins.id": id },
					bson::doc! { "$set": { "updated_at": now, "origins_config.needs_resync": false } },
					true,
				),
			]
		} else {
			vec![MongoReq::update::<mongo::EmoteSet>(
				bson::doc! { "_id": id, "updated_at": updated_at },
				bson::doc! { "$set": { "search_updated_at": now } },
				false,
			)]
		};

		global
			.updater
			.bulk(updates)
			.await
			.into_iter()
			.collect::<Result<Vec<_>, _>>()
			.context("failed to update emote set")?;

		Ok(())
	}
}
