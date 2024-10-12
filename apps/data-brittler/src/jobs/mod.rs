use std::collections::{BTreeMap, HashMap, HashSet};
use std::ops::AddAssign;
use std::sync::Arc;

use anyhow::Context;
use futures::future::BoxFuture;
use sailfish::TemplateOnce;
use scuffle_image_processor_proto::{event_callback, EventCallback};
use shared::database::badge::{Badge, BadgeId};
use shared::database::cron_job::{default_cron_jobs, CronJob};
use shared::database::emote::{Emote, EmoteId};
use shared::database::emote_moderation_request::EmoteModerationRequest;
use shared::database::emote_set::{EmoteSet, EmoteSetId};
use shared::database::entitlement::EntitlementEdge;
use shared::database::global::GlobalConfig;
use shared::database::image_set::{Image, ImageSet, ImageSetInput};
use shared::database::paint::{Paint, PaintId, PaintLayerType};
use shared::database::product::codes::RedeemCode;
use shared::database::product::special_event::SpecialEvent;
use shared::database::product::subscription::SubscriptionPeriod;
use shared::database::product::SubscriptionProduct;
use shared::database::role::{Role, RoleId};
use shared::database::stored_event::StoredEvent;
use shared::database::ticket::{Ticket, TicketId, TicketMessage};
use shared::database::user::ban::UserBan;
use shared::database::user::editor::UserEditor;
use shared::database::user::profile_picture::UserProfilePicture;
use shared::database::user::{User, UserId};
use shared::database::MongoCollection;
use tokio::sync::mpsc;
use tokio::time::Instant;

use crate::global::Global;
use crate::{error, report};

pub mod audit_logs;
pub mod bans;
pub mod cosmetics;
pub mod emote_sets;
pub mod emote_stats;
pub mod emotes;
pub mod entitlements;
pub mod messages;
pub mod prices;
pub mod redeem_codes;
pub mod reports;
pub mod roles;
pub mod subscriptions;
pub mod system;
pub mod users;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CdnFileRename {
	#[serde(rename = "o")]
	pub old_path: String,
	#[serde(rename = "n")]
	pub new_path: String,
}

#[must_use = "JobOutcomes must be used"]
pub struct JobOutcome {
	pub job_name: &'static str,
	pub errors: Vec<error::Error>,
	pub fetch_time: f64,
	pub insert_time: f64,
	pub processed_documents: u64,
	pub inserted_rows: u64,
}

impl JobOutcome {
	pub fn new(job_name: &'static str) -> Self {
		Self {
			job_name,
			errors: Vec::new(),
			processed_documents: 0,
			fetch_time: 0.0,
			insert_time: 0.0,
			inserted_rows: 0,
		}
	}
}

#[derive(Default)]
#[must_use = "ProcessOutcomes must be used"]
pub struct ProcessOutcome {
	pub errors: Vec<error::Error>,
	pub inserted_rows: u64,
}

impl ProcessOutcome {
	pub fn error(e: impl Into<error::Error>) -> Self {
		Self {
			errors: vec![e.into()],
			..Default::default()
		}
	}

	pub fn with_error(mut self, e: impl Into<error::Error>) -> Self {
		self.errors.push(e.into());
		self
	}
}

impl AddAssign<ProcessOutcome> for JobOutcome {
	fn add_assign(&mut self, mut rhs: ProcessOutcome) {
		self.errors.append(&mut rhs.errors);
		self.inserted_rows += rhs.inserted_rows;
	}
}

#[derive(Default)]
pub struct JobRunner {
	pub badges: HashMap<BadgeId, Badge>,
	pub paints: HashMap<PaintId, Paint>,
	pub roles: HashMap<RoleId, Role>,
	pub pending_tasks: Vec<(cosmetics::PendingTask, mpsc::Receiver<EventCallback>)>,

	pub emotes: HashMap<EmoteId, Emote>,
	pub users: HashMap<UserId, User>,
	pub editors: HashMap<(UserId, UserId), UserEditor>,
	pub profile_pictures: Vec<UserProfilePicture>,
	pub emote_sets: HashMap<EmoteSetId, EmoteSet>,
	pub true_emote_usage: HashMap<EmoteId, i32>,

	// Remove events that involve objects that are not apart of the above data
	pub stored_events: Vec<StoredEvent>,
	pub emote_stats: BTreeMap<(EmoteId, time::Date), i32>,

	pub public_cdn_rename: Vec<CdnFileRename>,
	pub internal_cdn_rename: Vec<CdnFileRename>,

	pub subscription_periods: Vec<SubscriptionPeriod>,

	pub bans: Vec<UserBan>,
	pub entitlements: HashSet<EntitlementEdge>,
	pub mod_requests: Vec<EmoteModerationRequest>,
	pub tickets: HashMap<TicketId, Ticket>,
	pub ticket_messages: Vec<TicketMessage>,

	pub redeem_codes: HashMap<String, RedeemCode>,

	pub cron_jobs: Vec<CronJob>,
	pub special_events: Vec<SpecialEvent>,
	pub subscription_products: Vec<SubscriptionProduct>,
	pub system: GlobalConfig,
}

pub struct JobOutcomes {
	pub cosmetics: JobOutcome,
	pub bans: JobOutcome,
	pub emotes: JobOutcome,
	pub users: JobOutcome,
	pub emote_sets: JobOutcome,
	pub entitlements: JobOutcome,
	pub messages: JobOutcome,
	pub audit_logs: JobOutcome,
	pub subscriptions: JobOutcome,
	pub reports: JobOutcome,
	pub redeem_codes: JobOutcome,
	pub system: JobOutcome,
}

impl JobRunner {
	pub async fn fetch(&mut self, global: &Arc<Global>) -> anyhow::Result<JobOutcomes> {
		let start = Instant::now();
		let mut cosmetics = cosmetics::run(cosmetics::RunInput {
			global,
			badges: &mut self.badges,
			paints: &mut self.paints,
			pending_tasks: &mut self.pending_tasks,
		})
		.await
		.context("cosmetics")?;
		cosmetics.fetch_time = start.elapsed().as_secs_f64();
		tracing::info!("cosmetics took {:.2}s", start.elapsed().as_secs_f64());

		self.roles.extend(roles::roles().into_iter().map(|r| (r.id, r)));

		let start = Instant::now();
		let mut bans = bans::run(bans::RunInput {
			global,
			bans: &mut self.bans,
		})
		.await
		.context("bans")?;
		bans.fetch_time = start.elapsed().as_secs_f64();
		tracing::info!("bans took {:.2}s", start.elapsed().as_secs_f64());

		let start = Instant::now();
		let mut emotes = emotes::run(emotes::RunInput {
			global,
			emotes: &mut self.emotes,
			internal_cdn_rename: &mut self.internal_cdn_rename,
			public_cdn_rename: &mut self.public_cdn_rename,
		})
		.await
		.context("emotes")?;
		emotes.fetch_time = start.elapsed().as_secs_f64();
		self.true_emote_usage = HashMap::from_iter(self.emotes.keys().map(|k| (*k, 0)));
		tracing::info!("emotes took {:.2}s", start.elapsed().as_secs_f64());

		let start = Instant::now();
		let mut users = users::run(users::RunInput {
			global,
			entitlements: &mut self.entitlements,
			users: &mut self.users,
			editors: &mut self.editors,
			profile_pictures: &mut self.profile_pictures,
			internal_cdn_rename: &mut self.internal_cdn_rename,
			public_cdn_rename: &mut self.public_cdn_rename,
		})
		.await
		.context("users")?;
		users.fetch_time = start.elapsed().as_secs_f64();
		tracing::info!("users took {:.2}s", start.elapsed().as_secs_f64());

		let start = Instant::now();
		let mut emote_sets = emote_sets::run(emote_sets::RunInput {
			global,
			emote_sets: &mut self.emote_sets,
			rankings: &mut self.true_emote_usage,
			users: &mut self.users,
			emotes: &self.emotes,
		})
		.await
		.context("emote sets")?;
		emote_sets.fetch_time = start.elapsed().as_secs_f64();
		tracing::info!("emote sets took {:.2}s", start.elapsed().as_secs_f64());

		let start = Instant::now();
		let mut entitlements = entitlements::run(entitlements::RunInput {
			global,
			edges: &mut self.entitlements,
			users: &mut self.users,
			roles: &self.roles,
			badges: &self.badges,
			paints: &self.paints,
		})
		.await
		.context("entitlements")?;
		entitlements.fetch_time = start.elapsed().as_secs_f64();
		tracing::info!("entitlements took {:.2}s", start.elapsed().as_secs_f64());

		let start = Instant::now();
		let mut messages = messages::run(messages::RunInput {
			global,
			emotes: &self.emotes,
			mod_requests: &mut self.mod_requests,
		})
		.await
		.context("messages")?;
		messages.fetch_time = start.elapsed().as_secs_f64();
		tracing::info!("messages took {:.2}s", start.elapsed().as_secs_f64());

		let start = Instant::now();
		let mut reports = reports::run(reports::RunInput {
			global,
			ticket_messages: &mut self.ticket_messages,
			tickets: &mut self.tickets,
		})
		.await
		.context("reports")?;
		reports.fetch_time = start.elapsed().as_secs_f64();
		tracing::info!("reports took {:.2}s", start.elapsed().as_secs_f64());

		let start = Instant::now();
		let mut audit_logs = audit_logs::run(audit_logs::RunInput {
			global,
			stats: &mut self.emote_stats,
			events: &mut self.stored_events,
			emotes: &self.emotes,
			emote_sets: &self.emote_sets,
			users: &self.users,
			roles: &self.roles,
			tickets: &self.tickets,
		})
		.await
		.context("audit logs")?;
		audit_logs.fetch_time = start.elapsed().as_secs_f64();
		tracing::info!("audit logs took {:.2}s", start.elapsed().as_secs_f64());

		let start = Instant::now();
		let mut subscriptions = subscriptions::run(subscriptions::RunInput {
			global,
			periods: &mut self.subscription_periods,
		})
		.await
		.context("subscriptions")?;
		subscriptions.fetch_time = start.elapsed().as_secs_f64();
		tracing::info!("subscriptions took {:.2}s", start.elapsed().as_secs_f64());

		self.cron_jobs = default_cron_jobs();
		self.special_events = subscriptions::benefits::special_events()
			.into_iter()
			.map(|e| e.special_event)
			.collect();

		let start = Instant::now();
		let mut redeem_codes = redeem_codes::run(redeem_codes::RunInput {
			global,
			redeem_codes: &mut self.redeem_codes,
		})
		.await
		.context("redeem codes")?;
		redeem_codes.fetch_time = start.elapsed().as_secs_f64();
		tracing::info!("redeem codes took {:.2}s", start.elapsed().as_secs_f64());

		self.subscription_products = prices::default_products();

		let start = Instant::now();
		let system = system::run(system::RunInput {
			global,
			global_config: &mut self.system,
			emote_sets: &mut self.emote_sets,
		})
		.await
		.context("system")?;
		tracing::info!("system took {:.2}s", start.elapsed().as_secs_f64());

		Ok(JobOutcomes {
			cosmetics,
			bans,
			emotes,
			users,
			emote_sets,
			entitlements,
			messages,
			audit_logs,
			subscriptions,
			reports,
			redeem_codes,
			system,
		})
	}
}

#[tracing::instrument(skip_all, name = "batch_insert", fields(job_name = %outcome.job_name, collection = %M::COLLECTION_NAME))]
async fn batch_insert<M: MongoCollection + serde::Serialize>(
	db: mongodb::Database,
	truncate: bool,
	mut outcome: JobOutcome,
	data: impl IntoIterator<Item = M>,
) -> JobOutcome {
	let start = Instant::now();

	if truncate {
		if let Err(err) = M::collection(&db).drop().await {
			tracing::error!("failed to drop collection: {:#}", err);
			outcome.errors.push(err.into());
			return outcome;
		}

		let indexes = M::indexes();
		if !indexes.is_empty() {
			if let Err(err) = M::collection(&db).create_indexes(indexes).await {
				tracing::error!("failed to create indexes: {:#}", err);
				outcome.errors.push(err.into());
				return outcome;
			}
		}
	}

	let result = M::collection(&db)
		.insert_many(data)
		.with_options(mongodb::options::InsertManyOptions::builder().ordered(false).build())
		.await;

	match result {
		Ok(result) => {
			outcome.inserted_rows += result.inserted_ids.len() as u64;
		}
		Err(e) => {
			tracing::error!("failed to insert documents: {:#}", e);
			outcome.errors.push(e.into());
		}
	}

	outcome.insert_time += start.elapsed().as_secs_f64();

	tracing::info!("{}({}) took {:.2}s", outcome.job_name, M::COLLECTION_NAME, start.elapsed().as_secs_f64());

	outcome
}

pub async fn run(global: Arc<Global>) -> anyhow::Result<()> {
	tracing::info!("starting jobs");

	let timer = Instant::now();
	let mut runner = JobRunner::default();
	let outcomes = runner.fetch(&global).await?;
	tracing::info!("fetched all data in {:.2}s", timer.elapsed().as_secs_f64());

	let mut futures = Vec::<BoxFuture<JobOutcome>>::new();

	macro_rules! insert_future {
		($should_run:expr, $fut:expr) => {
			if $should_run {
				futures.push(Box::pin($fut));
			}
		};
	}

	insert_future!(global.config.should_run_cosmetics(), async {
		let total_pending = runner.pending_tasks.len();
		while let Some((task, rx)) = runner.pending_tasks.first_mut() {
			let callback = rx.recv().await.expect("callback closed");
			match callback.event {
				Some(event_callback::Event::Start(_)) => {
					continue;
				}
				Some(event_callback::Event::Cancel(_)) => {
					tracing::info!("task canceled {:?}", task);
					match task {
						cosmetics::PendingTask::Badge(badge_id) => {
							runner.badges.remove(badge_id);
						}
						cosmetics::PendingTask::Paint(paint_id, _) => {
							runner.paints.remove(paint_id);
						}
					}
					runner.pending_tasks.remove(0);
				}
				Some(event_callback::Event::Fail(err)) => {
					tracing::error!("task failed {:?}: {:?}", task, err);
					match task {
						cosmetics::PendingTask::Badge(badge_id) => {
							runner.badges.remove(badge_id);
						}
						cosmetics::PendingTask::Paint(paint_id, _) => {
							runner.paints.remove(paint_id);
						}
					}
					runner.pending_tasks.remove(0);
				}
				Some(event_callback::Event::Success(success)) => {
					let outputs: Vec<_> = success
						.files
						.into_iter()
						.map(|i| Image {
							path: i.path.map(|p| p.path).unwrap_or_default(),
							mime: i.content_type,
							size: i.size as i64,
							width: i.width as i32,
							height: i.height as i32,
							frame_count: i.frame_count as i32,
							scale: i.scale.unwrap_or(1) as i32,
						})
						.collect();
					let input = success.input_metadata.unwrap();

					match task {
						cosmetics::PendingTask::Badge(badge_id) => {
							runner.badges.get_mut(badge_id).unwrap().image_set = ImageSet {
								input: ImageSetInput::Image(Image {
									frame_count: input.frame_count as i32,
									width: input.width as i32,
									height: input.height as i32,
									path: input.path.map(|p| p.path).unwrap_or_default(),
									mime: input.content_type,
									size: input.size as i64,
									scale: 1,
								}),
								outputs,
							}
						}
						cosmetics::PendingTask::Paint(paint_id, layer_id) => {
							if let Some(paint) = runner.paints.get_mut(paint_id) {
								paint.data.layers.iter_mut().find(|l| l.id == *layer_id).unwrap().ty =
									PaintLayerType::Image(ImageSet {
										input: ImageSetInput::Image(Image {
											frame_count: input.frame_count as i32,
											width: input.width as i32,
											height: input.height as i32,
											path: input.path.map(|p| p.path).unwrap_or_default(),
											mime: input.content_type,
											size: input.size as i64,
											scale: 1,
										}),
										outputs,
									});
							}
						}
					}
					runner.pending_tasks.remove(0);
				}
				None => continue,
			}
			tracing::info!(
				"processed {}/{} pending tasks",
				total_pending - runner.pending_tasks.len(),
				total_pending
			);
		}
		let outcome = tokio::spawn(batch_insert(
			global.target_db.clone(),
			global.config.truncate,
			outcomes.cosmetics,
			runner.badges.into_values(),
		))
		.await
		.unwrap();
		tokio::spawn(batch_insert(
			global.target_db.clone(),
			global.config.truncate,
			outcome,
			runner.paints.into_values(),
		))
		.await
		.unwrap()
	});
	insert_future!(global.config.should_run_bans(), async {
		tokio::spawn(batch_insert(
			global.target_db.clone(),
			global.config.truncate,
			outcomes.bans,
			runner.bans.into_iter(),
		))
		.await
		.unwrap()
	});
	insert_future!(global.config.should_run_emotes(), async {
		tokio::spawn(batch_insert(
			global.target_db.clone(),
			global.config.truncate,
			outcomes.emotes,
			runner.emotes.into_values(),
		))
		.await
		.unwrap()
	});
	insert_future!(global.config.should_run_users(), async {
		let outcome = tokio::spawn(batch_insert(
			global.target_db.clone(),
			global.config.truncate,
			outcomes.users,
			runner.users.into_values(),
		))
		.await
		.unwrap();

		tokio::spawn(batch_insert(
			global.target_db.clone(),
			global.config.truncate,
			outcome,
			runner.profile_pictures.into_iter(),
		))
		.await
		.unwrap()
	});
	insert_future!(global.config.should_run_emote_sets(), async {
		tokio::spawn(batch_insert(
			global.target_db.clone(),
			global.config.truncate,
			outcomes.emote_sets,
			runner.emote_sets.into_values(),
		))
		.await
		.unwrap()
	});
	insert_future!(global.config.should_run_entitlements(), async {
		tokio::spawn(batch_insert(
			global.target_db.clone(),
			global.config.truncate,
			outcomes.entitlements,
			runner.entitlements.into_iter(),
		))
		.await
		.unwrap()
	});
	insert_future!(global.config.should_run_messages(), async {
		tokio::spawn(batch_insert(
			global.target_db.clone(),
			global.config.truncate,
			outcomes.messages,
			runner.mod_requests.into_iter(),
		))
		.await
		.unwrap()
	});
	insert_future!(global.config.should_run_audit_logs(), async {
		tokio::spawn(batch_insert(
			global.target_db.clone(),
			global.config.truncate,
			outcomes.audit_logs,
			runner.stored_events.into_iter(),
		))
		.await
		.unwrap()
	});
	insert_future!(global.config.should_run_subscriptions(), async {
		tokio::spawn(batch_insert(
			global.target_db.clone(),
			global.config.truncate,
			outcomes.subscriptions,
			runner.subscription_periods.into_iter(),
		))
		.await
		.unwrap()
	});
	insert_future!(global.config.should_run_reports(), async {
		tokio::spawn(batch_insert(
			global.target_db.clone(),
			global.config.truncate,
			outcomes.reports,
			runner.ticket_messages.into_iter(),
		))
		.await
		.unwrap()
	});
	insert_future!(global.config.should_run_redeem_codes(), async {
		tokio::spawn(batch_insert(
			global.target_db.clone(),
			global.config.truncate,
			outcomes.redeem_codes,
			runner.redeem_codes.into_values(),
		))
		.await
		.unwrap()
	});
	insert_future!(global.config.should_run_prices(), async {
		tokio::spawn(batch_insert(
			global.target_db.clone(),
			global.config.truncate,
			JobOutcome::new("products"),
			runner.subscription_products.into_iter(),
		))
		.await
		.unwrap()
	});
	insert_future!(global.config.should_run_cron_jobs(), async {
		tokio::spawn(batch_insert(
			global.target_db.clone(),
			global.config.truncate,
			JobOutcome::new("cron_jobs"),
			runner.cron_jobs.into_iter(),
		))
		.await
		.unwrap()
	});
	insert_future!(global.config.should_run_special_events(), async {
		tokio::spawn(batch_insert(
			global.target_db.clone(),
			global.config.truncate,
			JobOutcome::new("special_events"),
			runner.special_events.into_iter(),
		))
		.await
		.unwrap()
	});
	insert_future!(global.config.should_run_roles(), async {
		tokio::spawn(batch_insert(
			global.target_db.clone(),
			global.config.truncate,
			JobOutcome::new("roles"),
			runner.roles.into_values(),
		))
		.await
		.unwrap()
	});
	insert_future!(global.config.should_run_system(), async {
		tokio::spawn(batch_insert(
			global.target_db.clone(),
			global.config.truncate,
			outcomes.system,
			std::iter::once(runner.system),
		))
		.await
		.unwrap()
	});
	insert_future!(global.config.should_run_cdn_rename(), async {
		let mut outcome = JobOutcome::new("cdn_rename");
		let start = Instant::now();

		fn write_file(file: &str, content: &[CdnFileRename]) -> anyhow::Result<()> {
			let content = serde_json::to_vec(content).context("serialize")?;
			std::fs::write(file, content).context("write")
		}

		let public_cdn_rename = runner.public_cdn_rename;
		if let Err(err) =
			tokio::task::spawn_blocking(move || write_file("./local/public_cdn_rename.json", &public_cdn_rename))
				.await
				.unwrap()
		{
			outcome.errors.push(error::Error::CdnRename(err));
		}

		let internal_cdn_rename = runner.internal_cdn_rename;
		if let Err(err) =
			tokio::task::spawn_blocking(move || write_file("./local/internal_cdn_rename.json", &internal_cdn_rename))
				.await
				.unwrap()
		{
			outcome.errors.push(error::Error::CdnRename(err));
		}

		outcome.insert_time = start.elapsed().as_secs_f64();

		tracing::info!("cdn_rename took {:.2}s", start.elapsed().as_secs_f64());

		outcome
	});
	insert_future!(global.config.should_run_emote_stats(), async {
		let mut outcome = JobOutcome::new("emote_stats");
		let start = Instant::now();

		if let Err(err) = tokio::spawn(emote_stats::run(emote_stats::RunInput {
			clickhouse: global.clickhouse.clone(),
			truncate: global.config.truncate,
			emote_stats: runner.emote_stats,
			true_emote_stats: runner.true_emote_usage,
		}))
		.await
		.unwrap()
		{
			outcome.errors.push(error::Error::EmoteStats(err));
		}

		outcome.insert_time = start.elapsed().as_secs_f64();

		tracing::info!("emote_stats took {:.2}s", start.elapsed().as_secs_f64());

		outcome
	});

	let outcomes = futures::future::join_all(futures).await;

	let total_documents = outcomes.iter().map(|o| o.inserted_rows).sum::<u64>();
	let total_rows = outcomes.iter().map(|o| o.processed_documents).sum::<u64>();

	tracing::info!("writing report");
	let report = report::ReportTemplate {
		outcomes,
		took_seconds: timer.elapsed().as_secs_f64(),
		total_documents: total_documents.into(),
		total_rows: total_rows.into(),
		created_at: chrono::Utc::now(),
	}
	.render_once()?;
	tokio::fs::write(&global.config.report_path, report).await?;
	tracing::info!("report written to {}", global.config.report_path.display());

	Ok(())
}
