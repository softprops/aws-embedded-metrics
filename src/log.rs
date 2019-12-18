use crate::{
    dimensions,
    env::{Detector, EnvironmentProvider},
};
use serde::Serialize;
use serde_json::Value;
use std::{collections::HashMap, time::UNIX_EPOCH};

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
///    metrics.put_dimensions(dimensions! {
///         "Service" => "Aggregator"
///    });
///    metrics.put_metric("ProcessingLatency", 100, Unit::Milliseconds);
///    metrics.set_property("RequestId", "422b1569-16f6-4a03-b8f0-fe3fd9b100f8");
/// });
/// # }
/// ```
pub fn metric_scope<T>(mut f: impl FnMut(&mut MetricLogger) -> T) -> T {
    f(&mut MetricLogger::default())
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
pub(crate) struct MetricValues {
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
    pub(crate) meta: HashMap<String, Value>,
    pub(crate) properties: HashMap<String, Value>,
    pub(crate) dimensions: Vec<HashMap<String, String>>,
    pub(crate) metrics: HashMap<String, MetricValues>,
}

impl MetricContext {
    pub fn set_namespace(
        &mut self,
        namespace: impl Into<String>,
    ) {
        self.namespace = namespace.into()
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
        dims: HashMap<String, String>,
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
            meta: dimensions!(
                "Timestamp" => UNIX_EPOCH.elapsed().unwrap_or_default().as_millis() as u64
            ),
            properties: HashMap::default(),
            dimensions: Vec::new(),
            metrics: HashMap::default(),
        }
    }
}

/// Metric logging interface
///
/// By default, metrics will live under a default namespace "aws-embedded-metrics",
/// You may customize this for your application with the `set_namespace` function
pub struct MetricLogger {
    context: MetricContext,
    get_env: Box<dyn EnvironmentProvider>,
}

impl Drop for MetricLogger {
    fn drop(&mut self) {
        self.flush()
    }
}

impl Default for MetricLogger {
    fn default() -> MetricLogger {
        MetricLogger {
            context: MetricContext::default(),
            get_env: Box::new(Detector::default()),
        }
    }
}

impl MetricLogger {
    /// Flushes the current context state to the configured sink.
    ///
    /// Then `MetricLogger` values are dropped, `flush` is called for you
    pub fn flush(&mut self) {
        let _ = self.get_env.get();
        // todo: syncs
        println!("metrics logger was flushed");
    }

    /// Set the CloudWatch namespace that metrics should be published to.
    pub fn set_namespace(
        &mut self,
        ns: impl Into<String>,
    ) {
        self.context.set_namespace(ns);
    }

    /// Set an aribtrary property on the published metrics.
    /// This is stored in the emitted log data and you are not
    /// charged for this data by CloudWatch Metrics.
    ///
    /// These values can be values that are useful for searching on,
    /// but have too high cardinality to emit as dimensions to
    /// CloudWatch Metrics. An example would be a request id
    pub fn set_property(
        &mut self,
        name: impl Into<String>,
        value: impl Into<Value>,
    ) {
        self.context.set_property(name, value);
    }

    /// Adds a dimension.
    /// This is generally a low cardinality key-value pair that is part of the metric identity.
    /// CloudWatch treats each unique combination of dimensions as a separate metric, even if the metrics have the same metric name.
    ///
    /// See [CloudWatch Dimensions](https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/cloudwatch_concepts.html#Dimension) for more information
    pub fn put_dimensions(
        &mut self,
        dims: HashMap<String, String>,
    ) {
        self.context.put_dimensions(dims);
    }

    /// Put a metric value.
    /// This value will be emitted to CloudWatch Metrics asyncronously and does not contribute to your
    /// account TPS limits. The value will also be available in your CloudWatch Logs
    ///
    /// Although the Value parameter accepts floating point numbers,
    /// CloudWatch rejects values that are either too small or too large.
    /// Values must be in the range of -2^360 to 2^360.
    /// In addition, special values (for example, NaN, +Infinity, -Infinity) are not supported.
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
