use std::collections::HashSet;
use std::sync::Arc;

use anyhow::Context;
use scuffle_context::ContextFutExt;
use scuffle_metrics::metrics;

use crate::global::Global;

#[metrics]
mod system {
	use std::sync::Arc;

	use scuffle_metrics::{GaugeF64, GaugeU64, MetricEnum};

	#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, MetricEnum)]
	pub enum MemoryKind {
		Total,
		Free,
		Used,
		Shared,
		Buffers,
		Cached,
	}

	pub fn memory(kind: MemoryKind) -> GaugeU64;

	#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, MetricEnum)]
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

	pub fn cpu_times(core: String, kind: CpuTimesKind) -> GaugeU64;

	#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, MetricEnum)]
	pub enum CpuLoadAvgKind {
		One,
		Five,
		Fifteen,
	}

	pub fn cpu_load_avg(kind: CpuLoadAvgKind) -> GaugeF64;

	#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, MetricEnum)]
	pub enum CpuCountKind {
		Physical,
		Logical,
	}

	pub fn cpu_count(kind: CpuCountKind) -> GaugeU64;

	#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, MetricEnum)]
	pub enum CpuStatKind {
		Interrupts,
		CtxSwitches,
		SoftInterrupts,
		Processes,
		ProcsRunning,
		ProcsBlocked,
	}

	pub fn cpu_stats(kind: CpuStatKind) -> GaugeU64;

	#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, MetricEnum)]
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

	pub fn network_stats(interface: Arc<str>, kind: NetworkStatsKind, physical: bool) -> GaugeU64;

	#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, MetricEnum)]
	pub enum UptimeKind {
		Host,
		Application,
	}

	pub fn uptime(kind: UptimeKind) -> GaugeU64;
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

pub async fn run(_: Arc<Global>, ctx: scuffle_context::Context) -> anyhow::Result<()> {
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

	while interval.tick().with_context(&ctx).await.is_some() {
		system::uptime(system::UptimeKind::Application).record(start.elapsed().as_secs());

		// Memory is in MB so we multiply by 1024 * 1024 to get bytes
		match sys_metrics::memory::get_memory() {
			Ok(memory) => {
				system::memory(system::MemoryKind::Total).record(memory.total * 1024 * 1024);
				system::memory(system::MemoryKind::Free).record(memory.free * 1024 * 1024);
				system::memory(system::MemoryKind::Used).record(memory.used * 1024 * 1024);
				system::memory(system::MemoryKind::Shared).record(memory.shared * 1024 * 1024);
				system::memory(system::MemoryKind::Buffers).record(memory.buffers * 1024 * 1024);
				system::memory(system::MemoryKind::Cached).record(memory.cached * 1024 * 1024);
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
					system::cpu_times(core.to_string(), system::CpuTimesKind::Core).record(cpu_times.core as u64);
					system::cpu_times(core.to_string(), system::CpuTimesKind::User).record(cpu_times.user);
					system::cpu_times(core.to_string(), system::CpuTimesKind::Nice).record(cpu_times.nice);
					system::cpu_times(core.to_string(), system::CpuTimesKind::System).record(cpu_times.system);
					system::cpu_times(core.to_string(), system::CpuTimesKind::Idle).record(cpu_times.idle);
					system::cpu_times(core.to_string(), system::CpuTimesKind::IoWait).record(cpu_times.iowait);
					system::cpu_times(core.to_string(), system::CpuTimesKind::Irq).record(cpu_times.irq);
					system::cpu_times(core.to_string(), system::CpuTimesKind::SoftIrq).record(cpu_times.softirq);
					system::cpu_times(core.to_string(), system::CpuTimesKind::Steal).record(cpu_times.steal);
					system::cpu_times(core.to_string(), system::CpuTimesKind::Guest).record(cpu_times.guest);
					system::cpu_times(core.to_string(), system::CpuTimesKind::GuestNice).record(cpu_times.guest_nice);
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
				system::cpu_load_avg(system::CpuLoadAvgKind::One).record(cpu_load_avg.one);
				system::cpu_load_avg(system::CpuLoadAvgKind::Five).record(cpu_load_avg.five);
				system::cpu_load_avg(system::CpuLoadAvgKind::Fifteen).record(cpu_load_avg.fifteen);
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
				system::cpu_count(system::CpuCountKind::Physical).record(cpu_physical_count as u64);
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
				system::cpu_count(system::CpuCountKind::Logical).record(cpu_logical_count as u64);
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
				system::cpu_stats(system::CpuStatKind::Interrupts).record(cpu_stat.interrupts);
				system::cpu_stats(system::CpuStatKind::CtxSwitches).record(cpu_stat.ctx_switches);
				system::cpu_stats(system::CpuStatKind::SoftInterrupts).record(cpu_stat.soft_interrupts);
				system::cpu_stats(system::CpuStatKind::Processes).record(cpu_stat.processes);
				system::cpu_stats(system::CpuStatKind::ProcsRunning).record(cpu_stat.procs_running);
				system::cpu_stats(system::CpuStatKind::ProcsBlocked).record(cpu_stat.procs_blocked);
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
							.record(d.rx_bytes);
						system::network_stats(interface.clone(), system::NetworkStatsKind::RxPackets, physical)
							.record(d.rx_packets);
						system::network_stats(interface.clone(), system::NetworkStatsKind::RxErrors, physical)
							.record(d.rx_errs);
						system::network_stats(interface.clone(), system::NetworkStatsKind::RxDropped, physical)
							.record(d.rx_drop);
						system::network_stats(interface.clone(), system::NetworkStatsKind::TxBytes, physical)
							.record(d.tx_bytes);
						system::network_stats(interface.clone(), system::NetworkStatsKind::TxPackets, physical)
							.record(d.tx_packets);
						system::network_stats(interface.clone(), system::NetworkStatsKind::TxErrors, physical)
							.record(d.tx_errs);
						system::network_stats(interface.clone(), system::NetworkStatsKind::TxDropped, physical)
							.record(d.tx_drop);
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
				system::uptime(system::UptimeKind::Host).record(host_info.uptime);
				state.uptime = true;
			}
			Err(e) if state.uptime => {
				tracing::error!(error = %e, "failed to get host info");
				state.uptime = false;
			}
			_ => {}
		}
	}

	Ok(())
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
