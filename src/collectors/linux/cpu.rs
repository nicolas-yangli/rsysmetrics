use std::collections::HashMap;
use std::fs;
use std::io::{self, BufRead, BufReader};

#[derive(Debug, Default, Clone, Copy)]
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
