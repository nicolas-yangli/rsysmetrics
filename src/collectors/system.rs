use async_trait::async_trait;
use sysinfo::System;

use super::{Collector, Metric};

pub struct SystemCollector;

impl SystemCollector {
    pub fn new() -> Self {
        SystemCollector
    }
}

#[async_trait]
impl Collector for SystemCollector {
    fn name(&self) -> &str {
        "system"
    }

    async fn collect(&mut self) -> Vec<Metric> {
        let mut metrics = Vec::new();

        // Uptime
        let uptime = System::uptime();
        metrics.push(Metric {
            name: "system_uptime".to_string(),
            value: uptime as f64,
            tags: vec![],
        });

        // Load average
        let load_avg = System::load_average();
        metrics.push(Metric {
            name: "system_load_average_1m".to_string(),
            value: load_avg.one,
            tags: vec![],
        });
        metrics.push(Metric {
            name: "system_load_average_5m".to_string(),
            value: load_avg.five,
            tags: vec![],
        });
        metrics.push(Metric {
            name: "system_load_average_15m".to_string(),
            value: load_avg.fifteen,
            tags: vec![],
        });

        metrics
    }
}
