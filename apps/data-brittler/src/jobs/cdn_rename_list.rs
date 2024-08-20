use std::sync::Arc;

use futures::StreamExt;
use mongodb::Cursor;
use shared::database::{badge::BadgeId, image_set};

use super::{Job, ProcessOutcome};
use crate::{error, global::Global, types};

pub struct CdnRenameJob {
	global: Arc<Global>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CdnFileRename {
	#[serde(rename = "o")]
	pub old_path: String,
	#[serde(rename = "n")]
	pub new_path: String,
}

impl Job for CdnRenameJob {
	type T = ();

	const NAME: &'static str = "cdn_rename";

	async fn new(global: Arc<Global>) -> anyhow::Result<Self> {
		Ok(Self { global })
	}

	async fn finish(self) -> ProcessOutcome {
		tracing::info!("creating cdn rename list");

		let mut outcome = ProcessOutcome::default();

		let mut public_cdn_files = vec![];
		let mut private_cdn_files = vec![];

		let mut emotes: Cursor<types::Emote> = match self
			.global
			.source_db()
			.collection("emotes")
			.find(mongodb::bson::doc! {})
			.await
		{
			Ok(emotes) => emotes,
			Err(e) => {
				return outcome.with_error(e);
			}
		};

		while let Some(emote) = emotes.next().await {
			match emote {
				Ok(emote) => {
					for v in emote.versions {
						if v.state.lifecycle == types::EmoteLifecycle::Live {
							let old_path = v.input_file.key.clone();
							match image_set::Image::try_from(v.input_file) {
								Ok(new) => private_cdn_files.push(CdnFileRename {
									old_path,
									new_path: new.path,
								}),
								Err(e) => {
									outcome = outcome.with_error(error::Error::InvalidCdnFile(e));
								}
							}

							for old in v.image_files {
								let old_path = old.key.clone();
								match image_set::Image::try_from(old) {
									Ok(new) => public_cdn_files.push(CdnFileRename {
										old_path,
										new_path: new.path,
									}),
									Err(e) => {
										outcome = outcome.with_error(error::Error::InvalidCdnFile(e));
									}
								}
							}
						}
					}
				}
				Err(e) => {
					outcome = outcome.with_error(e);
				}
			}
		}

		let mut users: Cursor<types::User> =
			match self.global.source_db().collection("users").find(mongodb::bson::doc! {}).await {
				Ok(emotes) => emotes,
				Err(e) => {
					return outcome.with_error(e);
				}
			};

		while let Some(user) = users.next().await {
			match user {
				Ok(user) => {
					if let Some(types::UserAvatar::Processed {
						input_file, image_files, ..
					}) = user.avatar
					{
						let old_path = input_file.key.clone();
						match image_set::Image::try_from(input_file) {
							Ok(new) => private_cdn_files.push(CdnFileRename {
								old_path,
								new_path: new.path,
							}),
							Err(e) => {
								outcome = outcome.with_error(error::Error::InvalidCdnFile(e));
							}
						}

						for old in image_files {
							let old_path = old.key.clone();
							match image_set::Image::try_from(old) {
								Ok(new) => public_cdn_files.push(CdnFileRename {
									old_path,
									new_path: new.path,
								}),
								Err(e) => {
									outcome = outcome.with_error(error::Error::InvalidCdnFile(e));
								}
							}
						}
					}
				}
				Err(e) => {
					outcome = outcome.with_error(e);
				}
			}
		}

		let mut badges: Cursor<types::Cosmetic> = match self
			.global
			.source_db()
			.collection("cosmetics")
			.find(mongodb::bson::doc! {"kind": "BADGE"})
			.await
		{
			Ok(emotes) => emotes,
			Err(e) => {
				return outcome.with_error(e);
			}
		};

		while let Some(badge) = badges.next().await {
			match badge {
				Ok(badge) => {
					let id = BadgeId::from(badge.id);

					for file in &["1x", "2x", "3x"] {
						public_cdn_files.push(CdnFileRename {
							old_path: format!("badge/{}/{}", badge.id, file),
							new_path: format!("badge/{}/{}.webp", id, file),
						});
					}
				}
				Err(e) => {
					outcome = outcome.with_error(e);
				}
			}
		}

		tracing::info!("writing cdn rename list");
		let file = match std::fs::File::create("local/public_cdn_rename.json") {
			Ok(file) => file,
			Err(e) => {
				return outcome.with_error(e);
			}
		};
		let writer = std::io::BufWriter::new(file);
		if let Err(e) = serde_json::to_writer(writer, &public_cdn_files) {
			return outcome.with_error(e);
		}

		let file = match std::fs::File::create("local/private_cdn_rename.json") {
			Ok(file) => file,
			Err(e) => {
				return outcome.with_error(e);
			}
		};
		let writer = std::io::BufWriter::new(file);
		if let Err(e) = serde_json::to_writer(writer, &private_cdn_files) {
			return outcome.with_error(e);
		}

		outcome
	}
}
