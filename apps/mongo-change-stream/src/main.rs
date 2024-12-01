use global::Global;
use scuffle_signal::SignalSvc;

mod config;
mod global;
mod stream;

scuffle_bootstrap::main! {
	Global {
		stream::run,
		SignalSvc,
		scuffle_bootstrap_telemetry::TelemetrySvc,
	}
}
