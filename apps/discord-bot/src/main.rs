use global::Global;
use scuffle_bootstrap_telemetry::TelemetrySvc;
use scuffle_signal::SignalSvc;

mod bot;
mod config;
mod global;

scuffle_bootstrap::main! {
	Global {
		bot::run,
		SignalSvc,
		TelemetrySvc,
	}
}
