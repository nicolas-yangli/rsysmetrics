mod config;
mod collectors;
mod exporters;

use crate::config::{Config, Exporter};
use collectors::cpu::CpuCollector;
use collectors::Collector;
use reqwest::Client;
use std::env;
use std::fs;
use tokio::time::{self, Duration};

#[tokio::main]
async fn main() {
    // Check for --oneshot argument
    let oneshot = env::args().any(|arg| arg == "--oneshot");

    // Load configuration
    let config_str = fs::read_to_string("rsysmetrics.toml").expect("Failed to read config file");
    let config: Config = toml::from_str(&config_str).expect("Failed to parse config file");
    println!("Loaded config: {:#?}", config);

    // Create collectors
    let mut cpu_collector = CpuCollector::new();

    // Create HTTP client
    let client = Client::new();

    // Start the collection loop
    println!("Starting metrics collection...");
    let mut interval = time::interval(Duration::from_secs(config.collect_interval));

    // The first tick completes immediately, let's consume it
    interval.tick().await;

    if oneshot {
        println!("Running in oneshot mode for testing. Metrics will be printed to the console.");
        for i in 0..2 {
            println!("\n--- Sample {} ---", i + 1);
            interval.tick().await;
            let metrics = cpu_collector.collect().await;
            println!("Collected metrics: {:#?}", metrics);
        }
        println!("\nOneshot mode finished.");
    } else {
        println!("Running in continuous mode. Metrics will be exported to InfluxDB.");
        loop {
            interval.tick().await;
            let metrics = cpu_collector.collect().await;

            match &config.exporter {
                Exporter::InfluxDB(influx_config) => {
                    let lines = exporters::influxdb::format_metrics(&metrics);
                    if let Err(e) = exporters::influxdb::export_metrics(&client, influx_config, &lines).await {
                        eprintln!("[Error] Failed to export metrics: {:#?}", e);
                    }
                }
            }
        }
    }
}
