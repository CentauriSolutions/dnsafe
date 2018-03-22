#[macro_use] extern crate nom;

mod header;
mod result_code;

pub use header::DnsHeader;
pub use result_code::ResultCode;