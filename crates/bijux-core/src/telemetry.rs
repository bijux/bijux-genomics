use std::collections::BTreeMap;

pub trait TelemetryAdapter: Send + Sync {
    fn start_pipeline(&self, name: &str, attrs: &BTreeMap<String, String>) -> TelemetrySpan;
    fn start_stage(&self, name: &str, attrs: &BTreeMap<String, String>) -> TelemetrySpan;
}

pub enum TelemetrySpan {
    Noop,
    #[cfg(feature = "otel")]
    Otel(opentelemetry::global::BoxedSpan),
}

impl TelemetrySpan {
    pub fn end(self) {
        #[cfg(feature = "otel")]
        if let TelemetrySpan::Otel(mut span) = self {
            use opentelemetry::trace::Span as _;
            span.end();
        }
    }
}

pub struct NoopTelemetryAdapter;

impl TelemetryAdapter for NoopTelemetryAdapter {
    fn start_pipeline(&self, _name: &str, _attrs: &BTreeMap<String, String>) -> TelemetrySpan {
        TelemetrySpan::Noop
    }

    fn start_stage(&self, _name: &str, _attrs: &BTreeMap<String, String>) -> TelemetrySpan {
        TelemetrySpan::Noop
    }
}

#[cfg(feature = "otel")]
pub struct OtelTelemetryAdapter {
    tracer: opentelemetry::global::BoxedTracer,
}

#[cfg(feature = "otel")]
impl OtelTelemetryAdapter {
    pub fn new() -> Self {
        let tracer = opentelemetry::global::tracer("bijux-core");
        Self { tracer }
    }

    fn start_span(&self, name: &str, attrs: &BTreeMap<String, String>) -> TelemetrySpan {
        use opentelemetry::trace::Tracer as _;
        let mut span = self.tracer.start(name.to_string());
        for (key, value) in attrs {
            use opentelemetry::trace::Span as _;
            span.set_attribute(opentelemetry::KeyValue::new(key.clone(), value.clone()));
        }
        TelemetrySpan::Otel(span)
    }
}

#[cfg(feature = "otel")]
impl TelemetryAdapter for OtelTelemetryAdapter {
    fn start_pipeline(&self, name: &str, attrs: &BTreeMap<String, String>) -> TelemetrySpan {
        self.start_span(name, attrs)
    }

    fn start_stage(&self, name: &str, attrs: &BTreeMap<String, String>) -> TelemetrySpan {
        self.start_span(name, attrs)
    }
}

#[must_use]
pub fn build_telemetry_adapter() -> Box<dyn TelemetryAdapter> {
    if std::env::var("BIJUX_OTEL").ok().as_deref() == Some("1") {
        #[cfg(feature = "otel")]
        {
            return Box::new(OtelTelemetryAdapter::new());
        }
    }
    Box::new(NoopTelemetryAdapter)
}
