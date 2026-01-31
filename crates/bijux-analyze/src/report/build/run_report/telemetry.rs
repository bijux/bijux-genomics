use std::fs;
use std::path::Path;

use bijux_core::TelemetryEventV1;

pub(super) fn telemetry_path_from_stage_report(path: Option<&str>) -> Option<String> {
    path.and_then(|path| {
        Path::new(path).parent().map(|parent| {
            parent
                .join("telemetry")
                .join("events.jsonl")
                .display()
                .to_string()
        })
    })
}

pub(super) fn telemetry_counts(paths: &[String]) -> (usize, usize) {
    let mut total_events = 0usize;
    let mut error_events = 0usize;
    for path in paths {
        let Ok(raw) = fs::read_to_string(path) else {
            continue;
        };
        for line in raw.lines() {
            if line.trim().is_empty() {
                continue;
            }
            total_events += 1;
            if let Ok(event) = serde_json::from_str::<TelemetryEventV1>(line) {
                if event.event_name == "error" || event.status == "error" {
                    error_events += 1;
                }
            }
        }
    }
    (total_events, error_events)
}
