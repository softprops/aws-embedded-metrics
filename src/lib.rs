mod log;
pub use log::{metric_scope, MetricLogger, Unit};
mod serialize;
//pub use serialize::*;
mod env;
//pub use env::*;
mod config;
//pub use config::*;
// re-export for easy of contructing btreemaps
pub use maplit::btreemap as dimensions;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
