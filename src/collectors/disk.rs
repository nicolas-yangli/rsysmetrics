use async_trait::async_trait;

use super::{Collector, Metric};

#[cfg(target_os = "linux")]
use super::linux;

pub struct DiskCollector {
    #[cfg(target_os = "linux")]
    collector: linux::disk::DiskIoCollector,
}

impl DiskCollector {
    pub fn new() -> Self {
        DiskCollector {
            #[cfg(target_os = "linux")]
            collector: linux::disk::DiskIoCollector::new(),
        }
    }
}

#[async_trait]
impl Collector for DiskCollector {
    fn name(&self) -> &str {
        "disk"
    }

    async fn collect(&mut self) -> Vec<Metric> {
        #[cfg(target_os = "linux")]
        {
            if let Ok(io_map) = self.collector.collect() {
                let mut metrics = Vec::new();
                for (disk_name, io) in io_map {
                    let tags = vec![
                        ("device".to_string(), disk_name),
                        ("disk_id".to_string(), io.disk_id),
                    ];
                    metrics.push(Metric { name: "disk_read_bytes".to_string(), value: io.read_bytes as f64, tags: tags.clone() });
                    metrics.push(Metric { name: "disk_written_bytes".to_string(), value: io.written_bytes as f64, tags: tags.clone() });
                    metrics.push(Metric { name: "disk_reads".to_string(), value: io.reads as f64, tags: tags.clone() });
                    metrics.push(Metric { name: "disk_writes".to_string(), value: io.writes as f64, tags: tags.clone() });
                    metrics.push(Metric { name: "disk_read_time".to_string(), value: io.read_time as f64, tags: tags.clone() });
                    metrics.push(Metric { name: "disk_write_time".to_string(), value: io.write_time as f64, tags: tags.clone() });
                    metrics.push(Metric { name: "disk_io_in_progress".to_string(), value: io.io_in_progress as f64, tags: tags.clone() });
                }
                return metrics;
            }
        }

        Vec::new()
    }
}
