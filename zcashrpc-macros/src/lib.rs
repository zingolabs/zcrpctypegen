mod calls;
mod responses;

use proc_macro::TokenStream;

#[proc_macro]
pub fn declare_rpc_client_methods(input: TokenStream) -> TokenStream {
    calls::declare_rpc_client_methods(input.into()).into()
}

#[proc_macro]
pub fn create_rpc_response_str_literals(_input: TokenStream) -> TokenStream {
    responses::create_rpc_response_str_literals().into()
}
