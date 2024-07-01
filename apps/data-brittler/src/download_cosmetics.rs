/// In case you run into rate limiting problems with imgur, you can run this first to request and store all image files before issuing the processing jobs.

pub async fn run(global: Arc<Global>) -> anyhow::Result<()> {
	let cosmetics = Cosmetics::collection(global.source_db())
		.find(doc! {})
		.await
		.context("failed to query cosmetics")?;

    while let Some(c) = cosmetics.try_next().await.context("failed to query cosmetics")? {
        if scuffle_foundations::context::Context::global().is_done() {
            tracing::info!("job cancelled");
            break;
        }

        process_cosmetic(c).await?;
    }

	Ok(())
}

async fn process_cosmetic(c: Cosmetic) -> anyhow::Result<()> {

}
