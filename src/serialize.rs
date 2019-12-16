use crate::log::{MetricContext, MetricValues, Unit};
use serde::Serialize as SerdeSerialize;
use serde_json::Value;
use std::collections::BTreeMap;

// https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/CloudWatch_Embedded_Metric_Format_Specification.html?shortFooter=true

const MAX_DIMENSIONS: usize = 9;

#[derive(SerdeSerialize)]
struct Metric {
    name: String,
    unit: Unit,
}

#[derive(SerdeSerialize)]
struct MetricDefinition {
    namespace: String,
    dimensions: Vec<Vec<String>>,
    metrics: Vec<Metric>,
}

#[derive(SerdeSerialize)]
struct Metadata {
    cloud_watch_metrics: Vec<MetricDefinition>,
    #[serde(flatten)]
    meta: BTreeMap<String, Value>,
}

#[derive(SerdeSerialize)]
struct Payload {
    _aws: Metadata,
    #[serde(flatten)]
    target_values: BTreeMap<String, Value>,
}

pub trait Serialize {
    fn serialize(
        &self,
        context: MetricContext,
    ) -> String;
}

pub struct Log;

impl Serialize for Log {
    fn serialize(
        &self,
        context: MetricContext,
    ) -> String {
        let MetricContext {
            namespace,
            meta,
            mut properties,
            dimensions,
            metrics,
        } = context;

        let (dimensions, mut target_values) = dimensions.into_iter().fold(
            (Vec::new(), BTreeMap::new()),
            |(mut keys, mut dims), dim| {
                dims.append(
                    &mut dim
                        .iter()
                        .map(|(key, value)| (key.clone(), Value::from(value.clone())))
                        .collect(),
                );
                keys.push(
                    dim.keys()
                        .take(MAX_DIMENSIONS)
                        .cloned()
                        .collect::<Vec<String>>(),
                );
                (keys, dims)
            },
        );
        target_values.append(&mut properties);

        let payload = metrics.into_iter().fold(
            Payload {
                _aws: Metadata {
                    meta,
                    cloud_watch_metrics: vec![MetricDefinition {
                        namespace,
                        dimensions,
                        metrics: Vec::new(),
                    }],
                },
                target_values,
            },
            |mut payload, (name, metric)| {
                let MetricValues { values, unit } = metric;
                let val: Value = if values.len() == 1 {
                    values[0].into()
                } else {
                    values.into()
                };
                payload.target_values.insert(name.clone(), val);
                payload._aws.cloud_watch_metrics[0]
                    .metrics
                    .push(Metric { name, unit });
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
