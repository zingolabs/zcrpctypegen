//! Sub-components of response messages.

use crate::ZecAmount;
use serde::{Deserialize, Serialize};

pub type ZGetNewAddressResponse = String;

pub type GenerateResponse = Vec<String>;

zcashrpc_macros::create_rpc_response_str_literals!();
