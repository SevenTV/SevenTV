use global::Global;
use scuffle_bootstrap_telemetry::TelemetrySvc;
use scuffle_signal::SignalSvc;

mod config;
mod global;
mod http;
mod subscription;
mod utils;

scuffle_bootstrap::main! {
	Global {
		http::run,
		subscription::run,
		SignalSvc,
		TelemetrySvc,
	}
}
