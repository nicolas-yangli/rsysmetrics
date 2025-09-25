use std::fs;
use std::path::{Path, PathBuf};
use crate::collectors::Metric;

fn read_metric(path: &Path) -> Option<f64> {
    fs::read_to_string(path).ok()?.trim().parse::<f64>().ok()
}

fn find_hwmon_path(card_path: &Path) -> Option<PathBuf> {
    let hwmon_path = card_path.join("device/hwmon");
    if let Ok(entries) = fs::read_dir(hwmon_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                return Some(path);
            }
        }
    }
    None
}

fn collect_hwmon_metrics(metrics: &mut Vec<Metric>, hwmon_path: &Path, tags: &Vec<(String, String)>) {
    if let Ok(entries) = fs::read_dir(hwmon_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(file_name) = path.file_name().and_then(|s| s.to_str()) {
                if file_name.starts_with("temp") && file_name.ends_with("_input") {
                    if let Some(value) = read_metric(&path) {
                        let label_path = path.with_file_name(file_name.replace("_input", "_label"));
                        let label = fs::read_to_string(label_path).ok().map_or_else(
                            || "unknown".to_string(),
                            |s| s.trim().to_lowercase().replace(' ', "_"),
                        );
                        metrics.push(Metric {
                            name: format!("gpu_temperature_{}", label),
                            value: value / 1000.0,
                            tags: tags.clone(),
                        });
                    }
                } else if file_name.starts_with("in") && file_name.ends_with("_input") {
                    if let Some(value) = read_metric(&path) {
                        let label_path = path.with_file_name(file_name.replace("_input", "_label"));
                        let label = fs::read_to_string(label_path).ok().map_or_else(
                            || "unknown".to_string(),
                            |s| s.trim().to_lowercase().replace(' ', "_"),
                        );
                        metrics.push(Metric {
                            name: format!("gpu_voltage_{}", label),
                            value,
                            tags: tags.clone(),
                        });
                    }
                }
            }
        }
    }

    if let Some(value) = read_metric(&hwmon_path.join("power1_average")) {
        metrics.push(Metric {
            name: "gpu_power_average".to_string(),
            value: value / 1_000_000.0,
            tags: tags.clone(),
        });
    }

    if let Some(value) = read_metric(&hwmon_path.join("fan1_input")) {
        metrics.push(Metric {
            name: "gpu_fan_speed".to_string(),
            value,
            tags: tags.clone(),
        });
    }
}

pub async fn collect_gpu_metrics() -> Vec<Metric> {
    let mut metrics = Vec::new();
    if let Ok(entries) = fs::read_dir("/sys/class/drm") {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Some(card_name) = path.file_name().and_then(|s| s.to_str()) {
                    if card_name.starts_with("card") {
                        let vendor_path = path.join("device/vendor");
                        if let Ok(vendor) = fs::read_to_string(vendor_path) {
                            if vendor.trim() == "0x1002" {
                                let tags = vec![("card".to_string(), card_name.to_string())];

                                if let Some(hwmon_path) = find_hwmon_path(&path) {
                                    collect_hwmon_metrics(&mut metrics, &hwmon_path, &tags);
                                }

                                if let Some(sclk) = read_metric(&path.join("device/pp_dpm_sclk")) {
                                    metrics.push(Metric {
                                        name: "gpu_core_clock".to_string(),
                                        value: sclk,
                                        tags: tags.clone(),
                                    });
                                }

                                if let Some(mclk) = read_metric(&path.join("device/pp_dpm_mclk")) {
                                    metrics.push(Metric {
                                        name: "gpu_memory_clock".to_string(),
                                        value: mclk,
                                        tags: tags.clone(),
                                    });
                                }

                                if let Some(vram_used) = read_metric(&path.join("device/mem_info_vram_used")) {
                                    metrics.push(Metric {
                                        name: "gpu_vram_used".to_string(),
                                        value: vram_used,
                                        tags: tags.clone(),
                                    });
                                }

                                if let Some(vram_total) = read_metric(&path.join("device/mem_info_vram_total")) {
                                    metrics.push(Metric {
                                        name: "gpu_vram_total".to_string(),
                                        value: vram_total,
                                        tags: tags.clone(),
                                    });
                                }

                                if let Some(gtt_used) = read_metric(&path.join("device/mem_info_gtt_used")) {
                                    metrics.push(Metric {
                                        name: "gpu_gtt_used".to_string(),
                                        value: gtt_used,
                                        tags: tags.clone(),
                                    });
                                }

                                if let Some(gtt_total) = read_metric(&path.join("device/mem_info_gtt_total")) {
                                    metrics.push(Metric {
                                        name: "gpu_gtt_total".to_string(),
                                        value: gtt_total,
                                        tags: tags.clone(),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    metrics
}
