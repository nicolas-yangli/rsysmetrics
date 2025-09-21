use async_trait::async_trait;
#[cfg(not(target_os = "linux"))]
use sysinfo::System;

use super::{Collector, Metric};

#[cfg(target_os = "linux")]
use super::linux;

pub struct MemoryCollector {
    #[cfg(not(target_os = "linux"))]
    system: System,
    #[cfg(target_os = "linux")]
    linux_collector: linux::memory::LinuxMemoryCollector,
}

impl MemoryCollector {
    pub fn new() -> Self {
        Self {
            #[cfg(not(target_os = "linux"))]
            system: System::new(),
            #[cfg(target_os = "linux")]
            linux_collector: linux::memory::LinuxMemoryCollector::new(),
        }
    }
}

#[async_trait]
impl Collector for MemoryCollector {
    fn name(&self) -> &str {
        "memory"
    }

    async fn collect(&mut self) -> Vec<Metric> {
        #[cfg(target_os = "linux")]
        {
            self.linux_collector.collect().await
        }

        #[cfg(not(target_os = "linux"))]
        {
            self.system.refresh_memory();
            let mut metrics = Vec::new();

            metrics.push(Metric {
                name: "memory_total".to_string(),
                value: self.system.total_memory() as f64,
                tags: vec![],
            });

            metrics.push(Metric {
                name: "memory_used".to_string(),
                value: self.system.used_memory() as f64,
                tags: vec![],
            });

            metrics.push(Metric {
                name: "memory_available".to_string(),
                value: self.system.available_memory() as f64,
                tags: vec![],
            });

            metrics.push(Metric {
                name: "memory_free".to_string(),
                value: self.system.free_memory() as f64,
                tags: vec![],
            });

            metrics.push(Metric {
                name: "swap_total".to_string(),
                value: self.system.total_swap() as f64,
                tags: vec![],
            });

            metrics.push(Metric {
                name: "swap_used".to_string(),
                value: self.system.used_swap() as f64,
                tags: vec![],
            });

            metrics.push(Metric {
                name: "swap_free".to_string(),
                value: self.system.free_swap() as f64,
                tags: vec![],
            });

            metrics
        }
    }
}