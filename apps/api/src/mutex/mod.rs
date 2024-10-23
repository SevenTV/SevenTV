use anyhow::Context;
use shared::database::Id;

pub struct DistributedMutex {
	redis: fred::clients::RedisPool,
	mutex_lock: fred::types::Function,
	mutex_free: fred::types::Function,
}

const LUA_SCRIPT: &str = include_str!("mutex.lua");

#[derive(thiserror::Error, Debug)]
pub enum MutexError {
	#[error("failed to acquire mutex after {0} attempts")]
	Acquire(usize),
	#[error("lost mutex lock while waiting for operation to complete")]
	Lost,
	#[error("redis error: {0}")]
	Redis(#[from] fred::error::RedisError),
}

#[derive(Debug, Default)]
pub struct MutexAquireRequest<T: std::fmt::Display> {
	pub key: T,
	pub attempts: usize,
	pub delay: std::time::Duration,
}

impl<T: std::fmt::Display> From<T> for MutexAquireRequest<T> {
	fn from(key: T) -> Self {
		Self {
			key,
			delay: std::time::Duration::from_millis(30),
			attempts: 350,
		}
	}
}

impl DistributedMutex {
	pub async fn new(redis: fred::clients::RedisPool) -> anyhow::Result<Self> {
		let lib = fred::types::Library::from_code(redis.next(), LUA_SCRIPT).await?;

		Ok(Self {
			mutex_lock: lib
				.functions()
				.get("api_mutex_lock")
				.context("failed to get api_ratelimit function")?
				.clone(),
			mutex_free: lib
				.functions()
				.get("api_mutex_free")
				.context("failed to get api_ratelimit function")?
				.clone(),
			redis,
		})
	}

	pub async fn acquire<R, T: std::fmt::Display>(
		&self,
		req: impl Into<MutexAquireRequest<T>>,
		f: impl std::future::Future<Output = R>,
	) -> Result<R, MutexError> {
		let req = req.into();
		let lock = Id::<()>::new().to_string();
		let key = req.key.to_string();

		let mut aquired = false;

		for _ in 0..req.attempts {
			match self
				.mutex_lock
				.fcall::<bool, _, _, _>(&self.redis, &[&key], &[&lock, "5"]) // 5 second lock duration
				.await?
			{
				true => {
					aquired = true;
					break;
				}
				false => {
					tokio::time::sleep(req.delay).await;
				}
			}
		}

		if !aquired {
			return Err(MutexError::Acquire(req.attempts));
		}

		let mut f = std::pin::pin!(f);

		loop {
			tokio::select! {
				result = &mut f => {
					if let Err(err) = self
						.mutex_free
						.fcall::<(), _, _, _>(&self.redis, &[&key], &[&lock])
						.await
					{
						tracing::warn!(error = %err, "operation completed but failed to release lock: {}", req.key);
					}

					return Ok(result);
				}
				// Refresh the lock every 2 seconds
				_ = tokio::time::sleep(std::time::Duration::from_secs(2)) => {
					match self
						.mutex_lock
						.fcall::<bool, _, _, _>(&self.redis, &[&key], &[&lock])
						.await?
					{
						true => {},
						false => {
							return Err(MutexError::Lost);
						}
					}
				}
			}
		}
	}
}
