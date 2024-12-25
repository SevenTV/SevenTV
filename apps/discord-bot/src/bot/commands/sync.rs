use std::sync::Arc;

use serenity::all::{CacheHttp, Colour, CommandInteraction, Context, CreateEmbed, EditInteractionResponse};
use serenity::builder::CreateCommand;

use crate::bot::utils::query_sync_roles::{query_sync_roles, QuerySyncRolesError};
use crate::global::Global;

pub async fn run(global: &Arc<Global>, ctx: &Context, command: &CommandInteraction) {
	let _ = command.defer(ctx).await;

	let response = match query_sync_roles(global, ctx.http(), command.member.as_ref().unwrap()).await {
		Ok(synced_roles) => {
			let mut embed = CreateEmbed::new()
				.title("Synced Roles")
				.description(format!("<@{}>", command.user.id))
				.colour(Colour::new(6356792));

			if !synced_roles.added.is_empty() {
				embed = embed.field(
					"Added Roles",
					synced_roles
						.added
						.iter()
						.map(|r| format!("`🟢 {}`", r.as_str()))
						.collect::<Vec<_>>()
						.join(" "),
					false,
				);
			}
			if !synced_roles.removed.is_empty() {
				embed = embed.field(
					"Removed Roles",
					synced_roles
						.removed
						.iter()
						.map(|r| format!("`🔴 {}`", r.as_str()))
						.collect::<Vec<_>>()
						.join(" "),
					false,
				);
			}

			embed
		}
		Err(QuerySyncRolesError::FailedQuery(err)) => {
			tracing::error!("failed to query: {err}");

			CreateEmbed::new()
				.title("Failed to query database")
				.colour(Colour::new(16725838))
		}
		Err(QuerySyncRolesError::UserNotFound) => CreateEmbed::new()
			.title("You do not have a connected 7TV account")
			.colour(Colour::new(16753720)),

		Err(QuerySyncRolesError::SyncRole(err)) => {
			tracing::error!("{err} to: {}", command.member.as_ref().unwrap().user.id);

			CreateEmbed::new()
				.title("Failed to apply roles")
				.colour(Colour::new(16725838))
		}
	};

	if let Err(err) = command
		.edit_response(ctx, EditInteractionResponse::new().embed(response))
		.await
	{
		tracing::error!("failed to respond to slash command {} due to {}", command.data.name, err);
	}
}

pub fn register() -> CreateCommand {
	CreateCommand::new("sync")
		.description("Sync your 7TV roles")
		.dm_permission(false)
}
