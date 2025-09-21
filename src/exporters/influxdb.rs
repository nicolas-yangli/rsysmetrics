use crate::collectors::Metric;
use crate::config::InfluxDBConfig;
use reqwest::Client;

/// Formats a slice of metrics into InfluxDB line protocol format.
pub fn format_metrics(metrics: &[Metric], hostname: &str) -> String {
    if metrics.is_empty() {
        return String::new();
    }

    let mut lines = Vec::new();
    let mut current_line = String::new();
    let mut last_measurement = String::new();
    let mut last_tags = String::new();

    for metric in metrics {
        let mut parts = metric.name.splitn(2, '_');
        let measurement = parts.next().unwrap_or(&metric.name).to_string();
        let field = parts.next().unwrap_or("value").to_string();

        let mut tags = metric.tags.clone();
        tags.sort_by(|a, b| a.0.cmp(&b.0));
        let tags_str = tags
            .iter()
            .map(|(k, v)| format!( ",{}={}", k, v))
            .collect::<String>();

        if measurement == last_measurement && tags_str == last_tags {
            // Same group, append field
            current_line.push_str(&format!( ",{}={}", field, metric.value));
        } else {
            // New group, finalize previous line (if any) and start a new one
            if !current_line.is_empty() {
                lines.push(current_line);
            }
            current_line = format!(
                "{measurement}{tags_str},host={} {}={}",
                hostname, field, metric.value
            );
            last_measurement = measurement;
            last_tags = tags_str;
        }
    }

    // Push the last line
    if !current_line.is_empty() {
        lines.push(current_line);
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
        .query(&[("org", &config.org), ("bucket", &config.bucket), ("precision", &"s".to_string())])
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collectors::Metric;

    #[test]
    fn test_format_metrics() {
        let metrics = vec![
            Metric {
                name: "cpu_usage".to_string(),
                value: 0.5,
                tags: vec![("core".to_string(), "cpu0".to_string())],
            },
            Metric {
                name: "cpu_temperature".to_string(),
                value: 60.0,
                tags: vec![("core".to_string(), "cpu0".to_string())],
            },
            Metric {
                name: "memory_total".to_string(),
                value: 1024.0,
                tags: vec![],
            },
            Metric {
                name: "memory_used".to_string(),
                value: 512.0,
                tags: vec![],
            },
        ];

        let formatted = format_metrics(&metrics, "test-host");
        let expected = "cpu,core=cpu0,host=test-host usage=0.5,temperature=60\nmemory,host=test-host total=1024,used=512";
        assert_eq!(formatted, expected);
    }

    #[test]
    fn test_format_metrics_no_grouping() {
        let metrics = vec![
            Metric {
                name: "cpu_usage".to_string(),
                value: 0.5,
                tags: vec![("core".to_string(), "cpu0".to_string())],
            },
            Metric {
                name: "memory_total".to_string(),
                value: 1024.0,
                tags: vec![],
            },
            Metric {
                name: "cpu_temperature".to_string(),
                value: 60.0,
                tags: vec![("core".to_string(), "cpu0".to_string())],
            },
        ];

        let formatted = format_metrics(&metrics, "test-host");
        let expected = "cpu,core=cpu0,host=test-host usage=0.5\nmemory,host=test-host total=1024\ncpu,core=cpu0,host=test-host temperature=60";
        assert_eq!(formatted, expected);
    }
}
