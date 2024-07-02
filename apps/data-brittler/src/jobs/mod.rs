use std::ops::AddAssign;
use std::pin::Pin;
use std::sync::Arc;

use anyhow::Context;
use bson::doc;
use entitlements::EntitlementsJob;
use futures::stream::FuturesUnordered;
use futures::{Future, TryStreamExt};
use sailfish::TemplateOnce;
use tokio::time::Instant;
use tracing::Instrument;

use self::emotes::EmotesJob;
use self::users::UsersJob;
use crate::format::Number;
use crate::global::Global;
use crate::jobs::audit_logs::AuditLogsJob;
use crate::jobs::bans::BansJob;
use crate::jobs::cosmetics::CosmeticsJob;
use crate::jobs::emote_sets::EmoteSetsJob;
use crate::jobs::messages::MessagesJob;
use crate::jobs::prices::PricesJob;
use crate::jobs::reports::ReportsJob;
use crate::jobs::roles::RolesJob;
use crate::jobs::system::SystemJob;
use crate::{error, report};

pub mod audit_logs;
pub mod bans;
pub mod cosmetics;
pub mod emote_sets;
pub mod emotes;
pub mod entitlements;
pub mod messages;
pub mod prices;
pub mod reports;
pub mod roles;
pub mod system;
pub mod users;

pub struct JobOutcome {
	pub job_name: String,
	pub errors: Vec<error::Error>,
	pub took_seconds: f64,
	pub processed_documents: u64,
	pub inserted_rows: u64,
}

#[derive(Default)]
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

pub trait Job: Sized + Send + Sync {
	const NAME: &'static str;
	type T: serde::de::DeserializeOwned + Send + Sync;

	async fn new(global: Arc<Global>) -> anyhow::Result<Self>;

	#[tracing::instrument(name = "job", skip_all, fields(job = Self::NAME))]
	fn conditional_init_and_run(
		global: &Arc<Global>,
		should_run: bool,
	) -> anyhow::Result<Option<Pin<Box<impl Future<Output = anyhow::Result<JobOutcome>>>>>> {
		if should_run {
			let fut = Box::pin(
				async move {
					match Self::new(global.clone()).await {
						Ok(job) => job.run().await,
						Err(e) => Err(e),
					}
				}
				.in_current_span(),
			);
			Ok(Some(fut))
		} else {
			tracing::info!("skipping {} job", Self::NAME);
			Ok(None)
		}
	}

	async fn run(mut self) -> anyhow::Result<JobOutcome> {
		let timer = Instant::now();

		let collection = self.collection().await;

		// count
		let count = collection.count_documents(doc! {}).await?;
		let tenth = count / 10;
		tracing::info!("found {} documents", Number::from(count));

		// query
		let mut outcome = JobOutcome {
			errors: Vec::new(),
			job_name: Self::NAME.to_string(),
			took_seconds: 0.0,
			processed_documents: 0,
			inserted_rows: 0,
		};
		let mut documents = collection.find(doc! {}).await.context("failed to query documents")?;

		while let Some(r) = documents.try_next().await.transpose() {
			if scuffle_foundations::context::Context::global().is_done() {
				tracing::info!("job cancelled");
				break;
			}

			match r {
				Ok(t) => outcome += self.process(t).await,
				Err(e) => outcome.errors.push(error::Error::Deserialize(e)),
			}

			if tenth != 0 && outcome.processed_documents % tenth == 0 {
				tracing::info!(
					"{:.1}% ({}/{}) ({} errors)",
					outcome.processed_documents as f64 / count as f64 * 100.0,
					Number::from(outcome.processed_documents),
					Number::from(count),
					Number::from(outcome.errors.len())
				);
			}

			outcome.processed_documents += 1;
		}

		outcome += self.finish().await;

		let took_seconds = timer.elapsed().as_secs_f64();

		tracing::info!(
			"processed {} documents in {:.2}s (see report for details)",
			Number::from(outcome.processed_documents),
			took_seconds
		);

		outcome.took_seconds = took_seconds;

		Ok(outcome)
	}

	async fn collection(&self) -> mongodb::Collection<Self::T>;
	async fn process(&mut self, t: Self::T) -> ProcessOutcome;
	async fn finish(self) -> ProcessOutcome {
		ProcessOutcome::default()
	}
}

pub async fn run(global: Arc<Global>) -> anyhow::Result<()> {
	tracing::info!("starting jobs");

	let timer = Instant::now();

	let futures: FuturesUnordered<Pin<Box<dyn Future<Output = anyhow::Result<JobOutcome>> + Send>>> =
		FuturesUnordered::new();

	// ugly ass code
	if let Some(j) = UsersJob::conditional_init_and_run(&global, global.config().should_run_users())? {
		futures.push(j);
	}
	if let Some(j) = BansJob::conditional_init_and_run(&global, global.config().should_run_bans())? {
		futures.push(j);
	}
	if let Some(j) = EmotesJob::conditional_init_and_run(&global, global.config().should_run_emotes())? {
		futures.push(j);
	}
	if let Some(j) = EmoteSetsJob::conditional_init_and_run(&global, global.config().should_run_emote_sets())? {
		futures.push(j);
	}
	if let Some(j) = EntitlementsJob::conditional_init_and_run(&global, global.config().should_run_entitlements())? {
		futures.push(j);
	}
	if let Some(j) = CosmeticsJob::conditional_init_and_run(&global, global.config().should_run_cosmetics())? {
		futures.push(j);
	}
	if let Some(j) = RolesJob::conditional_init_and_run(&global, global.config().should_run_roles())? {
		futures.push(j);
	}
	if let Some(j) = ReportsJob::conditional_init_and_run(&global, global.config().should_run_reports())? {
		futures.push(j);
	}
	if let Some(j) = AuditLogsJob::conditional_init_and_run(&global, global.config().should_run_audit_logs())? {
		futures.push(j);
	}
	if let Some(j) = MessagesJob::conditional_init_and_run(&global, global.config().should_run_messages())? {
		futures.push(j);
	}
	if let Some(j) = SystemJob::conditional_init_and_run(&global, global.config().should_run_system())? {
		futures.push(j);
	}
	if let Some(j) = PricesJob::conditional_init_and_run(&global, global.config().should_run_prices())? {
		futures.push(j);
	}

	let results: Vec<JobOutcome> = futures.try_collect().await?;

	let took_seconds = timer.elapsed().as_secs_f64();

	let total_documents: u64 = results.iter().map(|o| o.processed_documents).sum();
	let total_rows: u64 = results.iter().map(|o| o.inserted_rows).sum();

	tracing::info!("writing report to {}", global.config().report_path.display());
	let report = report::ReportTemplate {
		outcomes: results,
		took_seconds,
		total_documents: total_documents.into(),
		total_rows: total_rows.into(),
		..Default::default()
	}
	.render_once()?;
	tokio::fs::write(&global.config().report_path, report).await?;

	Ok(())
}
