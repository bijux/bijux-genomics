use std::collections::BTreeMap;
use std::path::Path;

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub(super) struct FastqcMetricsV2 {
    pub(super) schema_version: String,
    pub(super) source: String,
    pub(super) per_base_quality: Option<PerBaseQualitySummary>,
    pub(super) gc_distribution: Option<GcDistributionSummary>,
    pub(super) adapter_content: Option<AdapterContentSummary>,
    pub(super) duplication: Option<DuplicationSummary>,
    pub(super) n_content: Option<NContentSummary>,
    pub(super) kmer_content: Option<KmerContentSummary>,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct PerBaseQualitySummary {
    mean_min: f64,
    mean_max: f64,
    mean_mean: f64,
    bases_below_q20: u64,
    bases_below_q30: u64,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct GcDistributionSummary {
    mean_gc: f64,
    std_gc: f64,
    outlier: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct AdapterContentSummary {
    pub(super) max_percent: f64,
    pub(super) mean_percent: f64,
    adapters: Vec<AdapterSignal>,
}

#[derive(Debug, Clone, Serialize)]
struct AdapterSignal {
    name: String,
    max_percent: f64,
    mean_percent: f64,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct DuplicationSummary {
    unique_fraction: f64,
    pub(super) duplication_rate: f64,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct NContentSummary {
    pub(super) mean_percent: f64,
    max_percent: f64,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct KmerContentSummary {
    pub(super) warning_count: u64,
    kmers: Vec<KmerSignal>,
}

#[derive(Debug, Clone, Serialize)]
struct KmerSignal {
    kmer: String,
    count: u64,
    percent: f64,
}

pub(super) fn fastqc_metrics_v2_from_dir(dir: &Path) -> Option<FastqcMetricsV2> {
    let path = find_fastqc_data(dir)?;
    let raw = std::fs::read_to_string(path).ok()?;
    let modules = parse_fastqc_modules(&raw);

    let per_base_quality = modules
        .get("Per base sequence quality")
        .and_then(|lines| parse_per_base_quality(lines));
    let gc_distribution = modules
        .get("Per sequence GC content")
        .and_then(|lines| parse_gc_distribution(lines));
    let adapter_content = modules
        .get("Adapter Content")
        .and_then(|lines| parse_adapter_content(lines));
    let duplication = modules
        .get("Sequence Duplication Levels")
        .map(|lines| parse_duplication(lines));
    let n_content = modules
        .get("Per base N content")
        .and_then(|lines| parse_n_content(lines));
    let kmer_content = modules
        .get("Kmer Content")
        .map(|lines| parse_kmer_content(lines));

    Some(FastqcMetricsV2 {
        schema_version: "bijux.fastqc_metrics.v2".to_string(),
        source: dir.display().to_string(),
        per_base_quality,
        gc_distribution,
        adapter_content,
        duplication,
        n_content,
        kmer_content,
    })
}

fn find_fastqc_data(dir: &Path) -> Option<std::path::PathBuf> {
    let candidates = [
        dir.join("fastqc_data.txt"),
        dir.join("fastqc_data"),
        dir.join("fastqc_data.txt.gz"),
    ];
    candidates.into_iter().find(|candidate| candidate.exists())
}

fn parse_fastqc_modules(raw: &str) -> BTreeMap<String, Vec<String>> {
    let mut modules: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut current = None;
    for line in raw.lines() {
        if line.starts_with(">>") {
            if line.starts_with(">>END_MODULE") {
                current = None;
            } else {
                let name = line
                    .trim_start_matches(">>")
                    .split('\t')
                    .next()
                    .unwrap_or("");
                if !name.is_empty() {
                    modules.insert(name.to_string(), Vec::new());
                    current = Some(name.to_string());
                }
            }
            continue;
        }
        if let Some(name) = &current {
            modules
                .entry(name.clone())
                .or_default()
                .push(line.to_string());
        }
    }
    modules
}

fn parse_per_base_quality(lines: &[String]) -> Option<PerBaseQualitySummary> {
    let mut means = Vec::new();
    for line in lines {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 2 {
            continue;
        }
        let mean = parts.get(1).and_then(|v| v.parse::<f64>().ok());
        if let Some(mean) = mean {
            means.push(mean);
        }
    }
    if means.is_empty() {
        return None;
    }
    let mean_min = means.iter().copied().fold(f64::INFINITY, f64::min);
    let mean_max = means.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    #[allow(clippy::cast_precision_loss)]
    let mean_mean = means.iter().sum::<f64>() / means.len() as f64;
    let bases_below_q20 = means.iter().filter(|v| **v < 20.0).count() as u64;
    let bases_below_q30 = means.iter().filter(|v| **v < 30.0).count() as u64;
    Some(PerBaseQualitySummary {
        mean_min,
        mean_max,
        mean_mean,
        bases_below_q20,
        bases_below_q30,
    })
}

fn parse_gc_distribution(lines: &[String]) -> Option<GcDistributionSummary> {
    let mut total = 0.0;
    let mut weighted_sum = 0.0;
    let mut counts = Vec::new();
    for line in lines {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 2 {
            continue;
        }
        let gc = parts.first().and_then(|v| v.parse::<f64>().ok());
        let count = parts.get(1).and_then(|v| v.parse::<f64>().ok());
        if let (Some(gc), Some(count)) = (gc, count) {
            total += count;
            weighted_sum += gc * count;
            counts.push(count);
        }
    }
    if total <= 0.0 {
        return None;
    }
    let mean_gc = weighted_sum / total;
    let mut var_sum = 0.0;
    for line in lines {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 2 {
            continue;
        }
        let gc = parts.first().and_then(|v| v.parse::<f64>().ok());
        let count = parts.get(1).and_then(|v| v.parse::<f64>().ok());
        if let (Some(gc), Some(count)) = (gc, count) {
            var_sum += (gc - mean_gc).powi(2) * count;
        }
    }
    let std_gc = (var_sum / total).sqrt();
    #[allow(clippy::cast_precision_loss)]
    let mean_count = counts.iter().sum::<f64>() / counts.len() as f64;
    let mut count_var = 0.0;
    for count in &counts {
        count_var += (count - mean_count).powi(2);
    }
    #[allow(clippy::cast_precision_loss)]
    let count_std = (count_var / counts.len() as f64).sqrt();
    let outlier = counts
        .iter()
        .any(|count| *count > mean_count + (3.0 * count_std));
    Some(GcDistributionSummary {
        mean_gc,
        std_gc,
        outlier,
    })
}

fn parse_adapter_content(lines: &[String]) -> Option<AdapterContentSummary> {
    let mut header: Option<Vec<String>> = None;
    let mut per_adapter: BTreeMap<String, Vec<f64>> = BTreeMap::new();
    for line in lines {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.is_empty() {
            continue;
        }
        if header.is_none() && parts[0].to_lowercase().contains("position") {
            header = Some(parts.iter().map(std::string::ToString::to_string).collect());
            continue;
        }
        let Some(header) = header.as_ref() else {
            continue;
        };
        if parts.len() < header.len() {
            continue;
        }
        for (idx, name) in header.iter().enumerate().skip(1) {
            let value = parts
                .get(idx)
                .and_then(|v| v.parse::<f64>().ok())
                .unwrap_or(0.0);
            per_adapter.entry(name.clone()).or_default().push(value);
        }
    }
    if per_adapter.is_empty() {
        return None;
    }
    let mut adapters = Vec::new();
    let mut max_percent: f64 = 0.0;
    let mut sum = 0.0;
    let mut count = 0.0;
    for (name, values) in &per_adapter {
        if values.is_empty() {
            continue;
        }
        let local_max = values.iter().copied().fold(0.0, f64::max);
        #[allow(clippy::cast_precision_loss)]
        let local_mean = values.iter().sum::<f64>() / values.len() as f64;
        max_percent = max_percent.max(local_max);
        sum += values.iter().sum::<f64>();
        #[allow(clippy::cast_precision_loss)]
        {
            count += values.len() as f64;
        }
        adapters.push(AdapterSignal {
            name: name.clone(),
            max_percent: local_max,
            mean_percent: local_mean,
        });
    }
    let mean_percent = if count > 0.0 { sum / count } else { 0.0 };
    Some(AdapterContentSummary {
        max_percent,
        mean_percent,
        adapters,
    })
}

fn parse_duplication(lines: &[String]) -> DuplicationSummary {
    let mut unique_fraction = None;
    for line in lines {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 2 {
            continue;
        }
        let level = parts.first().and_then(|v| v.parse::<u64>().ok());
        let percent = parts.get(1).and_then(|v| v.parse::<f64>().ok());
        if level == Some(1) {
            unique_fraction = percent.map(|v| v / 100.0);
            break;
        }
    }
    let unique_fraction = unique_fraction.unwrap_or(0.0);
    DuplicationSummary {
        unique_fraction,
        duplication_rate: (1.0 - unique_fraction).max(0.0),
    }
}

fn parse_n_content(lines: &[String]) -> Option<NContentSummary> {
    let mut values = Vec::new();
    for line in lines {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 2 {
            continue;
        }
        if let Some(value) = parts.get(1).and_then(|v| v.parse::<f64>().ok()) {
            values.push(value);
        }
    }
    if values.is_empty() {
        return None;
    }
    #[allow(clippy::cast_precision_loss)]
    let mean_percent = values.iter().sum::<f64>() / values.len() as f64;
    let max_percent = values.iter().copied().fold(0.0, f64::max);
    Some(NContentSummary {
        mean_percent,
        max_percent,
    })
}

fn parse_kmer_content(lines: &[String]) -> KmerContentSummary {
    let mut warning_count = 0;
    let mut kmers = Vec::new();
    for line in lines {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 3 {
            continue;
        }
        let kmer = parts[0].to_string();
        let count = parts[1].parse::<u64>().unwrap_or(0);
        let percent = parts[2].parse::<f64>().unwrap_or(0.0);
        if percent > 0.0 {
            warning_count += 1;
        }
        kmers.push(KmerSignal {
            kmer,
            count,
            percent,
        });
    }
    KmerContentSummary {
        warning_count,
        kmers,
    }
}
