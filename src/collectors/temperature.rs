use async_trait::async_trait;
use globset::{Glob, GlobSet, GlobSetBuilder};
use sysinfo::Components;

use crate::config::TemperatureCollectorConfig;

use super::{Collector, Metric};

pub struct TemperatureCollector {
    components: Components,
    include: GlobSet,
    exclude: GlobSet,
}

impl TemperatureCollector {
    pub fn new(config: TemperatureCollectorConfig) -> Self {
        let mut include_builder = GlobSetBuilder::new();
        for pattern in &config.include {
            match Glob::new(pattern) {
                Ok(glob) => {
                    include_builder.add(glob);
                }
                Err(e) => {
                    eprintln!("Invalid include pattern '{}': {}", pattern, e);
                }
            }
        }

        let mut exclude_builder = GlobSetBuilder::new();
        for pattern in &config.exclude {
            match Glob::new(pattern) {
                Ok(glob) => {
                    exclude_builder.add(glob);
                }
                Err(e) => {
                    eprintln!("Invalid exclude pattern '{}': {}", pattern, e);
                }
            }
        }

        let include = include_builder.build().unwrap_or_else(|e| {
            eprintln!("Error building include globset: {}", e);
            GlobSet::empty()
        });
        let exclude = exclude_builder.build().unwrap_or_else(|e| {
            eprintln!("Error building exclude globset: {}", e);
            GlobSet::empty()
        });

        TemperatureCollector {
            components: Components::new_with_refreshed_list(),
            include,
            exclude,
        }
    }
}

#[async_trait]
impl Collector for TemperatureCollector {
    fn name(&self) -> &str {
        "temperature"
    }

    async fn collect(&mut self) -> Vec<Metric> {
        self.components.refresh(false);
        let mut metrics = Vec::new();

        for component in &self.components {
            let component_label = component.label();

            if !self.include.is_empty() && !self.include.is_match(component_label) {
                continue;
            }

            if self.exclude.is_match(component_label) {
                continue;
            }

            if let Some(temperature) = component.temperature() {
                let mut tags = Vec::new();
                tags.push(("label".to_string(), component.label().to_string()));
                metrics.push(Metric {
                    name: "temperature".to_string(),
                    value: temperature as f64,
                    tags,
                });
            }
        }

        metrics
    }
}
