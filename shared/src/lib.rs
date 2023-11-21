pub mod auth;
pub mod client;
pub mod error;
pub mod model;
pub mod schema;
pub mod startup;
pub mod telemetry;
pub mod tracing;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
