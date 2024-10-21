use std::collections::HashSet;
use std::sync::Arc;

use anyhow::Context;
use scuffle_foundations::telemetry::metrics::metrics;

#[metrics]
mod system {
	use std::sync::atomic::AtomicU64;
	use std::sync::Arc;

	use scuffle_foundations::telemetry::metrics::prometheus_client::metrics::gauge::Gauge;

	#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
	pub enum MemoryKind {
		Total,
		Free,
		Used,
		Shared,
		Buffers,
		Cached,
	}

	pub fn memory(kind: MemoryKind) -> Gauge;

	#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
	pub enum CpuTimesKind {
		Core,
		User,
		Nice,
		System,
		Idle,
		IoWait,
		Irq,
		SoftIrq,
		Steal,
		Guest,
		GuestNice,
	}

	pub fn cpu_times(core: String, kind: CpuTimesKind) -> Gauge;

	#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
	pub enum CpuLoadAvgKind {
		One,
		Five,
		Fifteen,
	}

	pub fn cpu_load_avg(kind: CpuLoadAvgKind) -> Gauge<f64, AtomicU64>;

	#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
	pub enum CpuCountKind {
		Physical,
		Logical,
	}

	pub fn cpu_count(kind: CpuCountKind) -> Gauge;

	#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
	pub enum CpuStatKind {
		Interrupts,
		CtxSwitches,
		SoftInterrupts,
		Processes,
		ProcsRunning,
		ProcsBlocked,
	}

	pub fn cpu_stats(kind: CpuStatKind) -> Gauge;

	#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
	pub enum NetworkStatsKind {
		RxBytes,
		RxPackets,
		RxErrors,
		RxDropped,
		TxBytes,
		TxPackets,
		TxErrors,
		TxDropped,
	}

	pub fn network_stats(interface: Arc<str>, kind: NetworkStatsKind, physical: bool) -> Gauge;

	#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
	pub enum UptimeKind {
		Host,
		Application,
	}

	pub fn uptime(kind: UptimeKind) -> Gauge;
}

struct RecordState {
	memory: bool,
	cputimes: bool,
	cpuloadavg: bool,
	cpucount_physical: bool,
	cpucount_logical: bool,
	cpustats: bool,
	network: bool,
	uptime: bool,
}

pub async fn recorder() {
	let start = std::time::Instant::now();

	let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));

	let mut state = RecordState {
		memory: true,
		cputimes: true,
		cpuloadavg: true,
		cpucount_physical: true,
		cpucount_logical: true,
		cpustats: true,
		network: true,
		uptime: true,
	};

	loop {
		interval.tick().await;

		system::uptime(system::UptimeKind::Application).set(start.elapsed().as_secs() as i64);

		// Memory is in MB so we multiply by 1024 * 1024 to get bytes
		match sys_metrics::memory::get_memory() {
			Ok(memory) => {
				system::memory(system::MemoryKind::Total).set(memory.total as i64 * 1024 * 1024);
				system::memory(system::MemoryKind::Free).set(memory.free as i64 * 1024 * 1024);
				system::memory(system::MemoryKind::Used).set(memory.used as i64 * 1024 * 1024);
				system::memory(system::MemoryKind::Shared).set(memory.shared as i64 * 1024 * 1024);
				system::memory(system::MemoryKind::Buffers).set(memory.buffers as i64 * 1024 * 1024);
				system::memory(system::MemoryKind::Cached).set(memory.cached as i64 * 1024 * 1024);
				state.memory = true;
			}
			Err(e) if state.memory => {
				tracing::error!(error = %e, "failed to get memory");
				state.memory = false;
			}
			_ => {}
		};

		match sys_metrics::cpu::get_each_cputimes() {
			Ok(cpu_times) => {
				for (core, cpu_times) in cpu_times.iter().enumerate() {
					system::cpu_times(core.to_string(), system::CpuTimesKind::Core).set(cpu_times.core as i64);
					system::cpu_times(core.to_string(), system::CpuTimesKind::User).set(cpu_times.user as i64);
					system::cpu_times(core.to_string(), system::CpuTimesKind::Nice).set(cpu_times.nice as i64);
					system::cpu_times(core.to_string(), system::CpuTimesKind::System).set(cpu_times.system as i64);
					system::cpu_times(core.to_string(), system::CpuTimesKind::Idle).set(cpu_times.idle as i64);
					system::cpu_times(core.to_string(), system::CpuTimesKind::IoWait).set(cpu_times.iowait as i64);
					system::cpu_times(core.to_string(), system::CpuTimesKind::Irq).set(cpu_times.irq as i64);
					system::cpu_times(core.to_string(), system::CpuTimesKind::SoftIrq).set(cpu_times.softirq as i64);
					system::cpu_times(core.to_string(), system::CpuTimesKind::Steal).set(cpu_times.steal as i64);
					system::cpu_times(core.to_string(), system::CpuTimesKind::Guest).set(cpu_times.guest as i64);
					system::cpu_times(core.to_string(), system::CpuTimesKind::GuestNice).set(cpu_times.guest_nice as i64);
				}
				state.cputimes = true;
			}
			Err(e) if state.cputimes => {
				tracing::error!(error = %e, "failed to get cpu times");
				state.cputimes = false;
			}
			_ => {}
		}

		match sys_metrics::cpu::get_loadavg() {
			Ok(cpu_load_avg) => {
				system::cpu_load_avg(system::CpuLoadAvgKind::One).set(cpu_load_avg.one);
				system::cpu_load_avg(system::CpuLoadAvgKind::Five).set(cpu_load_avg.five);
				system::cpu_load_avg(system::CpuLoadAvgKind::Fifteen).set(cpu_load_avg.fifteen);
				state.cpuloadavg = true;
			}
			Err(e) if state.cpuloadavg => {
				tracing::error!(error = %e, "failed to get cpu load avg");
				state.cpuloadavg = false;
			}
			_ => {}
		}

		match sys_metrics::cpu::get_physical_count() {
			Ok(cpu_physical_count) => {
				system::cpu_count(system::CpuCountKind::Physical).set(cpu_physical_count as i64);
				state.cpucount_physical = true;
			}
			Err(e) if state.cpucount_physical => {
				tracing::error!(error = %e, "failed to get cpu count");
				state.cpucount_physical = false;
			}
			_ => {}
		}

		match sys_metrics::cpu::get_logical_count() {
			Ok(cpu_logical_count) => {
				system::cpu_count(system::CpuCountKind::Logical).set(cpu_logical_count as i64);
				state.cpucount_logical = true;
			}
			Err(e) if state.cpucount_logical => {
				tracing::error!(error = %e, "failed to get cpu count");
				state.cpucount_logical = false;
			}
			_ => {}
		}

		match sys_metrics::cpu::get_cpustats() {
			Ok(cpu_stat) => {
				system::cpu_stats(system::CpuStatKind::Interrupts).set(cpu_stat.interrupts as i64);
				system::cpu_stats(system::CpuStatKind::CtxSwitches).set(cpu_stat.ctx_switches as i64);
				system::cpu_stats(system::CpuStatKind::SoftInterrupts).set(cpu_stat.soft_interrupts as i64);
				system::cpu_stats(system::CpuStatKind::Processes).set(cpu_stat.processes as i64);
				system::cpu_stats(system::CpuStatKind::ProcsRunning).set(cpu_stat.procs_running as i64);
				system::cpu_stats(system::CpuStatKind::ProcsBlocked).set(cpu_stat.procs_blocked as i64);
				state.cpustats = true;
			}
			Err(e) if state.cpustats => {
				tracing::error!(error = %e, "failed to get cpu stat");
				state.cpustats = false;
			}
			_ => {}
		}

		match get_network_devices().await {
			Ok((physical_devices, virtual_devices)) => {
				physical_devices
					.into_iter()
					.map(|d| (d, true))
					.chain(virtual_devices.into_iter().map(|d| (d, false)))
					.for_each(|(d, physical)| {
						let interface: Arc<str> = d.interface.into();
						system::network_stats(interface.clone(), system::NetworkStatsKind::RxBytes, physical)
							.set(d.rx_bytes as i64);
						system::network_stats(interface.clone(), system::NetworkStatsKind::RxPackets, physical)
							.set(d.rx_packets as i64);
						system::network_stats(interface.clone(), system::NetworkStatsKind::RxErrors, physical)
							.set(d.rx_errs as i64);
						system::network_stats(interface.clone(), system::NetworkStatsKind::RxDropped, physical)
							.set(d.rx_drop as i64);
						system::network_stats(interface.clone(), system::NetworkStatsKind::TxBytes, physical)
							.set(d.tx_bytes as i64);
						system::network_stats(interface.clone(), system::NetworkStatsKind::TxPackets, physical)
							.set(d.tx_packets as i64);
						system::network_stats(interface.clone(), system::NetworkStatsKind::TxErrors, physical)
							.set(d.tx_errs as i64);
						system::network_stats(interface.clone(), system::NetworkStatsKind::TxDropped, physical)
							.set(d.tx_drop as i64);
					});
				state.network = true;
			}
			Err(e) if state.network => {
				tracing::error!(error = %e, "failed to get network devices");
				state.network = false;
			}
			_ => {}
		}

		match sys_metrics::host::get_host_info() {
			Ok(host_info) => {
				system::uptime(system::UptimeKind::Host).set(host_info.uptime as i64);
				state.uptime = true;
			}
			Err(e) if state.uptime => {
				tracing::error!(error = %e, "failed to get host info");
				state.uptime = false;
			}
			_ => {}
		}
	}
}

async fn get_network_devices() -> anyhow::Result<(Vec<sys_metrics::network::IoNet>, Vec<sys_metrics::network::IoNet>)> {
	let mut virtual_devices_reader = tokio::fs::read_dir("/sys/devices/virtual/net/").await.context("read_dir")?;

	let mut virtual_device_names = HashSet::new();
	while let Some(entry) = virtual_devices_reader.next_entry().await.context("next_entry")? {
		virtual_device_names.insert(entry.file_name().to_string_lossy().to_string());
	}

	let devices = sys_metrics::network::get_ionets().context("get_ionets")?;

	let mut physical_devices = vec![];
	let mut virtual_devices = vec![];

	for interface in devices.into_iter() {
		if virtual_device_names.contains(&interface.interface) {
			virtual_devices.push(interface);
		} else {
			physical_devices.push(interface);
		}
	}

	Ok((physical_devices, virtual_devices))
}
