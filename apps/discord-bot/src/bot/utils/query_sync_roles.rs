use std::sync::Arc;

use serenity::all::{Http, Member};
use shared::database::{
	entitlement::EntitlementEdgeKind,
	mongodb,
	queries::filter,
	user::{
		connection::{Platform, UserConnection},
		User,
	},
	MongoCollection,
};

use crate::global::Global;

use super::sync_roles::{sync_roles, SyncRolesError, SyncedRoles};

pub enum QuerySyncRolesError {
	FailedQuery(mongodb::error::Error),
	UserNotFound,
	SyncRole(SyncRolesError),
}

pub async fn query_sync_roles(
	global: &Arc<Global>,
	http: &Http,
	member: &Member,
) -> Result<SyncedRoles, QuerySyncRolesError> {
	let user = User::collection(&global.db)
		.find_one(filter::filter! {
			User {
				#[query(elem_match)]
				connections: UserConnection {
					platform: Platform::Discord,
					platform_id: member.user.id.to_string(),
				},
			}
		})
		.await
		.map_err(|err| QuerySyncRolesError::FailedQuery(err))?;

	match user {
		Some(user) => {
			let role_ids = user
				.cached
				.entitlements
				.iter()
				.filter_map(|e| {
					if let EntitlementEdgeKind::Role { role_id } = e {
						Some(role_id)
					} else {
						None
					}
				})
				.collect::<Vec<_>>();

			let synced_roles = sync_roles(&role_ids, member, http, global)
				.await
				.map_err(|err| QuerySyncRolesError::SyncRole(err))?;

			Ok(synced_roles)
		}
		None => Err(QuerySyncRolesError::UserNotFound),
	}
}
