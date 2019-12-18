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
//!    metrics.put_dimensions(dimensions! {
//!        "Service" => "Aggregator"
//!    });
//!    metrics.put_metric("ProcessingLatency", 100, Unit::Milliseconds);
//!    metrics.set_property("RequestId", "422b1569-16f6-4a03-b8f0-fe3fd9b100f8");
//! });
//! # }
// only pub for benches
#[doc(hidden)]
pub mod log;
pub use log::{metric_scope, MetricLogger, Unit};
mod config;
mod env;
// only pub for benches
#[doc(hidden)]
pub mod serialize;
mod sink;

#[macro_export]
macro_rules! dimensions {
    (@single $($x:tt)*) => (());
    (@count $($rest:expr),*) => (<[()]>::len(&[$(dimensions!(@single $rest)),*]));
    ($($key:expr => $value:expr,)+) => { dimentions!($($key => $value),+) };
    ($($key:expr => $value:expr),*) => {
        {
            let _cap = dimensions!(@count $($key),*);
            let mut _map = ::std::collections::HashMap::with_capacity(_cap);
            $(
                let _ = _map.insert($key.into(), $value.into());
            )*
            _map
        }
    };
}
