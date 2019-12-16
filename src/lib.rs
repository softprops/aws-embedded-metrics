mod log;
pub use log::{metric_scope, MetricLogger, Unit};
mod config;
mod env;
mod serialize;
pub use maplit::btreemap as dimensions;
