use async_trait::async_trait;
use std::collections::HashMap;
use std::fs;
use std::io::{self, BufRead, BufReader};

use crate::collectors::{Collector, Metric};

#[derive(Debug, Default)]
pub struct LinuxMemoryCollector {
    // No state needed for now
}

impl LinuxMemoryCollector {
    pub fn new() -> Self {
        Self::default()
    }

    fn parse_meminfo<R: BufRead>(&self, reader: R) -> io::Result<HashMap<String, u64>> {
        let mut meminfo = HashMap::new();

        for line in reader.lines() {
            let line = line?;
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let key = parts[0].trim_end_matches(':').to_string();
                let value = parts[1].parse().unwrap_or(0);
                meminfo.insert(key, value);
            }
        }

        Ok(meminfo)
    }

    fn build_metrics(&self, meminfo: &HashMap<String, u64>) -> Vec<Metric> {
        let mut metrics = Vec::new();
        let mem_total = meminfo.get("MemTotal").cloned().unwrap_or(0) as f64 * 1024.0;
        let mem_free = meminfo.get("MemFree").cloned().unwrap_or(0) as f64 * 1024.0;
        let mem_available = meminfo.get("MemAvailable").cloned().unwrap_or(0) as f64 * 1024.0;
        let buffers = meminfo.get("Buffers").cloned().unwrap_or(0) as f64 * 1024.0;
        let cached = meminfo.get("Cached").cloned().unwrap_or(0) as f64 * 1024.0;
        let mem_used = mem_total - mem_free;

        metrics.push(Metric { name: "memory_total".to_string(), value: mem_total, tags: vec![] });
        metrics.push(Metric { name: "memory_used".to_string(), value: mem_used, tags: vec![] });
        metrics.push(Metric { name: "memory_free".to_string(), value: mem_free, tags: vec![] });
        metrics.push(Metric { name: "memory_available".to_string(), value: mem_available, tags: vec![] });
        metrics.push(Metric { name: "memory_buffered".to_string(), value: buffers, tags: vec![] });
        metrics.push(Metric { name: "memory_cached".to_string(), value: cached, tags: vec![] });

        let swap_total = meminfo.get("SwapTotal").cloned().unwrap_or(0) as f64 * 1024.0;
        let swap_free = meminfo.get("SwapFree").cloned().unwrap_or(0) as f64 * 1024.0;
        let swap_used = swap_total - swap_free;
        let swap_cached = meminfo.get("SwapCached").cloned().unwrap_or(0) as f64 * 1024.0;

        metrics.push(Metric { name: "swap_total".to_string(), value: swap_total, tags: vec![] });
        metrics.push(Metric { name: "swap_used".to_string(), value: swap_used, tags: vec![] });
        metrics.push(Metric { name: "swap_free".to_string(), value: swap_free, tags: vec![] });
        metrics.push(Metric { name: "swap_cached".to_string(), value: swap_cached, tags: vec![] });

        let active = meminfo.get("Active").cloned().unwrap_or(0) as f64 * 1024.0;
        let inactive = meminfo.get("Inactive").cloned().unwrap_or(0) as f64 * 1024.0;
        let dirty = meminfo.get("Dirty").cloned().unwrap_or(0) as f64 * 1024.0;
        let shmem = meminfo.get("Shmem").cloned().unwrap_or(0) as f64 * 1024.0;
        let slab = meminfo.get("Slab").cloned().unwrap_or(0) as f64 * 1024.0;
        let pagetables = meminfo.get("PageTables").cloned().unwrap_or(0) as f64 * 1024.0;
        let zswap = meminfo.get("Zswap").cloned().unwrap_or(0) as f64 * 1024.0;
        let zswapped = meminfo.get("Zswapped").cloned().unwrap_or(0) as f64 * 1024.0;

        metrics.push(Metric { name: "memory_active".to_string(), value: active, tags: vec![] });
        metrics.push(Metric { name: "memory_inactive".to_string(), value: inactive, tags: vec![] });
        metrics.push(Metric { name: "memory_dirty".to_string(), value: dirty, tags: vec![] });
        metrics.push(Metric { name: "memory_shmem".to_string(), value: shmem, tags: vec![] });
        metrics.push(Metric { name: "memory_slab".to_string(), value: slab, tags: vec![] });
        metrics.push(Metric { name: "memory_pagetables".to_string(), value: pagetables, tags: vec![] });
        metrics.push(Metric { name: "zswap_size".to_string(), value: zswap, tags: vec![] });
        metrics.push(Metric { name: "zswap_stored".to_string(), value: zswapped, tags: vec![] });

        metrics
    }
}

#[async_trait]
impl Collector for LinuxMemoryCollector {
    fn name(&self) -> &str {
        "memory"
    }

    async fn collect(&mut self) -> Vec<Metric> {
        if let Ok(file) = fs::File::open("/proc/meminfo") {
            let reader = BufReader::new(file);
            if let Ok(meminfo) = self.parse_meminfo(reader) {
                return self.build_metrics(&meminfo);
            }
        }
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    const REAL_MEMINFO_DATA: &str = r#"MemTotal:       32499764 kB
MemFree:        21048968 kB
MemAvailable:   27735004 kB
Buffers:            2672 kB
Cached:          6205420 kB
SwapCached:            0 kB
Active:          6685232 kB
Inactive:        3552724 kB
Active(anon):    3266624 kB
Inactive(anon):        0 kB
Active(file):    3418608 kB
Inactive(file):  3552724 kB
Unevictable:          64 kB
Mlocked:              64 kB
SwapTotal:      25165820 kB
SwapFree:       25165820 kB
Zswap:                 0 kB
Zswapped:              0 kB
Dirty:              2208 kB
Writeback:             0 kB
AnonPages:       4001980 kB
Mapped:          1012428 kB
Shmem:             86232 kB
KReclaimable:     166152 kB
Slab:             406816 kB
SReclaimable:     166152 kB
SUnreclaim:       240664 kB
KernelStack:       22832 kB
PageTables:        56652 kB
SecPageTables:      4108 kB
NFS_Unstable:          0 kB
Bounce:                0 kB
WritebackTmp:          0 kB
CommitLimit:    41415700 kB
Committed_AS:   11450308 kB
VmallocTotal:   34359738367 kB
VmallocUsed:       93236 kB
VmallocChunk:          0 kB
Percpu:            38656 kB
HardwareCorrupted:     0 kB
AnonHugePages:    352256 kB
ShmemHugePages:        0 kB
ShmemPmdMapped:        0 kB
FileHugePages:    544768 kB
FilePmdMapped:    335872 kB
CmaTotal:              0 kB
CmaFree:               0 kB
Unaccepted:            0 kB
Balloon:               0 kB
HugePages_Total:       0
HugePages_Free:        0
HugePages_Rsvd:        0
HugePages_Surp:        0
Hugepagesize:       2048 kB
Hugetlb:               0 kB
DirectMap4k:      348700 kB
DirectMap2M:    12929024 kB
DirectMap1G:    19922944 kB
"#;

    #[test]
    fn test_parse_meminfo_with_real_data() {
        let collector = LinuxMemoryCollector::new();
        let reader = BufReader::new(Cursor::new(REAL_MEMINFO_DATA));
        let meminfo = collector.parse_meminfo(reader).unwrap();

        assert_eq!(meminfo.get("MemTotal"), Some(&32499764));
        assert_eq!(meminfo.get("MemFree"), Some(&21048968));
        assert_eq!(meminfo.get("MemAvailable"), Some(&27735004));
        assert_eq!(meminfo.get("Buffers"), Some(&2672));
        assert_eq!(meminfo.get("Cached"), Some(&6205420));
        assert_eq!(meminfo.get("SwapTotal"), Some(&25165820));
        assert_eq!(meminfo.get("SwapFree"), Some(&25165820));
        assert_eq!(meminfo.get("SwapCached"), Some(&0));
        assert_eq!(meminfo.get("Active"), Some(&6685232));
        assert_eq!(meminfo.get("Inactive"), Some(&3552724));
        assert_eq!(meminfo.get("Dirty"), Some(&2208));
        assert_eq!(meminfo.get("Shmem"), Some(&86232));
        assert_eq!(meminfo.get("Slab"), Some(&406816));
        assert_eq!(meminfo.get("PageTables"), Some(&56652));
        assert_eq!(meminfo.get("Zswap"), Some(&0));
        assert_eq!(meminfo.get("Zswapped"), Some(&0));
    }

    #[test]
    fn test_build_metrics_with_real_data() {
        let collector = LinuxMemoryCollector::new();
        let reader = BufReader::new(Cursor::new(REAL_MEMINFO_DATA));
        let meminfo = collector.parse_meminfo(reader).unwrap();
        let metrics = collector.build_metrics(&meminfo);

        assert_eq!(metrics.len(), 18);

        let memory_total = metrics.iter().find(|m| m.name == "memory_total").unwrap();
        assert_eq!(memory_total.value, 32499764.0 * 1024.0);

        let memory_used = metrics.iter().find(|m| m.name == "memory_used").unwrap();
        assert_eq!(memory_used.value, (32499764.0 - 21048968.0) * 1024.0);
        
        let memory_available = metrics.iter().find(|m| m.name == "memory_available").unwrap();
        assert_eq!(memory_available.value, 27735004.0 * 1024.0);

        let memory_buffered = metrics.iter().find(|m| m.name == "memory_buffered").unwrap();
        assert_eq!(memory_buffered.value, 2672.0 * 1024.0);

        let memory_cached = metrics.iter().find(|m| m.name == "memory_cached").unwrap();
        assert_eq!(memory_cached.value, 6205420.0 * 1024.0);

        let swap_total = metrics.iter().find(|m| m.name == "swap_total").unwrap();
        assert_eq!(swap_total.value, 25165820.0 * 1024.0);

        let swap_used = metrics.iter().find(|m| m.name == "swap_used").unwrap();
        assert_eq!(swap_used.value, (25165820.0 - 25165820.0) * 1024.0);

        let swap_cached = metrics.iter().find(|m| m.name == "swap_cached").unwrap();
        assert_eq!(swap_cached.value, 0.0);

        let memory_active = metrics.iter().find(|m| m.name == "memory_active").unwrap();
        assert_eq!(memory_active.value, 6685232.0 * 1024.0);

        let memory_inactive = metrics.iter().find(|m| m.name == "memory_inactive").unwrap();
        assert_eq!(memory_inactive.value, 3552724.0 * 1024.0);

        let memory_dirty = metrics.iter().find(|m| m.name == "memory_dirty").unwrap();
        assert_eq!(memory_dirty.value, 2208.0 * 1024.0);

        let memory_shmem = metrics.iter().find(|m| m.name == "memory_shmem").unwrap();
        assert_eq!(memory_shmem.value, 86232.0 * 1024.0);

        let memory_slab = metrics.iter().find(|m| m.name == "memory_slab").unwrap();
        assert_eq!(memory_slab.value, 406816.0 * 1024.0);

        let memory_pagetables = metrics.iter().find(|m| m.name == "memory_pagetables").unwrap();
        assert_eq!(memory_pagetables.value, 56652.0 * 1024.0);

        let zswap = metrics.iter().find(|m| m.name == "zswap_size").unwrap();
        assert_eq!(zswap.value, 0.0 * 1024.0);

        let zswapped = metrics.iter().find(|m| m.name == "zswap_stored").unwrap();
        assert_eq!(zswapped.value, 0.0 * 1024.0);
    }
}