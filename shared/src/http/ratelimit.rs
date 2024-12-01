use std::collections::HashMap;
use std::sync::Arc;

use ipnet::IpNet;
use rate_limit::{IpType, RuleType};
use scuffle_metrics::metrics;

use crate::config;

#[metrics]
mod rate_limit {
	use ipnet::IpNet;
	use scuffle_metrics::{CounterU64, HistogramF64, MetricEnum};

	#[derive(Debug, Clone, Copy, MetricEnum)]
	pub enum RuleType {
		Subnet,
		Bucket,
	}

	#[derive(Debug, Clone, Copy, MetricEnum)]
	pub enum IpType {
		V4,
		V6,
	}

	pub fn deny(range: IpNet, rule: RuleType, limit: String) -> CounterU64;

	pub fn accept_bucket(ip: IpType, prefix: String) -> HistogramF64;

	pub fn accept_subnet(subnet: String) -> HistogramF64;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum LimitType {
	Subnet(IpNet),
	Bucket(IpNet),
}

struct Limit {
	range: LimitType,
	connections: u64,
}

impl Limit {
	fn report(&self) {
		match self.range {
			LimitType::Subnet(ip) => {
				rate_limit::deny(ip, RuleType::Subnet, self.connections.to_string()).incr();
			}
			LimitType::Bucket(ip) => {
				rate_limit::deny(ip.trunc(), RuleType::Bucket, self.connections.to_string()).incr();
			}
		}
	}

	fn report_accept(&self, count: u64) {
		match self.range {
			LimitType::Subnet(ip) => {
				rate_limit::accept_subnet(ip.to_string()).observe(count as f64);
			}
			LimitType::Bucket(ip) => {
				rate_limit::accept_bucket(
					match ip {
						IpNet::V4(_) => IpType::V4,
						IpNet::V6(_) => IpType::V6,
					},
					ip.prefix_len().to_string(),
				)
				.observe(count as f64);
			}
		}
	}
}

pub struct RateLimiter {
	enabled: bool,
	ip_buckets: spin::Mutex<HashMap<LimitType, u64>>,
	limited_subnets: Vec<config::RateLimitRangeBucket>,
	ipv4_buckets: Vec<config::RateLimitPrefixBucket>,
	ipv6_buckets: Vec<config::RateLimitPrefixBucket>,
}

pub struct RateLimitDropGuard {
	ip: std::net::IpAddr,
	limiter: Option<Arc<RateLimiter>>,
}

impl Drop for RateLimitDropGuard {
	fn drop(&mut self) {
		if let Some(limiter) = self.limiter.take() {
			limiter.release(self.ip);
		}
	}
}

impl RateLimiter {
	pub fn new(config: &config::RateLimit) -> Arc<Self> {
		Arc::new(Self {
			enabled: config.enabled,
			ip_buckets: spin::Mutex::new(HashMap::new()),
			limited_subnets: config.range_buckets.clone(),
			ipv4_buckets: config.ipv4_buckets.clone(),
			ipv6_buckets: config.ipv6_buckets.clone(),
		})
	}

	fn get_limits(&self, ip: std::net::IpAddr) -> Vec<Limit> {
		let limit = self.limited_subnets.iter().filter(|s| s.range.contains(&ip)).min_by(|a, b| {
			// Prefer buckets with a limit over buckets without a limit
			// Prefer lower limits over higher limits
			match (a.concurrent_connections, b.concurrent_connections) {
				(Some(a), Some(b)) => a.cmp(&b),
				(Some(_), None) => std::cmp::Ordering::Less,
				(None, Some(_)) => std::cmp::Ordering::Greater,
				(None, None) => std::cmp::Ordering::Equal,
			}
		});

		if let Some(limit) = limit {
			let Some(connections) = limit.concurrent_connections else {
				return Vec::new();
			};

			vec![Limit {
				range: LimitType::Subnet(limit.range),
				connections,
			}]
		} else {
			match ip {
				std::net::IpAddr::V4(ip) => self
					.ipv4_buckets
					.iter()
					.filter_map(|limit| {
						let ip = IpNet::V4(ipnet::Ipv4Net::new(ip, limit.prefix_length).ok()?.trunc());
						Some(Limit {
							range: LimitType::Bucket(ip),
							connections: limit.concurrent_connections,
						})
					})
					.collect(),
				std::net::IpAddr::V6(ip) => self
					.ipv6_buckets
					.iter()
					.filter_map(|limit| {
						let ip = IpNet::V6(ipnet::Ipv6Net::new(ip, limit.prefix_length).ok()?.trunc());
						Some(Limit {
							range: LimitType::Bucket(ip),
							connections: limit.concurrent_connections,
						})
					})
					.collect(),
			}
		}
	}

	fn release(&self, ip: std::net::IpAddr) {
		let limits = self.get_limits(ip);

		let mut ip_buckets = self.ip_buckets.lock();

		for limit in limits {
			if let Some(value) = ip_buckets.get_mut(&limit.range) {
				*value -= 1;
				if *value == 0 {
					ip_buckets.remove(&limit.range);
				}
			}
		}
	}

	pub fn acquire(self: &Arc<Self>, ip: std::net::IpAddr) -> Option<RateLimitDropGuard> {
		let ip = ip.to_canonical();

		let limits = self.get_limits(ip);
		if limits.is_empty() || !self.enabled {
			return Some(RateLimitDropGuard { ip, limiter: None });
		}

		// If any bucket has a limit of 0 that means we should deny this ip address
		if let Some(limit) = limits.iter().find(|limit| limit.connections == 0) {
			limit.report();
			return None;
		}

		let mut ip_buckets = self.ip_buckets.lock();

		let buckets = limits
			.iter()
			.map(|limit| *ip_buckets.get(&limit.range).unwrap_or(&0))
			.collect::<Vec<_>>();

		if let Some((_, limit)) = buckets
			.iter()
			.copied()
			.zip(limits.iter())
			.find(|(bucket, limit)| bucket + 1 > limit.connections)
		{
			limit.report();
			return None;
		}

		limits.iter().for_each(|limit| {
			let value = ip_buckets.entry(limit.range).or_insert(0);
			*value += 1;
			limit.report_accept(*value);
		});

		Some(RateLimitDropGuard {
			ip,
			limiter: Some(self.clone()),
		})
	}
}
