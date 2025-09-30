use std::collections::HashMap;
use std::fs;
use std::io::{self, BufRead, BufReader};

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct CpuTimes {
    pub user: u64,
    pub nice: u64,
    pub system: u64,
    pub idle: u64,
    pub iowait: u64,
    pub irq: u64,
    pub softirq: u64,
    pub steal: u64,
    pub guest: u64,
    pub guest_nice: u64,
}

impl CpuTimes {
    pub fn total(&self) -> u64 {
        self.user
            + self.nice
            + self.system
            + self.idle
            + self.iowait
            + self.irq
            + self.softirq
            + self.steal
            + self.guest
            + self.guest_nice
    }
}

impl std::ops::Sub for CpuTimes {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self {
            user: self.user.saturating_sub(other.user),
            nice: self.nice.saturating_sub(other.nice),
            system: self.system.saturating_sub(other.system),
            idle: self.idle.saturating_sub(other.idle),
            iowait: self.iowait.saturating_sub(other.iowait),
            irq: self.irq.saturating_sub(other.irq),
            softirq: self.softirq.saturating_sub(other.softirq),
            steal: self.steal.saturating_sub(other.steal),
            guest: self.guest.saturating_sub(other.guest),
            guest_nice: self.guest_nice.saturating_sub(other.guest_nice),
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct CpuUsage {
    pub user: f64,
    pub system: f64,
    pub idle: f64,
    pub iowait: f64,
    pub irq: f64,
    pub softirq: f64,
    pub steal: f64,
    pub guest: f64,
}

pub fn normalize(times: CpuTimes) -> CpuUsage {
    let total = times.total();
    if total == 0 {
        return CpuUsage::default();
    }
    let total = total as f64;
    CpuUsage {
        user: ((times.user + times.nice) as f64 / total) * 100.0,
        system: (times.system as f64 / total) * 100.0,
        idle: (times.idle as f64 / total) * 100.0,
        iowait: (times.iowait as f64 / total) * 100.0,
        irq: (times.irq as f64 / total) * 100.0,
        softirq: (times.softirq as f64 / total) * 100.0,
        steal: (times.steal as f64 / total) * 100.0,
        guest: ((times.guest + times.guest_nice) as f64 / total) * 100.0,
    }
}

#[derive(Debug, Default)]
pub struct CpuTimesCollector {
    last_times: HashMap<String, CpuTimes>,
}

impl CpuTimesCollector {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn collect(&mut self) -> io::Result<HashMap<String, CpuTimes>> {
        let file = fs::File::open("/proc/stat")?;
        let reader = BufReader::new(file);
        self.collect_from_reader(reader)
    }

    pub fn collect_from_reader<R: BufRead>(&mut self, reader: R) -> io::Result<HashMap<String, CpuTimes>> {
        let mut current_times = HashMap::new();
        let mut deltas = HashMap::new();

        for line in reader.lines() {
            let line = line?;
            if line.starts_with("cpu") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 9 { // cpu + at least 8 fields
                    let cpu_name = parts[0].to_string();
                    let times = CpuTimes {
                        user: parts[1].parse().unwrap_or(0),
                        nice: parts[2].parse().unwrap_or(0),
                        system: parts[3].parse().unwrap_or(0),
                        idle: parts[4].parse().unwrap_or(0),
                        iowait: parts[5].parse().unwrap_or(0),
                        irq: parts[6].parse().unwrap_or(0),
                        softirq: parts[7].parse().unwrap_or(0),
                        steal: parts[8].parse().unwrap_or(0),
                        guest: if parts.len() > 9 { parts[9].parse().unwrap_or(0) } else { 0 },
                        guest_nice: if parts.len() > 10 { parts[10].parse().unwrap_or(0) } else { 0 },
                    };

                    if let Some(last) = self.last_times.get(&cpu_name) {
                        deltas.insert(cpu_name.clone(), times - *last);
                    }
                    current_times.insert(cpu_name, times);
                }
            }
        }

        self.last_times = current_times;
        Ok(deltas)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    const PROC_STAT_SAMPLE_1: &str = r#"cpu  62191 107 55994 16988691 15914 5073 2279 0 0 0
cpu0 2676 4 3034 527276 942 280 670 0 0 0
"#;

    const PROC_STAT_SAMPLE_2: &str = r#"cpu  62291 107 56094 16989691 15914 5073 2279 0 0 0
cpu0 2776 4 3134 527376 942 280 670 0 0 0
"#;

    #[test]
    fn test_collect_from_reader() {
        let mut collector = CpuTimesCollector::new();

        // First collection, should return no deltas
        let reader1 = BufReader::new(Cursor::new(PROC_STAT_SAMPLE_1));
        let deltas1 = collector.collect_from_reader(reader1).unwrap();
        assert!(deltas1.is_empty());

        // Second collection, should return deltas
        let reader2 = BufReader::new(Cursor::new(PROC_STAT_SAMPLE_2));
        let deltas2 = collector.collect_from_reader(reader2).unwrap();
        
        assert_eq!(deltas2.len(), 2);

        let expected_delta_cpu = CpuTimes {
            user: 100,
            nice: 0,
            system: 100,
            idle: 1000,
            iowait: 0,
            irq: 0,
            softirq: 0,
            steal: 0,
            guest: 0,
            guest_nice: 0,
        };
        assert_eq!(deltas2.get("cpu"), Some(&expected_delta_cpu));

        let expected_delta_cpu0 = CpuTimes {
            user: 100,
            nice: 0,
            system: 100,
            idle: 100,
            iowait: 0,
            irq: 0,
            softirq: 0,
            steal: 0,
            guest: 0,
            guest_nice: 0,
        };
        assert_eq!(deltas2.get("cpu0"), Some(&expected_delta_cpu0));
    }

    #[test]
    fn test_normalize() {
        let times = CpuTimes {
            user: 100,
            nice: 100,
            system: 100,
            idle: 600,
            iowait: 0,
            irq: 0,
            softirq: 0,
            steal: 0,
            guest: 50,
            guest_nice: 50,
        };

        let usage = normalize(times);

        assert!((usage.user - 20.0).abs() < 1e-9);
        assert!((usage.system - 10.0).abs() < 1e-9);
        assert!((usage.idle - 60.0).abs() < 1e-9);
        assert!((usage.guest - 10.0).abs() < 1e-9);
    }
}
