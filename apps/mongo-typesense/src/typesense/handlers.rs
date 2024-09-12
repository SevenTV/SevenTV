use std::collections::HashSet;
use std::sync::Arc;

use anyhow::Context;
use bson::Document;
use chrono::Datelike;
use mongodb::change_stream::event::{ChangeStreamEvent, OperationType};
use shared::clickhouse::emote_stat::EmoteStat;
use shared::database::emote_set::EmoteSetId;
use shared::database::entitlement::{EntitlementEdgeId, EntitlementEdgeKind};
use shared::database::entitlement_edge::EntitlementEdgeGraphTraverse;
use shared::database::graph::{Direction, GraphTraverse};
use shared::database::product::special_event::SpecialEventId;
use shared::database::product::subscription::SubscriptionId;
use shared::database::product::{ProductId, SubscriptionProductId};
use shared::database::queries::{filter, update};
use shared::database::role::RoleId;
use shared::database::updater::MongoReq;
use shared::database::user::editor::UserEditorId;
use shared::database::user::relation::UserRelationId;
use shared::database::user::UserId;
use shared::database::{MongoCollection, SearchableMongoCollection};
use shared::typesense::types::{TypesenseCollection, TypesenseString};

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
	($batcher:ident, $mongo_collection:ty) => {
		impl SupportedMongoCollection for $mongo_collection {
			#[tracing::instrument(skip_all, fields(id))]
			async fn handle_delete(
				global: &Arc<Global>,
				id: Self::Id,
				_: ChangeStreamEvent<Document>,
			) -> anyhow::Result<()> {
				typesense_codegen::apis::documents_api::delete_document(
					&global.typesense,
					<$mongo_collection as SearchableMongoCollection>::Typesense::COLLECTION_NAME,
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

				let updated_at = data.updated_at;

				#[allow(irrefutable_let_patterns)]
				let Ok(data) = data.try_into() else {
					return Ok(());
				};

				global.$batcher.inserter.execute(data).await?;

				// Perhaps this could be a batcher?
				global
					.updater
					.update::<$mongo_collection>(
						filter::filter! {
							$mongo_collection {
								#[query(rename = "_id")]
								id: id,
								updated_at,
							}
						},
						update::update! {
							#[query(set)]
							$mongo_collection {
								search_updated_at: chrono::Utc::now(),
							}
						},
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

default_impl!(redeem_code_batcher, mongo::RedeemCode);
default_impl!(invoice_batcher, mongo::Invoice);
default_impl!(subscription_period_batcher, mongo::SubscriptionPeriod);
default_impl!(user_ban_template_batcher, mongo::UserBanTemplate);
default_impl!(user_ban_batcher, mongo::UserBan);
default_impl!(event_batcher, mongo::StoredEvent);
default_impl!(automod_rule_batcher, mongo::AutomodRule);
default_impl!(badge_batcher, mongo::Badge);
default_impl!(emote_moderation_request_batcher, mongo::EmoteModerationRequest);
default_impl!(emote_batcher, mongo::Emote);
default_impl!(page_batcher, mongo::Page);
default_impl!(paint_batcher, mongo::Paint);
default_impl!(ticket_batcher, mongo::Ticket);
default_impl!(ticket_message_batcher, mongo::TicketMessage);

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

		let updated_at = data.updated_at;

		global.user_editor_batcher.inserter.execute(data.into()).await?;

		global
			.updater
			.update(
				filter::filter! {
					mongo::UserEditor {
						#[query(rename = "_id", serde)]
						id: id,
						updated_at,
					}
				},
				update::update! {
					#[query(set)]
					mongo::UserEditor {
						search_updated_at: chrono::Utc::now(),
					}
				},
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

		let updated_at = data.updated_at;
		global.user_relation_batcher.inserter.execute(data.into()).await?;

		global
			.updater
			.update(
				filter::filter! {
					mongo::UserRelation {
						#[query(rename = "_id", serde)]
						id: id,
						updated_at,
					}
				},
				update::update! {
					#[query(set)]
					mongo::UserRelation {
						search_updated_at: chrono::Utc::now(),
					}
				},
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
	async fn handle_any(global: &Arc<Global>, id: UserId, change: ChangeStreamEvent<Document>) -> anyhow::Result<()> {
		let Ok(Some(mut data)) = global.user_batcher.loader.load(id).await else {
			anyhow::bail!("failed to load data");
		};

		if data.search_updated_at.is_some_and(|u| u > data.updated_at) {
			return Ok(());
		}

		let event_time = change.wall_time.map(|t| t.to_chrono()).unwrap_or_else(chrono::Utc::now);

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

		let old_role_rank = data.cached.role_rank;
		data.cached.role_rank = roles.values().map(|role| role.rank).max().unwrap_or(0);

		let old_role_hoist_rank = data.cached.role_hoist_rank;
		data.cached.role_hoist_rank = roles.values().filter(|r| r.hoist).map(|role| role.rank).max().unwrap_or(0);

		let old_entitlements = data.cached.entitlements.clone();
		data.cached.entitlements = granted_entitlements.iter().map(|edge| edge.id.to.clone()).collect();
		data.cached.entitlements.sort();
		data.cached.entitlements.dedup();

		let old_emotes = data.cached.active_emotes.clone();
		data.cached.active_emotes = emote_set
			.into_iter()
			.flat_map(|emote_set| emote_set.emotes)
			.map(|emote| emote.id)
			.collect();
		data.cached.active_emotes.sort();
		data.cached.active_emotes.dedup();

		let old_emote_set_id = data.cached.emote_set_id;
		data.cached.emote_set_id = data.style.active_emote_set_id;

		let update = update::update! {
			#[query(set)]
			mongo::User {
				search_updated_at: chrono::Utc::now(),
				#[query(flatten)]
				cached: mongo::UserCached {
					#[query(optional, serde)]
					entitlements: if old_entitlements != data.cached.entitlements {
						Some(&data.cached.entitlements)
					} else {
						None
					},
					#[query(optional, serde)]
					active_emotes: if old_emotes != data.cached.active_emotes {
						Some(&data.cached.active_emotes)
					} else {
						None
					},
					#[query(optional)]
					emote_set_id: if old_emote_set_id != data.cached.emote_set_id {
						Some(data.cached.emote_set_id)
					} else {
						None
					},
					#[query(optional)]
					role_rank: if old_role_rank != data.cached.role_rank {
						Some(data.cached.role_rank)
					} else {
						None
					},
					#[query(optional)]
					role_hoist_rank: if old_role_hoist_rank != data.cached.role_hoist_rank {
						Some(data.cached.role_hoist_rank)
					} else {
						None
					},
				}
			}
		};

		if old_emotes != data.cached.active_emotes {
			let old_emotes = old_emotes.iter().collect::<HashSet<_>>();
			let new_emotes = data.cached.active_emotes.iter().collect::<HashSet<_>>();

			let added = new_emotes.difference(&old_emotes).copied().copied();
			let removed = old_emotes.difference(&new_emotes).copied().copied();

			let today = event_time.date_naive();

			let date = time::Month::try_from(today.month0() as u8 + 1)
				.ok()
				.and_then(|month| time::Date::from_calendar_date(today.year(), month, today.day() as u8).ok());

			if let Some(date) = date {
				for output in global
					.emote_stats_batcher
					.execute_many(
						added
							.map(|id| (id, 1))
							.chain(removed.map(|id| (id, -1)))
							.map(|(emote_id, count)| EmoteStat { emote_id, count, date }),
					)
					.await
				{
					output?;
				}
			}
		}

		let updated_at = data.updated_at;

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

		global
			.updater
			.update(
				filter::filter! {
					mongo::User {
						#[query(rename = "_id")]
						id: id,
						updated_at,
					}
				},
				update,
				false,
			)
			.await
			.context("failed to update user")?;

		Ok(())
	}
}

impl SupportedMongoCollection for mongo::EntitlementEdge {
	#[tracing::instrument(skip_all, fields(to = %id.to, from = %id.from, managed_by = %id.managed_by.as_ref().map(|m| m.to_string()).unwrap_or_default()))]
	async fn handle_any(global: &Arc<Global>, id: EntitlementEdgeId, _: ChangeStreamEvent<Document>) -> anyhow::Result<()> {
		let now = chrono::Utc::now();
		let user_update = || {
			MongoReq::update(
				filter::filter! {
					mongo::User {
						#[query(flatten)]
						cached: mongo::UserCached {
							#[query(serde)]
							entitlements: &id.from,
						}
					}
				},
				update::update! {
					#[query(set)]
					mongo::User {
						updated_at: now,
					}
				},
				true,
			)
		};

		let updates = match &id.from {
			EntitlementEdgeKind::User { user_id } => {
				vec![MongoReq::update(
					filter::filter! {
						mongo::User {
							#[query(rename = "_id")]
							id: user_id,
						}
					},
					update::update! {
						#[query(set)]
						mongo::User {
							updated_at: now,
						}
					},
					false,
				)]
			}
			EntitlementEdgeKind::Role { role_id } => vec![
				MongoReq::update(
					filter::filter! {
						mongo::Role {
							#[query(rename = "_id")]
							id: role_id,
						}
					},
					update::update! {
						#[query(set)]
						mongo::Role {
							updated_at: now,
						}
					},
					false,
				),
				user_update(),
			],
			EntitlementEdgeKind::SpecialEvent { special_event_id } => vec![
				MongoReq::update(
					filter::filter! {
						mongo::SpecialEvent {
							#[query(rename = "_id")]
							id: special_event_id,
						}
					},
					update::update! {
						#[query(set)]
						mongo::SpecialEvent {
							updated_at: now,
						}
					},
					false,
				),
				user_update(),
			],
			EntitlementEdgeKind::GlobalDefaultEntitlementGroup => {
				vec![MongoReq::update(
					filter::filter! {
						mongo::User {}
					},
					update::update! {
						#[query(set)]
						mongo::User {
							updated_at: now,
						}
					},
					true,
				)]
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
				vec![
					MongoReq::update(
						filter::filter! {
							mongo::Subscription {
								#[query(rename = "_id", serde)]
								id: subscription_id,
							}
						},
						update::update! {
							#[query(set)]
							mongo::Subscription {
								updated_at: now,
							}
						},
						false,
					),
					user_update(),
				]
			}
			EntitlementEdgeKind::SubscriptionBenefit { subscription_benefit_id } => {
				vec![
					MongoReq::update(
						filter::filter! {
							mongo::SubscriptionProduct {
								#[query(flatten)]
								benefits: mongo::SubscriptionBenefit {
									id: subscription_benefit_id,
								}
							}
						},
						update::update! {
							#[query(set)]
							mongo::SubscriptionProduct {
								updated_at: now,
							}
						},
						false,
					),
					user_update(),
				]
			}
			EntitlementEdgeKind::Product { product_id } => {
				vec![
					MongoReq::update(
						filter::filter! {
							mongo::Product {
								#[query(rename = "_id")]
								id: product_id,
							}
						},
						update::update! {
							#[query(set)]
							mongo::Product {
								updated_at: now,
							}
						},
						false,
					),
					user_update(),
				]
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

		let updated_at = data.updated_at;

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

		global
			.updater
			.update(
				filter::filter! {
					mongo::Product {
						#[query(rename = "_id")]
						id: id,
						updated_at,
					}
				},
				update::update! {
					#[query(set)]
					mongo::Product {
						search_updated_at: chrono::Utc::now(),
					}
				},
				false,
			)
			.await
			.context("failed to update product")?;

		Ok(())
	}
}

impl SupportedMongoCollection for mongo::SubscriptionProduct {
	async fn handle_delete(
		global: &Arc<Global>,
		id: SubscriptionProductId,
		_: ChangeStreamEvent<Document>,
	) -> anyhow::Result<()> {
		typesense_codegen::apis::documents_api::delete_document(
			&global.typesense,
			typesense::SubscriptionProduct::COLLECTION_NAME,
			&id.to_string(),
		)
		.await
		.context("failed to delete document")?;
		Ok(())
	}

	#[tracing::instrument(skip_all, fields(id))]
	async fn handle_any(
		global: &Arc<Global>,
		id: SubscriptionProductId,
		_: ChangeStreamEvent<Document>,
	) -> anyhow::Result<()> {
		let Ok(Some(data)) = global.subscription_product_batcher.loader.load(id).await else {
			anyhow::bail!("failed to load data");
		};

		if data.search_updated_at.is_some_and(|u| u > data.updated_at) {
			return Ok(());
		}

		let updated_at = data.updated_at;

		let granted_entitlements = global
			.entitlement_outbound_loader
			.load_many(data.benefits.iter().map(|b| EntitlementEdgeKind::SubscriptionBenefit {
				subscription_benefit_id: b.id,
			}))
			.await
			.map_err(|()| anyhow::anyhow!("failed to load entitlements"))?
			.into_values()
			.flatten()
			.map(|e| e.id.to)
			.collect::<HashSet<_>>();

		global
			.subscription_product_batcher
			.inserter
			.execute(typesense::SubscriptionProduct::from_db(data, granted_entitlements))
			.await?;

		global
			.updater
			.update(
				filter::filter! {
					mongo::SubscriptionProduct {
						#[query(rename = "_id")]
						id,
						updated_at,
					}
				},
				update::update! {
					#[query(set)]
					mongo::SubscriptionProduct {
						search_updated_at: chrono::Utc::now(),
					}
				},
				false,
			)
			.await
			.context("failed to update product")?;

		Ok(())
	}
}

impl SupportedMongoCollection for mongo::Subscription {
	async fn handle_delete(global: &Arc<Global>, id: SubscriptionId, _: ChangeStreamEvent<Document>) -> anyhow::Result<()> {
		typesense_codegen::apis::documents_api::delete_document(
			&global.typesense,
			typesense::SubscriptionProduct::COLLECTION_NAME,
			&id.to_string(),
		)
		.await
		.context("failed to delete document")?;
		Ok(())
	}

	#[tracing::instrument(skip_all, fields(id))]
	async fn handle_any(global: &Arc<Global>, id: SubscriptionId, _: ChangeStreamEvent<Document>) -> anyhow::Result<()> {
		let Ok(Some(data)) = global.subscription_batcher.loader.load(id).await else {
			anyhow::bail!("failed to load data");
		};

		if data.search_updated_at.is_some_and(|u| u > data.updated_at) {
			return Ok(());
		}

		let updated_at = data.updated_at;

		let granted_entitlements = global
			.entitlement_outbound_loader
			.load(EntitlementEdgeKind::Subscription { subscription_id: id })
			.await
			.map_err(|()| anyhow::anyhow!("failed to load entitlements"))?
			.unwrap_or_default();

		global
			.subscription_batcher
			.inserter
			.execute(typesense::Subscription::from_db(
				data,
				granted_entitlements.iter().map(|e| e.id.to.clone()),
			))
			.await?;

		global
			.updater
			.update(
				filter::filter! {
					mongo::Subscription {
						#[query(rename = "_id", serde)]
						id,
						updated_at,
					}
				},
				update::update! {
					#[query(set)]
					mongo::Subscription {
						search_updated_at: chrono::Utc::now(),
					}
				},
				false,
			)
			.await
			.context("failed to update product")?;

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
		let Ok(Some(data)) = global.role_batcher.loader.load(id).await else {
			anyhow::bail!("failed to load data");
		};

		if data.search_updated_at.is_some_and(|u| u > data.updated_at) {
			return Ok(());
		}

		let updated_at = data.updated_at;
		let update_rank = if data.applied_rank.is_some_and(|r| r == data.rank) {
			None
		} else {
			Some(data.rank)
		};

		let granted_entitlements = global
			.entitlement_outbound_loader
			.load(EntitlementEdgeKind::Role { role_id: id })
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

		let now = chrono::Utc::now();

		let mut updates = vec![MongoReq::update(
			filter::filter! {
				mongo::Role {
					#[query(rename = "_id")]
					id: id,
					updated_at,
				}
			},
			update::update! {
				#[query(set)]
				mongo::Role {
					#[query(optional)]
					applied_rank: update_rank,
					search_updated_at: now,
				}
			},
			false,
		)];

		if update_rank.is_some() {
			updates.push(MongoReq::update(
				filter::filter! {
					mongo::User {
						#[query(flatten)]
						cached: mongo::UserCached {
							#[query(serde)]
							entitlements: &EntitlementEdgeKind::Role { role_id: id },
						}
					}
				},
				update::update! {
					#[query(set)]
					mongo::User {
						updated_at: now,
					}
				},
				true,
			));
		}

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

impl SupportedMongoCollection for mongo::SpecialEvent {
	async fn handle_delete(global: &Arc<Global>, id: SpecialEventId, _: ChangeStreamEvent<Document>) -> anyhow::Result<()> {
		typesense_codegen::apis::documents_api::delete_document(
			&global.typesense,
			typesense::SpecialEvent::COLLECTION_NAME,
			&id.to_string(),
		)
		.await
		.context("failed to delete document")?;
		Ok(())
	}

	#[tracing::instrument(skip_all, fields(id))]
	async fn handle_any(global: &Arc<Global>, id: SpecialEventId, _: ChangeStreamEvent<Document>) -> anyhow::Result<()> {
		let Ok(Some(data)) = global.special_event_batcher.loader.load(id).await else {
			anyhow::bail!("failed to load data");
		};

		if data.search_updated_at.is_some_and(|u| u > data.updated_at) {
			return Ok(());
		}

		let updated_at = data.updated_at;

		let granted_entitlements = global
			.entitlement_outbound_loader
			.load(EntitlementEdgeKind::SpecialEvent { special_event_id: id })
			.await
			.map_err(|()| anyhow::anyhow!("failed to load entitlements"))?
			.unwrap_or_default();

		global
			.special_event_batcher
			.inserter
			.execute(typesense::SpecialEvent::from_db(
				data,
				granted_entitlements.into_iter().map(|edge| edge.id.to),
			))
			.await?;

		global
			.updater
			.update(
				filter::filter! {
					mongo::SpecialEvent {
						#[query(rename = "_id")]
						id: id,
						updated_at,
					}
				},
				update::update! {
					#[query(set)]
					mongo::SpecialEvent {
						search_updated_at: chrono::Utc::now(),
					}
				},
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
		let Ok(Some(data)) = global.emote_set_batcher.loader.load(id).await else {
			anyhow::bail!("failed to load data");
		};

		if data.search_updated_at.is_some_and(|u| u > data.updated_at) {
			return Ok(());
		}

		let emotes_changed = data.emotes_changed_since_reindex;
		let updated_at = data.updated_at;

		global.emote_set_batcher.inserter.execute(data.into()).await?;

		let now = chrono::Utc::now();

		let updates = if emotes_changed {
			vec![
				MongoReq::update(
					filter::filter! {
						mongo::EmoteSet {
							#[query(rename = "_id")]
							id: id,
							updated_at,
						}
					},
					update::update! {
						#[query(set)]
						mongo::EmoteSet {
							emotes_changed_since_reindex: false,
							search_updated_at: now,
						}
					},
					false,
				),
				MongoReq::update(
					filter::filter! {
						mongo::User {
							#[query(flatten)]
							style: mongo::UserStyle {
								active_emote_set_id: id,
							}
						}
					},
					update::update! {
						#[query(set)]
						mongo::User {
							updated_at: now,
						}
					},
					true,
				),
				MongoReq::update(
					filter::filter! {
						mongo::EmoteSet {
							#[query(flatten)]
							origin_config: mongo::EmoteSetOriginConfig {
								#[query(flatten)]
								origins: mongo::EmoteSetOrigin {
									id,
								}
							}
						}
					},
					update::update! {
						#[query(set)]
						mongo::EmoteSet {
							updated_at: now,
							#[query(flatten)]
							origin_config: mongo::EmoteSetOriginConfig {
								needs_resync: false,
							}
						}
					},
					true,
				),
			]
		} else {
			vec![MongoReq::update(
				filter::filter! {
					mongo::EmoteSet {
						#[query(rename = "_id")]
						id: id,
						updated_at,
					}
				},
				update::update! {
					#[query(set)]
					mongo::EmoteSet {
						search_updated_at: now,
					}
				},
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
