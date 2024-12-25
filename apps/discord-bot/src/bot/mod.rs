use std::sync::{
	atomic::{AtomicBool, Ordering},
	Arc,
};

use anyhow::Context as _;
use serenity::{
	all::{
		ActivityData, CacheHttp, ChannelId, Colour, Context, CreateEmbed, CreateInteractionResponse,
		CreateInteractionResponseMessage, CreateMessage, EventHandler, GatewayIntents, GuildId, Interaction, Member,
		OnlineStatus, Ready,
	},
	async_trait,
	futures::StreamExt,
	Client,
};
use shared::{
	database::{
		emote::{Emote, EmoteFlags},
		entitlement::EntitlementEdgeKind,
		stored_event::{ImageProcessorEvent, StoredEventEmoteData},
		user::{connection::Platform, User},
	},
	event::{InternalEventData, InternalEventPayload, InternalEventUserData},
};
use utils::{
	query_sync_roles::{query_sync_roles, QuerySyncRolesError},
	sync_roles::sync_roles,
};

use crate::global::Global;

mod commands;
mod utils;

struct Handler {
	global: Arc<Global>,
	is_listening_to_nats: AtomicBool,
}

#[async_trait]
impl EventHandler for Handler {
	async fn ready(&self, ctx: Context, _ready: Ready) {
		tracing::info!("discord bot online");

		ctx.set_presence(Some(ActivityData::custom("7TV")), OnlineStatus::Online);

		let guild_id = GuildId::new(self.global.config.bot.guild_id);
		if let Err(err) = guild_id
			.set_commands(&ctx.http, vec![commands::sync::register(), commands::admin::register()])
			.await
		{
			tracing::error!("failed to set guild commands: {err}");
		}

		if !self.is_listening_to_nats.load(Ordering::Relaxed) {
			tokio::spawn({
				let global = self.global.clone();
				let ctx = Arc::new(ctx);

				async move {
					listen_to_nats(global, ctx).await;
				}
			});

			self.is_listening_to_nats.swap(true, Ordering::Relaxed);
		}
	}

	async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
		if let Interaction::Command(command) = interaction {
			match command.data.name.as_str() {
				"sync" => commands::sync::run(&self.global, &ctx, &command).await,
				"admin" => commands::admin::run(&self.global, &ctx, &command).await,
				_ => {
					if let Err(err) = command
						.create_response(
							ctx,
							CreateInteractionResponse::Message(
								CreateInteractionResponseMessage::new().embed(
									CreateEmbed::new()
										.title("This command does not exist")
										.colour(Colour::new(16725838)),
								),
							),
						)
						.await
					{
						tracing::error!("failed to respond to slash command {} due to {}", command.data.name, err);
					}
				}
			};
		}
	}

	async fn guild_member_addition(&self, ctx: Context, member: Member) {
		if let Err(err) = query_sync_roles(&self.global, ctx.http(), &member).await {
			match err {
				QuerySyncRolesError::FailedQuery(err) => tracing::error!("failed to query: {err}"),
				QuerySyncRolesError::SyncRole(err) => tracing::error!("{err} to user: {}", member.user.id),
				_ => {}
			}
		}
	}
}

pub async fn run(global: Arc<Global>, _ctx: scuffle_context::Context) -> anyhow::Result<()> {
	let mut client = Client::builder(&global.config.bot.discord_token, GatewayIntents::GUILD_MEMBERS)
		.event_handler(Handler {
			global: global.clone(),
			is_listening_to_nats: AtomicBool::new(false),
		})
		.await
		.context("discord client creation")?;

	client.start().await?;

	Ok(())
}

async fn listen_to_nats(global: Arc<Global>, ctx: Arc<Context>) {
	let sub = global.nats.subscribe("api.v4.events").await;

	match sub {
		Ok(mut sub) => {
			while let Some(message) = sub.next().await {
				let payload: InternalEventPayload = match rmp_serde::from_slice(&message.payload) {
					Ok(payload) => payload,
					Err(err) => {
						tracing::warn!(err = ?err, "malformed message");
						break;
					}
				};

				for event in payload.events {
					match event.data {
						InternalEventData::Emote { after, data } => {
							if let StoredEventEmoteData::Process { event } = data {
								if matches!(event, ImageProcessorEvent::Success) {
									tokio::spawn({
										let global = global.clone();
										let ctx = ctx.clone();

										async move {
											handle_emote_process_success(&global, &ctx, after).await;
										}
									});
								}
							}
						}
						InternalEventData::User { after, data } => match data {
							InternalEventUserData::AddEntitlement { target } => {
								let discord_connection =
									after.connections.iter().find(|c| matches!(c.platform, Platform::Discord));

								if let Some(discord_connection) = discord_connection {
									if let EntitlementEdgeKind::Role { role_id: _ } = target {
										if let Ok(discord_id) = discord_connection.platform_id.parse::<u64>() {
											tokio::spawn({
												let global = global.clone();
												let ctx = ctx.clone();

												async move {
													let _ = handle_add_entitlement(&global, &ctx, discord_id, after);
												}
											});
										}
									}
								}
							}
							InternalEventUserData::RemoveEntitlement { target } => {
								let discord_connection =
									after.connections.iter().find(|c| matches!(c.platform, Platform::Discord));

								if let Some(discord_connection) = discord_connection {
									match target {
										EntitlementEdgeKind::Role { role_id: _ } => {
											if let Ok(discord_id) = discord_connection.platform_id.parse::<u64>() {
												tokio::spawn({
													let global = global.clone();
													let ctx = ctx.clone();

													async move {
														let _ = handle_remove_entitlement(&global, &ctx, discord_id, after);
													}
												});
											}
										}
										_ => {}
									}
								}
							}
							InternalEventUserData::AddConnection { connection } => {
								if matches!(connection.platform, Platform::Discord) {
									if let Ok(discord_id) = connection.platform_id.parse::<u64>() {
										tokio::spawn({
											let global = global.clone();
											let ctx = ctx.clone();

											async move {
												let _ = handle_add_connection(&global, &ctx, discord_id, after);
											}
										});
									}
								}
							}
							InternalEventUserData::RemoveConnection { connection } => {
								if matches!(connection.platform, Platform::Discord) {
									if let Ok(discord_id) = connection.platform_id.parse::<u64>() {
										tokio::spawn({
											let global = global.clone();
											let ctx = ctx.clone();

											async move {
												let _ = handle_remove_connection(&global, &ctx, discord_id);
											}
										});
									}
								}
							}
							_ => {}
						},
						_ => {}
					}
				}
			}

			tracing::warn!("nats subscription closed");
		}
		Err(_) => {
			tracing::error!("failed to subscribe to nats subject")
		}
	}
}

async fn handle_emote_process_success(global: &Global, ctx: &Context, emote: Emote) {
	let channels = GuildId::new(global.config.bot.guild_id).channels(ctx.http()).await;

	match channels {
		Ok(channels) => {
			let activity_feed = channels.get(&ChannelId::new(global.config.bot.activity_feed_channel_id));

			match activity_feed {
				Some(activity_feed) => {
					let image_url = if emote.flags.contains(EmoteFlags::Animated) {
						format!("https://cdn.7tv.app/emote/{}/4x.gif", emote.id)
					} else {
						format!("https://cdn.7tv.app/emote/{}/4x.png", emote.id)
					};

					let embed = CreateEmbed::new()
						.title(emote.default_name)
						.colour(Colour::new(3709695))
						.url(format!("https://7tv.app/emotes/{}", emote.id))
						.image(image_url);

					let _ = activity_feed.send_message(ctx, CreateMessage::new().embed(embed)).await;
				}
				None => tracing::error!("activity feed channel does not exist"),
			}
		}
		Err(err) => {
			tracing::error!("failed to get guild channels: {err}")
		}
	}
}

async fn handle_add_entitlement(global: &Global, ctx: &Context, discord_id: u64, user: User) {
	let member = GuildId::new(global.config.bot.guild_id).member(&ctx, discord_id).await;

	if let Ok(member) = member {
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

		let _ = sync_roles(&role_ids, &member, ctx.http(), &global).await;
	}
}

async fn handle_remove_entitlement(global: &Global, ctx: &Context, discord_id: u64, user: User) {
	let member = GuildId::new(global.config.bot.guild_id).member(&ctx, discord_id).await;

	if let Ok(member) = member {
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

		let _ = sync_roles(&role_ids, &member, ctx.http(), &global).await;
	}
}

async fn handle_add_connection(global: &Global, ctx: &Context, discord_id: u64, user: User) {
	let member = GuildId::new(global.config.bot.guild_id).member(&ctx, discord_id).await;

	if let Ok(member) = member {
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

		let _ = sync_roles(&role_ids, &member, ctx.http(), &global).await;
	}
}

async fn handle_remove_connection(global: &Global, ctx: &Context, discord_id: u64) {
	let member = GuildId::new(global.config.bot.guild_id).member(&ctx, discord_id).await;

	if let Ok(member) = member {
		let _ = sync_roles(&[], &member, ctx.http(), &global).await;
	}
}