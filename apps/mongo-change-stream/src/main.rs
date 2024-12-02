use global::Global;
use scuffle_bootstrap_telemetry::TelemetrySvc;
use scuffle_signal::SignalSvc;

mod config;
mod global;
mod stream;

scuffle_bootstrap::main! {
	Global {
		stream::run,
		SignalSvc,
		TelemetrySvc,
	}
}
