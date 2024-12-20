use global::Global;
use scuffle_bootstrap_telemetry::TelemetrySvc;
use scuffle_signal::SignalSvc;

mod cache;
mod cdn_purge;
mod config;
mod global;
mod http;
mod metrics;

scuffle_bootstrap::main! {
	Global {
		http::run,
		cdn_purge::run,
		metrics::run,
		SignalSvc,
		TelemetrySvc,
	}
}
