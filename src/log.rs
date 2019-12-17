use crate::env::{Detector, EnvironmentProvider};
use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeMap;

const DEFAULT_NAMEPSACE: &str = "aws-embedded-metrics";

/// Central api for logging acquiring metric logger
///
/// You can capture up to 100 metrics at a time
///
/// # example
/// ```rust,edition2018
/// use aws_embedded_metrics::{metric_scope, Unit, dimensions};
///
/// # fn main() {
/// metric_scope(|mut metrics| {
///    metrics.put_dimensions(dimensions! { "Service".into() => "Aggregator".into() });
///    metrics.put_metric("ProcessingLatency", 100, Unit::Milliseconds);
///    metrics.set_property("RequestId", "422b1569-16f6-4a03-b8f0-fe3fd9b100f8");
/// });
/// # }
/// ```
pub fn metric_scope<T>(mut f: impl FnMut(&mut MetricLogger) -> T) -> T {
    f(&mut MetricLogger::create())
}

/// Metric unit types
#[derive(Serialize, Debug, Copy, Clone)]
pub enum Unit {
    Seconds,
    Microseconds,
    Milliseconds,
    Bytes,
    Kilobytes,
    Megabytes,
    Gigabytes,
    Terabytes,
    Bits,
    Kilobits,
    Megabits,
    Gigabits,
    Terabits,
    Percent,
    Count,
    #[serde(rename = "Bytes/Second")]
    BytesPerSecond,
    #[serde(rename = "Kilobytes/Second")]
    KilobytesPerSecond,
    #[serde(rename = "Megabytes/Second")]
    MegabytesPerSecond,
    #[serde(rename = "Gigabytes/Second")]
    GigabytesPerSecond,
    #[serde(rename = "Terabytes/Second")]
    TerabytesPerSecond,
    #[serde(rename = "Bits/Second")]
    BitsPerSecond,
    #[serde(rename = "Kilobits/Second")]
    KilobitsPerSecond,
    #[serde(rename = "Megabits/Second")]
    MegabitsPerSecond,
    #[serde(rename = "Gigabits/Second")]
    GigabitsPerSecond,
    #[serde(rename = "Terabits/Second")]
    TerabitsPerSecond,
    #[serde(rename = "Count/Second")]
    CountPerSecond,
    None,
}

impl Default for Unit {
    fn default() -> Unit {
        Unit::None
    }
}

#[derive(Debug)]
pub struct MetricValues {
    pub(crate) values: Vec<f64>,
    pub(crate) unit: Unit,
}

impl MetricValues {
    pub fn add(
        &mut self,
        value: f64,
    ) {
        self.values.push(value)
    }
}

#[derive(Debug)]
pub struct MetricContext {
    pub(crate) namespace: String,
    pub(crate) meta: BTreeMap<String, Value>,
    pub(crate) properties: BTreeMap<String, Value>,
    pub(crate) dimensions: Vec<BTreeMap<String, String>>,
    pub(crate) metrics: BTreeMap<String, MetricValues>,
}

impl MetricContext {
    pub fn set_namespace(
        &mut self,
        ns: impl Into<String>,
    ) {
        self.namespace = ns.into()
    }

    pub fn set_property(
        &mut self,
        name: impl Into<String>,
        value: impl Into<Value>,
    ) {
        self.properties.insert(name.into(), value.into());
    }

    pub fn put_dimensions(
        &mut self,
        dims: BTreeMap<String, String>,
    ) {
        self.dimensions.push(dims);
    }

    pub fn put_metric(
        &mut self,
        name: impl Into<String>,
        value: impl Into<f64>,
        unit: Unit,
    ) {
        self.metrics
            .entry(name.into())
            .or_insert_with(|| MetricValues {
                values: Vec::new(),
                unit,
            })
            .add(value.into());
    }
}

impl Default for MetricContext {
    fn default() -> MetricContext {
        MetricContext {
            namespace: DEFAULT_NAMEPSACE.into(),
            meta: BTreeMap::default(),
            properties: BTreeMap::default(),
            dimensions: Vec::new(),
            metrics: BTreeMap::default(),
        }
    }
}

/// Logging interface
pub struct MetricLogger {
    context: MetricContext,
    get_env: Box<dyn EnvironmentProvider>,
}

impl Drop for MetricLogger {
    fn drop(&mut self) {
        self.flush()
    }
}

impl MetricLogger {
    /// Create a new `MetricLogger` instance
    pub fn create() -> MetricLogger {
        MetricLogger {
            context: MetricContext::default(),
            get_env: Box::new(Detector::default()),
        }
    }

    pub fn flush(&mut self) {
        let _ = self.get_env.get();
        // todo: syncs
        println!("metrics logger was flushed");
    }

    /// Override default metric namespace
    pub fn set_namespace(
        &mut self,
        ns: impl Into<String>,
    ) {
        self.context.set_namespace(ns);
    }

    /// Set a request property to the metric event
    ///
    /// This will be logged for querying in CloudWatch insights
    /// but not part of a metric dimension in CloudWatch metrics
    pub fn set_property(
        &mut self,
        name: impl Into<String>,
        value: impl Into<Value>,
    ) {
        self.context.set_property(name, value);
    }

    /// Config a set of metrics dimensions
    pub fn put_dimensions(
        &mut self,
        dims: BTreeMap<String, String>,
    ) {
        self.context.put_dimensions(dims);
    }

    /// Add a metric value to the metric event
    ///
    /// You can add up to 100 metrics in a single log event
    pub fn put_metric(
        &mut self,
        name: impl Into<String>,
        value: impl Into<f64>,
        unit: Unit,
    ) {
        self.context.put_metric(name, value, unit);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metric_scope_api() {
        assert_eq!(
            metric_scope(|metrics: &mut MetricLogger| {
                metrics.put_metric("foo", 1, Unit::Count);
                1
            }),
            1
        )
    }

    #[test]
    fn default_namepace() {
        assert_eq!(MetricContext::default().namespace, DEFAULT_NAMEPSACE)
    }

    #[test]
    fn unit_serializes() {
        for (unit, expected) in &[
            (Unit::Seconds, "Seconds"),
            (Unit::Microseconds, "Microseconds"),
            (Unit::Milliseconds, "Milliseconds"),
            (Unit::Bytes, "Bytes"),
            (Unit::Kilobytes, "Kilobytes"),
            (Unit::Gigabytes, "Gigabytes"),
            (Unit::Terabytes, "Terabytes"),
            (Unit::Bits, "Bits"),
            (Unit::Kilobits, "Kilobits"),
            (Unit::Megabits, "Megabits"),
            (Unit::Gigabits, "Gigabits"),
            (Unit::Terabits, "Terabits"),
            (Unit::Percent, "Percent"),
            (Unit::Count, "Count"),
            (Unit::BytesPerSecond, "Bytes/Second"),
            (Unit::KilobytesPerSecond, "Kilobytes/Second"),
            (Unit::MegabytesPerSecond, "Megabytes/Second"),
            (Unit::GigabytesPerSecond, "Gigabytes/Second"),
            (Unit::TerabytesPerSecond, "Terabytes/Second"),
            (Unit::BitsPerSecond, "Bits/Second"),
            (Unit::KilobitsPerSecond, "Kilobits/Second"),
            (Unit::MegabitsPerSecond, "Megabits/Second"),
            (Unit::GigabitsPerSecond, "Gigabits/Second"),
            (Unit::TerabitsPerSecond, "Terabits/Second"),
            (Unit::CountPerSecond, "Count/Second"),
            (Unit::None, "None"),
        ] {
            assert_eq!(
                serde_json::to_string(&unit).unwrap(),
                format!("\"{}\"", expected)
            );
        }
    }
}
