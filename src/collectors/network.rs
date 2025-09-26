use async_trait::async_trait;
use sysinfo::{Networks};

use super::{Collector, Metric};

#[cfg(target_os = "linux")]
use std::sync::LazyLock;
#[cfg(target_os = "linux")]
use regex::Regex;

#[cfg(target_os = "linux")]
static INTERFACE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(en|wl|ww)").unwrap()
});

pub struct NetworkCollector {
    networks: Networks,
}

impl NetworkCollector {
    pub fn new() -> Self {
        NetworkCollector {
            networks: Networks::new_with_refreshed_list(),
        }
    }
}

#[async_trait]
impl Collector for NetworkCollector {
    fn name(&self) -> &str {
        "network"
    }

    async fn collect(&mut self) -> Vec<Metric> {
        self.networks.refresh();
        let mut metrics = Vec::new();

        for (interface_name, data) in self.networks.iter() {
            #[cfg(target_os = "linux")]
            if !INTERFACE_RE.is_match(interface_name) {
                continue;
            }
            let tags = vec![("interface".to_string(), interface_name.to_string())];
            metrics.push(Metric {
                name: "network_received".to_string(),
                value: data.total_received() as f64,
                tags: tags.clone(),
            });
            metrics.push(Metric {
                name: "network_transmitted".to_string(),
                value: data.total_transmitted() as f64,
                tags: tags.clone(),
            });
            metrics.push(Metric {
                name: "network_packets_received".to_string(),
                value: data.packets_received() as f64,
                tags: tags.clone(),
            });
            metrics.push(Metric {
                name: "network_packets_transmitted".to_string(),
                value: data.packets_transmitted() as f64,
                tags: tags.clone(),
            });
        }

        metrics
    }
}
