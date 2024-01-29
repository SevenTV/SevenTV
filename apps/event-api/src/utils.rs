use std::pin::Pin;
use std::time::Duration;

use futures_util::Future;
use rand::Rng;
use scuffle_utils::context::{CancelReason, Context};

/// Jitter a duration by up to 10%.
/// This function is useful for adding jitter to timeouts.
/// You might want to add jitter to timeouts to avoid thundering herds, where
/// many threads all wake up at the same time when a timeout expires.
pub fn jitter(duration: Duration) -> Duration {
	let mut rng = rand::thread_rng();
	let jitter = rng.gen_range(0..=duration.as_millis() / 10);
	duration + Duration::from_millis(jitter as u64)
}

/// A future that can be cancelled.
pub trait ContextExt {
	/// Attach a context to a future, allowing it to be cancelled remotely.
	fn context(self, ctx: &Context) -> FutureWithContext<'_, Self>
	where
		Self: Sized;
}

impl<F: Future> ContextExt for F {
	fn context(self, ctx: &Context) -> FutureWithContext<'_, Self> {
		FutureWithContext {
			future: self,
			ctx: ctx.done(),
		}
	}
}

#[pin_project::pin_project]
#[must_use = "futures do nothing unless you `.await` or poll them"]
/// A future that can be cancelled remotely by a context.
pub struct FutureWithContext<'a, F> {
	#[pin]
	future: F,
	ctx: Pin<Box<dyn Future<Output = CancelReason> + 'a + Send + Sync>>,
}

impl<'a, F: Future> Future for FutureWithContext<'a, F> {
	type Output = Result<F::Output, CancelReason>;

	fn poll(mut self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
		let this = self.as_mut().project();

		match this.ctx.as_mut().poll(cx) {
			std::task::Poll::Ready(reason) => {
				return std::task::Poll::Ready(Err(reason));
			}
			std::task::Poll::Pending => {}
		}

		match this.future.poll(cx) {
			std::task::Poll::Ready(v) => std::task::Poll::Ready(Ok(v)),
			std::task::Poll::Pending => std::task::Poll::Pending,
		}
	}
}
