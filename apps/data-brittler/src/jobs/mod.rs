use std::ops::AddAssign;
use std::pin::Pin;
use std::sync::Arc;

use anyhow::Context;
use futures::stream::FuturesUnordered;
use futures::{Future, TryStreamExt};
use sailfish::TemplateOnce;
use tokio::time::Instant;
use tracing::Instrument;

use self::emotes::EmotesJob;
use self::users::UsersJob;
use crate::format::Number;
use crate::global::Global;
use crate::jobs::bans::BansJob;
use crate::jobs::cosmetics::CosmeticsJob;
use crate::jobs::emote_sets::EmoteSetsJob;
use crate::jobs::reports::ReportsJob;
use crate::jobs::roles::RolesJob;
use crate::{error, report};

pub mod bans;
pub mod cosmetics;
pub mod emote_sets;
pub mod emotes;
pub mod reports;
pub mod roles;
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

impl AddAssign<ProcessOutcome> for JobOutcome {
	fn add_assign(&mut self, mut rhs: ProcessOutcome) {
		self.errors.append(&mut rhs.errors);
		self.inserted_rows += rhs.inserted_rows;
	}
}

pub trait Job: Sized {
	const NAME: &'static str;
	type T: serde::de::DeserializeOwned;

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
						Ok(job) => job.run(global.clone()).await,
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

	async fn run(mut self, global: Arc<Global>) -> anyhow::Result<JobOutcome> {
		let timer = Instant::now();

		let collection = self.collection().await;

		// count
		let count = collection.count_documents(None, None).await?;
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
		let mut documents = collection.find(None, None).await.context("failed to query documents")?;

		while let Some(r) = documents.try_next().await.transpose() {
			if global.ctx().is_done() {
				tracing::info!("job cancelled");
				break;
			}

			match r {
				Ok(t) => outcome += self.process(t).await,
				Err(e) => outcome.errors.push(error::Error::Deserialize(e)),
			}

			if outcome.processed_documents % tenth == 0 {
				tracing::info!(
					"{:.1}% ({}/{})",
					outcome.processed_documents as f64 / count as f64 * 100.0,
					Number::from(outcome.processed_documents),
					Number::from(count)
				);
			}

			outcome.processed_documents += 1;
		}

		if let Err(e) = self.finish().await {
			tracing::error!("failed to finish job (susge?): {}", e);
		}

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
	async fn finish(self) -> anyhow::Result<()> {
		Ok(())
	}
}

pub async fn run(global: Arc<Global>) -> anyhow::Result<()> {
	let any_run = global.config().users
		|| global.config().bans
		|| global.config().emotes
		|| global.config().emote_sets
		|| global.config().cosmetics
		|| global.config().roles
		|| global.config().reports;

	let timer = Instant::now();

	let futures: FuturesUnordered<Pin<Box<dyn Future<Output = anyhow::Result<JobOutcome>> + Send>>> =
		FuturesUnordered::new();

	// ugly ass code
	UsersJob::conditional_init_and_run(
		&global,
		any_run && global.config().users || !any_run && !global.config().skip_users,
	)?
	.map(|j| futures.push(j));
	BansJob::conditional_init_and_run(
		&global,
		any_run && global.config().bans || !any_run && !global.config().skip_bans,
	)?
	.map(|j| futures.push(j));
	EmotesJob::conditional_init_and_run(
		&global,
		any_run && global.config().emotes || !any_run && !global.config().skip_emotes,
	)?
	.map(|j| futures.push(j));
	EmoteSetsJob::conditional_init_and_run(
		&global,
		any_run && global.config().skip_emote_sets || !any_run && !global.config().emote_sets,
	)?
	.map(|j| futures.push(j));
	CosmeticsJob::conditional_init_and_run(
		&global,
		any_run && global.config().cosmetics || !any_run && !global.config().skip_cosmetics,
	)?
	.map(|j| futures.push(j));
	RolesJob::conditional_init_and_run(
		&global,
		any_run && global.config().roles || !any_run && !global.config().skip_roles,
	)?
	.map(|j| futures.push(j));
	ReportsJob::conditional_init_and_run(
		&global,
		any_run && global.config().reports || !any_run && !global.config().skip_reports,
	)?
	.map(|j| futures.push(j));

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
