pub fn make_call(
    input: (proc_macro2::Ident, proc_macro2::Group),
) -> proc_macro2::TokenStream {
    let (call_ident, response_ident, typed_arg_stream, untyped_arg_stream) =
        crate::utils::format_input("Response", input);
    quote::quote!(
        pub fn #call_ident(
            &mut self,
            #typed_arg_stream
        ) -> impl Future<Output = ResponseResult<#response_ident>> {
            rpc_call!(self.#call_ident(#untyped_arg_stream))
        }
    )
    .into()
}
