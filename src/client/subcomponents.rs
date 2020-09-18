//! Sub-components of response messages.

//Whether a float is a ZecZmount is not inferrable. This type alias will likely
//be phased out.
//use crate::ZecAmount;

//Haven't implmented this yet in macro/build, as z_ calls need additional
//handling to convert between CamelCase and lowercase
pub type ZGetNewAddressResponse = String;

//Haven't implemented this yet in macro/build, as generate needs an argument
//in order to give a coherent response type, which needs additional handling
//in build script
pub type GenerateResponse = Vec<String>;

zcashrpc_macros::declare_rpc_response_types!();
