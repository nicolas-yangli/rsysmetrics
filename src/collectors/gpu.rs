use crate::collectors::Collector;
use async_trait::async_trait;
use crate::collectors::Metric;

#[cfg(target_os = "linux")]
use crate::collectors::linux::gpu::collect_gpu_metrics;

pub struct GpuCollector;

#[async_trait]
impl Collector for GpuCollector {
    fn name(&self) -> &'static str {
        "gpu"
    }

    async fn collect(&mut self) -> Vec<Metric> {
        #[cfg(target_os = "linux")]
        {
            collect_gpu_metrics().await
        }
        #[cfg(not(target_os = "linux"))]
        {
            Vec::new()
        }
    }
}
