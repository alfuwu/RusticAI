pub mod client;
pub mod methods;
pub mod types;
pub(crate) mod requester;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        client::main();
    }
}
