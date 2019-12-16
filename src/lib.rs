//! Provides an interface for recording high cardinality application metrics
//! using AWS CloudWatch embedded metrics
//!
//! # example
//!
//! ```rust,edition2018
//! use aws_embedded_metrics::{metric_scope, Unit, dimensions};
//!
//! # fn main() {
//! metric_scope(|mut metrics| {
//!    metrics.put_dimensions(dimensions! { "Service".into() => "Aggregator".into() });
//!    metrics.put_metric("ProcessingLatency", 100, Unit::Milliseconds);
//!    metrics.set_property("RequestId", "422b1569-16f6-4a03-b8f0-fe3fd9b100f8");
//! });
//! # }
//! ```
mod log;
pub use log::{metric_scope, MetricLogger, Unit};
mod config;
mod env;
mod serialize;
pub use maplit::btreemap as dimensions;
