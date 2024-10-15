use std::sync::Arc;

use anyhow::Context;
use futures::TryStreamExt;
use mongodb::bson::doc;

use crate::global::Global;
use crate::types::{self, Cosmetic, PaintData};

/// In case you run into rate limiting problems with imgur, you can run this
/// first to request and store all image files before issuing the processing
/// jobs.
pub async fn run(global: Arc<Global>) -> anyhow::Result<()> {
	let mut cosmetics = global
		.main_source_db
		.collection::<Cosmetic>("cosmetics")
		.find(doc! {})
		.await
		.context("failed to query cosmetics")?;

	while let Some(c) = cosmetics.try_next().await.context("failed to query cosmetics")? {
		if scuffle_foundations::context::Context::global().is_done() {
			tracing::info!("job cancelled");
			break;
		}

		let id = c.id;
		if let Err(e) = process_cosmetic(&global, c).await {
			tracing::error!(id = %id, error = %e, "failed to process cosmetic");
		}
	}

	Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum RequestImageError {
	#[error("{0}")]
	Reqwest(#[from] reqwest::Error),
	#[error("failed to download image: {0}")]
	Status(reqwest::StatusCode),
}

pub async fn request_image(global: &Arc<Global>, url: &str) -> Result<bytes::Bytes, RequestImageError> {
	tracing::debug!(url = %url, "requesting image");
	match global.http_client.get(url).send().await {
		Ok(res) if res.status().is_success() => match res.bytes().await {
			Ok(bytes) => Ok(bytes),
			Err(e) => Err(e.into()),
		},
		Ok(res) => Err(RequestImageError::Status(res.status())),
		Err(e) => Err(e.into()),
	}
}

async fn process_cosmetic(global: &Arc<Global>, c: Cosmetic) -> anyhow::Result<()> {
	tokio::fs::create_dir_all("local/cosmetics").await?;

	let download_url = match c.data {
		types::CosmeticData::Badge { .. } => format!("https://cdn.7tv.app/badge/{}/2x", c.id),
		types::CosmeticData::Paint {
			data: PaintData::Url {
				image_url: Some(image_url),
			},
			..
		} => image_url,
		_ => return Ok(()),
	};

	tracing::debug!(cosmetic_id = %c.id, "processing cosmetic");

	let image = request_image(global, &download_url).await?;
	let path = format!("local/cosmetics/{}", c.id);

	tracing::info!(cosmetic_id = %c.id, "writing image to disk {}KB", image.len() / 1024);

	tokio::fs::write(&path, &image).await?;

	Ok(())
}
