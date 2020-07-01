pub mod client; // TODO: Remove this level of hierarchy lifting here.
mod json;

pub use self::client::{Client, Error};
