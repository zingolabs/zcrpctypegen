use proc_macro::TokenStream;

#[proc_macro]
declare_rpc_response_structs(arg: TokenStream) -> TokenStream {
    arg
}
