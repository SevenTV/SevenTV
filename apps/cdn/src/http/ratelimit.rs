use std::sync::Arc;

use ipnet::IpNet;
use scc::hash_map::OccupiedEntry;

use crate::config;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum LimitType {
    Subnet(IpNet),
    Bucket(IpNet),
}

struct Limit {
    range: LimitType,
    connections: u64,
}

pub struct RateLimiter {
    ip_buckets: scc::HashMap<LimitType, u64>,
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

struct DropZero<'a>(Option<OccupiedEntry<'a, LimitType, u64>>);

impl<'a> DropZero<'a> {
    fn new(entry: OccupiedEntry<'a, LimitType, u64>) -> Self {
        Self(Some(entry))
    }

    fn get(&self) -> u64 {
        self.0.as_ref().map(|entry| *entry.get()).unwrap_or(0)
    }

    fn increment(&mut self) {
        if let Some(entry) = self.0.as_mut() {
            *entry.get_mut() += 1;
        }
    }

    fn decrement(&mut self) {
        if let Some(entry) = self.0.as_mut() {
            if *entry.get() != 0 {
                *entry.get_mut() -= 1;
            }
        }
    }
}

impl Drop for DropZero<'_> {
    fn drop(&mut self) {
        if let Some(entry) = self.0.take() {
            if *entry.get() == 0 {
                let _ = entry.remove();
            }
        }
    }
}

impl RateLimiter {
    pub fn new(config: &config::RateLimit) -> Arc<Self> {
        Arc::new(Self {
            ip_buckets: scc::HashMap::new(),
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
                std::net::IpAddr::V4(ip) => self.ipv4_buckets.iter().filter_map(|limit| {
                    Some(Limit {
                        range: LimitType::Bucket(IpNet::V4(ipnet::Ipv4Net::new(ip, limit.prefix_length).ok()?.trunc())),
                        connections: limit.concurrent_connections,
                    })
                }).collect(),
                std::net::IpAddr::V6(ip) => self.ipv6_buckets.iter().filter_map(|limit| {
                    Some(Limit {
                        range: LimitType::Bucket(IpNet::V6(ipnet::Ipv6Net::new(ip, limit.prefix_length).ok()?.trunc())),
                        connections: limit.concurrent_connections,
                    })
                }).collect(),
            }
        }
    }

    fn release(&self, ip: std::net::IpAddr) {
        let limits = self.get_limits(ip);

        for limit in limits {
            DropZero::new(self.ip_buckets.entry(limit.range).or_insert_with(|| 0)).decrement();
        }
    }

    pub fn acquire(self: &Arc<Self>, ip: std::net::IpAddr) -> Option<RateLimitDropGuard> {
        let limits = self.get_limits(ip);
        if limits.is_empty() {
            return Some(RateLimitDropGuard {
                ip,
                limiter: None,
            });
        }

        // If any bucket has a limit of 0 that means we should deny this ip address
        if limits.iter().any(|limit| limit.connections == 0) {
            return None;
        }

        let buckets = limits.iter().map(|limit| self.ip_buckets.entry(limit.range).or_insert_with(|| 0)).map(DropZero::new).collect::<Vec<_>>();

        if buckets.iter().zip(limits.iter()).any(|(bucket, limit)| bucket.get() + 1 > limit.connections) {  
            return None;
        }

        for mut bucket in buckets {
            bucket.increment();
        }

        Some(RateLimitDropGuard {
            ip,
            limiter: Some(self.clone()),
        })
    }
}
