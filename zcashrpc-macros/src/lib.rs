mod calls;
mod responses;

use proc_macro::TokenStream;

#[proc_macro]
pub fn declare_rpc_client_methods(input: TokenStream) -> TokenStream {
    calls::declare_rpc_client_methods(input.into()).into()
}

#[proc_macro]
pub fn declare_rpc_response_types(_input: TokenStream) -> TokenStream {
    responses::declare_rpc_response_types().into()
}
