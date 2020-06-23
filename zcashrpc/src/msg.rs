#[macro_use]
mod defapi;
mod request;

pub use self::request::Request;

// TODO: Replace this with a lossless-precision decimal type:
pub type ZecAmount = f64;

define_api! {
  getinfo {
      Request: {}
      Response: {
        balance: ZecAmount,
        blocks: u64,
        connections: u64,
        difficulty: f64,
        errors: String,
        keypoololdest: u64,
        keypoolsize: u64,
        paytxfee: ZecAmount,
        protocolversion: u64,
        proxy: String,
        relayfee: ZecAmount,
        testnet: bool,
        timeoffset: u64,
        version: u64,
        walletversion: u64,
      }
  },

  getblockchaininfo {
      Request: {}
      Response: {}
  }
}
