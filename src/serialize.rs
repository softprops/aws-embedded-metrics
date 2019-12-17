use crate::log::{MetricContext, MetricValues, Unit};
use serde::Serialize as SerdeSerialize;
use serde_json::Value;
use std::collections::BTreeMap;

// https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/CloudWatch_Embedded_Metric_Format_Specification.html?shortFooter=true

const MAX_DIMENSIONS: usize = 9;

#[derive(SerdeSerialize)]
struct Metric<'a> {
    name: &'a str,
    unit: Unit,
}

#[derive(SerdeSerialize)]
struct MetricDefinition<'a> {
    namespace: &'a str,
    dimensions: Vec<Vec<&'a str>>,
    metrics: Vec<Metric<'a>>,
}

#[derive(SerdeSerialize)]
struct Metadata<'a> {
    cloud_watch_metrics: Vec<MetricDefinition<'a>>,
    #[serde(flatten)]
    meta: BTreeMap<&'a str, Value>,
}

#[derive(SerdeSerialize)]
struct Payload<'a> {
    _aws: Metadata<'a>,
    #[serde(flatten)]
    target_values: BTreeMap<&'a str, Value>,
}

pub(crate) trait Serialize {
    fn serialize(
        &self,
        context: MetricContext,
    ) -> String;
}

pub(crate) struct Log;

impl Serialize for Log {
    fn serialize(
        &self,
        context: MetricContext,
    ) -> String {
        let MetricContext {
            namespace,
            meta,
            properties,
            dimensions,
            metrics,
        } = context;

        let (dimensions, mut target_values) = dimensions.iter().fold(
            (Vec::new(), BTreeMap::new()),
            |(mut keys, mut dims), dim| {
                dims.append(
                    &mut dim
                        .iter()
                        .map(|(key, value)| (key.as_str(), Value::from(value.clone())))
                        .collect(),
                );
                keys.push(
                    dim.keys()
                        .take(MAX_DIMENSIONS)
                        .map(|s| s.as_str())
                        .collect(),
                );
                (keys, dims)
            },
        );
        target_values.append(
            &mut properties
                .iter()
                .map(|(k, v)| (k.as_str(), v.to_owned()))
                .collect(),
        );

        let payload = metrics.iter().fold(
            Payload {
                _aws: Metadata {
                    meta: meta
                        .iter()
                        .map(|(k, v)| (k.as_str(), v.to_owned()))
                        .collect(),
                    cloud_watch_metrics: vec![MetricDefinition {
                        namespace: namespace.as_str(),
                        dimensions,
                        metrics: Vec::new(),
                    }],
                },
                target_values,
            },
            move |mut payload, (name, metric)| {
                let MetricValues { values, unit } = metric;
                let val: Value = if values.len() == 1 {
                    values[0].into()
                } else {
                    values.to_owned().into()
                };
                payload.target_values.insert(name, val);
                payload._aws.cloud_watch_metrics[0]
                    .metrics
                    .push(Metric { name, unit: *unit });
                payload
            },
        );
        serde_json::to_string(&payload).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log_serializes_metrics() {
        let mut ctx = MetricContext::default();
        ctx.put_metric("foo", 1, Unit::Bytes);
        println!("{}", Log.serialize(ctx));
    }
}
