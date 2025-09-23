mod config;
mod collectors;
mod exporters;

use crate::config::{Config, Exporter};
use collectors::cpu::CpuCollector;
use collectors::memory::MemoryCollector;
use collectors::disk::DiskCollector;
use collectors::network::NetworkCollector;
use collectors::system::SystemCollector;
use collectors::Collector;
use reqwest::Client;
use std::env;
use std::fs;
use sysinfo::System;
use tokio::time::{self, Duration};

#[tokio::main]
async fn main() {
    // Check for --oneshot argument
    let oneshot = env::args().any(|arg| arg == "--oneshot");

    // Load configuration
    let config_str = fs::read_to_string("rsysmetrics.toml").expect("Failed to read config file");
    let config: Config = toml::from_str(&config_str).expect("Failed to parse config file");
    println!("Loaded config: {:#?}", config);

    // Get hostname
    let hostname = System::host_name().unwrap_or_else(|| {
        eprintln!("Error: Could not determine hostname.");
        std::process::exit(1);
    });

    // Create collectors
    let mut collectors: Vec<Box<dyn Collector>> = Vec::new();
    if config.collectors.cpu {
        collectors.push(Box::new(CpuCollector::new()));
    }
    if config.collectors.memory {
        collectors.push(Box::new(MemoryCollector::new()));
    }
    if config.collectors.network {
        collectors.push(Box::new(NetworkCollector::new()));
    }
    if config.collectors.disk {
        collectors.push(Box::new(DiskCollector::new()));
    }
    if config.collectors.system {
        collectors.push(Box::new(SystemCollector::new()));
    }

    // Create HTTP client
    let client = Client::new();

    // Start the collection loop
    println!("Starting metrics collection...");
    let mut interval = time::interval(Duration::from_secs(config.collect_interval));

    // The first tick completes immediately, let's consume it
    interval.tick().await;

    if oneshot {
        println!("Running in oneshot mode for testing. Metrics will be printed to the console.");
        let mut metrics = Vec::new();
        for _i in 0..2 {
            metrics.clear();
            for collector in &mut collectors {
                metrics.extend(collector.collect().await);
            }
            if _i == 0 {
                interval.tick().await;
            }
        }
        println!("Collected metrics: {:#?}", metrics);
        println!("\nOneshot mode finished.");
    } else {
        println!("Running in continuous mode. Metrics will be exported to InfluxDB.");
        loop {
            interval.tick().await;
            let mut metrics = Vec::new();
            for collector in &mut collectors {
                metrics.extend(collector.collect().await);
            }

            match &config.exporter {
                Exporter::InfluxDB(influx_config) => {
                    let lines = exporters::influxdb::format_metrics(&metrics, &hostname);
                    if let Err(e) = exporters::influxdb::export_metrics(&client, influx_config, &lines).await {
                        eprintln!("[Error] Failed to export metrics: {:#?}", e);
                    }
                }
            }
        }
    }
}
