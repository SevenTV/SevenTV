use std::collections::{BTreeMap, HashMap, HashSet};
use std::ops::AddAssign;
use std::sync::Arc;

use anyhow::Context;
use sailfish::TemplateOnce;
use shared::database::badge::{Badge, BadgeId};
use shared::database::cron_job::{default_cron_jobs, CronJob};
use shared::database::emote::{Emote, EmoteId};
use shared::database::emote_moderation_request::EmoteModerationRequest;
use shared::database::emote_set::{EmoteSet, EmoteSetId};
use shared::database::entitlement::EntitlementEdge;
use shared::database::paint::{Paint, PaintId};
use shared::database::stored_event::StoredEvent;
use shared::database::user::ban::UserBan;
use special_events::SpecialEventsJob;
use tokio::time::Instant;

use crate::global::Global;
use crate::jobs::cdn_rename_list::CdnRenameJob;
use crate::jobs::cosmetics::CosmeticsJob;
use crate::jobs::prices::PricesJob;
use crate::jobs::reports::ReportsJob;
use crate::jobs::roles::RolesJob;
use crate::jobs::subscriptions::SubscriptionsJob;
use crate::jobs::system::SystemJob;
use crate::jobs::users::UsersJob;
use crate::{error, report};

pub mod audit_logs;
pub mod bans;
pub mod cdn_rename_list;
pub mod cosmetics;
pub mod emote_sets;
pub mod emotes;
pub mod entitlements;
pub mod messages;
pub mod prices;
pub mod reports;
pub mod roles;
pub mod special_events;
pub mod subscriptions;
pub mod system;
pub mod users;

#[must_use = "JobOutcomes must be used"]
pub struct JobOutcome {
	pub job_name: &'static str,
	pub errors: Vec<error::Error>,
	pub started_at: std::time::Instant,
	pub processed_documents: u64,
	pub inserted_rows: u64,
}

impl JobOutcome {
	pub fn new(job_name: &'static str) -> Self {
		Self {
			job_name,
			errors: Vec::new(),
			processed_documents: 0,
			started_at: std::time::Instant::now(),
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
	pub stored_events: Vec<StoredEvent>,
	pub bans: Vec<UserBan>,
	pub badges: HashMap<BadgeId, Badge>,
	pub paints: HashMap<PaintId, Paint>,
	pub cron_jobs: Vec<CronJob>,
	pub emote_sets: HashMap<EmoteSetId, EmoteSet>,
	pub emotes: HashMap<EmoteId, Emote>,
	pub entitlements: HashSet<EntitlementEdge>,
	pub emote_stats: BTreeMap<(EmoteId, time::Date), i32>,
	pub pending_tasks: HashMap<String, cosmetics::PendingTask>,
	pub mod_requests: Vec<EmoteModerationRequest>,
}

impl JobRunner {
	pub async fn fetch(&mut self, global: &Arc<Global>) -> anyhow::Result<HashMap<&'static str, JobOutcome>> {
		let mut outcomes = HashMap::new();

		let outcome = if global.config().should_run_audit_logs() {
			// Will fetch the audit logs from the source db and construct stats for emotes
			// if we run the job
			audit_logs::run(audit_logs::RunInput {
				global,
				stats: &mut self.emote_stats,
				events: &mut self.stored_events,
			})
			.await
			.context("audit logs")?
		} else {
			// Will skip fetching audit logs and only fetches stats from the target db
			audit_logs::skip(audit_logs::RunInput {
				global,
				stats: &mut self.emote_stats,
				events: &mut self.stored_events,
			})
			.await
			.context("audit logs skip")?
		};
		outcomes.insert(outcome.job_name, outcome);

		let outcome = if global.config().should_run_bans() {
			// Will fetch the bans from the source db if we run the job
			bans::run(bans::RunInput {
				global,
				bans: &mut self.bans,
			})
			.await
			.context("bans")?
		} else {
			// Will fetch the bans from the target db if we skip the job
			bans::skip(bans::RunInput {
				global,
				bans: &mut self.bans,
			})
			.await
			.context("bans skip")?
		};
		outcomes.insert(outcome.job_name, outcome);

		let outcome = if global.config().should_run_cosmetics() {
			// Will fetch the cosmetics from the source db if we run the job
			cosmetics::run(cosmetics::RunInput {
				global,
				badges: &mut self.badges,
				paints: &mut self.paints,
				pending_tasks: &mut self.pending_tasks,
			})
			.await
			.context("cosmetics")?
		} else {
			// Will fetch the cosmetics from the target db if we skip the job
			cosmetics::skip(cosmetics::RunInput {
				global,
				badges: &mut self.badges,
				paints: &mut self.paints,
				pending_tasks: &mut self.pending_tasks,
			})
			.await
			.context("cosmetics skip")?
		};
		outcomes.insert(outcome.job_name, outcome);

		// CronJobs are always constructed but inserts may be skipped if the job is not
		// run
		self.cron_jobs = default_cron_jobs();

		if global.config().should_run_entitlements() {
			// Will fetch the entitlements from the source db if we run the job
			let outcome = entitlements::run(entitlements::RunInput {
				global,
				edges: &mut self.entitlements,
				badge_filter: Box::new(|badge_id| self.badges.contains_key(&badge_id)),
				paint_filter: Box::new(|paint_id| self.paints.contains_key(&paint_id)),
			})
			.await
			.context("entitlements")?;

			outcomes.insert(outcome.job_name, outcome);
		}

		if global.config().should_run_messages() {
			// Will fetch the messages from the source db if we run the job
			let outcome = messages::run(messages::RunInput {
				global,
				mod_requests: &mut self.mod_requests,
			})
			.await
			.context("messages")?;

			outcomes.insert(outcome.job_name, outcome);
		}

		Ok(outcomes)
	}
}

pub async fn run(global: Arc<Global>) -> anyhow::Result<()> {
	tracing::info!("starting jobs");

	let timer = Instant::now();

	// macro_rules! job {
	// 	(
	// 		$([$name:ident, $fn:ident]),+$(,)?
	// 	) => {
	// 		{
	// 			let mut futures = Vec::new();

	// 			let wrap_future = |f: tokio::task::JoinHandle<anyhow::Result<JobOutcome>>|
	// async move { 				match f.await {
	// 					Ok(Ok(outcome)) => Ok(outcome),
	// 					Ok(Err(e)) => anyhow::bail!("job error: {:#}", e),
	// 					Err(e) => anyhow::bail!("job error: {:#}", e),
	// 				}
	// 			};

	// 			$(
	// 				if let Some(j) = $name::conditional_init_and_run(global.clone(),
	// global.config().$fn()) { 					futures.push(wrap_future(tokio::spawn(j)));
	// 				}
	// 			)+

	// 			futures
	// 		}
	// 	};
	// }

	// let futures = job! {
	// 	[UsersJob, should_run_users],
	// 	[BansJob, should_run_bans],
	// 	[EmotesJob, should_run_emotes],
	// 	[CdnRenameJob, should_run_cdn_rename],
	// 	[AuditLogsJob, should_run_audit_logs],
	// 	[EmoteSetsJob, should_run_emote_sets],
	// 	[EntitlementsJob, should_run_entitlements],
	// 	[CosmeticsJob, should_run_cosmetics],
	// 	[RolesJob, should_run_roles],
	// 	[ReportsJob, should_run_reports],
	// 	[MessagesJob, should_run_messages],
	// 	[SystemJob, should_run_system],
	// 	[PricesJob, should_run_prices],
	// 	[SubscriptionsJob, should_run_subscriptions],
	// 	[CronJobsJob, should_run_cron_jobs],
	// 	[SpecialEventsJob, should_run_special_events],
	// };

	// let results: Vec<JobOutcome> = futures::future::try_join_all(futures).await?;

	// let took_seconds = timer.elapsed().as_secs_f64();

	// let total_documents: u64 = results.iter().map(|o|
	// o.processed_documents).sum(); let total_rows: u64 = results.iter().map(|o|
	// o.inserted_rows).sum();

	// tracing::info!("writing report");
	// let report = report::ReportTemplate {
	// 	outcomes: results,
	// 	took_seconds,
	// 	total_documents: total_documents.into(),
	// 	total_rows: total_rows.into(),
	// 	..Default::default()
	// }
	// .render_once()?;
	// tokio::fs::write(&global.config().report_path, report).await?;
	// tracing::info!("report written to {}",
	// global.config().report_path.display());

	Ok(())
}
