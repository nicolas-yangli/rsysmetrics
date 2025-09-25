use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    #[serde(default = "default_collect_interval")]
    pub collect_interval: u64,
    pub exporter: Exporter,
    #[serde(default)]
    pub collectors: Collectors,
}

#[derive(Deserialize, Debug, Default)]
pub struct Collectors {
    #[serde(default = "default_true")]
    pub cpu: bool,
    #[serde(default = "default_true")]
    pub memory: bool,
    #[serde(default = "default_true")]
    pub network: bool,
    #[serde(default = "default_true")]
    pub disk: bool,
    #[serde(default = "default_true")]
    pub system: bool,
    #[serde(default = "default_true")]
    pub gpu: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Deserialize, Debug)]
pub enum Exporter {
    #[serde(rename = "influxdb")]
    InfluxDB(InfluxDBConfig),
}

#[derive(Deserialize, Debug)]
pub struct InfluxDBConfig {
    pub url: String,
    pub token: Option<String>,
    pub bucket: String,
    pub org: String,
}

fn default_collect_interval() -> u64 {
    15
}
