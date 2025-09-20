#[cfg(target_os = "linux")]
pub mod linux;

pub mod cpu;

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
    fn name(&self) -> &str;
    async fn collect(&mut self) -> Vec<Metric>;
}
