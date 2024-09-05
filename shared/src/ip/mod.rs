use std::io::Read;
use std::path::PathBuf;

use anyhow::Context;
use scuffle_foundations::settings::auto_settings;
use url::Url;

#[auto_settings]
pub struct GeoIpConfig {
	#[settings(default = "./GeoLite2-Country.mmdb".parse().unwrap())]
	pub path: PathBuf,
	/// If the file is not found, download it from the maxmind website
	pub download: Option<GeoIpConfigDownload>,
}

#[auto_settings]
pub struct GeoIpConfigDownload {
	pub account_id: String,
	pub api_token: String,
	#[settings(default = "https://download.maxmind.com/geoip/databases/GeoLite2-Country/download?suffix=tar.gz".parse().unwrap())]
	pub url: Url,
	/// Cache the file to the path specified in `path`
	#[settings(default = true)]
	pub cache_download: bool,
}

pub struct GeoIpResolver {
	reader: maxminddb::Reader<Vec<u8>>,
}

impl GeoIpResolver {
	pub async fn new(config: &GeoIpConfig) -> anyhow::Result<Self> {
		let data = match tokio::fs::read(&config.path).await {
			Ok(file) => file,
			Err(e) if config.download.is_some() => {
				tracing::warn!("failed to read geoip file, downloading: {e}");
				let download_config = config.download.as_ref().unwrap();

				let request = reqwest::RequestBuilder::from_parts(
					reqwest::Client::new(),
					reqwest::Request::new(reqwest::Method::GET, download_config.url.clone()),
				)
				.basic_auth(download_config.account_id.clone(), Some(download_config.api_token.clone()));

				let response = request.send().await.context("failed to download geoip file")?;
				response.error_for_status_ref().context("failed to download geoip file")?;
				let gzip_tar = response.bytes().await.context("failed to download geoip file")?;

				let content = tokio::task::spawn_blocking(move || {
					let mut archive = tar::Archive::new(flate2::read::GzDecoder::new(gzip_tar.as_ref()));
					let mut buffer = Vec::new();

					for entry in archive.entries().context("failed to extract geoip file")? {
						let mut entry = entry.context("failed to extract geoip file")?;
						let path = entry.path().context("failed to extract geoip file")?;
						if path.extension().map(|e| e == "mmdb").unwrap_or(false) {
							buffer.resize(entry.size() as usize, 0);
							entry.read_exact(&mut buffer).context("failed to extract geoip file")?;
							break;
						}
					}

					anyhow::Ok(buffer)
				})
				.await
				.context("failed to extract geoip file")??;

				if download_config.cache_download {
					tokio::fs::write(&config.path, &content)
						.await
						.context("failed to write geoip file")?;
				}

				content
			}
			Err(e) => return Err(e.into()),
		};

		let reader = maxminddb::Reader::from_source(data).context("failed to parse geoip file")?;

		Ok(Self { reader })
	}

	pub fn lookup(&self, ip: std::net::IpAddr) -> Option<maxminddb::geoip2::country::Country<'_>> {
		self.reader.lookup::<maxminddb::geoip2::Country>(ip).ok()?.country
	}
}
