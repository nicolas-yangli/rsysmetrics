use async_trait::async_trait;
use sysinfo::System;

use super::{Collector, Metric};

#[cfg(target_os = "linux")]
use super::linux;

pub struct CpuCollector {
    system: System,
    #[cfg(target_os = "linux")]
    times_collector: linux::cpu::CpuTimesCollector,
}

impl CpuCollector {
    pub fn new() -> Self {
        CpuCollector {
            system: System::new(),
            #[cfg(target_os = "linux")]
            times_collector: linux::cpu::CpuTimesCollector::new(),
        }
    }
}

#[async_trait]
impl Collector for CpuCollector {
    fn name(&self) -> &str {
        "cpu"
    }

    async fn collect(&mut self) -> Vec<Metric> {
        self.system.refresh_cpu_usage();
        let mut metrics = Vec::new();

        // Per-core usage (cross-platform)
        for cpu in self.system.cpus() {
            metrics.push(Metric {
                name: "cpu_usage".to_string(),
                value: cpu.cpu_usage() as f64,
                tags: vec![("core".to_string(), cpu.name().to_string())],
            });
        }

        // Detailed CPU times (Linux-only)
        #[cfg(target_os = "linux")]
        {
            if let Ok(times_map) = self.times_collector.collect() {
                for (cpu_name, times) in times_map {
                    let usage = linux::cpu::normalize(times);
                    let tags = vec![("core".to_string(), cpu_name)];
                    metrics.push(Metric { name: "cpu_usage_user".to_string(), value: usage.user, tags: tags.clone() });
                    metrics.push(Metric { name: "cpu_usage_system".to_string(), value: usage.system, tags: tags.clone() });
                    metrics.push(Metric { name: "cpu_usage_idle".to_string(), value: usage.idle, tags: tags.clone() });
                    metrics.push(Metric { name: "cpu_usage_iowait".to_string(), value: usage.iowait, tags: tags.clone() });
                    metrics.push(Metric { name: "cpu_usage_irq".to_string(), value: usage.irq, tags: tags.clone() });
                    metrics.push(Metric { name: "cpu_usage_softirq".to_string(), value: usage.softirq, tags: tags.clone() });
                    metrics.push(Metric { name: "cpu_usage_steal".to_string(), value: usage.steal, tags: tags.clone() });
                    metrics.push(Metric { name: "cpu_usage_guest".to_string(), value: usage.guest, tags: tags.clone() });
                }
            }
        }

        metrics
    }
}