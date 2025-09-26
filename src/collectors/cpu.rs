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
                    let tags = vec![("core".to_string(), cpu_name)];
                    metrics.push(Metric { name: "cpu_time_user".to_string(), value: times.user as f64, tags: tags.clone() });
                    metrics.push(Metric { name: "cpu_time_nice".to_string(), value: times.nice as f64, tags: tags.clone() });
                    metrics.push(Metric { name: "cpu_time_system".to_string(), value: times.system as f64, tags: tags.clone() });
                    metrics.push(Metric { name: "cpu_time_idle".to_string(), value: times.idle as f64, tags: tags.clone() });
                    metrics.push(Metric { name: "cpu_time_iowait".to_string(), value: times.iowait as f64, tags: tags.clone() });
                    metrics.push(Metric { name: "cpu_time_irq".to_string(), value: times.irq as f64, tags: tags.clone() });
                    metrics.push(Metric { name: "cpu_time_softirq".to_string(), value: times.softirq as f64, tags: tags.clone() });
                    metrics.push(Metric { name: "cpu_time_steal".to_string(), value: times.steal as f64, tags: tags.clone() });
                    metrics.push(Metric { name: "cpu_time_guest".to_string(), value: times.guest as f64, tags: tags.clone() });
                    metrics.push(Metric { name: "cpu_time_guest_nice".to_string(), value: times.guest_nice as f64, tags: tags.clone() });
                }
            }
        }

        metrics
    }
}