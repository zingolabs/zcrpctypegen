#[macro_use]
mod defreqresp;
mod request;

pub use request::Request;

define_request_response! {
  getinfo {
      Request: {},
      Response: {}
  },

  getblockchaininfo {
      Request: {},
      Response: {}
  }
}
