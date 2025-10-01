use std::collections::HashMap;
use std::fs;
use std::io::{self, BufRead, BufReader};
use regex::Regex;
use std::sync::LazyLock;

static DEVICE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(nvme[0-9]+n[0-9]+|sd[a-z]+)$").unwrap()
});

#[derive(Debug, Default, Clone, PartialEq)]
pub struct DiskIo {
    pub read_bytes: u64,
    pub written_bytes: u64,
    pub reads: u64,
    pub writes: u64,
    pub read_time: u64,
    pub write_time: u64,
    pub io_in_progress: u64,
    pub disk_id: String,
    pub temperature: Option<HashMap<String, f64>>,
}



#[derive(Debug, Default)]
pub struct DiskIoCollector {
    device_to_id: HashMap<String, String>,
}

use std::path::Path;

impl DiskIoCollector {
    pub fn new() -> Self {
        let mut device_to_id = HashMap::new();
        if let Ok(entries) = fs::read_dir("/dev/disk/by-id/") {
            for entry in entries {
                if let Ok(entry) = entry {
                    let id_path = entry.path();
                    if let Ok(target_path) = fs::read_link(&id_path) {
                        if let Some(target_str) = target_path.to_str() {
                            if let Some(device_name) = Path::new(target_str).file_name().and_then(|s| s.to_str()) {
                                if let Some(id_str) = id_path.file_name().and_then(|s| s.to_str()) {
                                    device_to_id.insert(device_name.to_string(), id_str.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }

        Self {
            device_to_id,
        }
    }

    #[cfg(test)]
    pub fn new_with_device_to_id_mapping(device_to_id: HashMap<String, String>) -> Self {
        Self {
            device_to_id,
        }
    }

    pub fn collect(&self) -> io::Result<HashMap<String, DiskIo>> {
        let file = fs::File::open("/proc/diskstats")?;
        let reader = BufReader::new(file);
        self.collect_from_reader(reader, None)
    }

    pub fn collect_from_reader<R: BufRead>(&self, reader: R, root_path: Option<&Path>) -> io::Result<HashMap<String, DiskIo>> {
        let mut current_io = HashMap::new();

        for line in reader.lines() {
            let line = line?;
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 14 {
                continue;
            }

            let device_name = parts[2].to_string();

            if !DEVICE_RE.is_match(&device_name) {
                continue;
            }

            let reads = parts[3].parse().unwrap_or(0);
            let read_sectors = parts[5].parse().unwrap_or(0);
            let read_time = parts[6].parse().unwrap_or(0);
            let writes = parts[7].parse().unwrap_or(0);
            let written_sectors = parts[9].parse().unwrap_or(0);
            let write_time = parts[10].parse().unwrap_or(0);
            let io_in_progress = parts[11].parse().unwrap_or(0);

            let disk_id = self.device_to_id.get(&device_name).unwrap_or(&device_name).clone();
            let temperature = self.get_disk_temperatures(&device_name, root_path).ok();

            let io = DiskIo {
                read_bytes: read_sectors * 512,
                written_bytes: written_sectors * 512,
                reads,
                writes,
                read_time,
                write_time,
                io_in_progress,
                disk_id,
                temperature,
            };

            current_io.insert(device_name, io);
        }

        Ok(current_io)
    }

    fn get_disk_temperatures(&self, device_name: &str, root_path: Option<&Path>) -> io::Result<HashMap<String, f64>> {
        let device_path = root_path.unwrap_or_else(|| Path::new("/sys/class/block")).join(device_name).join("device");
        let mut temperatures = HashMap::new();

        for entry in fs::read_dir(device_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() && entry.file_name().to_string_lossy().starts_with("hwmon") {
                for temp_entry in fs::read_dir(path)? {
                    let temp_entry = temp_entry?;
                    let temp_path = temp_entry.path();
                    if let Some(file_name) = temp_path.file_name().and_then(|s| s.to_str()) {
                        if file_name.starts_with("temp") && file_name.ends_with("_input") {
                            let label_path = temp_path.with_file_name(file_name.replace("_input", "_label"));
                            let label = fs::read_to_string(label_path)
                                .unwrap_or_else(|_| "temp".to_string())
                                .trim()
                                .to_string();
                            if let Ok(temp_str) = fs::read_to_string(&temp_path) {
                                if let Ok(temp) = temp_str.trim().parse::<f64>() {
                                    temperatures.insert(label, temp / 1000.0);
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(temperatures)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;



    const PROC_DISKSTATS_SAMPLE: &str = r#"259       0 nvme0n1 77560 2503 6796613 23765 210848 619 7175916 2434865 0 66886 2481479 3691 0 52413048 14264 4799 8583
259       1 nvme0n1p1 611 1645 12829 173 14 0 12 12 0 31 185 0 0 0 0 0 0
259       2 nvme0n1p2 76856 858 6780736 23589 210830 619 7175904 2434850 0 71586 2472704 3691 0 52413048 14264 0 0
259       3 nvme1n1 82 0 2936 18 0 0 0 0 0 18 18 0 0 0 0 0 0
253       0 dm-0 77671 0 6779560 25579 211446 0 7175904 155364 0 72803 193638 3691 0 52413048 12695 0 0
8       0 sda 200 0 2000 0 200 0 2000 0 0 0 0
8       1 sda1 20 0 200 0 20 0 200 0 0 0 0
7       0 loop0 20 0 200 0 20 0 200 0 0 0 0
240     16 zd16 20 0 200 0 20 0 200 0 0 0 0
240     17 zd16p1 20 0 200 0 20 0 200 0 0 0 0
"#;

    #[test]
    fn test_collect_from_reader() {
        let mut mapping = HashMap::new();
        mapping.insert("nvme0n1".to_string(), "nvme-eui.0123456789abcdef".to_string());
        mapping.insert("sda".to_string(), "ata-VBOX_HARDDISK_VB0d1a2b3c-4d5e6f7a8b9c".to_string());
        let collector = DiskIoCollector::new_with_device_to_id_mapping(mapping);

        // Create a mock sysfs directory
        let temp_dir = tempfile::tempdir().unwrap();
        let mock_sysfs = temp_dir.path();
        let device_path = mock_sysfs.join("nvme0n1").join("device");
        fs::create_dir_all(&device_path).unwrap();
        let hwmon_path = device_path.join("hwmon1");
        fs::create_dir(&hwmon_path).unwrap();
        fs::write(hwmon_path.join("temp1_input"), "36850").unwrap();
        fs::write(hwmon_path.join("temp1_label"), "Composite").unwrap();
        fs::write(hwmon_path.join("temp2_input"), "35850").unwrap();
        fs::write(hwmon_path.join("temp2_label"), "Sensor 1").unwrap();

        // Collection should return the raw values from the sample
        let reader = BufReader::new(Cursor::new(PROC_DISKSTATS_SAMPLE));
        let result = collector.collect_from_reader(reader, Some(mock_sysfs)).unwrap();
        
        assert_eq!(result.len(), 3);

        let mut expected_temps = HashMap::new();
        expected_temps.insert("Composite".to_string(), 36.85);
        expected_temps.insert("Sensor 1".to_string(), 35.85);

        let expected_nvme0n1 = DiskIo {
            read_bytes: 6796613 * 512,
            written_bytes: 7175916 * 512,
            reads: 77560,
            writes: 210848,
            read_time: 23765,
            write_time: 2434865,
            io_in_progress: 0,
            disk_id: "nvme-eui.0123456789abcdef".to_string(),
            temperature: Some(expected_temps),
        };
        assert_eq!(result.get("nvme0n1"), Some(&expected_nvme0n1));

        let expected_nvme1n1 = DiskIo {
            read_bytes: 2936 * 512,
            written_bytes: 0,
            reads: 82,
            writes: 0,
            read_time: 18,
            write_time: 0,
            io_in_progress: 0,
            disk_id: "nvme1n1".to_string(),
            temperature: None,
        };
        assert_eq!(result.get("nvme1n1"), Some(&expected_nvme1n1));

        let expected_sda = DiskIo {
            read_bytes: 2000 * 512,
            written_bytes: 2000 * 512,
            reads: 200,
            writes: 200,
            read_time: 0,
            write_time: 0,
            io_in_progress: 0,
            disk_id: "ata-VBOX_HARDDISK_VB0d1a2b3c-4d5e6f7a8b9c".to_string(),
            temperature: None,
        };
        assert_eq!(result.get("sda"), Some(&expected_sda));
    }
}
