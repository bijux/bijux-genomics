#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricClass {
    Integrity,
    Retention,
    QualityShift,
    Contamination,
    Composition,
}
