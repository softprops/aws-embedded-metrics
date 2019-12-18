use crate::log::{MetricContext, MetricValues, Unit};
use serde::Serialize as SerdeSerialize;
use serde_json::Value;
use std::collections::BTreeMap;

// https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/CloudWatch_Embedded_Metric_Format_Specification.html?shortFooter=true

/// Each dimension set is capped at maximum of 9 dimension names
const MAX_DIMENSIONS: usize = 9;

#[derive(SerdeSerialize)]
#[serde(rename_all = "PascalCase")]
struct Metric<'a> {
    name: &'a str,
    unit: Unit,
}

#[derive(SerdeSerialize)]
#[serde(rename_all = "PascalCase")]
struct MetricDefinition<'a> {
    namespace: &'a str,
    dimensions: Vec<Vec<&'a str>>,
    metrics: Vec<Metric<'a>>,
}

#[derive(SerdeSerialize)]
#[serde(rename_all = "PascalCase")]
struct Metadata<'a> {
    cloud_watch_metrics: [MetricDefinition<'a>; 1],
    #[serde(flatten)]
    meta: BTreeMap<&'a str, Value>,
}

#[derive(SerdeSerialize)]
struct Payload<'a> {
    _aws: Metadata<'a>,
    #[serde(flatten)]
    target_values: BTreeMap<&'a str, Value>,
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
            properties,
            dimensions,
            metrics,
        } = context;

        let (dimensions, mut target_values) = dimensions.iter().fold(
            (Vec::new(), BTreeMap::new()),
            |(mut keys, mut dims), dim| {
                keys.push(
                    dim.keys()
                        .take(MAX_DIMENSIONS)
                        .map(|s| s.as_str())
                        .collect(),
                );
                dims.append(
                    &mut dim
                        .iter()
                        .map(|(key, value)| (key.as_str(), Value::from(value.clone())))
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
                    cloud_watch_metrics: [MetricDefinition {
                        namespace: namespace.as_str(),
                        dimensions,
                        metrics: Vec::with_capacity(metrics.len()),
                    }],
                },
                target_values,
            },
            move |mut payload, (name, metric)| {
                let MetricValues { values, unit } = metric;
                // if there is only one metric value, unwrap it to make querying easier
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
    use jsonschema_valid::validate;
    use std::error::Error as StdError;

    #[test]
    fn log_serializes_metrics() {
        let mut ctx = MetricContext::default();
        ctx.put_metric("foo", 1, Unit::Bytes);
        println!("{}", Log.serialize(ctx));
    }

    #[test]
    fn log_serializes_valid_payload() -> Result<(), Box<dyn StdError>> {
        let mut ctx = MetricContext::default();
        ctx.put_metric("foo", 1, Unit::Bytes);
        let payload = Log.serialize(ctx);
        let result = validate(
            &serde_json::from_str(&payload)?,
            &serde_json::from_str(include_str!("../data/schema.json"))?,
            None,
            false,
        );
        assert!(
            result.get_errors().is_empty(),
            "payload contained validation errors\n\n{}\n\n{:?}",
            payload,
            result.get_errors()
        );
        Ok(())
    }
}
