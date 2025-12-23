pub mod client;
pub mod error;
pub mod http;
pub mod tui;
pub mod variable;

pub use client::Client;
pub use error::{Error, Result};
pub use http::{HttpFile, Parser, Request};
pub use variable::substitute;
