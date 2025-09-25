#[cfg(target_os = "linux")]
pub mod linux;

pub mod cpu;
pub mod memory;
pub mod network;
pub mod disk;
pub mod system;
pub mod gpu;
pub mod temperature;

use async_trait::async_trait;

// A generic metric point
#[derive(Debug)]
pub struct Metric {
    pub name: String,
    pub value: f64,
    pub tags: Vec<(String, String)>,
}

#[async_trait]
pub trait Collector {
    #[allow(dead_code)]
    fn name(&self) -> &str;
    async fn collect(&mut self) -> Vec<Metric>;
}
