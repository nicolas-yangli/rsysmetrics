use crate::collectors::Metric;
use crate::config::InfluxDBConfig;
use reqwest::Client;

/// Formats a slice of metrics into InfluxDB line protocol format.
pub fn format_metrics(metrics: &[Metric]) -> String {
    let mut lines = Vec::new();
    for metric in metrics {
        let tags = if metric.tags.is_empty() {
            "".to_string()
        } else {
            metric
                .tags
                .iter()
                .map(|(k, v)| format!( ",{}={}", k, v))
                .collect::<String>()
        };
        lines.push(format!(
            "{}{},host=my-test-host value={}",
            metric.name, tags, metric.value
        ));
    }
    lines.join("\n")
}

/// Exports metrics to InfluxDB.
pub async fn export_metrics(
    client: &Client,
    config: &InfluxDBConfig,
    lines: &str,
) -> Result<(), reqwest::Error> {
    let url = format!("{}/api/v2/write", config.url);

    let mut request_builder = client
        .post(&url)
        .query(&[(&"org", &config.org), (&"bucket", &config.bucket), (&"precision", &"s".to_string())])
        .header("Content-Type", "text/plain; charset=utf-8")
        .body(lines.to_string());

    if let Some(token) = &config.token {
        if !token.is_empty() {
            request_builder = request_builder.header("Authorization", format!("Token {}", token));
        }
    }

    request_builder.send().await?.error_for_status()?;

    Ok(())
}
