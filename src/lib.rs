mod log;
pub use log::*;
mod serialize;
pub use serialize::*;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
