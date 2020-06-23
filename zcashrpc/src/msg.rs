#[macro_use]
mod defreqresp;
mod request;

pub use self::request::Request;

define_request_response! {
  getinfo {
      Request: {}
      Response: {
        blocks: u64,
      }
  },

  getblockchaininfo {
      Request: {}
      Response: {}
  }
}
